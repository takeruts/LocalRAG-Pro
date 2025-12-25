@echo off
setlocal
chcp 65001 > nul

set APP_NAME=LocalRAG_Pro
set DIST_DIR=dist\%APP_NAME%
set ZIP_NAME=%APP_NAME%.zip

echo ============================================
echo   %APP_NAME% 配布パッケージ作成ツール
echo ============================================

:: 1. READMEのコピー
echo [1/3] README.md をコピー中...
if exist "README.md" (
    copy /Y "README.md" "%DIST_DIR%\README.md"
) else (
    echo [警告] README.md が見つかりません。
)

:: 2. 不要な一時フォルダの削除 (もしあれば)
echo [2/3] 一時ファイルをクリーンアップ中...
if exist "build" rmdir /s /q "build"

:: 3. ZIP圧縮の実行 (PowerShellを利用)
echo [3/3] ZIPファイルを作成中...
if exist "%ZIP_NAME%" del "%ZIP_NAME%"

powershell -Command "Compress-Archive -Path '%DIST_DIR%\*' -DestinationPath '%ZIP_NAME%'"

echo ============================================
echo   完了! %ZIP_NAME% が作成されました。
echo ============================================
pause