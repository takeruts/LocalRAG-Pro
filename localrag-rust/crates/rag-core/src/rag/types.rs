use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio_stream::wrappers::ReceiverStream;

use crate::vectordb::SearchResult;

/// インデックス作成の進捗情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexProgress {
    /// ファイルスキャン中
    Scanning {
        current: usize,
        total: usize,
    },
    /// ドキュメント読み込み中
    Loading {
        current: usize,
        total: usize,
        file: PathBuf,
    },
    /// テキスト分割中
    Splitting {
        current: usize,
        total: usize,
    },
    /// Embedding生成中
    Embedding {
        current: usize,
        total: usize,
    },
    /// DB登録中
    Storing {
        current: usize,
        total: usize,
    },
    /// 完了
    Complete {
        stats: IndexStats,
    },
}

impl IndexProgress {
    pub fn percentage(&self) -> f32 {
        match self {
            IndexProgress::Scanning { current, total }
            | IndexProgress::Loading { current, total, .. }
            | IndexProgress::Splitting { current, total }
            | IndexProgress::Embedding { current, total }
            | IndexProgress::Storing { current, total } => {
                if *total == 0 {
                    0.0
                } else {
                    (*current as f32 / *total as f32) * 100.0
                }
            }
            IndexProgress::Complete { .. } => 100.0,
        }
    }
}

/// インデックス作成統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub total_files: usize,
    pub indexed_files: usize,
    pub skipped_files: usize,
    pub total_chunks: usize,
    pub total_embeddings: usize,
    pub errors: Vec<String>,
}

impl IndexStats {
    pub fn new() -> Self {
        Self {
            total_files: 0,
            indexed_files: 0,
            skipped_files: 0,
            total_chunks: 0,
            total_embeddings: 0,
            errors: Vec::new(),
        }
    }
}

/// クエリレスポンス
#[derive(Debug)]
pub struct QueryResponse {
    pub answer: Option<String>,
    pub sources: Vec<SearchResult>,
    pub stream: Option<ReceiverStream<crate::Result<String>>>,
}

impl QueryResponse {
    pub fn new(answer: String, sources: Vec<SearchResult>) -> Self {
        Self {
            answer: Some(answer),
            sources,
            stream: None,
        }
    }

    pub fn with_stream(
        sources: Vec<SearchResult>,
        stream: ReceiverStream<crate::Result<String>>,
    ) -> Self {
        Self {
            answer: None,
            sources,
            stream: Some(stream),
        }
    }
}

/// エージェントの進捗情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentProgress {
    /// 質問を分析中
    Analyzing,
    /// 検索キーワード抽出完了
    Keywords(Vec<String>),
    /// 検索中
    Searching(String),
    /// 資料発見
    Found(usize),
    /// 資料の十分性を検証中
    ValidatingSufficiency,
    /// 回答生成中
    Generating,
    /// 完了
    Complete,
}
