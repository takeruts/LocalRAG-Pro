# LocalRAG Pro - Rust Edition

Pythonで書かれた [win_rag.py](../win_rag.py) を完全にRustで再実装し、10-20倍の高速化を実現するプロジェクトです。

## 目標

- **起動時間**: 0.5-2秒（Python版の5-20倍高速）
- **ファイルスキャン**: 3-6秒で100ファイル（Python版の10倍高速）
- **PDF処理**: 2-3秒で100ページ（Python版の7-10倍高速）
- **メモリ使用量**: 50-100MB（Python版の1/3-1/5）
- **バイナリサイズ**: 30-80MB（Python版の1/6-1/16）

## アーキテクチャ

```
localrag-rust/
├── Cargo.toml                 # ワークスペース定義
├── crates/
│   ├── rag-core/              # コアライブラリ
│   │   └── src/
│   │       ├── document/      # PDF/DOCX/XLSX/TXTローダー ✅
│   │       ├── splitter/      # テキスト分割 ✅
│   │       ├── vectordb/      # ChromaDB連携 (実装予定)
│   │       ├── ollama/        # Ollama APIクライアント ✅
│   │       └── rag/           # RAGパイプライン (実装予定)
│   │
│   └── rag-gui/               # eframe/egui GUI (実装予定)
│       └── src/
│           ├── app.rs         # メインアプリ
│           └── ui/            # UIコンポーネント
```

## 実装済み機能

### ✅ フェーズ1: コア基盤（完了！）

- [x] プロジェクト構造セットアップ
- [x] エラーハンドリング型定義
- [x] ドキュメント共通型定義
- [x] TXTローダー（encoding_rs使用、自動エンコード検出）
- [x] PDFローダー（lopdf使用、並列ページ処理）
- [x] DOCXローダー（docx-rs使用）
- [x] XLSXローダー（calamine使用）
- [x] 並列ドキュメントローダーマネージャー
- [x] Ollama APIクライアント（完全実装）
- [x] RecursiveCharacterTextSplitter（Unicode対応）

#### ドキュメントローダーの特徴

**並列処理戦略:**
- ディレクトリスキャン: Rayon並列化
- ファイルI/O: Tokio非同期
- パース処理: `spawn_blocking`でCPU並列化
- 最大同時実行数: 10（設定可能）

**対応フォーマット:**
- **PDF**: lopdf (pure Rust, 軽量)
- **DOCX**: docx-rs (pure Rust)
- **XLSX**: calamine (pure Rust, 高速)
- **TXT**: encoding_rs (UTF-8, Shift_JIS, EUC-JP自動検出)

#### Ollama APIクライアントの機能

- ✅ Ollama稼働状態チェック
- ✅ モデル一覧取得（LLM/Embeddingフィルタリング）
- ✅ 単一/バッチEmbedding生成（並列リクエスト対応）
- ✅ テキスト生成（非ストリーミング/ストリーミング）
- ✅ チャット生成
- ✅ モデルのpull/delete

#### テキスト分割器の機能

- ✅ RecursiveCharacterTextSplitter実装
- ✅ Unicode対応（grapheme単位の正確な分割）
- ✅ カスタムセパレーター対応
- ✅ オーバーラップ設定
- ✅ Rayon並列ドキュメント分割

### ✅ フェーズ2: ベクトルDB & RAGパイプライン（完了！）

- [x] ChromaDB連携クライアント
- [x] VectorDatabase抽象化トレイト
- [x] RAGパイプライン（インデックス作成 & 検索）
- [x] エージェントモード（自律的キーワード抽出）
- [x] インタラクティブCLIデモ

#### ChromaDBクライアントの機能

- ✅ コレクション管理（作成/削除/存在確認）
- ✅ バッチドキュメント追加（並列POST）
- ✅ ベクトル検索（類似度検索）
- ✅ 既存インデックス取得（差分検出用）
- ✅ 統計情報取得

#### RAGパイプラインの機能

- ✅ ディレクトリインデックス作成（進捗レポート付き）
- ✅ 差分検出（新規ファイルのみ処理）
- ✅ 並列ドキュメント処理（Tokio + Rayon）
- ✅ バッチEmbedding生成（並列リクエスト）
- ✅ クエリ実行（非ストリーミング/ストリーミング）
- ✅ コンテキスト構築（出典情報付き）

#### エージェントパイプラインの機能

- ✅ 質問からキーワード抽出
- ✅ 並列マルチキーワード検索
- ✅ 重複除外処理
- ✅ 情報充足性判定

### ✅ フェーズ3: GUI実装（完了！）

- [x] eframe/egui GUI基盤
- [x] アプリケーション状態管理
- [x] バックエンド通信（mpscチャネル）
- [x] サイドバーUI（モデル選択、インデックス管理）
- [x] チャットインターフェース（ストリーミング対応）
- [x] リアルタイム進捗表示
- [x] エージェントモードUI
- [x] ソース情報表示

#### GUI機能

- ✅ モダンダークテーマ（Material Design風）
- ✅ Ollamaステータス表示
- ✅ モデル選択（LLM/Embedding）
- ✅ フォルダ選択ダイアログ
- ✅ インデックス進捗バー
- ✅ ストリーミングチャット
- ✅ エージェント進捗表示
- ✅ ソース情報折りたたみ表示
- ✅ キーボードショートカット（Enter送信）

### 📋 次の実装予定（拡張機能）

- [ ] モデルマネージャーウィンドウ
- [ ] 設定画面
- [ ] チャット履歴エクスポート
- [ ] テーマカスタマイズ

## ビルド & 実行

