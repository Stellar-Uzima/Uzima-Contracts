#!/bin/bash
# Verify the naming inconsistency fixes

echo "🔍 Verifying naming fixes..."
echo "=============================="

# Check 1: Verify ErrorLevel was changed to Error
echo ""
echo "1. Checking LogLevel enum..."
if grep -q "ErrorLevel" contracts/medical_records/src/lib.rs; then
    echo "❌ FAIL: ErrorLevel still exists in lib.rs"
else
    echo "✅ PASS: ErrorLevel correctly changed to Error"
fi

# Check 2: Verify TimelockNotElasped was changed to TimelockNotElapsed
echo ""
echo "2. Checking TimelockNotElasped typo..."
if grep -q "TimelockNotElasped" contracts/medical_records/src/errors.rs; then
    echo "❌ FAIL: TimelockNotElasped still exists in errors.rs"
else
    echo "✅ PASS: TimelockNotElasped correctly changed to TimelockNotElapsed"
fi

# Check 3: Verify all references are updated
echo ""
echo "3. Checking all references are updated..."
ERROR_COUNT=0

# Check lib.rs for old references
if grep -q "Error::TimelockNotElasped" contracts/medical_records/src/lib.rs; then
    echo "❌ FAIL: Error::TimelockNotElasped found in lib.rs"
    ERROR_COUNT=$((ERROR_COUNT + 1))
fi

if grep -q "LogLevel::ErrorLevel" contracts/medical_records/src/lib.rs; then
    echo "❌ FAIL: LogLevel::ErrorLevel found in lib.rs"
    ERROR_COUNT=$((ERROR_COUNT + 1))
fi

# Check test file
if grep -q "Error::TimelockNotElasped" contracts/medical_records/tests/crypto_security_tests.rs; then
    echo "❌ FAIL: Error::TimelockNotElasped found in crypto_security_tests.rs"
    ERROR_COUNT=$((ERROR_COUNT + 1))
fi

# Check for correct new references
echo ""
echo "4. Checking new references exist..."
if grep -q "Error::TimelockNotElapsed" contracts/medical_records/src/lib.rs; then
    echo "✅ PASS: Error::TimelockNotElapsed found in lib.rs"
else
    echo "❌ FAIL: Error::TimelockNotElapsed not found in lib.rs"
    ERROR_COUNT=$((ERROR_COUNT + 1))
fi

if grep -q "LogLevel::Error" contracts/medical_records/src/lib.rs; then
    echo "✅ PASS: LogLevel::Error found in lib.rs"
else
    echo "❌ FAIL: LogLevel::Error not found in lib.rs"
    ERROR_COUNT=$((ERROR_COUNT + 1))
fi

if grep -q "Error::TimelockNotElapsed" contracts/medical_records/tests/crypto_security_tests.rs; then
    echo "✅ PASS: Error::TimelockNotElapsed found in crypto_security_tests.rs"
else
    echo "❌ FAIL: Error::TimelockNotElapsed not found in crypto_security_tests.rs"
    ERROR_COUNT=$((ERROR_COUNT + 1))
fi

echo ""
echo "=============================="
if [ $ERROR_COUNT -eq 0 ]; then
    echo "✅ ALL CHECKS PASSED: Naming fixes verified!"
else
    echo "❌ $ERROR_COUNT check(s) failed"
    exit 1
fi