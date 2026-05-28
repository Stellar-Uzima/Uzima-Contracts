#!/usr/bin/env bash
# quickstart_dev.sh — Deploy core Uzima contracts to a local network with test data.
#
# Usage: ./scripts/quickstart_dev.sh [--skip-build]
#
# Prerequisites:
#   - soroban CLI installed (v21.7.7+)
#   - Rust + wasm32-unknown-unknown target
#   - Local Stellar network running (make start-local)

set -euo pipefail

NETWORK="local"
RPC_URL="http://localhost:8000/soroban/rpc"
PASSPHRASE="Standalone Network ; February 2017"
SKIP_BUILD=false

for arg in "$@"; do
  [[ "$arg" == "--skip-build" ]] && SKIP_BUILD=true
done

log() { echo "[quickstart] $*"; }
die() { echo "[quickstart] ERROR: $*" >&2; exit 1; }

# ── Prerequisites ────────────────────────────────────────────────────────────

command -v soroban >/dev/null 2>&1 || die "soroban CLI not found. Install: cargo install --locked soroban-cli"
command -v cargo   >/dev/null 2>&1 || die "cargo not found. Install Rust: https://rustup.rs"

# ── Network ──────────────────────────────────────────────────────────────────

log "Configuring local network..."
soroban config network add "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$PASSPHRASE" 2>/dev/null || true

# ── Identities ───────────────────────────────────────────────────────────────

log "Generating test identities..."
for id in dev-admin dev-doctor dev-patient; do
  soroban config identity generate "$id" 2>/dev/null || true
done

ADMIN=$(soroban config identity address dev-admin)
DOCTOR=$(soroban config identity address dev-doctor)
PATIENT=$(soroban config identity address dev-patient)

log "  admin:   $ADMIN"
log "  doctor:  $DOCTOR"
log "  patient: $PATIENT"

# ── Build ────────────────────────────────────────────────────────────────────

if [[ "$SKIP_BUILD" == "false" ]]; then
  log "Building contracts (optimized)..."
  cargo build --release --target wasm32-unknown-unknown \
    -p medical_records \
    -p patient_consent_management \
    -p healthcare_payment \
    -p fhir_integration \
    2>&1 | tail -5
fi

WASM_DIR="target/wasm32-unknown-unknown/release"

# ── Deploy helper ─────────────────────────────────────────────────────────────

deploy_contract() {
  local name="$1"
  local wasm="$WASM_DIR/${name}.wasm"
  [[ -f "$wasm" ]] || { log "  SKIP $name (wasm not found)"; return; }
  local id
  id=$(soroban contract deploy \
    --wasm "$wasm" \
    --source dev-admin \
    --network "$NETWORK")
  echo "$id"
}

# ── Deploy contracts ──────────────────────────────────────────────────────────

log "Deploying medical_records..."
MR_ID=$(deploy_contract "medical_records")
log "  medical_records: $MR_ID"

log "Deploying patient_consent_management..."
PCM_ID=$(deploy_contract "patient_consent_management")
log "  patient_consent_management: $PCM_ID"

log "Deploying fhir_integration..."
FHIR_ID=$(deploy_contract "fhir_integration")
log "  fhir_integration: $FHIR_ID"

# ── Initialize contracts ──────────────────────────────────────────────────────

log "Initializing medical_records..."
soroban contract invoke --id "$MR_ID" --source dev-admin --network "$NETWORK" \
  -- initialize --admin "$ADMIN" 2>/dev/null || log "  (already initialized)"

log "Initializing patient_consent_management..."
soroban contract invoke --id "$PCM_ID" --source dev-admin --network "$NETWORK" \
  -- initialize --admin "$ADMIN" 2>/dev/null || log "  (already initialized)"

# ── Seed test data ────────────────────────────────────────────────────────────

log "Seeding test data..."

# Register users
soroban contract invoke --id "$MR_ID" --source dev-admin --network "$NETWORK" \
  -- register_user --user "$DOCTOR" --role Doctor 2>/dev/null || true
soroban contract invoke --id "$MR_ID" --source dev-admin --network "$NETWORK" \
  -- register_user --user "$PATIENT" --role Patient 2>/dev/null || true

# Grant consent
soroban contract invoke --id "$PCM_ID" --source dev-patient --network "$NETWORK" \
  -- grant_consent --patient "$PATIENT" --provider "$DOCTOR" 2>/dev/null || true

log ""
log "✅ Local dev environment ready!"
log ""
log "Contract IDs:"
log "  medical_records:            $MR_ID"
log "  patient_consent_management: $PCM_ID"
log "  fhir_integration:           $FHIR_ID"
log ""
log "Test identities:"
log "  admin:   $ADMIN"
log "  doctor:  $DOCTOR"
log "  patient: $PATIENT"
log ""
log "Example:"
log "  soroban contract invoke --id $MR_ID --source dev-doctor --network local \\"
log "    -- write_record --patient $PATIENT --encrypted_data 'QmTest' --category General --is_confidential false"
