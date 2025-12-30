# LocalRAG-Pro ビルドガイド

このガイドでは、LocalRAG-ProをWindows実行ファイル（.exe）にビルドする方法を説明します。

## 📋 前提条件

- Windows 10/11
- Python 3.9以上がインストール済み
- Git（オプション）

## 🚀 ビルド手順

### 1. 依存関係のインストール

まず、必要なPythonパッケージをインストールします。

```cmd
# 仮想環境を作成（推奨）
python -m venv venv
venv\Scripts\activate

# 依存パッケージをインストール
pip install -r requirements.txt
```

### 2. ビルドの実行

**方法A: バッチファイルを使用（推奨・簡単）**

```cmd
# build.bat を実行するだけ
build.bat
```

**方法B: PyInstallerを直接使用**

```cmd
# カスタムspecファイルを使用
pyinstaller win_rag.spec
```

**方法C: コマンドラインで詳細指定**

```cmd
pyinstaller --name "LocalRAG-Pro" ^
    --noconsole ^
    --onedir ^
    --add-data "models;models" ^
    --hidden-import=langchain_community ^
    --hidden-import=sentence_transformers ^
    win_rag.py
```

### 3. 実行ファイルの確認

ビルドが成功すると、以下の場所に実行ファイルが生成されます：

```
dist/
└── LocalRAG-Pro/
    ├── LocalRAG-Pro.exe  ← これが実行ファイル
    ├── _internal/        ← 依存ライブラリ（削除しないこと）
    └── ...
```

## 📦 配布方法

### 配布用のZIPファイル作成

```cmd
# dist/LocalRAG-Pro フォルダをZIP圧縮
# （Windowsエクスプローラーで右クリック → 送る → 圧縮フォルダー）
```

配布時には以下を含めてください：
- `dist/LocalRAG-Pro/` フォルダ全体
- `README.md`（使い方）

## ⚙️ ビルドオプションの説明

### `--onedir` vs `--onefile`

- **`--onedir`（推奨）**:
  - フォルダ形式で出力
  - 起動が速い
  - サイズ: 500MB～1GB程度

- **`--onefile`**:
  - 単一ファイルで出力
  - 起動が遅い（初回展開が必要）
  - 配布は簡単

### サイズ削減のコツ

1. **不要なパッケージを除外**
   ```cmd
   --exclude-module matplotlib
   --exclude-module scipy
   ```

2. **UPX圧縮を有効化**（デフォルトで有効）
   ```cmd
   --upx-dir=C:\path\to\upx
   ```

3. **モデルファイルは外部化**
   - AIモデル（数GB）は実行ファイルに含めない
   - 初回起動時に自動ダウンロードさせる（現在の実装）

## 🐛 トラブルシューティング

### エラー: `ModuleNotFoundError: No module named 'xxx'`

**解決策**: 隠しインポートを追加

```cmd
pyinstaller --hidden-import=xxx win_rag.py
```

または、`win_rag.spec` の `hiddenimports` リストに追加：

```python
hiddenimports=['xxx', 'yyy', ...]
```

### エラー: `FileNotFoundError: customtkinter`

**解決策**: CustomTkinterのデータファイルを追加

```cmd
pyinstaller --add-data "path\to\customtkinter;customtkinter" win_rag.py
```

### 実行ファイルが起動しない

**デバッグ方法**:

1. コンソールモードで実行してエラーを確認
   ```cmd
   # win_rag.spec で console=True に変更
   console=True
   ```

2. ログを確認
   ```cmd
   # dist/LocalRAG-Pro/LocalRAG-Pro.exe を
   # コマンドプロンプトから実行
   cd dist\LocalRAG-Pro
   LocalRAG-Pro.exe
   ```

### 実行ファイルが大きすぎる

**対策**:

1. 不要なパッケージを除外（`--exclude-module`）
2. `--onedir` を使用
3. モデルファイルを外部化
4. Python 3.11以降を使用（サイズが小さくなる）

## 📝 注意事項

### セキュリティ

- Windows Defenderが誤検知する場合があります
- 配布前にウイルススキャンを推奨します
- コード署名証明書があれば署名することを推奨

### ライセンス

- 配布する際は使用しているライブラリのライセンスを確認してください
- 主要ライブラリのライセンス:
  - LangChain: MIT
  - CustomTkinter: MIT
  - ChromaDB: Apache 2.0
  - Sentence Transformers: Apache 2.0

### パフォーマンス

- 初回起動時は数秒～数十秒かかります（通常の動作）
- AIモデルのダウンロードが発生する場合は数分かかります

## 🔧 高度な設定

### アイコンの追加

```cmd
pyinstaller --icon=myicon.ico win_rag.py
```

### バージョン情報の追加

```cmd
pyinstaller --version-file=version.txt win_rag.py
```

### デジタル署名

```cmd
# signtool.exe を使用
signtool sign /f certificate.pfx /p password dist\LocalRAG-Pro\LocalRAG-Pro.exe
```

## 📚 参考資料

- [PyInstaller公式ドキュメント](https://pyinstaller.org/)
- [CustomTkinter GitHub](https://github.com/TomSchimansky/CustomTkinter)
- [LangChain Documentation](https://python.langchain.com/)

## 🆘 サポート

問題が発生した場合は、以下を確認してください：

1. Python バージョン: `python --version`
2. PyInstaller バージョン: `pyinstaller --version`
3. エラーメッセージの全文
4. 使用しているOSのバージョン

---

**ビルド成功を祈ります！ 🎉**
