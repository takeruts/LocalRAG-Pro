use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use rag_core::{HnswClient, IndexProgress, VectorDatabase};

use crate::state::{AppState, FileAnalysis, FolderAnalysis, IndexAnalysisPayload, IndexProgressPayload, IndexStatsPayload};

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

    let state = app.state::<AppState>();
    let config = state.get_hnsw_config().await;
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
    tracing::info!("start_indexing called with folder: {}", folder);
    let state = app.state::<AppState>();

    // Check if already indexing
    if *state.is_indexing.read().await {
        tracing::warn!("Already indexing, returning early");
        return Err("Already indexing".to_string());
    }

    let folder_path = PathBuf::from(&folder);
    if !folder_path.exists() {
        tracing::error!("Folder does not exist: {}", folder);
        return Err("Folder does not exist".to_string());
    }

    tracing::info!("Starting indexing for folder: {}", folder);

    // Set indexing flag
    *state.is_indexing.write().await = true;

    // Create cancellation token
    let cancel_token = CancellationToken::new();
    *state.cancel_token.lock().await = Some(cancel_token.clone());

    // Create pipeline
    tracing::info!("Creating pipeline...");
    let pipeline = state.create_pipeline().await;
    tracing::info!("Pipeline created successfully");

    // Store pipeline
    *state.pipeline.lock().await = Some(pipeline);

    let app_handle = app.clone();
    let folder_clone = folder.clone();
    let folder_for_progress = folder.clone();

    tauri::async_runtime::spawn(async move {
        let state = app_handle.state::<AppState>();

        // Create progress channel
        let (progress_tx, mut progress_rx) = tokio::sync::mpsc::channel(100);

        // Progress reporter task
        let app_for_progress = app_handle.clone();
        let progress_task = tauri::async_runtime::spawn(async move {
            while let Some(progress) = progress_rx.recv().await {
                match progress {
                    IndexProgress::Scanning { current, total } => {
                        let _ = app_for_progress.emit(
                            "index-progress",
                            IndexProgressPayload {
                                progress: 0.0,
                                file: if total == 0 {
                                    "Scanning files...".to_string()
                                } else {
                                    format!("Found {} files", total)
                                },
                                phase: "loading".to_string(),
                                current,
                                total,
                            },
                        );
                    }
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
                                phase: "loading".to_string(),
                                current,
                                total,
                            },
                        );
                    }
                    IndexProgress::Splitting { current, total } => {
                        let progress_pct = if total > 0 {
                            current as f32 / total as f32
                        } else {
                            0.0
                        };

                        let _ = app_for_progress.emit(
                            "index-progress",
                            IndexProgressPayload {
                                progress: progress_pct,
                                file: "Splitting documents...".to_string(),
                                phase: "splitting".to_string(),
                                current,
                                total,
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
                                file: format!("{}/{} chunks", current, total),
                                phase: "embedding".to_string(),
                                current,
                                total,
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
                                file: format!("{}/{} files", current, total),
                                phase: "storing".to_string(),
                                current,
                                total,
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
                                indexed_folder: Some(folder_for_progress.clone()),
                            },
                        );
                    }
                    IndexProgress::Complete { .. } => {}
                }
            }
        });

        // Index directory
        let pipeline_guard = state.pipeline.lock().await;
        if let Some(pipeline) = &*pipeline_guard {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    tracing::info!("Indexing cancelled - fetching stats from disk");

                    // キャンセル時はディスクから統計を取得
                    // （パイプラインのメモリ上のデータではなく、実際に保存されたデータを取得）
                    let config = state.get_hnsw_config().await;
                    let fresh_db = HnswClient::new(config);

                    let (db_count, file_count, indexed_folder) = match fresh_db.count().await {
                        Ok(count) => {
                            tracing::info!("Cancelled - disk DB count: {}", count);
                            let sources = fresh_db.get_indexed_sources().await.unwrap_or_default();
                            let file_count = sources.len();
                            let folder = derive_common_folder(&sources);
                            tracing::info!("Cancelled - disk file count: {}, folder: {:?}", file_count, folder);
                            (count, file_count, folder)
                        }
                        Err(e) => {
                            tracing::warn!("Cancelled - failed to get disk DB count: {}", e);
                            (0, 0, None)
                        }
                    };

                    tracing::info!("Emitting index-complete with db_count={}, file_count={}", db_count, file_count);

                    // 常にindex-completeを発行して統計情報を更新
                    // indexed_folder はディスクのデータから取得、なければ元のフォルダを使用
                    let emit_result = app_handle.emit(
                        "index-complete",
                        IndexStatsPayload {
                            total_files: file_count,
                            indexed_files: file_count,
                            skipped_files: 0,
                            total_chunks: db_count,
                            total_embeddings: db_count,
                            indexed_folder: indexed_folder.or_else(|| Some(folder_clone.clone())),
                        },
                    );
                    tracing::info!("index-complete event emit result: {:?}", emit_result);
                }
                result = pipeline.index_directory(&folder_path, Some(progress_tx)) => {
                    match result {
                        Ok(stats) => {
                            tracing::info!("Indexing completed successfully: {} files, {} chunks", stats.indexed_files, stats.total_chunks);

                            // Emit completion event immediately with stats from pipeline
                            // (skip slow DB count query to avoid UI delay)
                            let _ = app_handle.emit(
                                "index-complete",
                                IndexStatsPayload {
                                    total_files: stats.total_files,
                                    indexed_files: stats.indexed_files,
                                    skipped_files: stats.skipped_files,
                                    total_chunks: stats.total_chunks,
                                    total_embeddings: stats.total_embeddings,
                                    indexed_folder: Some(folder_clone),
                                },
                            );
                            tracing::info!("index-complete event emitted");
                        }
                        Err(e) => {
                            tracing::error!("Indexing error: {}", e);
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

/// Clear all indexed data
#[tauri::command]
pub async fn clear_index(app: AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();

    // Check if indexing is in progress
    if *state.is_indexing.read().await {
        return Err("Cannot clear index while indexing is in progress".to_string());
    }

    let config = state.get_hnsw_config().await;
    let vector_db = HnswClient::new(config);

    vector_db
        .delete_collection()
        .await
        .map_err(|e| format!("Failed to clear index: {}", e))?;

    tracing::info!("Index cleared successfully");

    // Emit event to update UI
    let _ = app.emit("index-cleared", ());

    Ok(())
}

/// Get current index statistics
#[tauri::command]
pub async fn get_index_stats(app: AppHandle) -> Result<Option<IndexStatsPayload>, String> {
    tracing::info!("get_index_stats called");
    let state = app.state::<AppState>();
    let config = state.get_hnsw_config().await;
    tracing::info!("get_index_stats: db_path={:?}", config.db_path);
    let vector_db = HnswClient::new(config);

    match vector_db.count().await {
        Ok(count) => {
            tracing::info!("get_index_stats: count={}", count);
            let sources = vector_db.get_indexed_sources().await.unwrap_or_default();
            let file_count = sources.len();
            let indexed_folder = derive_common_folder(&sources);
            tracing::info!("get_index_stats: file_count={}, indexed_folder={:?}", file_count, indexed_folder);

            // count が0でも、以前インデックスがあった場合は統計を返す
            // UIで「インデックスがない」ことを示すために None を返す
            if count > 0 || file_count > 0 {
                Ok(Some(IndexStatsPayload {
                    total_files: file_count,
                    indexed_files: file_count,
                    skipped_files: 0,
                    total_chunks: count,
                    total_embeddings: count,
                    indexed_folder,
                }))
            } else {
                tracing::info!("get_index_stats: no documents found");
                Ok(None)
            }
        }
        Err(e) => {
            tracing::error!("get_index_stats error: {}", e);
            Err(format!("Failed to get index stats: {}", e))
        }
    }
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

/// Analyze index contents
#[tauri::command]
pub async fn analyze_index(app: AppHandle) -> Result<IndexAnalysisPayload, String> {
    use std::collections::HashMap;

    tracing::info!("analyze_index called");
    let state = app.state::<AppState>();
    let config = state.get_hnsw_config().await;
    let vector_db = HnswClient::new(config);

    let sources = vector_db
        .get_indexed_sources()
        .await
        .map_err(|e| format!("Failed to get indexed sources: {}", e))?;

    // Count chunks per file
    let chunk_counts = vector_db
        .get_chunk_counts_per_source()
        .await
        .unwrap_or_default();

    // Group by folder
    let mut folder_map: HashMap<String, (usize, usize)> = HashMap::new(); // (file_count, chunk_count)
    let mut file_analyses: Vec<FileAnalysis> = Vec::new();

    for source in &sources {
        let path = PathBuf::from(source);
        let folder = path
            .parent()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let chunk_count = chunk_counts.get(source).copied().unwrap_or(0);

        // Add to folder map
        let entry = folder_map.entry(folder).or_insert((0, 0));
        entry.0 += 1; // file count
        entry.1 += chunk_count; // chunk count

        // Add file analysis
        file_analyses.push(FileAnalysis {
            path: source.clone(),
            chunk_count,
        });
    }

    // Convert to folder analyses
    let mut folders: Vec<FolderAnalysis> = folder_map
        .into_iter()
        .map(|(folder, (file_count, chunk_count))| FolderAnalysis {
            folder,
            file_count,
            chunk_count,
        })
        .collect();

    // Sort by file count descending
    folders.sort_by(|a, b| b.file_count.cmp(&a.file_count));

    // Sort files by path
    file_analyses.sort_by(|a, b| a.path.cmp(&b.path));

    let total_chunks = vector_db.count().await.unwrap_or(0);

    Ok(IndexAnalysisPayload {
        total_files: sources.len(),
        total_chunks,
        folders,
        files: file_analyses,
    })
}
