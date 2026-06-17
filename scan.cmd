@echo off
setlocal EnableDelayedExpansion

set "SCRIPT_DIR=%~dp0"
set "SONAR_SCAN_VERSION=1.1.0"

:: Detect architecture (PROCESSOR_ARCHITEW6432 is set when 32-bit cmd runs on a 64-bit OS)
set "ARCH="
if "%PROCESSOR_ARCHITECTURE%"=="AMD64"  set "ARCH=x86_64"
if "%PROCESSOR_ARCHITECTURE%"=="ARM64"  set "ARCH=aarch64"
if "%PROCESSOR_ARCHITEW6432%"=="AMD64"  set "ARCH=x86_64"
if "%PROCESSOR_ARCHITEW6432%"=="ARM64"  set "ARCH=aarch64"

if "!ARCH!"=="" (
    echo Unsupported architecture: %PROCESSOR_ARCHITECTURE% 1>&2
    exit /b 1
)

:: customize environment variables here

set "BINARY=!SCRIPT_DIR!target\dist\sonar-scan-!ARCH!-windows.exe"
if not exist "!BINARY!" (
    set "BINARY=%USERPROFILE%\.sonar\cache\sonar-scan-!SONAR_SCAN_VERSION!\sonar-scan-!ARCH!-windows.exe"
    if not exist "!BINARY!" (
        if not exist "%USERPROFILE%\.sonar\cache\sonar-scan-!SONAR_SCAN_VERSION!" (
            mkdir "%USERPROFILE%\.sonar\cache\sonar-scan-!SONAR_SCAN_VERSION!"
        )
        set "URL=https://github.com/alban-auzeill/sonar-scan/releases/download/v!SONAR_SCAN_VERSION!/sonar-scan-!ARCH!-windows.exe"
        curl -sSLf -o "!BINARY!" "!URL!"
        if !ERRORLEVEL! neq 0 (
            echo Error: curl failed to download !URL! into !BINARY! 1>&2
            if exist "!BINARY!" del "!BINARY!"
            exit /b 1
        )
        for %%F in ("!BINARY!") do if %%~zF==0 (
            echo Error: !BINARY! is empty after download 1>&2
            del "!BINARY!"
            exit /b 1
        )
    )
)

"!BINARY!" %*
exit /b !ERRORLEVEL!
