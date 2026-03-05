# Cortex Professional Windows Installer
# Purpose: High-speed binary installation of Cortex runtime for Windows

$ErrorActionPreference = 'Stop'

Write-Host "Cortex Professional Installer Starting..." -ForegroundColor Blue

# 1. Dependency Checks - None strictly required for binary download besides PowerShell 5+

# 2. Version Discovery
Write-Host "Detecting latest version..." -ForegroundColor Blue
try {
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/Dopove/Cortex/releases/latest"
    $LATEST_TAG = $release.tag_name
} catch {
    Write-Host "Warning: Could not detect latest version via API, falling back to v2.5.9" -ForegroundColor Yellow
    $LATEST_TAG = "v2.5.9"
}
Write-Host "Target Version: $LATEST_TAG" -ForegroundColor Green

# 3. Platform Detection
$ARCH = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
$SUFFIX = "x64_windows"

if ($ARCH -ne 'X64') {
    Write-Host "Unsupported Windows architecture: $ARCH. Cortex currently only supports x64 on Windows." -ForegroundColor Red
    exit 1
}

# 4. Binary Download
$DOWNLOAD_URL = "https://github.com/Dopove/Cortex/releases/download/${LATEST_TAG}/cortex_${LATEST_TAG}_${SUFFIX}.zip"

Write-Host "Platform: Windows ($ARCH)" -ForegroundColor Blue
Write-Host "Downloading binary from: $DOWNLOAD_URL" -ForegroundColor Blue

$TMP_DIR = Join-Path $env:TEMP ([Guid]::NewGuid().ToString())
New-Item -ItemType Directory -Path $TMP_DIR | Out-Null

$ZIP_FILE = Join-Path $TMP_DIR "cortex.zip"

try {
    Invoke-WebRequest -Uri $DOWNLOAD_URL -OutFile $ZIP_FILE
    Expand-Archive -Path $ZIP_FILE -DestinationPath $TMP_DIR -Force
    $BINARY_PATH = Join-Path $TMP_DIR "cortex.exe"
} catch {
    Write-Host "Error: Failed to download or extract binary. Please check your internet connection or the release status." -ForegroundColor Red
    exit 1
}

# 5. Installation
$INSTALL_DIR = Join-Path $env:USERPROFILE ".local\bin"
if (-not (Test-Path $INSTALL_DIR)) {
    New-Item -ItemType Directory -Path $INSTALL_DIR | Out-Null
}

$DEST_BINARY = Join-Path $INSTALL_DIR "cortex.exe"
Write-Host "Installing binary to $INSTALL_DIR..." -ForegroundColor Blue
Copy-Item -Path $BINARY_PATH -Destination $DEST_BINARY -Force

# Add to PATH for current session if not already there
$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($currentPath -notlike "*$INSTALL_DIR*") {
    $newPath = "$currentPath;$INSTALL_DIR"
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    $env:Path = "$env:Path;$INSTALL_DIR"
    Write-Host "Added $INSTALL_DIR to User PATH." -ForegroundColor Yellow
}

# 6. Initialization
Write-Host "Cortex installed successfully!" -ForegroundColor Green
Write-Host "Running 'cortex init'..." -ForegroundColor Blue
& $DEST_BINARY init

Write-Host "`n========================================" -ForegroundColor Green
Write-Host "Cortex $LATEST_TAG is now available globally." -ForegroundColor Green
Write-Host "Try running: cortex --version" -ForegroundColor Blue
Write-Host "========================================" -ForegroundColor Green

# Cleanup
Remove-Item -Path $TMP_DIR -Recurigid -Force -ErrorAction SilentlyContinue
