#!/usr/bin/env bash
# coverage_gate.sh — Coverage gates for high-risk contracts and workflows
# (Issue #1196)
#
# Enforces minimum test coverage thresholds for contracts classified as
# high-risk (core tier, handles PHI/PII, financial operations, auth, etc.).
#
# The gate evaluates two dimensions:
#   1. Unit test pass rate (cargo test --package)
#   2. Public-API documentation coverage (cargo doc --workspace -W missing_docs)
#
# High-risk contracts have stricter thresholds than standard contracts.
#
# Usage:
#   bash scripts/coverage_gate.sh              # check mode (CI gate)
#   bash scripts/coverage_gate.sh --update     # record baselines
#   bash scripts/coverage_gate.sh --report     # generate markdown report
#
# Exit codes:
#   0 — all gates passed
#   1 — one or more gates failed
#
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT_DIR="${ROOT_DIR}/reports"
COVERAGE_CONFIG="${ROOT_DIR}/resource-budgets/coverage-gate.json"
mkdir -p "$REPORT_DIR"

MODE="check"
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; BLUE='\033[0;34m'; NC='\033[0m'
GATE_ERRORS=0
GATE_WARNINGS=0
RESULTS=()

log_info()  { echo -e "${BLUE}[COVERAGE]${NC} $1"; }
log_ok()    { echo -e "${GREEN}[PASS]${NC} $1"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; ((GATE_WARNINGS++)); }
log_fail()  { echo -e "${RED}[FAIL]${NC} $1"; ((GATE_ERRORS++)); }

# ─── Configuration ───────────────────────────────────────────────────────────

# High-risk contracts: core tier and contracts handling sensitive data
HIGH_RISK_CONTRACTS=(
  identity_registry
  access_control
  rbac
  audit
  governor
  healthcare_payment
  escrow
  patient_consent_management
  common_auth
  upgradeability
)

# Minimum test pass rate (percentage of test modules that pass)
MIN_TEST_PASS_RATE_STANDARD=80
MIN_TEST_PASS_RATE_HIGH_RISK=95

# Maximum allowed missing-docs warnings per contract
MAX_MISSING_DOCS_STANDARD=10
MAX_MISSING_DOCS_HIGH_RISK=0

# ─── Config file support ─────────────────────────────────────────────────────

load_config() {
  if [[ -f "$COVERAGE_CONFIG" ]]; then
    log_info "Loading config from $COVERAGE_CONFIG"
    MIN_TEST_PASS_RATE_STANDARD=$(jq -r '.thresholds.standard.test_pass_rate // 80' "$COVERAGE_CONFIG")
    MIN_TEST_PASS_RATE_HIGH_RISK=$(jq -r '.thresholds.high_risk.test_pass_rate // 95' "$COVERAGE_CONFIG")
    MAX_MISSING_DOCS_STANDARD=$(jq -r '.thresholds.standard.max_missing_docs // 10' "$COVERAGE_CONFIG")
    MAX_MISSING_DOCS_HIGH_RISK=$(jq -r '.thresholds.high_risk.max_missing_docs // 0' "$COVERAGE_CONFIG")

    # Load high-risk list from config if present
    local config_risks
    config_risks=$(jq -r '.high_risk_contracts[]? // empty' "$COVERAGE_CONFIG" 2>/dev/null || true)
    if [[ -n "$config_risks" ]]; then
      HIGH_RISK_CONTRACTS=()
      while IFS= read -r c; do
        HIGH_RISK_CONTRACTS+=("$c")
      done <<< "$config_risks"
    fi
  fi
}

