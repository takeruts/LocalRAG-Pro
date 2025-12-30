# Local RAG Pro: Secure Desktop Document Intelligence 🛡️🤖

![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)
![Python: 3.10+](https://img.shields.io/badge/python-3.10%2B-blue.svg)
![Ollama: Supported](https://img.shields.io/badge/Ollama-Supported-orange.svg)
![AI-Generated: Yes](https://img.shields.io/badge/AI--Generated-Code-blueviolet)

**Local RAG Pro** は、機密情報を一切クラウドに送信することなく、PCローカル環境でPDFやOffice文書を解析・検索できる、プライバシー重視のデスクトップRAGシステムです。

---

## 🤖 AI-Assisted Development / AIによる開発について
- **English:** This software was developed with significant assistance from Large Language Models (LLM). The core logic, including retrieval strategies, UI implementation, and robust error handling, was generated and refined through interaction with AI.
- **日本語:** 本ソフトウェアは、大規模言語モデル（LLM）による高度なコード生成・修正支援を受けて開発されました。検索アルゴリズム、UI実装、および例外処理等の主要ロジックにはAIによって生成・最適化されたコードが含まれています。

---

## 🌟 Key Features / 主な特長

- **🔒 100% Local & Private**: 外部APIキー不要。データ流出の心配はありません。
- **🤖 AI Agent Mode**: AIが質問を分析し、最適な検索キーワードを自律的に生成・実行。
- **🎯 High-Precision Retrieval**: ベクトル検索（PLamo/E5）＋ AIリランカー（BGE）構成。
- **⚡ Smart UX**: 差分インデックス（既登録スキップ）、中断・再開機能、可変サイドバー搭載。
- **📄 Evidence Tracking**: 回答の根拠となったPDFの該当ページをブラウザ（Edge等）で自動表示。
- **🧠 Transparent Reasoning**: エージェントの思考過程をリアルタイムで可視化。

---

## 🏗️ Architecture / 構成図



1. **Ingestion**: `PyMuPDF` 等で文書をロード。
2. **Indexing**: メタデータを自動洗浄し `ChromaDB` へバッチ登録。
3. **Retrieval**:
   - **通常モード**: ベクトル検索後、`Cross-Encoder` で再ランキング
   - **エージェントモード**: AIが検索キーワードを生成 → 複数キーワードで検索 → 重複除外 → 再ランキング → 資料十分性チェック
4. **Generation**: `Ollama` (Gemma3等) を用いたローカル推論。

---

## 🚀 準備・インストール

### 1. 前提条件 (Ollama)
本アプリの知能（LLM）には **Ollama** を使用します。
1. [Ollama公式サイト](https://ollama.com/) からインストールします。
2. 以下のコマンドを実行してモデルをダウンロードします。
```powershell
ollama pull gemma3:4b
```

### 2. 環境構築 (Python環境で動かす場合)

1. **仮想環境の作成と有効化**:

```powershell
python -m venv .venv
.\.venv\Scripts\activate
```

2. **依存ライブラリのインストール**:

```powershell
pip install -r requirements.txt
```

3. **Pythonの入出力をUTF-8に強制**:
```powershell
$env:PYTHONUTF8 = "1"
```
---

## 🌐 Proxy Settings / 社内ネットワーク環境での設定

社内プロキシ環境でモデルをダウンロードする際は設定が必要ですが、**ローカルで動作する Ollama との通信にはプロキシをバイパスする設定（NO_PROXY）が必須**です。

**PowerShellでの設定例:**

```powershell
# 外部接続用（モデルダウンロードに必要）
$env:HTTP_PROXY="http://your-proxy-server:port"
$env:HTTPS_PROXY="http://your-proxy-server:port"

# ローカルバイパス用（Ollamaとの通信に必須）
$env:NO_PROXY="localhost,127.0.0.1"
```
同じシェルから、LcalRAG_Pro を実行 
```powershell
./LocalRAG_Pro.exe
```
---

## 🤖 AI Agent Mode / エージェントモード

### 概要

エージェントモードは、AIが自律的に資料検索を行う高度な機能です。通常モードでは質問文そのままで検索しますが、エージェントモードではAIが質問を分析し、最適な検索戦略を立てます。

### エージェントの動作フロー

```
質問入力
  ↓
🤔 質問分析（AIが検索キーワードを3つ生成）
  ↓
🔍 複数キーワード検索（並列実行、重複除外）
  ↓
📚 資料収集（全キーワードからの結果を統合）
  ↓
🎯 リランキング（最も関連性の高い上位5件を選定）
  ↓
🧠 資料十分性チェック（質問に答えられるか判断）
  ↓
💬 詳細な回答生成（参照箇所を明示）
```

### 使用例

#### 通常モード向きの質問
```
「この製品の保証期間は？」
→ 単純な情報検索、1回の検索で十分
```

#### エージェントモード向きの質問（推奨）

**比較分析**
```
「A製品とB製品を比較して、どちらがコストパフォーマンスが高いか？」
→ AIが「A製品 価格」「B製品 価格」「性能比較」などで自律検索
```

**複合的な質問**
```
「この技術の利点と欠点、および実装時の注意点は？」
→ AIが「技術 メリット」「技術 デメリット」「実装 注意事項」で検索
```

**曖昧な質問**
```
「最近の業界動向について教えて」
→ AIが「業界 トレンド」「最新技術」「市場動向」など関連キーワードで網羅的に検索
```

### エージェントの思考過程の可視化

エージェントモードでは、以下のような情報がリアルタイムで表示されます：

```
【Agent】🤔 質問を分析中...
【Agent】💡 検索キーワード: 価格比較, 性能ベンチマーク, ユーザーレビュー
【Agent】🔍 「価格比較」で検索中...
【Agent】🔍 「性能ベンチマーク」で検索中...
【Agent】🔍 「ユーザーレビュー」で検索中...
【Agent】📚 12件の資料を発見
【Agent】🎯 最も関連性の高い資料を選定中...
【Agent】🧠 資料を読み込んで回答を生成中...
【Assistant】（詳細な回答）
```

### 通常モードとの比較

| 項目 | 通常モード | エージェントモード |
|------|-----------|------------------|
| **検索方法** | 質問文そのまま | AIが最適なキーワードを生成 |
| **検索回数** | 1回 | 最大3回（キーワードごと） |
| **資料の多様性** | 限定的 | 複数の視点から収集 |
| **思考過程** | 非表示 | 全ステップを可視化 |
| **資料数** | 3-10件 | 最大15件から上位5件を選定 |
| **回答の質** | 標準 | より詳細で多角的 |
| **処理時間** | 速い | やや遅い（3倍程度） |
| **推奨用途** | 単純な質問 | 複雑・曖昧な質問 |

### 有効化方法

1. サイドバーの「エージェントモード(自律検索)」スイッチをON
2. リランカーも併用すると更に高精度
3. 質問を入力して送信

### 技術詳細

- **キーワード生成**: Ollamaによるプロンプトベース生成
- **並列検索**: 各キーワードで独立に検索を実行
- **重複除外**: ソースファイル名ベースで重複を排除
- **十分性チェック**: 収集した資料で質問に答えられるかをAIが事前判定
- **回答品質**: 参照箇所を明示した詳細な回答を生成

---

## 🛠️ How to Build / 実行ファイルの作成方法

本リポジトリにはビルド済みの `.exe` ファイルも含まれていますが（リリースセクション参照）、自身でビルドする場合は仮想環境を有効化した状態で以下を実行してください。

### 1. 簡単なビルド方法（推奨）

仮想環境を有効化した状態で、用意されたビルドスクリプトを実行するだけです。

```powershell
.\build.bat
```

このスクリプトは以下を自動的に実行します：
- 仮想環境の検出とアクティベート
- PyInstallerのインストール確認
- 既存ビルド成果物のクリーンアップ
- 実行中のプロセスの自動終了
- win_rag.specを使用した最適化ビルド

### 2. 手動ビルド方法

より細かい制御が必要な場合は、以下の手順で手動ビルドできます。

#### ビルド用ツールの準備

```powershell
pip install pyinstaller
```

#### ビルドコマンド

```powershell
pyinstaller win_rag.spec --clean --noconfirm
```

または、.specファイルを使用しない場合：

```powershell
pyinstaller --name "LocalRAG-Pro" --noconsole --onedir `
 --hidden-import=sentence_transformers --hidden-import=sklearn --hidden-import=scipy `
 --collect-all customtkinter --collect-all sentence_transformers --collect-all chromadb `
 --collect-all langchain_community --collect-all langchain_huggingface `
 --collect-all sklearn --collect-all scipy --collect-all transformers --collect-all tokenizers `
 win_rag.py
```

### 3. ビルド成果物

ビルド完了後、以下の場所に実行ファイルが生成されます：

```
dist/LocalRAG-Pro/
├── LocalRAG-Pro.exe          # メイン実行ファイル
├── _internal/                # 依存ライブラリ（必須）
├── models/                   # AIモデルキャッシュ（実行時に作成）
└── chroma_db/                # ベクトルDB（実行時に作成）
```

**重要**: `LocalRAG-Pro`フォルダ全体を配布してください。実行ファイル単体では動作しません。

### 4. トラブルシューティング

#### ビルドエラーが発生する場合

1. **既存のプロセスを終了**
   ```powershell
   taskkill /F /IM LocalRAG-Pro.exe
   ```

2. **手動でクリーンアップ**
   ```powershell
   Remove-Item -Recurse -Force dist, build
   ```

3. **依存パッケージを再インストール**
   ```powershell
   pip install -r requirements.txt --force-reinstall
   ```

#### モジュールが見つからないエラー

`win_rag.spec`ファイルの`hiddenimports`セクションに不足しているモジュールを追加してください。

#### サイズが大きすぎる場合

不要なパッケージを`excludes`セクションに追加することで、サイズを削減できます。

---

## 📦 配布パッケージの作成

実行ファイルを他のユーザーに配布する場合、以下の手順でパッケージを作成できます。

### 配布パッケージの作成方法

```powershell
.\package.bat
```

このスクリプトは以下を自動的に実行します：

1. **ビルド済み実行ファイルの確認**
2. **配布フォルダの作成**
3. **必要なファイルのコピー**:
   - `LocalRAG-Pro.exe` とすべての依存ファイル
   - `README.md` （詳細ドキュメント）
   - `LICENSE` （ライセンスファイル、存在する場合）
4. **追加ドキュメントの自動生成**:
   - `QUICKSTART.md` - 初めてのユーザー向けガイド
   - `SYSTEM_REQUIREMENTS.md` - システム要件
   - `起動.bat` - 簡単起動スクリプト
5. **ZIPファイルの作成**: `package\LocalRAG-Pro-v1.0.0-Windows.zip`

### 配布パッケージの内容

作成されるZIPファイルには以下が含まれます：

```
LocalRAG-Pro-v1.0.0-Windows/
├── LocalRAG-Pro.exe          # メイン実行ファイル
├── _internal/                # 依存ライブラリ（必須）
├── README.md                 # 詳細ドキュメント
├── QUICKSTART.md             # クイックスタート
├── SYSTEM_REQUIREMENTS.md    # システム要件
└── 起動.bat                  # 簡単起動スクリプト
```

### 配布方法

1. `package\LocalRAG-Pro-v1.0.0-Windows.zip` を配布
2. 受け取った人は以下の手順で使用:
   - ZIPファイルを展開
   - Ollama をインストール（https://ollama.com/）
   - `起動.bat` をダブルクリック、または `LocalRAG-Pro.exe` を直接起動

### 受け取る側の要件

- **Windows 10/11** (64bit)
- **Ollama** のインストールが必要
- 初回起動時は**インターネット接続**が必要（AIモデルのダウンロード）
- **8GB以上のRAM**（16GB推奨）

### バージョン管理

`package.bat` の3行目でバージョン番号を変更できます：

```batch
set VERSION=1.0.0
```

新しいバージョンをリリースする際は、このバージョン番号を更新してから実行してください。

### GitHub での配布方法

実行ファイルは大容量のため、Gitリポジトリには含めません。代わりに **GitHub Releases** を使用して配布します。

#### 1. リリースの作成

1. GitHubリポジトリページで「Releases」→「Create a new release」をクリック
2. タグ名を入力（例: `v1.0.0`）
3. リリースタイトルを入力（例: `LocalRAG-Pro v1.0.0`）
4. リリースノートを記載:
   ```markdown
   ## 新機能
   - 🤖 AIエージェントモード追加
   - 🎯 高精度リランカー統合

   ## システム要件
   - Windows 10/11 (64bit)
   - RAM 8GB以上推奨
   - Ollama インストール必須

   ## インストール方法
   1. LocalRAG-Pro-v1.0.0-Windows.zip をダウンロード
   2. ZIPを展開
   3. 起動.bat を実行
   ```
5. 「Attach binaries」で `package\LocalRAG-Pro-v1.0.0-Windows.zip` をアップロード
6. 「Publish release」をクリック

#### 2. Git から大容量ファイルを削除済みの場合

既に`deploy/`や`dist/`をプッシュしてしまった場合は、以下を実行:

```powershell
.\fix-git.bat
```

このスクリプトは:
- ビルド成果物をGitの追跡から削除（ローカルファイルは保持）
- `.gitignore`を適切に適用
- 変更を自動コミット

その後、プッシュ:
```powershell
git push origin main --force
```

**注意**: `--force`は履歴を書き換えます。他の人と共同作業している場合は事前に相談してください。

#### 3. 推奨ワークフロー

```powershell
# 1. コードの変更
git add .
git commit -m "feat: Add new feature"
git push

# 2. ビルド
.\build.bat

# 3. パッケージ作成
.\package.bat

# 4. GitHub Releases で配布
# → WebUI から package\*.zip をアップロード
```

---

## 📜 License

MIT License

```