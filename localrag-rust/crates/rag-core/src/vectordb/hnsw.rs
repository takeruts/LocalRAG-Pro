//! HNSW ベクトルデータベースクライアント
//!
//! instant-distanceを使用した純粋Rust実装の埋め込みベクトルデータベース。
//! 外部サーバー不要で、単一アプリケーション内で動作する。

use async_trait::async_trait;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::sync::Arc;

use instant_distance::{Builder, HnswMap, Point, Search};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::types::{SearchResult, VectorDbStats};
use super::VectorDatabase;
use crate::error::{RagError, Result};

/// ベクトルポイント
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VectorPoint {
    pub values: Vec<f32>,
}

impl Point for VectorPoint {
    fn distance(&self, other: &Self) -> f32 {
        // コサイン距離 = 1 - コサイン類似度
        let dot: f32 = self.values.iter().zip(other.values.iter()).map(|(a, b)| a * b).sum();
        let norm_a: f32 = self.values.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = other.values.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 1.0;
        }

        1.0 - (dot / (norm_a * norm_b))
    }
}

/// ドキュメントメタデータ
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocumentData {
    pub id: String,
    pub document: String,
    pub source: String,
    pub page: String,
    pub vector: Vec<f32>,
}

/// 永続化用データ構造（ベクトルとドキュメントのみ）
#[derive(Serialize, Deserialize)]
struct PersistedData {
    documents: Vec<DocumentData>,
}

/// ランタイムデータ（HNSWインデックス含む）
struct RuntimeData {
    hnsw: HnswMap<VectorPoint, usize>,
    documents: Vec<DocumentData>,
}

/// HNSW設定
#[derive(Debug, Clone)]
pub struct HnswConfig {
    /// データベースパス
    pub db_path: PathBuf,
    /// コレクション名
    pub collection_name: String,
    /// 埋め込み次元数
    pub embedding_dimension: usize,
}

impl HnswConfig {
    pub fn new(collection_name: impl Into<String>, embedding_dimension: usize) -> Self {
        Self {
            db_path: PathBuf::from("vectordb_data"),
            collection_name: collection_name.into(),
            embedding_dimension,
        }
    }

    pub fn with_db_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.db_path = path.into();
        self
    }

    fn data_file_path(&self) -> PathBuf {
        self.db_path.join(format!("{}.bin", self.collection_name))
    }
}

/// HNSWベースのベクトルDBクライアント
pub struct HnswClient {
    config: HnswConfig,
    data: Arc<RwLock<Option<RuntimeData>>>,
}

impl HnswClient {
    /// 新しいHNSWクライアントを作成
    pub fn new(config: HnswConfig) -> Self {
        Self {
            config,
            data: Arc::new(RwLock::new(None)),
        }
    }

    /// ドキュメントからHNSWインデックスを構築
    fn build_index(documents: &[DocumentData]) -> HnswMap<VectorPoint, usize> {
        let points: Vec<VectorPoint> = documents
            .iter()
            .map(|d| VectorPoint { values: d.vector.clone() })
            .collect();
        let indices: Vec<usize> = (0..documents.len()).collect();

        Builder::default().build(points, indices)
    }

    /// データをロード
    async fn load_data(&self) -> Result<()> {
        let path = self.config.data_file_path();

        if !path.exists() {
            tracing::info!("No existing data file found at {:?}", path);
            return Ok(());
        }

        tracing::info!("Loading HNSW data from {:?}", path);

        let file = File::open(&path)
            .map_err(|e| RagError::VectorDb(format!("Failed to open data file: {}", e)))?;
        let reader = BufReader::new(file);

        let persisted: PersistedData = bincode::deserialize_from(reader)
            .map_err(|e| RagError::VectorDb(format!("Failed to deserialize data: {}", e)))?;

        tracing::info!("Loaded {} documents, rebuilding HNSW index...", persisted.documents.len());

        // HNSWインデックスを再構築
        let hnsw = Self::build_index(&persisted.documents);

        let runtime_data = RuntimeData {
            hnsw,
            documents: persisted.documents,
        };

        *self.data.write().await = Some(runtime_data);
        tracing::info!("HNSW index rebuilt successfully");

        Ok(())
    }

    /// データを保存
    async fn save_data(&self) -> Result<()> {
        let data = self.data.read().await;
        let Some(ref runtime) = *data else {
            return Ok(());
        };

        // ディレクトリを作成
        if let Err(e) = fs::create_dir_all(&self.config.db_path) {
            tracing::warn!("Failed to create db directory: {}", e);
        }

        let path = self.config.data_file_path();
        tracing::info!("Saving HNSW data to {:?}", path);

        let persisted = PersistedData {
            documents: runtime.documents.clone(),
        };

        let file = File::create(&path)
            .map_err(|e| RagError::VectorDb(format!("Failed to create data file: {}", e)))?;
        let writer = BufWriter::new(file);

        bincode::serialize_into(writer, &persisted)
            .map_err(|e| RagError::VectorDb(format!("Failed to serialize data: {}", e)))?;

        tracing::info!("Saved {} documents to HNSW data file", runtime.documents.len());
        Ok(())
    }

