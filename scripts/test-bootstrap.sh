#!/bin/bash
# scripts/test-bootstrap.sh - Dev environment smoke test
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

PASS=0
FAIL=0

assert() {
    local name="$1"; shift
    if "$@" &>/dev/null; then
        echo -e "${GREEN}  PASS${NC} $name"; ((PASS++))
    else
        echo -e "${RED}  FAIL${NC} $name"; ((FAIL++))
    fi
}

echo "=== Dev Environment Bootstrap Smoke Test ==="
echo ""

echo "[Rust toolchain]"
assert "rustc installed" command -v rustc
assert "cargo installed" command -v cargo
assert "wasm32-unknown-unknown target" rustup target list --installed | grep -q wasm32-unknown-unknown
assert "rustfmt component" rustup component list --installed | grep -q rustfmt
assert "clippy component" rustup component list --installed | grep -q clippy

echo ""
echo "[Soroban CLI]"
assert "soroban installed" command -v soroban

echo ""
echo "[Docker]"
assert "docker installed" command -v docker
if command -v docker &>/dev/null; then
    assert "docker daemon running" docker info
fi

echo ""
echo "[Docker Compose]"
assert "docker-compose installed" command -v docker-compose || command -v "docker compose" &>/dev/null

echo ""
echo "[Project structure]"
assert "Cargo.toml exists" test -f Cargo.toml
assert "contracts/ directory exists" test -d contracts
assert "scripts/ directory exists" test -d scripts
assert "tests/ directory exists" test -d tests
assert "setup.sh exists" test -f setup.sh

echo ""
echo "[Build check]"
assert "cargo check passes" cargo check --all-targets

echo ""
echo "==============================="
echo "Results: ${PASS} passed, ${FAIL} failed"
if [ "$FAIL" -gt 0 ]; then
    echo -e "${RED}Some checks failed!${NC}"
    exit 1
fi
echo -e "${GREEN}All checks passed!${NC}"