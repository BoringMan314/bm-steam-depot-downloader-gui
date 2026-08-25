@echo off
setlocal EnableExtensions EnableDelayedExpansion

chcp 65001 >nul
cd /d "%~dp0"
set "ROOT=%CD%"

set "DIST=%ROOT%\dist"
set "TARGET=%ROOT%\src-tauri\target\release"
set "BUNDLE=%TARGET%\bundle"
rem Cargo names the binary after the crate (vectum); dist gets the release naming.
set "BIN_NAME=vectum.exe"
set "PRODUCT_NAME=SteamDepotDownloaderGUI"
set "PORTABLE_NAME="

set "BUNDLES=nsis"
if /i "%~1"=="msi" set "BUNDLES=msi"
if /i "%~1"=="all" set "BUNDLES=nsis,msi"
if /i "%~2"=="msi" set "BUNDLES=msi"
if /i "%~2"=="all" set "BUNDLES=nsis,msi"

echo.
echo ========================================
echo   bm-steam-depot-downloader-gui
echo   Windows 10 Build Script
echo ========================================
echo.

call :CheckRoot || goto :end_fail
call :DetectToolchain || goto :end_fail
call :KillRunningApp || goto :end_fail
call :InstallDeps || goto :end_fail
call :ClearDist || goto :end_fail
call :BuildTauri || goto :end_fail
call :CollectArtifacts || goto :end_fail

echo.
echo ========================================
echo   Build completed successfully
echo ========================================
echo   Output : %DIST%
echo.
dir /b "%DIST%" 2>nul
echo.
goto :end_ok

:CheckRoot
if not exist "%ROOT%\package.json" (
    echo [ERROR] package.json not found. Run this script from the repo root.
    exit /b 1
)
if not exist "%ROOT%\src-tauri\tauri.conf.json" (
    echo [ERROR] src-tauri\tauri.conf.json not found.
    exit /b 1
)
exit /b 0

:DetectToolchain
if exist "%USERPROFILE%\.cargo\bin\cargo.exe" set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"

where node >nul 2>&1
if errorlevel 1 (
    echo [ERROR] node not in PATH. Install Node.js 20+.
    exit /b 1
)

where pnpm >nul 2>&1
if errorlevel 1 (
    echo [ERROR] pnpm not in PATH. Install with: npm i -g pnpm
    exit /b 1
)

where cargo >nul 2>&1
if errorlevel 1 (
    echo [ERROR] cargo not in PATH. Install Rust from https://rustup.rs
    exit /b 1
)

for /f "delims=" %%V in ('node -v 2^>nul') do set "NODE_VER=%%V"
for /f "delims=" %%V in ('pnpm -v 2^>nul') do set "PNPM_VER=%%V"
for /f "tokens=2" %%V in ('cargo --version 2^>nul') do set "CARGO_VER=%%V"
echo [INFO] node=!NODE_VER!  pnpm=!PNPM_VER!  cargo=!CARGO_VER!
echo [INFO] bundles=%BUNDLES%
exit /b 0

:KillRunningApp
tasklist /fi "imagename eq %BIN_NAME%" 2>nul | find /i "%BIN_NAME%" >nul
if not errorlevel 1 (
    echo [INFO] Closing running %BIN_NAME%
    taskkill /f /im "%BIN_NAME%" /t >nul 2>&1
)
tasklist /fi "imagename eq %PRODUCT_NAME%*" 2>nul | find /i "%PRODUCT_NAME%" >nul
if not errorlevel 1 (
    echo [INFO] Closing running %PRODUCT_NAME%*.exe
    taskkill /f /fi "imagename eq %PRODUCT_NAME%*" /t >nul 2>&1
)
exit /b 0

:InstallDeps
if exist "%ROOT%\node_modules\.pnpm" (
    echo [INFO] Dependencies already installed, skipping pnpm install.
    exit /b 0
)
echo [INFO] Installing frontend dependencies...
call pnpm install --frozen-lockfile
if errorlevel 1 (
    echo [ERROR] pnpm install failed.
    exit /b 1
)
exit /b 0

:ClearDist
if not exist "%DIST%" mkdir "%DIST%"
del /q /f "%DIST%\*" 2>nul
for /d %%D in ("%DIST%\*") do rd /s /q "%%D" 2>nul
echo [INFO] Cleared %DIST%
rem Tauri keeps older installers here, so collecting would pick up stale versions.
if exist "%BUNDLE%\nsis" rd /s /q "%BUNDLE%\nsis" 2>nul
if exist "%BUNDLE%\msi" rd /s /q "%BUNDLE%\msi" 2>nul
exit /b 0

:BuildTauri
echo.
echo [BUILD] pnpm tauri build --bundles %BUNDLES%
call pnpm tauri build --bundles %BUNDLES%
if errorlevel 1 (
    echo [ERROR] Tauri build failed.
    exit /b 1
)
exit /b 0

:CollectArtifacts
set "FOUND=0"
set "SETUP_STEM="

for %%F in ("%BUNDLE%\nsis\*-setup.exe") do (
    copy /y "%%F" "%DIST%\%%~nxF" >nul
    if errorlevel 1 (
        echo [ERROR] Failed to copy %%~nxF to dist.
        exit /b 1
    )
    echo [OK] %%~nxF - NSIS installer
    set "FOUND=1"
    set "SETUP_STEM=%%~nF"
)

for %%F in ("%BUNDLE%\msi\*.msi") do (
    copy /y "%%F" "%DIST%\%%~nxF" >nul
    if errorlevel 1 (
        echo [ERROR] Failed to copy %%~nxF to dist.
        exit /b 1
    )
    echo [OK] %%~nxF - MSI installer
    set "FOUND=1"
)

if "!FOUND!"=="0" (
    echo [ERROR] No installer found under %BUNDLE%
    exit /b 1
)

rem Keep the portable exe on the same naming scheme as the installer.
if defined SETUP_STEM (
    set "PORTABLE_NAME=!SETUP_STEM:-setup=-portable!.exe"
) else (
    call :ResolveVersion || exit /b 1
    set "PORTABLE_NAME=%PRODUCT_NAME%_!APP_VERSION!_x64-portable.exe"
)

if not exist "%TARGET%\%BIN_NAME%" (
    echo [ERROR] Missing %TARGET%\%BIN_NAME%
    exit /b 1
)
copy /y "%TARGET%\%BIN_NAME%" "%DIST%\!PORTABLE_NAME!" >nul
if errorlevel 1 (
    echo [ERROR] Failed to copy portable exe to dist.
    exit /b 1
)
echo [OK] !PORTABLE_NAME! - portable
exit /b 0

:ResolveVersion
set "APP_VERSION="
for /f "tokens=2 delims=:" %%V in ('findstr /r /c:"\"version\"" "%ROOT%\src-tauri\tauri.conf.json"') do (
    set "APP_VERSION=%%V"
)
set "APP_VERSION=!APP_VERSION: =!"
set "APP_VERSION=!APP_VERSION:,=!"
set APP_VERSION=!APP_VERSION:"=!
if not defined APP_VERSION (
    echo [ERROR] Could not read version from src-tauri\tauri.conf.json
    exit /b 1
)
exit /b 0

:end_fail
if /i "%~1"=="nopause" exit /b 1
if /i "%~2"=="nopause" exit /b 1
echo.
pause
exit /b 1

:end_ok
if /i "%~1"=="nopause" exit /b 0
if /i "%~2"=="nopause" exit /b 0
echo.
pause
exit /b 0