    /// データが存在するか確認
    async fn ensure_loaded(&self) -> Result<()> {
        if self.data.read().await.is_none() {
            self.load_data().await?;
        }
        Ok(())
    }
}

#[async_trait]
impl VectorDatabase for HnswClient {
    async fn collection_exists(&self) -> Result<bool> {
        let path = self.config.data_file_path();
        Ok(path.exists())
    }

    async fn create_collection(&self) -> Result<()> {
        // ディレクトリを作成
        if let Err(e) = fs::create_dir_all(&self.config.db_path) {
            tracing::warn!("Failed to create db directory: {}", e);
        }
        tracing::info!("HNSW collection ready: {}", self.config.collection_name);
        Ok(())
    }

    async fn delete_collection(&self) -> Result<()> {
        let path = self.config.data_file_path();
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| RagError::VectorDb(format!("Failed to delete data file: {}", e)))?;
        }
        *self.data.write().await = None;
        tracing::info!("Deleted HNSW collection: {}", self.config.collection_name);
        Ok(())
    }

    async fn add_documents(
        &self,
        embeddings: Vec<Vec<f32>>,
        documents: Vec<String>,
        metadatas: Vec<HashMap<String, String>>,
        ids: Option<Vec<String>>,
    ) -> Result<()> {
        if embeddings.is_empty() {
            return Ok(());
        }

        let num_docs = embeddings.len();
        tracing::info!("Adding {} documents to HNSW index", num_docs);

        // IDを生成または使用
        let ids: Vec<String> = ids.unwrap_or_else(|| {
            (0..num_docs)
                .map(|_| uuid::Uuid::new_v4().to_string())
                .collect()
        });

        // ドキュメントデータを作成
        let mut new_docs: Vec<DocumentData> = Vec::with_capacity(num_docs);
        for i in 0..num_docs {
            new_docs.push(DocumentData {
                id: ids[i].clone(),
                document: documents[i].clone(),
                source: metadatas[i].get("source").cloned().unwrap_or_default(),
                page: metadatas[i].get("page").cloned().unwrap_or_default(),
                vector: embeddings[i].clone(),
            });
        }

        // 既存データをロード
        self.ensure_loaded().await?;

        // 既存データと結合
        let mut all_docs = Vec::new();
        {
            let data = self.data.read().await;
            if let Some(ref existing) = *data {
                all_docs = existing.documents.clone();
            }
        }

        all_docs.extend(new_docs);

        // HNSWインデックスを再構築
        tracing::info!("Rebuilding HNSW index with {} total documents", all_docs.len());
        let hnsw = Self::build_index(&all_docs);

        // データを更新
        *self.data.write().await = Some(RuntimeData {
            hnsw,
            documents: all_docs,
        });

        // 保存
        self.save_data().await?;

        tracing::info!("Added {} documents to HNSW index", num_docs);
        Ok(())
    }

    async fn query(
        &self,
        query_embedding: Vec<f32>,
        k: usize,
        _filter: Option<HashMap<String, String>>,
    ) -> Result<Vec<SearchResult>> {
        self.ensure_loaded().await?;

        let data = self.data.read().await;
        let Some(ref runtime) = *data else {
            tracing::warn!("No data in HNSW index");
            return Ok(vec![]);
        };

        let query_point = VectorPoint {
            values: query_embedding,
        };

        let mut search = Search::default();
        let results: Vec<_> = runtime.hnsw.search(&query_point, &mut search).take(k).collect();

        let mut search_results = Vec::with_capacity(results.len());
        for result in results {
            let doc_idx = *result.value;
            let doc = &runtime.documents[doc_idx];
            let mut metadata = HashMap::new();
            if !doc.source.is_empty() {
                metadata.insert("source".to_string(), doc.source.clone());
            }
            if !doc.page.is_empty() {
                metadata.insert("page".to_string(), doc.page.clone());
            }

            search_results.push(SearchResult::new(
                doc.id.clone(),
                doc.document.clone(),
                metadata,
                result.point.distance(&query_point),
            ));
        }

        tracing::debug!("Query returned {} results", search_results.len());
        Ok(search_results)
    }

    async fn get_indexed_sources(&self) -> Result<Vec<String>> {
        self.ensure_loaded().await?;

        let data = self.data.read().await;
        let Some(ref runtime) = *data else {
            return Ok(vec![]);
        };

        let sources: std::collections::HashSet<String> = runtime
            .documents
            .iter()
            .filter(|d| !d.source.is_empty())
            .map(|d| d.source.clone())
            .collect();

        Ok(sources.into_iter().collect())
    }

    async fn get_stats(&self) -> Result<VectorDbStats> {
        let count = self.count().await?;
        Ok(VectorDbStats {
            total_documents: count,
            total_embeddings: count,
            collection_name: self.config.collection_name.clone(),
        })
    }

    async fn count(&self) -> Result<usize> {
        self.ensure_loaded().await?;

        let data = self.data.read().await;
        let count = match &*data {
            Some(runtime) => runtime.documents.len(),
            None => 0,
        };

        tracing::info!("HNSW document count: {}", count);
        Ok(count)
    }
}

impl Clone for HnswClient {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            data: self.data.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hnsw_client_creation() {
        let config = HnswConfig::new("test_collection", 768);
        let _client = HnswClient::new(config);
    }
}
