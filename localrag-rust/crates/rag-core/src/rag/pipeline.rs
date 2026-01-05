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
        let indexed_sources: HashSet<String> = self
            .vector_db
            .get_indexed_sources()
            .await?
            .into_iter()
            .collect();

        tracing::info!("Found {} already indexed sources", indexed_sources.len());

        // 3. ファイルスキャン
        let loader = ParallelDocumentLoader::new();
        let all_files = loader.scan_directory(dir)?;

        stats.total_files = all_files.len();

        // 差分検出: 新しいファイルのみ
        let new_files: Vec<PathBuf> = all_files
            .into_iter()
            .filter(|f| {
                let normalized = f.display().to_string();
                !indexed_sources.contains(&normalized)
            })
            .collect();

        if new_files.is_empty() {
            tracing::info!("All files are already indexed");
            if let Some(tx) = progress_tx {
                let _ = tx.send(IndexProgress::Complete { stats: stats.clone() }).await;
            }
            return Ok(stats);
        }

        tracing::info!("Found {} new files to index", new_files.len());

        // 4. ドキュメント読み込み（並列）
        let (load_progress_tx, mut load_progress_rx) = mpsc::channel::<crate::document::LoadProgress>(100);

        let loader_clone = loader.clone();
        let new_files_clone = new_files.clone();
        let progress_tx_clone = progress_tx.clone();

        tokio::spawn(async move {
            while let Some(load_prog) = load_progress_rx.recv().await {
                if let Some(tx) = &progress_tx_clone {
                    let _ = tx
                        .send(IndexProgress::Loading {
                            current: load_prog.current,
                            total: load_prog.total,
                            file: load_prog.current_file,
                        })
                        .await;
                }
            }
        });

        let documents = loader_clone
            .load_files(&new_files_clone, Some(load_progress_tx))
            .await?;

        if documents.is_empty() {
            tracing::warn!("No documents were loaded");
            stats.skipped_files = new_files.len();
            if let Some(tx) = progress_tx {
                let _ = tx.send(IndexProgress::Complete { stats: stats.clone() }).await;
            }
            return Ok(stats);
        }

        stats.indexed_files = documents.len();

        // 5. テキスト分割
        if let Some(tx) = &progress_tx {
            let _ = tx
                .send(IndexProgress::Splitting {
                    current: 0,
                    total: documents.len(),
                })
                .await;
        }

        let splits = self.text_splitter.split_documents(documents);
        stats.total_chunks = splits.len();

        tracing::info!("Split into {} chunks", splits.len());

        if let Some(tx) = &progress_tx {
            let _ = tx
                .send(IndexProgress::Splitting {
                    current: splits.len(),
                    total: splits.len(),
                })
                .await;
        }

        // 6. Embedding生成（バッチ並列、進捗報告付き）
        let texts: Vec<String> = splits.iter().map(|d| d.content.clone()).collect();
        let total_texts = texts.len();

        tracing::info!("Starting embedding generation for {} texts", total_texts);

        if let Some(tx) = &progress_tx {
            let _ = tx
                .send(IndexProgress::Embedding {
                    current: 0,
                    total: total_texts,
                })
                .await;
        }

        // 進捗チャネルを作成
        let (embed_progress_tx, mut embed_progress_rx) = tokio::sync::mpsc::channel::<(usize, usize)>(100);

        // 進捗報告タスク
        let progress_tx_clone = progress_tx.clone();
        let progress_task = tokio::spawn(async move {
            while let Some((current, total)) = embed_progress_rx.recv().await {
                tracing::debug!("Embedding progress: {}/{}", current, total);
                if let Some(tx) = &progress_tx_clone {
                    let _ = tx.send(IndexProgress::Embedding { current, total }).await;
                }
            }
        });

        // Embedding生成
        let embeddings = {
            let embeddings = self
                .ollama_client
                .embed_batch_with_progress(
                    &self.embedding_model,
                    texts.clone(),
                    30,  // バッチサイズ（タイムアウト対策で削減）
                    5,   // 並列数（タイムアウト対策で削減）
                    move |current, total| {
                        let _ = embed_progress_tx.try_send((current, total));
                    },
                )
                .await
                .map_err(|e| RagError::Embedding(format!("Failed to generate embeddings: {}", e)))?;

            // 進捗タスクを終了（チャネルがクロージャで消費されたので自動的にクローズされる）
            let _ = progress_task.await;
            embeddings
        };

        stats.total_embeddings = embeddings.len();

        tracing::info!("Generated {} embeddings", embeddings.len());

        // 7. メタデータ準備
        let metadatas: Vec<_> = splits
            .iter()
            .map(|d| d.metadata_to_map())
            .collect();

        // 8. ベクトルDB登録（バッチ分割）
        let max_batch_size = 5000; // ChromaDBの制限より小さい値
        let total_docs = embeddings.len();

        if let Some(tx) = &progress_tx {
            let _ = tx
                .send(IndexProgress::Storing {
                    current: 0,
                    total: total_docs,
                })
                .await;
        }

        // バッチに分割して保存
        for (batch_idx, chunk_size) in (0..total_docs).step_by(max_batch_size).enumerate() {
            let end = std::cmp::min(chunk_size + max_batch_size, total_docs);
            let batch_embeddings = embeddings[chunk_size..end].to_vec();
            let batch_texts = texts[chunk_size..end].to_vec();
            let batch_metadatas = metadatas[chunk_size..end].to_vec();

            tracing::info!(
                "Storing batch {}: {} documents ({}-{})",
                batch_idx + 1,
                batch_embeddings.len(),
                chunk_size,
                end
            );

            self.vector_db
                .add_documents(batch_embeddings, batch_texts, batch_metadatas, None)
                .await?;

            if let Some(tx) = &progress_tx {
                let _ = tx
                    .send(IndexProgress::Storing {
                        current: end,
                        total: total_docs,
                    })
                    .await;
            }
        }

        tracing::info!("Stored all {} documents in vector database", total_docs);

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
