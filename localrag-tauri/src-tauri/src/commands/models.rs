use tauri::{AppHandle, Emitter, Manager};

use rag_core::OllamaClient;

use crate::state::{AppState, CurrentModelsPayload, ModelsPayload};

/// Start Ollama status checker
pub async fn start_ollama_checker(app: AppHandle) {
    let ollama = OllamaClient::default();

    loop {
        let is_running = ollama.check_running().await;
        let _ = app.emit("ollama-status", is_running);

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

/// Check Ollama status
#[tauri::command]
pub async fn check_ollama_status() -> Result<bool, String> {
    let ollama = OllamaClient::default();
    Ok(ollama.check_running().await)
}

/// Refresh available models
#[tauri::command]
pub async fn refresh_models(app: AppHandle) -> Result<(), String> {
    let ollama = OllamaClient::default();

    match ollama.list_models().await {
        Ok(models) => {
            // Separate models into LLM and embedding
            let embedding_prefixes = ["nomic-embed", "mxbai-embed", "all-minilm", "snowflake-arctic-embed"];

            let mut llm_models = Vec::new();
            let mut embedding_models = Vec::new();

            for model in models {
                // ModelInfo has .name field
                let model_name = model.name.clone();
                let is_embedding = embedding_prefixes
                    .iter()
                    .any(|prefix| model_name.to_lowercase().starts_with(prefix));

                if is_embedding {
                    embedding_models.push(model_name);
                } else {
                    llm_models.push(model_name);
                }
            }

            let _ = app.emit(
                "models-refreshed",
                ModelsPayload {
                    llm_models,
                    embedding_models,
                },
            );

            Ok(())
        }
        Err(e) => {
            let _ = app.emit("error", format!("Failed to refresh models: {}", e));
            Err(e.to_string())
        }
    }
}

/// Set LLM model
#[tauri::command]
pub async fn set_llm_model(app: AppHandle, model: String) -> Result<(), String> {
    let state = app.state::<AppState>();
    *state.llm_model.write().await = model.clone();

    // Recreate pipeline with new model
    let new_pipeline = state.create_pipeline().await;
    *state.pipeline.lock().await = Some(new_pipeline);

    tracing::info!("LLM model changed to: {}", model);
    Ok(())
}

/// Set embedding model
#[tauri::command]
pub async fn set_embedding_model(app: AppHandle, model: String) -> Result<(), String> {
    let state = app.state::<AppState>();
    *state.embedding_model.write().await = model.clone();

    // Recreate pipeline with new model
    let new_pipeline = state.create_pipeline().await;
    *state.pipeline.lock().await = Some(new_pipeline);

    tracing::info!("Embedding model changed to: {}", model);
    Ok(())
}

/// Get current models
#[tauri::command]
pub async fn get_current_models(app: AppHandle) -> Result<CurrentModelsPayload, String> {
    let state = app.state::<AppState>();

    let llm_model = state.llm_model.read().await.clone();
    let embedding_model = state.embedding_model.read().await.clone();

    Ok(CurrentModelsPayload {
        llm_model,
        embedding_model,
    })
}
