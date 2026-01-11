mod commands;
mod state;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter("localrag_pro=debug,rag_core=info")
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Initialize app state with app data directory
            let state = AppState::new();

            // Set up data directory in app's local data folder
            if let Some(app_data_dir) = app.path().app_local_data_dir().ok() {
                let vectordb_path = app_data_dir.join("vectordb_data");
                tracing::info!("Using vectordb path: {:?}", vectordb_path);

                // Create directory if it doesn't exist
                if let Err(e) = std::fs::create_dir_all(&vectordb_path) {
                    tracing::warn!("Failed to create vectordb directory: {}", e);
                }

                state.set_data_dir(vectordb_path);
            } else {
                tracing::warn!("Could not get app data directory, using default");
            }

            app.manage(state);

            // Start background tasks
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                commands::models::start_ollama_checker(app_handle).await;
            });

            // Load existing index on startup
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                commands::indexing::load_existing_index(app_handle).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::indexing::select_folder,
            commands::indexing::start_indexing,
            commands::indexing::stop_indexing,
            commands::query::send_query,
            commands::models::refresh_models,
            commands::models::set_llm_model,
            commands::models::set_embedding_model,
            commands::models::get_current_models,
            commands::models::check_ollama_status,
            commands::models::check_ollama_status_info,
            commands::models::get_system_info,
            commands::models::get_full_system_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
