use tauri::{AppHandle, Emitter, Manager};
use serde::Serialize;
use std::process::Command;
use sysinfo::System;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use rag_core::OllamaClient;

/// Windows flag to hide console window
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

use crate::state::{AppState, CurrentModelsPayload, ModelsPayload};

#[derive(Debug, Clone, Serialize)]
pub struct OllamaStatusInfo {
    pub installed: bool,
    pub running: bool,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemInfo {
    pub cpu_name: Option<String>,
    pub cpu_cores: Option<u32>,
    pub cpu_frequency_mhz: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FullSystemStatus {
    pub ollama: OllamaStatusInfo,
    pub system: SystemInfo,
}

/// Get all CPU info using sysinfo crate (no external commands needed)
fn get_cpu_info() -> SystemInfo {
    let sys = System::new_all();
    let cpus = sys.cpus();

    // Get CPU name from first CPU
    let cpu_name = cpus.first().map(|cpu| cpu.brand().to_string());

    // Get physical core count (this is a static function)
    let cpu_cores = System::physical_core_count().map(|c| c as u32);

    // Get CPU frequency (max frequency from first CPU, in MHz)
    let cpu_frequency_mhz = cpus.first().map(|cpu| cpu.frequency());

    SystemInfo {
        cpu_name,
        cpu_cores,
        cpu_frequency_mhz,
    }
}

/// Check if Ollama is installed by trying to run `ollama --version`
fn check_ollama_installed() -> (bool, Option<String>) {
    #[cfg(target_os = "windows")]
    let result = Command::new("cmd")
        .args(["/C", "ollama --version"])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    #[cfg(not(target_os = "windows"))]
    let result = Command::new("ollama")
        .arg("--version")
        .output();

    match result {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_string();
            (true, Some(version))
        }
        _ => (false, None),
    }
}

/// Start Ollama status checker
pub async fn start_ollama_checker(app: AppHandle) {
    let ollama = OllamaClient::default();

    // Track previous state to avoid flickering
    let mut last_emitted_running: Option<bool> = None;
    let mut consecutive_same_state = 0u32;

    // Require multiple consecutive same-state checks before changing
    const STATE_CHANGE_THRESHOLD: u32 = 2;

    // Check interval
    const CHECK_INTERVAL_SECS: u64 = 10;

    // Check installation status once at startup
    let (installed, version) = check_ollama_installed();

    // Wait a bit before first check to let the app initialize
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Initial check
    let initial_running = ollama.check_running().await;
    let initial_status = OllamaStatusInfo {
        installed,
        running: initial_running,
        version: version.clone(),
    };
    let _ = app.emit("ollama-status-info", &initial_status);
    let _ = app.emit("ollama-status", initial_running);
    last_emitted_running = Some(initial_running);
    tracing::info!("Initial Ollama status: installed={}, running={}", installed, initial_running);

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(CHECK_INTERVAL_SECS)).await;

        // Skip status check while indexing - Ollama is busy
        let state = app.state::<crate::state::AppState>();
        let is_indexing = *state.is_indexing.read().await;
        if is_indexing {
            consecutive_same_state = 0;
            continue;
        }

        let current_running = ollama.check_running().await;
        let last_state = last_emitted_running.unwrap_or(current_running);

        if current_running == last_state {
            // Same state as before, reset counter
            consecutive_same_state = 0;
        } else {
            // Different state, increment counter
            consecutive_same_state += 1;

            // Only change state after consecutive confirmations
            if consecutive_same_state >= STATE_CHANGE_THRESHOLD {
                let status = OllamaStatusInfo {
                    installed,
                    running: current_running,
                    version: version.clone(),
                };

                let _ = app.emit("ollama-status-info", &status);
                let _ = app.emit("ollama-status", current_running);

                last_emitted_running = Some(current_running);
                consecutive_same_state = 0;
                tracing::info!("Ollama status changed: running={}", current_running);
            }
        }
    }
}

/// Check Ollama status
#[tauri::command]
pub async fn check_ollama_status() -> Result<bool, String> {
    let ollama = OllamaClient::default();
    Ok(ollama.check_running().await)
}

/// Check Ollama installation and running status
#[tauri::command]
pub async fn check_ollama_status_info() -> Result<OllamaStatusInfo, String> {
    let ollama = OllamaClient::default();
    let is_running = ollama.check_running().await;
    let (installed, version) = check_ollama_installed();

    Ok(OllamaStatusInfo {
        installed,
        running: is_running,
        version,
    })
}

/// Get system information (CPU only)
#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    Ok(get_cpu_info())
}

/// Get full system status including Ollama and hardware info
#[tauri::command]
pub async fn get_full_system_status() -> Result<FullSystemStatus, String> {
    let ollama = OllamaClient::default();
    let is_running = ollama.check_running().await;
    let (installed, version) = check_ollama_installed();

    let ollama_status = OllamaStatusInfo {
        installed,
        running: is_running,
        version,
    };

    let system = get_cpu_info();

    Ok(FullSystemStatus {
        ollama: ollama_status,
        system,
    })
}

/// Refresh available models
#[tauri::command]
pub async fn refresh_models(app: AppHandle) -> Result<(), String> {
    let ollama = OllamaClient::default();

    // First check if Ollama is running
    if !ollama.check_running().await {
        // Ollama is not running, silently return without error
        tracing::debug!("Ollama is not running, skipping model refresh");
        return Ok(());
    }

    match ollama.list_models().await {
        Ok(models) => {
            // Separate models into LLM and embedding
            let embedding_prefixes = ["nomic-embed", "mxbai-embed", "all-minilm", "snowflake-arctic-embed", "bge-"];

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
            // Log the error but don't show it to the user if it's a connection error
            tracing::warn!("Failed to refresh models: {}", e);
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
