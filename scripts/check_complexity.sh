#!/usr/bin/env bash
# Enforce contract complexity thresholds in CI.
# Runs scoring, checks warn/fail thresholds, writes PR comment file.
# Exit codes: 0 = pass, 0 (warnings) = warn-only, 1 = fail.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

CONTRACTS_PATH="${1:-contracts}"
REPORT_OUTPUT="${2:-dashboard/data/complexity_report.json}"
COMMENT_OUTPUT="${3:-reports/complexity_pr_comment.txt}"

echo "Checking contract complexity in ${CONTRACTS_PATH}..."

cargo run --quiet -p contract_optimizer --features cli -- check-complexity \
  --contracts-path "$CONTRACTS_PATH" \
  --output "$REPORT_OUTPUT" \
  --comment-output "$COMMENT_OUTPUT"
