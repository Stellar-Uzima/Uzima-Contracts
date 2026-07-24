#!/usr/bin/env bash
# check_no_std.sh
#
# Verifies that all workspace-member Soroban contracts include the required
# #![no_std] attribute in their lib.rs.  Optionally enforces #![forbid(alloc)]
# to prevent accidental use of heap allocators.
#
# Usage:
#   ./scripts/check_no_std.sh [--enforce-alloc]
#
# Exit codes:
#   0 — all contracts comply
#   1 — one or more violations found

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
CONTRACTS_DIR="$ROOT_DIR/contracts"
WORKSPACE_TOML="$ROOT_DIR/Cargo.toml"
ENFORCE_ALLOC=false

for arg in "$@"; do
  [[ "$arg" == "--enforce-alloc" ]] && ENFORCE_ALLOC=true
done

# ---------------------------------------------------------------------------
# Determine which directories are excluded from the workspace so we skip them.
# ---------------------------------------------------------------------------
declare -A EXCLUDED=()
in_exclude=false
while IFS= read -r line; do
  trimmed="$(echo "$line" | sed 's/^[[:space:]]*//')"
  if [[ "$trimmed" == "exclude = [" ]]; then
    in_exclude=true
    continue
  fi
  if $in_exclude; then
    if [[ "$trimmed" == "]" ]]; then
      in_exclude=false
      continue
    fi
    dir="$(echo "$trimmed" | sed 's/.*"contracts\///;s/".*//')"
    if [[ -n "$dir" ]]; then
      EXCLUDED["$dir"]=1
    fi
  fi
done < "$WORKSPACE_TOML"

# Also exclude non-contract directories (test harnesses, integration test repos)
EXCLUDED["contract_behavior_fuzzing"]=1
EXCLUDED["governance_integration_tests"]=1
EXCLUDED["storage-snapshot"]=1
EXCLUDED["test-helpers"]=1

# ---------------------------------------------------------------------------
# Scan
# ---------------------------------------------------------------------------
no_std_missing=0
forbid_alloc_missing=0
checked=0
skipped=0

while IFS= read -r -d '' lib_file; do
  contract="$(basename "$(dirname "$(dirname "$lib_file")")")"

  if [[ -v "EXCLUDED[$contract]" ]]; then
    skipped=$((skipped + 1))
    continue
  fi

  checked=$((checked + 1))
  content="$(cat "$lib_file")"

  if ! echo "$content" | grep -q '#!\[no_std\]'; then
    echo "FAIL [no_std]: $lib_file is missing #![no_std]"
    no_std_missing=$((no_std_missing + 1))
  fi

  if $ENFORCE_ALLOC; then
    if ! echo "$content" | grep -q '#!\[forbid(alloc)\]'; then
      echo "FAIL [forbid(alloc)]: $lib_file is missing #![forbid(alloc)]"
      forbid_alloc_missing=$((forbid_alloc_missing + 1))
    fi
  fi
done < <(find "$CONTRACTS_DIR" -maxdepth 3 -name 'lib.rs' -path '*/src/lib.rs' -print0 | sort -z)

# ---------------------------------------------------------------------------
# Report
# ---------------------------------------------------------------------------
echo
echo "no_std compliance scan results:"
echo "  Contracts checked:  ${checked}"
echo "  Contracts skipped:  ${skipped} (excluded from workspace)"
echo "  Missing #![no_std]:  ${no_std_missing}"
if $ENFORCE_ALLOC; then
  echo "  Missing #![forbid(alloc)]: ${forbid_alloc_missing}"
fi
echo

total_violations=$((no_std_missing + forbid_alloc_missing))

if (( total_violations > 0 )); then
  echo "FAIL: ${total_violations} violation(s) found."
  echo
  echo "Fix: Add the following attributes near the top of lib.rs:"
  echo "  #![no_std]"
  if $ENFORCE_ALLOC; then
    echo "  #![forbid(alloc)]"
  fi
  exit 1
fi

echo "OK: all workspace-member contracts have the required attributes."