#!/bin/bash

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

usage() {
    cat <<'EOF'
Usage:
  ./scripts/migrate_contract.sh <contract_name> --network <network> --dry-run [--plan <path>]
  ./scripts/migrate_contract.sh <contract_name> --network <network> --i-understand-this-is-live [--identity <name>] [--plan <path>]

Description:
  Reads a planned contract migration from deployments/<network>/<contract_name>/plan.json
  and either:
    - renders a dry-run projection for storage diff, gas, and events
    - executes the live upgrade after an explicit acknowledgement and prompt

Options:
  --network <network>                   Soroban network name (for example: testnet)
  --plan <path>                        Override the default plan path
  --identity <identity>                Soroban identity for live execution
  --dry-run                            Print a projected migration report without submitting
  --i-understand-this-is-live          Required for live execution
EOF
}

require_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        print_error "Required command '$1' is not installed or not on PATH"
        exit 1
    fi
}

parse_args() {
    CONTRACT_NAME=""
    NETWORK=""
    PLAN_PATH=""
    IDENTITY=""
    DRY_RUN=false
    LIVE_ACK=false

    while [[ $# -gt 0 ]]; do
        case "$1" in
            --network)
                NETWORK="${2:-}"
                shift 2
                ;;
            --plan)
                PLAN_PATH="${2:-}"
                shift 2
                ;;
            --identity)
                IDENTITY="${2:-}"
                shift 2
                ;;
            --dry-run)
                DRY_RUN=true
                shift
                ;;
            --i-understand-this-is-live)
                LIVE_ACK=true
                shift
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            -*)
                print_error "Unknown option: $1"
                usage
                exit 1
                ;;
            *)
                if [[ -z "$CONTRACT_NAME" ]]; then
                    CONTRACT_NAME="$1"
                else
                    print_error "Unexpected positional argument: $1"
                    usage
                    exit 1
                fi
                shift
                ;;
        esac
    done

    if [[ -z "$CONTRACT_NAME" || -z "$NETWORK" ]]; then
        print_error "Both <contract_name> and --network are required"
        usage
        exit 1
    fi

    if [[ "$DRY_RUN" == true && "$LIVE_ACK" == true ]]; then
        print_error "Choose either --dry-run or --i-understand-this-is-live, not both"
        exit 1
    fi

    if [[ "$DRY_RUN" == false && "$LIVE_ACK" == false ]]; then
        print_error "Specify either --dry-run or --i-understand-this-is-live"
        exit 1
    fi

    if [[ -z "$PLAN_PATH" ]]; then
        PLAN_PATH="$PROJECT_ROOT/deployments/$NETWORK/$CONTRACT_NAME/plan.json"
    fi
}

validate_plan() {
    if [[ ! -f "$PLAN_PATH" ]]; then
        print_error "Migration plan not found: $PLAN_PATH"
        exit 1
    fi

    python3 - "$PLAN_PATH" "$CONTRACT_NAME" "$NETWORK" <<'PY'
import json
import sys
from pathlib import Path

plan_path = Path(sys.argv[1])
contract_name = sys.argv[2]
network = sys.argv[3]

with plan_path.open("r", encoding="utf-8") as handle:
    plan = json.load(handle)

required_fields = [
    "new_wasm_hash",
    "version_bump",
    "expected_storage_migration_steps",
    "expected_gas",
    "expected_event_emissions",
]

missing = [field for field in required_fields if field not in plan]
if missing:
    raise SystemExit(f"Plan is missing required field(s): {', '.join(missing)}")

version_bump = plan["version_bump"]
if not isinstance(version_bump, dict) or "from" not in version_bump or "to" not in version_bump:
    raise SystemExit("Plan field 'version_bump' must be an object with 'from' and 'to'")

if not isinstance(plan["expected_storage_migration_steps"], list) or not plan["expected_storage_migration_steps"]:
    raise SystemExit("Plan field 'expected_storage_migration_steps' must be a non-empty array")

if not isinstance(plan["expected_event_emissions"], list):
    raise SystemExit("Plan field 'expected_event_emissions' must be an array")

plan_contract = plan.get("contract_name")
if plan_contract and plan_contract != contract_name:
    raise SystemExit(
        f"Plan contract_name '{plan_contract}' does not match requested contract '{contract_name}'"
    )

plan_network = plan.get("network")
if plan_network and plan_network != network:
    raise SystemExit(
        f"Plan network '{plan_network}' does not match requested network '{network}'"
    )

print("ok")
PY
}

