use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use rag_core::{
    AgentPipeline, HnswClient, HnswConfig, IndexProgress, OllamaClient, RagPipeline, VectorDatabase,
};

use crate::state::{Command, Event, IndexStats, SourceInfo};

/// デフォルトLLMモデル
const DEFAULT_LLM_MODEL: &str = "gemma3:4b";
/// デフォルト埋め込みモデル
const DEFAULT_EMBEDDING_MODEL: &str = "nomic-embed-text";
/// HNSWコレクション名
const HNSW_COLLECTION_NAME: &str = "localrag_collection";
/// 埋め込み次元数
const EMBEDDING_DIMENSION: usize = 768;

/// バックエンド処理
pub struct Backend {
    event_tx: mpsc::Sender<Event>,
    pipeline: Arc<Mutex<Option<RagPipeline<HnswClient>>>>,
    agent_pipeline: Arc<Mutex<Option<AgentPipeline<HnswClient>>>>,
    ollama_client: Arc<OllamaClient>,
    folder_path: Arc<Mutex<Option<PathBuf>>>,
    cancel_token: Arc<Mutex<Option<CancellationToken>>>,
}

impl Backend {
    pub fn new(event_tx: mpsc::Sender<Event>) -> Self {
        Self {
            event_tx,
            pipeline: Arc::new(Mutex::new(None)),
            agent_pipeline: Arc::new(Mutex::new(None)),
            ollama_client: Arc::new(OllamaClient::default()),
            folder_path: Arc::new(Mutex::new(None)),
            cancel_token: Arc::new(Mutex::new(None)),
        }
    }

    /// コマンドループ実行
    pub async fn run(&self, mut command_rx: mpsc::Receiver<Command>) {
        // Ollamaステータスチェック
        self.check_ollama_status().await;

        // GUIの初期化を待つ（少し待機）
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // 起動時にLanceDBから既存インデックスを読み込む
        self.load_existing_index().await;

        while let Some(cmd) = command_rx.recv().await {
            match cmd {
                Command::SelectFolder(path) => {
                    *self.folder_path.lock().await = Some(path.clone());
                    let _ = self.event_tx.send(Event::FolderSelected(path)).await;
                }
                Command::StartIndexing => {
                    self.start_indexing().await;
                }
                Command::StopIndexing => {
                    self.stop_indexing().await;
                }
                Command::SendQuery(question) => {
                    self.send_query(question, false).await;
                }
                Command::SendAgentQuery(question) => {
                    self.send_query(question, true).await;
                }
                Command::RefreshModels => {
                    self.refresh_models().await;
                }
                Command::SetLlmModel(model) => {
                    // パイプライン再作成
                    self.recreate_pipeline(Some(model), None).await;
                }
                Command::SetEmbeddingModel(model) => {
                    // パイプライン再作成
                    self.recreate_pipeline(None, Some(model)).await;
                }
            }
        }
    }

    /// Ollamaステータスチェック
    async fn check_ollama_status(&self) {
        let running = self.ollama_client.check_running().await;
        let _ = self.event_tx.send(Event::OllamaStatus(running)).await;
    }

    /// モデル一覧を更新
    async fn refresh_models(&self) {
        match self.ollama_client.list_models().await {
            Ok(models) => {
                let llm_models: Vec<String> = models
                    .iter()
                    .filter(|m| !m.name.contains("embed"))
                    .map(|m| m.name.clone())
                    .collect();

                let embedding_models: Vec<String> = models
                    .iter()
                    .filter(|m| m.name.contains("embed"))
                    .map(|m| m.name.clone())
                    .collect();

                let _ = self
                    .event_tx
                    .send(Event::ModelsRefreshed {
                        llm_models,
                        embedding_models,
                    })
                    .await;
            }
            Err(e) => {
                let _ = self
                    .event_tx
                    .send(Event::Error(format!("Failed to list models: {}", e)))
                    .await;
            }
        }
    }

