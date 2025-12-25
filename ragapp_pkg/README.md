
---

# 📂 Local RAG System Pro

完全にローカル環境で動作する、プライバシー重視の高精度RAG（検索拡張生成）システムです。
国産の高性能 Embedding モデル「PLamo」と、検索精度を向上させる「リランカー」を搭載し、Intel CPU 向けに最適化されています。

## ✨ 主な特徴

* **完全ローカル動作**: データが外部サーバーに送信されることはありません。
* **マルチモデル対応**:
* **Embedding**: Multilingual-E5 (高速) / PLamo Embedding 1B (高精度・国産)
* **LLM**: Gemma 3:4b (Ollama経由)


* **高精度検索**: BGE-Reranker による再順位付け（Reranking）機能を搭載。
* **Intel PC 最適化**: AVX-512 命令セット等を利用した高速な推論設定。
* **多様なファイル形式**: PDF, PPTX, DOCX, XLSX, TXT を軽量に処理。

## 🚀 セットアップ方法

### 1. 前提条件

* **Ollama**: [公式HP](https://ollama.com/) からインストールし、以下のモデルをプルしておいてください。
```powershell
ollama pull gemma3:4b

```



### 2. 環境構築

バッチファイルで `.venv` を使用するため、以下の手順で仮想環境を作成します。

```powershell
# 仮想環境の作成
python -m venv .venv

# 必要なライブラリのインストール
.\.venv\Scripts\pip install streamlit langchain langchain-community langchain-huggingface chromadb pymupdf python-pptx docx2txt sentence-transformers openpyxl

```

### 3. モデルの準備

初回起動時にモデルが自動的にダウンロードされますが、オフライン環境へ持ち出す場合は、一度オンライン環境で全てのモデルを選択してロードを完了させてください。

## 🛠 使い方

1. **起動**: `run.bat` をダブルクリックします。
2. **フォルダ選択**: サイドバーの「フォルダを選択」から、ドキュメントが入ったフォルダを指定します。
3. **スキャン**: 「スキャン開始」ボタンを押し、ベクターDBを構築します。
4. **チャット**: 下部のチャット入力欄から質問を投げてください。
* *Tip: 精度を上げたい場合は「リランカーを有効にする」をONにしてください。*



## 📁 プロジェクト構成

```text
.
├── ragapp.py         # アプリ本体
├── run.bat           # 起動用バッチファイル
├── .venv/            # Python仮想環境
├── models/           # AIモデルのキャッシュ（自動生成）
└── chroma_db/        # ベクターデータベース（自動生成）

```

---

## ⚠️ トラブルシューティング

* **起動しない**: 8501番ポートが他のアプリ（別のStreamlitなど）に使われていないか確認してください。
* **接続拒否**: Ollama がバックグラウンドで起動しているか確認してください。
* **メモリ不足**: PLamo Embedding や リランカーはメモリを消費します。動作が重い場合はリランカーをOFFにし、E5モデルを選択してください。

---

## ⚖️ 免責事項 (Disclaimer)
* 本ツールはローカル環境での利用を前提としており、機密情報の取り扱いには十分注意してください。
* 生成された回答は必ずしも正確であるとは限りません。重要な判断を行う際は必ず一次ソース（参照元ファイル）を確認してください。
* 本ツールの利用によって生じた損害等について、開発者は一切の責任を負いません。

## 📄 ライセンス (License)
本プロジェクトのソースコードは MIT License で公開されています。
ただし、同梱または利用される以下のモデル・ライブラリについては、それぞれのライセンスに従ってください。
- [Ollama / Gemma 3](https://ollama.com/library/gemma3)
- [PFN PLamo Embedding](https://huggingface.co/pfnet/plamo-embedding-1b)
- [LangChain](https://github.com/langchain-ai/langchain)
