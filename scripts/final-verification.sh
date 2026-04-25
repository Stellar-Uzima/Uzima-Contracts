#!/bin/bash
# Final verification of all naming fixes

echo "🔍 FINAL VERIFICATION OF NAMING FIXES"
echo "====================================="

ALL_PASS=true

echo ""
echo "1. Checking for remaining ErrorLevel issues..."
if grep -r "ErrorLevel" contracts/ --include="*.rs" 2>/dev/null; then
    echo "❌ FAIL: ErrorLevel still exists somewhere"
    ALL_PASS=false
else
    echo "✅ PASS: No ErrorLevel found anywhere"
fi

echo ""
echo "2. Checking for remaining TimelockNotElasped typos..."
if grep -r "TimelockNotElasped" contracts/ --include="*.rs" 2>/dev/null; then
    echo "❌ FAIL: TimelockNotElasped still exists somewhere"
    ALL_PASS=false
else
    echo "✅ PASS: No TimelockNotElasped found anywhere"
fi

echo ""
echo "3. Checking that fixes are applied correctly..."
echo "   a) medical_records LogLevel::Error..."
if grep -q "LogLevel::Error" contracts/medical_records/src/lib.rs; then
    echo "      ✅ PASS: LogLevel::Error exists in medical_records"
else
    echo "      ❌ FAIL: LogLevel::Error missing in medical_records"
    ALL_PASS=false
fi

echo "   b) genomic_data LogLevel::Error..."
if grep -q "LogLevel::Error" contracts/genomic_data/src/lib.rs; then
    echo "      ✅ PASS: LogLevel::Error exists in genomic_data"
else
    echo "      ❌ FAIL: LogLevel::Error missing in genomic_data"
    ALL_PASS=false
fi

echo "   c) medical_records Error::TimelockNotElapsed..."
if grep -q "Error::TimelockNotElapsed" contracts/medical_records/src/lib.rs; then
    echo "      ✅ PASS: Error::TimelockNotElapsed exists in medical_records"
else
    echo "      ❌ FAIL: Error::TimelockNotElapsed missing in medical_records"
    ALL_PASS=false
fi

echo "   d) test file Error::TimelockNotElapsed..."
if grep -q "Error::TimelockNotElapsed" contracts/medical_records/tests/crypto_security_tests.rs; then
    echo "      ✅ PASS: Error::TimelockNotElapsed exists in test file"
else
    echo "      ❌ FAIL: Error::TimelockNotElapsed missing in test file"
    ALL_PASS=false
fi

echo ""
echo "4. Checking standards documentation exists..."
if [ -f "docs/CODING_STANDARDS.md" ] && [ -f ".clippy.toml" ] && [ -f "CONTRIBUTING.md" ]; then
    echo "✅ PASS: All standards documentation exists"
else
    echo "❌ FAIL: Missing some standards documentation"
    ALL_PASS=false
fi

echo ""
echo "5. Checking automation scripts exist..."
if [ -f "scripts/check-naming.sh" ] && [ -f "scripts/verify-fixes.sh" ]; then
    echo "✅ PASS: All automation scripts exist"
else
    echo "❌ FAIL: Missing some automation scripts"
    ALL_PASS=false
fi

echo ""
echo "====================================="
if $ALL_PASS; then
    echo " ALL VERIFICATIONS PASSED!"
    echo "All naming inconsistencies have been fixed."
    echo "Standards and automation are in place."
else
    echo "❌ SOME VERIFICATIONS FAILED"
    exit 1
fi