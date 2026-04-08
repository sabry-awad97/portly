<#
.SYNOPSIS
    Installs Portly CLI tool for Windows

.DESCRIPTION
    Downloads the latest release from GitHub and installs to user's local directory.
    Automatically detects system architecture and adds to PATH.

.PARAMETER Version
    Specific version to install (default: latest)

.PARAMETER InstallDir
    Installation directory (default: $env:LOCALAPPDATA\portly\bin)

.EXAMPLE
    iwr -useb https://raw.githubusercontent.com/sabry-awad97/portly/main/scripts/install.ps1 | iex

.EXAMPLE
    .\install.ps1 -Version "0.1.0"

.EXAMPLE
    .\install.ps1 -InstallDir "C:\Tools\portly"

.NOTES
    Author: Sabry Awad
    Repository: https://github.com/sabry-awad97/portly
#>

param(
    [string]$Version = "latest",
    [string]$InstallDir = "$env:LOCALAPPDATA\portly\bin"
)

$ErrorActionPreference = "Stop"

# Configuration
$REPO = "sabry-awad97/portly"

Write-Host "`nInstalling Portly..." -ForegroundColor Cyan

# Detect architecture
$arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "i686" }
Write-Host "Detected architecture: $arch" -ForegroundColor Gray

# Get release information
$releaseUrl = if ($Version -eq "latest") {
    "https://api.github.com/repos/$REPO/releases/latest"
} else {
    "https://api.github.com/repos/$REPO/releases/tags/v$Version"
}

try {
    Write-Host "Fetching release information..." -ForegroundColor Gray
    $release = Invoke-RestMethod $releaseUrl -Headers @{ "User-Agent" = "portly-installer" }
    $version = $release.tag_name
    Write-Host "Found version: $version" -ForegroundColor Green
} catch {
    Write-Host "Error: Failed to fetch release information" -ForegroundColor Red
    Write-Host "Please check your internet connection and try again" -ForegroundColor Yellow
    Write-Host "URL attempted: $releaseUrl" -ForegroundColor Gray
    exit 1
}

# Find matching asset
$assetPattern = "*windows*$arch*.zip"
$asset = $release.assets | Where-Object { $_.name -like $assetPattern } | Select-Object -First 1

if (-not $asset) {
    Write-Host "Error: No Windows binary found for architecture $arch" -ForegroundColor Red
    Write-Host "Available assets:" -ForegroundColor Yellow
    $release.assets | ForEach-Object { Write-Host "  - $($_.name)" -ForegroundColor Gray }
    exit 1
}

# Download binary
$downloadUrl = $asset.browser_download_url
$zipFile = Join-Path $env:TEMP "portly.zip"

try {
    Write-Host "Downloading from $downloadUrl..." -ForegroundColor Gray
    Invoke-WebRequest $downloadUrl -OutFile $zipFile -UseBasicParsing
    Write-Host "Downloaded successfully" -ForegroundColor Green
} catch {
    Write-Host "Error: Failed to download binary" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Yellow
    exit 1
}

# Create install directory
if (-not (Test-Path $InstallDir)) {
    Write-Host "Creating installation directory: $InstallDir" -ForegroundColor Gray
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

# Extract binary
try {
    Write-Host "Extracting binary..." -ForegroundColor Gray
    Expand-Archive $zipFile -DestinationPath $InstallDir -Force
    Remove-Item $zipFile
    Write-Host "Extracted successfully" -ForegroundColor Green
} catch {
    Write-Host "Error: Failed to extract binary" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Yellow
    exit 1
}

# Add to PATH if not present
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
$pathUpdated = $false

if ($userPath -notlike "*$InstallDir*") {
    Write-Host "Adding to PATH..." -ForegroundColor Gray
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$InstallDir", "User")
    $pathUpdated = $true
    Write-Host "Added to PATH" -ForegroundColor Green
} else {
    Write-Host "Already in PATH" -ForegroundColor Gray
}

# Refresh PATH in current session
Write-Host "Refreshing PATH in current session..." -ForegroundColor Gray
$env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
Write-Host "PATH refreshed - portly command is now available!" -ForegroundColor Green

# Verify installation
$portlyExe = Join-Path $InstallDir "portly.exe"
if (Test-Path $portlyExe) {
    Write-Host "`nVerifying installation..." -ForegroundColor Gray
    try {
        $installedVersion = & $portlyExe --version 2>&1
        Write-Host "✓ Portly installed successfully!" -ForegroundColor Green
        Write-Host "  Version: $installedVersion" -ForegroundColor Gray
        Write-Host "  Location: $portlyExe" -ForegroundColor Gray
        
        # Test if portly command works
        Write-Host "`nTesting portly command..." -ForegroundColor Gray
        $testResult = portly --version 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✓ 'portly' command is ready to use!" -ForegroundColor Green
        } else {
            Write-Host "Note: 'portly' command will work after restarting terminal" -ForegroundColor Yellow
        }
        
        Write-Host "`nGet started with:" -ForegroundColor Cyan
        Write-Host "  portly          # List all ports" -ForegroundColor White
        Write-Host "  portly --help   # Show all commands" -ForegroundColor White
        Write-Host "  portly ps       # List all processes" -ForegroundColor White
    } catch {
        Write-Host "Warning: Binary installed but version check failed" -ForegroundColor Yellow
        Write-Host "  Location: $portlyExe" -ForegroundColor Gray
    }
} else {
    Write-Host "Warning: Binary not found at expected location" -ForegroundColor Yellow
    Write-Host "Please check $InstallDir" -ForegroundColor Yellow
    exit 1
}
