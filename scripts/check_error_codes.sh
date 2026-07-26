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

# ---------------------------------------------------------------------------
# NEW: Detect contracts that define errors inline in lib.rs (no separate
# errors.rs) and verify every contract directory with a Cargo.toml either
# has an errors.rs or appears in the registry (inline errors).
# ---------------------------------------------------------------------------
echo "Checking inline error definitions in lib.rs..."
while IFS= read -r -d '' cargo_file; do
    contract="$(basename "$(dirname "$cargo_file")")"
    lib_dir="$(dirname "$cargo_file")/src"
    lib_file="$lib_dir/lib.rs"
    # Skip if contract already has a dedicated errors.rs (handled above)
    if [[ -f "$lib_dir/errors.rs" ]]; then
        continue
    fi
    # Skip if not a contract directory (no src/lib.rs)
    if [[ ! -f "$lib_file" ]]; then
        continue
    fi
    # Skip if contract already appears in registry
    if grep -q "^${contract}|" "$tmp_registry"; then
        continue
    fi
    # Check for inline error enum definition
    if grep -q 'pub enum Error' "$lib_file" 2>/dev/null; then
        echo "FAIL: $contract defines errors inline in lib.rs but has no registry entry in docs/ERROR_CODES.md"
        violations=$((violations + 1))
    fi
done < <(find "$CONTRACTS_DIR" -maxdepth 2 -name 'Cargo.toml' -print0)

# ---------------------------------------------------------------------------
# NEW: For each registered contract (any status), verify the declared range
# does not overlap with any unique-range. This catches legacy contracts
# whose ranges encroach on reserved unique ranges.
# ---------------------------------------------------------------------------
echo "Checking legacy ranges against unique ranges..."
unique_tmp="$(mktemp)"
trap 'rm -f "$tmp_registry" "$unique_tmp"' EXIT
# Re-extract unique ranges for comparison
while IFS='|' read -r contract status ranges; do
    [[ "$status" != "unique-range" ]] && continue
    IFS=',' read -ra parts <<< "$ranges"
    for part in "${parts[@]}"; do
        trimmed="$(echo "$part" | xargs)"
        start="${trimmed%-*}"
        end="${trimmed#*-}"
        echo "$contract|$start|$end" >> "$unique_tmp"
    done
done < "$tmp_registry"

while IFS='|' read -r contract status ranges; do
    [[ "$status" == "unique-range" ]] && continue
    IFS=',' read -ra parts <<< "$ranges"
    for part in "${parts[@]}"; do
        trimmed="$(echo "$part" | xargs)"
        start="${trimmed%-*}"
        end="${trimmed#*-}"
        if [[ ! "$start" =~ ^[0-9]+$ || ! "$end" =~ ^[0-9]+$ ]]; then
            continue
        fi
        while IFS='|' read -r u_contract u_start u_end; do
            if (( start <= u_end && u_start <= end )); then
                echo "FAIL: $contract range $start-$end overlaps with unique-range $u_contract ($u_start-$u_end)"
                violations=$((violations + 1))
            fi
        done < "$unique_tmp"
    done
done < "$tmp_registry"

if (( violations > 0 )); then
    echo
    echo "FAIL: $violations error code registry violation(s) found."
    exit 1
fi

echo "OK: unique ranges do not overlap and all contracts are documented."
