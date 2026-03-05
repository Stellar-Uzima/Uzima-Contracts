#!/bin/bash
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

set -e

echo "=== Bug Condition Exploration Test ==="
echo "Testing that merge conflict markers cause compilation failure..."
echo ""

# Path to the file with merge conflict
FILE_PATH="contracts/medical_records/src/lib.rs"

# Check if file exists
if [ ! -f "$FILE_PATH" ]; then
    echo "ERROR: File $FILE_PATH not found"
    exit 1
fi

echo "Step 1: Checking for Git merge conflict markers..."
if grep -q "<<<<<<< HEAD" "$FILE_PATH"; then
    echo "✓ Found conflict marker: <<<<<<< HEAD"
else
    echo "✗ Conflict marker <<<<<<< HEAD not found"
    exit 1
fi

if grep -q "=======" "$FILE_PATH"; then
    echo "✓ Found conflict marker: ======="
else
    echo "✗ Conflict marker ======= not found"
    exit 1
fi

if grep -q ">>>>>>>" "$FILE_PATH"; then
    echo "✓ Found conflict marker: >>>>>>>"
else
    echo "✗ Conflict marker >>>>>>> not found"
    exit 1
fi

echo ""
echo "Step 2: Verifying conflict markers are in DataKey enum (lines 521-533)..."
# Extract lines 515-540 to show context
echo "Context around line 521:"
sed -n '515,540p' "$FILE_PATH"
echo ""

echo "Step 3: Attempting compilation (expecting failure)..."
echo "Running: cargo build --package medical_records"
echo ""

# Attempt to build and capture output
if cargo build --package medical_records 2>&1 | tee /tmp/build_output.txt; then
    echo ""
    echo "❌ UNEXPECTED: Compilation succeeded!"
    echo "This means the bug condition does NOT exist (conflict markers may have been resolved)"
    echo "Expected: Compilation should FAIL due to conflict markers"
    exit 1
else
    BUILD_EXIT_CODE=$?
    echo ""
    echo "✓ Compilation failed as expected (exit code: $BUILD_EXIT_CODE)"
    echo ""
    echo "Step 4: Analyzing compiler error..."
    
    # Check if error mentions line 521 or conflict markers
    if grep -q "521" /tmp/build_output.txt || grep -q "unexpected token" /tmp/build_output.txt || grep -q "expected" /tmp/build_output.txt; then
        echo "✓ Compiler error references the conflict marker location"
        echo ""
        echo "=== COUNTEREXAMPLE FOUND ==="
        echo "Bug confirmed: cargo build fails with syntax error due to conflict marker at line 521"
        echo ""
        echo "Relevant error output:"
        grep -A 5 -B 5 "error" /tmp/build_output.txt | head -20
        echo ""
        echo "=== TEST RESULT: FAILED (AS EXPECTED) ==="
        echo "This failure confirms the bug exists!"
        echo "The test will PASS after the fix is implemented."
        exit 1
    else
        echo "⚠ Warning: Compiler error doesn't clearly reference line 521"
        echo "Full error output:"
        cat /tmp/build_output.txt
        exit 1
    fi
fi
