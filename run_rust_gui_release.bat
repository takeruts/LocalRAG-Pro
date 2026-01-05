@echo off
REM LocalRAG Pro Rust版 GUI起動スクリプト (Release版 - 最適化ビルド)
REM UTF-8エンコーディング設定
chcp 65001 > nul

echo ==========================================
echo   LocalRAG Pro - Rust Edition GUI
echo   Release Mode (Optimized)
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
echo [2/3] Building optimized release version...
echo WARNING: First build may take 10-15 minutes (LTO optimization)
echo.
cargo build --release -p rag-gui
if errorlevel 1 (
    echo ERROR: Build failed
    pause
    exit /b 1
)

echo.
echo [3/3] Starting LocalRAG Pro GUI (Release)...
echo.
.\target\release\rag-gui.exe

echo.
echo Application closed.
pause
