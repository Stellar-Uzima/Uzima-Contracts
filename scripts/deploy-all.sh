#!/usr/bin/env bash
#
# deploy-all.sh — One-command reproducible deployment for local and testnet.
#
# Usage:
#   ./scripts/deploy-all.sh [network] [--skip-build] [--skip-tests] [--contracts list]
#
# Examples:
#   ./scripts/deploy-all.sh                    # deploy all to local
#   ./scripts/deploy-all.sh testnet            # deploy all to testnet
#   ./scripts/deploy-all.sh local --skip-build # skip build, deploy to local
#   ./scripts/deploy-all.sh testnet --contracts medical_records,identity_registry
#
# Prerequisites:
#   - Rust toolchain installed (via setup.sh)
#   - Soroban CLI v21.7.7 installed
#   - For testnet: identity configured with testnet funds
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# ── Defaults ─────────────────────────────────────────────────────────────────

NETWORK="${1:-local}"
shift 2>/dev/null || true

SKIP_BUILD=false
SKIP_TESTS=false
CONTRACTS_FILTER=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-build)   SKIP_BUILD=true; shift ;;
    --skip-tests)   SKIP_TESTS=true; shift ;;
    --contracts)    CONTRACTS_FILTER="$2"; shift 2 ;;
    *) echo "Unknown option: $1"; exit 1 ;;
  esac
done

# ── Helpers ──────────────────────────────────────────────────────────────────

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log()   { echo -e "${GREEN}[DEPLOY]${NC} $1"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1" >&2; }
step()  { echo -e "${BLUE}[STEP]${NC} $1"; }

# ── Validate network ────────────────────────────────────────────────────────

case "$NETWORK" in
  local)
    RPC_URL="http://localhost:8000/soroban/rpc"
    PASSPHRASE="Standalone Network ; February 2017"
    ;;
  testnet)
    RPC_URL="https://soroban-testnet.stellar.org:443"
    PASSPHRASE="Test SDF Network ; September 2015"
    ;;
  futurenet)
    RPC_URL="https://rpc-futurenet.stellar.org:443"
    PASSPHRASE="Test SDF Future Network ; October 2022"
    ;;
  *)
    error "Unknown network: $NETWORK. Use: local, testnet, futurenet"
    exit 1
    ;;
esac

# ── Preflight checks ───────────────────────────────────────────────────────

step "Preflight checks"

if ! command -v cargo &>/dev/null; then
  error "cargo not found. Run: ./setup.sh"
  exit 1
fi

if ! command -v soroban &>/dev/null; then
  error "soroban not found. Run: ./setup.sh"
  exit 1
fi

if [[ ! -f "$PROJECT_ROOT/Cargo.toml" ]]; then
  error "Not in project root. Run from the repository root."
  exit 1
fi

log "Network: $NETWORK"
log "RPC URL: $RPC_URL"

# ── Configure network ───────────────────────────────────────────────────────

step "Configuring Soroban network"

soroban config network add "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$PASSPHRASE" \
  2>/dev/null || log "Network '$NETWORK' already configured"

# ── Configure identity ──────────────────────────────────────────────────────

step "Configuring identity"

if ! soroban config identity show default &>/dev/null; then
  soroban config identity generate default
  log "Default identity created"
fi

if [[ "$NETWORK" == "testnet" || "$NETWORK" == "futurenet" ]]; then
  ADDRESS=$(soroban config identity address default 2>/dev/null || echo "")
  if [[ -n "$ADDRESS" ]]; then
    log "Deployer address: $ADDRESS"
    log "Ensure this account is funded (friendbot for testnet)"
  fi
fi

# ── Build ────────────────────────────────────────────────────────────────────

if [[ "$SKIP_BUILD" == "false" ]]; then
  step "Building optimized WASM contracts"
  cargo build --release --target wasm32-unknown-unknown \
    --workspace \
    --exclude contract_optimizer \
    --exclude uzima-tests \
    --exclude interoperability_suite
  log "Build complete"
else
  warn "Skipping build (--skip-build)"
fi

# ── Tests ────────────────────────────────────────────────────────────────────

if [[ "$SKIP_TESTS" == "false" ]]; then
  step "Running tests"
  cargo test --all
  log "All tests passed"
else
  warn "Skipping tests (--skip-tests)"
fi

# ── Collect contracts to deploy ─────────────────────────────────────────────

step "Discovering contracts"

if [[ -n "$CONTRACTS_FILTER" ]]; then
  IFS=',' read -ra CONTRACTS <<< "$CONTRACTS_FILTER"
else
  CONTRACTS=()
  for dir in "$PROJECT_ROOT"/contracts/*/; do
    if [[ -d "$dir/src" && -f "$dir/Cargo.toml" ]]; then
      name=$(basename "$dir")
      # Skip contracts that aren't in the workspace or are excluded
      if grep -q "\"contracts/$name\"" "$PROJECT_ROOT/Cargo.toml" 2>/dev/null || \
         ls "$PROJECT_ROOT"/target/wasm32-unknown-unknown/release/"$name".wasm &>/dev/null; then
        CONTRACTS+=("$name")
      fi
    fi
  done
fi

log "Contracts to deploy: ${#CONTRACTS[@]}"

# ── Deploy ──────────────────────────────────────────────────────────────────

step "Deploying contracts to $NETWORK"

DEPLOYED=()
FAILED=()

for contract in "${CONTRACTS[@]}"; do
  wasm_file="$PROJECT_ROOT/target/wasm32-unknown-unknown/release/${contract}.wasm"
  if [[ ! -f "$wasm_file" ]]; then
    warn "WASM not found for $contract, skipping"
    continue
  fi

  echo -n "  Deploying $contract... "
  if "$SCRIPT_DIR/deploy.sh" "$contract" "$NETWORK" default >/dev/null 2>&1; then
    echo -e "${GREEN}OK${NC}"
    DEPLOYED+=("$contract")
  else
    echo -e "${RED}FAILED${NC}"
    FAILED+=("$contract")
  fi
done

# ── Summary ──────────────────────────────────────────────────────────────────

echo ""
echo "══════════════════════════════════════════════════════"
echo " Deployment Summary"
echo "══════════════════════════════════════════════════════"
echo " Network:   $NETWORK"
echo " Deployed:  ${#DEPLOYED[@]}"
echo " Failed:    ${#FAILED[@]}"

if [[ ${#FAILED[@]} -gt 0 ]]; then
  echo " Failed:    ${FAILED[*]}"
fi

echo "══════════════════════════════════════════════════════"

if [[ ${#FAILED[@]} -gt 0 ]]; then
  exit 1
fi

log "Deployment complete!"
