#!/usr/bin/env bash
# deploy_dryrun.sh - Simulate contract deployment and output JSON plan
# for release candidates without actually deploying.
#
# Usage:
#   ./scripts/deploy_dryrun.sh <contract_name> <network> [--output <file>]
#
# Examples:
#   ./scripts/deploy_dryrun.sh medical_records testnet
#   ./scripts/deploy_dryrun.sh governor mainnet --output plan.json
#   ./scripts/deploy_dryrun.sh --all testnet

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
CONTRACTS_DIR="$ROOT_DIR/contracts"
DEPLOYMENTS_DIR="$ROOT_DIR/deployments"
MANIFEST="$DEPLOYMENTS_DIR/deployment-manifest.json"
PLAN_SCHEMA="$DEPLOYMENTS_DIR/plan-schema.json"

source "$SCRIPT_DIR/logger.sh"

# ─── Helpers ─────────────────────────────────────────────────────────────────

usage() {
    cat <<EOF
Usage: $(basename "$0") <contract_name|\"--all\"> <network> [options]

Options:
  --output <file>    Write plan to file (default: stdout)
  --verbose          Include detailed analysis
  -h, --help         Show this help message

Examples:
  $(basename "$0") medical_records testnet
  $(basename "$0") governor mainnet --output plan.json
  $(basename "$0") --all testnet --output full-plan.json
EOF
}

validate_network() {
    local network="$1"
    case "$network" in
        local|testnet|futurenet|mainnet) return 0 ;;
        *)
            log "ERROR" "Unknown network: $network"
            exit 1
            ;;
    esac
}

check_contract_exists() {
    local contract="$1"
    if [[ ! -d "$CONTRACTS_DIR/$contract" ]]; then
        log "ERROR" "Contract directory not found: $CONTRACTS_DIR/$contract"
        exit 1
    fi
}

get_wasm_path() {
    local contract="$1"
    echo "target/wasm32-unknown-unknown/release/${contract}.wasm"
}

get_network_passphrase() {
    local network="$1"
    case "$network" in
        local) echo "Standalone Network ; February 2017" ;;
        testnet) echo "Test SDF Network ; September 2015" ;;
        futurenet) echo "Test SDF Future Network ; October 2022" ;;
        mainnet) echo "Public Global Stellar Network ; September 2015" ;;
    esac
}

get_network_rpc() {
    local network="$1"
    case "$network" in
        local) echo "http://localhost:8000/soroban/rpc" ;;
        testnet) echo "https://soroban-testnet.stellar.org" ;;
        futurenet) echo "https://rpc-futurenet.stellar.org" ;;
        mainnet) echo "https://soroban-mainnet.stellar.org" ;;
    esac
}

check_wasm_exists() {
    local wasm_path="$1"
    if [[ -f "$ROOT_DIR/$wasm_path" ]]; then
        local size
        size=$(wc -c < "$ROOT_DIR/$wasm_path" 2>/dev/null || echo "0")
        echo "$size"
    else
        echo "-1"
    fi
}

get_manifest_entry() {
    local contract="$1"
    if [[ -f "$MANIFEST" ]]; then
        # Simple JSON extraction for contract entry
        python3 -c "
import json, sys
with open('$MANIFEST') as f:
    data = json.load(f)
for c in data.get('contracts', []):
    if c.get('name') == '$contract':
        print(json.dumps(c))
        sys.exit(0)
print('{}')
" 2>/dev/null || echo "{}"
    else
        echo "{}"
    fi
}

estimate_gas() {
    local contract="$1"
    # Heuristic: larger contracts need more instructions
    local wasm_path
    wasm_path=$(get_wasm_path "$contract")
    local size
    size=$(check_wasm_exists "$wasm_path")

    if [[ "$size" == "-1" ]]; then
        echo "50000000"
    elif (( size < 100000 )); then
        echo "20000000"
    elif (( size < 500000 )); then
        echo "50000000"
    else
        echo "100000000"
    fi
}

check_dependencies() {
    local contract="$1"
    local manifest_entry
    manifest_entry=$(get_manifest_entry "$contract")
    if [[ "$manifest_entry" != "{}" ]]; then
        echo "$manifest_entry" | python3 -c "
import json, sys
data = json.load(sys.stdin)
deps = data.get('dependencies', [])
print(json.dumps(deps))
" 2>/dev/null || echo "[]"
    else
        echo "[]"
    fi
}

# ─── Plan Generation ────────────────────────────────────────────────────────

generate_plan_single() {
    local contract="$1"
    local network="$2"
    local verbose="${3:-false}"

    check_contract_exists "$contract"

    local wasm_path
    wasm_path=$(get_wasm_path "$contract")
    local wasm_size
    wasm_size=$(check_wasm_exists "$wasm_path")
    local passphrase
    passphrase=$(get_network_passphrase "$network")
    local rpc_url
    rpc_url=$(get_network_rpc "$network")
    local gas_estimate
    gas_estimate=$(estimate_gas "$contract")
    local deps
    deps=$(check_dependencies "$contract")
    local manifest_entry
    manifest_entry=$(get_manifest_entry "$contract")

    # Determine if build is needed
    local build_status="skipped"
    local build_check="passed"
    if [[ "$wasm_size" == "-1" ]]; then
        build_status="required"
        build_check="wasm_not_found"
    fi

    # Determine safety checks
    local safety_status="passed"
    local safety_warnings="[]"
    if [[ "$network" == "mainnet" ]]; then
        safety_status="requires_approval"
        safety_warnings='["mainnet_deployment_requires_manual_approval"]'
    fi

    # Build the plan JSON
    python3 -c "
import json

plan = {
    'plan_version': '1.0.0',
    'generated_at': '$(date -u +"%Y-%m-%dT%H:%M:%SZ")',
    'dry_run': True,
    'network': {
        'name': '$network',
        'passphrase': '$passphrase',
        'rpc_url': '$rpc_url'
    },
    'contract': {
        'name': '$contract',
        'wasm_path': '$wasm_path',
        'wasm_size_bytes': $wasm_size if $wasm_size != -1 else None,
        'dependencies': $deps
    },
    'build': {
        'status': '$build_status',
        'check': '$build_check'
    },
    'deployment': {
        'estimated_gas': $gas_estimate,
        'source_account': '<identity>',
        'init_required': True
    },
    'safety': {
        'status': '$safety_status',
        'warnings': $safety_warnings
    },
    'validation': {
        'contract_exists': True,
        'wasm_available': $( [[ "$wasm_size" != "-1" ]] && echo "true" || echo "false" ),
        'dependencies_met': True
    },
    'estimated_cost_xlm': round($gas_estimate * 0.00001, 6)
}
print(json.dumps(plan, indent=2))
"
}

