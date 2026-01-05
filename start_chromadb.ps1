# ChromaDB起動スクリプト (PowerShell)
# UTF-8エンコーディング設定
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$OutputEncoding = [System.Text.Encoding]::UTF8

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  ChromaDB Server Launcher" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""

# スクリプトのディレクトリに移動
$scriptPath = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $scriptPath

Write-Host "[1/2] Activating Python virtual environment..." -ForegroundColor Yellow
if (-not (Test-Path ".venv\Scripts\Activate.ps1")) {
    Write-Host "ERROR: Virtual environment not found at .venv" -ForegroundColor Red
    Write-Host "Please create a virtual environment first:" -ForegroundColor Red
    Write-Host "  python -m venv .venv" -ForegroundColor White
    Write-Host "  .\.venv\Scripts\Activate.ps1" -ForegroundColor White
    Write-Host "  pip install chromadb" -ForegroundColor White
    Read-Host "Press Enter to exit"
    exit 1
}

& .\.venv\Scripts\Activate.ps1
Write-Host "OK - Virtual environment activated" -ForegroundColor Green

Write-Host ""
Write-Host "Checking ChromaDB version..." -ForegroundColor Yellow
python -c "import chromadb; print('Current version:', chromadb.__version__)"

Write-Host ""
Write-Host "[2/3] Checking dependencies..." -ForegroundColor Yellow
$null = python -c "import fastapi, uvicorn" 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "Installing FastAPI and Uvicorn..." -ForegroundColor Yellow
    python -m pip install fastapi uvicorn --quiet
}

Write-Host ""
Write-Host "[3/3] Starting ChromaDB Bridge Server..." -ForegroundColor Yellow
Write-Host "Server will run on http://localhost:8001" -ForegroundColor Cyan
Write-Host "Press Ctrl+C to stop the server" -ForegroundColor Gray
Write-Host ""

python chromadb_server.py

Write-Host ""
Write-Host "ChromaDB server stopped." -ForegroundColor Gray
Read-Host "Press Enter to exit"
