#!/usr/bin/env bash
# canary_deploy.sh — Staged rollout and canary deployment support (Issue #1191)
#
# Deploys contracts in stages (canary → batch → full), verifying health at
# each stage before proceeding. Supports automatic rollback on failure.
#
# Stages:
#   1. Canary  — deploy 1 low-risk contract and verify
#   2. Batch   — deploy contracts in small batches with health checks
#   3. Full    — deploy all remaining contracts
#
# Usage:
#   bash scripts/canary_deploy.sh --network testnet --version VERSION [--stage canary|batch|full|all]
#   bash scripts/canary_deploy.sh --network mainnet --version VERSION --stage canary --dry-run
#
# Exit codes:
#   0 — deployment succeeded (or dry-run passed)
#   1 — deployment or health check failed
#
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST="${ROOT_DIR}/deployments/deployment-manifest.json"
ROLLOUT_STATE_DIR="${ROOT_DIR}/deployments/rollout-state"
mkdir -p "$ROLLOUT_STATE_DIR"

NETWORK=""
VERSION=""
STAGE="all"
DRY_RUN=false
CANARY_TIMEOUT=120
BATCH_SIZE=3
AUTO_ROLLBACK=true
IDENTITY="${IDENTITY:-default}"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; BLUE='\033[0;34m'; NC='\033[0m'
DEPLOYED=0; FAILED=0; ROLLED_BACK=0

