use async_trait::async_trait;
use docx_rs::*;
use std::path::Path;
use tokio::fs;

use crate::document::{Document, DocumentLoader, FileType, Metadata};
use crate::error::{RagError, Result};

/// DOCXファイルローダー
pub struct DocxLoader;

impl DocxLoader {
    pub fn new() -> Self {
        Self
    }

    /// DOCXからテキストを抽出
    fn extract_text_from_docx(bytes: &[u8]) -> Result<String> {
        let docx = read_docx(bytes).map_err(|e| {
            RagError::DocxParse(format!("Failed to parse DOCX: {:?}", e))
        })?;

        let mut text_content = Vec::new();

        // ドキュメントの本文を走査
        for child in docx.document.children {
            Self::extract_text_from_element(&child, &mut text_content);
        }

        Ok(text_content.join("\n"))
    }

    /// 要素からテキストを再帰的に抽出
    fn extract_text_from_element(element: &DocumentChild, text_content: &mut Vec<String>) {
        match element {
            DocumentChild::Paragraph(para) => {
                let mut para_text = Vec::new();
                for child in &para.children {
                    if let ParagraphChild::Run(run) = child {
                        for run_child in &run.children {
                            if let RunChild::Text(text) = run_child {
                                para_text.push(text.text.clone());
                            }
                        }
                    }
                }
                let combined = para_text.join("");
                if !combined.trim().is_empty() {
                    text_content.push(combined);
                }
            }
            DocumentChild::Table(_table) => {
                // テーブルのテキスト抽出は複雑なので、今のところスキップ
                // 必要に応じて後で実装
            }
            _ => {}
        }
    }
}

#[async_trait]
impl DocumentLoader for DocxLoader {
    async fn load(&self, path: &Path) -> Result<Vec<Document>> {
        // ファイル読み込み（非同期I/O）
        let bytes = fs::read(path).await.map_err(|e| {
            RagError::DocumentLoad(format!("Failed to read DOCX file {}: {}", path.display(), e))
        })?;

        let path_str = path.display().to_string();

        // DOCX解析とテキスト抽出（CPU-bound処理なのでspawn_blocking）
        let content = tokio::task::spawn_blocking(move || -> Result<String> {
            Self::extract_text_from_docx(&bytes)
        })
        .await
        .map_err(|e| RagError::TaskJoin(e))??;

        // 空チェック
        if content.trim().is_empty() {
            tracing::warn!("No text content found in DOCX: {}", path.display());
            return Ok(vec![]);
        }

        let metadata = Metadata::new(path.display().to_string(), FileType::Docx);

        Ok(vec![Document::new(content, metadata)])
    }

    fn supported_extensions(&self) -> &[&str] {
        &["docx"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_docx_loader_supports() {
        let loader = DocxLoader::new();
        assert!(loader.supports(Path::new("document.docx")));
        assert!(loader.supports(Path::new("FILE.DOCX")));
        assert!(!loader.supports(Path::new("document.pdf")));
    }
}
