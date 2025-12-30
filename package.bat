@echo off
chcp 65001 >nul
echo ========================================
echo   LocalRAG-Pro 配布パッケージ作成
echo ========================================
echo.

REM バージョン情報
set VERSION=1.0.0
set PACKAGE_NAME=LocalRAG-Pro-v%VERSION%-Windows

REM 配布フォルダのパス
set DIST_DIR=dist\LocalRAG-Pro
set PACKAGE_DIR=package\%PACKAGE_NAME%
set ZIP_FILE=package\%PACKAGE_NAME%.zip

REM ビルド済み実行ファイルの確認
if not exist "%DIST_DIR%\LocalRAG-Pro.exe" (
    echo [ERROR] ビルド済みの実行ファイルが見つかりません。
    echo [INFO] 先に build.bat を実行してください。
    pause
    exit /b 1
)

echo [INFO] 配布パッケージを作成します...
echo [INFO] バージョン: %VERSION%
echo.

REM 既存のパッケージフォルダを削除
if exist "package" (
    echo [INFO] 既存のパッケージフォルダを削除中...
    rd /s /q "package" 2>NUL
)

REM パッケージフォルダを作成
echo [INFO] パッケージフォルダを作成中...
mkdir "%PACKAGE_DIR%" 2>NUL

REM 実行ファイルと依存ファイルをコピー
echo [INFO] 実行ファイルをコピー中...
xcopy "%DIST_DIR%" "%PACKAGE_DIR%" /E /I /Y >nul
if errorlevel 1 (
    echo [ERROR] ファイルのコピーに失敗しました。
    pause
    exit /b 1
)

REM READMEをコピー
if exist "README.md" (
    echo [INFO] READMEをコピー中...
    copy README.md "%PACKAGE_DIR%\README.md" >nul
)

REM ライセンスファイルをコピー（存在する場合）
if exist "LICENSE" (
    echo [INFO] LICENSEをコピー中...
    copy LICENSE "%PACKAGE_DIR%\LICENSE" >nul
)

REM クイックスタートガイドを作成
echo [INFO] クイックスタートガイドを作成中...
(
echo # LocalRAG-Pro クイックスタートガイド
echo.
echo ## インストール不要
echo.
echo このフォルダをそのまま任意の場所にコピーしてください。
echo Python のインストールは不要です。
echo.
echo ## 必要な環境
echo.
echo 1. **Windows 10/11** ^(64bit^)
echo 2. **Ollama** - AI モデルサーバー
echo    - https://ollama.com/ からダウンロード・インストール
echo    - インストール後、以下のコマンドを実行:
echo      ```
echo      ollama pull gemma3:4b
echo      ```
echo.
echo ## 起動方法
echo.
echo 1. LocalRAG-Pro.exe をダブルクリック
echo 2. 初回起動時は AI モデルのダウンロードが発生します（数分〜数十分）
echo 3. フォルダを選択してインデックス化を開始
echo.
echo ## 使い方
echo.
echo ### 1. フォルダ選択
echo - 「📁 フォルダ選択」ボタンで、検索したい文書が入ったフォルダを選択
echo.
echo ### 2. Embedding モデル選択
echo - **Multilingual-E5-Small**: 軽量・高速（推奨）
echo - **PLamo-Embedding-1B**: 日本語特化・高性能
echo.
echo ### 3. オプション設定
echo - **リランカー**: より高精度な検索（初回は追加ダウンロード）
echo - **エージェントモード**: AI が自律的に最適な検索キーワードを生成
echo.
echo ### 4. インデックス化
echo - 「⚡ Indexing 開始/再開」ボタンをクリック
echo - 既にインデックス済みのファイルはスキップされます
echo.
echo ### 5. 質問
echo - 下部のテキストボックスに質問を入力して送信
echo.
echo ## エージェントモードについて
echo.
echo エージェントモードは複雑な質問に適しています：
echo - 比較分析: 「A製品とB製品を比較して...」
echo - 複合質問: 「利点と欠点、および注意点は？」
echo - 曖昧な質問: 「最近の動向について教えて」
echo.
echo 単純な質問（「価格は？」など）は通常モードで十分です。
echo.
echo ## トラブルシューティング
echo.
echo ### Ollama エラー
echo - Ollama が起動しているか確認: `ollama list`
echo - モデルがインストールされているか確認: `ollama pull gemma3:4b`
echo.
echo ### モデルダウンロードエラー
echo - インターネット接続を確認
echo - プロキシ環境の場合は環境変数を設定
echo.
echo ### メモリ不足
echo - 他のアプリケーションを閉じる
echo - より軽量なモデルを使用（Multilingual-E5-Small）
echo.
echo ## サポートファイル
echo.
echo - `models/` - AI モデルのキャッシュ（自動作成）
echo - `chroma_db/` - ベクトルデータベース（自動作成）
echo.
echo ## 詳細情報
echo.
echo 詳しい使い方は README.md をご覧ください。
echo.
echo ---
echo.
echo LocalRAG-Pro v%VERSION%
echo https://github.com/your-repo/LocalRAG-Pro
) > "%PACKAGE_DIR%\QUICKSTART.md"

