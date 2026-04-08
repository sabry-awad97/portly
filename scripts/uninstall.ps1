<#
.SYNOPSIS
    Uninstalls Portly CLI tool from Windows

.DESCRIPTION
    Removes Portly binary, cleans up PATH, and optionally removes configuration files.

.PARAMETER Force
    Skip confirmation prompts

.PARAMETER RemoveConfig
    Also remove configuration files from %APPDATA%\portly

.EXAMPLE
    iwr -useb https://raw.githubusercontent.com/sabry-awad97/portly/main/scripts/uninstall.ps1 | iex

.EXAMPLE
    .\uninstall.ps1 -Force

.EXAMPLE
    .\uninstall.ps1 -RemoveConfig

.NOTES
    Author: Sabry Awad
    Repository: https://github.com/sabry-awad97/portly
#>

param(
    [switch]$Force,
    [switch]$RemoveConfig
)

$ErrorActionPreference = "Stop"

Write-Host "`nUninstalling Portly..." -ForegroundColor Cyan

# Default installation directory
$InstallDir = "$env:LOCALAPPDATA\portly\bin"
$ConfigDir = "$env:APPDATA\portly"

# Check if installed
if (-not (Test-Path $InstallDir)) {
    Write-Host "Portly is not installed at $InstallDir" -ForegroundColor Yellow
    exit 0
}

# Confirmation
if (-not $Force) {
    $response = Read-Host "Are you sure you want to uninstall Portly? (y/N)"
    if ($response -ne 'y' -and $response -ne 'Y') {
        Write-Host "Uninstall cancelled" -ForegroundColor Yellow
        exit 0
    }
}

# Remove binary
try {
    Write-Host "Removing binary from $InstallDir..." -ForegroundColor Gray
    Remove-Item -Path $InstallDir -Recurse -Force
    Write-Host "✓ Binary removed" -ForegroundColor Green
} catch {
    Write-Host "Error: Failed to remove binary" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Yellow
    exit 1
}

# Remove from PATH
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -like "*$InstallDir*") {
    Write-Host "Removing from PATH..." -ForegroundColor Gray
    $newPath = ($userPath -split ';' | Where-Object { $_ -ne $InstallDir }) -join ';'
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    Write-Host "✓ Removed from PATH" -ForegroundColor Green
}

# Remove config if requested
if ($RemoveConfig -and (Test-Path $ConfigDir)) {
    if (-not $Force) {
        $response = Read-Host "Also remove configuration files from $ConfigDir? (y/N)"
        if ($response -ne 'y' -and $response -ne 'Y') {
            Write-Host "Configuration files kept" -ForegroundColor Gray
        } else {
            Remove-Item -Path $ConfigDir -Recurse -Force
            Write-Host "✓ Configuration files removed" -ForegroundColor Green
        }
    } else {
        Remove-Item -Path $ConfigDir -Recurse -Force
        Write-Host "✓ Configuration files removed" -ForegroundColor Green
    }
}

Write-Host "`n✓ Portly uninstalled successfully!" -ForegroundColor Green
Write-Host "Restart your terminal for PATH changes to take effect" -ForegroundColor Gray
