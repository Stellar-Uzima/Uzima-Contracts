#!/usr/bin/env bash
#
# setup.sh — Reproducible developer environment bootstrap for Uzima Contracts.
#
# This script installs the correct toolchain versions (as pinned in
# rust-toolchain.toml), builds the project, and verifies the environment.
#
# Requirements: bash 4+, curl, git
# Optional: make, shellcheck
#
# Usage:
#   ./setup.sh              # full setup
#   ./setup.sh --skip-build # skip build and test
#

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

SKIP_BUILD=false
[[ "${1:-}" == "--skip-build" ]] && SKIP_BUILD=true

log()   { echo -e "${GREEN}[INFO]${NC} $1"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1" >&2; }
step()  { echo -e "${BLUE}[STEP]${NC} $1"; }

# ── Preflight ────────────────────────────────────────────────────────────────

if [[ ! -f "Cargo.toml" ]]; then
  error "No Cargo.toml found. Run this script from the repository root."
  exit 1
fi

if [[ ! -f "rust-toolchain.toml" ]]; then
  error "No rust-toolchain.toml found. Cannot determine required Rust version."
  exit 1
fi

REQUIRED_RUST=$(grep '^channel' rust-toolchain.toml | sed 's/.*= *"\(.*\)"/\1/')
log "Required Rust version: $REQUIRED_RUST"

# ── Step 1: Install Rust via rustup ─────────────────────────────────────────

step "1/6 Installing Rust toolchain"

if ! command -v rustup &>/dev/null; then
  log "rustup not found. Installing Rust..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain "$REQUIRED_RUST"
  source "$HOME/.cargo/env"
else
  log "rustup found. Ensuring correct toolchain..."
  rustup install "$REQUIRED_RUST"
fi

# Set the pinned toolchain as default for this directory
rustup override set "$REQUIRED_RUST"

INSTALLED_RUST=$(rustc --version | awk '{print $2}')
if [[ "$INSTALLED_RUST" != "$REQUIRED_RUST" ]]; then
  error "Rust version mismatch: expected $REQUIRED_RUST, got $INSTALLED_RUST"
  exit 1
fi
log "Rust $INSTALLED_RUST installed"

# ── Step 2: Add required targets and components ─────────────────────────────

step "2/6 Adding Rust targets and components"

rustup target add wasm32-unknown-unknown
rustup component add rustfmt clippy rust-src
log "Targets and components configured"

# ── Step 3: Install Soroban CLI ─────────────────────────────────────────────

step "3/6 Installing Soroban CLI"

SOROBAN_VERSION="21.7.7"

if command -v soroban &>/dev/null; then
  CURRENT_SOROBAN=$(soroban --version 2>/dev/null | awk '{print $2}' || echo "unknown")
  if [[ "$CURRENT_SOROBAN" == "$SOROBAN_VERSION" ]]; then
    log "Soroban CLI v$SOROBAN_VERSION already installed"
  else
    warn "Soroban CLI v$CURRENT_SOROBAN found (need v$SOROBAN_VERSION). Installing..."
    cargo install --locked --version "$SOROBAN_VERSION" soroban-cli
  fi
else
  log "Installing Soroban CLI v$SOROBAN_VERSION..."
  cargo install --locked --version "$SOROBAN_VERSION" soroban-cli
fi

log "Soroban CLI installed"

# ── Step 4: Configure Soroban identity and networks ─────────────────────────

step "4/6 Configuring Soroban identity and networks"

if ! soroban config identity show default &>/dev/null; then
  soroban config identity generate default
  log "Default identity created"
else
  log "Default identity already exists"
fi

soroban config network add local \
  --rpc-url http://localhost:8000/soroban/rpc \
  --network-passphrase "Standalone Network ; February 2017" \
  2>/dev/null || true

soroban config network add testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015" \
  2>/dev/null || true

soroban config network add futurenet \
  --rpc-url https://rpc-futurenet.stellar.org:443 \
  --network-passphrase "Test SDF Future Network ; October 2022" \
  2>/dev/null || true

log "Networks configured"

# ── Step 5: Make scripts executable ─────────────────────────────────────────

step "5/6 Setting permissions"

chmod +x scripts/*.sh 2>/dev/null || true
log "Scripts are executable"

# ── Step 6: Build and test ──────────────────────────────────────────────────

step "6/6 Building and testing"

if [[ "$SKIP_BUILD" == "true" ]]; then
  warn "Skipping build (--skip-build flag)"
else
  log "Building all contracts..."
  cargo build --all-targets

  log "Running tests..."
  cargo test --all

  log "Running formatting check..."
  cargo fmt --all -- --check || {
    warn "Formatting issues found. Run: cargo fmt --all"
  }

  log "Running clippy..."
  cargo clippy --workspace --all-targets -- -D warnings || {
    warn "Clippy warnings found. Review output above."
  }
fi

# ── Summary ──────────────────────────────────────────────────────────────────

echo ""
echo -e "${GREEN}Setup complete!${NC}"
echo ""
echo "Toolchain:"
echo "  Rust:      $(rustc --version)"
echo "  Soroban:   $(soroban --version 2>/dev/null || echo 'not installed')"
echo "  Targets:   $(rustup target list --installed | tr '\n' ' ')"
echo ""
echo "Quick start:"
echo "  make help          Show all make targets"
echo "  make build         Build all contracts"
echo "  make test          Run all tests"
echo "  make check         Run fmt + clippy + test"
echo "  make start-local   Start local Stellar network"
echo ""
