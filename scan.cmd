@echo off
setlocal EnableDelayedExpansion

set "SONAR_SCAN_VERSION=1.4.0"

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

if "%~1"=="--install" (
    if "%~2"=="" (
        echo Error: --install needs an existing directory as argument 1>&2
        exit /b 1
    )
    if not exist "%~2\" (
        echo Error: --install needs an existing directory as argument: %~2 1>&2
        exit /b 1
    )
    echo Installing sonar-scan into %~f2
    set "BINARY=%~f2\sonar-scan.exe"
) else (
    set "BINARY=%USERPROFILE%\.sonar\cache\sonar-scan-!SONAR_SCAN_VERSION!\sonar-scan-!ARCH!-windows.exe"
    if exist "!BINARY!" (
        "!BINARY!" %*
        exit /b !ERRORLEVEL!
    )
    if not exist "%USERPROFILE%\.sonar\cache\sonar-scan-!SONAR_SCAN_VERSION!" (
        mkdir "%USERPROFILE%\.sonar\cache\sonar-scan-!SONAR_SCAN_VERSION!"
    )
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

if "%~1"=="--install" (
    exit /b 0
) else (
    "!BINARY!" %*
    exit /b !ERRORLEVEL!
)
