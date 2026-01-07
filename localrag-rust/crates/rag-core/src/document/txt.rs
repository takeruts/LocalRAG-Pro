use async_trait::async_trait;
use encoding_rs::{Encoding, UTF_8, SHIFT_JIS, EUC_JP};
use std::path::Path;
use tokio::fs;

use crate::document::{Document, DocumentLoader, FileType, Metadata};
use crate::error::{RagError, Result};

/// TXTファイルローダー
pub struct TxtLoader;

impl TxtLoader {
    pub fn new() -> Self {
        Self
    }

    /// 文字エンコーディングを自動検出
    fn detect_encoding(bytes: &[u8]) -> &'static Encoding {
        // BOM検出
        if bytes.len() >= 3 {
            if &bytes[0..3] == b"\xEF\xBB\xBF" {
                return UTF_8;
            }
        }

        // UTF-8として解釈を試みる
        if std::str::from_utf8(bytes).is_ok() {
            return UTF_8;
        }

        // Shift_JIS を試す
        let (decoded, encoding_used, had_errors) = SHIFT_JIS.decode(bytes);
        if !had_errors && !decoded.is_empty() {
            return SHIFT_JIS;
        }

        // EUC-JP を試す
        let (decoded, encoding_used, had_errors) = EUC_JP.decode(bytes);
        if !had_errors && !decoded.is_empty() {
            return EUC_JP;
        }

        // デフォルトはUTF-8
        UTF_8
    }
}

#[async_trait]
impl DocumentLoader for TxtLoader {
    async fn load(&self, path: &Path) -> Result<Vec<Document>> {
        // ファイル読み込み（非同期I/O）
        let bytes = fs::read(path).await.map_err(|e| {
            RagError::DocumentLoad(format!("Failed to read file {}: {}", path.display(), e))
        })?;

        // エンコーディング検出とデコード（CPU-bound処理なのでspawn_blocking）
        let path_str = path.display().to_string();
        let content = tokio::task::spawn_blocking(move || -> Result<String> {
            let encoding = Self::detect_encoding(&bytes);
            let (decoded, _, had_errors) = encoding.decode(&bytes);

            if had_errors {
                tracing::warn!("Encoding errors detected in file: {}", path_str);
            }

            Ok(decoded.into_owned())
        })
        .await
        .map_err(|e| RagError::TaskJoin(e))??;

        // 空のファイルチェック
        if content.trim().is_empty() {
            tracing::warn!("Empty or whitespace-only file: {}", path.display());
            return Ok(vec![]);
        }

        // 正規化されたパスを使用（差分検出で一致させるため）
        let normalized_path = path.canonicalize()
            .map(|p| {
                let s = p.display().to_string();
                // Windows拡張パスプレフィックスを削除
                if s.starts_with(r"\\?\") { s[4..].to_string() } else { s }
            })
            .unwrap_or_else(|_| path.display().to_string());
        let metadata = Metadata::new(normalized_path, FileType::Txt);

        Ok(vec![Document::new(content, metadata)])
    }

    fn supported_extensions(&self) -> &[&str] {
        &["txt"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_encoding_utf8() {
        let utf8_bytes = "Hello, World! こんにちは".as_bytes();
        let encoding = TxtLoader::detect_encoding(utf8_bytes);
        assert_eq!(encoding, UTF_8);
    }

    #[test]
    fn test_detect_encoding_with_bom() {
        let mut bom_bytes = vec![0xEF, 0xBB, 0xBF]; // UTF-8 BOM
        bom_bytes.extend_from_slice("Hello".as_bytes());
        let encoding = TxtLoader::detect_encoding(&bom_bytes);
        assert_eq!(encoding, UTF_8);
    }

    #[tokio::test]
    async fn test_txt_loader_supports() {
        let loader = TxtLoader::new();
        assert!(loader.supports(Path::new("test.txt")));
        assert!(loader.supports(Path::new("document.TXT")));
        assert!(!loader.supports(Path::new("document.pdf")));
    }
}
