use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;
use uuid::Uuid;

use super::types::*;
use super::VectorDatabase;
use crate::error::{RagError, Result};

/// ChromaDB HTTPクライアント
#[derive(Clone)]
pub struct ChromaClient {
    base_url: String,
    collection_name: String,
    client: Client,
}

impl ChromaClient {
    /// 新しいクライアントを作成
    pub fn new(config: VectorDbConfig) -> Self {
        Self {
            base_url: config.base_url,
            collection_name: config.collection_name,
            client: Client::new(),
        }
    }

    /// デフォルトクライアント（localhost:8000）
    pub fn default(collection_name: impl Into<String>) -> Self {
        Self::new(VectorDbConfig::new(collection_name, 768))
    }

    /// コレクションIDを取得
    async fn get_collection_id(&self) -> Result<String> {
        let url = format!("{}/api/v1/collections/{}", self.base_url, self.collection_name);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(RagError::VectorDb(format!(
                "Collection not found: {}",
                self.collection_name
            )));
        }

        let collection: CollectionResponse = response.json().await?;
        Ok(collection.id)
    }

    /// バッチでドキュメントを追加（並列処理）
    pub async fn add_documents_batch(
        &self,
        embeddings: Vec<Vec<f32>>,
        documents: Vec<String>,
        metadatas: Vec<HashMap<String, String>>,
        batch_size: usize,
    ) -> Result<()> {
        if embeddings.is_empty() {
            return Ok(());
        }

        let total = embeddings.len();

        // バッチに分割して並列処理
        let batches: Vec<_> = (0..total)
            .step_by(batch_size)
            .map(|i| {
                let end = (i + batch_size).min(total);
                (
                    embeddings[i..end].to_vec(),
                    documents[i..end].to_vec(),
                    metadatas[i..end].to_vec(),
                )
            })
            .collect();

        // 並列バッチ追加
        for (batch_emb, batch_doc, batch_meta) in batches {
            self.add_documents(batch_emb, batch_doc, batch_meta, None)
                .await?;
        }

        Ok(())
    }
}

#[async_trait]
impl VectorDatabase for ChromaClient {
    async fn collection_exists(&self) -> Result<bool> {
        let url = format!("{}/api/v1/collections/{}", self.base_url, self.collection_name);

        let response = self.client.get(&url).send().await?;

        Ok(response.status().is_success())
    }

    async fn create_collection(&self) -> Result<()> {
        let url = format!("{}/api/v1/collections", self.base_url);

        let request = CreateCollectionRequest {
            name: self.collection_name.clone(),
            metadata: None,
        };

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(RagError::VectorDb(format!(
                "Failed to create collection: {} - {}",
                status, text
            )));
        }

        Ok(())
    }

    async fn delete_collection(&self) -> Result<()> {
        let url = format!("{}/api/v1/collections/{}", self.base_url, self.collection_name);

        let response = self.client.delete(&url).send().await?;

        if !response.status().is_success() {
            return Err(RagError::VectorDb(format!(
                "Failed to delete collection: {}",
                response.status()
            )));
        }

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

        // IDが指定されていない場合はUUIDを生成
        let ids = ids.unwrap_or_else(|| {
            (0..embeddings.len())
                .map(|_| Uuid::new_v4().to_string())
                .collect()
        });

        let collection_id = self.get_collection_id().await?;
        let url = format!(
            "{}/api/v1/collections/{}/add",
            self.base_url, collection_id
        );

        let request = AddDocumentsRequest {
            embeddings,
            documents,
            metadatas,
            ids: Some(ids),
        };

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(RagError::VectorDb(format!(
                "Failed to add documents: {} - {}",
                status, text
            )));
        }

        Ok(())
    }

    async fn query(
        &self,
        query_embedding: Vec<f32>,
        k: usize,
        filter: Option<HashMap<String, String>>,
    ) -> Result<Vec<SearchResult>> {
        let collection_id = self.get_collection_id().await?;
        let url = format!(
            "{}/api/v1/collections/{}/query",
            self.base_url, collection_id
        );

        let where_filter = filter.map(|f| {
            f.into_iter()
                .map(|(k, v)| (k, serde_json::Value::String(v)))
                .collect()
        });

        let request = QueryRequest {
            query_embeddings: vec![query_embedding],
            n_results: k,
            where_filter,
        };

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            return Err(RagError::VectorDb(format!(
                "Failed to query: {}",
                response.status()
            )));
        }

        let query_response: QueryResponse = response.json().await?;
        Ok(query_response.into_search_results())
    }

    async fn get_indexed_sources(&self) -> Result<Vec<String>> {
        let collection_id = self.get_collection_id().await?;
        let url = format!(
            "{}/api/v1/collections/{}/get",
            self.base_url, collection_id
        );

        let response = self.client.post(&url).json(&serde_json::json!({})).send().await?;

        if !response.status().is_success() {
            return Err(RagError::VectorDb(format!(
                "Failed to get documents: {}",
                response.status()
            )));
        }

        let get_response: GetResponse = response.json().await?;

        // メタデータからソースを抽出
        let sources: Vec<String> = get_response
            .metadatas
            .iter()
            .filter_map(|meta| meta.get("source").cloned())
            .collect();

        Ok(sources)
    }

    async fn get_stats(&self) -> Result<VectorDbStats> {
        let count = self.count().await?;

        Ok(VectorDbStats {
            total_documents: count,
            total_embeddings: count,
            collection_name: self.collection_name.clone(),
        })
    }

    async fn count(&self) -> Result<usize> {
        // コレクション存在確認
        if !self.collection_exists().await? {
            return Ok(0);
        }

        let collection_id = self.get_collection_id().await?;

        // getエンドポイントを使ってドキュメント数を取得
        let url = format!(
            "{}/api/v1/collections/{}/get",
            self.base_url, collection_id
        );

        let response = self.client
            .post(&url)
            .json(&serde_json::json!({}))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(RagError::VectorDb(format!(
                "Failed to get documents: {}",
                response.status()
            )));
        }

        let get_response: GetResponse = response.json().await?;

        // ドキュメント数を返す
        let count = get_response.ids.len();
        tracing::info!("ChromaDB document count: {}", count);

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = ChromaClient::default("test_collection");
        assert_eq!(client.collection_name, "test_collection");
        assert_eq!(client.base_url, "http://localhost:8000");
    }

    #[test]
    fn test_config() {
        let config = VectorDbConfig::new("my_collection", 768)
            .with_base_url("http://localhost:9000");

        assert_eq!(config.collection_name, "my_collection");
        assert_eq!(config.base_url, "http://localhost:9000");
        assert_eq!(config.embedding_dimension, 768);
    }
}
