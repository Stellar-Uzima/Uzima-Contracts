#!/usr/bin/env bash
# ============================================================================
# generate_budgets.sh
# Generates per-contract resource budget JSON files by inspecting contract
# WASM builds.  Must be run after `cargo build --release --target wasm32-unknown-unknown`.
#
# Usage:
#   ./scripts/generate_budgets.sh
#
# Output:
#   resource-budgets/<contract>.json  for every workspace member
# ============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
WASM_DIR="$ROOT_DIR/target/wasm32-unknown-unknown/release"
BUDGET_DIR="$ROOT_DIR/resource-budgets"

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

info() { echo -e "${GREEN}[INFO]${NC} $1"; }
fail() { echo -e "${RED}[FAIL]${NC} $1"; }

mkdir -p "$BUDGET_DIR"

# Soroban constants
MAX_READ_ENTRIES=2000
MAX_WRITE_ENTRIES=1000
MAX_INSTRUCTIONS=100_000_000
MAX_MEM_BYTES=50 * 1024 * 1024  # 50 MiB

Count=0
for manifest in "$ROOT_DIR"/contracts/*/Cargo.toml; do
  [[ -f "$manifest" ]] || continue
  ContractName=$(basename "$(dirname "$manifest")")
  WASM_FILE="$WASM_DIR/${ContractName}.wasm"

  if [[ ! -f "$WASM_FILE" ]]; then
    # Skip contracts not in workspace
    continue
  fi

  WASM_SIZE=$(stat -f%z "$WASM_FILE" 2>/dev/null || stat -c%s "$WASM_FILE" 2>/dev/null || echo 0)

  # Derive rough budget estimates from WASM size (conservative heuristics)
  # 1 byte wasm ≈ 2-4 CPU instructions on average for Soroban
  EstimatedInstructions=$((WASM_SIZE * 3))
  if [[ $EstimatedInstructions -gt $MAX_INSTRUCTIONS ]]; then
    EstimatedInstructions=$MAX_INSTRUCTIONS
  fi

  # Typical contract reads ≤ 200 entries per call
  EstimatedReads=200
  # Typical writes ≤ 50 per call
  EstimatedWrites=50

  cat > "$BUDGET_DIR/${ContractName}.json" <<EOF
{
  "contract": "$ContractName",
  "wasm_size_bytes": $WASM_SIZE,
  "wasm_size_human": "$(numfmt --to=iec $WASM_SIZE 2>/dev/null || echo "${WASM_SIZE}B")",
  "soroban_limits": {
    "max_read_entries": $MAX_READ_ENTRIES,
    "max_write_entries": $MAX_WRITE_ENTRIES,
    "max_instructions": $MAX_INSTRUCTIONS,
    "max_memory_bytes": $((50 * 1024 * 1024))
  },
  "estimated_per_call": {
    "instructions": $EstimatedInstructions,
    "read_entries": $EstimatedReads,
    "write_entries": $EstimatedWrites,
    "memory_bytes": $((WASM_SIZE * 4))
  },
  "budget_utilization_pct": {
    "instructions": $(echo "scale=2; $EstimatedInstructions * 100 / $MAX_INSTRUCTIONS" | bc 2>/dev/null || echo "0"),
    "read_entries": $(echo "scale=2; $EstimatedReads * 100 / $MAX_READ_ENTRIES" | bc 2>/dev/null || echo "10"),
    "write_entries": $(echo "scale=2; $EstimatedWrites * 100 / $MAX_WRITE_ENTRIES" | bc 2>/dev/null || echo "5")
  },
  "generated_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

  Count=$((Count + 1))
  info "$ContractName → resource-budgets/${ContractName}.json (wasm: ${WASM_SIZE}B)"
done

echo ""
info "Generated $Count budget files in resource-budgets/"
