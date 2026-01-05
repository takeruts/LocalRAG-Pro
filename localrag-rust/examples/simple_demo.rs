use rag_core::{
    document::loader::ParallelDocumentLoader,
    OllamaClient, RecursiveCharacterTextSplitter, TextSplitter,
};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // トレーシングの初期化
    tracing_subscriber::fmt::init();

    println!("🦀 LocalRAG Pro - Rust Edition Demo\n");

    // 1. Ollamaステータスチェック
    println!("📡 Checking Ollama status...");
    let ollama = OllamaClient::default();

    if ollama.check_running().await {
        println!("✅ Ollama is running!\n");

        // モデル一覧を取得
        println!("📋 Available models:");
        match ollama.list_models().await {
            Ok(models) => {
                for model in models.iter().take(5) {
                    println!("  - {}", model.name);
                }
                if models.len() > 5 {
                    println!("  ... and {} more", models.len() - 5);
                }
            }
            Err(e) => println!("  ⚠️  Failed to list models: {}", e),
        }
    } else {
        println!("⚠️  Ollama is not running");
        println!("  Please start Ollama and try again.");
    }

    println!();

    // 2. ドキュメントローダーのデモ
    println!("📁 Document Loader Demo");
    println!("  Creating parallel document loader...");

    let loader = ParallelDocumentLoader::new()
        .with_max_concurrent(5);

    println!("  ✅ Loader created (max 5 concurrent)");
    println!("  📝 Supported formats: PDF, DOCX, XLSX, TXT");
    println!();

    // 3. テキスト分割器のデモ
    println!("✂️  Text Splitter Demo");

    let splitter = RecursiveCharacterTextSplitter::new(100, 20);

    let sample_text = r#"
LocalRAG Pro is a powerful Retrieval-Augmented Generation application.
It combines the power of local language models with efficient document processing.

The Rust version offers significant performance improvements:
- 10-20x faster startup time
- 3-5x lower memory usage
- Parallel document processing
- Pure Rust implementation with no Python dependencies

Key features include:
- PDF, DOCX, XLSX, and TXT support
- Automatic encoding detection
- Parallel file processing
- Ollama integration
- Vector database support
"#;

    let chunks = splitter.split_text(sample_text);

    println!("  Original text: {} characters", sample_text.len());
    println!("  Split into {} chunks:", chunks.len());
    for (i, chunk) in chunks.iter().take(3).enumerate() {
        println!("\n  Chunk {}: ({} chars)", i + 1, chunk.len());
        println!("  {}", chunk.trim().lines().next().unwrap_or(""));
        if chunk.lines().count() > 1 {
            println!("  ...");
        }
    }

    if chunks.len() > 3 {
        println!("\n  ... and {} more chunks", chunks.len() - 3);
    }

    println!();

    // 4. パフォーマンス情報
    println!("⚡ Performance Highlights");
    println!("  - Rayon: Parallel document processing");
    println!("  - Tokio: Async I/O for file operations");
    println!("  - Pure Rust: No Python overhead");
    println!("  - Unicode-safe: Proper grapheme handling");
    println!();

    println!("✨ Demo completed!");
    println!();
    println!("Next steps:");
    println!("  1. Implement ChromaDB integration");
    println!("  2. Build RAG pipeline");
    println!("  3. Create GUI with eframe/egui");

    Ok(())
}
