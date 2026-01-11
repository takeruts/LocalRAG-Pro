use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::document::loader::ParallelDocumentLoader;
use crate::error::{RagError, Result};
use crate::ollama::OllamaClient;
use crate::splitter::{RecursiveCharacterTextSplitter, TextSplitter};
use crate::vectordb::{SearchResult, VectorDatabase};

use super::types::{IndexProgress, IndexStats, QueryResponse};

/// パスを正規化（絶対パス化して統一形式に変換）
fn normalize_path(path: &Path) -> String {
    // 絶対パスに変換
    let abs_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    };

    // canonicalizeを試みる（シンボリックリンク解決、パス正規化）
    let normalized = abs_path.canonicalize().unwrap_or(abs_path);

    // Windowsのプレフィックス(\\?\)を削除して表示形式に変換
    let path_str = normalized.display().to_string();

    // Windows拡張パスプレフィックスを削除
    if path_str.starts_with(r"\\?\") {
        path_str[4..].to_string()
    } else {
        path_str
    }
}

/// RAGパイプライン
#[derive(Clone)]
pub struct RagPipeline<D: VectorDatabase> {
    pub(crate) ollama_client: Arc<OllamaClient>,
    pub(crate) vector_db: Arc<D>,
    pub(crate) text_splitter: RecursiveCharacterTextSplitter,
    pub embedding_model: String,
    pub llm_model: String,
}

impl<D: VectorDatabase> RagPipeline<D> {
    /// 新しいパイプラインを作成
    pub fn new(
        ollama_client: OllamaClient,
        vector_db: D,
        embedding_model: impl Into<String>,
        llm_model: impl Into<String>,
    ) -> Self {
        Self {
            ollama_client: Arc::new(ollama_client),
            vector_db: Arc::new(vector_db),
            text_splitter: RecursiveCharacterTextSplitter::new(1000, 100),
            embedding_model: embedding_model.into(),
            llm_model: llm_model.into(),
        }
    }

    /// チャンクサイズとオーバーラップを設定
    pub fn with_splitter(
        mut self,
        chunk_size: usize,
        chunk_overlap: usize,
    ) -> Self {
        self.text_splitter = RecursiveCharacterTextSplitter::new(chunk_size, chunk_overlap);
        self
    }

    /// ベクトルDBのドキュメント数を取得
    pub async fn count(&self) -> Result<usize> {
        self.vector_db.count().await
    }

