use thiserror::Error;

#[derive(Error, Debug)]
pub enum RagError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Document loading error: {0}")]
    DocumentLoad(String),

    #[error("PDF parsing error: {0}")]
    PdfParse(String),

    #[error("DOCX parsing error: {0}")]
    DocxParse(String),

    #[error("XLSX parsing error: {0}")]
    XlsxParse(String),

    #[error("Text encoding error")]
    Encoding(String),

    #[error("Ollama API error: {0}")]
    OllamaApi(String),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Vector database error: {0}")]
    VectorDb(String),

    #[error("Embedding generation error: {0}")]
    Embedding(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Task join error: {0}")]
    TaskJoin(#[from] tokio::task::JoinError),

    #[error("Operation cancelled")]
    Cancelled,

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, RagError>;
