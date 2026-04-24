@echo off
setlocal EnableExtensions EnableDelayedExpansion

set "SCRIPT_DIR=%~dp0"
set "VERIFY_SCRIPT=%SCRIPT_DIR%verify-prerequisites.bat"
set "FAILURES=0"
set "FAILED_STEPS="

echo ===========================================================
echo cs2_webradar_extreme Windows prerequisites installer
echo ===========================================================
echo.

call :ensure_admin
if errorlevel 2 exit /b 0
if errorlevel 1 exit /b 1

call :ensure_winget
if errorlevel 1 (
    call :print_winget_fallback
    exit /b 1
)

call :install_visual_studio_build_tools
call :install_webview2_runtime
call :install_git
call :install_node
call :install_rust_and_targets

echo.
if not "!FAILURES!"=="0" (
    echo [ERROR] Completed with !FAILURES! failed step^(s^).
    echo [ERROR] Failed steps: !FAILED_STEPS!
    echo [ERROR] Review the errors above, fix them, and run this script again.
    exit /b 1
)

if exist "%VERIFY_SCRIPT%" (
    echo [INFO] Running verification script...
    call "%VERIFY_SCRIPT%"
    if errorlevel 1 (
        echo [ERROR] Verification reported missing prerequisites.
        exit /b 1
    )
)

echo.
echo [SUCCESS] All prerequisites are installed and verified.
echo.
echo Next steps:
echo   1. Open cs2_webradar_extreme.sln in Visual Studio.
echo   2. Select Configuration: Release.
echo   3. Select Platform: x64.
echo   4. Build the solution.
exit /b 0

:ensure_admin
fsutil dirty query %SystemDrive% >nul 2>&1
if %errorlevel% EQU 0 (
    echo [OK] Running with administrator privileges.
    exit /b 0
)

echo [WARN] Administrator privileges are required for dependency installation.
where powershell >nul 2>&1
if errorlevel 1 (
    echo [ERROR] PowerShell is unavailable, so auto-elevation cannot be requested.
    echo [INFO] Re-run this script from an elevated Command Prompt.
    exit /b 1
)

echo [INFO] Attempting to relaunch this script as Administrator...
powershell -NoProfile -ExecutionPolicy Bypass -Command "Start-Process -FilePath '%~f0' -WorkingDirectory '%CD%' -Verb RunAs"
if errorlevel 1 (
    echo [ERROR] Elevation was canceled or failed.
    echo [INFO] Re-run this script with Run as administrator.
    exit /b 1
)

echo [INFO] Elevated installer launched in a new window.
exit /b 2

:ensure_winget
where winget >nul 2>&1
if errorlevel 1 (
    echo [ERROR] winget was not found on this machine.
    exit /b 1
)

echo [OK] winget detected.
exit /b 0

:print_winget_fallback
echo [INFO] Install winget (App Installer) and run this script again.
echo [INFO] winget install docs: https://learn.microsoft.com/windows/package-manager/winget/
where choco >nul 2>&1
if not errorlevel 1 (
    echo [INFO] Chocolatey detected. Optional fallback command:
    echo        choco install -y git nodejs-lts rustup.install microsoft-edge-webview2-runtime visualstudio2022buildtools visualstudio2022-workload-vctools
    echo [INFO] Verify package names in your Chocolatey feed before using the fallback command.
)
exit /b 0

:run_winget_install
set "PACKAGE_ID=%~1"
set "DISPLAY_NAME=%~2"
set "OVERRIDE_ARGS=%~3"

echo [INSTALL] !DISPLAY_NAME!
if defined OVERRIDE_ARGS (
    winget install --id "!PACKAGE_ID!" --exact --source winget --accept-package-agreements --accept-source-agreements --silent --override "!OVERRIDE_ARGS!"
) else (
    winget install --id "!PACKAGE_ID!" --exact --source winget --accept-package-agreements --accept-source-agreements --silent
)
if errorlevel 1 (
    echo [ERROR] Install failed: !DISPLAY_NAME!
    exit /b 1
)

echo [OK] Install command completed: !DISPLAY_NAME!
exit /b 0

:mark_failure
set /a FAILURES+=1
if defined FAILED_STEPS (
    set "FAILED_STEPS=!FAILED_STEPS!, %~1"
) else (
    set "FAILED_STEPS=%~1"
)
exit /b 0

:find_vs_build_tools
set "VS_BUILD_TOOLS_PATH="
set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
if not exist "%VSWHERE%" exit /b 0
for /f "usebackq delims=" %%I in (`"%VSWHERE%" -latest -products * -requires Microsoft.VisualStudio.Workload.VCTools -property installationPath`) do set "VS_BUILD_TOOLS_PATH=%%I"
exit /b 0

:install_visual_studio_build_tools
echo.
echo [STEP] Visual Studio Build Tools + C++ workload
call :find_vs_build_tools
if defined VS_BUILD_TOOLS_PATH (
    echo [OK] Visual Studio Build Tools detected: !VS_BUILD_TOOLS_PATH!
    exit /b 0
)

set "VS_OVERRIDE=--wait --quiet --norestart --nocache --add Microsoft.VisualStudio.Workload.VCTools --add Microsoft.VisualStudio.Component.VC.Tools.x86.x64 --add Microsoft.VisualStudio.Component.Windows10SDK.19041 --includeRecommended"
call :run_winget_install "Microsoft.VisualStudio.2022.BuildTools" "Visual Studio 2022 Build Tools (C++ workload)" "!VS_OVERRIDE!"
if errorlevel 1 (
    call :mark_failure "Visual Studio Build Tools"
    exit /b 0
)

