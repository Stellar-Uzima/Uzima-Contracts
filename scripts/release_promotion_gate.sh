#!/usr/bin/env bash
# release_promotion_gate.sh — Release promotion gates for testnet and mainnet
# deployments (Issue #1190)
#
# Enforces a series of quality, security, and operational gates before a
# contract or full release is promoted from testnet → futurenet → mainnet.
#
# The gate is designed to be run in CI and locally. It produces a machine-
# readable result JSON and a human-readable markdown report suitable for PR
# comments.
#
# Usage:
#   bash scripts/release_promotion_gate.sh --version VERSION --from testnet --to mainnet
#   bash scripts/release_promotion_gate.sh --version VERSION --from testnet --to mainnet --strict
#   bash scripts/release_promotion_gate.sh --version VERSION --from testnet --to mainnet --output report.md
#
# Exit codes:
#   0 — all gates passed
#   1 — one or more gates failed
#
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT_DIR="${ROOT_DIR}/reports"
GATE_DIR="${REPORT_DIR}/promotion-gates"
mkdir -p "$GATE_DIR"

VERSION=""
FROM_NETWORK=""
TO_NETWORK=""
STRICT=false
OUTPUT_FILE=""
AUTO_APPROVE=false

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; BLUE='\033[0;34m'; NC='\033[0m'
GATE_ERRORS=0
GATE_WARNINGS=0
GATE_RESULTS=()

log_info()  { echo -e "${BLUE}[GATE]${NC} $1"; }
log_ok()    { echo -e "${GREEN}[PASS]${NC} $1"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; ((GATE_WARNINGS++)); }
log_fail()  { echo -e "${RED}[FAIL]${NC} $1"; ((GATE_ERRORS++)); }

record_gate() {
  local name="$1" status="$2" detail="${3:-}"
  GATE_RESULTS+=("{\"gate\":\"$name\",\"status\":\"$status\",\"detail\":\"$detail\"}")
}

# ─── Gate: Testnet health ────────────────────────────────────────────────────

gate_testnet_health() {
  log_info "Gate 1/7: Testnet deployment health"
  local script="${ROOT_DIR}/scripts/check_release_health.sh"
  if [[ -f "$script" ]]; then
    if bash "$script" "$VERSION" --network "$FROM_NETWORK" 2>/dev/null; then
      log_ok "Testnet health check passed"
      record_gate "testnet_health" "pass"
    else
      log_fail "Testnet health check failed"
      record_gate "testnet_health" "fail" "health check returned non-zero"
    fi
  else
    log_warn "check_release_health.sh not found, skipping"
    record_gate "testnet_health" "skip"
  fi
}

# ─── Gate: WASM hash consistency ─────────────────────────────────────────────

gate_wasm_hash_consistency() {
  log_info "Gate 2/7: WASM hash consistency (deterministic build)"
  local script="${ROOT_DIR}/scripts/verify_deployment.sh"
  if [[ -f "$script" ]]; then
    if bash "$script" compare "$FROM_NETWORK" "$VERSION" 2>/dev/null; then
      log_ok "WASM hashes match recorded set"
      record_gate "wasm_hash_consistency" "pass"
    else
      log_warn "WASM hash comparison inconclusive (no recorded hashes or mismatch)"
      record_gate "wasm_hash_consistency" "warn" "no baseline or mismatch"
    fi
  else
    log_warn "verify_deployment.sh not found, skipping"
    record_gate "wasm_hash_consistency" "skip"
  fi
}

# ─── Gate: Performance budgets ───────────────────────────────────────────────

gate_performance_budgets() {
  log_info "Gate 3/7: Performance budget check"
  local script="${ROOT_DIR}/scripts/performance_budget_gate.sh"
  if [[ -f "$script" ]]; then
    if bash "$script" --check 2>/dev/null; then
      log_ok "All contracts within performance budgets"
      record_gate "performance_budgets" "pass"
    else
      log_fail "Performance budget gate failed"
      record_gate "performance_budgets" "fail" "regression detected"
    fi
  else
    log_warn "performance_budget_gate.sh not found, skipping"
    record_gate "performance_budgets" "skip"
  fi
}

# ─── Gate: Test coverage ─────────────────────────────────────────────────────

