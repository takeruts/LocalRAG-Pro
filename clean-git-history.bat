@echo off
chcp 65001 >nul
echo ========================================
echo   Git履歴完全クリーンアップ
echo ========================================
echo.
echo [警告] この操作は:
echo - Git履歴全体を書き換えます
echo - deploy/, dist/, build/, package/ を完全に削除
echo - 元に戻すことはできません
echo.
echo 続行しますか？ ^(10秒以内にCtrl+Cで中断^)
timeout /t 10

echo.
echo [INFO] Git履歴をクリーンアップ中...
echo [INFO] この処理には数分かかる場合があります。
echo.

REM filter-branchで履歴から完全に削除
git filter-branch --force --index-filter "git rm -r --cached --ignore-unmatch deploy dist build package models chroma_db *.zip" --prune-empty --tag-name-filter cat -- --all

if errorlevel 1 (
    echo.
    echo [ERROR] Git filter-branchに失敗しました。
    pause
    exit /b 1
)

echo.
echo [INFO] 古い参照を削除中...
git for-each-ref --format="delete %(refname)" refs/original | git update-ref --stdin

echo.
echo [INFO] Reflogをクリーンアップ中...
git reflog expire --expire=now --all

echo.
echo [INFO] ガベージコレクション実行中...
git gc --prune=now --aggressive

echo.
echo ========================================
echo   クリーンアップ完了
echo ========================================
echo.
echo [次のステップ]
echo 以下のコマンドで強制プッシュしてください:
echo.
echo    git push origin main --force
echo.
echo [注意]
echo - 他の人がこのリポジトリをクローンしている場合は通知してください
echo - 強制プッシュ後、他の人は再クローンが必要です
echo.
pause
