use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::error::Result;
use crate::vectordb::{SearchResult, VectorDatabase};

use super::pipeline::RagPipeline;
use super::types::{AgentProgress, QueryResponse};

/// エージェントパイプライン
///
/// 自律的に検索キーワードを抽出し、複数検索を並列実行してRAG推論を行う
#[derive(Clone)]
pub struct AgentPipeline<D: VectorDatabase> {
    pipeline: Arc<RagPipeline<D>>,
}

impl<D: VectorDatabase> AgentPipeline<D> {
    /// 新しいエージェントパイプラインを作成
    pub fn new(pipeline: RagPipeline<D>) -> Self {
        Self {
            pipeline: Arc::new(pipeline),
        }
    }

    /// エージェントモードでクエリを実行
    pub async fn query_agent(
        &self,
        question: &str,
        progress_tx: Option<mpsc::Sender<AgentProgress>>,
    ) -> Result<QueryResponse> {
        // ステップ1: 質問を分析してキーワードを抽出
        if let Some(tx) = &progress_tx {
            let _ = tx.send(AgentProgress::Analyzing).await;
        }

        let keywords = self.extract_keywords(question).await?;

        tracing::info!("Extracted keywords: {:?}", keywords);

        if let Some(tx) = &progress_tx {
            let _ = tx.send(AgentProgress::Keywords(keywords.clone())).await;
        }

        // ステップ2: 並列検索
        let mut all_docs = Vec::new();
        let mut seen_sources = HashSet::new();

        for keyword in &keywords {
            if let Some(tx) = &progress_tx {
                let _ = tx.send(AgentProgress::Searching(keyword.clone())).await;
            }

            // キーワードごとに検索
            let query_embedding = self
                .pipeline
                .ollama_client
                .embed_single(&self.pipeline.embedding_model, keyword.clone())
                .await?;

            let docs = self
                .pipeline
                .vector_db
                .query(query_embedding, 5, None)
                .await?;

            // 重複除外して追加
            for doc in docs {
                if let Some(source) = doc.source() {
                    if !seen_sources.contains(source) {
                        all_docs.push(doc.clone());
                        seen_sources.insert(source.to_string());
                    }
                }
            }
        }

        if let Some(tx) = &progress_tx {
            let _ = tx.send(AgentProgress::Found(all_docs.len())).await;
        }

        if all_docs.is_empty() {
            return self.pipeline.query(question, 3).await;
        }

        // ステップ3: 資料の十分性を検証
        if let Some(tx) = &progress_tx {
            let _ = tx.send(AgentProgress::ValidatingSufficiency).await;
        }

        let is_sufficient = self.validate_sufficiency(question, &all_docs).await?;

        if !is_sufficient {
            tracing::warn!("Found resources are not sufficient, falling back to normal query");
            return self.pipeline.query(question, 5).await;
        }

        // ステップ4: 最終回答生成
        if let Some(tx) = &progress_tx {
            let _ = tx.send(AgentProgress::Generating).await;
        }

        let context = self.build_context(&all_docs);

        let prompt = format!(
            "以下の資料を参考に、質問に詳しく答えてください。\n\
             資料のどの部分を参照したかも明示してください。\n\n\
             資料:\n{}\n\n質問: {}",
            context, question
        );

        let ollama_stream = self
            .pipeline
            .ollama_client
            .generate_stream(&self.pipeline.llm_model, &prompt)
            .await?;

        // ストリームをReceiverStreamに変換
        let (tx_stream, rx_stream) = tokio::sync::mpsc::channel(100);
        tokio::spawn(async move {
            use futures::StreamExt;
            let mut stream = Box::pin(ollama_stream);
            while let Some(chunk) = stream.next().await {
                if tx_stream.send(chunk).await.is_err() {
                    break;
                }
            }
        });

        if let Some(tx) = progress_tx {
            let _ = tx.send(AgentProgress::Complete).await;
        }

        Ok(QueryResponse::with_stream(all_docs, tokio_stream::wrappers::ReceiverStream::new(rx_stream)))
    }

    /// 質問からキーワードを抽出
    async fn extract_keywords(&self, question: &str) -> Result<Vec<String>> {
        let analysis_prompt = format!(
            "以下の質問に答えるために、どのような資料を検索すべきか分析してください。\n\
             検索キーワードを3つ提案してください（カンマ区切り）。\n\n\
             質問: {}\n\n\
             検索キーワード:",
            question
        );

        let keywords_response = self
            .pipeline
            .ollama_client
            .generate(&self.pipeline.llm_model, &analysis_prompt)
            .await?;

        // キーワードをパース
        let keywords: Vec<String> = keywords_response
            .split(',')
            .take(3)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // フォールバック: キーワードが抽出できない場合は元の質問を使用
        if keywords.is_empty() {
            Ok(vec![question.to_string()])
        } else {
            Ok(keywords)
        }
    }

    /// 資料が十分かどうかを検証
    async fn validate_sufficiency(
        &self,
        question: &str,
        documents: &[SearchResult],
    ) -> Result<bool> {
        if documents.is_empty() {
            return Ok(false);
        }

        let context = self.build_context(documents);
        let context_preview = if context.len() > 2000 {
            format!("{}...", &context[..2000])
        } else {
            context
        };

        let check_prompt = format!(
            "以下の資料を使って、この質問に答えられますか？\n\
             「はい」または「いいえ」だけで答えてください。\n\n\
             質問: {}\n\n\
             資料:\n{}\n\n\
             回答:",
            question, context_preview
        );

        let sufficiency = self
            .pipeline
            .ollama_client
            .generate(&self.pipeline.llm_model, &check_prompt)
            .await?;

        let sufficiency_lower = sufficiency.trim().to_lowercase();

        // "はい", "yes", "十分", "可能"などが含まれていればOK
        Ok(sufficiency_lower.contains("はい")
            || sufficiency_lower.contains("yes")
            || sufficiency_lower.contains("十分")
            || sufficiency_lower.contains("可能"))
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

    // TODO: モックを使ったテストを追加
}
