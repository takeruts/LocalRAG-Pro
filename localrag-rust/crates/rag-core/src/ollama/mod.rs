pub mod client;
pub mod types;

pub use client::OllamaClient;
pub use types::{
    ModelInfo, EmbedRequest, EmbedResponse, GenerateRequest, GenerateResponse,
    ChatMessage, OllamaError,
};
