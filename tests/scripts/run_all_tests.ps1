# Master test runner for all installation script tests
# Runs all test suites and reports results

$ErrorActionPreference = "Stop"

$testsPassed = 0
$testsFailed = 0

Write-Host "`n╔════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║  Portly Installation Scripts - Test Suite Runner          ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════╝`n" -ForegroundColor Cyan

# Test 1: PowerShell install script
Write-Host "Running test suite 1/3: PowerShell Install Script..." -ForegroundColor Yellow
try {
    & ".\tests\scripts\test_install_ps1.ps1"
    $testsPassed++
    Write-Host "✓ Test suite 1/3 passed`n" -ForegroundColor Green
} catch {
    $testsFailed++
    Write-Host "✗ Test suite 1/3 failed`n" -ForegroundColor Red
}

# Test 2: Bash install script
Write-Host "Running test suite 2/3: Bash Install Script..." -ForegroundColor Yellow
try {
    & ".\tests\scripts\test_install_sh.ps1"
    $testsPassed++
    Write-Host "✓ Test suite 2/3 passed`n" -ForegroundColor Green
} catch {
    $testsFailed++
    Write-Host "✗ Test suite 2/3 failed`n" -ForegroundColor Red
}

# Test 3: Uninstall scripts
Write-Host "Running test suite 3/3: Uninstall Scripts..." -ForegroundColor Yellow
try {
    & ".\tests\scripts\test_uninstall.ps1"
    $testsPassed++
    Write-Host "✓ Test suite 3/3 passed`n" -ForegroundColor Green
} catch {
    $testsFailed++
    Write-Host "✗ Test suite 3/3 failed`n" -ForegroundColor Red
}

# Summary
Write-Host "╔════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║  Test Results Summary                                      ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════╝`n" -ForegroundColor Cyan

Write-Host "Test Suites Passed: $testsPassed/3" -ForegroundColor $(if ($testsPassed -eq 3) { "Green" } else { "Yellow" })
Write-Host "Test Suites Failed: $testsFailed/3" -ForegroundColor $(if ($testsFailed -eq 0) { "Green" } else { "Red" })

if ($testsFailed -eq 0) {
    Write-Host "`n✓ All test suites passed!" -ForegroundColor Green
    exit 0
} else {
    Write-Host "`n✗ Some test suites failed" -ForegroundColor Red
    exit 1
}
