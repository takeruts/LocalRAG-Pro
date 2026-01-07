use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;

use rag_core::{HnswClient, HnswConfig, OllamaClient, RagPipeline};

pub const DEFAULT_LLM_MODEL: &str = "gemma3:4b";
pub const DEFAULT_EMBEDDING_MODEL: &str = "nomic-embed-text";
pub const HNSW_COLLECTION_NAME: &str = "localrag_collection";
pub const EMBEDDING_DIMENSION: usize = 768;

/// Application state managed by Tauri
pub struct AppState {
    pub pipeline: Arc<Mutex<Option<RagPipeline<HnswClient>>>>,
    pub llm_model: Arc<RwLock<String>>,
    pub embedding_model: Arc<RwLock<String>>,
    pub folder_path: Arc<RwLock<Option<PathBuf>>>,
    pub cancel_token: Arc<Mutex<Option<CancellationToken>>>,
    pub is_indexing: Arc<RwLock<bool>>,
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
        }
    }

    pub async fn create_pipeline(&self) -> RagPipeline<HnswClient> {
        let config = HnswConfig::new(HNSW_COLLECTION_NAME, EMBEDDING_DIMENSION);
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
