@echo off
setlocal EnableDelayedExpansion

REM Read private key from file
set /p TAURI_SIGNING_PRIVATE_KEY=<"%USERPROFILE%\.tauri\localrag.key"
set TAURI_SIGNING_PRIVATE_KEY_PASSWORD=

cd /d "%~dp0"
npm run tauri build
