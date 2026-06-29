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


---

## Contract Interaction Examples

The following examples use `soroban contract invoke` directly. Replace `<CONTRACT_ID>` with the deployed contract address and `<NETWORK>` with `local`, `testnet`, or `futurenet`.

### medical_records

```bash
# Initialize
soroban contract invoke --id <CONTRACT_ID> --source admin --network <NETWORK> \
  -- initialize --admin <ADMIN_ADDRESS>

# Register a user
soroban contract invoke --id <CONTRACT_ID> --source admin --network <NETWORK> \
  -- register_user --user <USER_ADDRESS> --role Doctor

# Write a record
soroban contract invoke --id <CONTRACT_ID> --source doctor --network <NETWORK> \
  -- write_record \
  --patient <PATIENT_ADDRESS> \
  --encrypted_data "QmXxx..." \
  --category "General" \
  --is_confidential false

# Read a record
soroban contract invoke --id <CONTRACT_ID> --source doctor --network <NETWORK> \
  -- read_record --record_id 1
```

### patient_consent_management

```bash
# Initialize
soroban contract invoke --id <CONTRACT_ID> --source admin --network <NETWORK> \
  -- initialize --admin <ADMIN_ADDRESS>

# Grant consent
soroban contract invoke --id <CONTRACT_ID> --source patient --network <NETWORK> \
  -- grant_consent --patient <PATIENT_ADDRESS> --provider <PROVIDER_ADDRESS>

# Revoke consent
soroban contract invoke --id <CONTRACT_ID> --source patient --network <NETWORK> \
  -- revoke_consent --patient <PATIENT_ADDRESS> --provider <PROVIDER_ADDRESS>

# Check consent
soroban contract invoke --id <CONTRACT_ID> --source anyone --network <NETWORK> \
  -- has_consent --patient <PATIENT_ADDRESS> --provider <PROVIDER_ADDRESS>
```

### healthcare_payment

```bash
# Initialize
soroban contract invoke --id <CONTRACT_ID> --source admin --network <NETWORK> \
  -- initialize --admin <ADMIN_ADDRESS> --token <TOKEN_ADDRESS>

# Submit a claim
soroban contract invoke --id <CONTRACT_ID> --source patient --network <NETWORK> \
  -- submit_claim \
  --patient <PATIENT_ADDRESS> \
  --provider <PROVIDER_ADDRESS> \
  --service_id "SVC-001" \
  --amount 5000000 \
  --policy_id "POL-123"

# Approve a claim
soroban contract invoke --id <CONTRACT_ID> --source admin --network <NETWORK> \
  -- approve_claim --claim_id 1
```

### emergency_access_override

```bash
# Initialize
soroban contract invoke --id <CONTRACT_ID> --source admin --network <NETWORK> \
  -- initialize \
  --admin <ADMIN_ADDRESS> \
  --approvers '["<APPROVER1>","<APPROVER2>"]' \
  --threshold 1

# Grant emergency access
soroban contract invoke --id <CONTRACT_ID> --source approver --network <NETWORK> \
  -- grant_emergency_access \
  --approver <APPROVER_ADDRESS> \
  --patient <PATIENT_ADDRESS> \
  --provider <PROVIDER_ADDRESS> \
  --duration_seconds 3600
```

### zkp_registry

```bash
# Register a proof
soroban contract invoke --id <CONTRACT_ID> --source submitter --network <NETWORK> \
  -- register_proof \
  --proof_type SNARK \
  --circuit_id "age_verification_v1" \
  --proof_data <HEX_BYTES> \
  --public_inputs '[]' \
  --vk_hash <32_BYTE_HASH>

# Verify a proof
soroban contract invoke --id <CONTRACT_ID> --source verifier --network <NETWORK> \
  -- verify_proof --proof_id 1
```

### cross_chain_bridge

```bash
# Send a cross-chain message
soroban contract invoke --id <CONTRACT_ID> --source relayer --network <NETWORK> \
  -- relay_message \
  --message_id <32_BYTE_ID> \
  --source_chain Ethereum \
  --payload "..." \
  --signature <64_BYTE_SIG>
```

### fhir_integration

```bash
# Store a FHIR observation
soroban contract invoke --id <CONTRACT_ID> --source provider --network <NETWORK> \
  -- store_observation \
  --patient <PATIENT_ADDRESS> \
  --resource_type Observation \
  --data "..."

# Retrieve FHIR resources for a patient
soroban contract invoke --id <CONTRACT_ID> --source provider --network <NETWORK> \
  -- get_patient_resources --patient <PATIENT_ADDRESS>
```
