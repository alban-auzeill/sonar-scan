# sonar-scan launcher for PowerShell
#
# Execute without installing:
#   & ([scriptblock]::Create((Invoke-RestMethod 'https://my.sonarqube.com/scan.ps1'))) --help
#   & ([scriptblock]::Create((Invoke-RestMethod 'https://my.sonarqube.com/scan.ps1'))) --token TOKEN --url URL
#
# Install sonar-scan.exe to a directory on PATH:
#   & ([scriptblock]::Create((Invoke-RestMethod 'https://my.sonarqube.com/scan.ps1'))) --install "$env:USERPROFILE\bin"

$SONAR_SCAN_VERSION = "1.2.0"

# Detect architecture (PROCESSOR_ARCHITEW6432 is set when 32-bit PowerShell runs on a 64-bit OS)
$arch = $null
if     ($env:PROCESSOR_ARCHITECTURE  -eq "AMD64") { $arch = "x86_64" }
elseif ($env:PROCESSOR_ARCHITECTURE  -eq "ARM64") { $arch = "aarch64" }
elseif ($env:PROCESSOR_ARCHITEW6432  -eq "AMD64") { $arch = "x86_64" }
elseif ($env:PROCESSOR_ARCHITEW6432  -eq "ARM64") { $arch = "aarch64" }

if (-not $arch) {
    Write-Host "Unsupported architecture: $env:PROCESSOR_ARCHITECTURE" -ForegroundColor Red
    exit 1
}

# customize environment variables here

if ($args.Count -ge 1 -and $args[0] -eq "--install") {
    if ($args.Count -lt 2 -or -not $args[1]) {
        Write-Host "Error: --install needs an existing directory as argument" -ForegroundColor Red
        exit 1
    }
    $installDir = $args[1]
    if (-not (Test-Path -LiteralPath $installDir -PathType Container)) {
        Write-Host "Error: --install needs an existing directory as argument: $installDir" -ForegroundColor Red
        exit 1
    }
    $installDir = (Resolve-Path -LiteralPath $installDir).Path
    Write-Host "Installing sonar-scan into $installDir"
    $binary = Join-Path $installDir "sonar-scan.exe"
} else {
    $binary = Join-Path $env:USERPROFILE ".sonar\cache\sonar-scan-$SONAR_SCAN_VERSION\sonar-scan-$arch-windows.exe"
    if (Test-Path -LiteralPath $binary) {
        & $binary @args
        exit $LASTEXITCODE
    }
    $cacheDir = Join-Path $env:USERPROFILE ".sonar\cache\sonar-scan-$SONAR_SCAN_VERSION"
    if (-not (Test-Path -LiteralPath $cacheDir)) {
        New-Item -ItemType Directory -Path $cacheDir | Out-Null
    }
}

$url = "https://github.com/alban-auzeill/sonar-scan/releases/download/v$SONAR_SCAN_VERSION/sonar-scan-$arch-windows.exe"
$ProgressPreference = 'SilentlyContinue'
try {
    Invoke-WebRequest -Uri $url -OutFile $binary -UseBasicParsing -ErrorAction Stop
} catch {
    Write-Host "Error: Failed to download $url into $binary : $_" -ForegroundColor Red
    if (Test-Path -LiteralPath $binary) { Remove-Item -LiteralPath $binary -Force }
    exit 1
}

if (-not (Test-Path -LiteralPath $binary) -or (Get-Item -LiteralPath $binary).Length -eq 0) {
    Write-Host "Error: $binary is empty or not found after download" -ForegroundColor Red
    if (Test-Path -LiteralPath $binary) { Remove-Item -LiteralPath $binary -Force }
    exit 1
}

if ($args.Count -ge 1 -and $args[0] -eq "--install") {
    exit 0
} else {
    & $binary @args
    exit $LASTEXITCODE
}
