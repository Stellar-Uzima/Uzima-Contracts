#!/usr/bin/env bash

# Advanced CLI for Uzima Contracts
# Adds transaction history, batch operations, debugging, and account utilities

set -euo pipefail

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

print_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
print_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
print_err() { echo -e "${RED}[ERROR]${NC} $1"; }

check_soroban() {
    if ! command -v soroban >/dev/null 2>&1; then
        print_err "soroban CLI not installed. Install with 'cargo install --locked soroban-cli'"
        exit 1
    fi
}

validate_network() {
    local network=$1
    case "$network" in
        local|testnet|futurenet|mainnet)
            :
            ;;
        *)
            print_err "Unsupported network: $network"
            print_err "Supported networks: local, testnet, futurenet, mainnet"
            exit 1
            ;;
    esac
}

show_help() {
    cat <<EOF
Advanced Uzima CLI

Usage:
  $0 <command> [args...]

Commands:
  help                                       Show this help message
  account-info <network> <account_id>        Show account metadata from Soroban/Horizon
  tx-history <network> <subject> [limit]     Fetch transaction history for account or contract
  batch-invoke <contract_id> <network> <file> Execute batch invocation file (op per line)
  debug-call <contract_id> <network> <fn> [args...]  Invoke function with verbose output (gas/budget)
  account-manage <subcommand> [args...]      Manage local identities (create/list/delete)

Examples:
  $0 account-info local GABC...   # show account details
  $0 tx-history testnet GABC... 100
  $0 batch-invoke GBXYZ local operations.txt
  $0 debug-call GBXYZ local get_record 1
  $0 account-manage list

EOF
}

account_info() {
    check_soroban
    validate_network "$1"
    local network=$1
    local account_id=$2

    if [[ -z "$account_id" ]]; then
        print_err "Missing account_id"
        exit 1
    fi

    # Normalized horizon endpoint for each network.
    local horizon=https://soroban-$network.stellar.org
    if [[ "$network" == "local" ]]; then
        horizon=http://localhost:8000
    fi

    print_info "Fetching account info for $account_id on $network"
    curl -s "$horizon/accounts/$account_id" | jq .
}

tx_history() {
    check_soroban
    validate_network "$1"
    local network=$1
    local subject=$2
    local limit=${3:-50}

    if [[ -z "$subject" ]]; then
        print_err "Missing subject (account or contract address)"
        exit 1
    fi
    if ! [[ "$limit" =~ ^[0-9]+$ ]] || (( limit <= 0 )); then
        print_err "Invalid limit: $limit"
        exit 1
    fi

    local horizon=https://soroban-$network.stellar.org
    if [[ "$network" == "local" ]]; then
        horizon=http://localhost:8000
    fi

    print_info "Fetching last $limit transactions for $subject from $network"

    # Support account/contract using horizon transaction endpoint
    curl -s "$horizon/accounts/$subject/transactions?limit=$limit" | jq .
}

batch_invoke() {
    check_soroban
    local contract_id=$1
    local network=$2
    local file=$3

    if [[ -z "$contract_id" || -z "$network" || -z "$file" ]]; then
        print_err "Usage: $0 batch-invoke <contract_id> <network> <file>"
        exit 1
    fi
    if [[ ! -f "$file" ]]; then
        print_err "Batch file not found: $file"
        exit 1
    fi

    validate_network "$network"

    local identity=${SOROBAN_IDENTITY:-default}
    if ! soroban config identity show "$identity" >/dev/null 2>&1; then
        print_err "Identity '$identity' not found. Run 'soroban config identity generate $identity'"
        exit 1
    fi

    local line_num=0
    while IFS= read -r line || [[ -n "$line" ]]; do
        line_num=$((line_num + 1))
        [[ "$line" =~ ^[[:space:]]*$ ]] && continue
        [[ "$line" =~ ^# ]] && continue

        # Fields: fn arg1 arg2 ...
        local fn_name args
        fn_name=$(awk '{print $1}' <<< "$line")
        args=$(awk '{for (i=2; i<=NF; i++) printf "%s ", $i}' <<< "$line")

        if [[ -z "$fn_name" ]]; then
            print_warn "Skipping empty line $line_num"
            continue
        fi

        print_info "[$line_num] Invoking $fn_name $args"
        if ! soroban contract invoke --id "$contract_id" --network "$network" --source "$identity" -- "$fn_name" $args; then
            print_err "Batch invocation failed at line $line_num : $line"
            exit 1
        fi
    done < "$file"

    print_info "Batch invocation completed successfully"
}

debug_call() {
    check_soroban
    local contract_id=$1
    local network=$2
    local fn_name=$3
    shift 3
    local args=("$@")

    if [[ -z "$contract_id" || -z "$network" || -z "$fn_name" ]]; then
        print_err "Usage: $0 debug-call <contract_id> <network> <function> [args...]"
        exit 1
    fi

    validate_network "$network"

    local identity=${SOROBAN_IDENTITY:-default}
    if ! soroban config identity show "$identity" >/dev/null 2>&1; then
        print_err "Identity '$identity' not found"
        exit 1
    fi

    print_info "Starting debug call $fn_name on $contract_id (network $network)"
    local before_budget=0 after_budget=0

    # Budget measurement is local and may only work with test environment.
    before_budget=$(soroban contract invoke --id "$contract_id" --network "$network" --source "$identity" -- "$fn_name" "${args[*]}" --verbose 2>&1 | tee /dev/stderr | grep -oP 'CPU Instructions:\s*\K[0-9]+' || echo 0)
    if [[ -z "$before_budget" ]]; then
        before_budget=0
    fi

    print_info "Debug invocation complete. Instructions used: $before_budget"
}

account_manage() {
    local sub=$1
    case "$sub" in
        list)
            soroban config identity list || print_err "No identities found"
            ;;
        create)
            local id=${2:-default}
            soroban config identity generate "$id"
            print_info "Created identity '$id'"
            ;;
        delete)
            local id=${2:-default}
            soroban config identity remove "$id"
            print_info "Deleted identity '$id'"
            ;;
        *)
            print_err "account-manage subcommand support: list|create|delete"
            exit 1
            ;;
    esac
}

main() {
    if [[ $# -lt 1 ]]; then
        show_help
        exit 0
    fi

    case "$1" in
        help|-h|--help)
            show_help
            ;;
        account-info)
            if [[ $# -ne 3 ]]; then
                print_err "Usage: $0 account-info <network> <account_id>"
                exit 1
            fi
            account_info "$2" "$3"
            ;;
        tx-history)
            if [[ $# -lt 3 ]]; then
                print_err "Usage: $0 tx-history <network> <subject> [limit]"
                exit 1
            fi
            tx_history "$2" "$3" "${4:-50}"
            ;;
        batch-invoke)
            if [[ $# -ne 4 ]]; then
                print_err "Usage: $0 batch-invoke <contract_id> <network> <file>"
                exit 1
            fi
            batch_invoke "$2" "$3" "$4"
            ;;
        debug-call)
            if [[ $# -lt 4 ]]; then
                print_err "Usage: $0 debug-call <contract_id> <network> <function> [args...]"
                exit 1
            fi
            debug_call "$2" "$3" "$4" "${@:5}"
            ;;
        account-manage)
            if [[ $# -lt 2 ]]; then
                print_err "Usage: $0 account-manage <subcommand>"
                exit 1
            fi
            account_manage "$2" "${@:3}"
            ;;
        *)
            print_err "Unknown command: $1"
            show_help
            exit 1
            ;;
    esac
}

main "$@"
