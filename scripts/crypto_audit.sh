#!/bin/bash

# crypto_audit.sh - Targeted cryptographic/security checks for Uzima contracts
#
# This script is intended for developer workstations/CI (it runs cargo commands).

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_header() { echo -e "\n${BLUE}>>> $1${NC}"; }
print_ok() { echo -e "${GREEN}✓ $1${NC}"; }
print_warn() { echo -e "${YELLOW}⚠ $1${NC}"; }
print_err() { echo -e "${RED}✗ $1${NC}" >&2; }

trap 'print_err "crypto audit failed"' ERR

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

print_header "Format"
cargo fmt -- --check
print_ok "cargo fmt --check"

print_header "Clippy (crypto-related crates)"
cargo clippy -p medical_records -p crypto_registry -p homomorphic_registry -p mpc_manager --all-targets --all-features -- -D warnings
print_ok "cargo clippy"

print_header "Tests (crypto-related crates)"
cargo test -p medical_records -p crypto_registry -p homomorphic_registry -p mpc_manager
print_ok "cargo test"

print_header "Cargo audit (optional)"
if command -v cargo-audit >/dev/null 2>&1; then
  # Advisory DB fetch may require network; tolerate failure in restricted environments.
  if cargo audit; then
    print_ok "cargo audit"
  else
    print_warn "cargo audit failed (network/DB unavailable?)"
  fi
else
  print_warn "cargo-audit not installed"
  print_warn "Install with: cargo install cargo-audit"
fi
