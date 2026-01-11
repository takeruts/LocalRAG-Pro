use async_trait::async_trait;
use calamine::{open_workbook_auto_from_rs, DataType, Reader, Sheets};
use std::io::Cursor;
use std::path::Path;
use tokio::fs;

use crate::document::{Document, DocumentLoader, FileType, Metadata};
use crate::error::{RagError, Result};

/// XLSXファイルローダー
pub struct XlsxLoader;

impl XlsxLoader {
    pub fn new() -> Self {
        Self
    }

    /// XLSXからテキストを抽出
    fn extract_text_from_xlsx(bytes: &[u8]) -> Result<String> {
        let cursor = Cursor::new(bytes);

        let mut workbook: Sheets<_> = open_workbook_auto_from_rs(cursor).map_err(|e| {
            RagError::XlsxParse(format!("Failed to parse XLSX: {:?}", e))
        })?;

        let mut text_content = Vec::new();

        // 全シートを走査
        for sheet_name in workbook.sheet_names().to_owned() {
            if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                // シート名を追加
                text_content.push(format!("=== Sheet: {} ===", sheet_name));

                // 全セルを走査
                for row in range.rows() {
                    let mut row_text = Vec::new();
                    for cell in row {
                        let cell_str = cell.as_string().unwrap_or_default();

                        if !cell_str.trim().is_empty() {
                            row_text.push(cell_str);
                        }
                    }

                    if !row_text.is_empty() {
                        text_content.push(row_text.join("\t"));
                    }
                }

                text_content.push(String::new()); // シート間の空行
            }
        }

        Ok(text_content.join("\n"))
    }
}

#[async_trait]
impl DocumentLoader for XlsxLoader {
    async fn load(&self, path: &Path) -> Result<Vec<Document>> {
        // ファイル読み込み（非同期I/O）
        let bytes = fs::read(path).await.map_err(|e| {
            RagError::DocumentLoad(format!("Failed to read XLSX file {}: {}", path.display(), e))
        })?;

        // 正規化されたパスを使用（差分検出で一致させるため）
        let path_str = path.canonicalize()
            .map(|p| {
                let s = p.display().to_string();
                if s.starts_with(r"\\?\") { s[4..].to_string() } else { s }
            })
            .unwrap_or_else(|_| path.display().to_string());

        // XLSX解析とテキスト抽出（CPU-bound処理なのでspawn_blocking）
        let content = tokio::task::spawn_blocking(move || -> Result<String> {
            Self::extract_text_from_xlsx(&bytes)
        })
        .await
        .map_err(|e| RagError::TaskJoin(e))??;

        // 空チェック
        if content.trim().is_empty() {
            tracing::warn!("No text content found in XLSX: {}", path.display());
            return Ok(vec![]);
        }

        // 正規化されたパスを使用（差分検出で一致させるため）
        let normalized_path = path.canonicalize()
            .map(|p| {
                let s = p.display().to_string();
                // Windows拡張パスプレフィックスを削除
                if s.starts_with(r"\\?\") { s[4..].to_string() } else { s }
            })
            .unwrap_or_else(|_| path_str);
        let metadata = Metadata::new(normalized_path, FileType::Xlsx);

        Ok(vec![Document::new(content, metadata)])
    }

    fn supported_extensions(&self) -> &[&str] {
        &["xlsx", "xls"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_xlsx_loader_supports() {
        let loader = XlsxLoader::new();
        assert!(loader.supports(Path::new("document.xlsx")));
        assert!(loader.supports(Path::new("FILE.XLS")));
        assert!(loader.supports(Path::new("data.XLSX")));
        assert!(!loader.supports(Path::new("document.pdf")));
    }
}
