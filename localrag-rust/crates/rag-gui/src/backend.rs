use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use reqwest::Client;

use rag_core::{
    AgentPipeline, ChromaClient, IndexProgress, OllamaClient, RagPipeline, VectorDatabase,
    VectorDbConfig,
};

use crate::state::{Command, Event, IndexStats, SourceInfo};

/// バックエンド処理
pub struct Backend {
    event_tx: mpsc::Sender<Event>,
    pipeline: Arc<Mutex<Option<RagPipeline<ChromaClient>>>>,
    agent_pipeline: Arc<Mutex<Option<AgentPipeline<ChromaClient>>>>,
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
        let folder = self.folder_path.lock().await.clone();
        if folder.is_none() {
            return;
        }

        // 既存のパイプラインから設定を取得または新規作成
        let (current_llm, current_embed) = {
            let pipeline = self.pipeline.lock().await;
            if let Some(p) = &*pipeline {
                (p.llm_model.clone(), p.embedding_model.clone())
            } else {
                ("gemma2:2b".to_string(), "nomic-embed-text".to_string())
            }
        };

        let llm = llm_model.unwrap_or(current_llm);
        let embed = embedding_model.unwrap_or(current_embed);

        let config = VectorDbConfig::new("localrag_collection", 768);
        let vector_db = ChromaClient::new(config);

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

    /// ChromaDB接続チェック
    async fn check_chromadb(&self) -> bool {
        let client = Client::new();
        match client
            .get("http://localhost:8001/api/v1/heartbeat")  // ポート8001を使用
            .timeout(Duration::from_secs(2))
            .send()
            .await
        {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
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

        // ChromaDB接続チェック
        if !self.check_chromadb().await {
            let error_msg = "ChromaDBブリッジサーバーに接続できません。\n\n以下のスクリプトでChromaDBサーバーを起動してください:\n\n".to_string()
                + "Windowsバッチ:\n"
                + "  start_chromadb.bat\n\n"
                + "PowerShell:\n"
                + "  .\\start_chromadb.ps1\n\n"
                + "サーバーはポート8001で起動します。";

            let _ = self.event_tx.send(Event::Error(error_msg)).await;
            return;
        }

        // パイプライン作成
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
                    IndexProgress::Complete { stats } => {
                        tracing::info!("GUI: Indexing complete!");
                        let _ = event_tx
                            .send(Event::IndexComplete {
                                stats: IndexStats {
                                    total_files: stats.total_files,
                                    indexed_files: stats.indexed_files,
                                    total_chunks: stats.total_chunks,
                                    total_embeddings: stats.total_embeddings,
                                },
                            })
                            .await;
                    }
                }
                    }
                }
            }
        });

        // インデックス作成実行
        let event_tx_error = self.event_tx.clone();
        tokio::spawn(async move {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    tracing::info!("Indexing cancelled by user");
                }
                result = pipeline.index_directory(&folder, Some(progress_tx)) => {
                    if let Err(e) = result {
                        tracing::error!("Indexing error: {}", e);
                        let _ = event_tx_error.send(Event::Error(format!("インデックス作成エラー: {}", e))).await;
                    }
                }
            }
        });
    }

    /// クエリ送信
    async fn send_query(&self, question: String, agent_mode: bool) {
        let pipeline = self.pipeline.lock().await.clone();
        let agent_pipeline = self.agent_pipeline.lock().await.clone();

        if pipeline.is_none() {
            let _ = self
                .event_tx
                .send(Event::Error("Please index documents first".to_string()))
                .await;
            return;
        }

        let event_tx = self.event_tx.clone();

        if agent_mode {
            if let Some(agent) = agent_pipeline {
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
            }
        } else {
            if let Some(pipeline) = pipeline {
                tokio::spawn(async move {
                    match pipeline.query_stream(&question, 5).await {
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
}
