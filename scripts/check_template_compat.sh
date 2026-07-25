#!/usr/bin/env bash
# check_template_compat.sh — Verify that scaffolded contracts are compatible
# with the canonical contracts/contract_template structure.
#
# Usage:
#   ./scripts/check_template_compat.sh [--verbose] [--skip-exclude] [CONTRACT ...]
#
# Options:
#   --verbose       Show PASS results in addition to failures.
#   --skip-exclude  Also check contracts in the workspace exclude list.
#   --list-checks   Print the check catalogue and exit.
#
# If CONTRACT names are given only those are checked; otherwise all
# contracts/ subdirectories are scanned.
#
# Checks performed:
#   C1  src/lib.rs exists
#   C2  Cargo.toml exists
#   C3  #![no_std] declared (or no_std-exempt comment present)
#   C4  #[contract] struct present
#   C5  #[contractimpl] block present
#   C6  initialize function exists
#   C7  require_auth() called
#   C8  Error type (contracterror) defined
#   C9  Event emission (events module or env.events())
#   C10 Cargo.toml uses workspace version/edition
#   C11 soroban-sdk dependency declared
#   C12 crate-type includes "cdylib"
#   C13 No 'use std::' imports in lib.rs
#   C14 No format! macro in lib.rs (requires alloc)
#   C15 README.md exists
#
# Exit codes:
#   0  All checks passed.
#   1  One or more failures.
#   2  Template directory not found.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEMPLATE_DIR="$PROJECT_ROOT/contracts/contract_template"
CONTRACTS_DIR="$PROJECT_ROOT/contracts"
WORKSPACE_TOML="$PROJECT_ROOT/Cargo.toml"

VERBOSE=false
SKIP_EXCLUDE=false
TARGET_CONTRACTS=()

for arg in "$@"; do
  case "$arg" in
    --verbose)      VERBOSE=true ;;
    --skip-exclude) SKIP_EXCLUDE=true ;;
    --list-checks)
      grep -E "^#   C[0-9]" "$0" | sed 's/^# //'
      exit 0 ;;
    --*)
      echo "Unknown option: $arg"
      echo "Usage: $0 [--verbose] [--skip-exclude] [CONTRACT ...]"
      exit 1 ;;
    *)
      TARGET_CONTRACTS+=("$arg") ;;
  esac
done

if [[ ! -d "$TEMPLATE_DIR/src" ]]; then
  echo "ERROR: Template directory not found: $TEMPLATE_DIR"
  exit 2
fi

# ---------------------------------------------------------------------------
# Build excluded-contract set from workspace Cargo.toml
# ---------------------------------------------------------------------------
declare -A EXCLUDED
if [[ -f "$WORKSPACE_TOML" ]] && ! $SKIP_EXCLUDE; then
  in_block=false
  while IFS= read -r line; do
    if echo "$line" | grep -q "^exclude\s*=\s*\["; then
      in_block=true
    fi
    if $in_block; then
      while IFS= read -r entry; do
        name="${entry##contracts/}"
        EXCLUDED["$name"]=1
      done < <(echo "$line" | grep -oP '"contracts/[^"]*"' | tr -d '"' | sed 's|contracts/||')
      if echo "$line" | grep -q "\]"; then
        in_block=false
      fi
    fi
  done < "$WORKSPACE_TOML"
fi

