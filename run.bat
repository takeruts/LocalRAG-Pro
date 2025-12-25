@echo off
setlocal
cd /d %~dp0
chcp 65001 > nul

:: --- 仮想環境フォルダ名の設定 ---
set VENV_NAME=.venv

:: 存在チェック：.venv フォルダの中に python.exe があるか確認
if not exist "%VENV_NAME%\Scripts\python.exe" (
    echo =======================================================
    echo 【エラー】仮想環境 "%VENV_NAME%" が見つかりません。
    echo 以下の場所に .venv フォルダを作成してください：
    echo %~dp0%VENV_NAME%
    echo =======================================================
    pause
    exit /b
)

:: --- Intel PC (Core i-series) 最適化設定 ---
set OLLAMA_INTEL_GPU=1
set ONEDNN_MAX_CPU_ISA=AVX512_CORE_VNNI

:: 仮想環境のパスを優先的に追加
set PATH=%~dp0%VENV_NAME%\Scripts;%PATH%

echo =======================================================
echo   Local RAG App (Intel PC Optimized) を起動中...
echo   ※ブラウザを自動的に立ち上げます。
echo =======================================================

:: ブラウザでアプリのURLを開く
start http://localhost:8501

:: Streamlitを起動
:: -m streamlit run を使用することで正しく起動します
"%~dp0%VENV_NAME%\Scripts\python.exe" -m streamlit run ragapp.py --server.port 8501 --server.headless true

pause