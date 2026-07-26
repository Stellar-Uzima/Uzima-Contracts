# Soroban CLI Examples and Best Practices

This comprehensive guide provides practical examples and best practices for using the Soroban CLI in various scenarios.

## Table of Contents

1. [Quick Start Examples](#quick-start-examples)
2. [Common Patterns](#common-patterns)
3. [Advanced Examples](#advanced-examples)
4. [Best Practices](#best-practices)
5. [Troubleshooting Guide](#troubleshooting-guide)
6. [Performance Tips](#performance-tips)

## Quick Start Examples

### Deploy Contract

```bash
# Deploy contract to local network
soroban contract deploy \
    --wasm ./contract.wasm \
    --source alice \
    --network local

# Deploy contract to testnet
soroban contract deploy \
    --wasm ./contract.wasm \
    --source alice \
    --network testnet

# Deploy contract to mainnet (with safety checks)
soroban contract deploy \
    --wasm ./contract_optimized.wasm \
    --source alice \
    --network mainnet \
    --fee 1000
```

### Invoke Function

```bash
# Simple function invocation
soroban contract invoke \
    --id CONTRACT_ID \
    --function transfer \
    --arg-to RECIPIENT \
    --arg-amount 1000 \
    --source alice \
    --network testnet

# Function with complex arguments
soroban contract invoke \
    --id CONTRACT_ID \
    --function create_record \
    --arg-patient-id "12345" \
    --arg-doctor-id "67890" \
    --arg-diagnosis "Hypertension" \
    --arg-treatment "Lifestyle changes" \
    --arg-is-chronic true \
    --source doctor \
    --network testnet

# Read-only query
soroban contract invoke \
    --id CONTRACT_ID \
    --function get_balance \
    --arg-user alice \
    --source alice \
    --network testnet \
    --dry-run
```

## Common Patterns

### Identity Management

```bash
# Create identities for different environments
soroban config identity generate dev-admin
soroban config identity generate test-admin  
soroban config identity generate prod-admin

# List all identities
soroban config identity list

# Show identity details
soroban config identity show dev-admin

# Get identity address
soroban config identity address dev-admin
```

### Network Configuration

```bash
# Add local network
soroban config network add --global local \
    --rpc-url http://localhost:8000/soroban/rpc \
    --network-passphrase "Standalone Network ; February 2017"

# Add testnet network
soroban config network add --global testnet \
    --rpc-url https://soroban-testnet.stellar.org:443 \
    --network-passphrase "Test SDF Network ; September 2015"

# Add mainnet network
soroban config network add --global mainnet \
    --rpc-url https://soroban-rpc.mainnet.stellar.org:443 \
    --network-passphrase "Public Global Stellar Network ; September 2015"

# List configured networks
soroban config network list
```

### Batch Operations

```bash
# Deploy multiple contracts
contracts=("medical_records" "identity_registry" "payment_router")
for contract in "${contracts[@]}"; do
    echo "Deploying $contract..."
    soroban contract deploy \
        --wasm target/wasm32-unknown-unknown/release/$contract.wasm \
        --source admin \
        --network testnet
done

# Batch function calls
cat > batch_calls.txt <<EOF
transfer GB7TAYUEQH6TZTYNNO6R7JQ2GTGJZYJSTZQYV2MYQMT2E2D2BZJQQQY 1000
approve GD5D2B2F4G4J4K4L4M4N4O4P4Q4R4S4T4U4V4W4X4Y4Z5A5B5C5D5E5F5G5H5 500
get_balance
EOF

while IFS= read -r line; do
    if [[ $line =~ ^#.*$ ]] || [[ -z $line ]]; then
        continue
    fi
    read -ra parts <<< "$line"
    function="${parts[0]}"
    args=("${parts[@]:1}")
    
    cmd="soroban contract invoke --id $CONTRACT_ID --function $function --source alice --network testnet"
    for arg in "${args[@]}"; do
        cmd+=" --arg-$arg"
    done
    
    echo "Executing: $cmd"
    eval $cmd
done < batch_calls.txt
```

## Advanced Examples

### Contract Upgrade Pattern

```bash
# Deploy version 1
echo "Deploying contract v1..."
CONTRACT_V1=$(soroban contract deploy \
    --wasm contract_v1.wasm \
    --source admin \
    --network testnet)

# Initialize v1
soroban contract invoke \
    --id $CONTRACT_V1 \
    --function initialize \
    --arg-admin "$(soroban config identity address admin)" \
    --source admin \
    --network testnet

# Add some data
soroban contract invoke \
    --id $CONTRACT_V1 \
    --function add_record \
    --arg-id "1" \
    --arg-data "test data" \
    --source admin \
    --network testnet

# Deploy version 2
echo "Deploying contract v2..."
CONTRACT_V2=$(soroban contract deploy \
    --wasm contract_v2.wasm \
    --source admin \
    --network testnet)

# Migrate data from v1 to v2
soroban contract invoke \
    --id $CONTRACT_V2 \
    --function migrate_from_v1 \
    --arg-old-contract $CONTRACT_V1 \
    --source admin \
    --network testnet

# Verify migration
soroban contract invoke \
    --id $CONTRACT_V2 \
    --function get_record \
    --arg-id "1" \
    --source admin \
    --network testnet \
    --dry-run
```

### Cross-Contract Interaction

```bash
# Deploy token contract
TOKEN_CONTRACT=$(soroban contract deploy \
    --wasm token.wasm \
    --source admin \
    --network testnet)

# Initialize token
soroban contract invoke \
    --id $TOKEN_CONTRACT \
    --function initialize \
    --arg-name "Test Token" \
    --arg-symbol "TEST" \
    --arg-decimals 18 \
    --arg-admin "$(soroban config identity address admin)" \
    --source admin \
    --network testnet

# Deploy marketplace contract
MARKETPLACE_CONTRACT=$(soroban contract deploy \
    --wasm marketplace.wasm \
    --source admin \
    --network testnet)

# Initialize marketplace with token
soroban contract invoke \
    --id $MARKETPLACE_CONTRACT \
    --function initialize \
    --arg-token-contract $TOKEN_CONTRACT \
    --arg-admin "$(soroban config identity address admin)" \
    --source admin \
    --network testnet

# Create marketplace listing
soroban contract invoke \
    --id $MARKETPLACE_CONTRACT \
    --function create_listing \
    --arg-item-id "item123" \
    --arg-price "1000000000000000000" \
    --source seller \
    --network testnet

# Purchase item (cross-contract call)
soroban contract invoke \
    --id $MARKETPLACE_CONTRACT \
    --function purchase \
    --arg-item-id "item123" \
    --source buyer \
    --network testnet
```

### Event Monitoring

```bash
# Monitor contract events
monitor_events() {
    local contract_id=$1
    local network=$2
    local duration=$3
    
    echo "Monitoring events for contract: $contract_id"
    echo "Network: $network"
    echo "Duration: ${duration}s"
    
    start_time=$(date +%s)
    end_time=$((start_time + duration))
    
    while [ $(date +%s) -lt $end_time ]; do
        current_ledger=$(soroban info ledger --network $network | jq -r '.sequence')
        start_ledger=$((current_ledger - 10))
        
        events=$(soroban contract events \
            --id $contract_id \
            --network $network \
            --start-ledger $start_ledger \
            --end-ledger $current_ledger)
        
        if [ "$events" != "null" ] && [ "$events" != "" ]; then
            echo "[$(date)] New events:"
            echo "$events" | jq -r '.events[] | "Type: \(.type), Data: \(.body)"'
        fi
        
        sleep 5
    done
}

# Usage
monitor_events $CONTRACT_ID testnet 300
```

### Transaction Building

```bash
# Build complex transaction
build_complex_transaction() {
    local contract_id=$1
    local source=$2
    local network=$3
    
    # Create new transaction
    soroban tx new --source $source --network $network > complex_tx.xdr
    
    # Add multiple operations
    operations=(
        "transfer recipient1 1000"
        "approve recipient2 500"
        "transfer recipient3 200"
    )
    
    for op in "${operations[@]}"; do
        read -ra parts <<< "$op"
        function="${parts[0]}"
        args=("${parts[@]:1}")
        
        cmd="soroban tx add-operation --file complex_tx.xdr --invoke-contract --id $contract_id --function $function"
        for arg in "${args[@]}"; do
            cmd+=" --arg-$arg"
        done
        
        eval $cmd
    done
    
    # Set transaction options
    soroban tx set-options \
        --file complex_tx.xdr \
        --fee 2000 \
        --memo "Complex batch operation"
    
    # Sign and submit
    soroban tx sign --file complex_tx.xdr --source $source --network $network
    result=$(soroban tx submit --file complex_tx.xdr --network $network)
    
    echo "Transaction submitted: $result"
    rm complex_tx.xdr
}

# Usage
build_complex_transaction $CONTRACT_ID alice testnet
```

## Best Practices

### Security Practices

#### 1. Use Environment-Specific Identities

```bash
# Create separate identities for each environment
soroban config identity generate dev-admin
soroban config identity generate test-admin
soroban config identity generate prod-admin

# Use descriptive names
soroban config identity generate medical-records-admin
soroban config identity generate payment-router-operator

# Regularly rotate production identities
soroban config identity generate prod-admin-$(date +%Y%m)
```

#### 2. Validate Before Mainnet Deployment

```bash
# Pre-deployment validation script
validate_deployment() {
    local contract_name=$1
    local wasm_file=$2
    
    echo "Validating $contract_name deployment..."
    
    # Check WASM file exists
    if [ ! -f "$wasm_file" ]; then
        echo "❌ WASM file not found: $wasm_file"
        return 1
    fi
    
    # Check WASM size
    wasm_size=$(stat -c%s "$wasm_file")
    max_size=$((2 * 1024 * 1024))  # 2MB limit
    
    if [ $wasm_size -gt $max_size ]; then
        echo "❌ WASM file too large: ${wasm_size} bytes (max: ${max_size} bytes)"
        return 1
    fi
    
    # Test on local network
    echo "Testing on local network..."
    if ! soroban contract deploy --wasm "$wasm_file" --source dev-admin --network local --dry-run; then
        echo "❌ Local deployment test failed"
        return 1
    fi
    
    # Test on testnet
    echo "Testing on testnet..."
    if ! soroban contract deploy --wasm "$wasm_file" --source test-admin --network testnet --dry-run; then
        echo "❌ Testnet deployment test failed"
        return 1
    fi
    
    echo "✅ Validation passed"
    return 0
}

# Usage
validate_deployment medical_records contract.wasm
```

#### 3. Use Deterministic Deployment

```bash
# Deploy with fixed salt for predictable address
deploy_deterministic() {
    local wasm_file=$1
    local salt=$2
    local network=$3
    local source=$4
    
    contract_id=$(soroban contract deploy \
        --wasm "$wasm_file" \
        --salt "$salt" \
        --source "$source" \
        --network "$network")
    
    echo "Contract deployed with deterministic address: $contract_id"
    echo "$contract_id" > deterministic_address.txt
    
    return 0
}

# Usage
deploy_deterministic contract.wasm "fixed-salt-2024" testnet admin
```

### Performance Optimization

#### 1. Optimize WASM Files

```bash
# Build optimization script
optimize_contract() {
    local contract_name=$1
    
    echo "Optimizing $contract_name..."
    
    # Clean build
    cargo clean -p $contract_name
    
    # Build with optimizations
    RUSTFLAGS='-C opt-level=s -C lto=fat' \
    cargo build -p $contract_name --target wasm32-unknown-unknown --release
    
    # Optimize with Soroban CLI
    soroban contract optimize \
        --wasm target/wasm32-unknown-unknown/release/$contract_name.wasm \
        --output target/wasm32-unknown-unknown/release/$contract_name_optimized.wasm \
        --level 3
    
    # Compare sizes
    original_size=$(stat -c%s target/wasm32-unknown-unknown/release/$contract_name.wasm)
    optimized_size=$(stat -c%s target/wasm32-unknown-unknown/release/$contract_name_optimized.wasm)
    
    reduction=$((original_size - optimized_size))
    percentage=$((reduction * 100 / original_size))
    
    echo "Original size: $original_size bytes"
    echo "Optimized size: $optimized_size bytes"
    echo "Size reduction: $reduction bytes ($percentage%)"
}

# Usage
optimize_contract medical_records
```

#### 2. Batch Operations

```bash
# Batch operation executor
execute_batch() {
    local contract_id=$1
    local operations_file=$2
    local network=$3
    local source=$4
    
    echo "Executing batch operations from $operations_file"
    
    success_count=0
    failure_count=0
    
    while IFS= read -r line; do
        if [[ $line =~ ^#.*$ ]] || [[ -z $line ]]; then
            continue
        fi
        
        read -ra parts <<< "$line"
        function="${parts[0]}"
        args=("${parts[@]:1}")
        
        cmd="soroban contract invoke --id $contract_id --function $function --source $source --network $network"
        for arg in "${args[@]}"; do
            cmd+=" --arg-$arg"
        done
        
        echo "Executing: $function"
        if eval $cmd; then
            ((success_count++))
            echo "✅ Success"
        else
            ((failure_count++))
            echo "❌ Failed"
        fi
    done < "$operations_file"
    
    echo "Batch execution completed:"
    echo "Success: $success_count"
    echo "Failed: $failure_count"
    
    return $failure_count
}

# Usage
execute_batch $CONTRACT_ID batch_operations.txt testnet alice
```

### Development Workflow

#### 1. Local Development Setup

```bash
# Development environment setup
setup_dev_env() {
    echo "Setting up development environment..."
    
    # Start local network
    echo "Starting local Stellar network..."
    docker run --rm -d -p 8000:8000 \
        --name soroban-dev \
        stellar/soroban-rpc:latest \
        --port 8000
    
    sleep 5
    
    # Create development identities
    echo "Creating development identities..."
    soroban config identity generate dev-admin
    soroban config identity generate dev-user1
    soroban config identity generate dev-user2
    
    # Fund accounts
    echo "Funding development accounts..."
    soroban keys fund dev-admin --network local
    soroban keys fund dev-user1 --network local
    soroban keys fund dev-user2 --network local
    
    # Deploy contracts
    echo "Deploying contracts..."
    contracts=("medical_records" "identity_registry" "payment_router")
    for contract in "${contracts[@]}"; do
        soroban contract deploy \
            --wasm target/wasm32-unknown-unknown/release/$contract.wasm \
            --source dev-admin \
            --network local
    done
    
    echo "✅ Development environment ready"
}

# Usage
setup_dev_env
```

#### 2. Testing Workflow

```bash
# Comprehensive testing workflow
run_tests() {
    local network=$1
    
    echo "Running comprehensive tests on $network"
    
    # 1. Unit tests
    echo "1. Running unit tests..."
    if ! cargo test; then
        echo "❌ Unit tests failed"
        return 1
    fi
    
    # 2. Integration tests
    echo "2. Running integration tests..."
    if ! make test-integration; then
        echo "❌ Integration tests failed"
        return 1
    fi
    
    # 3. Contract deployment tests
    echo "3. Testing contract deployments..."
    contracts=("medical_records" "identity_registry" "payment_router")
    for contract in "${contracts[@]}"; do
        if ! soroban contract deploy \
            --wasm target/wasm32-unknown-unknown/release/$contract.wasm \
            --source test-admin \
            --network $network \
            --dry-run; then
            echo "❌ Deployment test failed for $contract"
            return 1
        fi
    done
    
    # 4. Performance tests
    echo "4. Running performance tests..."
    if ! ./scripts/profile_functions.sh $network performance_report.csv; then
        echo "❌ Performance tests failed"
        return 1
    fi
    
    echo "✅ All tests passed"
    return 0
}

# Usage
run_tests testnet
```

## Troubleshooting Guide

### Common Issues and Solutions

#### 1. Network Connectivity Issues

```bash
# Diagnose network issues
diagnose_network() {
    local network=$1
    
    echo "Diagnosing $network network issues..."
    
    # Test RPC connectivity
    case $network in
        "local")
            rpc_url="http://localhost:8000/soroban/rpc"
            ;;
        "testnet")
            rpc_url="https://soroban-testnet.stellar.org:443"
            ;;
        "mainnet")
            rpc_url="https://soroban-rpc.mainnet.stellar.org:443"
            ;;
    esac
    
    echo "Testing RPC endpoint: $rpc_url"
    if curl -s -X POST -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' \
        "$rpc_url" | grep -q "healthy"; then
        echo "✅ RPC endpoint is healthy"
    else
        echo "❌ RPC endpoint is not responding"
        return 1
    fi
    
    # Check network configuration
    echo "Checking network configuration..."
    soroban config network show $network || echo "Network not configured"
    
    # Test account access
    echo "Testing account access..."
    if soroban account info --identity test-admin --network $network >/dev/null 2>&1; then
        echo "✅ Account access working"
    else
        echo "❌ Account access failed"
        return 1
    fi
    
    return 0
}

# Usage
diagnose_network testnet
```

#### 2. Identity Issues

```bash
# Fix identity problems
fix_identity() {
    local identity_name=$1
    local network=$2
    
    echo "Fixing identity: $identity_name"
    
    # Check if identity exists
    if ! soroban config identity show $identity_name >/dev/null 2>&1; then
        echo "Identity not found, creating new identity..."
        soroban config identity generate $identity_name
    fi
    
    # Get identity address
    address=$(soroban config identity address $identity_name)
    echo "Identity address: $address"
    
    # Check if account is funded
    if soroban account info --identity $identity_name --network $network >/dev/null 2>&1; then
        echo "✅ Account is funded"
    else
        echo "Account not funded, attempting to fund..."
        if [ "$network" = "testnet" ] || [ "$network" = "local" ]; then
            soroban keys fund $identity_name --network $network
        else
            echo "❌ Cannot auto-fund $network. Please fund manually."
            return 1
        fi
    fi
    
    return 0
}

# Usage
fix_identity test-admin testnet
```

#### 3. Build Issues

```bash
# Fix build problems
fix_build() {
    local contract_name=$1
    
    echo "Fixing build issues for $contract_name"
    
    # Clean build artifacts
    echo "Cleaning build artifacts..."
    cargo clean -p $contract_name
    
    # Update dependencies
    echo "Updating dependencies..."
    cargo update -p $contract_name
    
    # Check for compilation errors
    echo "Checking compilation..."
    if cargo check -p $contract_name --target wasm32-unknown-unknown; then
        echo "✅ Compilation check passed"
    else
        echo "❌ Compilation errors found"
        return 1
    fi
    
    # Build with verbose output
    echo "Building with verbose output..."
    cargo build -p $contract_name --target wasm32-unknown-unknown --release --verbose
    
    return 0
}

# Usage
fix_build medical_records
```

## Performance Tips

### 1. Reduce Transaction Costs

```bash
# Optimize transaction fees
optimize_fees() {
    local contract_id=$1
    local function=$2
    local network=$3
    
    echo "Optimizing fees for $function"
    
    # Test different fee levels
    for fee in 100 500 1000 2000; do
        echo "Testing fee level: $fee"
        
        start_time=$(date +%s.%N)
        result=$(soroban contract invoke \
            --id $contract_id \
            --function $function \
            --source test-admin \
            --network $network \
            --fee $fee \
            --dry-run 2>/dev/null)
        
        end_time=$(date +%s.%N)
        duration=$(echo "$end_time - $start_time" | bc)
        
        if [ $? -eq 0 ]; then
            echo "✅ Success - Fee: $fee, Duration: ${duration}s"
            break
        else
            echo "❌ Failed - Fee: $fee"
        fi
    done
}

# Usage
optimize_fees $CONTRACT_ID expensive_function testnet
```

### 2. Batch Operations for Efficiency

```bash
# Efficient batch processing
efficient_batch() {
    local contract_id=$1
    local operations_file=$2
    local network=$3
    local source=$4
    local batch_size=$5
    
    echo "Processing operations in batches of $batch_size"
    
    temp_file=$(mktemp)
    line_count=0
    
    while IFS= read -r line; do
        if [[ $line =~ ^#.*$ ]] || [[ -z $line ]]; then
            continue
        fi
        
        echo "$line" >> "$temp_file"
        ((line_count++))
        
        if [ $((line_count % batch_size)) -eq 0 ]; then
            echo "Processing batch ($((line_count / batch_size)))..."
            execute_batch $contract_id "$temp_file" $network $source
            > "$temp_file"
        fi
    done < "$operations_file"
    
    # Process remaining operations
    if [ -s "$temp_file" ]; then
        echo "Processing final batch..."
        execute_batch $contract_id "$temp_file" $network $source
    fi
    
    rm "$temp_file"
    echo "Batch processing completed"
}

# Usage
efficient_batch $CONTRACT_ID large_batch.txt testnet alice 10
```

### 3. Caching and Optimization

```bash
# Cache contract information
cache_contract_info() {
    local contract_id=$1
    local network=$2
    local cache_file=$3
    
    echo "Caching contract information..."
    
    # Get contract info
    info=$(soroban contract info --id $contract_id --network $network)
    
    # Cache the information
    echo "$info" > "$cache_file"
    
    # Extract useful information
    wasm_hash=$(echo "$info" | jq -r '.wasm_hash')
    ledger_created=$(echo "$info" | jq -r '.created_ledger')
    
    echo "Contract info cached:"
    echo "WASM Hash: $wasm_hash"
    echo "Created Ledger: $ledger_created"
    echo "Cache file: $cache_file"
}

# Usage
cache_contract_info $CONTRACT_ID testnet contract_cache.json
```

---

This examples guide provides practical patterns and solutions for common Soroban CLI usage scenarios. For comprehensive documentation, see the [Deployment Guide](./SOROBAN_CLI_DEPLOYMENT.md), [Interaction Guide](./SOROBAN_CLI_INTERACTION.md), and [Development Guide](./SOROBAN_CLI_DEVELOPMENT.md).
