@echo off
REM ChromaDB起動スクリプト
REM UTF-8エンコーディング設定
chcp 65001 > nul

echo ==========================================
echo   ChromaDB Server Launcher
echo ==========================================
echo.

REM カレントディレクトリをスクリプトの場所に変更
cd /d "%~dp0"

echo [1/2] Activating Python virtual environment...
if not exist ".venv\Scripts\activate.bat" (
    echo ERROR: Virtual environment not found at .venv
    echo Please create a virtual environment first:
    echo   python -m venv .venv
    echo   .venv\Scripts\activate.bat
    echo   pip install chromadb
    pause
    exit /b 1
)

call .venv\Scripts\activate.bat
echo OK - Virtual environment activated

echo.
echo Checking ChromaDB version...
python -c "import chromadb; print('Current version:', chromadb.__version__)"

echo.
echo [2/3] Checking dependencies...
python -c "import fastapi, uvicorn" 2>nul
if errorlevel 1 (
    echo Installing FastAPI and Uvicorn...
    python -m pip install fastapi uvicorn --quiet
)

echo.
echo [3/3] Starting ChromaDB Bridge Server...
echo Server will run on http://localhost:8001
echo Press Ctrl+C to stop the server
echo.

python chromadb_server.py

echo.
echo ChromaDB server stopped.
pause