render_dry_run_report() {
    print_step "Rendering dry-run migration projection from $PLAN_PATH"
    python3 - "$PLAN_PATH" "$CONTRACT_NAME" "$NETWORK" <<'PY'
import json
import sys
from collections import Counter

plan_path = sys.argv[1]
contract_name = sys.argv[2]
network = sys.argv[3]

with open(plan_path, "r", encoding="utf-8") as handle:
    plan = json.load(handle)

contract_id = plan.get("contract_id", "<set contract_id in plan>")
version_bump = plan["version_bump"]
steps = plan["expected_storage_migration_steps"]
events = plan["expected_event_emissions"]
gas = plan["expected_gas"]

symbols = {
    "add": "+",
    "backfill": "+",
    "update": "~",
    "rename": "~",
    "remove": "-",
    "delete": "-",
    "noop": "=",
}

counter = Counter()

print(f"Migration dry-run for {contract_name} on {network}")
print(f"  Contract ID: {contract_id}")
print(f"  New WASM hash: {plan['new_wasm_hash']}")
print(f"  Version bump: v{version_bump['from']} -> v{version_bump['to']}")
print()

print("Projected storage state diff:")
for index, step in enumerate(steps, start=1):
    action = str(step.get("action", "update")).lower()
    key = step.get("key") or step.get("storage_key") or f"migration_step_{index}"
    before = step.get("from", "<unchanged>")
    after = step.get("to", "<unchanged>")
    description = step.get("description")
    symbol = symbols.get(action, "~")
    counter[action] += 1
    print(f"  {symbol} [{action}] {key}: {before} -> {after}")
    if description:
        print(f"    note: {description}")
print(
    f"  Summary: {sum(counter.values())} step(s) "
    f"across {counter.get('add', 0) + counter.get('backfill', 0)} adds/backfills, "
    f"{counter.get('update', 0) + counter.get('rename', 0)} updates, "
    f"{counter.get('remove', 0) + counter.get('delete', 0)} removals"
)
print()

print("Projected gas cost:")
if isinstance(gas, dict):
    for key, value in gas.items():
        print(f"  - {key}: {value}")
else:
    print(f"  - total: {gas}")
print()

print("Projected event emissions:")
if events:
    for index, event in enumerate(events, start=1):
        topic = event.get("topic", f"event_{index}")
        data = event.get("data", "<no data>")
        description = event.get("description")
        print(f"  - {topic}: {data}")
        if description:
            print(f"    note: {description}")
else:
    print("  - none declared")
print()

notes = plan.get("notes", [])
if notes:
    print("Operator notes:")
    for note in notes:
        print(f"  - {note}")
    print()

print("Dry-run mode only: no Soroban transaction has been submitted.")
PY
}

plan_value() {
    local key="$1"
    python3 - "$PLAN_PATH" "$key" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    plan = json.load(handle)

key = sys.argv[2]

if key == "contract_id":
    value = plan.get("contract_id", "")
elif key == "new_wasm_hash":
    value = plan["new_wasm_hash"]
elif key == "to_version":
    value = plan["version_bump"]["to"]
elif key == "admin_identity":
    value = plan.get("admin_identity", "")
elif key == "upgrade_entrypoint":
    value = plan.get("upgrade_entrypoint", "upgrade")
elif key == "caller_arg_name":
    value = plan.get("caller_arg_name", "caller")
else:
    raise SystemExit(f"Unsupported key: {key}")

print(value)
PY
}

confirm_live_run() {
    local contract_id="$1"
    local wasm_hash="$2"
    local to_version="$3"
    local identity="$4"

    print_warn "Live mode will submit an upgrade transaction."
    echo "  Network: $NETWORK"
    echo "  Contract: $CONTRACT_NAME"
    echo "  Contract ID: $contract_id"
    echo "  Identity: $identity"
    echo "  New WASM hash: $wasm_hash"
    echo "  Target version: $to_version"
    echo
    read -r -p "Type LIVE to continue: " confirmation
    if [[ "$confirmation" != "LIVE" ]]; then
        print_error "Live migration aborted by operator"
        exit 1
    fi
}

run_live_migration() {
    require_command soroban

    local contract_id
    contract_id="$(plan_value "contract_id")"
    local wasm_hash
    wasm_hash="$(plan_value "new_wasm_hash")"
    local to_version
    to_version="$(plan_value "to_version")"
    local default_identity
    default_identity="$(plan_value "admin_identity")"
    local upgrade_entrypoint
    upgrade_entrypoint="$(plan_value "upgrade_entrypoint")"
    local caller_arg_name
    caller_arg_name="$(plan_value "caller_arg_name")"

    if [[ -z "$contract_id" ]]; then
        print_error "Plan must include contract_id for live execution"
        exit 1
    fi

    if [[ -z "$IDENTITY" ]]; then
        IDENTITY="$default_identity"
    fi

    if [[ -z "$IDENTITY" ]]; then
        print_error "No live identity supplied. Use --identity or add admin_identity to the plan."
        exit 1
    fi

    local caller_address
    caller_address="$(soroban config identity address "$IDENTITY")"

    confirm_live_run "$contract_id" "$wasm_hash" "$to_version" "$IDENTITY"

    print_step "Submitting live migration transaction"
    soroban contract invoke \
        --id "$contract_id" \
        --source "$IDENTITY" \
        --network "$NETWORK" \
        -- \
        "$upgrade_entrypoint" \
        "--$caller_arg_name" "$caller_address" \
        --new_wasm_hash "$wasm_hash" \
        --new_version "$to_version"

    print_info "Live migration submitted successfully"
}

main() {
    require_command python3
    parse_args "$@"
    validate_plan >/dev/null

    if [[ "$DRY_RUN" == "true" ]]; then
        render_dry_run_report
        return 0
    fi

    run_live_migration
}

main "$@"