REM 起動スクリプト（ランチャー）を作成
echo [INFO] 起動スクリプトを作成中...
(
echo @echo off
echo echo ========================================
echo echo   LocalRAG-Pro を起動します
echo echo ========================================
echo echo.
echo.
echo REM Ollamaの起動確認
echo ollama list ^>nul 2^>^&1
echo if errorlevel 1 ^(
echo     echo [WARNING] Ollama が起動していない可能性があります。
echo     echo [INFO] Ollama をインストールしていない場合は、以下からダウンロードしてください:
echo     echo [INFO] https://ollama.com/
echo     echo.
echo     pause
echo ^)
echo.
echo REM LocalRAG-Pro を起動
echo start "" "LocalRAG-Pro.exe"
echo.
echo echo LocalRAG-Pro を起動しました。
echo timeout /t 2 /nobreak ^>nul
) > "%PACKAGE_DIR%\起動.bat"

REM システム要件ファイルを作成
echo [INFO] システム要件ファイルを作成中...
(
echo # システム要件
echo.
echo ## 必須
echo.
echo - **OS**: Windows 10 または Windows 11 ^(64bit^)
echo - **RAM**: 8GB 以上（16GB 推奨）
echo - **ストレージ**: 5GB 以上の空き容量
echo - **Ollama**: https://ollama.com/ からインストール
echo.
echo ## 推奨
echo.
echo - **CPU**: マルチコアプロセッサ
echo - **RAM**: 16GB 以上
echo - **インターネット**: 初回起動時のモデルダウンロード用
echo.
echo ## 対応ファイル形式
echo.
echo - PDF ^(.pdf^)
echo - Microsoft Word ^(.docx^)
echo - Microsoft Excel ^(.xlsx^)
echo - Microsoft PowerPoint ^(.pptx^)
echo - テキストファイル ^(.txt^)
echo.
) > "%PACKAGE_DIR%\SYSTEM_REQUIREMENTS.md"

REM ZIPファイルを作成（PowerShellを使用）
echo [INFO] ZIPファイルを作成中...
powershell -Command "Compress-Archive -Path '%PACKAGE_DIR%' -DestinationPath '%ZIP_FILE%' -Force"

if errorlevel 1 (
    echo [ERROR] ZIPファイルの作成に失敗しました。
    pause
    exit /b 1
)

REM ファイルサイズを取得
for %%A in ("%ZIP_FILE%") do set SIZE=%%~zA
set /a SIZE_MB=%SIZE% / 1048576

echo.
echo ========================================
echo   配布パッケージ作成完了！
echo ========================================
echo.
echo 📦 パッケージ: %ZIP_FILE%
echo 📊 サイズ: 約 %SIZE_MB% MB
echo.
echo [内容物]
echo - LocalRAG-Pro.exe （メイン実行ファイル）
echo - _internal/ （依存ライブラリ）
echo - README.md （詳細ドキュメント）
echo - QUICKSTART.md （クイックスタートガイド）
echo - SYSTEM_REQUIREMENTS.md （システム要件）
echo - 起動.bat （簡単起動スクリプト）
echo.
echo [配布方法]
echo 1. %ZIP_FILE% を配布
echo 2. 受け取った人は ZIP を展開して「起動.bat」を実行
echo.
echo [注意事項]
echo - 受け取る側も Ollama のインストールが必要です
echo - 初回起動時にインターネット接続が必要です（モデルダウンロード）
echo.
pause
