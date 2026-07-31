#!/usr/bin/env bash
# enforce_no_std.sh — Verify that all Soroban contract crates declare
# #![no_std] and do not import the Rust standard library.
#
# Usage:
#   ./scripts/enforce_no_std.sh [--fix] [--verbose]
#
# Options:
#   --fix      Auto-prepend #![no_std] to any lib.rs missing it,
#              provided the file does not contain 'use std::' imports.
#   --verbose  Print PASS results in addition to failures.
#
# Opt-out: If a file intentionally uses std (e.g. a fuzz harness), add
# the following comment anywhere in the file:
#   // no_std-exempt: <reason>
#
# Exit codes:
#   0  All contracts are compliant.
#   1  One or more violations found.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

FIX=false
VERBOSE=false
for arg in "$@"; do
  case "$arg" in
    --fix)     FIX=true ;;
    --verbose) VERBOSE=true ;;
    *)
      echo "Unknown option: $arg"
      echo "Usage: $0 [--fix] [--verbose]"
      exit 1 ;;
  esac
done

PASS=0; FAIL=0; EXEMPT=0; FIXED=0
FAILURES=()

echo "=== no_std Compliance Check ==="
mapfile -t LIB_FILES < <(
  find "$PROJECT_ROOT/contracts" "$PROJECT_ROOT/libs" \
    -name "lib.rs" -path "*/src/lib.rs" | sort
)
echo "Checking ${#LIB_FILES[@]} lib.rs files under contracts/ and libs/"
echo ""

for lib_rs in "${LIB_FILES[@]}"; do
  rel="${lib_rs#"$PROJECT_ROOT/"}"

  if grep -q "no_std-exempt" "$lib_rs" 2>/dev/null; then
    EXEMPT=$((EXEMPT + 1))
    $VERBOSE && echo "  [EXEMPT] $rel"
    continue
  fi

  if grep -q "#!\[no_std\]" "$lib_rs"; then
    PASS=$((PASS + 1))
    $VERBOSE && echo "  [PASS]   $rel"
    continue
  fi

  FAIL=$((FAIL + 1))
  FAILURES+=("$rel")

  if $FIX; then
    if grep -q "use std::" "$lib_rs"; then
      echo "  [FAIL]   $rel  (cannot auto-fix: contains 'use std::' imports)"
    else
      TMP=$(mktemp)
      awk '!found && !/^[[:space:]]*$/ && !/^\/\/!/ {
        print "#![no_std]"; print ""; found=1
      } { print }' "$lib_rs" > "$TMP"
      mv "$TMP" "$lib_rs"
      FIXED=$((FIXED + 1))
      echo "  [FIXED]  $rel"
    fi
  else
    echo "  [FAIL]   $rel  (missing #![no_std])"
  fi
done

echo ""
echo "=== Summary ==="
echo "  Checked   : ${#LIB_FILES[@]}"
echo "  Compliant : $PASS"
echo "  Exempt    : $EXEMPT"
$FIX && echo "  Fixed     : $FIXED"
echo "  Failures  : $FAIL"

if [[ $FAIL -gt 0 ]]; then
  echo ""
  echo "Violating files:"
  for f in "${FAILURES[@]}"; do
    echo "  - $f"
  done
  echo ""
  echo "Fix: Add '#![no_std]' as the first non-comment line in each file."
  echo "     Or run: ./scripts/enforce_no_std.sh --fix"
  echo "     To exempt a file: add '// no_std-exempt: <reason>' anywhere in it."
  exit 1
fi

echo ""
echo "All checked files are no_std compliant. ✓"
exit 0
