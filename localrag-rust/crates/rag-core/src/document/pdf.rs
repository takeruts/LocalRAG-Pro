use async_trait::async_trait;
use lopdf::Document as PdfDocument;
use rayon::prelude::*;
use std::path::Path;
use tokio::fs;

use crate::document::{Document, DocumentLoader, FileType, Metadata};
use crate::error::{RagError, Result};

/// PDFファイルローダー
pub struct PdfLoader;

impl PdfLoader {
    pub fn new() -> Self {
        Self
    }

    /// PDF からテキストを抽出（並列処理）
    fn extract_text_from_pdf(pdf: &PdfDocument) -> Result<Vec<(usize, String)>> {
        let pages = pdf.get_pages();

        // Rayonで並列にページを処理
        let results: Vec<_> = pages
            .par_iter()
            .filter_map(|(page_num, page_id)| {
                match pdf.extract_text(&[page_id.0]) {
                    Ok(text) => {
                        if text.trim().is_empty() {
                            None
                        } else {
                            Some((*page_num as usize, text))
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to extract text from page {}: {}", page_num, e);
                        None
                    }
                }
            })
            .collect();

        Ok(results)
    }
}

#[async_trait]
impl DocumentLoader for PdfLoader {
    async fn load(&self, path: &Path) -> Result<Vec<Document>> {
        // ファイル読み込み（非同期I/O）
        let bytes = fs::read(path).await.map_err(|e| {
            RagError::DocumentLoad(format!("Failed to read PDF file {}: {}", path.display(), e))
        })?;

        // 正規化されたパスを使用（差分検出で一致させるため）
        let path_str = path.canonicalize()
            .map(|p| {
                let s = p.display().to_string();
                if s.starts_with(r"\\?\") { s[4..].to_string() } else { s }
            })
            .unwrap_or_else(|_| path.display().to_string());

        // PDF解析とテキスト抽出（CPU-bound処理なのでspawn_blocking）
        let documents = tokio::task::spawn_blocking(move || -> Result<Vec<Document>> {
            // PDFドキュメントロード
            let pdf = PdfDocument::load_mem(&bytes).map_err(|e| {
                RagError::PdfParse(format!("Failed to parse PDF: {}", e))
            })?;

            // 並列テキスト抽出
            let page_texts = Self::extract_text_from_pdf(&pdf)?;

            if page_texts.is_empty() {
                tracing::warn!("No text content found in PDF: {}", path_str);
                return Ok(vec![]);
            }

            // Documentオブジェクトに変換
            let documents = page_texts
                .into_iter()
                .map(|(page_num, text)| {
                    let metadata = Metadata::new(path_str.clone(), FileType::Pdf)
                        .with_page(page_num);
                    Document::new(text, metadata)
                })
                .collect();

            Ok(documents)
        })
        .await
        .map_err(|e| RagError::TaskJoin(e))??;

        Ok(documents)
    }

    fn supported_extensions(&self) -> &[&str] {
        &["pdf"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pdf_loader_supports() {
        let loader = PdfLoader::new();
        assert!(loader.supports(Path::new("document.pdf")));
        assert!(loader.supports(Path::new("FILE.PDF")));
        assert!(!loader.supports(Path::new("document.txt")));
    }
}
