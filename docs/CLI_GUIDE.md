# Uzima CLI Guide

This document supplements `README.md` with advanced CLI commands for transaction management, batch operations, debugging, and account utilities.

## New commands in `scripts/advanced_cli.sh`

- `account-info <network> <account_id>`
  - Fetches account data via Horizon or local node.

- `tx-history <network> <subject> [limit]`
  - Fetches transaction history with a limit.

- `batch-invoke <contract_id> <network> <file>`
  - Runs multiple calls from a file, comment lines with `#`.

- `debug-call <contract_id> <network> <function> [args...]`
  - Runs a call with verbose information and instruction cost.

- `account-manage <list|create|delete> [name]`
  - Manage Soroban identities used by commands.

## Usage examples

```bash
# Account info
./scripts/advanced_cli.sh account-info local GABC...

# Transaction history
./scripts/advanced_cli.sh tx-history testnet GABC... 20

# Batch invoke
cat > batch.txt <<EOF
add_record GDoctor GPatient "Cold" "Rest" false "\"tag\"" "Modern" "Med" "Qm..."
get_record 1
EOF
./scripts/advanced_cli.sh batch-invoke GContract local batch.txt

# Debug call
./scripts/advanced_cli.sh debug-call GContract local get_record 1

# Account management
./scripts/advanced_cli.sh account-manage list
./scripts/advanced_cli.sh account-manage create dev-user
./scripts/advanced_cli.sh account-manage delete dev-user
```

## Validation and error handling

- `account-info` and `tx-history` validate network and limit.
- `batch-invoke` verifies the file exists and fails at first invalid step.
- `debug-call` checks for a configured identity.
- Input errors print a message and exit with code 1.

## Testing

Run unit tests:

```bash
bash tests/cli/advanced_cli_tests.sh
```

## Backward compatibility

- Existing scripts like `scripts/interact.sh` remain unchanged.
- `advanced_cli.sh` is additive and does not modify existing command semantics.

