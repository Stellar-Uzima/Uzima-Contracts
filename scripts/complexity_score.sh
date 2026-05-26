#!/usr/bin/env bash
# Generate contract complexity scores for the developer dashboard.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

CONTRACTS_PATH="${1:-contracts}"
OUTPUT="${2:-dashboard/data/complexity_report.json}"
TRENDS="${3:-dashboard/data/complexity_trends.json}"

echo "Analyzing contract complexity in ${CONTRACTS_PATH}..."
cargo run --quiet -p contract_optimizer -- complexity \
  --contracts-path "$CONTRACTS_PATH" \
  --output "$OUTPUT" \
  --trends "$TRENDS"

echo "Done. Open dashboard/index.html and load Complexity section."
