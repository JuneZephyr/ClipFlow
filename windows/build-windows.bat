@echo off
rem Builds a Windows version of ClipFlow.
rem Put this folder on a Windows 10/11 PC, then double-click this file.
setlocal
cd /d "%~dp0.."

where cargo >nul 2>nul
if errorlevel 1 (
    echo.
    echo Rust is not installed yet.
    echo Install it from https://rustup.rs ^(choose the default options^),
    echo then double-click this file again.
    echo.
    pause
    exit /b 1
)

echo Building ClipFlow. The first build can take a few minutes...
cargo build --release
if errorlevel 1 (
    echo.
    echo The build did not finish. The messages above show why.
    echo.
    pause
    exit /b 1
)

if not exist "windows\out" mkdir "windows\out"
copy /y "target\release\clipflow.exe" "windows\out\ClipFlow.exe" >nul
copy /y "windows\README.txt" "windows\out\README.txt" >nul

powershell -NoProfile -Command "if (Test-Path 'windows\ClipFlow-windows.zip') { Remove-Item 'windows\ClipFlow-windows.zip' -Force }; Compress-Archive -Path 'windows\out\*' -DestinationPath 'windows\ClipFlow-windows.zip'"

echo.
echo Done. Your app is here:
echo   %~dp0ClipFlow-windows.zip
echo Open that zip, take the folder out, and double-click ClipFlow.exe.
echo.
pause