    /// ディレクトリをインデックス作成
    pub async fn index_directory(
        &self,
        dir: &Path,
        progress_tx: Option<mpsc::Sender<IndexProgress>>,
    ) -> Result<IndexStats> {
        let mut stats = IndexStats::new();

        // 1. コレクション存在確認
        if !self.vector_db.collection_exists().await? {
            tracing::info!("Creating new collection...");
            self.vector_db.create_collection().await?;
        }

        // 2. 既にインデックス済みのソースを取得（差分検出）
        let indexed_sources_raw: Vec<String> = self
            .vector_db
            .get_indexed_sources()
            .await?;

        // インデックス済みソースも正規化してHashSetに変換
        let indexed_sources: HashSet<String> = indexed_sources_raw
            .iter()
            .map(|s| {
                // 既存のパス文字列を正規化（存在しないファイルはそのまま）
                let path = Path::new(s);
                if path.exists() {
                    normalize_path(path)
                } else {
                    s.clone()
                }
            })
            .collect();

        tracing::info!("Found {} already indexed sources", indexed_sources.len());

        // 3. ファイルスキャン - 開始を即座に通知
        if let Some(tx) = &progress_tx {
            let _ = tx.send(IndexProgress::Scanning { current: 0, total: 0 }).await;
        }

        let loader = ParallelDocumentLoader::new();
        let all_files = loader.scan_directory(dir)?;

        // スキャン完了を通知
        if let Some(tx) = &progress_tx {
            let _ = tx.send(IndexProgress::Scanning {
                current: all_files.len(),
                total: all_files.len()
            }).await;
        }

        stats.total_files = all_files.len();

        // 差分検出: 新しいファイルのみ（正規化パスで比較）
        let new_files: Vec<PathBuf> = all_files
            .into_iter()
            .filter(|f| {
                let normalized = normalize_path(f);
                let is_indexed = indexed_sources.contains(&normalized);
                if is_indexed {
                    tracing::debug!("Skipping already indexed file: {}", normalized);
                }
                !is_indexed
            })
            .collect();

        // スキップされたファイル数を記録
        stats.skipped_files = stats.total_files - new_files.len();
        tracing::info!("Skipping {} already indexed files", stats.skipped_files);

        if new_files.is_empty() {
            tracing::info!("All files are already indexed");
            if let Some(tx) = progress_tx {
                let _ = tx.send(IndexProgress::Complete { stats: stats.clone() }).await;
            }
            return Ok(stats);
        }

        tracing::info!("Found {} new files to index", new_files.len());

        // 100ファイルずつバッチ処理して即座にDBに保存
        // これにより途中で停止しても処理済みファイルは保存される
        const FILE_BATCH_SIZE: usize = 100;
        let total_new_files = new_files.len();

        for (file_batch_idx, file_chunk) in new_files.chunks(FILE_BATCH_SIZE).enumerate() {
            let batch_start = file_batch_idx * FILE_BATCH_SIZE;
            let batch_end = std::cmp::min(batch_start + FILE_BATCH_SIZE, total_new_files);

            tracing::info!(
                "Processing file batch {}: files {}-{} of {}",
                file_batch_idx + 1,
                batch_start + 1,
                batch_end,
                total_new_files
            );

            // 4. ドキュメント読み込み（バッチ分）
            let (load_progress_tx, mut load_progress_rx) = mpsc::channel::<crate::document::LoadProgress>(100);

            let loader_clone = loader.clone();
            let file_chunk_vec: Vec<PathBuf> = file_chunk.to_vec();
            let progress_tx_clone = progress_tx.clone();
            let batch_offset = batch_start;

            tokio::spawn(async move {
                while let Some(load_prog) = load_progress_rx.recv().await {
                    if let Some(tx) = &progress_tx_clone {
                        let _ = tx
                            .send(IndexProgress::Loading {
                                current: batch_offset + load_prog.current,
                                total: total_new_files,
                                file: load_prog.current_file,
                            })
                            .await;
                    }
                }
            });

            let batch_file_count = file_chunk_vec.len();
            let documents = loader_clone
                .load_files(&file_chunk_vec, Some(load_progress_tx))
                .await?;

            if documents.is_empty() {
                tracing::warn!("No documents loaded in batch {}", file_batch_idx + 1);
                // ドキュメントがロードされなくてもファイルは処理済み
                stats.indexed_files += batch_file_count;
                continue;
            }

            // ファイル数をカウント（ドキュメント数ではなく実際のファイル数）
            stats.indexed_files += batch_file_count;

            // 5. テキスト分割
            if let Some(tx) = &progress_tx {
                let _ = tx
                    .send(IndexProgress::Splitting {
                        current: batch_start,
                        total: total_new_files,
                    })
                    .await;
            }

            let raw_splits = self.text_splitter.split_documents(documents);
            tracing::info!("Batch {}: Split into {} raw chunks", file_batch_idx + 1, raw_splits.len());

            // 6. Embedding生成
            // nomic-embed-textのコンテキスト長は8192トークン
            // 日本語は1文字≒2-3トークンなので、安全のため1000文字でトランケート
            // 空のテキストはスキップする
            const MAX_CHARS_PER_CHUNK: usize = 1000;

            // 空でないチャンクのみをフィルタし、テキストをトランケート
            let mut splits = Vec::new();
            let mut texts = Vec::new();
            for (idx, mut doc) in raw_splits.into_iter().enumerate() {
                let trimmed = doc.content.trim();
                if trimmed.is_empty() {
                    tracing::warn!("Skipping empty chunk {}", idx);
                    continue;
                }

                let char_count = trimmed.chars().count();
                let text = if char_count > MAX_CHARS_PER_CHUNK {
                    tracing::warn!("Truncating chunk {} from {} to {} chars", idx, char_count, MAX_CHARS_PER_CHUNK);
                    trimmed.chars().take(MAX_CHARS_PER_CHUNK).collect::<String>()
                } else {
                    trimmed.to_string()
                };

                doc.content = text.clone();
                splits.push(doc);
                texts.push(text);
            }

            stats.total_chunks += splits.len();
            let batch_texts_len = texts.len();

            tracing::info!("Batch {}: {} valid chunks after filtering", file_batch_idx + 1, batch_texts_len);

            if splits.is_empty() {
                continue;
            }

            tracing::info!("Batch {}: Generating embeddings for {} texts", file_batch_idx + 1, batch_texts_len);

            // 進捗チャネルを作成
            let (embed_progress_tx, mut embed_progress_rx) = tokio::sync::mpsc::channel::<(usize, usize)>(100);

            // Embeddingフェーズ開始を通知（チャンク数ベース: 0/total_chunks）
            let total_chunks_for_embed = texts.len();
            if let Some(tx) = &progress_tx {
                let _ = tx
                    .send(IndexProgress::Embedding {
                        current: 0,
                        total: total_chunks_for_embed,
                    })
                    .await;
            }

            // 進捗報告タスク
            let progress_tx_clone = progress_tx.clone();
            let progress_task = tokio::spawn(async move {
                while let Some((current, total)) = embed_progress_rx.recv().await {
                    tracing::info!("Embedding progress: {}/{}", current, total);
                    if let Some(tx) = &progress_tx_clone {
                        let _ = tx.send(IndexProgress::Embedding { current, total }).await;
                    }
                }
            });

            // Embedding生成
            // バッチサイズ8、並列数8でバランスを取る
            let embeddings = self
                .ollama_client
                .embed_batch_with_progress(
                    &self.embedding_model,
                    texts.clone(),
                    8,   // バッチサイズ（8チャンクずつ - 安定性重視）
                    8,   // 並列数（8同時リクエスト）
                    move |current, total| {
                        let _ = embed_progress_tx.try_send((current, total));
                    },
                )
                .await
                .map_err(|e| RagError::Embedding(format!("Failed to generate embeddings: {}", e)))?;

            // embed_progress_tx は move されてクロージャに渡されているので、
            // embed_batch_with_progress 完了時に自動的にドロップされ、
            // progress_task は正常に終了する
            // (ただしタイムアウトを設けて確実に完了させる)
            tracing::info!("Batch {}: Waiting for progress task to complete...", file_batch_idx + 1);
            let timeout_result = tokio::time::timeout(
                std::time::Duration::from_secs(1),
                progress_task
            ).await;
            match timeout_result {
                Ok(_) => tracing::info!("Batch {}: Progress task completed normally", file_batch_idx + 1),
                Err(_) => tracing::info!("Batch {}: Progress task timed out (continuing anyway)", file_batch_idx + 1),
            }

            stats.total_embeddings += embeddings.len();

            tracing::info!("Batch {}: Generated {} embeddings, proceeding to store", file_batch_idx + 1, embeddings.len());

            // 7. メタデータ準備
            let metadatas: Vec<_> = splits
                .iter()
                .map(|d| d.metadata_to_map())
                .collect();

            // 8. ベクトルDB登録（即座に保存）
            if let Some(tx) = &progress_tx {
                let _ = tx
                    .send(IndexProgress::Storing {
                        current: batch_start,
                        total: total_new_files,
                    })
                    .await;
            }

            tracing::info!(
                "Batch {}: Storing {} documents to database",
                file_batch_idx + 1,
                embeddings.len()
            );

            self.vector_db
                .add_documents(embeddings, texts, metadatas, None)
                .await?;

            tracing::info!(
                "Batch {}: Successfully stored to database. Total indexed so far: {} files, {} embeddings",
                file_batch_idx + 1,
                stats.indexed_files,
                stats.total_embeddings
            );

            // バッチ完了通知（リアルタイム統計更新）
            if let Some(tx) = &progress_tx {
                // 進捗通知
                let _ = tx
                    .send(IndexProgress::Storing {
                        current: batch_end,
                        total: total_new_files,
                    })
                    .await;

                // リアルタイム統計更新
                let _ = tx
                    .send(IndexProgress::BatchComplete {
                        stats: stats.clone(),
                    })
                    .await;
            }
        }

        tracing::info!(
            "Indexing complete: {} files, {} chunks, {} embeddings",
            stats.indexed_files,
            stats.total_chunks,
            stats.total_embeddings
        );

        // 9. 完了
        if let Some(tx) = progress_tx {
            let _ = tx.send(IndexProgress::Complete { stats: stats.clone() }).await;
        }

        Ok(stats)
    }

