pub mod error;
pub mod document;
pub mod ollama;
pub mod splitter;
pub mod vectordb;
pub mod rag;

pub use error::{RagError, Result};
pub use document::{Document, Metadata, FileType, DocumentLoader, LoadProgress};
pub use ollama::{OllamaClient, ModelInfo, ChatMessage};
pub use splitter::{RecursiveCharacterTextSplitter, TextSplitter};
pub use vectordb::{VectorDatabase, ChromaClient, HnswClient, HnswConfig, VectorDbConfig, SearchResult};
pub use rag::{RagPipeline, AgentPipeline, IndexProgress, IndexStats, QueryResponse, AgentProgress};
