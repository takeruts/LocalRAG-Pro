use rayon::prelude::*;
use unicode_segmentation::UnicodeSegmentation;

use crate::document::Document;
use super::TextSplitter;

/// RecursiveCharacterTextSplitter
///
/// Langchainの RecursiveCharacterTextSplitter と同等の機能を持つテキスト分割器
#[derive(Debug, Clone)]
pub struct RecursiveCharacterTextSplitter {
    chunk_size: usize,
    chunk_overlap: usize,
    separators: Vec<String>,
}

impl RecursiveCharacterTextSplitter {
    /// 新しい分割器を作成
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
            separators: vec![
                "\n\n".to_string(),
                "\n".to_string(),
                " ".to_string(),
                "".to_string(),
            ],
        }
    }

    /// カスタムセパレーターで作成
    pub fn with_separators(mut self, separators: Vec<String>) -> Self {
        self.separators = separators;
        self
    }

    /// テキストを再帰的に分割
    fn split_text_recursive(&self, text: &str, separators: &[String]) -> Vec<String> {
        if text.is_empty() {
            return vec![];
        }

        let text_len = text.graphemes(true).count();

        // チャンクサイズ以下なら分割不要
        if text_len <= self.chunk_size {
            return vec![text.to_string()];
        }

        // セパレーターを順番に試す
        for (i, separator) in separators.iter().enumerate() {
            let splits: Vec<&str> = if separator.is_empty() {
                // 空文字の場合は文字単位で分割
                text.split("").filter(|s| !s.is_empty()).collect()
            } else {
                text.split(separator.as_str()).collect()
            };

            // 分割できた場合
            if splits.len() > 1 {
                let mut chunks = Vec::new();
                let mut current_chunk = Vec::new();
                let mut current_length = 0;

                for split in splits {
                    let split_len = split.graphemes(true).count();

                    // 現在のチャンクに追加可能かチェック
                    if current_length + split_len + separator.len() <= self.chunk_size {
                        current_chunk.push(split);
                        current_length += split_len + separator.len();
                    } else {
                        // 現在のチャンクを確定
                        if !current_chunk.is_empty() {
                            let chunk = current_chunk.join(separator);
                            if !chunk.trim().is_empty() {
                                chunks.push(chunk);
                            }
                        }

                        // splitが大きすぎる場合は再帰的に分割
                        if split_len > self.chunk_size {
                            let remaining_separators = &separators[i + 1..];
                            if !remaining_separators.is_empty() {
                                chunks.extend(self.split_text_recursive(split, remaining_separators));
                            } else {
                                // セパレーターがない場合は文字単位で分割
                                chunks.extend(self.split_by_characters(split));
                            }
                        } else {
                            current_chunk = vec![split];
                            current_length = split_len;
                        }
                    }
                }

                // 残りのチャンクを追加
                if !current_chunk.is_empty() {
                    let chunk = current_chunk.join(separator);
                    if !chunk.trim().is_empty() {
                        chunks.push(chunk);
                    }
                }

                // オーバーラップを適用
                return self.apply_overlap(chunks);
            }
        }

        // 分割できなかった場合は文字単位で分割
        self.split_by_characters(text)
    }

    /// 文字単位で分割
    fn split_by_characters(&self, text: &str) -> Vec<String> {
        let graphemes: Vec<&str> = text.graphemes(true).collect();
        let mut chunks = Vec::new();
        let mut i = 0;

        while i < graphemes.len() {
            let end = (i + self.chunk_size).min(graphemes.len());
            let chunk = graphemes[i..end].join("");
            chunks.push(chunk);
            i += self.chunk_size - self.chunk_overlap;
        }

        chunks
    }

    /// オーバーラップを適用
    fn apply_overlap(&self, chunks: Vec<String>) -> Vec<String> {
        if self.chunk_overlap == 0 || chunks.len() <= 1 {
            return chunks;
        }

        let mut overlapped_chunks = Vec::new();

        for (i, chunk) in chunks.iter().enumerate() {
            if i == 0 {
                overlapped_chunks.push(chunk.clone());
            } else {
                // 前のチャンクの末尾をオーバーラップとして追加
                let prev_chunk = &chunks[i - 1];
                let prev_graphemes: Vec<&str> = prev_chunk.graphemes(true).collect();

                if prev_graphemes.len() > self.chunk_overlap {
                    let overlap_start = prev_graphemes.len() - self.chunk_overlap;
                    let overlap = prev_graphemes[overlap_start..].join("");
                    overlapped_chunks.push(format!("{}{}", overlap, chunk));
                } else {
                    overlapped_chunks.push(chunk.clone());
                }
            }
        }

        overlapped_chunks
    }
}