    /// パイプラインを再作成
    async fn recreate_pipeline(&self, llm_model: Option<String>, embedding_model: Option<String>) {
        // 既存のパイプラインから設定を取得または新規作成
        let (current_llm, current_embed) = {
            let pipeline = self.pipeline.lock().await;
            if let Some(p) = &*pipeline {
                (p.llm_model.clone(), p.embedding_model.clone())
            } else {
                (DEFAULT_LLM_MODEL.to_string(), DEFAULT_EMBEDDING_MODEL.to_string())
            }
        };

        let llm = llm_model.unwrap_or(current_llm);
        let embed = embedding_model.unwrap_or(current_embed);

        // HNSWクライアントを作成
        let config = HnswConfig::new(HNSW_COLLECTION_NAME, EMBEDDING_DIMENSION);
        let vector_db = HnswClient::new(config);

        let new_pipeline = RagPipeline::new(
            OllamaClient::default(),
            vector_db.clone(),
            embed.clone(),
            llm.clone(),
        );

        let new_agent_pipeline = AgentPipeline::new(new_pipeline.clone());

        *self.pipeline.lock().await = Some(new_pipeline);
        *self.agent_pipeline.lock().await = Some(new_agent_pipeline);
    }

    /// 起動時に既存インデックスを読み込む
    async fn load_existing_index(&self) {
        // 読み込み開始を通知
        tracing::info!("Sending index loading message to GUI");
        let result = self
            .event_tx
            .send(Event::IndexProgress {
                progress: 0.0,
                file: "既存インデックスを読み込み中...".to_string(),
            })
            .await;

        if let Err(e) = result {
            tracing::error!("Failed to send loading message: {}", e);
        }

        // HNSWクライアントを直接作成（フォルダ不要）
        tracing::info!("Creating HNSW client for index check...");
        let config = HnswConfig::new(HNSW_COLLECTION_NAME, EMBEDDING_DIMENSION);
        let vector_db = HnswClient::new(config);

        // 一時的なパイプラインを作成（統計取得のみに使用）
        let temp_pipeline = RagPipeline::new(
            OllamaClient::default(),
            vector_db.clone(),
            DEFAULT_EMBEDDING_MODEL.to_string(),
            DEFAULT_LLM_MODEL.to_string(),
        );

        // HNSWから統計情報を取得
        let event_tx = self.event_tx.clone();
        tokio::spawn(async move {
            tracing::info!("Checking existing index...");
            match temp_pipeline.count().await {
                Ok(count) if count > 0 => {
                    tracing::info!("Found existing index with {} documents", count);

                    // インデックスされたソースファイルを取得
                    let sources = vector_db.get_indexed_sources().await.unwrap_or_default();
                    let file_count = sources.len();
                    tracing::info!("Found {} unique source files", file_count);

                    // フォルダパスを推定（最も共通する親ディレクトリ）
                    let indexed_folder = derive_common_folder(&sources);
                    tracing::info!("Derived indexed folder: {:?}", indexed_folder);

                    // 既存インデックスの統計情報を送信
                    let _ = event_tx
                        .send(Event::IndexComplete {
                            stats: IndexStats {
                                total_files: file_count,
                                indexed_files: file_count,
                                skipped_files: 0,
                                total_chunks: count,
                                total_embeddings: count,
                                indexed_folder,
                            },
                        })
                        .await;
                }
                Ok(count) => {
                    tracing::info!("No existing documents in HNSW index (count: {})", count);
                    // 0件でも統計情報を送信（進捗表示をクリア）
                    let _ = event_tx
                        .send(Event::IndexComplete {
                            stats: IndexStats::default(),
                        })
                        .await;
                }
                Err(e) => {
                    tracing::warn!("Failed to check existing index: {}", e);
                    // エラー時も統計をクリア
                    let _ = event_tx
                        .send(Event::IndexComplete {
                            stats: IndexStats::default(),
                        })
                        .await;
                }
            }
        });
    }

