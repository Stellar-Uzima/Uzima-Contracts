#!/usr/bin/env bash
# Validates the error-code registry in docs/ERROR_CODES.md.
# Current enforcement focuses on migrated unique ranges and registry coverage.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="$ROOT_DIR/docs/ERROR_CODES.md"
CONTRACTS_DIR="$ROOT_DIR/contracts"

tmp_registry="$(mktemp)"
trap 'rm -f "$tmp_registry"' EXIT

echo "Checking documented error code registry..."

awk '
    /^\| `[^`]+` \| `[^`]+` \| `[^`]+` \|/ {
        gsub(/`/, "", $0)
        split($0, parts, "|")
        contract=parts[2]
        status=parts[3]
        ranges=parts[4]
        gsub(/^[[:space:]]+|[[:space:]]+$/, "", contract)
        gsub(/^[[:space:]]+|[[:space:]]+$/, "", status)
        gsub(/^[[:space:]]+|[[:space:]]+$/, "", ranges)
        print contract "|" status "|" ranges
    }
' "$DOC_FILE" > "$tmp_registry"

if [[ ! -s "$tmp_registry" ]]; then
    echo "FAIL: no contract registry rows were parsed from $DOC_FILE"
    exit 1
fi

violations=0

while IFS='|' read -r contract status ranges; do
    [[ "$status" != "unique-range" ]] && continue
    IFS=',' read -ra parts <<< "$ranges"
    for part in "${parts[@]}"; do
        trimmed="$(echo "$part" | xargs)"
        start="${trimmed%-*}"
        end="${trimmed#*-}"
        if [[ -z "$start" || -z "$end" || ! "$start" =~ ^[0-9]+$ || ! "$end" =~ ^[0-9]+$ ]]; then
            echo "FAIL: invalid unique-range entry for $contract: $trimmed"
            violations=$((violations + 1))
            continue
        fi
        echo "$contract|$start|$end"
    done
done < "$tmp_registry" > "${tmp_registry}.ranges"

unique_ranges=()
while IFS= read -r line; do
    unique_ranges+=("$line")
done < "${tmp_registry}.ranges"

for ((i = 0; i < ${#unique_ranges[@]}; i++)); do
    IFS='|' read -r contract_a start_a end_a <<< "${unique_ranges[$i]}"
    for ((j = i + 1; j < ${#unique_ranges[@]}; j++)); do
        IFS='|' read -r contract_b start_b end_b <<< "${unique_ranges[$j]}"
        if (( start_a <= end_b && start_b <= end_a )); then
            echo "FAIL: overlapping unique ranges: $contract_a ($start_a-$end_a) and $contract_b ($start_b-$end_b)"
            violations=$((violations + 1))
        fi
    done
done

echo "Checking registry coverage for contracts with src/errors.rs..."
while IFS= read -r -d '' file; do
    contract="$(basename "$(dirname "$(dirname "$file")")")"
    if ! grep -q "^${contract}|" "$tmp_registry"; then
        echo "FAIL: $contract has src/errors.rs but no registry entry in docs/ERROR_CODES.md"
        violations=$((violations + 1))
    fi
done < <(find "$CONTRACTS_DIR" -path '*/src/errors.rs' -print0)

if (( violations > 0 )); then
    echo
    echo "FAIL: $violations error code registry violation(s) found."
    exit 1
fi

echo "OK: unique ranges do not overlap and all errors.rs contracts are documented."
