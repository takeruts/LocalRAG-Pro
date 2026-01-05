# LocalRAG Pro Rust版 - 使い方ガイド

## 起動方法

### 方法1: バッチファイルで起動（簡単）

プロジェクトルートで以下のバッチファイルをダブルクリック：

```
LocalRAG-Pro/
├── run_rust_gui.bat              # 開発版（デバッグ情報付き、起動速い）
└── run_rust_gui_release.bat      # リリース版（最適化、最高速）
```

### 方法2: PowerShellスクリプトで起動

```powershell
cd C:\Users\taker\LocalRAG-Pro\localrag-rust
.\run_gui.ps1
```

### 方法3: コマンドラインから直接起動

```powershell
# UTF-8設定
chcp 65001
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

# 開発版実行
cd C:\Users\taker\LocalRAG-Pro\localrag-rust
cargo run -p rag-gui

# リリース版実行（初回は10-15分かかります）
cargo build --release -p rag-gui
.\target\release\rag-gui.exe
```

## 使用前の準備

### 1. Ollamaのインストールと起動

```powershell
# Ollamaがインストールされているか確認
ollama --version

# モデルをダウンロード（初回のみ）
ollama pull gemma2:2b           # LLMモデル
ollama pull nomic-embed-text    # Embeddingモデル

# Ollamaが起動していることを確認
ollama list
```

### 2. ChromaDBの起動

**重要**: Rust版GUIを使用する前に、ChromaDBサーバーを起動する必要があります。

#### 方法1: 起動スクリプトを使用（簡単・推奨）

プロジェクトルートで以下のスクリプトをダブルクリック：

```
LocalRAG-Pro/
├── start_chromadb.bat      # Windowsバッチ版
└── start_chromadb.ps1       # PowerShell版
```

#### 方法2: コマンドラインから手動起動

```powershell
# Pythonの仮想環境をアクティベート
cd C:\Users\taker\LocalRAG-Pro
.\.venv\Scripts\Activate.ps1

# ChromaDBを起動
chroma run --path ./chroma_db --port 8000
```

#### 方法3: Dockerを使用

```powershell
docker run -p 8000:8000 chromadb/chroma
```

**注意**: ChromaDBサーバーが起動していない状態でインデックス作成を実行すると、接続エラーが表示されます。

## GUIアプリの使い方

### 1. 起動画面

アプリが起動すると、以下の画面が表示されます：

```
┌─────────────────────────────────────────────────────┐
│ ⚡ LocalRAG Pro                                     │
├─────────────────────────────────────────────────────┤
│ ● Ollama 実行中                        🔄          │
│                                                     │
│ 💬 LLMモデル                                        │
│ [gemma2:2b                            ▼]           │
│                                                     │
│ 🔢 Embeddingモデル                                  │
│ [nomic-embed-text                     ▼]           │
│                                                     │
│ 📁 ドキュメントフォルダ                              │
│ [フォルダを選択してください              ]          │
│ [📂 フォルダを選択                    ]             │
│ [⚡ インデックス作成                  ]             │
│                                                     │
│ 📊 インデックス統計                                 │
│ Total Files: 0                                      │
│ Indexed: 0                                          │
│ Chunks: 0                                           │
│ Embeddings: 0                                       │
└─────────────────────────────────────────────────────┘
```

### 2. ドキュメントをインデックス化

1. **フォルダを選択** ボタンをクリック
2. PDFやDOCX、XLSX、TXTファイルがあるフォルダを選択
3. **インデックス作成** ボタンをクリック
4. 進捗バーで処理状況を確認

```
[████████████████████████] 100% (42 files)
✅ Indexing complete!
   Total files: 42
   Indexed: 42
   Chunks: 1,234
   Embeddings: 1,234
```

### 3. 質問する

右側のチャットエリアで：

1. テキスト入力欄に質問を入力
2. **Enter**キーまたは **→** ボタンで送信
3. リアルタイムでストリーミング応答が表示される

**通常モード**:
```
ユーザー: RAGとは何ですか？

アシスタント: RAG (Retrieval-Augmented Generation) は...
              [ストリーミング表示]

📚 ソース情報
  1. rag_basics.pdf (P.1) - Score: 0.89
  2. architecture.md - Score: 0.85
```

**エージェントモード** (🤖トグルをON):
- 自動的にキーワードを抽出
- 並列で複数検索を実行
- より包括的な回答を生成

```
🔍 Analyzing question...
🔑 Keywords: RAG, 検索拡張生成, ベクトルDB
🔍 Searching: RAG
🔍 Searching: 検索拡張生成
📚 Found 15 documents
✨ Generating answer...
```

## パフォーマンス比較

| 項目 | Python版 | Rust版（開発） | Rust版（リリース） |
|------|---------|-------------|----------------|
| 起動時間 | 5-10秒 | 1-2秒 | <1秒 |
| 100ファイルスキャン | 30-60秒 | 5-8秒 | 3-6秒 |
| PDF 100ページ | 20秒 | 3-4秒 | 2-3秒 |
| メモリ使用量 | 300-500MB | 120-150MB | 100-150MB |

## トラブルシューティング

### Q: Ollamaに接続できない

**A**:
```powershell
# Ollamaが起動しているか確認
ollama list

# 起動していない場合
ollama serve
```

### Q: ChromaDBに接続できない

**A**:
```powershell
# ChromaDBが起動しているか確認（別ターミナルで）
chroma run --path ./chroma_db --port 8000

# またはDockerで
docker run -p 8000:8000 chromadb/chroma
```

### Q: ビルドが遅い

**A**:
- 初回ビルドは依存関係のダウンロードで5-10分かかります
- リリースビルドはLTO最適化で10-15分かかります
- 2回目以降は数秒で完了します

### Q: 日本語が文字化けする（コンソールログ）

**A**: GUIウィンドウ内の表示は正常です。コンソールログの文字化けを防ぐには：

```powershell
chcp 65001
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
```

または、付属の起動スクリプトを使用してください（自動設定されます）。

### Q: モデルが表示されない

**A**:
```powershell
# モデルをダウンロード
ollama pull gemma2:2b
ollama pull nomic-embed-text

# GUIで🔄ボタンをクリックしてリフレッシュ
```

## 推奨設定

### LLMモデル（応答生成用）
- **軽量**: gemma2:2b (2GB、高速）
- **バランス**: qwen2.5:7b (4GB、高品質）
- **高性能**: qwen2.5:14b (8GB、最高品質）

### Embeddingモデル（検索用）
- **推奨**: nomic-embed-text (768次元、高速・高精度）
- **代替**: all-minilm (384次元、超高速）

## キーボードショートカット

- **Enter**: メッセージ送信
- **Esc**: （将来実装予定）

## ファイル形式サポート

- ✅ PDF (.pdf)
- ✅ Word (.docx)
- ✅ Excel (.xlsx, .xls)
- ✅ テキスト (.txt)

## システム要件

- **OS**: Windows 10/11（64-bit）
- **メモリ**: 4GB以上（8GB推奨）
- **ストレージ**: 2GB以上の空き容量
- **必須ソフトウェア**:
  - Rust (https://rustup.rs/)
  - Ollama (https://ollama.ai/)
  - ChromaDB (Python: `pip install chromadb`)

## サポート

問題が発生した場合は、以下を確認してください：

1. Ollamaが起動しているか
2. ChromaDBが起動しているか（ポート8000）
3. モデルがダウンロードされているか（`ollama list`）
4. Rustが正しくインストールされているか（`cargo --version`）

ログを確認：
```powershell
# ログレベルを変更
$env:RUST_LOG="debug"
cargo run -p rag-gui
```
