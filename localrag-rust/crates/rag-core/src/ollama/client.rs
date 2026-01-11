use futures::stream::{Stream, StreamExt};
use reqwest::{Client, StatusCode};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use super::types::*;
use crate::error::{RagError, Result};

/// Ollama APIクライアント
#[derive(Clone)]
pub struct OllamaClient {
    base_url: String,
    client: Client,
    timeout: Duration,
}

impl OllamaClient {
    /// 新しいクライアントを作成（高速化のためHTTPクライアントを最適化）
    pub fn new(base_url: impl Into<String>) -> Self {
        // 接続プール・keep-alive・HTTP/2を有効化して高速化
        let client = Client::builder()
            .pool_max_idle_per_host(20)       // 接続プールサイズ増加
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_keepalive(Duration::from_secs(60))
            .tcp_nodelay(true)                 // Nagleアルゴリズム無効化（低レイテンシ）
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            base_url: base_url.into(),
            client,
            timeout: Duration::from_secs(300), // 5分に延長
        }
    }

    /// デフォルトのクライアント（localhost:11434）
    pub fn default() -> Self {
        Self::new("http://localhost:11434")
    }

    /// タイムアウトを設定
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Ollamaが実行中かチェック
    /// Note: Timeout increased to 5 seconds to accommodate slower GPU initialization
    /// (especially Intel GPU with OLLAMA_INTEL_GPU=1)
    pub async fn check_running(&self) -> bool {
        let url = format!("{}/api/tags", self.base_url);
        match self
            .client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    /// 利用可能なモデル一覧を取得
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        let url = format!("{}/api/tags", self.base_url);

        let response = self
            .client
            .get(&url)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    RagError::OllamaApi("Request timeout".to_string())
                } else {
                    RagError::Http(e)
                }
            })?;

        if !response.status().is_success() {
            return Err(RagError::OllamaApi(format!(
                "Failed to list models: {}",
                response.status()
            )));
        }

        let models_response: ModelsResponse = response.json().await?;
        Ok(models_response.models)
    }

    /// LLMモデルのみをフィルタして取得
    pub async fn list_llm_models(&self) -> Result<Vec<ModelInfo>> {
        let all_models = self.list_models().await?;
        Ok(all_models
            .into_iter()
            .filter(|m| !m.name.to_lowercase().contains("embed"))
            .collect())
    }

    /// Embeddingモデルのみをフィルタして取得
    pub async fn list_embedding_models(&self) -> Result<Vec<ModelInfo>> {
        let all_models = self.list_models().await?;
        Ok(all_models
            .into_iter()
            .filter(|m| m.name.to_lowercase().contains("embed"))
            .collect())
    }

    /// 単一テキストのEmbeddingを生成
    pub async fn embed_single(&self, model: &str, text: String) -> Result<Vec<f32>> {
        let embeddings = self.embed(model, vec![text]).await?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| RagError::Embedding("No embedding returned".to_string()))
    }

    /// 複数テキストのEmbeddingを生成
    pub async fn embed(&self, model: &str, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        let url = format!("{}/api/embed", self.base_url);

        let request = EmbedRequest {
            model: model.to_string(),
            input: texts,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    RagError::OllamaApi("Embedding request timeout".to_string())
                } else {
                    RagError::Http(e)
                }
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            tracing::error!("Embedding API error: status={}, body={}", status, error_body);
            return Err(RagError::OllamaApi(format!(
                "Failed to generate embeddings: {} - {}",
                status, error_body
            )));
        }

        let embed_response: EmbedResponse = response.json().await?;
        Ok(embed_response.embeddings)
    }

    /// バッチEmbedding生成（並列リクエスト）
    pub async fn embed_batch(
        &self,
        model: &str,
        texts: Vec<String>,
        batch_size: usize,
        max_concurrent: usize,
    ) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let chunks: Vec<_> = texts.chunks(batch_size).map(|c| c.to_vec()).collect();

        // 並列リクエスト（max_concurrent同時）
        let stream = futures::stream::iter(chunks)
            .map(|chunk| {
                let client = self.clone();
                let model = model.to_string();
                async move { client.embed(&model, chunk).await }
            })
            .buffer_unordered(max_concurrent);

        let results: Vec<_> = stream.collect().await;

        // 結果を平坦化
        let mut all_embeddings = Vec::new();
        for result in results {
            match result {
                Ok(embeddings) => all_embeddings.extend(embeddings),
                Err(e) => return Err(e),
            }
        }

        Ok(all_embeddings)
    }

    /// バッチEmbedding生成（進捗報告付き）
    pub async fn embed_batch_with_progress<F>(
        &self,
        model: &str,
        texts: Vec<String>,
        batch_size: usize,
        max_concurrent: usize,
        mut progress_callback: F,
    ) -> Result<Vec<Vec<f32>>>
    where
        F: FnMut(usize, usize) + Send,
    {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let total = texts.len();
        let chunks: Vec<_> = texts.chunks(batch_size).map(|c| c.to_vec()).collect();

        // 並列リクエスト（max_concurrent同時）
        use futures::StreamExt;

        let mut stream = futures::stream::iter(chunks.into_iter().enumerate())
            .map(|(idx, chunk)| {
                let client = self.clone();
                let model = model.to_string();
                async move {
                    let result = client.embed(&model, chunk.clone()).await;
                    (idx, chunk.len(), result)
                }
            })
            .buffer_unordered(max_concurrent);

        // ストリームから結果を受け取りながら進捗報告
        let mut results = Vec::new();
        let mut processed = 0;

        while let Some((idx, chunk_len, result)) = stream.next().await {
            results.push((idx, chunk_len, result));
            processed += chunk_len;
            // リアルタイムで進捗報告
            progress_callback(processed, total);
            tracing::info!("Embedding batch completed: {}/{}", processed, total);
        }

        tracing::info!("All embedding batches received, sorting {} results", results.len());

        // インデックス順にソート
        results.sort_by_key(|(idx, _, _)| *idx);

        tracing::info!("Results sorted, flattening {} batch results", results.len());

        // 結果を平坦化
        let mut all_embeddings = Vec::new();
        for (batch_idx, (_, _, result)) in results.into_iter().enumerate() {
            match result {
                Ok(embeddings) => {
                    tracing::info!("Batch result {}: {} embeddings", batch_idx, embeddings.len());
                    all_embeddings.extend(embeddings);
                }
                Err(e) => {
                    tracing::error!("Batch result {} failed: {}", batch_idx, e);
                    return Err(e);
                }
            }
        }

        tracing::info!("All embeddings flattened: {} total", all_embeddings.len());
        Ok(all_embeddings)
    }

    /// テキスト生成（非ストリーミング）
    pub async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
        let url = format!("{}/api/generate", self.base_url);

        let request = GenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            system: None,
            stream: false,
            context: None,
            options: None,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .timeout(self.timeout)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(RagError::OllamaApi(format!(
                "Failed to generate text: {}",
                response.status()
            )));
        }

        let generate_response: GenerateResponse = response.json().await?;
        Ok(generate_response.response)
    }

    /// テキスト生成（ストリーミング）
    pub async fn generate_stream(
        &self,
        model: &str,
        prompt: &str,
    ) -> Result<impl Stream<Item = Result<String>>> {
        let url = format!("{}/api/generate", self.base_url);

        let request = GenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            system: None,
            stream: true,
            context: None,
            options: None,
        };

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            return Err(RagError::OllamaApi(format!(
                "Failed to start streaming: {}",
                response.status()
            )));
        }

        let (tx, rx) = mpsc::channel(100);

        tokio::spawn(async move {
            let mut stream = response.bytes_stream();

            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        // 各行をJSONとしてパース
                        for line in bytes.split(|&b| b == b'\n') {
                            if line.is_empty() {
                                continue;
                            }

                            match serde_json::from_slice::<GenerateResponse>(line) {
                                Ok(resp) => {
                                    if tx.send(Ok(resp.response)).await.is_err() {
                                        return; // Receiver dropped
                                    }

                                    if resp.done {
                                        return;
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!("Failed to parse streaming response: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx
                            .send(Err(RagError::OllamaApi(format!("Stream error: {}", e))))
                            .await;
                        return;
                    }
                }
            }
        });

        Ok(ReceiverStream::new(rx))
    }

    /// チャット生成（非ストリーミング）
    pub async fn chat(&self, model: &str, messages: Vec<ChatMessage>) -> Result<String> {
        let url = format!("{}/api/chat", self.base_url);

        let request = ChatRequest {
            model: model.to_string(),
            messages,
            stream: false,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .timeout(self.timeout)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(RagError::OllamaApi(format!(
                "Failed to chat: {}",
                response.status()
            )));
        }

        let chat_response: ChatResponse = response.json().await?;
        Ok(chat_response.message.content)
    }

    /// モデルをpull（インストール）
    pub async fn pull_model(&self, model_name: &str) -> Result<()> {
        let url = format!("{}/api/pull", self.base_url);

        let request = serde_json::json!({
            "name": model_name,
        });

        let response = self
            .client
            .post(&url)
            .json(&request)
            .timeout(Duration::from_secs(600)) // 10分タイムアウト
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(RagError::OllamaApi(format!(
                "Failed to pull model: {}",
                response.status()
            )));
        }

        Ok(())
    }

    /// モデルを削除
    pub async fn delete_model(&self, model_name: &str) -> Result<()> {
        let url = format!("{}/api/delete", self.base_url);

        let request = serde_json::json!({
            "name": model_name,
        });

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            return Err(RagError::OllamaApi(format!(
                "Failed to delete model: {}",
                response.status()
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = OllamaClient::default();
        assert_eq!(client.base_url, "http://localhost:11434");
    }

    #[test]
    fn test_custom_base_url() {
        let client = OllamaClient::new("http://example.com:8080");
        assert_eq!(client.base_url, "http://example.com:8080");
    }

    #[tokio::test]
    async fn test_check_running_timeout() {
        let client = OllamaClient::new("http://localhost:99999"); // Invalid port
        let running = client.check_running().await;
        assert!(!running);
    }
}