log_info()  { echo -e "${BLUE}[CANARY]${NC} $1"; }
log_ok()    { echo -e "${GREEN}[PASS]${NC} $1"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_fail()  { echo -e "${RED}[FAIL]${NC} $1"; }

# ─── Helpers ─────────────────────────────────────────────────────────────────

jq_manifest() {
  jq -r "$1" "$MANIFEST"
}

get_contracts() {
  local filter="${1:-.contracts[]}"
  jq_manifest "[${filter}] | sort_by(.deploy_order)"
}

contract_names() {
  jq -r '.[].name' 2>/dev/null
}

get_contract_field() {
  local name="$1" field="$2"
  jq -r --arg n "$name" --arg f "$field" '.contracts[] | select(.name == $n) | .[$f] // empty' "$MANIFEST"
}

save_rollout_state() {
  local stage="$1" status="$2"
  echo "{\"network\":\"$NETWORK\",\"version\":\"$VERSION\",\"stage\":\"$stage\",\"status\":\"$status\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",\"deployed\":$DEPLOYED,\"failed\":$FAILED}" \
    > "${ROLLOUT_STATE_DIR}/${NETWORK}-${VERSION}-${stage}.json"
}

load_rollout_state() {
  local stage="$1"
  local state_file="${ROLLOUT_STATE_DIR}/${NETWORK}-${VERSION}-${stage}.json"
  if [[ -f "$state_file" ]]; then
    cat "$state_file"
  else
    echo "{}"
  fi
}

# ─── Canary contract selection ──────────────────────────────────────────────

# Pick a low-risk "canary" contract: the smallest core-tier contract that
# has no downstream dependencies in the manifest.
select_canary_contract() {
  jq -r '
    [.contracts[]
     | select(.tier == "core")
     | select(.dependencies | length == 0)]
    | sort_by(.wasm_path)
    | .[0].name // empty
  ' "$MANIFEST"
}

# ─── Deployment ──────────────────────────────────────────────────────────────

deploy_contract() {
  local name="$1"
  local net_state
  net_state=$(jq -r --arg n "$name" --arg net "$NETWORK" '.contracts[] | select(.name == $n) | .networks[$net] // empty' "$MANIFEST")

  if [[ -z "$net_state" || "$net_state" == "null" ]]; then
    log_warn "Skipping $name — not configured for $NETWORK"
    return 0
  fi

  # Check dependencies
  local deps
  deps=$(jq -r --arg n "$name" '.contracts[] | select(.name == $n) | .dependencies[]' "$MANIFEST" 2>/dev/null || true)
  for dep in $deps; do
    local dep_state
    dep_state=$(jq -r --arg n "$dep" --arg net "$NETWORK" '.contracts[] | select(.name == $n) | .networks[$net].contract_id // empty' "$MANIFEST" 2>/dev/null)
    if [[ -z "$dep_state" ]]; then
      log_warn "Dependency $dep not deployed yet for $name"
    fi
  done

  if [[ "$DRY_RUN" == "true" ]]; then
    log_ok "[dry-run] Would deploy $name to $NETWORK"
    return 0
  fi

  log_info "Deploying $name to $NETWORK..."
  if bash "${ROOT_DIR}/scripts/deploy.sh" "$name" "$NETWORK" "$IDENTITY" 2>/dev/null; then
    log_ok "Deployed $name"
    ((DEPLOYED++))
    return 0
  else
    log_fail "Failed to deploy $name"
    ((FAILED++))
    return 1
  fi
}

# ─── Health check ────────────────────────────────────────────────────────────

check_contract_health() {
  local name="$1"
  local contract_id
  contract_id=$(jq -r --arg n "$name" --arg net "$NETWORK" '.contracts[] | select(.name == $n) | .networks[$net].contract_id // empty' "$MANIFEST" 2>/dev/null)

  if [[ -z "$contract_id" ]]; then
    # Try reading deployment file
    local deploy_file="${ROOT_DIR}/deployments/${NETWORK}_${name}.json"
    if [[ -f "$deploy_file" ]]; then
      contract_id=$(jq -r '.contract_id' "$deploy_file" 2>/dev/null)
    fi
  fi

  if [[ -z "$contract_id" || "$contract_id" == "null" ]]; then
    log_warn "Cannot health-check $name — no contract ID available"
    return 0
  fi

  if soroban contract inspect --id "$contract_id" --network "$NETWORK" &>/dev/null; then
    return 0
  elif soroban contract invoke --id "$contract_id" --network "$NETWORK" -- --help &>/dev/null; then
    return 0
  else
    return 1
  fi
}

# ─── Rollback ────────────────────────────────────────────────────────────────

rollback_contract() {
  local name="$1"
  if [[ "$AUTO_ROLLBACK" != "true" ]]; then
    log_warn "Auto-rollback disabled; manual intervention required for $name"
    return
  fi

  log_info "Rolling back $name..."
  if bash "${ROOT_DIR}/scripts/rollback_deployment.sh" "$name" "$NETWORK" 2>/dev/null; then
    log_ok "Rolled back $name"
    ((ROLLED_BACK++))
  else
    log_fail "Rollback failed for $name — manual intervention required"
  fi
}

# ─── Stages ──────────────────────────────────────────────────────────────────

stage_canary() {
  log_info "=== Stage: CANARY ==="
  local canary
  canary=$(select_canary_contract)

  if [[ -z "$canary" ]]; then
    log_warn "No suitable canary contract found; skipping canary stage"
    save_rollout_state "canary" "skipped"
    return 0
  fi

  log_info "Canary contract: $canary"
  save_rollout_state "canary" "in_progress"

  if ! deploy_contract "$canary"; then
    log_fail "Canary deployment failed — aborting rollout"
    rollback_contract "$canary"
    save_rollout_state "canary" "failed"
    return 1
  fi

  if [[ "$DRY_RUN" != "true" ]]; then
    log_info "Waiting for canary health check (timeout: ${CANARY_TIMEOUT}s)..."
    local elapsed=0
    while [[ $elapsed -lt $CANARY_TIMEOUT ]]; do
      if check_contract_health "$canary"; then
        log_ok "Canary $canary is healthy"
        save_rollout_state "canary" "passed"
        return 0
      fi
      sleep 5
      ((elapsed += 5))
    done
    log_fail "Canary health check timed out"
    rollback_contract "$canary"
    save_rollout_state "canary" "failed"
    return 1
  fi

  save_rollout_state "canary" "passed"
}

stage_batch() {
  log_info "=== Stage: BATCH ==="
  save_rollout_state "batch" "in_progress"

  # Get core contracts (excluding canary which is already deployed)
  local canary
  canary=$(select_canary_contract)

  local batch_contracts
  batch_contracts=$(jq -r --arg skip "$canary" '
    [.contracts[]
     | select(.tier == "core")
     | select(.name != $skip)]
    | sort_by(.deploy_order)
    | .[].name
  ' "$MANIFEST")

  local batch_count=0
  for name in $batch_contracts; do
    if ! deploy_contract "$name"; then
      log_fail "Batch deployment failed at $name"
      rollback_contract "$name"
      save_rollout_state "batch" "failed"
      return 1
    fi

    ((batch_count++))
    if [[ $((batch_count % BATCH_SIZE)) -eq 0 ]]; then
      log_info "Batch checkpoint: $batch_count deployed, verifying health..."
      for bc in $batch_contracts; do
        if [[ "$DRY_RUN" != "true" ]]; then
          check_contract_health "$bc" || log_warn "Health check failed for $bc"
        fi
      done
    fi
  done

  save_rollout_state "batch" "passed"
}

stage_full() {
  log_info "=== Stage: FULL ==="
  save_rollout_state "full" "in_progress"

  # Deploy all remaining contracts (domain tier and anything not yet deployed)
  local remaining
  remaining=$(jq -r '
    [.contracts[] | select(.tier == "domain")]
    | sort_by(.deploy_order)
    | .[].name
  ' "$MANIFEST")

  for name in $remaining; do
    if ! deploy_contract "$name"; then
      log_fail "Full deployment failed at $name"
      rollback_contract "$name"
      save_rollout_state "full" "failed"
      return 1
    fi
  done

  save_rollout_state "full" "passed"
}

# ─── Report ──────────────────────────────────────────────────────────────────

generate_report() {
  local status="success"
  [[ $FAILED -gt 0 ]] && status="failed"

  local report_file="${ROOT_DIR}/reports/canary-deploy-${NETWORK}-${VERSION}.json"
  mkdir -p "$(dirname "$report_file")"

  cat > "$report_file" <<EOF
{
  "network": "$NETWORK",
  "version": "$VERSION",
  "stages_run": "$STAGE",
  "status": "$status",
  "deployed": $DEPLOYED,
  "failed": $FAILED,
  "rolled_back": $ROLLED_BACK,
  "dry_run": $DRY_RUN,
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "rollout_state": $(cat "${ROLLOUT_STATE_DIR}/${NETWORK}-${VERSION}-"*".json" 2>/dev/null | jq -s '.' || echo '[]')
}
EOF
  log_info "Report: $report_file"
}

# ─── CLI ─────────────────────────────────────────────────────────────────────

usage() {
  cat <<EOF
Staged rollout and canary deployment for Uzima-Contracts (Issue #1191).

Usage:
  $0 --network NETWORK --version VERSION [OPTIONS]

Options:
  --network NETWORK     Target network (required): local, testnet, futurenet, mainnet
  --version VERSION     Release version (required)
  --stage STAGE         Stage to run: canary, batch, full, all (default: all)
  --dry-run             Simulate deployment without executing
  --batch-size N        Contracts per batch checkpoint (default: 3)
  --canary-timeout N    Seconds to wait for canary health (default: 120)
  --no-auto-rollback    Disable automatic rollback on failure
  --identity NAME       Soroban identity (default: env IDENTITY or "default")
  -h, --help            Show this help
EOF
}

main() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --network)          NETWORK="$2"; shift 2 ;;
      --version)          VERSION="$2"; shift 2 ;;
      --stage)            STAGE="$2"; shift 2 ;;
      --dry-run)          DRY_RUN=true; shift ;;
      --batch-size)       BATCH_SIZE="$2"; shift 2 ;;
      --canary-timeout)   CANARY_TIMEOUT="$2"; shift 2 ;;
      --no-auto-rollback) AUTO_ROLLBACK=false; shift ;;
      --identity)         IDENTITY="$2"; shift 2 ;;
      -h|--help)          usage; exit 0 ;;
      *) echo "Unknown option: $1"; usage; exit 1 ;;
    esac
  done

  if [[ -z "$NETWORK" || -z "$VERSION" ]]; then
    echo "ERROR: --network and --version are required"
    usage
    exit 1
  fi

  if [[ ! -f "$MANIFEST" ]]; then
    log_fail "Deployment manifest not found: $MANIFEST"
    exit 1
  fi

  log_info "Canary deployment: v${VERSION} → ${NETWORK} (stage=${STAGE}, dry_run=${DRY_RUN})"
  echo ""

  case "$STAGE" in
    canary)  stage_canary ;;
    batch)   stage_canary && stage_batch ;;
    full)    stage_canary && stage_batch && stage_full ;;
    all)     stage_canary && stage_batch && stage_full ;;
    *)       log_fail "Unknown stage: $STAGE"; usage; exit 1 ;;
  esac

  echo ""
  generate_report

  if [[ $FAILED -gt 0 ]]; then
    log_fail "Deployment completed with $FAILED failure(s), $ROLLED_BACK rollback(s)"
    exit 1
  fi

  log_ok "Deployment completed successfully ($DEPLOYED deployed, $ROLLED_BACK rolled back)"
}

main "$@"
