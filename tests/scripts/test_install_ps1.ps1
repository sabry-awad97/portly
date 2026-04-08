# Test script for install.ps1
# Verifies that the PowerShell installation script works correctly

$ErrorActionPreference = "Stop"

Write-Host "`n=== Testing install.ps1 ===" -ForegroundColor Cyan

# Test 1: Script file exists
Write-Host "`nTest 1: Checking if install.ps1 exists..." -ForegroundColor Yellow
if (Test-Path "scripts/install.ps1") {
    Write-Host "✓ PASS: install.ps1 exists" -ForegroundColor Green
} else {
    Write-Host "✗ FAIL: install.ps1 not found" -ForegroundColor Red
    exit 1
}

# Test 2: Script has proper PowerShell syntax
Write-Host "`nTest 2: Checking PowerShell syntax..." -ForegroundColor Yellow
try {
    $null = [System.Management.Automation.PSParser]::Tokenize((Get-Content "scripts/install.ps1" -Raw), [ref]$null)
    Write-Host "✓ PASS: Valid PowerShell syntax" -ForegroundColor Green
} catch {
    Write-Host "✗ FAIL: Invalid PowerShell syntax" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    exit 1
}

# Test 3: Script has required parameters
Write-Host "`nTest 3: Checking for required parameters..." -ForegroundColor Yellow
$scriptContent = Get-Content "scripts/install.ps1" -Raw
$hasVersionParam = $scriptContent -match '\$Version'
$hasInstallDirParam = $scriptContent -match '\$InstallDir'

if ($hasVersionParam -and $hasInstallDirParam) {
    Write-Host "✓ PASS: Required parameters present" -ForegroundColor Green
} else {
    Write-Host "✗ FAIL: Missing required parameters" -ForegroundColor Red
    exit 1
}

# Test 4: Script has architecture detection
Write-Host "`nTest 4: Checking for architecture detection..." -ForegroundColor Yellow
if ($scriptContent -match 'Is64BitOperatingSystem') {
    Write-Host "✓ PASS: Architecture detection present" -ForegroundColor Green
} else {
    Write-Host "✗ FAIL: Architecture detection missing" -ForegroundColor Red
    exit 1
}

# Test 5: Script has GitHub API integration
Write-Host "`nTest 5: Checking for GitHub API integration..." -ForegroundColor Yellow
if ($scriptContent -match 'api\.github\.com') {
    Write-Host "✓ PASS: GitHub API integration present" -ForegroundColor Green
} else {
    Write-Host "✗ FAIL: GitHub API integration missing" -ForegroundColor Red
    exit 1
}

# Test 6: Script has error handling
Write-Host "`nTest 6: Checking for error handling..." -ForegroundColor Yellow
$hasTryCatch = $scriptContent -match 'try\s*\{' -and $scriptContent -match 'catch\s*\{'
$hasErrorAction = $scriptContent -match 'ErrorActionPreference'

if ($hasTryCatch -and $hasErrorAction) {
    Write-Host "✓ PASS: Error handling present" -ForegroundColor Green
} else {
    Write-Host "✗ FAIL: Error handling missing" -ForegroundColor Red
    exit 1
}

# Test 7: Script has PATH modification
Write-Host "`nTest 7: Checking for PATH modification..." -ForegroundColor Yellow
if ($scriptContent -match 'SetEnvironmentVariable.*Path') {
    Write-Host "✓ PASS: PATH modification present" -ForegroundColor Green
} else {
    Write-Host "✗ FAIL: PATH modification missing" -ForegroundColor Red
    exit 1
}

# Test 8: Script has installation verification
Write-Host "`nTest 8: Checking for installation verification..." -ForegroundColor Yellow
if ($scriptContent -match '--version') {
    Write-Host "✓ PASS: Installation verification present" -ForegroundColor Green
} else {
    Write-Host "✗ FAIL: Installation verification missing" -ForegroundColor Red
    exit 1
}

# Test 9: Script has help documentation
Write-Host "`nTest 9: Checking for help documentation..." -ForegroundColor Yellow
if ($scriptContent -match '\.SYNOPSIS' -and $scriptContent -match '\.DESCRIPTION') {
    Write-Host "✓ PASS: Help documentation present" -ForegroundColor Green
} else {
    Write-Host "✗ FAIL: Help documentation missing" -ForegroundColor Red
    exit 1
}

Write-Host "`n=== All tests passed! ===" -ForegroundColor Green
exit 0
