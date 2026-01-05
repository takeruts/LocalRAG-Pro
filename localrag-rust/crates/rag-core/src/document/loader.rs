use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore};
use walkdir::WalkDir;

use crate::document::{
    Document, DocumentLoader, FileType, LoadProgress,
    pdf::PdfLoader, docx::DocxLoader, xlsx::XlsxLoader, txt::TxtLoader,
};
use crate::error::Result;

/// 並列ドキュメントローダーマネージャー
#[derive(Clone)]
pub struct ParallelDocumentLoader {
    loaders: Arc<HashMap<FileType, Box<dyn DocumentLoader>>>,
    max_concurrent: usize,
}

impl ParallelDocumentLoader {
    /// 新しいローダーを作成
    pub fn new() -> Self {
        let mut loaders: HashMap<FileType, Box<dyn DocumentLoader>> = HashMap::new();
        loaders.insert(FileType::Txt, Box::new(TxtLoader::new()));
        loaders.insert(FileType::Pdf, Box::new(PdfLoader::new()));
        loaders.insert(FileType::Docx, Box::new(DocxLoader::new()));
        loaders.insert(FileType::Xlsx, Box::new(XlsxLoader::new()));

        Self {
            loaders: Arc::new(loaders),
            max_concurrent: 10, // デフォルトは10並列
        }
    }

    /// 最大同時実行数を設定
    pub fn with_max_concurrent(mut self, max_concurrent: usize) -> Self {
        self.max_concurrent = max_concurrent;
        self
    }

    /// ディレクトリをスキャンして対応ファイルを検索
    pub fn scan_directory(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        use rayon::prelude::*;

        let supported_extensions: Vec<&str> = self
            .loaders
            .values()
            .flat_map(|loader| loader.supported_extensions())
            .copied()
            .collect();

        // Rayonで並列ディレクトリスキャン
        let files: Vec<PathBuf> = WalkDir::new(dir)
            .into_iter()
            .par_bridge()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| supported_extensions.contains(&ext.to_lowercase().as_str()))
                    .unwrap_or(false)
            })
            .map(|entry| entry.path().to_path_buf())
            .collect();

        Ok(files)
    }

    /// ファイルのリストからドキュメントを並列ロード
    pub async fn load_files(
        &self,
        files: &[PathBuf],
        progress_tx: Option<mpsc::Sender<LoadProgress>>,
    ) -> Result<Vec<Document>> {
        if files.is_empty() {
            return Ok(vec![]);
        }

        let semaphore = Arc::new(Semaphore::new(self.max_concurrent));
        let total = files.len();
        let loaders = self.loaders.clone();

        // 各ファイルを非同期タスクとして起動
        let tasks: Vec<_> = files
            .iter()
            .enumerate()
            .map(|(idx, path)| {
                let sem = semaphore.clone();
                let path = path.clone();
                let loaders_clone = loaders.clone();
                let progress_tx = progress_tx.clone();

                tokio::spawn(async move {
                    // セマフォで同時実行数を制限
                    let _permit = sem.acquire().await.unwrap();

                    // 進捗通知
                    if let Some(tx) = &progress_tx {
                        let progress = LoadProgress {
                            current: idx + 1,
                            total,
                            current_file: path.clone(),
                        };
                        let _ = tx.send(progress).await;
                    }

                    // ファイルタイプを判定してローダー取得
                    let file_type = path
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .and_then(FileType::from_extension);

                    // ドキュメントロード
                    match file_type.and_then(|ft| loaders_clone.get(&ft)) {
                        Some(loader) => loader.load(&path).await,
                        None => Ok(vec![]),
                    }
                })
            })
            .collect();

        // 全タスクの完了を待つ
        let results = futures::future::join_all(tasks).await;

        // 結果を集約（エラーはスキップ）
        let mut all_documents = Vec::new();
        for result in results {
            match result {
                Ok(Ok(docs)) => all_documents.extend(docs),
                Ok(Err(e)) => {
                    tracing::warn!("Failed to load document: {}", e);
                }
                Err(e) => {
                    tracing::error!("Task join error: {}", e);
                }
            }
        }

        Ok(all_documents)
    }

    /// ディレクトリ全体をスキャンしてロード
    pub async fn load_directory(
        &self,
        dir: &Path,
        progress_tx: Option<mpsc::Sender<LoadProgress>>,
    ) -> Result<Vec<Document>> {
        let files = self.scan_directory(dir)?;
        self.load_files(&files, progress_tx).await
    }

    /// パスに対応するローダーを取得
    fn get_loader_for_path(&self, path: &Path) -> Option<&dyn DocumentLoader> {
        path.extension()
            .and_then(|ext| FileType::from_extension(&ext.to_string_lossy()))
            .and_then(|file_type| self.loaders.get(&file_type))
            .map(|boxed| &**boxed)
    }
}

impl Default for ParallelDocumentLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_loader_creation() {
        let loader = ParallelDocumentLoader::new();
        assert_eq!(loader.max_concurrent, 10);
        assert_eq!(loader.loaders.len(), 4); // TXT, PDF, DOCX, XLSX
    }

    #[test]
    fn test_with_max_concurrent() {
        let loader = ParallelDocumentLoader::new().with_max_concurrent(5);
        assert_eq!(loader.max_concurrent, 5);
    }

    #[tokio::test]
    async fn test_load_empty_files() {
        let loader = ParallelDocumentLoader::new();
        let result = loader.load_files(&[], None).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }
}
