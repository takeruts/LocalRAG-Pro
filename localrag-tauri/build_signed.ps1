# Build LocalRAG Pro with signing
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content -Raw "$env:USERPROFILE\.tauri\localrag.key"
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""

Set-Location $PSScriptRoot
npm run tauri build
