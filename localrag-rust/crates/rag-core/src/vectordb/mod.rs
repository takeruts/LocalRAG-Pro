pub mod types;
pub mod chroma;
pub mod hnsw;

pub use types::{VectorDbConfig, SearchResult, VectorDbStats};
pub use chroma::ChromaClient;
pub use hnsw::{HnswClient, HnswConfig};

use async_trait::async_trait;
use std::collections::HashMap;

use crate::error::Result;

/// ベクトルデータベーストレイト
#[async_trait]
pub trait VectorDatabase: Send + Sync {
    /// コレクション/インデックスが存在するか確認
    async fn collection_exists(&self) -> Result<bool>;

    /// コレクション/インデックスを作成
    async fn create_collection(&self) -> Result<()>;

    /// コレクション/インデックスを削除
    async fn delete_collection(&self) -> Result<()>;

    /// ドキュメントを追加（バッチ）
    async fn add_documents(
        &self,
        embeddings: Vec<Vec<f32>>,
        documents: Vec<String>,
        metadatas: Vec<HashMap<String, String>>,
        ids: Option<Vec<String>>,
    ) -> Result<()>;

    /// ベクトル検索
    async fn query(
        &self,
        query_embedding: Vec<f32>,
        k: usize,
        filter: Option<HashMap<String, String>>,
    ) -> Result<Vec<SearchResult>>;

    /// インデックス済みソースを取得（差分検出用）
    async fn get_indexed_sources(&self) -> Result<Vec<String>>;

    /// データベース統計情報
    async fn get_stats(&self) -> Result<VectorDbStats>;

    /// ドキュメント数を取得
    async fn count(&self) -> Result<usize>;
}
