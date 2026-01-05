use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ベクトルDB設定
#[derive(Debug, Clone)]
pub struct VectorDbConfig {
    pub base_url: String,
    pub collection_name: String,
    pub embedding_dimension: usize,
}

impl VectorDbConfig {
    pub fn new(collection_name: impl Into<String>, embedding_dimension: usize) -> Self {
        Self {
            base_url: "http://localhost:8001".to_string(),  // ChromaDBブリッジサーバーのポート
            collection_name: collection_name.into(),
            embedding_dimension,
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }
}

/// 検索結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub document: String,
    pub metadata: HashMap<String, String>,
    pub distance: f32,
}

impl SearchResult {
    pub fn new(
        id: String,
        document: String,
        metadata: HashMap<String, String>,
        distance: f32,
    ) -> Self {
        Self {
            id,
            document,
            metadata,
            distance,
        }
    }

    /// ソースファイルパスを取得
    pub fn source(&self) -> Option<&str> {
        self.metadata.get("source").map(|s| s.as_str())
    }

    /// ページ番号を取得
    pub fn page(&self) -> Option<usize> {
        self.metadata
            .get("page")
            .and_then(|s| s.parse().ok())
    }

    /// スコアを取得（距離の逆数、0-1の範囲に正規化）
    pub fn score(&self) -> f32 {
        // 距離を類似度スコアに変換（距離が小さいほど高スコア）
        1.0 - self.distance.min(1.0)
    }
}

/// ベクトルDB統計情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorDbStats {
    pub total_documents: usize,
    pub total_embeddings: usize,
    pub collection_name: String,
}

/// ChromaDB: コレクション作成リクエスト
#[derive(Debug, Serialize)]
pub struct CreateCollectionRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// ChromaDB: コレクションレスポンス
#[derive(Debug, Deserialize)]
pub struct CollectionResponse {
    pub name: String,
    pub id: String,
    #[serde(default)]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// ChromaDB: ドキュメント追加リクエスト
#[derive(Debug, Serialize)]
pub struct AddDocumentsRequest {
    pub embeddings: Vec<Vec<f32>>,
    pub documents: Vec<String>,
    pub metadatas: Vec<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<Vec<String>>,
}

/// ChromaDB: クエリリクエスト
#[derive(Debug, Serialize)]
pub struct QueryRequest {
    pub query_embeddings: Vec<Vec<f32>>,
    pub n_results: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub where_filter: Option<HashMap<String, serde_json::Value>>,
}

/// ChromaDB: クエリレスポンス
#[derive(Debug, Deserialize)]
pub struct QueryResponse {
    pub ids: Vec<Vec<String>>,
    pub distances: Vec<Vec<f32>>,
    pub documents: Vec<Vec<String>>,
    pub metadatas: Vec<Vec<HashMap<String, String>>>,
}

impl QueryResponse {
    /// SearchResultに変換
    pub fn into_search_results(self) -> Vec<SearchResult> {
        let mut results = Vec::new();

        if !self.ids.is_empty() {
            let ids = &self.ids[0];
            let distances = &self.distances[0];
            let documents = &self.documents[0];
            let metadatas = &self.metadatas[0];

            for i in 0..ids.len() {
                results.push(SearchResult::new(
                    ids[i].clone(),
                    documents[i].clone(),
                    metadatas[i].clone(),
                    distances[i],
                ));
            }
        }

        results
    }
}

/// ChromaDB: Get レスポンス
#[derive(Debug, Deserialize)]
pub struct GetResponse {
    pub ids: Vec<String>,
    #[serde(default)]
    pub documents: Vec<String>,
    #[serde(default)]
    pub metadatas: Vec<HashMap<String, String>>,
}