    /// インデックス作成停止
    async fn stop_indexing(&self) {
        let mut cancel_lock = self.cancel_token.lock().await;
        if let Some(token) = cancel_lock.take() {
            tracing::info!("Cancelling indexing operation");
            token.cancel();
            let _ = self.event_tx.send(Event::Error("インデックス作成がキャンセルされました".to_string())).await;
        }
    }

    /// インデックス作成開始
    async fn start_indexing(&self) {
        let folder = {
            let lock = self.folder_path.lock().await;
            lock.clone()
        };

        let Some(folder) = folder else {
            let _ = self
                .event_tx
                .send(Event::Error("No folder selected".to_string()))
                .await;
            return;
        };

        // パイプライン作成（LanceDBは外部サーバー不要）
        self.recreate_pipeline(None, None).await;

        let pipeline = {
            let lock = self.pipeline.lock().await;
            lock.clone()
        };

        let Some(pipeline) = pipeline else {
            return;
        };

        // キャンセルトークンを作成
        let cancel_token = CancellationToken::new();
        *self.cancel_token.lock().await = Some(cancel_token.clone());

        let event_tx = self.event_tx.clone();
        let (progress_tx, mut progress_rx) = mpsc::channel(100);

        // 進捗レポート
        let cancel_token_clone = cancel_token.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel_token_clone.cancelled() => {
                        tracing::info!("Progress reporting cancelled");
                        break;
                    }
                    Some(progress) = progress_rx.recv() => {
                match progress {
                    IndexProgress::Scanning { current, total } => {
                        tracing::debug!("GUI: Scanning progress {}/{}", current, total);
                        let _ = event_tx
                            .send(Event::IndexProgress {
                                progress: current as f32 / total as f32,
                                file: format!("Scanning files: {}/{}", current, total),
                            })
                            .await;
                    }
                    IndexProgress::Loading {
                        current,
                        total,
                        file,
                    } => {
                        tracing::debug!("GUI: Loading progress {}/{}", current, total);
                        let _ = event_tx
                            .send(Event::IndexProgress {
                                progress: current as f32 / total as f32,
                                file: format!("Loading: {}", file.display()),
                            })
                            .await;
                    }
                    IndexProgress::Splitting { current, total } => {
                        tracing::debug!("GUI: Splitting progress {}/{}", current, total);
                        let _ = event_tx
                            .send(Event::IndexProgress {
                                progress: current as f32 / total as f32,
                                file: "Splitting documents...".to_string(),
                            })
                            .await;
                    }
                    IndexProgress::Embedding { current, total } => {
                        tracing::debug!("GUI: Embedding progress {}/{}", current, total);
                        let _ = event_tx
                            .send(Event::IndexProgress {
                                progress: current as f32 / total as f32,
                                file: format!("Generating embeddings: {}/{}", current, total),
                            })
                            .await;
                    }
                    IndexProgress::Storing { current, total } => {
                        tracing::debug!("GUI: Storing progress {}/{}", current, total);
                        let _ = event_tx
                            .send(Event::IndexProgress {
                                progress: current as f32 / total as f32,
                                file: "Storing in database...".to_string(),
                            })
                            .await;
                    }
                    IndexProgress::BatchComplete { stats } => {
                        // バッチ完了時にリアルタイムで統計を更新
                        tracing::info!("GUI: Batch complete - indexed: {}, chunks: {}, embeddings: {}",
                            stats.indexed_files, stats.total_chunks, stats.total_embeddings);
                        let _ = event_tx
                            .send(Event::IndexStatsUpdate {
                                stats: IndexStats {
                                    total_files: stats.total_files,
                                    indexed_files: stats.indexed_files,
                                    skipped_files: stats.skipped_files,
                                    total_chunks: stats.total_chunks,
                                    total_embeddings: stats.total_embeddings,
                                    indexed_folder: None, // インデックス中はフォルダ未確定
                                },
                            })
                            .await;
                    }
                    IndexProgress::Complete { stats: _ } => {
                        // 完了処理はインデックス作成タスク側でDB統計取得後に行う
                        tracing::info!("GUI: IndexProgress::Complete received (stats will be sent after DB query)");
                    }
                }
                    }
                }
            }
        });

        // インデックス作成実行
        let event_tx_complete = self.event_tx.clone();
        let event_tx_error = self.event_tx.clone();
        let folder_path_str = folder.display().to_string();
        tokio::spawn(async move {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    tracing::info!("Indexing cancelled by user");
                }
                result = pipeline.index_directory(&folder, Some(progress_tx)) => {
                    match result {
                        Ok(stats) => {
                            // インデックス完了後、DBから最新の統計を取得
                            tracing::info!("Indexing succeeded, fetching DB stats...");
                            match pipeline.count().await {
                                Ok(db_count) => {
                                    tracing::info!("DB count after indexing: {}", db_count);
                                    let _ = event_tx_complete
                                        .send(Event::IndexComplete {
                                            stats: IndexStats {
                                                total_files: stats.total_files,
                                                indexed_files: stats.indexed_files,
                                                skipped_files: stats.skipped_files,
                                                total_chunks: stats.total_chunks,
                                                total_embeddings: db_count, // DBの実際のカウントを使用
                                                indexed_folder: Some(folder_path_str.clone()),
                                            },
                                        })
                                        .await;
                                }
                                Err(e) => {
                                    tracing::warn!("Failed to get DB count: {}", e);
                                    // フォールバック: 元のstatsを使用
                                    let _ = event_tx_complete
                                        .send(Event::IndexComplete {
                                            stats: IndexStats {
                                                total_files: stats.total_files,
                                                indexed_files: stats.indexed_files,
                                                skipped_files: stats.skipped_files,
                                                total_chunks: stats.total_chunks,
                                                total_embeddings: stats.total_embeddings,
                                                indexed_folder: Some(folder_path_str.clone()),
                                            },
                                        })
                                        .await;
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Indexing error: {}", e);
                            let _ = event_tx_error.send(Event::Error(format!("インデックス作成エラー: {}", e))).await;
                        }
                    }
                }
            }
        });
    }

    /// クエリ送信（インデックス作成と独立して動作）
    async fn send_query(&self, question: String, agent_mode: bool) {
        // クエリ用に独立したパイプラインを作成（インデックス作成と並行動作可能）
        let config = HnswConfig::new(HNSW_COLLECTION_NAME, EMBEDDING_DIMENSION);
        let vector_db = HnswClient::new(config);

        // 既存のパイプラインからモデル設定を取得
        let (llm_model, embedding_model) = {
            let pipeline = self.pipeline.lock().await;
            if let Some(p) = &*pipeline {
                (p.llm_model.clone(), p.embedding_model.clone())
            } else {
                (DEFAULT_LLM_MODEL.to_string(), DEFAULT_EMBEDDING_MODEL.to_string())
            }
        };

        let query_pipeline = RagPipeline::new(
            OllamaClient::default(),
            vector_db,
            embedding_model,
            llm_model,
        );

        // インデックスにドキュメントがあるか確認
        match query_pipeline.count().await {
            Ok(count) if count == 0 => {
                let _ = self
                    .event_tx
                    .send(Event::Error("ドキュメントをインデックスしてください".to_string()))
                    .await;
                return;
            }
            Err(e) => {
                let _ = self
                    .event_tx
                    .send(Event::Error(format!("インデックス確認エラー: {}", e)))
                    .await;
                return;
            }
            _ => {}
        }

        let event_tx = self.event_tx.clone();

        if agent_mode {
            let agent = AgentPipeline::new(query_pipeline);
            tokio::spawn(async move {
                let (progress_tx, mut progress_rx) = mpsc::channel(100);

                // 進捗レポート
                let event_tx_clone = event_tx.clone();
                tokio::spawn(async move {
                    while let Some(progress) = progress_rx.recv().await {
                        let msg = match progress {
                            rag_core::AgentProgress::Analyzing => {
                                "🔍 Analyzing question...".to_string()
                            }
                            rag_core::AgentProgress::Keywords(keywords) => {
                                format!("🔑 Keywords: {}", keywords.join(", "))
                            }
                            rag_core::AgentProgress::Searching(keyword) => {
                                format!("🔍 Searching: {}", keyword)
                            }
                            rag_core::AgentProgress::Found(count) => {
                                format!("📚 Found {} documents", count)
                            }
                            rag_core::AgentProgress::ValidatingSufficiency => {
                                "🔍 Validating sufficiency...".to_string()
                            }
                            rag_core::AgentProgress::Generating => {
                                "✨ Generating answer...".to_string()
                            }
                            rag_core::AgentProgress::Complete => {
                                "✅ Complete".to_string()
                            }
                        };
                        let _ = event_tx_clone.send(Event::AgentProgress(msg)).await;
                    }
                });

                match agent.query_agent(&question, Some(progress_tx)).await {
                    Ok(response) => {
                        // ストリーミング応答処理
                        if let Some(stream) = response.stream {
                            use tokio_stream::StreamExt;
                            let mut stream = stream;
                            while let Some(chunk) = stream.next().await {
                                if let Ok(text) = chunk {
                                    let _ = event_tx.send(Event::QueryChunk(text)).await;
                                }
                            }
                        } else if let Some(answer) = response.answer {
                            let _ = event_tx.send(Event::QueryChunk(answer)).await;
                        }

                        // ソース情報送信
                        let sources: Vec<SourceInfo> = response
                            .sources
                            .into_iter()
                            .map(|s: rag_core::SearchResult| SourceInfo {
                                source: s.source().unwrap_or("Unknown").to_string(),
                                page: s.page(),
                                score: s.score(),
                            })
                            .collect();

                        let _ = event_tx.send(Event::QueryComplete { sources }).await;
                    }
                    Err(e) => {
                        let _ = event_tx
                            .send(Event::Error(format!("Query failed: {}", e)))
                            .await;
                    }
                }
            });
        } else {
            tokio::spawn(async move {
                match query_pipeline.query_stream(&question, 5).await {
                    Ok(response) => {
                        // ストリーミング応答処理
                        if let Some(mut stream) = response.stream {
                            use tokio_stream::StreamExt;
                            while let Some(chunk) = stream.next().await {
                                if let Ok(text) = chunk {
                                    let _ = event_tx.send(Event::QueryChunk(text)).await;
                                }
                            }
                        } else if let Some(answer) = response.answer {
                            let _ = event_tx.send(Event::QueryChunk(answer)).await;
                        }

                        // ソース情報送信
                        let sources: Vec<SourceInfo> = response
                            .sources
                            .into_iter()
                            .map(|s: rag_core::SearchResult| SourceInfo {
                                source: s.source().unwrap_or("Unknown").to_string(),
                                page: s.page(),
                                score: s.score(),
                            })
                            .collect();

                        let _ = event_tx.send(Event::QueryComplete { sources }).await;
                    }
                    Err(e) => {
                        let _ = event_tx
                            .send(Event::Error(format!("Query failed: {}", e)))
                            .await;
                    }
                }
            });
        }
    }
}

/// ソースファイルのリストから共通の親フォルダを推定する
fn derive_common_folder(sources: &[String]) -> Option<String> {
    if sources.is_empty() {
        return None;
    }

    // すべてのソースからパスコンポーネントを取得
    let paths: Vec<PathBuf> = sources.iter().map(PathBuf::from).collect();

    // 最初のファイルの親ディレクトリを起点にする
    let first_parent = paths[0].parent()?;

    // すべてのファイルが共通して持つ最も深いディレクトリを探す
    let mut common = first_parent.to_path_buf();

    for path in &paths[1..] {
        if let Some(parent) = path.parent() {
            // 共通部分を見つける
            let mut new_common = PathBuf::new();
            for (a, b) in common.components().zip(parent.components()) {
                if a == b {
                    new_common.push(a);
                } else {
                    break;
                }
            }
            common = new_common;
        }
    }

    if common.as_os_str().is_empty() {
        None
    } else {
        Some(common.display().to_string())
    }
}
