use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use rag_core::{HnswClient, HnswConfig, IndexProgress, VectorDatabase};

use crate::state::{
    AppState, IndexProgressPayload, IndexStatsPayload, EMBEDDING_DIMENSION, HNSW_COLLECTION_NAME,
};

/// Select a folder using native dialog
#[tauri::command]
pub async fn select_folder(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::{DialogExt, FilePath};

    let (tx, rx) = oneshot::channel();

    app.dialog()
        .file()
        .set_title("Select Document Folder")
        .pick_folder(move |result: Option<FilePath>| {
            let _ = tx.send(result);
        });

    // Wait for dialog result
    match rx.await {
        Ok(Some(path)) => {
            let path_str = path.to_string();

            // Update state
            let state = app.state::<AppState>();
            *state.folder_path.write().await = Some(PathBuf::from(&path_str));

            // Emit event
            let _ = app.emit("folder-selected", &path_str);

            Ok(Some(path_str))
        }
        Ok(None) => Ok(None),
        Err(_) => Err("Dialog cancelled".to_string()),
    }
}

/// Load existing index on startup
pub async fn load_existing_index(app: AppHandle) {
    tracing::info!("Checking for existing index...");

    let config = HnswConfig::new(HNSW_COLLECTION_NAME, EMBEDDING_DIMENSION);
    let vector_db = HnswClient::new(config);

    match vector_db.count().await {
        Ok(count) if count > 0 => {
            tracing::info!("Found existing index with {} documents", count);

            let sources = vector_db.get_indexed_sources().await.unwrap_or_default();
            let file_count = sources.len();

            // Derive common folder
            let indexed_folder = derive_common_folder(&sources);

            let stats = IndexStatsPayload {
                total_files: file_count,
                indexed_files: file_count,
                skipped_files: 0,
                total_chunks: count,
                total_embeddings: count,
                indexed_folder,
            };

            let _ = app.emit("index-complete", stats);
        }
        Ok(_) => {
            tracing::info!("No existing index found");
        }
        Err(e) => {
            tracing::warn!("Failed to check existing index: {}", e);
        }
    }
}

/// Start indexing documents
#[tauri::command]
pub async fn start_indexing(app: AppHandle, folder: String) -> Result<(), String> {
    let state = app.state::<AppState>();

    // Check if already indexing
    if *state.is_indexing.read().await {
        return Err("Already indexing".to_string());
    }

    let folder_path = PathBuf::from(&folder);
    if !folder_path.exists() {
        return Err("Folder does not exist".to_string());
    }

    // Set indexing flag
    *state.is_indexing.write().await = true;

    // Create cancellation token
    let cancel_token = CancellationToken::new();
    *state.cancel_token.lock().await = Some(cancel_token.clone());

    // Create pipeline
    let pipeline = state.create_pipeline().await;

    // Store pipeline
    *state.pipeline.lock().await = Some(pipeline);

    let app_handle = app.clone();
    let folder_clone = folder.clone();

    tauri::async_runtime::spawn(async move {
        let state = app_handle.state::<AppState>();

        // Create progress channel
        let (progress_tx, mut progress_rx) = tokio::sync::mpsc::channel(100);

        // Progress reporter task
        let app_for_progress = app_handle.clone();
        let progress_task = tauri::async_runtime::spawn(async move {
            while let Some(progress) = progress_rx.recv().await {
                match progress {
                    IndexProgress::Loading { file, current, total } => {
                        let progress_pct = if total > 0 {
                            current as f32 / total as f32
                        } else {
                            0.0
                        };

                        let _ = app_for_progress.emit(
                            "index-progress",
                            IndexProgressPayload {
                                progress: progress_pct,
                                file: file.display().to_string(),
                            },
                        );
                    }
                    IndexProgress::Embedding { current, total } => {
                        let progress_pct = if total > 0 {
                            current as f32 / total as f32
                        } else {
                            0.0
                        };

                        let _ = app_for_progress.emit(
                            "index-progress",
                            IndexProgressPayload {
                                progress: progress_pct,
                                file: format!("Embedding: {}/{}", current, total),
                            },
                        );
                    }
                    IndexProgress::Storing { current, total } => {
                        let progress_pct = if total > 0 {
                            current as f32 / total as f32
                        } else {
                            0.0
                        };

                        let _ = app_for_progress.emit(
                            "index-progress",
                            IndexProgressPayload {
                                progress: progress_pct,
                                file: format!("Storing: {}/{}", current, total),
                            },
                        );
                    }
                    IndexProgress::BatchComplete { stats } => {
                        let _ = app_for_progress.emit(
                            "index-stats-update",
                            IndexStatsPayload {
                                total_files: stats.total_files,
                                indexed_files: stats.indexed_files,
                                skipped_files: stats.skipped_files,
                                total_chunks: stats.total_chunks,
                                total_embeddings: stats.total_embeddings,
                                indexed_folder: None,
                            },
                        );
                    }
                    IndexProgress::Complete { .. } | IndexProgress::Scanning { .. } | IndexProgress::Splitting { .. } => {}
                }
            }
        });

        // Index directory
        let pipeline_guard = state.pipeline.lock().await;
        if let Some(pipeline) = &*pipeline_guard {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    tracing::info!("Indexing cancelled");
                    let _ = app_handle.emit("indexing-cancelled", ());
                }
                result = pipeline.index_directory(&folder_path, Some(progress_tx)) => {
                    match result {
                        Ok(stats) => {
                            // Get actual count from DB
                            let db_count = pipeline.count().await.unwrap_or(stats.total_embeddings);

                            let _ = app_handle.emit(
                                "index-complete",
                                IndexStatsPayload {
                                    total_files: stats.total_files,
                                    indexed_files: stats.indexed_files,
                                    skipped_files: stats.skipped_files,
                                    total_chunks: stats.total_chunks,
                                    total_embeddings: db_count,
                                    indexed_folder: Some(folder_clone),
                                },
                            );
                        }
                        Err(e) => {
                            let _ = app_handle.emit("error", format!("Indexing error: {}", e));
                        }
                    }
                }
            }
        }
        drop(pipeline_guard);

        // Wait for progress task
        let _ = progress_task.await;

        // Reset state
        *state.is_indexing.write().await = false;
        *state.cancel_token.lock().await = None;
    });

    Ok(())
}

/// Stop indexing
#[tauri::command]
pub async fn stop_indexing(app: AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();

    if let Some(token) = state.cancel_token.lock().await.as_ref() {
        token.cancel();
    }

    Ok(())
}

/// Derive common folder from source paths
fn derive_common_folder(sources: &[String]) -> Option<String> {
    if sources.is_empty() {
        return None;
    }

    let paths: Vec<PathBuf> = sources.iter().map(PathBuf::from).collect();
    let first_parent = paths[0].parent()?;
    let mut common = first_parent.to_path_buf();

    for path in &paths[1..] {
        if let Some(parent) = path.parent() {
            let mut new_common = PathBuf::new();
            for (a, b) in common.components().zip(parent.components()) {
                if a == b {
                    new_common.push(a);
                } else {
                    break;
                }
            }
            common = new_common;
        }
    }

    if common.as_os_str().is_empty() {
        None
    } else {
        Some(common.display().to_string())
    }
}