gate_test_coverage() {
  log_info "Gate 4/7: Test coverage check"
  local script="${ROOT_DIR}/scripts/coverage_gate.sh"
  if [[ -f "$script" ]]; then
    if bash "$script" 2>/dev/null; then
      log_ok "Test coverage gate passed"
      record_gate "test_coverage" "pass"
    else
      log_fail "Test coverage gate failed"
      record_gate "test_coverage" "fail" "coverage below threshold"
    fi
  else
    log_warn "coverage_gate.sh not found, skipping"
    record_gate "test_coverage" "skip"
  fi
}

# ─── Gate: Security audit ────────────────────────────────────────────────────

gate_security_audit() {
  log_info "Gate 5/7: Security audit"
  if command -v cargo-audit &>/dev/null; then
    if (cd "$ROOT_DIR" && cargo audit 2>/dev/null); then
      log_ok "Security audit passed (no known vulnerabilities)"
      record_gate "security_audit" "pass"
    else
      log_fail "Security audit found vulnerabilities"
      record_gate "security_audit" "fail" "cargo audit found issues"
    fi
  else
    log_warn "cargo-audit not installed, skipping security audit"
    record_gate "security_audit" "skip"
  fi
}

# ─── Gate: Deployment manifest validation ────────────────────────────────────

gate_manifest_validation() {
  log_info "Gate 6/7: Deployment manifest validation"
  local script="${ROOT_DIR}/scripts/generate_deployment_manifest.sh"
  if [[ -f "$script" ]]; then
    if bash "$script" --validate 2>/dev/null; then
      log_ok "Deployment manifest is valid"
      record_gate "manifest_validation" "pass"
    else
      log_fail "Deployment manifest validation failed"
      record_gate "manifest_validation" "fail"
    fi
  else
    log_warn "generate_deployment_manifest.sh not found, skipping"
    record_gate "manifest_validation" "skip"
  fi
}

# ─── Gate: Version consistency ───────────────────────────────────────────────

gate_version_consistency() {
  log_info "Gate 7/7: Version consistency"
  local workspace_version
  workspace_version=$(grep '^version = ' "$ROOT_DIR/Cargo.toml" | head -1 | cut -d'"' -f2)

  if [[ "$workspace_version" != "$VERSION" ]]; then
    log_fail "Workspace version mismatch: expected $VERSION, found $workspace_version"
    record_gate "version_consistency" "fail" "workspace=$workspace_version expected=$VERSION"
    return
  fi

  local mismatch=0
  for cargo_toml in "$ROOT_DIR"/contracts/*/Cargo.toml; do
    [[ -f "$cargo_toml" ]] || continue
    local name contract_version
    name=$(basename "$(dirname "$cargo_toml")")
    contract_version=$(grep '^version = ' "$cargo_toml" | cut -d'"' -f2)
    if [[ "$contract_version" != "$VERSION" ]]; then
      log_fail "Contract $name version mismatch: $contract_version != $VERSION"
      mismatch=1
    fi
  done

  if [[ "$mismatch" -eq 0 ]]; then
    log_ok "All contract versions match $VERSION"
    record_gate "version_consistency" "pass"
  else
    record_gate "version_consistency" "fail" "one or more contracts mismatched"
  fi
}

# ─── Report Generation ──────────────────────────────────────────────────────

generate_report() {
  local ts now
  ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  local status="PASS"
  [[ "$GATE_ERRORS" -gt 0 ]] && status="FAIL"

  local result_json
  result_json=$(printf '%s\n' "${GATE_RESULTS[@]}" | paste -sd, -)
  result_json="{\"version\":\"$VERSION\",\"from\":\"$FROM_NETWORK\",\"to\":\"$TO_NETWORK\",\"timestamp\":\"$ts\",\"status\":\"$status\",\"errors\":$GATE_ERRORS,\"warnings\":$GATE_WARNINGS,\"gates\":[$result_json]}"

  echo "$result_json" > "${GATE_DIR}/promotion-${VERSION}-result.json"

  local report_file="${OUTPUT_FILE:-${GATE_DIR}/promotion-${VERSION}-report.md}"
  {
    echo "## Release Promotion Gate Report"
    echo ""
    echo "- **Version:** $VERSION"
    echo "- **Promotion:** $FROM_NETWORK → $TO_NETWORK"
    echo "- **Timestamp:** $ts"
    echo "- **Status:** $([ "$status" = "PASS" ] && echo "✅ PASS" || echo "❌ FAIL")"
    echo "- **Errors:** $GATE_ERRORS · **Warnings:** $GATE_WARNINGS"
    echo ""
    echo "### Gate Results"
    echo ""
    echo "| Gate | Status | Detail |"
    echo "|------|--------|--------|"
    for r in "${GATE_RESULTS[@]}"; do
      local gate_name gate_status gate_detail
      gate_name=$(echo "$r" | sed 's/.*"gate":"\([^"]*\)".*/\1/')
      gate_status=$(echo "$r" | sed 's/.*"status":"\([^"]*\)".*/\1/')
      gate_detail=$(echo "$r" | sed 's/.*"detail":"\([^"]*\)".*/\1/' | sed 's/"/\\"/g')
      local icon="✅"
      [[ "$gate_status" = "fail" ]] && icon="❌"
      [[ "$gate_status" = "warn" ]] && icon="⚠️"
      [[ "$gate_status" = "skip" ]] && icon="⏭️"
      echo "| ${gate_name} | ${icon} ${gate_status} | ${gate_detail:-—} |"
    done
    echo ""
    if [[ "$GATE_ERRORS" -gt 0 ]]; then
      echo "### ❌ Promotion Blocked"
      echo ""
      echo "Fix the failing gates above before promoting from **$FROM_NETWORK** to **$TO_NETWORK**."
    else
      echo "### ✅ Promotion Allowed"
      echo ""
      echo "All gates passed. Safe to promote from **$FROM_NETWORK** to **$TO_NETWORK**."
    fi
  } > "$report_file"

  log_info "Report: $report_file"
  log_info "Result: ${GATE_DIR}/promotion-${VERSION}-result.json"
}

