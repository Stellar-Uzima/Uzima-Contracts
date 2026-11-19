#!/usr/bin/env bash
# Regression test for issue #1963: dependency_health_check.sh must consume real
# `cargo audit --json` CVE data and fail on high/critical advisories.
# Self-contained: feeds JSON fixtures via CARGO_AUDIT_JSON (no cargo build).
set -uo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECK="$ROOT_DIR/scripts/dependency_health_check.sh"
FIXTURES="$ROOT_DIR/tests/fixtures/dependency_health"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
PASS=0
FAIL=0

ok() { echo "  ok   - $1"; PASS=$((PASS + 1)); }
nope() { echo "  FAIL - $1"; FAIL=$((FAIL + 1)); }
expect_exit() {
  if [[ "$2" -eq "$1" ]]; then ok "$3 (exit $2)"; else nope "$3 (want $1, got $2)"; fi
}
expect_contains() {
  if grep -qF -- "$2" "$1"; then ok "$3"; else nope "$3 (missing: $2)"; fi
}

echo "dependency_health_check.sh (CVE gating)"

printf '%s\n' '{"vulnerabilities":{"list":[]}}' > "$TMP/clean.json"
CARGO_AUDIT_JSON="$TMP/clean.json" bash "$CHECK" "$TMP/clean-report.json" >/dev/null 2>&1
expect_exit 0 $? "passes with no advisories"
expect_contains "$TMP/clean-report.json" '"high_critical_advisories": 0' "reports zero high/critical"

CARGO_AUDIT_JSON="$FIXTURES/audit_high.json" bash "$CHECK" "$TMP/high-report.json" >/dev/null 2>&1
expect_exit 1 $? "fails on a high-severity advisory"
expect_contains "$TMP/high-report.json" 'RUSTSEC-2026-0001' "report includes the advisory id"
expect_contains "$TMP/high-report.json" '"severity": "high"' "report includes the high issue"

printf '%s\n' '{"vulnerabilities":{"list":[{"package":{"name":"mild-crate"},"advisory":{"id":"RUSTSEC-2026-0003","severity":"medium","title":"Low impact"}}]}}' > "$TMP/medium.json"
CARGO_AUDIT_JSON="$TMP/medium.json" bash "$CHECK" "$TMP/medium-report.json" >/dev/null 2>&1
expect_exit 0 $? "does not fail on medium-only advisory"
expect_contains "$TMP/medium-report.json" '"advisories_total": 1' "still counts the advisory"

echo
echo "passed: $PASS, failed: $FAIL"
[[ "$FAIL" -eq 0 ]]
