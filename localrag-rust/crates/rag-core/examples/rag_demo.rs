use rag_core::{
    document::loader::ParallelDocumentLoader, AgentPipeline, ChromaClient, IndexProgress,
    OllamaClient, RagPipeline, VectorDbConfig,
};
use std::io::{self, Write};
use std::path::PathBuf;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // トレーシング初期化
    tracing_subscriber::fmt::init();

    println!("🦀 LocalRAG Pro - Complete RAG Demo\n");

    // 1. Ollamaステータスチェック
    println!("📡 Checking Ollama status...");
    let ollama = OllamaClient::default();

    if !ollama.check_running().await {
        println!("❌ Ollama is not running!");
        println!("   Please start Ollama and try again.");
        return Ok(());
    }

    println!("✅ Ollama is running!\n");

    // 2. モデルチェック
    println!("🤖 Checking models...");
    let llm_models = ollama.list_llm_models().await?;
    let embed_models = ollama.list_embedding_models().await?;

    if llm_models.is_empty() {
        println!("❌ No LLM models found!");
        println!("   Please install a model: ollama pull gemma2:2b");
        return Ok(());
    }

    if embed_models.is_empty() {
        println!("❌ No embedding models found!");
        println!("   Please install: ollama pull nomic-embed-text");
        return Ok(());
    }

    let llm_model = &llm_models[0].name;
    let embed_model = &embed_models[0].name;

    println!("   LLM: {}", llm_model);
    println!("   Embedding: {}", embed_model);
    println!();

    // 3. ディレクトリ選択
    print!("📁 Enter directory path to index (or press Enter to skip indexing): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let dir_path = input.trim();

    // 4. ChromaDB設定
    let chroma_config = VectorDbConfig::new("localrag_demo", 768);
    let chroma = ChromaClient::new(chroma_config);

    // 5. RAGパイプライン作成
    let pipeline = RagPipeline::new(
        ollama.clone(),
        chroma,
        embed_model.clone(),
        llm_model.clone(),
    )
    .with_splitter(1000, 100);

    // 6. インデックス作成（ディレクトリが指定されている場合）
    if !dir_path.is_empty() {
        println!("\n🔨 Starting indexing...");

        let (progress_tx, mut progress_rx) = mpsc::channel(100);

        let pipeline_clone = pipeline.clone();
        let dir_path_clone = PathBuf::from(dir_path);

        tokio::spawn(async move {
            if let Err(e) = pipeline_clone
                .index_directory(&dir_path_clone, Some(progress_tx))
                .await
            {
                eprintln!("❌ Indexing error: {}", e);
            }
        });

        // 進捗表示
        while let Some(progress) = progress_rx.recv().await {
            match progress {
                IndexProgress::Scanning { current, total } => {
                    println!("   📂 Scanning: {}/{}", current, total);
                }
                IndexProgress::Loading {
                    current,
                    total,
                    file,
                } => {
                    println!(
                        "   📄 Loading: {}/{} - {}",
                        current,
                        total,
                        file.file_name().unwrap_or_default().to_string_lossy()
                    );
                }
                IndexProgress::Splitting { current, total } => {
                    println!("   ✂️  Splitting: {}/{}", current, total);
                }
                IndexProgress::Embedding { current, total } => {
                    println!("   🧬 Embedding: {}/{}", current, total);
                }
                IndexProgress::Storing { current, total } => {
                    println!("   💾 Storing: {}/{}", current, total);
                }
                IndexProgress::Complete { stats } => {
                    println!("\n✅ Indexing complete!");
                    println!("   Total files: {}", stats.total_files);
                    println!("   Indexed: {}", stats.indexed_files);
                    println!("   Chunks: {}", stats.total_chunks);
                    println!("   Embeddings: {}", stats.total_embeddings);
                    break;
                }
            }
        }
    }

    // 7. エージェントパイプライン作成
    let agent = AgentPipeline::new(pipeline.clone());

    // 8. インタラクティブクエリループ
    println!("\n💬 Interactive Query Mode");
    println!("   Type 'quit' to exit");
    println!("   Type 'agent: <question>' for agent mode");
    println!();

    loop {
        print!("❓ Question: ");
        io::stdout().flush()?;

        let mut question = String::new();
        io::stdin().read_line(&mut question)?;
        let question = question.trim();

        if question.is_empty() {
            continue;
        }

        if question.eq_ignore_ascii_case("quit") {
            println!("👋 Goodbye!");
            break;
        }

        // エージェントモード判定
        let (use_agent, actual_question) = if question.starts_with("agent:") {
            (true, question[6..].trim())
        } else {
            (false, question)
        };

        if use_agent {
            println!("\n🤖 Agent Mode");

            let (progress_tx, mut progress_rx) = mpsc::channel(100);

            let agent_clone = agent.clone();
            let question_clone = actual_question.to_string();

            tokio::spawn(async move {
                match agent_clone
                    .query_agent(&question_clone, Some(progress_tx))
                    .await
                {
                    Ok(response) => {
                        if let Some(mut stream) = response.stream {
                            print!("\n📝 Answer: ");
                            io::stdout().flush().ok();

                            use tokio_stream::StreamExt;
                            while let Some(chunk_result) = stream.next().await {
                                match chunk_result {
                                    Ok(chunk) => {
                                        print!("{}", chunk);
                                        io::stdout().flush().ok();
                                    }
                                    Err(e) => {
                                        eprintln!("\n❌ Stream error: {}", e);
                                        break;
                                    }
                                }
                            }
                            println!("\n");
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Query error: {}", e);
                    }
                }
            });

            // エージェント進捗表示
            while let Some(progress) = progress_rx.recv().await {
                use rag_core::rag::AgentProgress;
                match progress {
                    AgentProgress::Analyzing => println!("   🤔 Analyzing question..."),
                    AgentProgress::Keywords(kw) => println!("   💡 Keywords: {:?}", kw),
                    AgentProgress::Searching(kw) => println!("   🔍 Searching: {}", kw),
                    AgentProgress::Found(n) => println!("   📚 Found {} documents", n),
                    AgentProgress::ValidatingSufficiency => {
                        println!("   ✓ Validating sufficiency...")
                    }
                    AgentProgress::Generating => {
                        println!("   🧠 Generating answer...");
                        break;
                    }
                    AgentProgress::Complete => break,
                }
            }

            // ストリーミング完了を待つ
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        } else {
            println!("\n💬 Normal Mode");

            let response = pipeline.query(actual_question, 3).await?;

            println!("\n📝 Answer:\n{}", response.answer);
            println!("\n📚 Sources:");
            for (i, source) in response.sources.iter().enumerate() {
                println!(
                    "   {}. {} (distance: {:.3})",
                    i + 1,
                    source.source().unwrap_or("Unknown"),
                    source.distance
                );
            }
        }

        println!();
    }

    Ok(())
}