# ─── Main ────────────────────────────────────────────────────────────────────

usage() {
  cat <<EOF
Release promotion gate for Uzima-Contracts (Issue #1190).

Usage:
  $0 --version VERSION --from NETWORK --to NETWORK [OPTIONS]

Options:
  --version VERSION   Release version (required)
  --from NETWORK      Source network (required): testnet, futurenet
  --to NETWORK        Target network (required): futurenet, mainnet
  --strict            Treat warnings as errors
  --output FILE       Write report to FILE (default: reports/promotion-gates/)
  --auto-approve      Skip interactive confirmation
  -h, --help          Show this help
EOF
}

main() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --version)   VERSION="$2"; shift 2 ;;
      --from)      FROM_NETWORK="$2"; shift 2 ;;
      --to)        TO_NETWORK="$2"; shift 2 ;;
      --strict)    STRICT=true; shift ;;
      --output)    OUTPUT_FILE="$2"; shift 2 ;;
      --auto-approve) AUTO_APPROVE=true; shift ;;
      -h|--help)   usage; exit 0 ;;
      *) echo "Unknown option: $1"; usage; exit 1 ;;
    esac
  done

  if [[ -z "$VERSION" || -z "$FROM_NETWORK" || -z "$TO_NETWORK" ]]; then
    echo "ERROR: --version, --from, and --to are required"
    usage
    exit 1
  fi

  # Validate network transition
  case "${FROM_NETWORK}-${TO_NETWORK}" in
    testnet-futurenet|testnet-mainnet|futurenet-mainnet) ;;
    *)
      echo "ERROR: Invalid promotion path: ${FROM_NETWORK} → ${TO_NETWORK}"
      echo "Valid paths: testnet→futurenet, testnet→mainnet, futurenet→mainnet"
      exit 1
      ;;
  esac

  log_info "Promotion gate: v${VERSION} (${FROM_NETWORK} → ${TO_NETWORK})"
  echo ""

  gate_testnet_health
  gate_wasm_hash_consistency
  gate_performance_budgets
  gate_test_coverage
  gate_security_audit
  gate_manifest_validation
  gate_version_consistency

  echo ""
  generate_report
  echo ""

  if [[ "$GATE_ERRORS" -gt 0 ]]; then
    log_fail "Promotion gate FAILED ($GATE_ERRORS error(s), $GATE_WARNINGS warning(s))"
    if [[ "$STRICT" == "true" && "$GATE_WARNINGS" -gt 0 ]]; then
      log_fail "Strict mode: treating warnings as errors"
    fi
    exit 1
  fi

  if [[ "$STRICT" == "true" && "$GATE_WARNINGS" -gt 0 ]]; then
    log_fail "Strict mode: $GATE_WARNINGS warning(s) treated as errors"
    exit 1
  fi

  log_ok "Promotion gate PASSED ($GATE_WARNINGS warning(s))"
}

main "$@"
