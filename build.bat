@echo off
cd /d "%~dp0"
echo === CrealityIM Tauri Build (portable EXE) ===
set PATH=%USERPROFILE%\.cargo\bin;%PATH%

echo [1/2] npm install...
cmd /c "npm install"

echo [2/2] Icon PNG aus ICO generieren...
python -c "from PIL import Image; img=Image.open('src-tauri/icons/icon.ico'); img.save('src-tauri/icons/icon.png')" 2>nul

echo [3/3] Tauri Release Build...
cmd /c "npx tauri build"

echo.
echo Kopiere Installer in Projektordner...
for %%f in ("src-tauri\target\release\bundle\nsis\*.exe") do copy /Y "%%f" "."
for %%f in ("src-tauri\target\release\bundle\msi\*.msi") do copy /Y "%%f" "."

echo.
dir /b "*.exe" "*.msi" 2>nul
echo Fertig!
pause
