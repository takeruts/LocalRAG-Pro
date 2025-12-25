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
- **🎯 High-Precision Retrieval**: ベクトル検索（PLamo/E5）＋ AIリランカー（BGE）構成。
- **⚡ Smart UX**: 差分インデックス（既登録スキップ）、中断・再開機能、可変サイドバー搭載。
- **📄 Evidence Tracking**: 回答の根拠となったPDFの該当ページをブラウザ（Edge等）で自動表示。

---

## 🏗️ Architecture / 構成図



1. **Ingestion**: `PyMuPDF` 等で文書をロード。
2. **Indexing**: メタデータを自動洗浄し `ChromaDB` へバッチ登録。
3. **Retrieval**: ベクトル検索後、`Cross-Encoder` で再ランキングして精度を向上。
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

## 🛠️ How to Build / 実行ファイルの作成方法

本リポジトリにはビルド済みの `.exe` ファイルも含まれていますが（リリースセクション参照）、自身でビルドする場合は仮想環境を有効化した状態で以下を実行してください。

### 1. ビルド用ツールの準備

```powershell
pip install pyinstaller
```

### 2. ビルドコマンド

```powershell
pyinstaller --noconsole --onedir --noconfirm --clean --name "LocalRAG_Pro" `
 --add-data ".venv\Lib\site-packages\customtkinter;customtkinter" `
 --collect-all langchain `
 --collect-all sentence_transformers `
 --collect-all chromadb `
 --collect-all transformers `
 win_rag.py
```

※ ビルド完了後、`dist/LocalRAG_Pro/` フォルダ内に実行ファイルが生成されます。

---

## 📜 License

MIT License

```