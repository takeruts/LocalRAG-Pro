pub mod recursive_character;

pub use recursive_character::RecursiveCharacterTextSplitter;

use crate::document::Document;

/// テキスト分割器トレイト
pub trait TextSplitter: Send + Sync {
    /// ドキュメントを分割
    fn split_documents(&self, documents: Vec<Document>) -> Vec<Document>;

    /// テキストを分割
    fn split_text(&self, text: &str) -> Vec<String>;
}