# ---------------------------------------------------------------------------
# Collect contracts to check
# ---------------------------------------------------------------------------
if [[ ${#TARGET_CONTRACTS[@]} -gt 0 ]]; then
  CONTRACTS=("${TARGET_CONTRACTS[@]}")
else
  mapfile -t CONTRACTS < <(
    find "$CONTRACTS_DIR" -maxdepth 1 -mindepth 1 -type d \
      | xargs -I{} basename {} | sort
  )
fi

pass_count=0
fail_count=0
skip_count=0

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
run_check() {
  local id="$1" desc="$2" result="$3" detail="${4:-}"
  if [[ "$result" == "pass" ]]; then
    pass_count=$((pass_count + 1))
    if [[ "$VERBOSE" == "true" ]]; then
      echo "    [PASS] $id $desc"
    fi
  else
    fail_count=$((fail_count + 1))
    echo "    [FAIL] $id $desc${detail:+ — $detail}"
  fi
}

check_contract() {
  local name="$1"
  local dir="$CONTRACTS_DIR/$name"

  # Skip non-Rust entries
  if [[ ! -d "$dir/src" && ! -f "$dir/Cargo.toml" ]]; then
    skip_count=$((skip_count + 1))
    if [[ "$VERBOSE" == "true" ]]; then
      echo "  [SKIP] $name  (no src/ or Cargo.toml)"
    fi
    return
  fi

  # Skip workspace-excluded contracts (unless --skip-exclude)
  if [[ -v "EXCLUDED[$name]" ]]; then
    skip_count=$((skip_count + 1))
    if [[ "$VERBOSE" == "true" ]]; then
      echo "  [SKIP] $name  (workspace exclude list)"
    fi
    return
  fi

  echo ""
  echo "Checking: $name"

  local lib_rs="$dir/src/lib.rs"
  local cargo_toml="$dir/Cargo.toml"
  local errors_rs="$dir/src/errors.rs"
  local events_rs="$dir/src/events.rs"
  local readme="$dir/README.md"

  # C1 — src/lib.rs exists
  if [[ -f "$lib_rs" ]]; then
    run_check "C1" "src/lib.rs exists" "pass"
  else
    run_check "C1" "src/lib.rs exists" "fail" "file not found"
    return
  fi

  # C2 — Cargo.toml exists
  if [[ -f "$cargo_toml" ]]; then
    run_check "C2" "Cargo.toml exists" "pass"
  else
    run_check "C2" "Cargo.toml exists" "fail" "file not found"
  fi

  # C3 — #![no_std]
  if grep -q "#!\[no_std\]" "$lib_rs" || grep -q "no_std-exempt" "$lib_rs"; then
    run_check "C3" "#![no_std] declared" "pass"
  else
    run_check "C3" "#![no_std] declared" "fail" "missing #![no_std]"
  fi

  # C4 — #[contract]
  if grep -q "#\[contract\]" "$lib_rs"; then
    run_check "C4" "#[contract] struct present" "pass"
  else
    run_check "C4" "#[contract] struct present" "fail" "no #[contract] attribute found"
  fi

  # C5 — #[contractimpl]
  if grep -q "#\[contractimpl\]" "$lib_rs"; then
    run_check "C5" "#[contractimpl] block present" "pass"
  else
    run_check "C5" "#[contractimpl] block present" "fail" "no #[contractimpl] attribute found"
  fi

  # C6 — initialize fn
  if grep -q "fn initialize" "$lib_rs"; then
    run_check "C6" "initialize function exists" "pass"
  else
    run_check "C6" "initialize function exists" "fail" "no 'fn initialize' found"
  fi

  # C7 — require_auth
  if grep -q "require_auth" "$lib_rs"; then
    run_check "C7" "require_auth() used" "pass"
  else
    run_check "C7" "require_auth() used" "fail" "no require_auth() — auth may be missing"
  fi

  # C8 — error type
  if [[ -f "$errors_rs" ]] || grep -qr "contracterror" "$dir/src/" 2>/dev/null; then
    run_check "C8" "error type (contracterror) defined" "pass"
  else
    run_check "C8" "error type (contracterror) defined" "fail" \
      "no errors.rs and no #[contracterror] in src/"
  fi

  # C9 — event emission
  if [[ -f "$events_rs" ]] || grep -qr "env\.events()" "$dir/src/" 2>/dev/null; then
    run_check "C9" "event emission present" "pass"
  else
    run_check "C9" "event emission present" "fail" \
      "no events.rs and no env.events() call found"
  fi

  if [[ -f "$cargo_toml" ]]; then
    # C10 — workspace version/edition
    if grep -q "version\.workspace\s*=\s*true" "$cargo_toml" \
       && grep -q "edition\.workspace\s*=\s*true" "$cargo_toml"; then
      run_check "C10" "Cargo.toml uses workspace version/edition" "pass"
    else
      run_check "C10" "Cargo.toml uses workspace version/edition" "fail" \
        "use 'version.workspace = true' and 'edition.workspace = true'"
    fi

    # C11 — soroban-sdk
    if grep -q "soroban-sdk" "$cargo_toml"; then
      run_check "C11" "soroban-sdk dependency declared" "pass"
    else
      run_check "C11" "soroban-sdk dependency declared" "fail" \
        "soroban-sdk missing from [dependencies]"
    fi

    # C12 — cdylib present in crate-type (may also include rlib)
    if grep -q 'crate-type\s*=\s*\[.*"cdylib".*\]' "$cargo_toml"; then
      run_check "C12" 'crate-type includes "cdylib"' "pass"
    else
      run_check "C12" 'crate-type includes "cdylib"' "fail" \
        "cdylib missing from crate-type — contract won't build to WASM"
    fi
  fi

  # C13 — no use std::
  if grep -q "use std::" "$lib_rs"; then
    run_check "C13" "no 'use std::' imports" "fail" \
      "found 'use std::' — use core:: or soroban-sdk equivalents"
  else
    run_check "C13" "no 'use std::' imports" "pass"
  fi

  # C14 — no format!
  if grep -q "format!" "$lib_rs"; then
    run_check "C14" "no format! macro" "fail" \
      "format! requires alloc — use soroban_sdk::String::from_str()"
  else
    run_check "C14" "no format! macro" "pass"
  fi

  # C15 — README.md
  if [[ -f "$readme" ]]; then
    run_check "C15" "README.md exists" "pass"
  else
    run_check "C15" "README.md exists" "fail" "no README.md found"
  fi
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
echo "================================================================"
echo "  Contract Template Compatibility Checker"
echo "  Template : $TEMPLATE_DIR"
echo "================================================================"

for contract in "${CONTRACTS[@]}"; do
  check_contract "$contract"
done

echo ""
echo "================================================================"
echo "  Summary"
echo "================================================================"
echo "  Skipped  : $skip_count"
echo "  Passes   : $pass_count"
echo "  Failures : $fail_count"
echo ""

if [[ $fail_count -gt 0 ]]; then
  echo "  ✗ $fail_count check(s) failed."
  echo ""
  echo "  Guidance:"
  echo "    • Use './scripts/scaffold-contract.sh <name>' for new contracts."
  echo "    • See docs/NO_STD_COMPLIANCE.md for no_std migration help."
  echo "    • Run './scripts/enforce_no_std.sh --fix' to auto-add #![no_std]."
  echo ""
  exit 1
fi

echo "  ✓ All checks passed — contracts are template-compatible."
echo ""
exit 0
