#!/usr/bin/env bash
# dead_code_scan.sh — Repository-wide dead-code and unused-dependency scan
# for Rust and JavaScript packages.
#
# Usage:
#   ./scripts/dead_code_scan.sh [--rust] [--js] [--verbose] [--report FILE]
#
# Options:
#   --rust      Run Rust-only scans (default: both).
#   --js        Run JS-only scans (default: both).
#   --verbose   Show extra output from underlying tools.
#   --report F  Write a plain-text summary report to file F.
#
# Rust checks:
#   1. cargo check with -D dead_code  (dead code lint as error)
#   2. cargo +nightly udeps            (unused dependencies, nightly required)
#      Falls back gracefully if nightly / cargo-udeps is not installed.
#
# JS / Node checks:
#   1. knip  (dead exports, unused files, unused deps — preferred)
#      Falls back to depcheck when knip is not installed.
#   2. depcheck  (unused/missing npm dependencies)
#      Falls back gracefully when neither tool is installed.
#
# Exit codes:
#   0  No dead code or unused dependencies found.
#   1  Issues found, or a scan tool reported an error.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

RUN_RUST=true
RUN_JS=true
VERBOSE=false
REPORT_FILE=""

for arg in "$@"; do
  case "$arg" in
    --rust)    RUN_JS=false ;;
    --js)      RUN_RUST=false ;;
    --verbose) VERBOSE=true ;;
    --report)  ;;  # handled below via shift
    *)
      # handle --report FILE
      if [[ "$arg" == --report=* ]]; then
        REPORT_FILE="${arg#--report=}"
      fi ;;
  esac
done

