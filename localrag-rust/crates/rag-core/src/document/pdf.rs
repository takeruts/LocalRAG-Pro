use async_trait::async_trait;
use pdfium_render::prelude::*;
use std::path::Path;
use tokio::fs;

use crate::document::{Document, DocumentLoader, FileType, Metadata};
use crate::error::{RagError, Result};

/// PDFファイルローダー
/// pdfium-renderを使用、日本語フォントに対応
pub struct PdfLoader;

impl PdfLoader {
    pub fn new() -> Self {
        Self
    }

    /// PDFiumを初期化して取得
    fn create_pdfium() -> Result<Pdfium> {
        // 複数の場所からpdfium.dllを探す
        let search_paths = Self::get_pdfium_search_paths();

        let mut last_error = String::new();

        for path in &search_paths {
            tracing::debug!("Searching for PDFium at: {}", path);
            match Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(path)) {
                Ok(bindings) => {
                    tracing::info!("Successfully loaded PDFium from: {}", path);
                    return Ok(Pdfium::new(bindings));
                }
                Err(e) => {
                    last_error = e.to_string();
                    tracing::debug!("Failed to load PDFium from {}: {}", path, e);
                }
            }
        }

        // システムライブラリを試す
        match Pdfium::bind_to_system_library() {
            Ok(bindings) => {
                tracing::info!("Successfully loaded PDFium from system library");
                return Ok(Pdfium::new(bindings));
            }
            Err(e) => {
                last_error = e.to_string();
            }
        }

        Err(RagError::DocumentLoad(format!(
            "Failed to load PDFium library: {}. Searched paths: {:?}",
            last_error, search_paths
        )))
    }

    /// PDFiumを検索するパスのリストを取得
    fn get_pdfium_search_paths() -> Vec<String> {
        let mut paths = Vec::new();

        // 1. 実行ファイルのディレクトリ
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                paths.push(exe_dir.display().to_string());
            }
        }

        // 2. カレントディレクトリ
        if let Ok(cwd) = std::env::current_dir() {
            paths.push(cwd.display().to_string());
        }

        // 3. 相対パス
        paths.push("./".to_string());
        paths.push(".".to_string());

        paths
    }

    /// PDF からテキストを抽出
    fn extract_text_from_pdf(pdfium: &Pdfium, bytes: &[u8]) -> Result<Vec<(usize, String)>> {
        tracing::info!("Loading PDF from {} bytes", bytes.len());
        let document = pdfium.load_pdf_from_byte_slice(bytes, None)
            .map_err(|e| {
                tracing::error!("Failed to load PDF: {}", e);
                RagError::DocumentLoad(format!("Failed to load PDF: {}", e))
            })?;
        tracing::info!("PDF loaded successfully, {} pages", document.pages().len());

        let mut results = Vec::new();

        for (page_index, page) in document.pages().iter().enumerate() {
            let text = page.text()
                .map_err(|e| {
                    tracing::debug!("Failed to get text object for page {}: {}", page_index + 1, e);
                    e
                })
                .ok()
                .map(|text_page| text_page.all())
                .unwrap_or_default();

            let cleaned = Self::clean_text(&text);
            if !cleaned.is_empty() {
                results.push((page_index, cleaned));
            }
        }

        Ok(results)
    }

    /// テキストをクリーンアップ
    fn clean_text(text: &str) -> String {
        // 連続する空白・改行を整理
        let cleaned: String = text
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        cleaned
    }
}

impl Default for PdfLoader {
    fn default() -> Self {
        Self::new()
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
        tracing::info!("Starting PDF processing for: {}", path_str);
        let documents = tokio::task::spawn_blocking(move || -> Result<Vec<Document>> {
            // PDFiumを作成（各呼び出しごとに作成）
            tracing::info!("Creating PDFium instance...");
            let pdfium = match Self::create_pdfium() {
                Ok(p) => {
                    tracing::info!("PDFium instance created successfully");
                    p
                }
                Err(e) => {
                    tracing::error!("Failed to create PDFium: {}", e);
                    return Err(e);
                }
            };

            // テキスト抽出
            let page_texts = match Self::extract_text_from_pdf(&pdfium, &bytes) {
                Ok(texts) => texts,
                Err(e) => {
                    tracing::warn!("Failed to extract text from PDF {}: {}", path_str, e);
                    return Ok(vec![]);
                }
            };

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

    #[test]
    fn test_clean_text() {
        let text = "Hello  World\n\n  Test  ";
        let cleaned = PdfLoader::clean_text(text);
        assert_eq!(cleaned, "Hello  World\nTest");
    }
}
