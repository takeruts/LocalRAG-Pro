use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;

use rag_core::{HnswClient, HnswConfig, OllamaClient, RagPipeline};

pub const DEFAULT_LLM_MODEL: &str = "gemma3:4b";
pub const DEFAULT_EMBEDDING_MODEL: &str = "bge-m3";
pub const HNSW_COLLECTION_NAME: &str = "localrag_collection";
pub const EMBEDDING_DIMENSION: usize = 1024;

/// Application state managed by Tauri
pub struct AppState {
    pub pipeline: Arc<Mutex<Option<RagPipeline<HnswClient>>>>,
    pub llm_model: Arc<RwLock<String>>,
    pub embedding_model: Arc<RwLock<String>>,
    pub folder_path: Arc<RwLock<Option<PathBuf>>>,
    pub cancel_token: Arc<Mutex<Option<CancellationToken>>>,
    pub is_indexing: Arc<RwLock<bool>>,
    pub data_dir: Arc<RwLock<PathBuf>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            pipeline: Arc::new(Mutex::new(None)),
            llm_model: Arc::new(RwLock::new(DEFAULT_LLM_MODEL.to_string())),
            embedding_model: Arc::new(RwLock::new(DEFAULT_EMBEDDING_MODEL.to_string())),
            folder_path: Arc::new(RwLock::new(None)),
            cancel_token: Arc::new(Mutex::new(None)),
            is_indexing: Arc::new(RwLock::new(false)),
            data_dir: Arc::new(RwLock::new(PathBuf::from("vectordb_data"))),
        }
    }

    pub fn set_data_dir(&self, path: PathBuf) {
        // Use blocking write since this is called during init
        let mut guard = self.data_dir.blocking_write();
        *guard = path;
    }

    pub async fn get_hnsw_config(&self) -> HnswConfig {
        let data_dir = self.data_dir.read().await.clone();
        HnswConfig::new(HNSW_COLLECTION_NAME, EMBEDDING_DIMENSION)
            .with_db_path(data_dir)
    }

    pub async fn create_pipeline(&self) -> RagPipeline<HnswClient> {
        let config = self.get_hnsw_config().await;
        let vector_db = HnswClient::new(config);
        let llm_model = self.llm_model.read().await.clone();
        let embedding_model = self.embedding_model.read().await.clone();

        RagPipeline::new(
            OllamaClient::default(),
            vector_db,
            embedding_model,
            llm_model,
        )
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Index statistics payload
#[derive(Clone, serde::Serialize)]
pub struct IndexStatsPayload {
    pub total_files: usize,
    pub indexed_files: usize,
    pub skipped_files: usize,
    pub total_chunks: usize,
    pub total_embeddings: usize,
    pub indexed_folder: Option<String>,
}

/// Index progress payload
#[derive(Clone, serde::Serialize)]
pub struct IndexProgressPayload {
    pub progress: f32,
    pub file: String,
    pub phase: String,      // "loading", "splitting", "embedding", "storing"
    pub current: usize,
    pub total: usize,
}

/// Source info payload
#[derive(Clone, serde::Serialize)]
pub struct SourceInfo {
    pub source: String,
    pub page: Option<usize>,
    pub score: f32,
}

/// Models payload
#[derive(Clone, serde::Serialize)]
pub struct ModelsPayload {
    pub llm_models: Vec<String>,
    pub embedding_models: Vec<String>,
}

/// Current models payload
#[derive(Clone, serde::Serialize)]
pub struct CurrentModelsPayload {
    pub llm_model: String,
    pub embedding_model: String,
}

/// Folder analysis info
#[derive(Clone, serde::Serialize)]
pub struct FolderAnalysis {
    pub folder: String,
    pub file_count: usize,
    pub chunk_count: usize,
}

/// Index analysis payload
#[derive(Clone, serde::Serialize)]
pub struct IndexAnalysisPayload {
    pub total_files: usize,
    pub total_chunks: usize,
    pub folders: Vec<FolderAnalysis>,
    pub files: Vec<FileAnalysis>,
}

/// File analysis info
#[derive(Clone, serde::Serialize)]
pub struct FileAnalysis {
    pub path: String,
    pub chunk_count: usize,
}
