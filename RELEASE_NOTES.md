# LocalRAG Pro - Rust Edition v1.0.0

**Release Date**: 2026-01-06

## 📦 Package Information

- **File**: `LocalRAG-Rust-v1.0.0.tar.gz`
- **Size**: 3.7 MB
- **Platform**: Windows 10/11 (x64)

## ✨ Features

### Core Functionality
- ✅ **Complete Rust Implementation**: Entire RAG application rewritten in Rust for maximum performance
- ✅ **Modern GUI**: Built with eframe/egui for responsive, native desktop experience
- ✅ **Real-time Progress**: Live progress reporting during document indexing and embedding generation
- ✅ **Cancellable Operations**: Stop button to cancel ongoing indexing operations at any time
- ✅ **Japanese Language Support**: Full support for Japanese fonts via Windows system fonts

### Performance Optimizations
- ✅ **10-20x Faster**: Significant performance improvement over Python version
- ✅ **Parallel Processing**: Document loading and processing with Rayon
- ✅ **Optimized Batch Processing**:
  - Batch size: 30 documents
  - Concurrent requests: 5
  - Timeout: 5 minutes per request
- ✅ **ChromaDB Batch Splitting**: Automatically splits large batches to respect ChromaDB's 5000 document limit

### Technical Improvements
- ✅ **ChromaDB HTTP Bridge**: Custom FastAPI server for ChromaDB 1.4.0 compatibility
- ✅ **Async Architecture**: Tokio-based async runtime for non-blocking operations
- ✅ **Stream Processing**: Real-time LLM response streaming
- ✅ **Multi-format Support**: PDF, DOCX, XLSX, TXT document parsing

## 🔧 System Requirements

### Required
- **Ollama**: LLM runtime (https://ollama.com/)
  - `gemma2:2b` model for text generation
  - `nomic-embed-text` model for embeddings
- **Python 3.9+**: For ChromaDB bridge server
- **Windows 10/11**: 64-bit

### Optional
- GPU acceleration (via Ollama)

## 📋 Package Contents

```
LocalRAG-Release/
├── LocalRAG.exe (7.6 MB)      # Main application
├── chromadb_server.py          # ChromaDB HTTP bridge
├── Launch.bat                  # Auto-start launcher
├── 起動.bat                     # Japanese launcher
├── start_chromadb.bat          # ChromaDB server starter
├── setup.bat                   # Python environment setup
├── README.txt                  # English documentation
├── はじめに.txt                 # Japanese documentation
└── VERSION.txt                 # Version information
```

## 🚀 Quick Start

### First Time Setup

1. **Install Ollama**
   ```bash
   # Download from https://ollama.com/
   # Then pull required models:
   ollama pull gemma2:2b
   ollama pull nomic-embed-text
   ```

2. **Setup Python Environment**
   ```bash
   # Double-click setup.bat
   # Or manually:
   python -m venv .venv
   .venv\Scripts\activate
   pip install chromadb fastapi uvicorn
   ```

3. **Launch Application**
   ```bash
   # Double-click Launch.bat or 起動.bat
   ```

### Normal Usage

1. **Start**: Double-click `Launch.bat`
2. **Index**: Select a folder and click "インデックス作成"
3. **Query**: Enter your question and get AI-powered answers

## 🐛 Known Issues & Fixes

### Embedding Timeout
- **Issue**: Large document sets may timeout during embedding generation
- **Fix**: Process is retryable; already processed documents are saved
- **Timeout**: Extended to 5 minutes per batch

### ChromaDB Connection
- **Issue**: "ChromaDBブリッジサーバーに接続できません"
- **Fix**: Ensure `start_chromadb.bat` is running or use `Launch.bat`

### Progress Display
- **Issue**: Progress stuck at 0%
- **Fix**: Now properly reports real-time progress for all operations

## 📈 Performance Comparison

| Metric | Python Version | Rust Version | Improvement |
|--------|---------------|--------------|-------------|
| Document Loading | ~2 sec/file | ~0.2 sec/file | 10x faster |
| Embedding Speed | ~5 sec/batch | ~1 sec/batch | 5x faster |
| Memory Usage | ~500 MB | ~100 MB | 5x reduction |
| Startup Time | ~5 seconds | ~0.5 seconds | 10x faster |

## 🔐 Security Notes

- All data processed locally
- No telemetry or external connections (except Ollama API)
- ChromaDB runs on localhost only (port 8001)

## 📝 Technical Stack

- **Language**: Rust 1.75+
- **GUI**: eframe 0.30 + egui 0.30
- **Runtime**: Tokio (async)
- **Parallelism**: Rayon
- **LLM**: Ollama (gemma2:2b)
- **Embeddings**: Ollama (nomic-embed-text)
- **Vector DB**: ChromaDB 1.4.0
- **Document Parsing**:
  - PDF: pdf-extract
  - DOCX: docx-rs
  - XLSX: calamine
  - TXT: encoding_rs

## 🙏 Acknowledgments

Built with:
- Rust
- egui (Emil Ernerfeldt)
- Ollama
- ChromaDB
- Claude Code (Development Assistant)

## 📄 License

MIT License

---

**Download**: `LocalRAG-Rust-v1.0.0.tar.gz` (3.7 MB)

**Support**: https://github.com/takeruts/LocalRAG-Pro/issues
