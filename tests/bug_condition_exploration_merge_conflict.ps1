# Bug Condition Exploration Test for CI Merge Conflict Fix
# **Validates: Requirements 1.1, 1.2, 1.3**
#
# Property 1: Fault Condition - Valid Rust Syntax Without Conflict Markers
#
# CRITICAL: This test MUST FAIL on unfixed code - failure confirms the bug exists
# EXPECTED OUTCOME: Test FAILS (this is correct - it proves the bug exists)
#
# This test verifies that:
# 1. Git merge conflict markers are present in the DataKey enum
# 2. Compilation fails due to these conflict markers
# 3. The compiler error points to line 521 where conflict markers begin

$ErrorActionPreference = "Continue"

Write-Host "=== Bug Condition Exploration Test ===" -ForegroundColor Cyan
Write-Host "Testing that merge conflict markers cause compilation failure..."
Write-Host ""

# Path to the file with merge conflict
$FILE_PATH = "contracts/medical_records/src/lib.rs"

# Check if file exists
if (-not (Test-Path $FILE_PATH)) {
    Write-Host "ERROR: File $FILE_PATH not found" -ForegroundColor Red
    exit 1
}

Write-Host "Step 1: Checking for Git merge conflict markers..." -ForegroundColor Yellow
$fileContent = Get-Content $FILE_PATH -Raw

$marker1 = "<<<<<<<" + " HEAD"
$marker2 = "======="
$marker3 = ">>>>>>>"

$hasMarker1 = $fileContent.Contains($marker1)
$hasMarker2 = $fileContent.Contains($marker2)
$hasMarker3 = $fileContent.Contains($marker3)

if ($hasMarker1) {
    Write-Host "Found conflict marker: $marker1" -ForegroundColor Green
} else {
    Write-Host "Conflict marker not found: $marker1" -ForegroundColor Red
    exit 1
}

if ($hasMarker2) {
    Write-Host "Found conflict marker: $marker2" -ForegroundColor Green
} else {
    Write-Host "Conflict marker not found: $marker2" -ForegroundColor Red
    exit 1
}

if ($hasMarker3) {
    Write-Host "Found conflict marker: $marker3" -ForegroundColor Green
} else {
    Write-Host "Conflict marker not found: $marker3" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Step 2: Verifying conflict markers are in DataKey enum (lines 521-533)..." -ForegroundColor Yellow
$lines = Get-Content $FILE_PATH
Write-Host "Context around line 521:" -ForegroundColor Cyan
for ($i = 514; $i -lt 540 -and $i -lt $lines.Count; $i++) {
    Write-Host "$($i + 1): $($lines[$i])"
}
Write-Host ""

Write-Host "Step 3: Attempting compilation (expecting failure)..." -ForegroundColor Yellow
Write-Host "Running: rustc --crate-type lib contracts/medical_records/src/lib.rs --edition 2021"
Write-Host ""

# Attempt to compile with rustc and capture output
$buildOutput = rustc --crate-type lib contracts/medical_records/src/lib.rs --edition 2021 2>&1 | Out-String
$buildExitCode = $LASTEXITCODE

Write-Host $buildOutput

if ($buildExitCode -eq 0) {
    Write-Host ""
    Write-Host "UNEXPECTED: Compilation succeeded!" -ForegroundColor Red
    Write-Host "This means the bug condition does NOT exist (conflict markers may have been resolved)"
    Write-Host "Expected: Compilation should FAIL due to conflict markers"
    exit 1
} else {
    Write-Host ""
    Write-Host "Compilation failed as expected (exit code: $buildExitCode)" -ForegroundColor Green
    Write-Host ""
    Write-Host "Step 4: Analyzing compiler error..." -ForegroundColor Yellow
    
    # Check if error mentions line 521 or conflict markers
    $hasLineRef = $buildOutput.Contains("521")
    $hasConflictMarker = $buildOutput.Contains("diff marker") -or $buildOutput.Contains("conflict")
    
    if ($hasLineRef -or $hasConflictMarker) {
        Write-Host "Compiler error references the conflict marker location" -ForegroundColor Green
        Write-Host ""
        Write-Host "=== COUNTEREXAMPLE FOUND ===" -ForegroundColor Magenta
        Write-Host "Bug confirmed: cargo build fails with syntax error due to conflict marker at line 521"
        Write-Host ""
        Write-Host "Relevant error output:" -ForegroundColor Cyan
        $errorLines = $buildOutput -split "`n" | Select-Object -First 30
        $errorLines | ForEach-Object { Write-Host $_ }
        Write-Host ""
        Write-Host "=== TEST RESULT: FAILED (AS EXPECTED) ===" -ForegroundColor Magenta
        Write-Host "This failure confirms the bug exists!"
        Write-Host "The test will PASS after the fix is implemented."
        exit 1
    } else {
        Write-Host "Warning: Compiler error does not clearly reference line 521" -ForegroundColor Yellow
        Write-Host "Full error output:"
        Write-Host $buildOutput
        exit 1
    }
}
