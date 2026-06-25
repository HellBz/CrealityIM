@echo off
chcp 65001 >nul
cd /d "%~dp0\.."
echo Starting CrealityIM CORS proxy...
node tools\cors_proxy.js
pause
