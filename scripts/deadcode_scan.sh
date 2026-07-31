#!/usr/bin/env bash
# deadcode_scan.sh
#
# Repository-wide dead-code and unused-dependency scan for Rust and JS packages.
# Runs cargo udeps (if available) for Rust unused deps, npm depcheck for JS,
# and provides a unified report.
#
# Usage:
#   ./scripts/deadcode_scan.sh [--json] [--rust-only] [--js-only]
#
# Exit codes:
#   0 — no issues found
#   1 — dead code or unused dependencies detected

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
JSON_OUTPUT=false
RUST_ONLY=false
JS_ONLY=false
REPORT_DIR="$ROOT_DIR/.deadcode-reports"

for arg in "$@"; do
  case "$arg" in
    --json)      JSON_OUTPUT=true ;;
    --rust-only) RUST_ONLY=true ;;
    --js-only)   JS_ONLY=true ;;
    *)           echo "Unknown option: $arg"; exit 1 ;;
  esac
done

mkdir -p "$REPORT_DIR"
TIMESTAMP="$(date +%Y%m%d_%H%M%S)"
VIOLATIONS=0

echo "=== Uzima Dead-Code & Unused Dependency Scan ==="
echo "Timestamp: $TIMESTAMP"
echo

# ---------------------------------------------------------------------------
# Rust: cargo udeps (unused dependencies)
# ---------------------------------------------------------------------------
if ! $JS_ONLY; then
  echo "--- Rust: Unused Dependency Scan ---"
  RUST_REPORT="$REPORT_DIR/rust-unused-deps-$TIMESTAMP.txt"

  if command -v cargo-udeps &>/dev/null; then
    echo "Running cargo udeps..."
    if cargo udeps --workspace 2>&1 | tee "$RUST_REPORT"; then
      echo "Rust: no unused dependencies found."
    else
      echo "Rust: potential unused dependencies detected."
      VIOLATIONS=$((VIOLATIONS + 1))
    fi
  else
    echo "cargo-udeps not installed. Falling back to cargo tree analysis..."
    echo "Install with: cargo install cargo-udeps"
    echo
    # Fallback: check for crates in Cargo.toml that aren't used in source
    echo "Checking workspace Cargo.toml dependencies against source usage..."
    while IFS= read -r dep_line; do
      dep="$(echo "$dep_line" | sed 's/ =.*//;s/"//g;s/[[:space:]]//g')"
      [[ -z "$dep" ]] && continue
      # Skip path dependencies and workspace references
      if echo "$dep_line" | grep -q 'path\s*='; then
        continue
      fi
      count="$(grep -r "use $dep" "$ROOT_DIR/contracts" --include="*.rs" 2>/dev/null | wc -l)"
      if (( count == 0 )); then
        echo "  WARN: dependency '$dep' in workspace Cargo.toml not found in any contract source"
        echo "  WARN: dependency '$dep' in workspace Cargo.toml not found in any contract source" >> "$RUST_REPORT"
      fi
    done < <(awk '/^\[workspace\.dependencies\]/,/^\[/' "$ROOT_DIR/Cargo.toml" | grep -E '^[a-z_]' | head -20)

    if [[ -f "$RUST_REPORT" ]] && [[ -s "$RUST_REPORT" ]]; then
      VIOLATIONS=$((VIOLATIONS + 1))
    fi
  fi

  echo
  echo "--- Rust: Dead Code Warnings (clippy) ---"
  CLIPPY_REPORT="$REPORT_DIR/rust-deadcode-clippy-$TIMESTAMP.txt"

  # Run clippy and capture warnings about dead_code
  if cargo clippy --workspace --message-format=short 2>&1 | grep -i 'dead_code\|unused' | tee "$CLIPPY_REPORT"; then
    echo "Clippy: dead code warnings found."
    VIOLATIONS=$((VIOLATIONS + 1))
  else
    echo "Clippy: no dead code warnings."
  fi
  echo
fi

# ---------------------------------------------------------------------------
# JS: npm depcheck (unused dependencies)
# ---------------------------------------------------------------------------
if ! $RUST_ONLY; then
  echo "--- JavaScript: Unused Dependency Scan ---"
  JS_REPORT="$REPORT_DIR/js-unused-deps-$TIMESTAMP.txt"

  if command -v npx &>/dev/null && [[ -f "$ROOT_DIR/package.json" ]]; then
    echo "Running npm depcheck..."
    cd "$ROOT_DIR"
    if npx depcheck 2>&1 | tee "$JS_REPORT"; then
      echo "JS: no unused dependencies found."
    else
      unused_count=$(grep -c 'unused' "$JS_REPORT" 2>/dev/null || echo 0)
      if (( unused_count > 0 )); then
        echo "JS: unused dependencies detected."
        VIOLATIONS=$((VIOLATIONS + 1))
      else
        echo "JS: depcheck completed (check report for details)."
      fi
    fi
  else
    echo "npx not available or package.json not found. Skipping JS scan."
  fi
  echo
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo "=== Scan Summary ==="
echo "  Reports saved to: $REPORT_DIR"
echo "  Total violations: $VIOLATIONS"
echo

if (( VIOLATIONS > 0 )); then
  echo "FAIL: $VIOLATIONS issue(s) found."
  echo "Review the reports in $REPORT_DIR for details."
  exit 1
fi

echo "OK: no dead code or unused dependency issues found."