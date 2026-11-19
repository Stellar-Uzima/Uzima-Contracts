#!/usr/bin/env bash
# deploy_clinical_trial.sh - Build, deploy, initialize and verify the
# clinical_trial contract (issue #1964).
#
# Usage:
#   ./scripts/deploy_clinical_trial.sh [network] [--identity <name>] [--dry-run]
#
#   network      local (default), testnet, futurenet or mainnet
#   --identity   Soroban identity used as deployer/admin (default: default)
#   --dry-run    Build + simulate only; print the exact deploy/invoke/verify
#                commands; make no on-chain writes.
#
# Examples:
#   ./scripts/deploy_clinical_trial.sh local --dry-run
#   ./scripts/deploy_clinical_trial.sh testnet
#   ./scripts/deploy_clinical_trial.sh mainnet --identity prod-deployer

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$SCRIPT_DIR/logger.sh"

CONTRACT_NAME="clinical_trial"
WASM_PATH="$ROOT_DIR/target/wasm32-unknown-unknown/release/$CONTRACT_NAME.wasm"
VERIFY_SCRIPT="$SCRIPT_DIR/verify_deployment.sh"

usage() {
  cat <<EOF
Usage: $(basename "$0") [network] [--identity <name>] [--dry-run] [-h|--help]
Networks: local, testnet, futurenet, mainnet
EOF
}

usage_exit() { usage; exit "${1:-0}"; }

NETWORK="${1:-local}"
if [[ "$NETWORK" == "-h" || "$NETWORK" == "--help" ]]; then usage_exit 0; fi
shift || true

IDENTITY="default"
DRY_RUN="false"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --identity)
      IDENTITY="${2:-}"
      [[ -n "$IDENTITY" ]] || { log "ERROR" "--identity needs a value"; exit 1; }
      shift 2
      ;;
    --dry-run) DRY_RUN="true"; shift ;;
    -h|--help) usage_exit 0 ;;
    *) log "ERROR" "Unknown option: $1"; usage; exit 1 ;;
  esac
done

case "$NETWORK" in
  local|testnet|futurenet|mainnet) ;;
  *) log "ERROR" "Unknown network: $NETWORK"; usage; exit 1 ;;
esac

case "$NETWORK" in
  local)     RPC_URL="http://localhost:8000/soroban/rpc"; PASSPHRASE="Standalone Network ; February 2017" ;;
  testnet)   RPC_URL="https://soroban-testnet.stellar.org"; PASSPHRASE="Test SDF Network ; September 2015" ;;
  futurenet) RPC_URL="https://rpc-futurenet.stellar.org"; PASSPHRASE="Test SDF Future Network ; October 2022" ;;
  mainnet)   RPC_URL="https://soroban-mainnet.stellar.org"; PASSPHRASE="Public Global Stellar Network ; September 2015" ;;
esac

log "INFO" "Deploying $CONTRACT_NAME to $NETWORK (identity=$IDENTITY, dry_run=$DRY_RUN)"

# Prerequisites — fail clearly instead of silently continuing.
command -v cargo >/dev/null 2>&1 || { log "ERROR" "cargo not found. Install Rust: https://rustup.rs"; exit 1; }
command -v soroban >/dev/null 2>&1 || { log "ERROR" "soroban CLI not found. Install: cargo install --locked soroban-cli"; exit 1; }
[[ -d "$ROOT_DIR/contracts/$CONTRACT_NAME" ]] || { log "ERROR" "Missing contracts/$CONTRACT_NAME"; exit 1; }

# Configure the network (idempotent).
soroban network add "$NETWORK" --rpc-url "$RPC_URL" --network-passphrase "$PASSPHRASE" 2>/dev/null || true

# Resolve or create the deployer identity.
if ! soroban config identity address "$IDENTITY" >/dev/null 2>&1; then
  if [[ "$DRY_RUN" == "true" ]]; then
    log "WARN" "Identity '$IDENTITY' not found; skipping generation in dry-run mode"
  else
    log "INFO" "Generating missing identity '$IDENTITY'"
    soroban config identity generate "$IDENTITY"
  fi
