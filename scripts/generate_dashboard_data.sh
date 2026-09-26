#!/usr/bin/env bash
#
# generate_dashboard_data.sh — Regenerate the committed dashboard data files.
#
# Usage:
#   ./scripts/generate_dashboard_data.sh
#
# Writes:
#   dashboard/contract_inventory.json  (via scripts/contract_inventory.sh)
#   dashboard/risk_matrix.json         (via scripts/risk_scoring_engine.py)
#
# Run this whenever you add, remove or modify a contract, then commit the
# result. scripts/check_dashboard_data.py fails CI when the committed files no
# longer match what the generators produce.
#
# The risk matrix scores include a git-history component, so it must be
# generated from a full clone. A shallow clone produces different history
# scores, which is why the CI job checks out with fetch-depth: 0.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

INVENTORY="dashboard/contract_inventory.json"
RISK_MATRIX="dashboard/risk_matrix.json"

echo "Generating ${INVENTORY}..."
./scripts/contract_inventory.sh "$INVENTORY"

echo "Generating ${RISK_MATRIX}..."
python3 ./scripts/risk_scoring_engine.py --out "$RISK_MATRIX"

echo "Dashboard data regenerated."