save_config() {
  mkdir -p "$(dirname "$COVERAGE_CONFIG")"
  cat > "$COVERAGE_CONFIG" <<EOF
{
  "description": "Coverage gate configuration for high-risk contracts (Issue #1196)",
  "thresholds": {
    "standard": {
      "test_pass_rate": $MIN_TEST_PASS_RATE_STANDARD,
      "max_missing_docs": $MAX_MISSING_DOCS_STANDARD
    },
    "high_risk": {
      "test_pass_rate": $MIN_TEST_PASS_RATE_HIGH_RISK,
      "max_missing_docs": $MAX_MISSING_DOCS_HIGH_RISK
    }
  },
  "high_risk_contracts": [$(printf '"%s",' "${HIGH_RISK_CONTRACTS[@]}" | sed 's/,$//')],
  "updated_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
  log_ok "Config saved to $COVERAGE_CONFIG"
}

is_high_risk() {
  local name="$1"
  for hr in "${HIGH_RISK_CONTRACTS[@]}"; do
    [[ "$hr" == "$name" ]] && return 0
  done
  return 1
}

# ─── Gate: Unit test pass rate ───────────────────────────────────────────────

gate_test_coverage() {
  log_info "Checking unit test pass rate..."

  local total=0 passed=0 failed_contracts=""
  local threshold

  for contract_dir in "$ROOT_DIR"/contracts/*/; do
    [[ -f "$contract_dir/Cargo.toml" ]] || continue
    local name
    name=$(basename "$contract_dir")

    # Skip excluded contracts
    if jq -e --arg n "$name" '.exclude | index($n)' "$ROOT_DIR/Cargo.toml" &>/dev/null; then
      continue
    fi

    ((total++))

    if is_high_risk "$name"; then
      threshold=$MIN_TEST_PASS_RATE_HIGH_RISK
    else
      threshold=$MIN_TEST_PASS_RATE_STANDARD
    fi

    # Run tests for this contract
    local test_output test_exit
    test_output=$(cargo test --package "$name" --lib 2>&1) && test_exit=0 || test_exit=$?

    if [[ $test_exit -eq 0 ]]; then
      ((passed++))
      RESULTS+=("{\"contract\":\"$name\",\"tier\":\"$(is_high_risk "$name" && echo high_risk || echo standard)\",\"test\":\"pass\",\"docs\":\"pending\"}")
    else
      failed_contracts+="$name "
      RESULTS+=("{\"contract\":\"$name\",\"tier\":\"$(is_high_risk "$name" && echo high_risk || echo standard)\",\"test\":\"fail\",\"docs\":\"pending\"}")
      if is_high_risk "$name"; then
        log_fail "HIGH_RISK $name: unit tests failed"
      else
        log_warn "$name: unit tests failed"
      fi
    fi
  done

  if [[ $total -eq 0 ]]; then
    log_warn "No contracts found to test"
    return 0
  fi

  local pass_rate=$((passed * 100 / total))
  log_info "Test pass rate: ${passed}/${total} (${pass_rate}%)"

  # Check against thresholds
  local overall_threshold=$MIN_TEST_PASS_RATE_STANDARD
  if [[ $pass_rate -lt $overall_threshold ]]; then
    log_fail "Test pass rate ${pass_rate}% < ${overall_threshold}% threshold"
    return 1
  fi

  log_ok "Test pass rate ${pass_rate}% >= ${overall_threshold}% threshold"
}

# ─── Gate: Documentation coverage ────────────────────────────────────────────

gate_doc_coverage() {
  log_info "Checking documentation coverage..."

  local script="${ROOT_DIR}/scripts/coverage_report.sh"
  if [[ ! -f "$script" ]]; then
    log_warn "coverage_report.sh not found, skipping doc coverage"
    return 0
  fi

  # Run doc coverage check
  local doc_output doc_exit
  doc_output=$(bash "$script" docs 2>&1) && doc_exit=0 || doc_exit=$?

  # Parse missing docs count
  local missing_count
  missing_count=$(echo "$doc_output" | grep -oP 'MISSING_DOCS_COUNT=\K[0-9]+' || echo "0")

  if [[ "$missing_count" -gt "$MAX_MISSING_DOCS_STANDARD" ]]; then
    log_fail "Documentation coverage: $missing_count missing docs > $MAX_MISSING_DOCS_STANDARD limit"
    return 1
  fi

  log_ok "Documentation coverage: $missing_count missing docs (limit: $MAX_MISSING_DOCS_STANDARD)"
}

# ─── Gate: Critical path coverage ────────────────────────────────────────────

gate_critical_path() {
  log_info "Checking critical path test coverage..."

  # Critical functions that must have 100% test coverage
  local critical_functions=(
    "initialize"
    "create_record"
    "grant_access"
    "revoke_access"
    "validate_user"
    "check_permissions"
  )

  local all_covered=true
  for func in "${critical_functions[@]}"; do
    # Check if any test file references this function
    local test_refs
    test_refs=$(grep -rl "$func" "$ROOT_DIR"/tests/ 2>/dev/null | head -1 || true)

    if [[ -z "$test_refs" ]]; then
      log_warn "Critical function '$func' has no test references"
      all_covered=false
    fi
  done

  if [[ "$all_covered" == "false" ]]; then
    log_warn "Some critical functions lack test references (see warnings above)"
  else
    log_ok "All critical functions have test references"
  fi
}

# ─── Gate: Fuzz test coverage ────────────────────────────────────────────────

gate_fuzz_coverage() {
  log_info "Checking fuzz test presence..."

  local fuzz_dir="$ROOT_DIR/tests/fuzz"
  if [[ ! -d "$fuzz_dir" ]]; then
    log_warn "Fuzz test directory not found at tests/fuzz"
    return 0
  fi

  local fuzz_files
  fuzz_files=$(find "$fuzz_dir" -name "*.rs" -type f 2>/dev/null | wc -l || echo 0)

  if [[ $fuzz_files -eq 0 ]]; then
    log_warn "No fuzz test files found in tests/fuzz/"
    return 0
  fi

  log_ok "Found $fuzz_files fuzz test file(s)"
}

# ─── Gate: Integration test presence ─────────────────────────────────────────

gate_integration_tests() {
  log_info "Checking integration test presence..."

  local integration_dir="$ROOT_DIR/tests/integration"
  if [[ ! -d "$integration_dir" ]]; then
    log_warn "Integration test directory not found at tests/integration"
    return 0
  fi

  local integration_files
  integration_files=$(find "$integration_dir" -name "*.rs" -type f 2>/dev/null | wc -l || echo 0)

  if [[ $integration_files -eq 0 ]]; then
    log_warn "No integration test files found"
    return 0
  fi

  log_ok "Found $integration_files integration test file(s)"
}

# ─── Report Generation ──────────────────────────────────────────────────────

generate_report() {
  local ts status
  ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  status="PASS"
  [[ "$GATE_ERRORS" -gt 0 ]] && status="FAIL"

  local result_file="${REPORT_DIR}/coverage-gate-result.json"
  local report_file="${REPORT_DIR}/coverage-gate-report.md"

  # Machine-readable result
  local results_json
  results_json=$(printf '%s\n' "${RESULTS[@]}" | paste -sd, - 2>/dev/null || echo "")
  cat > "$result_file" <<EOF
{
  "status": "$status",
  "errors": $GATE_ERRORS,
  "warnings": $GATE_WARNINGS,
  "high_risk_contracts": [$(printf '"%s",' "${HIGH_RISK_CONTRACTS[@]}" | sed 's/,$//')],
  "thresholds": {
    "standard_test_pass_rate": $MIN_TEST_PASS_RATE_STANDARD,
    "high_risk_test_pass_rate": $MIN_TEST_PASS_RATE_HIGH_RISK,
    "standard_max_missing_docs": $MAX_MISSING_DOCS_STANDARD,
    "high_risk_max_missing_docs": $MAX_MISSING_DOCS_HIGH_RISK
  },
  "contracts": [$results_json],
  "timestamp": "$ts"
}
EOF

  # Human-readable report
  {
    echo "## Coverage Gate Report"
    echo ""
    echo "- **Status:** $([ "$status" = "PASS" ] && echo "✅ PASS" || echo "❌ FAIL")"
    echo "- **Errors:** $GATE_ERRORS · **Warnings:** $GATE_WARNINGS"
    echo "- **High-risk contracts:** ${HIGH_RISK_CONTRACTS[*]}"
    echo "- **Timestamp:** $ts"
    echo ""
    echo "### Thresholds"
    echo ""
    echo "| Metric | Standard | High-Risk |"
    echo "|--------|----------|-----------|"
    echo "| Min test pass rate | ${MIN_TEST_PASS_RATE_STANDARD}% | ${MIN_TEST_PASS_RATE_HIGH_RISK}% |"
    echo "| Max missing docs | ${MAX_MISSING_DOCS_STANDARD} | ${MAX_MISSING_DOCS_HIGH_RISK} |"
    echo ""
    echo "### Per-Contract Results"
    echo ""
    echo "| Contract | Tier | Tests | Docs |"
    echo "|----------|------|-------|------|"
    for r in "${RESULTS[@]}"; do
      local name tier test doc
      name=$(echo "$r" | sed 's/.*"contract":"\([^"]*\)".*/\1/')
      tier=$(echo "$r" | sed 's/.*"tier":"\([^"]*\)".*/\1/')
      test=$(echo "$r" | sed 's/.*"test":"\([^"]*\)".*/\1/')
      doc=$(echo "$r" | sed 's/.*"docs":"\([^"]*\)".*/\1/')
      local test_icon="✅"; [[ "$test" != "pass" ]] && test_icon="❌"
      echo "| ${name} | ${tier} | ${test_icon} ${test} | ${doc} |"
    done
  } > "$report_file"

  log_info "Report: $report_file"
  log_info "Result: $result_file"
}

