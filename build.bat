@echo off
chcp 65001 >nul
echo ========================================
echo   LocalRAG-Pro ビルドスクリプト
echo ========================================
echo.

REM 仮想環境の確認とアクティベート
if exist ".venv\Scripts\activate.bat" (
    echo [INFO] 仮想環境を検出しました ^(.venv^)
    call .venv\Scripts\activate.bat
) else (
    if exist "venv\Scripts\activate.bat" (
        echo [INFO] 仮想環境を検出しました ^(venv^)
        call venv\Scripts\activate.bat
    ) else (
        echo [WARNING] 仮想環境が見つかりません。グローバル環境で実行します。
    )
)

REM 必要なパッケージのインストール確認
echo.
echo [INFO] 必要なパッケージを確認中...

python -c "import PyInstaller" 2>nul
if errorlevel 1 (
    echo [INFO] PyInstallerがインストールされていません。インストール中...
    pip install pyinstaller
    if errorlevel 1 (
        echo [ERROR] PyInstallerのインストールに失敗しました。
        pause
        exit /b 1
    )
)

REM 既存のビルド成果物をクリーンアップ
echo.
echo [INFO] 既存のビルド成果物をクリーンアップ中...

REM 実行中のプロセスを確認
tasklist /FI "IMAGENAME eq LocalRAG-Pro.exe" 2>NUL | find /I /N "LocalRAG-Pro.exe">NUL
if "%ERRORLEVEL%"=="0" (
    echo [WARNING] LocalRAG-Pro.exe が実行中です。終了してください。
    echo [WARNING] 5秒後に強制終了を試みます...
    timeout /t 5 /nobreak
    taskkill /F /IM LocalRAG-Pro.exe 2>NUL
    timeout /t 2 /nobreak
)

if exist "build" (
    echo [INFO] build フォルダを削除中...
    rd /s /q "build" 2>NUL
)

if exist "dist" (
    echo [INFO] dist フォルダを削除中...
    rd /s /q "dist" 2>NUL
    if exist "dist" (
        echo [WARNING] dist フォルダの削除に失敗しました。手動で削除してください。
        echo [WARNING] エクスプローラーやアプリケーションでファイルが開かれていないか確認してください。
        pause
        exit /b 1
    )
)

REM .specファイルを使用してビルド
echo.
echo [INFO] PyInstallerでビルドを開始します（Ollamaベース版）...
echo [INFO] 予想時間: 5-10分
echo [INFO] 最終サイズ: 約300-500MB
echo.

if exist "win_rag.spec" (
    echo [INFO] win_rag.spec を使用してビルド中...
    pyinstaller win_rag.spec --clean --noconfirm
) else (
    echo [WARNING] win_rag.spec が見つかりません。コマンドラインオプションでビルドします...
    pyinstaller ^
        --name "LocalRAG-Pro" ^
        --noconsole ^
        --onedir ^
        --optimize=2 ^
        --strip ^
        --upx-dir=. ^
        --hidden-import=langchain_community.llms.ollama ^
        --hidden-import=langchain_community.document_loaders ^
        --hidden-import=langchain_community.vectorstores.chroma ^
        --hidden-import=langchain_huggingface.embeddings ^
        --hidden-import=sentence_transformers.cross_encoder ^
        --hidden-import=sentence_transformers.SentenceTransformer ^
        --collect-all customtkinter ^
        --collect-all langchain_community ^
        --collect-all langchain_huggingface ^
        --collect-all langchain_core ^
        --collect-all langchain_text_splitters ^
        --collect-all chromadb ^
        --collect-all sentence_transformers ^
        --exclude-module torch ^
        --exclude-module transformers ^
        --exclude-module tokenizers ^
        --exclude-module accelerate ^
        --exclude-module bitsandbytes ^
        --exclude-module matplotlib ^
        --exclude-module pandas ^
        --exclude-module pytest ^
        --exclude-module notebook ^
        --exclude-module jupyter ^
        --exclude-module IPython ^
        win_rag.py
)

if errorlevel 1 (
    echo.
    echo [ERROR] ビルドに失敗しました。
    echo [ERROR] エラーメッセージを確認してください。
    pause
    exit /b 1
)

REM ビルド成功
echo.
echo ========================================
echo   ビルドが完了しました！
echo ========================================
echo.
echo 実行ファイルの場所: dist\LocalRAG-Pro\LocalRAG-Pro.exe
echo.
echo [注意事項]
echo - 初回起動時にAIモデルのダウンロードが発生します
echo - models フォルダと chroma_db フォルダは実行ファイルと同じ場所に作成されます
echo - Ollamaが起動していることを確認してください
echo.
echo [次のステップ]
echo 1. dist\LocalRAG-Pro フォルダ全体を配布してください
echo 2. LocalRAG-Pro.exe をダブルクリックで起動できます
echo.
pause
