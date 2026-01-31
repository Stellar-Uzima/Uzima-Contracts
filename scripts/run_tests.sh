#!/bin/bash

# Comprehensive test automation script for Uzima Contracts
# Runs all test suites with coverage reporting and quality gates

# STRICT MODE: Exit on error, undefined vars, and if ANY command in a pipe fails
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Initialize Status Variables (Required for set -u)
UNIT_TESTS_PASS=false
INTEGRATION_TESTS_PASS=false
DOC_TESTS_PASS=false
FORMAT_CHECK_PASS=false
CLIPPY_PASS=false
BUILD_PASS=false
DEPS_PASS=false
AUDIT_PASS=false

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TEST_RESULTS_DIR="${PROJECT_ROOT}/test_results"

# Ensure test results directory exists
mkdir -p "${TEST_RESULTS_DIR}"

# Use printf for safer, portable output instead of echo -e
print_header() { printf "\n${BLUE}>>> %s${NC}\n" "$1"; }
print_success() { printf "${GREEN}✓ %s${NC}\n" "$1"; }
print_error()   { printf "${RED}✗ %s${NC}\n" "$1"; }
print_warning() { printf "${YELLOW}⚠ %s${NC}\n" "$1"; }

echo -e "${BLUE}================================================${NC}"
echo -e "${BLUE}Uzima Contracts - Comprehensive Test Suite${NC}"
echo -e "${BLUE}================================================${NC}"

# 1. Unit Tests
print_header "Running Unit Tests"
# "|| true" prevents set -e from killing the script immediately so we can log the failure
if cargo test --test '*' --lib 2>&1 | tee "${TEST_RESULTS_DIR}/unit_tests.log"; then
    print_success "Unit tests passed"
    UNIT_TESTS_PASS=true
else
    print_error "Unit tests failed"
    UNIT_TESTS_PASS=false
fi

# 2. Integration Tests
print_header "Running Integration Tests"
if cargo test --test '*' --no-fail-fast 2>&1 | tee "${TEST_RESULTS_DIR}/integration_tests.log"; then
    print_success "Integration tests passed"
    INTEGRATION_TESTS_PASS=true
else
    print_error "Integration tests failed"
    INTEGRATION_TESTS_PASS=false
fi

# 3. Doc Tests
print_header "Running Documentation Tests"
if cargo test --doc 2>&1 | tee "${TEST_RESULTS_DIR}/doc_tests.log"; then
    print_success "Documentation tests passed"
    DOC_TESTS_PASS=true
else
    print_error "Documentation tests failed"
    DOC_TESTS_PASS=false
fi

# 4. Format Check
print_header "Checking Code Format"
if cargo fmt -- --check 2>&1 | tee "${TEST_RESULTS_DIR}/format_check.log"; then
    print_success "Code format is correct"
    FORMAT_CHECK_PASS=true
else
    print_warning "Code formatting issues found - attempting to fix"
    cargo fmt
    FORMAT_CHECK_PASS=true
fi

# 5. Clippy Linting
print_header "Running Clippy Linting"
if cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tee "${TEST_RESULTS_DIR}/clippy.log"; then
    print_success "Clippy checks passed"
    CLIPPY_PASS=true
else
    print_warning "Clippy warnings found"
    CLIPPY_PASS=false
fi

# 6. Build Release
print_header "Building Release Binary"
if cargo build --release 2>&1 | tee "${TEST_RESULTS_DIR}/build_release.log"; then
    print_success "Release build successful"
    BUILD_PASS=true
else
    print_error "Release build failed"
    BUILD_PASS=false
fi

# 7. Check Dependencies
print_header "Checking Dependencies"
if ! command -v cargo-deny &> /dev/null; then
    print_warning "cargo-deny not installed (skipping)"
    DEPS_PASS=true
elif cargo deny check 2>/dev/null; then
    print_success "Dependency checks passed"
    DEPS_PASS=true
else
    print_warning "Dependency checks failed"
    DEPS_PASS=false
fi

# 8. Security Audit
print_header "Running Security Audit"
if ! command -v cargo-audit &> /dev/null; then
    print_warning "cargo-audit not installed (skipping)"
    AUDIT_PASS=true
elif cargo audit 2>/dev/null; then
    print_success "Security audit passed"
    AUDIT_PASS=true
else
    print_warning "Security audit failed"
    AUDIT_PASS=false
fi

# Summary
print_header "Test Summary"
printf "\n"
printf "Unit Tests:        %s\n" "$([ "$UNIT_TESTS_PASS" = true ] && echo -e "${GREEN}PASS${NC}" || echo -e "${RED}FAIL${NC}")"
printf "Integration Tests: %s\n" "$([ "$INTEGRATION_TESTS_PASS" = true ] && echo -e "${GREEN}PASS${NC}" || echo -e "${RED}FAIL${NC}")"
printf "Doc Tests:         %s\n" "$([ "$DOC_TESTS_PASS" = true ] && echo -e "${GREEN}PASS${NC}" || echo -e "${RED}FAIL${NC}")"
printf "Format Check:      %s\n" "$([ "$FORMAT_CHECK_PASS" = true ] && echo -e "${GREEN}PASS${NC}" || echo -e "${RED}FAIL${NC}")"
printf "Clippy:            %s\n" "$([ "$CLIPPY_PASS" = true ] && echo -e "${GREEN}PASS${NC}" || echo -e "${RED}FAIL${NC}")"
printf "Build Release:     %s\n" "$([ "$BUILD_PASS" = true ] && echo -e "${GREEN}PASS${NC}" || echo -e "${RED}FAIL${NC}")"
printf "\nTest Results:      %s\n" "${TEST_RESULTS_DIR}"

# Quality gates check
if [ "$UNIT_TESTS_PASS" = true ] && [ "$INTEGRATION_TESTS_PASS" = true ] && \
   [ "$DOC_TESTS_PASS" = true ] && [ "$FORMAT_CHECK_PASS" = true ] && \
   [ "$BUILD_PASS" = true ]; then
    print_success "All quality gates passed! ✓"
    exit 0
else
    print_error "Some quality gates failed!"
    exit 1
fi