fi
ADMIN_ADDRESS="$(soroban config identity address "$IDENTITY" 2>/dev/null || printf 'ADMIN_ADDRESS_FROM_IDENTITY_%s' "$IDENTITY")"

# 1. Build — real failure, no `|| true`.
log "INFO" "Building $CONTRACT_NAME (wasm32-unknown-unknown, release)..."
(cd "$ROOT_DIR" && cargo build --release --target wasm32-unknown-unknown -p "$CONTRACT_NAME")
[[ -f "$WASM_PATH" ]] || { log "ERROR" "Build produced no WASM at $WASM_PATH"; exit 1; }
log "INFO" "WASM ready: $WASM_PATH ($(wc -c < "$WASM_PATH") bytes)"

# 2. Simulate deploy (also the safe path for --dry-run).
log "INFO" "Simulating deploy to $NETWORK..."
soroban contract deploy --wasm "$WASM_PATH" --source "$IDENTITY" --network "$NETWORK" --dry-run

if [[ "$DRY_RUN" == "true" ]]; then
  log "INFO" "DRY RUN complete — no deploy/invoke/verification executed."
  log "INFO" "Would run: soroban contract deploy --wasm $WASM_PATH --source $IDENTITY --network $NETWORK"
  log "INFO" "Would run: soroban contract invoke --id <CONTRACT_ID> --source $IDENTITY --network $NETWORK -- initialize --admin $ADMIN_ADDRESS"
  log "INFO" "Would run: $VERIFY_SCRIPT <CONTRACT_ID> $NETWORK $IDENTITY $CONTRACT_NAME"
  exit 0
fi

# Mainnet guard.
if [[ "$NETWORK" == "mainnet" ]]; then
  log "WARN" "MAINNET deployment spends real XLM and cannot be undone."
  read -r -p "Type CONFIRM to proceed: " reply
  [[ "$reply" == "CONFIRM" ]] || { log "ERROR" "Aborted by user"; exit 1; }
fi

# 3. Deploy.
log "INFO" "Deploying..."
CONTRACT_ID="$(soroban contract deploy --wasm "$WASM_PATH" --source "$IDENTITY" --network "$NETWORK")"
[[ -n "$CONTRACT_ID" ]] || { log "ERROR" "Deploy returned no contract id"; exit 1; }
log "INFO" "Deployed contract id: $CONTRACT_ID"

# 4. Initialize — clinical_trial::initialize(env, admin) needs admin auth.
log "INFO" "Initializing with admin $ADMIN_ADDRESS..."
soroban contract invoke --id "$CONTRACT_ID" --source "$IDENTITY" --network "$NETWORK" -- initialize --admin "$ADMIN_ADDRESS"

# 5. Verify via the repository's standard verifier.
log "INFO" "Verifying deployment..."
"$VERIFY_SCRIPT" "$CONTRACT_ID" "$NETWORK" "$IDENTITY" "$CONTRACT_NAME"

# 6. Persist a deployment record.
mkdir -p "$ROOT_DIR/deployments"
cat > "$ROOT_DIR/deployments/${NETWORK}_${CONTRACT_NAME}.json" <<EOF
{
  "contract_name": "$CONTRACT_NAME",
  "contract_id": "$CONTRACT_ID",
  "network": "$NETWORK",
  "identity": "$IDENTITY",
  "admin": "$ADMIN_ADDRESS",
  "deployed_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "wasm": "target/wasm32-unknown-unknown/release/$CONTRACT_NAME.wasm",
  "verified": true
}
EOF
log "INFO" "Deployment record: deployments/${NETWORK}_${CONTRACT_NAME}.json"
log "INFO" "Deployment complete: $CONTRACT_ID on $NETWORK"
