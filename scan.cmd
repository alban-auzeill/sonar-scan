@echo off
setlocal EnableDelayedExpansion

set "SCRIPT_DIR=%~dp0"

:: Detect architecture
:: On a 64-bit OS running a 32-bit shell, PROCESSOR_ARCHITEW6432 holds the real arch.
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
    echo Binary not found: !BINARY! 1>&2
    echo Run build-dist.sh to build the binaries. 1>&2
    exit /b 1
)

"!BINARY!" %*
exit /b !ERRORLEVEL!