    /// クエリを実行（非ストリーミング）
    pub async fn query(&self, question: &str, k: usize) -> Result<QueryResponse> {
        // 1. クエリのEmbedding生成
        let query_embedding = self
            .ollama_client
            .embed_single(&self.embedding_model, question.to_string())
            .await?;

        // 2. ベクトル検索
        let results = self.vector_db.query(query_embedding, k, None).await?;

        if results.is_empty() {
            return Ok(QueryResponse::new(
                "関連する資料が見つかりませんでした。".to_string(),
                vec![],
            ));
        }

        // 3. コンテキスト構築
        let context = self.build_context(&results);

        // 4. プロンプト構築
        let prompt = format!(
            "資料を参考に日本語で答えてください。\n資料:\n{}\n\n質問: {}",
            context, question
        );

        // 5. LLM生成
        let answer = self.ollama_client.generate(&self.llm_model, &prompt).await?;

        Ok(QueryResponse::new(answer, results))
    }

    /// クエリを実行（ストリーミング）
    pub async fn query_stream(
        &self,
        question: &str,
        k: usize,
    ) -> Result<QueryResponse> {
        // 1. クエリのEmbedding生成
        let query_embedding = self
            .ollama_client
            .embed_single(&self.embedding_model, question.to_string())
            .await?;

        // 2. ベクトル検索
        let results = self.vector_db.query(query_embedding, k, None).await?;

        if results.is_empty() {
            let (tx, rx) = mpsc::channel(1);
            let _ = tx
                .send(Ok("関連する資料が見つかりませんでした。".to_string()))
                .await;
            return Ok(QueryResponse::with_stream(vec![], tokio_stream::wrappers::ReceiverStream::new(rx)));
        }

        // 3. コンテキスト構築
        let context = self.build_context(&results);

        // 4. プロンプト構築
        let prompt = format!(
            "資料を参考に日本語で答えてください。\n資料:\n{}\n\n質問: {}",
            context, question
        );

        // 5. LLMストリーミング生成
        let ollama_stream = self
            .ollama_client
            .generate_stream(&self.llm_model, &prompt)
            .await?;

        // ストリームをReceiverStreamに変換
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        tokio::spawn(async move {
            use futures::StreamExt;
            let mut stream = Box::pin(ollama_stream);
            while let Some(chunk) = stream.next().await {
                if tx.send(chunk).await.is_err() {
                    break;
                }
            }
        });

        Ok(QueryResponse::with_stream(results, tokio_stream::wrappers::ReceiverStream::new(rx)))
    }

    /// コンテキストを構築
    fn build_context(&self, results: &[SearchResult]) -> String {
        results
            .iter()
            .map(|r| {
                let source = r.source().unwrap_or("Unknown");
                let page = r.page().map(|p| format!(" (P.{})", p + 1)).unwrap_or_default();
                format!("【出典: {}{}】\n{}", source, page, r.document)
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_stats_creation() {
        let stats = IndexStats::new();
        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.indexed_files, 0);
        assert_eq!(stats.total_chunks, 0);
    }

    #[test]
    fn test_index_progress_percentage() {
        let progress = IndexProgress::Loading {
            current: 50,
            total: 100,
            file: PathBuf::from("test.pdf"),
        };
        assert_eq!(progress.percentage(), 50.0);

        let complete = IndexProgress::Complete {
            stats: IndexStats::new(),
        };
        assert_eq!(complete.percentage(), 100.0);
    }
}
