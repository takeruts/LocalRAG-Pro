@echo off
REM LocalRAG Pro Rust版 GUI起動スクリプト
REM UTF-8エンコーディング設定
chcp 65001 > nul

echo ==========================================
echo   LocalRAG Pro - Rust Edition GUI
echo ==========================================
echo.

REM カレントディレクトリをスクリプトの場所に変更
cd /d "%~dp0"

REM Rustプロジェクトディレクトリに移動
cd localrag-rust

echo [1/3] Checking Rust installation...
where cargo > nul 2>&1
if errorlevel 1 (
    echo ERROR: Cargo not found. Please install Rust from https://rustup.rs/
    pause
    exit /b 1
)
echo OK - Cargo found

echo.
echo [2/3] Building GUI application...
echo This may take a few minutes on first run...
cargo build -p rag-gui
if errorlevel 1 (
    echo ERROR: Build failed
    pause
    exit /b 1
)

echo.
echo [3/3] Starting LocalRAG Pro GUI...
echo.
cargo run -p rag-gui

echo.
echo Application closed.
pause
