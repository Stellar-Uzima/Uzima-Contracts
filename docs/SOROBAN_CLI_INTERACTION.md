# Soroban CLI Interaction Guide

This comprehensive guide covers all aspects of interacting with deployed Soroban smart contracts using the Soroban CLI.

## Table of Contents

1. [Function Invocation](#function-invocation)
2. [Event Monitoring](#event-monitoring)
3. [State Inspection](#state-inspection)
4. [Transaction Building](#transaction-building)
5. [Query Methods](#query-methods)
6. [Error Handling](#error-handling)
7. [Advanced Patterns](#advanced-patterns)

## Function Invocation

### Basic Function Calls

```bash
# Simple function call without arguments
soroban contract invoke \
  --id CONTRACT_ID \
  --function get_value \
  --source alice \
  --network testnet

# Function call with single argument
soroban contract invoke \
  --id CONTRACT_ID \
  --function transfer \
  --arg-to "GB7TAYUEQH6TZTYNNO6R7JQ2GTGJZYJSTZQYV2MYQMT2E2D2BZJQQQY" \
  --arg-amount "1000" \
  --source alice \
  --network testnet
```

### Complex Function Arguments

```bash
# Function with multiple arguments
soroban contract invoke \
  --id CONTRACT_ID \
  --function create_record \
  --arg-patient-id "12345" \
  --arg-doctor-id "67890" \
  --arg-diagnosis "Hypertension" \
  --arg-treatment "Lifestyle changes" \
  --arg-is-chronic "true" \
  --source doctor \
  --network testnet

# Function with JSON arguments
soroban contract invoke \
  --id CONTRACT_ID \
  --function update_config \
  --arg-config '{"max_records": 1000, "retention_days": 365}' \
  --source admin \
  --network testnet

# Function with array arguments
soroban contract invoke \
  --id CONTRACT_ID \
  --function batch_update \
  --arg-records '["rec1", "rec2", "rec3"]' \
  --source admin \
  --network testnet
```

### Invocation with Custom Options

```bash
# Set custom fee
soroban contract invoke \
  --id CONTRACT_ID \
  --function expensive_operation \
  --source alice \
  --network testnet \
  --fee 5000

# Use specific nonce
soroban contract invoke \
  --id CONTRACT_ID \
  --function transfer \
  --arg-to "GB7T..." \
  --arg-amount "1000" \
  --source alice \
  --network testnet \
  --nonce 12345

# Dry run simulation
soroban contract invoke \
  --id CONTRACT_ID \
  --function transfer \
  --arg-to "GB7T..." \
  --arg-amount "1000" \
  --source alice \
  --network testnet \
  --dry-run
```

### Batch Operations

```bash
# Create batch file
cat > batch_operations.txt <<EOF
# Transfer tokens
transfer GB7TAYUEQH6TZTYNNO6R7JQ2GTGJZYJSTZQYV2MYQMT2E2D2BZJQQQY 1000
transfer GD5... 500

# Update records
update_record 12345 "New diagnosis"
update_record 67890 "Updated treatment"

# Query operations
get_balance
get_record 12345
EOF

# Execute batch operations
while IFS= read -r line; do
  # Skip comments and empty lines
  if [[ $line =~ ^#.*$ ]] || [[ -z $line ]]; then
    continue
  fi
  
  # Parse command and arguments
  read -ra parts <<< "$line"
  function="${parts[0]}"
  args=("${parts[@]:1}")
  
  # Build soroban command
  cmd="soroban contract invoke --id $CONTRACT_ID --function $function --source alice --network testnet"
  for arg in "${args[@]}"; do
    cmd+=" --arg-$arg"
  done
  
  # Execute command
  echo "Executing: $cmd"
  eval $cmd
done < batch_operations.txt
```

## Event Monitoring

### Real-time Event Monitoring

```bash
# Monitor events from a contract
soroban contract events \
  --id CONTRACT_ID \
  --network testnet \
  --start-ledger 100000 \
  --end-ledger 101000

# Monitor events with filters
soroban contract events \
  --id CONTRACT_ID \
  --network testnet \
  --start-ledger 100000 \
  --end-ledger 101000 \
  --topic "transfer"

# Monitor specific event types
soroban contract events \
  --id CONTRACT_ID \
  --network testnet \
  --start-ledger 100000 \
  --end-ledger 101000 \
  --topic "record_created" \
  --topic "record_updated"
```

### Event Analysis

```bash
# Get latest events
soroban contract events \
  --id CONTRACT_ID \
  --network testnet \
  --limit 100

# Parse events for analysis
soroban contract events \
  --id CONTRACT_ID \
  --network testnet \
  --start-ledger $(soroban info ledger --network testnet | jq -r '.sequence - 1000') \
  --end-ledger $(soroban info ledger --network testnet | jq -r '.sequence') \
  | jq '.events[] | {type: .type, data: .body, tx_hash: .tx_hash}'
```

### Event Filtering and Processing

```bash
# Filter events by type
soroban contract events \
  --id CONTRACT_ID \
  --network testnet \
  --start-ledger 100000 \
  --end-ledger 101000 \
  | jq '.events[] | select(.body.type == "transfer")'

# Extract specific event data
soroban contract events \
  --id CONTRACT_ID \
  --network testnet \
  --start-ledger 100000 \
  --end-ledger 101000 \
  | jq '.events[] | select(.body.type == "transfer") | {from: .body.from, to: .body.to, amount: .body.amount}'
```

## State Inspection

### Contract Information

```bash
# Get basic contract info
soroban contract info \
  --id CONTRACT_ID \
  --network testnet

# Get detailed contract information
soroban contract info \
  --id CONTRACT_ID \
  --network testnet \
  | jq '.'

# Check contract WASM hash
soroban contract info \
  --id CONTRACT_ID \
  --network testnet \
  | jq '.wasm_hash'
```

### Storage Inspection

```bash
# Read contract storage key
soroban contract read \
  --id CONTRACT_ID \
  --key "storage_key" \
  --network testnet

# Read multiple storage keys
for key in "balance:alice" "balance:bob" "config"; do
  echo "Reading key: $key"
  soroban contract read --id $CONTRACT_ID --key "$key" --network testnet
done

# Read storage with hexadecimal key
soroban contract read \
  --id CONTRACT_ID \
  --key "0x1234567890abcdef" \
  --network testnet
```

### Ledger State Queries

```bash
# Get current ledger info
soroban info ledger --network testnet

# Get account information
soroban account info \
  --identity alice \
  --network testnet

# Get transaction information
soroban tx info \
  --hash TRANSACTION_HASH \
  --network testnet
```

### Contract State Analysis

```bash
# Analyze contract storage
soroban contract info --id $CONTRACT_ID --network testnet | jq '.storage_keys'

# Get all contract data (if accessible)
soroban contract invoke \
  --id $CONTRACT_ID \
  --function get_all_data \
  --source alice \
  --network testnet

# Check contract configuration
soroban contract invoke \
  --id $CONTRACT_ID \
  --function get_config \
  --source alice \
  --network testnet
```

## Transaction Building

### Building Custom Transactions

```bash
# Create transaction file
soroban tx new \
  --source alice \
  --network testnet \
  > transaction.xdr

# Add operation to transaction
soroban tx add-operation \
  --file transaction.xdr \
  --invoke-contract \
  --id CONTRACT_ID \
  --function transfer \
  --arg-to "GB7T..." \
  --arg-amount "1000"

# Sign transaction
soroban tx sign \
  --file transaction.xdr \
  --source alice \
  --network testnet

# Submit transaction
soroban tx submit \
  --file transaction.xdr \
  --network testnet
```

### Advanced Transaction Building

```bash
# Build multi-operation transaction
soroban tx new \
  --source alice \
  --network testnet \
  > batch_tx.xdr

# Add multiple operations
soroban tx add-operation \
  --file batch_tx.xdr \
  --invoke-contract \
  --id CONTRACT_ID \
  --function transfer \
  --arg-to "GB7T..." \
  --arg-amount "1000"

soroban tx add-operation \
  --file batch_tx.xdr \
  --invoke-contract \
  --id CONTRACT_ID \
  --function approve \
  --arg-spender "GD5..." \
  --arg-amount "500"

# Set transaction options
soroban tx set-options \
  --file batch_tx.xdr \
  --fee 2000 \
  --memo "Batch transfer and approval"

# Sign and submit
soroban tx sign --file batch_tx.xdr --source alice --network testnet
soroban tx submit --file batch_tx.xdr --network testnet
```

### Transaction Simulation

```bash
# Simulate transaction without submitting
soroban tx simulate \
  --file transaction.xdr \
  --network testnet

# Simulate with detailed results
soroban tx simulate \
  --file transaction.xdr \
  --network testnet \
  --detailed

# Simulate and get cost information
soroban tx simulate \
  --file transaction.xdr \
  --network testnet \
  | jq '.simulation.transaction_resource_fee'
```

## Query Methods

### Read-Only Operations

```bash
# Query contract state without transaction
soroban contract invoke \
  --id CONTRACT_ID \
  --function get_balance \
  --arg-user "GB7T..." \
  --source alice \
  --network testnet \
  --dry-run

# Query multiple values
soroban contract invoke \
  --id CONTRACT_ID \
  --function get_user_info \
  --arg-user "GB7T..." \
  --source alice \
  --network testnet \
  --dry-run
```

### Pagination and Limits

```bash
# Query with pagination
soroban contract invoke \
  --id CONTRACT_ID \
  --function get_records \
  --arg-offset "0" \
  --arg-limit "10" \
  --source alice \
  --network testnet \
  --dry-run

# Query all records with pagination
offset=0
limit=50
while true; do
  result=$(soroban contract invoke \
    --id CONTRACT_ID \
    --function get_records \
    --arg-offset "$offset" \
    --arg-limit "$limit" \
    --source alice \
    --network testnet \
    --dry-run)
  
  echo "Records $offset-$((offset + limit - 1)):"
  echo "$result"
  
  # Check if there are more records
  count=$(echo "$result" | jq '.records | length')
  if [ "$count" -lt "$limit" ]; then
    break
  fi
  
  offset=$((offset + limit))
done
```

### Filtering and Search

```bash
# Search records by criteria
soroban contract invoke \
  --id CONTRACT_ID \
  --function search_records \
  --arg-criteria '{"patient_id": "12345", "date_range": ["2023-01-01", "2023-12-31"]}' \
  --source alice \
  --network testnet \
  --dry-run

# Filter results
soroban contract invoke \
  --id CONTRACT_ID \
  --function filter_records \
  --arg-filter '{"status": "active", "type": "medical"}' \
  --source alice \
  --network testnet \
  --dry-run
```

## Error Handling

### Common Error Types

```bash
# Handle insufficient balance error
if soroban contract invoke \
  --id CONTRACT_ID \
  --function transfer \
  --arg-to "GB7T..." \
  --arg-amount "1000000" \
  --source alice \
  --network testnet 2>&1 | grep -q "insufficient balance"; then
  echo "Error: Insufficient balance"
  exit 1
fi

# Handle contract error
result=$(soroban contract invoke \
  --id CONTRACT_ID \
  --function restricted_function \
  --source alice \
  --network testnet 2>&1)

if echo "$result" | grep -q "contract error"; then
  echo "Contract error occurred: $result"
  exit 1
fi
```

### Retry Logic

```bash
# Retry failed operations
max_retries=3
retry_delay=5

for i in $(seq 1 $max_retries); do
  if soroban contract invoke \
    --id CONTRACT_ID \
    --function transfer \
    --arg-to "GB7T..." \
    --arg-amount "1000" \
    --source alice \
    --network testnet; then
    echo "Operation succeeded on attempt $i"
    break
  else
    echo "Attempt $i failed, retrying in $retry_delay seconds..."
    sleep $retry_delay
  fi
  
  if [ $i -eq $max_retries ]; then
    echo "Operation failed after $max_retries attempts"
    exit 1
  fi
done
```

### Error Analysis

```bash
# Parse error messages
error_output=$(soroban contract invoke \
  --id CONTRACT_ID \
  --function transfer \
  --arg-to "GB7T..." \
  --arg-amount "1000" \
  --source alice \
  --network testnet 2>&1)

if echo "$error_output" | grep -q "transaction failed"; then
  error_code=$(echo "$error_output" | jq -r '.error.code')
  error_message=$(echo "$error_output" | jq -r '.error.message')
  echo "Error $error_code: $error_message"
fi
```

## Advanced Patterns

### Atomic Operations

```bash
# Atomic transfer with approval
soroban tx new --source alice --network testnet > atomic_tx.xdr

# Add transfer operation
soroban tx add-operation \
  --file atomic_tx.xdr \
  --invoke-contract \
  --id CONTRACT_ID \
  --function transfer \
  --arg-to "GB7T..." \
  --arg-amount "1000"

# Add approval operation
soroban tx add-operation \
  --file atomic_tx.xdr \
  --invoke-contract \
  --id CONTRACT_ID \
  --function approve \
  --arg-spender "GD5..." \
  --arg-amount "500"

# Execute atomically
soroban tx sign --file atomic_tx.xdr --source alice --network testnet
soroban tx submit --file atomic_tx.xdr --network testnet
```

### Conditional Operations

```bash
# Check condition before operation
balance=$(soroban contract invoke \
  --id CONTRACT_ID \
  --function get_balance \
  --arg-user "$(soroban config identity address alice)" \
  --source alice \
  --network testnet \
  --dry-run | jq -r '.result')

if [ "$balance" -gt "1000" ]; then
  soroban contract invoke \
    --id CONTRACT_ID \
    --function transfer \
    --arg-to "GB7T..." \
    --arg-amount "1000" \
    --source alice \
    --network testnet
else
  echo "Insufficient balance for transfer"
fi
```

### Cross-Contract Operations

```bash
# Call multiple contracts in sequence
# First contract: Token contract
token_result=$(soroban contract invoke \
  --id TOKEN_CONTRACT_ID \
  --function transfer \
  --arg-to "CONTRACT_ADDRESS" \
  --arg-amount "1000" \
  --source alice \
  --network testnet)

# Second contract: Target contract
if echo "$token_result" | grep -q "success"; then
  soroban contract invoke \
    --id TARGET_CONTRACT_ID \
    --function purchase \
    --arg-item "item_id" \
    --arg-amount "1000" \
    --source alice \
    --network testnet
fi
```

### Monitoring and Automation

```bash
# Monitor contract activity
monitor_contract() {
  local contract_id=$1
  local network=$2
  local interval=$3
  
  while true; do
    echo "[$(date)] Checking contract activity..."
    
    # Get latest ledger
    latest_ledger=$(soroban info ledger --network $network | jq -r '.sequence')
    
    # Get recent events
    events=$(soroban contract events \
      --id $contract_id \
      --network $network \
      --start-ledger $((latest_ledger - 10)) \
      --end-ledger $latest_ledger)
    
    if [ "$events" != "null" ] && [ "$events" != "" ]; then
      echo "Recent events:"
      echo "$events" | jq '.events[] | {type: .type, data: .body}'
    fi
    
    sleep $interval
  done
}

# Start monitoring
monitor_contract $CONTRACT_ID testnet 60
```

### Performance Optimization

```bash
# Batch read operations
batch_read() {
  local contract_id=$1
  local keys=("${@:2}")
  
  for key in "${keys[@]}"; do
    soroban contract read \
      --id $contract_id \
      --key "$key" \
      --network testnet &
  done
  
  wait
}

# Execute batch reads
batch_read $CONTRACT_ID "balance:alice" "balance:bob" "config" "total_supply"
```

---

This guide provides comprehensive coverage of Soroban CLI interaction patterns. For deployment guidance, see the [Deployment Guide](./SOROBAN_CLI_DEPLOYMENT.md), and for development workflows, see the [Development Guide](./SOROBAN_CLI_DEVELOPMENT.md).
