@echo off
cd /d "%~dp0"
set PATH=%USERPROFILE%\.cargo\bin;%PATH%
echo Starte Tauri Dev...
npx tauri dev
pause
