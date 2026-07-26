#!/usr/bin/env bash
#
# Tests for scripts/performance_budget_gate.sh (Issue #1086).
#
# Self-contained: builds synthetic .wasm fixtures and a synthetic storage
# summary in a temp dir, so it needs no cargo build and runs in seconds.
#
# Usage: bash tests/performance_budget_gate_test.sh
#
set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GATE="$ROOT_DIR/scripts/performance_budget_gate.sh"

PASS=0
FAIL=0

ok()   { echo "  ok   - $1"; PASS=$((PASS + 1)); }
nope() { echo "  FAIL - $1"; FAIL=$((FAIL + 1)); }

expect_exit() {
  local want="$1" got="$2" what="$3"
  if [[ "$got" -eq "$want" ]]; then ok "$what (exit $got)"; else nope "$what (want exit $want, got $got)"; fi
}

expect_contains() {
  local file="$1" needle="$2" what="$3"
  if grep -qF -- "$needle" "$file"; then ok "$what"; else nope "$what (missing: $needle)"; fi
}

# Build a sandbox with the given budget JSON, then create fixtures.
setup() {
  SANDBOX="$(mktemp -d)"
  mkdir -p "$SANDBOX/scripts" "$SANDBOX/wasm" "$SANDBOX/reports"
  cp "$GATE" "$SANDBOX/scripts/"
  cat > "$SANDBOX/scripts/performance_budgets.json"
}

teardown() { [[ -n "${SANDBOX:-}" ]] && rm -rf "$SANDBOX"; }

mkwasm() { head -c "$2" /dev/zero > "$SANDBOX/wasm/$1.wasm"; }

run_gate() {
  ( cd "$SANDBOX" && WASM_DIR="$SANDBOX/wasm" REPORT_DIR="$SANDBOX/reports" \
      BUDGET_FILE="$SANDBOX/scripts/performance_budgets.json" \
      TREND_FILE="$SANDBOX/.trends.json" \
      bash scripts/performance_budget_gate.sh "$@" >"$SANDBOX/out.txt" 2>&1 )
}

BASE_BUDGET='{
  "schema_version": 1,
  "limits": { "max_wasm_size_bytes": 65536, "warn_wasm_size_bytes": 61440,
    "max_storage_entries": 500, "warn_storage_entries": 100,
    "max_cpu_instructions": 10000000, "warn_cpu_instructions": 5000000 },
  "regression": { "wasm_size_pct": 10, "storage_entries_pct": 15, "cpu_instructions_pct": 15,
    "min_wasm_delta_bytes": 512, "min_cpu_delta_instructions": 50000 },
  "excluded": ["load_testing"],
  "contracts": {
    "steady":      { "tier": "standard", "wasm_size": 20000, "grandfathered_over_size_limit": false },
    "regressor":   { "tier": "standard", "wasm_size": 20000, "grandfathered_over_size_limit": false },
    "grandfather": { "tier": "critical", "wasm_size": 92000, "grandfathered_over_size_limit": true },
    "crosser":     { "tier": "critical", "wasm_size": 64000, "grandfathered_over_size_limit": false }
  }
}'

echo "performance_budget_gate.sh"

# ── 1. Size dimension: regression, hard-limit crossing, grandfathering ──────
echo "size budgets"
setup <<<"$BASE_BUDGET"
mkwasm steady 20000          # unchanged        -> ok
mkwasm regressor 25000       # +25%             -> fail
mkwasm grandfather 92000     # over limit       -> warn only
mkwasm crosser 66000         # crosses 64 KB    -> fail
mkwasm load_testing 99999    # excluded         -> ignored
run_gate --check
expect_exit 1 $? "fails when a contract regresses beyond budget"
REPORT="$SANDBOX/reports/performance_budget_report.md"
expect_contains "$REPORT" "+25.0%" "reports the regression percentage"
expect_contains "$REPORT" "crossed hard limit" "flags a newly crossed hard limit"
expect_contains "$REPORT" "grandfathered" "grandfathers a pre-existing breach"
if grep -q 'load_testing' "$REPORT"; then nope "excludes contracts on the exclude list"; else ok "excludes contracts on the exclude list"; fi
expect_contains "$REPORT" "cargo bloat" "emits actionable size recommendations"
teardown

# ── 2. All contracts within budget ─────────────────────────────────────────
echo "passing build"
setup <<<"$BASE_BUDGET"
mkwasm steady 20000
mkwasm regressor 20100       # +0.5%, under both tolerance and noise floor
mkwasm grandfather 92000
mkwasm crosser 64000
run_gate --check
expect_exit 0 $? "passes when every contract is within budget"
expect_contains "$SANDBOX/reports/performance_budget_report.md" \
  "within their performance budgets" "reports success"
teardown