impl TextSplitter for RecursiveCharacterTextSplitter {
    fn split_documents(&self, documents: Vec<Document>) -> Vec<Document> {
        // Rayonで並列分割
        documents
            .into_par_iter()
            .flat_map(|doc| {
                let chunks = self.split_text(&doc.content);
                chunks
                    .into_iter()
                    .map(|chunk| Document::new(chunk, doc.metadata.clone()))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn split_text(&self, text: &str) -> Vec<String> {
        self.split_text_recursive(text, &self.separators)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_split() {
        let splitter = RecursiveCharacterTextSplitter::new(20, 0);
        let text = "This is a test.\n\nThis is another paragraph.";
        let chunks = splitter.split_text(text);

        assert!(chunks.len() > 1);
        for chunk in &chunks {
            assert!(chunk.graphemes(true).count() <= 20);
        }
    }

    #[test]
    fn test_no_split_needed() {
        let splitter = RecursiveCharacterTextSplitter::new(100, 0);
        let text = "Short text";
        let chunks = splitter.split_text(text);

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }

    #[test]
    fn test_with_overlap() {
        let splitter = RecursiveCharacterTextSplitter::new(10, 3);
        let text = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let chunks = splitter.split_text(text);

        assert!(chunks.len() > 1);

        // オーバーラップのチェック
        for i in 1..chunks.len() {
            let current = &chunks[i];
            let previous = &chunks[i - 1];

            // 現在のチャンクの開始が前のチャンクの終わりと一部重複している
            let prev_end = previous.chars().rev().take(3).collect::<String>();
            assert!(current.starts_with(&prev_end.chars().rev().collect::<String>()));
        }
    }

    #[test]
    fn test_unicode_handling() {
        let splitter = RecursiveCharacterTextSplitter::new(10, 0);
        let text = "こんにちは世界。これはテストです。";
        let chunks = splitter.split_text(text);

        for chunk in &chunks {
            let grapheme_count = chunk.graphemes(true).count();
            assert!(grapheme_count <= 10, "Chunk too long: {} graphemes", grapheme_count);
        }
    }

    #[test]
    fn test_empty_text() {
        let splitter = RecursiveCharacterTextSplitter::new(10, 0);
        let chunks = splitter.split_text("");

        assert_eq!(chunks.len(), 0);
    }

    #[test]
    fn test_parallel_document_split() {
        use crate::document::{Metadata, FileType};

        let splitter = RecursiveCharacterTextSplitter::new(20, 5);

        let docs = vec![
            Document::new(
                "First document with some content.".to_string(),
                Metadata::new("doc1.txt", FileType::Txt),
            ),
            Document::new(
                "Second document with different content.".to_string(),
                Metadata::new("doc2.txt", FileType::Txt),
            ),
        ];

        let splits = splitter.split_documents(docs);

        assert!(splits.len() >= 2); // 少なくとも分割されているはず

        // メタデータが保持されているか確認
        for split in &splits {
            assert!(split.metadata.source == "doc1.txt" || split.metadata.source == "doc2.txt");
        }
    }
}
