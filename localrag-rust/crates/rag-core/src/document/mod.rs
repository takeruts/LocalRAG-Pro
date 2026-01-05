use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::Result;

pub mod loader;
pub mod pdf;
pub mod docx;
pub mod xlsx;
pub mod txt;

/// ドキュメントを表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// ドキュメントのテキストコンテンツ
    pub content: String,

    /// メタデータ
    pub metadata: Metadata,
}

impl Document {
    pub fn new(content: String, metadata: Metadata) -> Self {
        Self { content, metadata }
    }

    /// メタデータをHashMapに変換（VectorDB保存用）
    pub fn metadata_to_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("source".to_string(), self.metadata.source.clone());
        map.insert("file_type".to_string(), format!("{:?}", self.metadata.file_type));

        if let Some(page) = self.metadata.page {
            map.insert("page".to_string(), page.to_string());
        }

        map
    }
}

/// ドキュメントメタデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    /// ソースファイルパス
    pub source: String,

    /// ページ番号（該当する場合）
    pub page: Option<usize>,

    /// ファイルタイプ
    pub file_type: FileType,
}

impl Metadata {
    pub fn new(source: impl Into<String>, file_type: FileType) -> Self {
        Self {
            source: source.into(),
            page: None,
            file_type,
        }
    }

    pub fn with_page(mut self, page: usize) -> Self {
        self.page = Some(page);
        self
    }
}

/// ファイルタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FileType {
    Pdf,
    Docx,
    Xlsx,
    Txt,
}

impl FileType {
    /// 拡張子からFileTypeを判定
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "pdf" => Some(FileType::Pdf),
            "docx" => Some(FileType::Docx),
            "xlsx" | "xls" => Some(FileType::Xlsx),
            "txt" => Some(FileType::Txt),
            _ => None,
        }
    }

    /// 対応する拡張子のリスト
    pub fn extensions(&self) -> &[&str] {
        match self {
            FileType::Pdf => &["pdf"],
            FileType::Docx => &["docx"],
            FileType::Xlsx => &["xlsx", "xls"],
            FileType::Txt => &["txt"],
        }
    }
}

/// ドキュメントローダートレイト
#[async_trait]
pub trait DocumentLoader: Send + Sync {
    /// ファイルからドキュメントを読み込む
    async fn load(&self, path: &Path) -> Result<Vec<Document>>;

    /// サポートする拡張子
    fn supported_extensions(&self) -> &[&str];

    /// このローダーがファイルをサポートするかチェック
    fn supports(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            self.supported_extensions().contains(&ext.as_str())
        } else {
            false
        }
    }
}

/// ロード進捗情報
#[derive(Debug, Clone)]
pub struct LoadProgress {
    pub current: usize,
    pub total: usize,
    pub current_file: PathBuf,
}

impl LoadProgress {
    pub fn percentage(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            (self.current as f32 / self.total as f32) * 100.0
        }
    }
}