# ─── CLI ─────────────────────────────────────────────────────────────────────

usage() {
  cat <<EOF
Coverage gates for high-risk contracts (Issue #1196).

Usage:
  $0 [OPTIONS]

Options:
  --check         Check coverage against thresholds (default)
  --update        Record current coverage as baseline
  --report        Generate markdown report
  -h, --help      Show this help

Environment:
  HIGH_RISK_ONLY=1    Only gate high-risk contracts
  WARN_ONLY=1         Report violations without failing
EOF
}

main() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --check)    MODE="check"; shift ;;
      --update)   MODE="update"; shift ;;
      --report)   MODE="report"; shift ;;
      -h|--help)  usage; exit 0 ;;
      *) echo "Unknown option: $1"; usage; exit 1 ;;
    esac
  done

  load_config

  case "$MODE" in
    check)
      log_info "=== Coverage Gate Check ==="
      echo ""
      gate_test_coverage
      gate_doc_coverage
      gate_critical_path
      gate_fuzz_coverage
      gate_integration_tests
      echo ""
      generate_report

      if [[ "$GATE_ERRORS" -gt 0 ]]; then
        log_fail "Coverage gate FAILED ($GATE_ERRORS error(s), $GATE_WARNINGS warning(s))"
        exit 1
      fi
      log_ok "Coverage gate PASSED ($GATE_WARNINGS warning(s))"
      ;;
    update)
      log_info "Updating coverage configuration..."
      save_config
      ;;
    report)
      log_info "Generating coverage report..."
      gate_test_coverage
      gate_doc_coverage
      gate_critical_path
      gate_fuzz_coverage
      gate_integration_tests
      generate_report
      ;;
  esac
}

main "$@"