generate_plan_all() {
    local network="$1"
    local verbose="${2:-false}"

    # Collect all contract directories
    local contracts=()
    for dir in "$CONTRACTS_DIR"/*/; do
        if [[ -d "$dir" && -f "$dir/Cargo.toml" ]]; then
            local name
            name=$(basename "$dir")
            contracts+=("$name")
        fi
    done

    # Sort by deployment order from manifest if available
    python3 -c "
import json, sys, os

contracts = '$(printf "%s" "${contracts[*]}")'.split()
network = '$network'

# Try to read manifest for ordering
manifest_path = '$MANIFEST'
ordered = []
try:
    with open(manifest_path) as f:
        manifest = json.load(f)
    contract_order = {}
    for c in manifest.get('contracts', []):
        contract_order[c['name']] = c.get('deploy_order', 999)
    contracts.sort(key=lambda x: contract_order.get(x, 999))
except:
    pass

plan = {
    'plan_version': '1.0.0',
    'generated_at': '$(date -u +"%Y-%m-%dT%H:%M:%SZ")',
    'dry_run': True,
    'network': {
        'name': network,
        'passphrase': '$(get_network_passphrase "$network")',
        'rpc_url': '$(get_network_rpc "$network")'
    },
    'total_contracts': len(contracts),
    'deployment_order': [],
    'summary': {
        'total_estimated_gas': 0,
        'builds_needed': 0,
        'wasm_available': 0,
        'wasm_missing': 0,
        'safety_warnings': []
    }
}

for i, name in enumerate(contracts):
    wasm_path = f'target/wasm32-unknown-unknown/release/{name}.wasm'
    wasm_full = os.path.join('$ROOT_DIR', wasm_path)
    wasm_exists = os.path.isfile(wasm_full)
    wasm_size = os.path.getsize(wasm_full) if wasm_exists else -1

    gas_est = 50000000
    if wasm_exists and wasm_size < 100000:
        gas_est = 20000000
    elif wasm_exists and wasm_size < 500000:
        gas_est = 50000000

    entry = {
        'order': i + 1,
        'name': name,
        'wasm_path': wasm_path,
        'wasm_available': wasm_exists,
        'wasm_size_bytes': wasm_size if wasm_exists else None,
        'estimated_gas': gas_est,
        'estimated_cost_xlm': round(gas_est * 0.00001, 6)
    }
    plan['deployment_order'].append(entry)
    plan['summary']['total_estimated_gas'] += gas_est
    if wasm_exists:
        plan['summary']['wasm_available'] += 1
    else:
        plan['summary']['wasm_missing'] += 1
        plan['summary']['builds_needed'] += 1

if network == 'mainnet':
    plan['summary']['safety_warnings'].append('mainnet_deployment_requires_manual_approval')

plan['summary']['estimated_total_cost_xlm'] = round(plan['summary']['total_estimated_gas'] * 0.00001, 6)

print(json.dumps(plan, indent=2))
"
}

# ─── Main ────────────────────────────────────────────────────────────────────

main() {
    local contract=""
    local network=""
    local output_file=""
    local verbose="false"
    local run_all="false"

    while [[ $# -gt 0 ]]; do
        case "$1" in
            --all)
                run_all="true"
                shift
                ;;
            --output)
                output_file="$2"
                shift 2
                ;;
            --verbose)
                verbose="true"
                shift
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            -*)
                log "ERROR" "Unknown option: $1"
                usage
                exit 1
                ;;
            *)
                if [[ -z "$contract" ]]; then
                    contract="$1"
                elif [[ -z "$network" ]]; then
                    network="$1"
                fi
                shift
                ;;
        esac
    done

    if [[ -z "$network" ]]; then
        log "ERROR" "Network is required"
        usage
        exit 1
    fi

    validate_network "$network"

    local plan=""
    if [[ "$run_all" == "true" ]]; then
        log "INFO" "Generating dry-run plan for ALL contracts on '$network'..."
        plan=$(generate_plan_all "$network" "$verbose")
    elif [[ -n "$contract" ]]; then
        log "INFO" "Generating dry-run plan for '$contract' on '$network'..."
        plan=$(generate_plan_single "$contract" "$network" "$verbose")
    else
        log "ERROR" "Contract name or --all is required"
        usage
        exit 1
    fi

    # Output
    if [[ -n "$output_file" ]]; then
        echo "$plan" > "$output_file"
        log "INFO" "Plan written to: $output_file"
    else
        echo "$plan"
    fi

    log "INFO" "Dry-run plan generation complete (no deployments were made)"
}

main "$@"
