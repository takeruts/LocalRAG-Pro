# LocalRAG Pro Rust版 GUI起動スクリプト (PowerShell)
# UTF-8エンコーディング設定
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$OutputEncoding = [System.Text.Encoding]::UTF8

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  LocalRAG Pro - Rust Edition GUI" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""

# スクリプトのディレクトリに移動
$scriptPath = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $scriptPath

Write-Host "[1/3] Checking Rust installation..." -ForegroundColor Yellow
$cargoPath = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $cargoPath) {
    Write-Host "ERROR: Cargo not found. Please install Rust from https://rustup.rs/" -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit 1
}
Write-Host "OK - Cargo found at: $($cargoPath.Source)" -ForegroundColor Green

Write-Host ""
Write-Host "[2/3] Building GUI application..." -ForegroundColor Yellow
Write-Host "This may take a few minutes on first run..." -ForegroundColor Gray
cargo build -p rag-gui
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Build failed" -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit 1
}

Write-Host ""
Write-Host "[3/3] Starting LocalRAG Pro GUI..." -ForegroundColor Yellow
Write-Host ""
cargo run -p rag-gui

Write-Host ""
Write-Host "Application closed." -ForegroundColor Gray
Read-Host "Press Enter to exit"
