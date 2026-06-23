#!/usr/bin/env bash
# Measure storage and CPU cost reduction per call for Uzima contracts.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "🧪 Running Soroban Contract Budget Measurement..."
echo "-----------------------------------------------"

# Run tests with --nocapture to print CPU and storage budget measurements
cargo test --manifest-path contracts/medical_records/Cargo.toml -- --nocapture 2>&1 | grep -iE 'cpu|budget|storage|instructions|opt|savings' || true
cargo test --package cross_chain_bridge -- --nocapture 2>&1 | grep -iE 'cpu|budget|storage|instructions|opt|savings' || true
cargo test --package governor -- --nocapture 2>&1 | grep -iE 'cpu|budget|storage|instructions|opt|savings' || true

echo "-----------------------------------------------"
echo "✅ Measurement complete."
