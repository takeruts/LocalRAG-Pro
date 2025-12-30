@echo off
chcp 65001 >nul
echo ========================================
echo   Git 大容量ファイル問題の修正
echo ========================================
echo.

echo [警告] この操作は以下を実行します:
echo - dist/, build/, deploy/, package/ フォルダをGitの追跡から削除
echo - .gitignore に従って除外設定を適用
echo - ローカルのファイルは削除されません
echo.
echo 続行しますか？ (Ctrl+C で中断)
pause

echo.
echo [INFO] Gitキャッシュから大容量ファイルを削除中...

REM Gitの追跡から削除（ローカルファイルは保持）
git rm -r --cached dist/ 2>nul
git rm -r --cached build/ 2>nul
git rm -r --cached deploy/ 2>nul
git rm -r --cached package/ 2>nul
git rm -r --cached models/ 2>nul
git rm -r --cached chroma_db/ 2>nul
git rm --cached *.zip 2>nul

echo.
echo [INFO] .gitignore を再適用中...
git add .gitignore
git add .

echo.
echo [INFO] 変更をコミット中...
git commit -m "fix: Remove large binary files from Git tracking

- Remove dist/, build/, deploy/, package/ folders
- Remove models/ and chroma_db/ folders
- Apply .gitignore rules properly
- These files should not be tracked in Git"

echo.
echo ========================================
echo   修正完了
echo ========================================
echo.
echo [次のステップ]
echo 1. 以下のコマンドでプッシュしてください:
echo    git push origin main
echo.
echo 2. もしプッシュに失敗する場合は、強制プッシュが必要です:
echo    git push origin main --force
echo.
echo [注意]
echo - 実行ファイルはGitHubにプッシュしません
echo - 配布は GitHub Releases を使用してください
echo - または package.bat で作成したZIPを別途共有してください
echo.
pause