# Re-parse to pick up --report FILE (two-token form)
args=("$@")
for (( i=0; i<${#args[@]}; i++ )); do
  if [[ "${args[$i]}" == "--report" && $((i+1)) -lt ${#args[@]} ]]; then
    REPORT_FILE="${args[$((i+1))]}"
  fi
done

# ---------------------------------------------------------------------------
# Output helpers
# ---------------------------------------------------------------------------
section()  { echo ""; echo "══════════════════════════════════════════════"; echo "  $1"; echo "══════════════════════════════════════════════"; }
info()     { echo "  ℹ  $1"; }
ok()       { echo "  ✓  $1"; }
warn()     { echo "  ⚠  $1"; }
fail_msg() { echo "  ✗  $1"; }

RUST_ISSUES=0
JS_ISSUES=0

# Capture output for optional report file
REPORT_LINES=()
rline() { REPORT_LINES+=("$1"); }

# ---------------------------------------------------------------------------
# RUST SCANS
# ---------------------------------------------------------------------------
if $RUN_RUST; then
  section "Rust Dead-Code & Unused-Dependency Scan"
  cd "$PROJECT_ROOT"

  # ── 1. cargo check with dead_code lint ──────────────────────────────────
  info "Running: cargo check --workspace (dead_code lint)"
  rline "[RUST] cargo check --workspace (dead_code)"

  # Build a RUSTFLAGS that promotes the dead_code lint to deny
  DEAD_CODE_FLAGS="-D dead_code"

  set +e
  if $VERBOSE; then
    RUSTFLAGS="$DEAD_CODE_FLAGS" cargo check --workspace --message-format short 2>&1
    rust_dc_exit=$?
  else
    rust_dc_out=$(RUSTFLAGS="$DEAD_CODE_FLAGS" cargo check --workspace --message-format short 2>&1)
    rust_dc_exit=$?
  fi
  set -e

  if [[ $rust_dc_exit -eq 0 ]]; then
    ok "No dead_code violations found in workspace crates."
    rline "  PASS: no dead_code violations"
  else
    if ! $VERBOSE; then
      # Show only lines mentioning dead_code / unused
      echo "$rust_dc_out" | grep -E "(dead_code|unused|warning\[|error\[)" | head -60 || true
    fi
    warn "dead_code lint found issues (exit $rust_dc_exit)."
    warn "Run 'RUSTFLAGS=\"-D dead_code\" cargo check --workspace' for full output."
    rline "  WARN: dead_code lint reported issues (exit $rust_dc_exit)"
    RUST_ISSUES=$((RUST_ISSUES + 1))
  fi

  # ── 2. cargo-udeps (unused Cargo dependencies) ──────────────────────────
  info "Checking for unused Cargo dependencies (cargo-udeps)"
  rline "[RUST] cargo udeps"

  if ! command -v cargo-udeps &>/dev/null; then
    # Try installing it if cargo is available and we have nightly
    if rustup toolchain list 2>/dev/null | grep -q "nightly"; then
      info "cargo-udeps not found; attempting install..."
      if cargo install cargo-udeps --locked --quiet 2>/dev/null; then
        UDEPS_AVAILABLE=true
      else
        UDEPS_AVAILABLE=false
        warn "Could not install cargo-udeps — skipping unused-dep check."
        warn "Install manually: cargo install cargo-udeps --locked"
        rline "  SKIP: cargo-udeps not available"
      fi
    else
      UDEPS_AVAILABLE=false
      warn "cargo-udeps requires nightly toolchain — skipping."
      warn "Install: rustup toolchain add nightly && cargo install cargo-udeps --locked"
      rline "  SKIP: nightly toolchain not available for cargo-udeps"
    fi
  else
    UDEPS_AVAILABLE=true
  fi

  if ${UDEPS_AVAILABLE:-false}; then
    set +e
    if $VERBOSE; then
      cargo +nightly udeps --workspace 2>&1
      udeps_exit=$?
    else
      udeps_out=$(cargo +nightly udeps --workspace 2>&1)
      udeps_exit=$?
    fi
    set -e

    if [[ $udeps_exit -eq 0 ]]; then
      ok "No unused Cargo dependencies found."
      rline "  PASS: no unused Cargo dependencies"
    else
      if ! $VERBOSE; then
        echo "$udeps_out" | grep -v "^$" | tail -40 || true
      fi
      fail_msg "cargo-udeps found unused dependencies (exit $udeps_exit)."
      fail_msg "Remove unused [dependencies] entries from the relevant Cargo.toml files."
      rline "  FAIL: cargo-udeps found unused dependencies"
      RUST_ISSUES=$((RUST_ISSUES + 1))
    fi
  fi

  # ── 3. Pattern-based scan: pub items never referenced ───────────────────
  info "Scanning for unused pub items (heuristic grep)"
  rline "[RUST] heuristic unused-pub scan"

  # Look for pub fn / pub struct / pub enum defined in contracts/
  # that are never referenced outside their own file.
  UNUSED_PUB=()
  while IFS= read -r lib_rs; do
    rel="${lib_rs#"$PROJECT_ROOT/"}"
    # Skip test files
    [[ "$lib_rs" == *"/test"* ]] && continue
    # Collect pub function names from this file
    while IFS= read -r sym; do
      [[ -z "$sym" ]] && continue
      # Count references across the entire repo source
      ref_count=$(grep -rn --include="*.rs" "\b${sym}\b" "$PROJECT_ROOT/contracts" \
                    "$PROJECT_ROOT/libs" "$PROJECT_ROOT/tests" 2>/dev/null | wc -l)
      # Defined once; if total references == 1 it is only the definition
      if [[ "$ref_count" -le 1 ]]; then
        UNUSED_PUB+=("$rel: $sym")
      fi
    done < <(grep -oP '(?<=pub fn )\w+' "$lib_rs" 2>/dev/null || true)
  done < <(find "$PROJECT_ROOT/contracts" "$PROJECT_ROOT/libs" \
             -name "lib.rs" -path "*/src/lib.rs" 2>/dev/null)

  if [[ ${#UNUSED_PUB[@]} -eq 0 ]]; then
    ok "Heuristic scan found no obviously unused public functions."
    rline "  PASS: heuristic unused-pub scan clean"
  else
    warn "${#UNUSED_PUB[@]} potentially unused public function(s) found:"
    for item in "${UNUSED_PUB[@]}"; do
      echo "    - $item"
      rline "    - $item"
    done
    warn "Review the above — they may be contract entry points (expected) or dead code."
    rline "  WARN: ${#UNUSED_PUB[@]} potentially unused public functions"
    # This is a warning only; contract entry points will appear here.
  fi
fi

# ---------------------------------------------------------------------------
# JS / NODE SCANS
# ---------------------------------------------------------------------------
if $RUN_JS; then
  section "JavaScript / Node Unused-Dependency Scan"
  cd "$PROJECT_ROOT"

  PKG_JSON="$PROJECT_ROOT/package.json"
  if [[ ! -f "$PKG_JSON" ]]; then
    warn "No package.json found at project root — skipping JS scan."
  else
    # ── 1. Try knip (preferred — finds dead exports + unused deps) ──────────
    info "Looking for knip..."
    rline "[JS] knip / depcheck scan"

    if command -v knip &>/dev/null || npx --yes knip --version &>/dev/null 2>&1; then
      info "Running knip..."
      set +e
      if $VERBOSE; then
        npx knip 2>&1
        knip_exit=$?
      else
        knip_out=$(npx knip 2>&1)
        knip_exit=$?
      fi
      set -e

      if [[ $knip_exit -eq 0 ]]; then
        ok "knip: no unused files, exports, or dependencies found."
        rline "  PASS: knip found no issues"
      else
        if ! $VERBOSE; then echo "$knip_out" | head -60; fi
        fail_msg "knip found unused JS/TS code or dependencies (exit $knip_exit)."
        rline "  FAIL: knip found issues (exit $knip_exit)"
        JS_ISSUES=$((JS_ISSUES + 1))
      fi
    # ── 2. Fall back to depcheck ────────────────────────────────────────────
    elif command -v depcheck &>/dev/null || npm list -g depcheck &>/dev/null 2>&1; then
      info "knip not found — using depcheck as fallback..."
      set +e
      if $VERBOSE; then
        depcheck --skip-missing 2>&1
        dc_exit=$?
      else
        dc_out=$(depcheck --skip-missing 2>&1)
        dc_exit=$?
      fi
      set -e

      if [[ $dc_exit -eq 0 ]]; then
        ok "depcheck: no unused dependencies found."
        rline "  PASS: depcheck clean"
      else
        if ! $VERBOSE; then echo "$dc_out"; fi
        fail_msg "depcheck found unused dependencies (exit $dc_exit)."
        fail_msg "Remove or justify unused entries in package.json."
        rline "  FAIL: depcheck found unused deps"
        JS_ISSUES=$((JS_ISSUES + 1))
      fi
    else
      warn "Neither knip nor depcheck is installed — skipping JS unused-dep check."
      warn "Install: npm install -g knip   or   npm install -g depcheck"
      rline "  SKIP: no JS analysis tool installed"
    fi

    # ── 3. Check for obviously unused scripts (defined but never called) ───
    info "Checking for unreferenced npm scripts..."
    SCRIPT_NAMES=$(node -e "
      const p = require('./package.json');
      const s = p.scripts || {};
      console.log(Object.keys(s).join('\n'));
    " 2>/dev/null || true)

    if [[ -n "$SCRIPT_NAMES" ]]; then
      UNREFERENCED=()
      while IFS= read -r script; do
        [[ -z "$script" ]] && continue
        # Skip lifecycle scripts that npm calls automatically
        case "$script" in
          prepare|prepublish|postinstall|preinstall|test|start|build) continue ;;
        esac
        # Check if the script name is referenced in any workflow, Makefile, or doc
        ref=$(grep -rn "\"$script\"\|npm run $script\|yarn $script" \
              "$PROJECT_ROOT/.github" "$PROJECT_ROOT/makefile" \
              "$PROJECT_ROOT/README.md" 2>/dev/null | wc -l || echo 0)
        if [[ "$ref" -eq 0 ]]; then
          UNREFERENCED+=("$script")
        fi
      done <<< "$SCRIPT_NAMES"

      if [[ ${#UNREFERENCED[@]} -gt 0 ]]; then
        warn "${#UNREFERENCED[@]} npm script(s) appear unreferenced in CI/Makefile/README:"
        for s in "${UNREFERENCED[@]}"; do
          echo "    - $s"
          rline "    - $s"
        done
        warn "Consider documenting or removing them."
        rline "  WARN: ${#UNREFERENCED[@]} unreferenced npm scripts"
      else
        ok "All npm scripts appear referenced."
        rline "  PASS: npm scripts referenced"
      fi
    fi
  fi
fi

# ---------------------------------------------------------------------------
# Write optional report file
# ---------------------------------------------------------------------------
if [[ -n "$REPORT_FILE" ]]; then
  {
    echo "Dead-Code & Unused-Dependency Scan Report"
    echo "Generated: $(date -u '+%Y-%m-%dT%H:%M:%SZ')"
    echo "Project  : $PROJECT_ROOT"
    echo ""
    for line in "${REPORT_LINES[@]}"; do
      echo "$line"
    done
  } > "$REPORT_FILE"
  echo ""
  info "Report written to: $REPORT_FILE"
fi

# ---------------------------------------------------------------------------
# Final summary
# ---------------------------------------------------------------------------
section "Final Summary"
TOTAL_ISSUES=$((RUST_ISSUES + JS_ISSUES))

if $RUN_RUST; then echo "  Rust issues : $RUST_ISSUES"; fi
if $RUN_JS;   then echo "  JS issues   : $JS_ISSUES";   fi
echo ""

if [[ $TOTAL_ISSUES -gt 0 ]]; then
  fail_msg "$TOTAL_ISSUES scan(s) reported issues."
  echo ""
  echo "  Remediation tips:"
  echo "    Rust dead code  : Remove or annotate with #[allow(dead_code)]"
  echo "    Rust unused deps: Remove from Cargo.toml or add a justification comment"
  echo "    JS unused deps  : Remove from package.json or add to peerDependencies"
  echo ""
  exit 1
fi

ok "All scans passed — no dead code or unused dependencies detected."
echo ""
exit 0
