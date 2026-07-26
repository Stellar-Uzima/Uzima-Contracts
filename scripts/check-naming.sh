#!/bin/bash
# Naming Convention Check Script for Uzima Contracts
# This script helps identify naming inconsistencies across the codebase

set -e

echo "🔍 Checking naming conventions across Uzima Contracts..."
echo "======================================================"

# Check for non-snake_case function names
echo ""
echo "1. Checking function names (should be snake_case)..."
echo "--------------------------------------------------"
find contracts -name "*.rs" -type f -exec grep -H "pub fn [A-Z]" {} \; | head -20

# Check for non-SCREAMING_SNAKE_CASE constants
echo ""
echo "2. Checking constant names (should be SCREAMING_SNAKE_CASE)..."
echo "------------------------------------------------------------"
find contracts -name "*.rs" -type f -exec grep -H "const [a-z]" {} \; | head -20

# Check for non-PascalCase type definitions
echo ""
echo "3. Checking type names (should be PascalCase)..."
echo "------------------------------------------------"
find contracts -name "*.rs" -type f -exec grep -H "struct [a-z]" {} \; | head -10
find contracts -name "*.rs" -type f -exec grep -H "enum [a-z]" {} \; | head -10

# Check for Err instead of Error
echo ""
echo "4. Checking for 'Err' instead of 'Error'..."
echo "------------------------------------------"
find contracts -name "*.rs" -type f -exec grep -H "enum Err" {} \;

# Check module names
echo ""
echo "5. Checking module names (should be snake_case)..."
echo "-------------------------------------------------"
find contracts -name "mod.rs" -type f -exec dirname {} \; | xargs -I {} basename {} | sort | uniq

echo ""
echo "======================================================"
echo "✅ Naming check complete!"
echo ""
echo "To fix issues, refer to:"
echo "  - docs/CODING_STANDARDS.md for naming conventions"
echo "  - .clippy.toml for linting rules"
echo ""
echo "Run 'cargo clippy -- -D warnings' for detailed linting."