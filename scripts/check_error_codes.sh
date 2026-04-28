#!/usr/bin/env bash
# Validates that all error codes in errors.rs files fall within the approved
# category ranges defined in docs/ERROR_CODES.md. Exits non-zero on any violation.

set -euo pipefail

CONTRACTS_DIR="$(cd "$(dirname "$0")/.." && pwd)/contracts"
VIOLATIONS=0

check_file() {
    local file="$1"
    local contract
    contract=$(basename "$(dirname "$(dirname "$file")")")

    while IFS= read -r line; do
        # Match variant assignments: SomeName = 123,
        if [[ "$line" =~ ^[[:space:]]+[A-Za-z][A-Za-z0-9_]*[[:space:]]*=[[:space:]]*([0-9]+), ]]; then
            code="${BASH_REMATCH[1]}"
            # Validate code falls in one of the approved ranges
            if ! (( (code >= 100 && code <= 999) )); then
                echo "VIOLATION in $contract ($file): code $code is outside 100-999 range"
                VIOLATIONS=$((VIOLATIONS + 1))
            fi
            # Check for legacy sequential codes (1-99) which indicate not-yet-migrated files
            if (( code >= 1 && code <= 99 )); then
                echo "VIOLATION in $contract ($file): code $code uses legacy sequential numbering (expected 100+)"
                VIOLATIONS=$((VIOLATIONS + 1))
            fi
        fi
    done < "$file"
}

echo "Checking error codes across contracts..."

while IFS= read -r -d '' file; do
    check_file "$file"
done < <(find "$CONTRACTS_DIR" -name "errors.rs" -print0)

if (( VIOLATIONS > 0 )); then
    echo ""
    echo "FAIL: $VIOLATIONS error code violation(s) found."
    echo "See docs/ERROR_CODES.md for the approved ranges."
    exit 1
else
    echo "OK: all error codes are within approved ranges."
    exit 0
fi
