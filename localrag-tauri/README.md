# LocalRAG Pro - Tauri Edition

モダンなデスクトップRAGアプリケーション。Tauri v2 + React + TypeScriptで構築。

## 概要

LocalRAG Pro Tauri Editionは、ローカル環境で動作するRAG（Retrieval-Augmented Generation）システムのデスクトップアプリケーションです。プライバシーを重視し、機密文書を外部に送信することなくAIによる文書検索・質問応答が可能です。

### 主な特徴

- **完全ローカル動作**: Ollama + ChromaDBによるオフラインRAG
- **モダンUI**: React + TailwindCSSによる美しいインターフェース
- **自動更新**: Tauri Updaterによるアプリの自動更新
- **クロスプラットフォーム**: Windows対応（macOS/Linux対応予定）
- **軽量・高速**: Rust製バックエンドによる高いパフォーマンス

## 技術スタック

| カテゴリ | 技術 |
|---------|------|
| フレームワーク | Tauri v2 |
| フロントエンド | React 18 + TypeScript |
| スタイリング | TailwindCSS |
| ビルドツール | Vite |
| バックエンド | Rust |
| LLM | Ollama |
| ベクトルDB | ChromaDB |

## 必要条件

- **Node.js**: 18.0以上
- **Rust**: 1.75以上
- **Ollama**: インストール済み
- **Python**: 3.10以上（ChromaDB用）

## セットアップ

### 1. 依存関係のインストール

```bash
cd localrag-tauri
npm install
```

### 2. Ollamaモデルのインストール

```bash
ollama pull gemma2:2b
ollama pull nomic-embed-text
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
│   │   ├── Sidebar.tsx       # サイドバー
│   │   └── SourceInfo.tsx    # ソース情報表示
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
│   │   │   ├── models.rs     # モデル管理
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

### ドキュメントインデックス

- フォルダ選択によるバッチインデックス
- 対応フォーマット: PDF, DOCX, XLSX, TXT, MD
- 差分インデックス（既存ファイルスキップ）
- リアルタイム進捗表示

### RAGクエリ

- 自然言語での質問応答
- ストリーミングレスポンス
- ソースドキュメントの表示
- 関連度スコアの表示

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

### 署名付きビルド（Windows）

```powershell
.\build_signed.ps1
```

## 設定

### tauri.conf.json

主要な設定項目:

```json
{
  "productName": "LocalRAG Pro",
  "version": "3.0.0",
  "identifier": "com.localrag.pro",
  "plugins": {
    "updater": {
      "endpoints": [
        "https://raw.githubusercontent.com/takeruts/LocalRAG-Pro/main/latest.json"
      ]
    }
  }
}
```

### 自動更新の設定

`latest.json`でリリース情報を管理:

```json
{
  "version": "3.0.0",
  "platforms": {
    "windows-x86_64": {
      "url": "https://github.com/.../releases/download/v3.0.0/LocalRAG-Pro_3.0.0_x64-setup.exe",
      "signature": "..."
    }
  }
}
```

## リリース

GitHub Actionsによる自動リリース:

1. タグをプッシュ: `git tag v3.0.0 && git push --tags`
2. GitHub Actionsがビルドを実行
3. 署名付きインストーラーがReleasesにアップロード
4. `latest.json`が自動更新

## ライセンス

MIT License
