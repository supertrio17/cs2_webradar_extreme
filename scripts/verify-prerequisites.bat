@echo off
setlocal EnableExtensions EnableDelayedExpansion

set "MISSING_COUNT=0"

echo ===========================================
echo cs2_webradar_extreme prerequisite verification
echo ===========================================
echo.

call :check_visual_studio_build_tools
call :check_webview2_runtime
call :check_git
call :check_node
call :check_npm
call :check_rustup
call :check_rustc
call :check_cargo
call :check_rust_target

echo.
if not "!MISSING_COUNT!"=="0" (
    echo [FAIL] !MISSING_COUNT! prerequisite check^(s^) failed.
    exit /b 1
)

echo [SUCCESS] All required prerequisites are installed.
exit /b 0

:mark_missing
set /a MISSING_COUNT+=1
echo [MISSING] %~1
exit /b 0

:check_visual_studio_build_tools
echo [CHECK] Visual Studio Build Tools (C++ workload)
set "VS_PATH="
set "VS_VERSION="
set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
if not exist "%VSWHERE%" (
    call :mark_missing "Visual Studio Build Tools (vswhere not found)"
    exit /b 0
)

for /f "usebackq delims=" %%I in (`"%VSWHERE%" -latest -products * -requires Microsoft.VisualStudio.Workload.VCTools -property installationPath`) do set "VS_PATH=%%I"
for /f "usebackq delims=" %%I in (`"%VSWHERE%" -latest -products * -requires Microsoft.VisualStudio.Workload.VCTools -property catalog_productDisplayVersion`) do set "VS_VERSION=%%I"

if not defined VS_PATH (
    call :mark_missing "Visual Studio Build Tools + C++ workload"
    exit /b 0
)

if defined VS_VERSION (
    echo [OK] VS Build Tools !VS_VERSION! at !VS_PATH!
) else (
    echo [OK] VS Build Tools detected at !VS_PATH!
)
exit /b 0

:detect_webview2
set "WEBVIEW2_VERSION="
for %%K in (
    "HKLM\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"
    "HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"
    "HKCU\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"
) do (
    for /f "tokens=2,*" %%A in ('reg query "%%~K" /v pv 2^>nul ^| findstr /R /C:"pv"') do set "WEBVIEW2_VERSION=%%B"
    if defined WEBVIEW2_VERSION exit /b 0
)
exit /b 0

:check_webview2_runtime
echo [CHECK] Microsoft Edge WebView2 Runtime
call :detect_webview2
if defined WEBVIEW2_VERSION (
    echo [OK] WebView2 Runtime !WEBVIEW2_VERSION!
) else (
    call :mark_missing "Microsoft Edge WebView2 Runtime"
)
exit /b 0

:check_git
echo [CHECK] Git
where git >nul 2>&1
if errorlevel 1 (
    call :mark_missing "Git"
    exit /b 0
)
set "GIT_VERSION="
for /f "delims=" %%I in ('git --version 2^>nul') do if not defined GIT_VERSION set "GIT_VERSION=%%I"
if defined GIT_VERSION (
    echo [OK] !GIT_VERSION!
) else (
    echo [OK] Git detected.
)
exit /b 0

:check_node
echo [CHECK] Node.js
where node >nul 2>&1
if errorlevel 1 (
    call :mark_missing "Node.js"
    exit /b 0
)
set "NODE_VERSION="
for /f "delims=" %%I in ('node --version 2^>nul') do if not defined NODE_VERSION set "NODE_VERSION=%%I"
echo [OK] Node !NODE_VERSION!
exit /b 0

:check_npm
echo [CHECK] npm
where npm >nul 2>&1
if errorlevel 1 (
    call :mark_missing "npm"
    exit /b 0
)
set "NPM_VERSION="
for /f "delims=" %%I in ('npm --version 2^>nul') do if not defined NPM_VERSION set "NPM_VERSION=%%I"
echo [OK] npm !NPM_VERSION!
exit /b 0

:check_rustup
echo [CHECK] rustup
if exist "%USERPROFILE%\.cargo\bin\rustup.exe" set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
where rustup >nul 2>&1
if errorlevel 1 (
    call :mark_missing "rustup"
    exit /b 0
)
set "RUSTUP_VERSION="
for /f "delims=" %%I in ('rustup --version 2^>nul') do if not defined RUSTUP_VERSION set "RUSTUP_VERSION=%%I"
echo [OK] !RUSTUP_VERSION!
exit /b 0

:check_rustc
echo [CHECK] rustc
if exist "%USERPROFILE%\.cargo\bin\rustc.exe" set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
where rustc >nul 2>&1
if errorlevel 1 (
    call :mark_missing "rustc"
    exit /b 0
)
set "RUSTC_VERSION="
for /f "delims=" %%I in ('rustc --version 2^>nul') do if not defined RUSTC_VERSION set "RUSTC_VERSION=%%I"
echo [OK] !RUSTC_VERSION!
exit /b 0

:check_cargo
echo [CHECK] cargo
if exist "%USERPROFILE%\.cargo\bin\cargo.exe" set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
where cargo >nul 2>&1
if errorlevel 1 (
    call :mark_missing "cargo"
    exit /b 0
)
set "CARGO_VERSION="
for /f "delims=" %%I in ('cargo --version 2^>nul') do if not defined CARGO_VERSION set "CARGO_VERSION=%%I"
echo [OK] !CARGO_VERSION!
exit /b 0

:check_rust_target
echo [CHECK] Rust target x86_64-pc-windows-msvc
if exist "%USERPROFILE%\.cargo\bin\rustup.exe" set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
where rustup >nul 2>&1
if errorlevel 1 (
    call :mark_missing "Rust target x86_64-pc-windows-msvc (rustup unavailable)"
    exit /b 0
)

set "HAS_TARGET=0"
for /f "delims=" %%I in ('rustup target list --installed 2^>nul') do (
    if /I "%%I"=="x86_64-pc-windows-msvc" set "HAS_TARGET=1"
)

if "!HAS_TARGET!"=="1" (
    echo [OK] Rust target x86_64-pc-windows-msvc installed.
) else (
    call :mark_missing "Rust target x86_64-pc-windows-msvc"
)
exit /b 0
