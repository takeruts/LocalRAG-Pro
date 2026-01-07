use tauri::{AppHandle, Emitter, Manager};
use tokio_stream::StreamExt;

use rag_core::{AgentPipeline, AgentProgress, HnswClient, HnswConfig, OllamaClient, RagPipeline};

use crate::state::{
    AppState, SourceInfo, EMBEDDING_DIMENSION, HNSW_COLLECTION_NAME,
};

/// Send a query to the RAG system
#[tauri::command]
pub async fn send_query(
    app: AppHandle,
    question: String,
    agent_mode: bool,
) -> Result<(), String> {
    let state = app.state::<AppState>();

    // Create independent pipeline for query
    let config = HnswConfig::new(HNSW_COLLECTION_NAME, EMBEDDING_DIMENSION);
    let vector_db = HnswClient::new(config);

    let llm_model = state.llm_model.read().await.clone();
    let embedding_model = state.embedding_model.read().await.clone();

    let query_pipeline = RagPipeline::new(
        OllamaClient::default(),
        vector_db,
        embedding_model,
        llm_model,
    );

    // Check if index has documents
    match query_pipeline.count().await {
        Ok(count) if count == 0 => {
            let _ = app.emit("error", "Please index documents first");
            return Ok(());
        }
        Err(e) => {
            let _ = app.emit("error", format!("Index check error: {}", e));
            return Ok(());
        }
        _ => {}
    }

    let app_handle = app.clone();

    if agent_mode {
        let agent = AgentPipeline::new(query_pipeline);

        tauri::async_runtime::spawn(async move {
            let (progress_tx, mut progress_rx) = tokio::sync::mpsc::channel(100);

            // Progress reporter
            let app_for_progress = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                while let Some(progress) = progress_rx.recv().await {
                    let msg = match progress {
                        AgentProgress::Analyzing => "Analyzing question...".to_string(),
                        AgentProgress::Keywords(keywords) => {
                            format!("Keywords: {}", keywords.join(", "))
                        }
                        AgentProgress::Searching(keyword) => {
                            format!("Searching: {}", keyword)
                        }
                        AgentProgress::Found(count) => {
                            format!("Found {} documents", count)
                        }
                        AgentProgress::ValidatingSufficiency => {
                            "Validating sufficiency...".to_string()
                        }
                        AgentProgress::Generating => "Generating answer...".to_string(),
                        AgentProgress::Complete => "Complete".to_string(),
                    };
                    let _ = app_for_progress.emit("agent-progress", msg);
                }
            });

            match agent.query_agent(&question, Some(progress_tx)).await {
                Ok(response) => {
                    // Stream response
                    if let Some(stream) = response.stream {
                        let mut stream = stream;
                        while let Some(chunk) = stream.next().await {
                            if let Ok(text) = chunk {
                                let _ = app_handle.emit("query-chunk", text);
                            }
                        }
                    } else if let Some(answer) = response.answer {
                        let _ = app_handle.emit("query-chunk", answer);
                    }

                    // Send sources
                    let sources: Vec<SourceInfo> = response
                        .sources
                        .into_iter()
                        .map(|s| SourceInfo {
                            source: s.source().unwrap_or("Unknown").to_string(),
                            page: s.page(),
                            score: s.score(),
                        })
                        .collect();

                    let _ = app_handle.emit("query-complete", sources);
                }
                Err(e) => {
                    let _ = app_handle.emit("error", format!("Query failed: {}", e));
                }
            }
        });
    } else {
        tauri::async_runtime::spawn(async move {
            match query_pipeline.query_stream(&question, 5).await {
                Ok(response) => {
                    // Stream response
                    if let Some(mut stream) = response.stream {
                        while let Some(chunk) = stream.next().await {
                            if let Ok(text) = chunk {
                                let _ = app_handle.emit("query-chunk", text);
                            }
                        }
                    } else if let Some(answer) = response.answer {
                        let _ = app_handle.emit("query-chunk", answer);
                    }

                    // Send sources
                    let sources: Vec<SourceInfo> = response
                        .sources
                        .into_iter()
                        .map(|s| SourceInfo {
                            source: s.source().unwrap_or("Unknown").to_string(),
                            page: s.page(),
                            score: s.score(),
                        })
                        .collect();

                    let _ = app_handle.emit("query-complete", sources);
                }
                Err(e) => {
                    let _ = app_handle.emit("error", format!("Query failed: {}", e));
                }
            }
        });
    }

    Ok(())
}