call :find_vs_build_tools
if defined VS_BUILD_TOOLS_PATH (
    echo [OK] Visual Studio Build Tools ready: !VS_BUILD_TOOLS_PATH!
) else (
    echo [ERROR] Visual Studio Build Tools were not detected after install.
    call :mark_failure "Visual Studio Build Tools"
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

:install_webview2_runtime
echo.
echo [STEP] Microsoft Edge WebView2 Runtime
call :detect_webview2
if defined WEBVIEW2_VERSION (
    echo [OK] WebView2 Runtime detected: !WEBVIEW2_VERSION!
    exit /b 0
)

call :run_winget_install "Microsoft.EdgeWebView2Runtime" "Microsoft Edge WebView2 Runtime"
if errorlevel 1 (
    call :mark_failure "WebView2 Runtime"
    exit /b 0
)

call :detect_webview2
if defined WEBVIEW2_VERSION (
    echo [OK] WebView2 Runtime ready: !WEBVIEW2_VERSION!
) else (
    echo [ERROR] WebView2 Runtime was not detected after install.
    call :mark_failure "WebView2 Runtime"
)
exit /b 0

:install_git
echo.
echo [STEP] Git
where git >nul 2>&1
if errorlevel 1 (
    call :run_winget_install "Git.Git" "Git"
    if errorlevel 1 (
        call :mark_failure "Git"
        exit /b 0
    )
)

if exist "%ProgramFiles%\Git\cmd\git.exe" set "PATH=%ProgramFiles%\Git\cmd;%PATH%"
where git >nul 2>&1
if errorlevel 1 (
    echo [ERROR] git.exe is still unavailable after install.
    call :mark_failure "Git"
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

:install_node
echo.
echo [STEP] Node.js LTS + npm
set "NODE_PRESENT=0"
set "NPM_PRESENT=0"
set "NEED_NODE_INSTALL=0"
where node >nul 2>&1 && set "NODE_PRESENT=1"
where npm >nul 2>&1 && set "NPM_PRESENT=1"
if "!NODE_PRESENT!"=="0" set "NEED_NODE_INSTALL=1"
if "!NPM_PRESENT!"=="0" set "NEED_NODE_INSTALL=1"

if "!NEED_NODE_INSTALL!"=="1" (
    call :run_winget_install "OpenJS.NodeJS.LTS" "Node.js LTS"
    if errorlevel 1 (
        call :mark_failure "Node.js LTS"
        exit /b 0
    )
)

if exist "%ProgramFiles%\nodejs\node.exe" set "PATH=%ProgramFiles%\nodejs;%PATH%"

where node >nul 2>&1
if errorlevel 1 (
    echo [ERROR] node.exe is still unavailable after install.
    call :mark_failure "Node.js LTS"
    exit /b 0
)
where npm >nul 2>&1
if errorlevel 1 (
    echo [ERROR] npm is still unavailable after install.
    call :mark_failure "npm"
    exit /b 0
)

set "NODE_VERSION="
set "NPM_VERSION="
for /f "delims=" %%I in ('node --version 2^>nul') do if not defined NODE_VERSION set "NODE_VERSION=%%I"
for /f "delims=" %%I in ('npm --version 2^>nul') do if not defined NPM_VERSION set "NPM_VERSION=%%I"
echo [OK] Node !NODE_VERSION!, npm !NPM_VERSION!
exit /b 0

:ensure_rustup_on_path
if exist "%USERPROFILE%\.cargo\bin\rustup.exe" set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
exit /b 0

:install_rust_and_targets
echo.
echo [STEP] Rust toolchain (stable) + Windows target
call :ensure_rustup_on_path
where rustup >nul 2>&1
if errorlevel 1 (
    call :run_winget_install "Rustlang.Rustup" "Rustup"
    if errorlevel 1 (
        call :mark_failure "Rustup"
        exit /b 0
    )
)

call :ensure_rustup_on_path
where rustup >nul 2>&1
if errorlevel 1 (
    echo [ERROR] rustup is still unavailable after install.
    call :mark_failure "Rustup"
    exit /b 0
)

rustup toolchain install stable
if errorlevel 1 (
    echo [ERROR] Failed to install Rust stable toolchain.
    call :mark_failure "Rust stable toolchain"
    exit /b 0
)

rustup default stable
if errorlevel 1 (
    echo [ERROR] Failed to set Rust stable as default toolchain.
    call :mark_failure "Rust default toolchain"
    exit /b 0
)

rustup target add x86_64-pc-windows-msvc
if errorlevel 1 (
    echo [ERROR] Failed to add Rust target x86_64-pc-windows-msvc.
    call :mark_failure "Rust Windows target"
    exit /b 0
)

set "RUST_VERSION="
set "CARGO_VERSION="
for /f "delims=" %%I in ('rustc --version 2^>nul') do if not defined RUST_VERSION set "RUST_VERSION=%%I"
for /f "delims=" %%I in ('cargo --version 2^>nul') do if not defined CARGO_VERSION set "CARGO_VERSION=%%I"
echo [OK] !RUST_VERSION!
echo [OK] !CARGO_VERSION!
exit /b 0