# ── 3. Noise floor suppresses trivial absolute changes ─────────────────────
echo "noise floor"
setup <<<'{
  "schema_version": 1,
  "limits": { "max_wasm_size_bytes": 65536, "warn_wasm_size_bytes": 61440,
    "max_storage_entries": 500, "warn_storage_entries": 100,
    "max_cpu_instructions": 10000000, "warn_cpu_instructions": 5000000 },
  "regression": { "wasm_size_pct": 10, "storage_entries_pct": 15, "cpu_instructions_pct": 15,
    "min_wasm_delta_bytes": 512, "min_cpu_delta_instructions": 50000 },
  "excluded": [],
  "contracts": { "tiny": { "tier": "standard", "wasm_size": 1000, "grandfathered_over_size_limit": false } }
}'
mkwasm tiny 1400             # +40% but only +400 B, below the 512 B noise floor
run_gate --check
expect_exit 0 $? "does not fail a large percentage below the absolute noise floor"
teardown

# ── 4. Storage-entry and CPU dimensions ────────────────────────────────────
echo "storage and cpu budgets"
setup <<<'{
  "schema_version": 1,
  "limits": { "max_wasm_size_bytes": 65536, "warn_wasm_size_bytes": 61440,
    "max_storage_entries": 500, "warn_storage_entries": 100,
    "max_cpu_instructions": 10000000, "warn_cpu_instructions": 5000000 },
  "regression": { "wasm_size_pct": 10, "storage_entries_pct": 15, "cpu_instructions_pct": 15,
    "min_wasm_delta_bytes": 512, "min_cpu_delta_instructions": 50000 },
  "excluded": [],
  "contracts": {
    "entry_creep": { "tier": "standard", "wasm_size": 20000, "storage_entries": 40, "cpu_instructions": 1000000 },
    "cpu_creep":   { "tier": "standard", "wasm_size": 20000, "storage_entries": 40, "cpu_instructions": 1000000 }
  }
}'
mkwasm entry_creep 20000
mkwasm cpu_creep 20000
cat > "$SANDBOX/reports/storage_summary.json" <<'EOF'
[
 {"contract":"entry_creep","estimated_entries":60,"max_cpu":1000000},
 {"contract":"cpu_creep","estimated_entries":40,"max_cpu":1400000}
]
EOF
run_gate --check
expect_exit 1 $? "fails on storage-entry and CPU regressions"
REPORT="$SANDBOX/reports/performance_budget_report.md"
expect_contains "$REPORT" "+50.0%" "reports the storage-entry regression"
expect_contains "$REPORT" "+40.0%" "reports the CPU regression"
expect_contains "$REPORT" "temporary storage" "emits storage recommendations"
expect_contains "$REPORT" "paginate" "emits CPU recommendations"
teardown

# ── 5. Graceful degradation without benchmark data ─────────────────────────
echo "missing benchmark data"
setup <<<"$BASE_BUDGET"
mkwasm steady 20000
run_gate --check
expect_exit 0 $? "degrades to size-only when storage_summary.json is absent"
expect_contains "$SANDBOX/reports/performance_budget_report.md" \
  "was not found" "explains that budgets were not fully evaluated"
teardown

# ── 6. WARN_ONLY observation mode ──────────────────────────────────────────
echo "warn-only mode"
setup <<<"$BASE_BUDGET"
mkwasm regressor 25000
( cd "$SANDBOX" && WARN_ONLY=1 WASM_DIR="$SANDBOX/wasm" REPORT_DIR="$SANDBOX/reports" \
    BUDGET_FILE="$SANDBOX/scripts/performance_budgets.json" TREND_FILE="$SANDBOX/.trends.json" \
    bash scripts/performance_budget_gate.sh --check >/dev/null 2>&1 )
expect_exit 0 $? "WARN_ONLY=1 reports without failing the build"
teardown

# ── 7. --update re-records the budget ──────────────────────────────────────
echo "budget update"
setup <<<"$BASE_BUDGET"
mkwasm steady 20000
mkwasm regressor 25000
mkwasm grandfather 92000
mkwasm crosser 64000
run_gate --update
expect_exit 0 $? "--update succeeds"
run_gate --check
expect_exit 0 $? "build passes against the freshly recorded budget"
teardown

# ── 8. New contract over a hard limit is rejected ──────────────────────────
echo "new contracts"
setup <<<"$BASE_BUDGET"
mkwasm steady 20000
mkwasm brand_new 70000       # untracked and already over the 64 KB hard limit
run_gate --check
expect_exit 1 $? "rejects a new contract introduced over a hard limit"
teardown

# ── 9. Trend view ──────────────────────────────────────────────────────────
echo "trend view"
setup <<<"$BASE_BUDGET"
mkwasm steady 20000
run_gate --check
mkwasm steady 24000
run_gate --check
run_gate --trend
expect_exit 0 $? "--trend succeeds"
expect_contains "$SANDBOX/out.txt" "steady" "trend view lists a moved contract"
teardown

echo
echo "passed: $PASS, failed: $FAIL"
[[ "$FAIL" -eq 0 ]]
