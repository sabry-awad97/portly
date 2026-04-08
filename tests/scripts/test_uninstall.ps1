# Test script for uninstall scripts
# Verifies that uninstall scripts exist and have required functionality

$ErrorActionPreference = "Stop"

Write-Host "`n=== Testing Uninstall Scripts ===" -ForegroundColor Cyan

# Test PowerShell uninstall script
Write-Host "`n--- Testing uninstall.ps1 ---" -ForegroundColor Yellow

Write-Host "`nTest 1: Checking if uninstall.ps1 exists..." -ForegroundColor Yellow
if (Test-Path "scripts/uninstall.ps1") {
    Write-Host "✓ PASS: uninstall.ps1 exists" -ForegroundColor Green
} else {
    Write-Host "✗ FAIL: uninstall.ps1 not found" -ForegroundColor Red
    exit 1
}

$uninstallContent = Get-Content "scripts/uninstall.ps1" -Raw

Write-Host "`nTest 2: Checking for help documentation..." -ForegroundColor Yellow
if ($uninstallContent -match '\.SYNOPSIS') {
    Write-Host "✓ PASS: Help documentation present" -ForegroundColor Green
} else {
    Write-Host "✗ FAIL: Help documentation missing" -ForegroundColor Red
    exit 1
}

Write-Host "`nTest 3: Checking for binary removal..." -ForegroundColor Yellow
if ($uninstallContent -match 'Remove-Item') {
    Write-Host "✓ PASS: Binary removal present" -ForegroundColor Green
} else {
    Write-Host "✗ FAIL: Binary removal missing" -ForegroundColor Red
    exit 1
}

Write-Host "`nTest 4: Checking for PATH cleanup..." -ForegroundColor Yellow
if ($uninstallContent -match 'SetEnvironmentVariable.*Path') {
    Write-Host "✓ PASS: PATH cleanup present" -ForegroundColor Green
} else {
    Write-Host "✗ FAIL: PATH cleanup missing" -ForegroundColor Red
    exit 1
}

Write-Host "`nTest 5: Checking for confirmation prompt..." -ForegroundColor Yellow
if ($uninstallContent -match 'Read-Host|Confirm') {
    Write-Host "✓ PASS: Confirmation prompt present" -ForegroundColor Green
} else {
    Write-Host "✗ FAIL: Confirmation prompt missing" -ForegroundColor Red
    exit 1
}

# Test bash uninstall script
Write-Host "`n--- Testing uninstall.sh ---" -ForegroundColor Yellow

Write-Host "`nTest 6: Checking if uninstall.sh exists..." -ForegroundColor Yellow
if (Test-Path "scripts/uninstall.sh") {
    Write-Host "✓ PASS: uninstall.sh exists" -ForegroundColor Green
} else {
    Write-Host "✗ FAIL: uninstall.sh not found" -ForegroundColor Red
    exit 1
}

$uninstallShContent = Get-Content "scripts/uninstall.sh" -Raw

Write-Host "`nTest 7: Checking for bash shebang..." -ForegroundColor Yellow
if ($uninstallShContent -match '#!/.*bash') {
    Write-Host "✓ PASS: Valid bash shebang" -ForegroundColor Green
} else {
    Write-Host "✗ FAIL: Missing or invalid bash shebang" -ForegroundColor Red
    exit 1
}

Write-Host "`nTest 8: Checking for binary removal..." -ForegroundColor Yellow
if ($uninstallShContent -match 'rm') {
    Write-Host "✓ PASS: Binary removal present" -ForegroundColor Green
} else {
    Write-Host "✗ FAIL: Binary removal missing" -ForegroundColor Red
    exit 1
}

Write-Host "`nTest 9: Checking for confirmation prompt..." -ForegroundColor Yellow
if ($uninstallShContent -match 'read') {
    Write-Host "✓ PASS: Confirmation prompt present" -ForegroundColor Green
} else {
    Write-Host "✗ FAIL: Confirmation prompt missing" -ForegroundColor Red
    exit 1
}

Write-Host "`n=== All uninstall tests passed! ===" -ForegroundColor Green
exit 0
