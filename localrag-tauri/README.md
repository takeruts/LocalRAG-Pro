# CPURAG - Tauri Edition

モダンなデスクトップRAGアプリケーション。Tauri v2 + React + TypeScript + Rustで構築。

## 概要

CPURAG Tauri Editionは、ローカル環境で動作するRAG（Retrieval-Augmented Generation）システムのデスクトップアプリケーションです。プライバシーを重視し、機密文書を外部に送信することなくAIによる文書検索・質問応答が可能です。

### 主な特徴

- **完全ローカル動作**: Ollama + HNSWによるオフラインRAG
- **モダンUI**: React + TailwindCSSによる美しいインターフェース
- **高速エンベディング**: バッチ処理・並列リクエストによる最適化
- **差分インデックス**: 既にインデックス済みのファイルは自動スキップ
- **関連ドキュメント表示**: RAG回答後にファイル名・フォルダ・ページ番号を表示
- **CPU情報表示**: サイドバーにCPUモデル、コア数、周波数を表示
- **自動更新**: Tauri Updaterによるアプリの自動更新
- **クロスプラットフォーム**: Windows対応（macOS/Linux対応予定）
- **軽量・高速**: Rust製バックエンド + HNSWベクトル検索による高いパフォーマンス

## 技術スタック

| カテゴリ | 技術 |
|---------|------|
| フレームワーク | Tauri v2 |
| フロントエンド | React 18 + TypeScript |
| スタイリング | TailwindCSS |
| ビルドツール | Vite |
| バックエンド | Rust (rag-core) |
| LLM | Ollama |
| ベクトルDB | HNSW (instant-distance) |
| システム情報 | sysinfo |

## インストール（配布版）

### Windows

1. [Releases](https://github.com/takeruts/LocalRAG-Pro/releases)から最新のインストーラーをダウンロード
   - `CPURAG_x.x.x_x64_en-US.msi` (MSI) または
   - `CPURAG_x.x.x_x64-setup.exe` (NSIS)
2. インストーラーを実行
3. [Ollama](https://ollama.ai)をインストール
4. 必要なモデルをダウンロード:
   ```bash
   ollama pull gemma3:4b
   ollama pull bge-m3
   ```

### データ保存場所

- **インデックスデータ**: `%LOCALAPPDATA%\com.cpurag.app\vectordb_data\`
- インデックスをリセットしたい場合は上記フォルダを削除してください

## 必要条件（開発）

- **Node.js**: 18.0以上
- **Rust**: 1.75以上
- **Ollama**: インストール済み

## セットアップ（開発）

### 1. 依存関係のインストール

```bash
cd localrag-tauri
npm install
```

### 2. Ollamaモデルのインストール

```bash
ollama pull gemma3:4b
ollama pull bge-m3
```

### 3. 開発サーバーの起動

```bash
npm run tauri dev
```

### 4. プロダクションビルド

```bash
npm run tauri build
```

## プロジェクト構造

```
localrag-tauri/
├── src/                      # React フロントエンド
│   ├── components/           # UIコンポーネント
│   │   ├── ChatArea.tsx      # チャットエリア
│   │   ├── MessageBubble.tsx # メッセージ表示
│   │   ├── Sidebar.tsx       # サイドバー（CPU情報含む）
│   │   ├── SourceInfo.tsx    # ソース情報表示
│   │   └── OllamaSetupGuide.tsx # Ollamaセットアップガイド
│   ├── hooks/                # カスタムフック
│   │   ├── useBackend.ts     # バックエンド通信
│   │   └── useUpdater.ts     # 自動更新
│   ├── styles/               # スタイル
│   │   └── index.css         # TailwindCSS
│   ├── App.tsx               # メインアプリ
│   ├── main.tsx              # エントリーポイント
│   └── types.ts              # 型定義
├── src-tauri/                # Rust バックエンド
│   ├── src/
│   │   ├── commands/         # Tauriコマンド
│   │   │   ├── indexing.rs   # インデックス処理
│   │   │   ├── models.rs     # モデル管理・システム情報
│   │   │   └── query.rs      # クエリ処理
│   │   ├── lib.rs            # ライブラリエントリ
│   │   ├── main.rs           # アプリエントリ
│   │   └── state.rs          # アプリ状態管理
│   ├── Cargo.toml            # Rust依存関係
│   └── tauri.conf.json       # Tauri設定
├── package.json              # Node.js依存関係
├── tailwind.config.js        # TailwindCSS設定
├── vite.config.ts            # Vite設定
└── tsconfig.json             # TypeScript設定
```

## 機能

### システム情報表示

- CPUモデル名の表示
- コア数の表示
- 動作周波数の表示（GHz）
- Ollamaステータスの監視（安定したヒステリシス付き）

### ドキュメントインデックス

- フォルダ選択によるバッチインデックス
- 対応フォーマット: PDF, DOCX, XLSX, TXT, MD
- 差分インデックス（既存ファイルスキップ）
- リアルタイム進捗表示（フェーズ別: Loading → Splitting → Embedding → Storing）
- 高速バッチエンベディング（8チャンク×8並列リクエスト）
- HTTP接続最適化（コネクションプーリング、TCP Keep-Alive）

### RAGクエリ

- 自然言語での質問応答
- ストリーミングレスポンス
- 関連ドキュメントの詳細表示
  - ファイル名（大きく表示）
  - フォルダパス
  - ページ番号（PDFの場合）
  - 関連度スコア（パーセント表示）

### モデル管理

- インストール済みモデルの一覧表示
- LLMモデルの選択
- Embeddingモデルの選択

### 自動更新

- GitHub Releasesからの自動更新チェック
- ワンクリックアップデート
- バージョン情報表示

## 開発

### 開発モードで起動

```bash
npm run tauri dev
```

### フロントエンドのみ起動

```bash
npm run dev
```

### ビルド

```bash
npm run tauri build
```

## 設定

### tauri.conf.json

主要な設定項目:

```json
{
  "productName": "CPURAG",
  "version": "1.0.0",
  "identifier": "com.cpurag.app",
  "plugins": {
    "updater": {
      "endpoints": [
        "https://raw.githubusercontent.com/takeruts/LocalRAG-Pro/main/latest.json"
      ]
    }
  }
}
```

## リリース

GitHub Actionsによる自動リリース:

1. タグをプッシュ: `git tag v1.0.0 && git push --tags`
2. GitHub Actionsがビルドを実行
3. 署名付きインストーラーがReleasesにアップロード
4. `latest.json`が自動更新

## ライセンス

MIT License