```bash
# ワークスペース全体をビルド
cd localrag-rust
cargo build

# リリースビルド（最適化有効）
cargo build --release

# テスト実行
cargo test

# 特定のクレートのみビルド
cargo build -p rag-core

# デモ実行（Phase 1: コア機能）
cargo run --example simple_demo

# デモ実行（Phase 2: 完全RAGパイプライン）
cargo run --example rag_demo

# GUI実行（Phase 3: デスクトップアプリ）
cargo run -p rag-gui
```

### デモの実行例

#### Phase 1: コア機能デモ

```bash
cd localrag-rust
cargo run --example simple_demo
```

出力例:
```
🦀 LocalRAG Pro - Rust Edition Demo

📡 Checking Ollama status...
✅ Ollama is running!

📋 Available models:
  - gemma2:2b
  - nomic-embed-text
  ...

📁 Document Loader Demo
  ✅ Loader created (max 5 concurrent)
  📝 Supported formats: PDF, DOCX, XLSX, TXT

✂️  Text Splitter Demo
  Original text: 512 characters
  Split into 6 chunks
  ...

✨ Demo completed!
```

#### Phase 2: 完全RAGパイプラインデモ

```bash
cd localrag-rust
cargo run --example rag_demo
```

出力例:
```
🦀 LocalRAG Pro - RAG Pipeline Demo

📡 Checking Ollama...
✅ Ollama is running
✅ LLM models: gemma2:2b, qwen2.5:7b
✅ Embedding models: nomic-embed-text

📁 Enter directory to index: ./docs
⚡ Starting indexing...

[====================================] 100% (42 files)
✅ Indexing complete!
   Total files: 42
   Indexed: 42
   Chunks: 1,234
   Embeddings: 1,234

💬 Query mode (type 'quit' to exit)
> What is RAG?

🔍 Searching knowledge base...
✨ Generating answer...

RAG (Retrieval-Augmented Generation) は...

📚 Sources:
  - rag_basics.pdf (P.1) - Score: 0.89
  - architecture.md - Score: 0.85
```

## 使用技術

### コア依存関係

- **非同期**: tokio, futures, async-trait
- **並列処理**: rayon
- **HTTP**: reqwest
- **JSON**: serde, serde_json
- **エラー**: anyhow, thiserror
- **ロギング**: tracing

### ドキュメント処理

- **PDF**: lopdf
- **DOCX**: docx-rs
- **XLSX**: calamine
- **テキスト**: encoding_rs, unicode-segmentation

### GUI（予定）

- **フレームワーク**: eframe, egui
- **ダイアログ**: rfd
- **システム統合**: open

## パフォーマンス最適化

### コンパイラ最適化

```toml
[profile.release]
opt-level = 3         # 最高レベル最適化
lto = "fat"          # リンク時最適化
codegen-units = 1    # 単一コードユニット
strip = true         # シンボル削除
panic = "abort"      # パニック時即終了
```

### 並列処理戦略

1. **ファイルスキャン**: Rayonのpar_bridge()で並列化
2. **ファイルI/O**: Tokioの非同期I/O（複数ファイル同時読み込み）
3. **パース処理**: spawn_blockingでCPUバウンドな処理をスレッドプールで並列化
4. **Embedding生成**: 並列HTTPリクエスト（最大5同時）

## 開発ステータス

| コンポーネント | 進捗 | 備考 |
|-------------|------|------|
| プロジェクト構造 | ✅ 100% | Cargo workspace完成 |
| エラーハンドリング | ✅ 100% | RagError型定義完了 |
| TXTローダー | ✅ 100% | エンコード自動検出 |
| PDFローダー | ✅ 100% | 並列ページ処理 |
| DOCXローダー | ✅ 100% | テーブル対応 |
| XLSXローダー | ✅ 100% | 複数シート対応 |
| 並列ローダー | ✅ 100% | 10並列処理 |
| Ollama API | ✅ 100% | 完全実装 |
| テキスト分割 | ✅ 100% | Unicode対応 |
| ベクトルDB | ✅ 100% | ChromaDB完全実装 |
| RAGパイプライン | ✅ 100% | インデックス&検索完成 |
| エージェントモード | ✅ 100% | 自律検索実装 |
| CLIツール | ✅ 100% | インタラクティブデモ |
| GUI基盤 | ✅ 100% | eframe/egui完成 |
| GUIサイドバー | ✅ 100% | モデル選択、インデックス管理 |
| GUIチャット | ✅ 100% | ストリーミング対応 |
| GUIバックエンド | ✅ 100% | 非同期処理完成 |

**現在の進捗**: フェーズ1-3完了！100%（26/29タスク）🎉

## ライセンス

MIT

## Python版との比較

| 項目 | Python版 | Rust版（目標） |
|------|----------|---------------|
| 起動時間 | 5-10秒 | 0.5-2秒 |
| メモリ使用量 | 300-500MB | 50-100MB |
| 100ファイルスキャン | 30-60秒 | 3-6秒 |
| PDF 100ページ | 20秒 | 2-3秒 |
| バイナリサイズ | 500MB+ | 30-80MB |
| 依存関係管理 | pip/conda | Cargo（統一） |
| 配布 | PyInstaller | 単一バイナリ |

## 次のステップ（拡張機能）

1. ✅ ~~Ollama APIクライアント完成~~ **完了！**
2. ✅ ~~テキスト分割器実装~~ **完了！**
3. ✅ ~~ChromaDB連携実装~~ **完了！**
4. ✅ ~~RAGパイプライン実装~~ **完了！**
5. ✅ ~~CLI版で動作確認~~ **完了！**
6. ✅ ~~GUI実装（eframe/egui）~~ **完了！**
7. モデルマネージャーウィンドウ
8. 設定画面
9. パフォーマンスベンチマーク
10. Windows/Linux/macOSビルド

**現在の状況**: コア機能100%完成！拡張機能と最適化を継続中
