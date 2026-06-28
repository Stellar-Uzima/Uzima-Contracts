#!/usr/bin/env bash

# Unit tests for advanced_cli.sh
# Run as: bash tests/cli/advanced_cli_tests.sh

set -euo pipefail

SCRIPT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../scripts" && pwd)/advanced_cli.sh"

if [[ ! -f "$SCRIPT" ]]; then
    echo "Script not found: $SCRIPT" >&2
    exit 1
fi

# 1) Help output
if ! bash "$SCRIPT" --help | grep -q "Advanced Uzima CLI"; then
    echo "FAIL: help output missing" >&2
    exit 1
fi

echo "PASS: help output" 

# 2) Unsupported network validation
if bash "$SCRIPT" tx-history unsupportednet GABC 10 >/dev/null 2>&1; then
    echo "FAIL: unsupported network should error" >&2
    exit 1
fi

echo "PASS: unsupported network validation" 

# 3) Missing required args errors
set +e
bash "$SCRIPT" account-info local >/dev/null 2>&1
code=$?
set -e
if [[ $code -eq 0 ]]; then
    echo "FAIL: account-info missing args should fail" >&2
    exit 1
fi

echo "PASS: missing args validation" 

# 4) Account manage list should not fail
if ! bash "$SCRIPT" account-manage list >/dev/null 2>&1; then
    echo "FAIL: account-manage list failed" >&2
    exit 1
fi

echo "PASS: account-manage list" 

# 5) Batch-invoke missing file validation
if bash "$SCRIPT" batch-invoke GXYZ local nonexistent.txt >/dev/null 2>&1; then
    echo "FAIL: batch-invoke should fail for missing file" >&2
    exit 1
fi

echo "PASS: batch-invoke missing file validation" 

echo "All advanced CLI unit tests passed!"
