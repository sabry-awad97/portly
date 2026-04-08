# Test script for install.sh (PowerShell version)
# Verifies that the bash installation script has required components

$ErrorActionPreference = "Stop"

Write-Host "`n=== Testing install.sh ===" -ForegroundColor Cyan

# Test 1: Script file exists
Write-Host "`nTest 1: Checking if install.sh exists..." -ForegroundColor Yellow
if (Test-Path "scripts/install.sh") {
    Write-Host "✓ PASS: install.sh exists" -ForegroundColor Green
} else {
    Write-Host "✗ FAIL: install.sh not found" -ForegroundColor Red
    exit 1
}

$scriptContent = Get-Content "scripts/install.sh" -Raw

# Test 2: Script has proper bash shebang
Write-Host "`nTest 2: Checking for bash shebang..." -ForegroundColor Yellow
if ($scriptContent -match '#!/.*bash') {
    Write-Host "✓ PASS: Valid bash shebang" -ForegroundColor Green
} else {
    Write-Host "✗ FAIL: Missing or invalid bash shebang" -ForegroundColor Red
    exit 1
}

# Test 3: Script has set -e for error handling
Write-Host "`nTest 3: Checking for error handling (set -e)..." -ForegroundColor Yellow
if ($scriptContent -match 'set -e') {
    Write-Host "✓ PASS: Error handling present" -ForegroundColor Green
} else {
    Write-Host "✗ FAIL: Error handling missing" -ForegroundColor Red
    exit 1
}

# Test 4: Script has OS detection
Write-Host "`nTest 4: Checking for OS detection..." -ForegroundColor Yellow
if ($scriptContent -match 'uname -s') {
    Write-Host "✓ PASS: OS detection present" -ForegroundColor Green
} else {
    Write-Host "✗ FAIL: OS detection missing" -ForegroundColor Red
    exit 1
}

# Test 5: Script has architecture detection
Write-Host "`nTest 5: Checking for architecture detection..." -ForegroundColor Yellow
if ($scriptContent -match 'uname -m') {
    Write-Host "✓ PASS: Architecture detection present" -ForegroundColor Green
} else {
    Write-Host "✗ FAIL: Architecture detection missing" -ForegroundColor Red
    exit 1
}

# Test 6: Script has GitHub API integration
Write-Host "`nTest 6: Checking for GitHub API integration..." -ForegroundColor Yellow
if ($scriptContent -match 'api\.github\.com') {
    Write-Host "✓ PASS: GitHub API integration present" -ForegroundColor Green
} else {
    Write-Host "✗ FAIL: GitHub API integration missing" -ForegroundColor Red
    exit 1
}

# Test 7: Script has dependency checking
Write-Host "`nTest 7: Checking for dependency checking..." -ForegroundColor Yellow
if ($scriptContent -match 'command -v') {
    Write-Host "✓ PASS: Dependency checking present" -ForegroundColor Green
} else {
    Write-Host "✗ FAIL: Dependency checking missing" -ForegroundColor Red
    exit 1
}

# Test 8: Script has PATH checking
Write-Host "`nTest 8: Checking for PATH verification..." -ForegroundColor Yellow
if ($scriptContent -match '\$PATH') {
    Write-Host "✓ PASS: PATH verification present" -ForegroundColor Green
} else {
    Write-Host "✗ FAIL: PATH verification missing" -ForegroundColor Red
    exit 1
}

# Test 9: Script has installation verification
Write-Host "`nTest 9: Checking for installation verification..." -ForegroundColor Yellow
if ($scriptContent -match '--version') {
    Write-Host "✓ PASS: Installation verification present" -ForegroundColor Green
} else {
    Write-Host "✗ FAIL: Installation verification missing" -ForegroundColor Red
    exit 1
}

# Test 10: Script has color output
Write-Host "`nTest 10: Checking for color output..." -ForegroundColor Yellow
if ($scriptContent -match '\\033\[') {
    Write-Host "✓ PASS: Color output present" -ForegroundColor Green
} else {
    Write-Host "✗ FAIL: Color output missing" -ForegroundColor Red
    exit 1
}

Write-Host "`n=== All tests passed! ===" -ForegroundColor Green
exit 0
