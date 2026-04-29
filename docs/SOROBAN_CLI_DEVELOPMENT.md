# Soroban CLI Development Guide

This comprehensive guide covers development workflows, testing, debugging, and performance profiling using the Soroban CLI.

## Table of Contents

1. [Testing Commands](#testing-commands)
2. [Build Optimization](#build-optimization)
3. [Debug Techniques](#debug-techniques)
4. [Performance Profiling](#performance-profiling)
5. [Local Development Setup](#local-development-setup)
6. [Integration Testing](#integration-testing)
7. [Continuous Integration](#continuous-integration)

## Testing Commands

### Unit Testing

```bash
# Run all tests
cargo test

# Run tests for specific contract
cargo test -p medical_records

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_function_name

# Run tests in release mode
cargo test --release
```

### Contract Testing with Soroban CLI

```bash
# Test contract deployment locally
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/contract.wasm \
  --source test_admin \
  --network local

# Test contract functions
soroban contract invoke \
  --id $CONTRACT_ID \
  --function test_function \
  --source test_admin \
  --network local \
  --dry-run

# Run comprehensive test suite
./scripts/test_contract.sh local
```

### Integration Testing

```bash
# Setup test environment
make start-local

# Run integration tests
make test-integration

# Run specific integration test
cargo test -p integration_tests test_contract_interaction

# Cleanup test environment
make stop-local
```

### Test Data Management

```bash
# Generate test data
./scripts/generate_test_data.sh --network local --contracts medical_records,identity_registry

# Load test fixtures
soroban contract invoke \
  --id $CONTRACT_ID \
  --function load_test_data \
  --source test_admin \
  --network local

# Cleanup test data
soroban contract invoke \
  --id $CONTRACT_ID \
  --function cleanup_test_data \
  --source test_admin \
  --network local
```

## Build Optimization

### Optimized Builds

```bash
# Standard optimized build
cargo build --target wasm32-unknown-unknown --release

# Build with size optimization
RUSTFLAGS='-C opt-level=s' cargo build --target wasm32-unknown-unknown --release

# Build with performance optimization
RUSTFLAGS='-C opt-level=3' cargo build --target wasm32-unknown-unknown --release

# Build with link-time optimization
RUSTFLAGS='-C lto=fat' cargo build --target wasm32-unknown-unknown --release
```

### WASM Optimization

```bash
# Optimize WASM file with Soroban CLI
soroban contract optimize \
  --wasm target/wasm32-unknown-unknown/release/contract.wasm \
  --output target/wasm32-unknown-unknown/release/contract_optimized.wasm

# Optimize with specific level
soroban contract optimize \
  --wasm target/wasm32-unknown-unknown/release/contract.wasm \
  --output target/wasm32-unknown-unknown/release/contract_optimized.wasm \
  --level 3

# Compare file sizes
ls -lh target/wasm32-unknown-unknown/release/contract*.wasm
```

### Dependency Optimization

```bash
# Use only required dependencies
cargo tree -p medical_records

# Remove unused dependencies
cargo machete

# Update dependencies for performance
cargo update -p soroban-sdk

# Check for large dependencies
du -sh target/wasm32-unknown-unknown/release/deps/
```

### Build Performance

```bash
# Use build cache
export CARGO_TARGET_DIR=/tmp/soroban-build-cache

# Parallel builds
cargo build --target wasm32-unknown-unknown --release -j $(nproc)

# Incremental builds
cargo build --target wasm32-unknown-unknown

# Clean build
cargo clean && cargo build --target wasm32-unknown-unknown --release
```

## Debug Techniques

### Debug Builds

```bash
# Build with debug symbols
cargo build --target wasm32-unknown-unknown

# Build with debug info
cargo build --target wasm32-unknown-unknown --features debug

# Verbose build output
cargo build --target wasm32-unknown-unknown --verbose
```

### Contract Debugging

```bash
# Debug contract invocation
soroban contract invoke \
  --id $CONTRACT_ID \
  --function debug_function \
  --source test_admin \
  --network local \
  --debug

# Enable detailed logging
RUST_LOG=debug soroban contract invoke \
  --id $CONTRACT_ID \
  --function test_function \
  --source test_admin \
  --network local

# Capture debug output
soroban contract invoke \
  --id $CONTRACT_ID \
  --function test_function \
  --source test_admin \
  --network local \
  2>&1 | tee debug.log
```

### Error Analysis

```bash
# Parse transaction errors
soroban tx info \
  --hash TRANSACTION_HASH \
  --network local \
  | jq '.result.meta.result'

# Analyze failure reasons
soroban contract invoke \
  --id $CONTRACT_ID \
  --function failing_function \
  --source test_admin \
  --network local \
  --dry-run \
  | jq '.simulation.error'

# Check contract state after error
soroban contract info --id $CONTRACT_ID --network local
```

### Step-by-Step Debugging

```bash
# Create debug script
cat > debug_contract.sh <<'EOF'
#!/bin/bash

CONTRACT_ID=$1
NETWORK=$2
FUNCTION=$3

echo "Debugging contract: $CONTRACT_ID"
echo "Network: $NETWORK"
echo "Function: $FUNCTION"

# Check contract exists
echo "1. Checking contract exists..."
soroban contract info --id $CONTRACT_ID --network $NETWORK

# Check caller identity
echo "2. Checking caller identity..."
soroban config identity show debug_user

# Dry run invocation
echo "3. Dry run invocation..."
soroban contract invoke \
  --id $CONTRACT_ID \
  --function $FUNCTION \
  --source debug_user \
  --network $NETWORK \
  --dry-run

# Actual invocation with debug
echo "4. Actual invocation..."
soroban contract invoke \
  --id $CONTRACT_ID \
  --function $FUNCTION \
  --source debug_user \
  --network $NETWORK \
  --debug
EOF

chmod +x debug_contract.sh

# Use debug script
./debug_contract.sh $CONTRACT_ID local test_function
```

### Memory and Gas Analysis

```bash
# Analyze gas usage
soroban contract invoke \
  --id $CONTRACT_ID \
  --function expensive_function \
  --source test_admin \
  --network local \
  --dry-run \
  | jq '.simulation.transaction_resource_fee'

# Memory usage analysis
soroban contract invoke \
  --id $CONTRACT_ID \
  --function memory_intensive_function \
  --source test_admin \
  --network local \
  --dry-run \
  | jq '.simulation.memory_bytes'

# Compare function costs
for function in function1 function2 function3; do
  cost=$(soroban contract invoke \
    --id $CONTRACT_ID \
    --function $function \
    --source test_admin \
    --network local \
    --dry-run \
    | jq '.simulation.transaction_resource_fee')
  echo "$function: $cost"
done
```

## Performance Profiling

### Gas Profiling

```bash
# Profile all contract functions
cat > profile_functions.sh <<'EOF'
#!/bin/bash

CONTRACT_ID=$1
NETWORK=$2
OUTPUT_FILE=$3

echo "Function,ResourceFee,MemoryBytes,CpuInstructions" > $OUTPUT_FILE

functions=$(soroban contract inspect --wasm contract.wasm | jq -r '.spec.functions[].name')

for function in $functions; do
  result=$(soroban contract invoke \
    --id $CONTRACT_ID \
    --function $function \
    --source test_admin \
    --network $NETWORK \
    --dry-run 2>/dev/null)
  
  if [ $? -eq 0 ]; then
    fee=$(echo $result | jq -r '.simulation.transaction_resource_fee // "N/A"')
    memory=$(echo $result | jq -r '.simulation.memory_bytes // "N/A"')
    cpu=$(echo $result | jq -r '.simulation.cpu_instructions // "N/A"')
    echo "$function,$fee,$memory,$cpu" >> $OUTPUT_FILE
  else
    echo "$function,ERROR,ERROR,ERROR" >> $OUTPUT_FILE
  fi
done
EOF

chmod +x profile_functions.sh

# Run profiling
./profile_functions.sh $CONTRACT_ID local performance_profile.csv
```

### Benchmarking

```bash
# Create benchmark script
cat > benchmark_contract.sh <<'EOF'
#!/bin/bash

CONTRACT_ID=$1
NETWORK=$2
ITERATIONS=$3
FUNCTION=$4

echo "Benchmarking $FUNCTION with $ITERATIONS iterations..."

total_time=0
for i in $(seq 1 $ITERATIONS); do
  start_time=$(date +%s.%N)
  
  soroban contract invoke \
    --id $CONTRACT_ID \
    --function $FUNCTION \
    --source test_admin \
    --network $NETWORK \
    --dry-run >/dev/null 2>&1
  
  end_time=$(date +%s.%N)
  duration=$(echo "$end_time - $start_time" | bc)
  total_time=$(echo "$total_time + $duration" | bc)
  
  echo "Iteration $i: ${duration}s"
done

average_time=$(echo "scale=6; $total_time / $ITERATIONS" | bc)
echo "Average time: ${average_time}s"
echo "Total time: ${total_time}s"
EOF

chmod +x benchmark_contract.sh

# Run benchmark
./benchmark_contract.sh $CONTRACT_ID local 10 get_balance
```

### Load Testing

```bash
# Concurrent invocation testing
cat > load_test.sh <<'EOF'
#!/bin/bash

CONTRACT_ID=$1
NETWORK=$2
CONCURRENT_REQUESTS=$3
FUNCTION=$4

echo "Load testing: $CONCURRENT_REQUESTS concurrent requests"

pids=()
for i in $(seq 1 $CONCURRENT_REQUESTS); do
  (
    soroban contract invoke \
      --id $CONTRACT_ID \
      --function $FUNCTION \
      --source test_admin$i \
      --network $NETWORK \
      --dry-run \
      > "load_test_$i.log" 2>&1
    echo "Request $i completed"
  ) &
  pids+=($!)
done

# Wait for all requests to complete
for pid in "${pids[@]}"; do
  wait $pid
done

# Analyze results
successful=0
failed=0
for i in $(seq 1 $CONCURRENT_REQUESTS); do
  if grep -q "success" "load_test_$i.log"; then
    ((successful++))
  else
    ((failed++))
  fi
done

echo "Successful: $successful"
echo "Failed: $failed"
echo "Success rate: $(echo "scale=2; $successful * 100 / $CONCURRENT_REQUESTS" | bc)%"
EOF

chmod +x load_test.sh

# Run load test
./load_test.sh $CONTRACT_ID local 5 get_balance
```

## Local Development Setup

### Quick Start Environment

```bash
# Start local development environment
make start-local

# Create development identities
soroban config identity generate dev-admin
soroban config identity generate dev-user1
soroban config identity generate dev-user2

# Fund development accounts
soroban keys fund dev-admin --network local
soroban keys fund dev-user1 --network local
soroban keys fund dev-user2 --network local

# Deploy contracts for development
./scripts/deploy_contracts.sh local
```

### Development Workflow Script

```bash
# Create development workflow
cat > dev_workflow.sh <<'EOF'
#!/bin/bash

CONTRACT_NAME=$1
NETWORK=${2:-local}

echo "Development workflow for $CONTRACT_NAME on $NETWORK"

# 1. Clean build
echo "1. Cleaning and building..."
make clean
make build-opt

# 2. Deploy contract
echo "2. Deploying contract..."
CONTRACT_ID=$(./scripts/deploy_enhanced.sh $CONTRACT_NAME $NETWORK --identity dev-admin)
echo "Contract deployed: $CONTRACT_ID"

# 3. Run tests
echo "3. Running tests..."
./scripts/test_contract.sh $CONTRACT_ID $NETWORK

# 4. Setup test data
echo "4. Setting up test data..."
soroban contract invoke \
  --id $CONTRACT_ID \
  --function setup_test_data \
  --source dev-admin \
  --network $NETWORK

# 5. Run integration tests
echo "5. Running integration tests..."
make test-integration

echo "Development workflow completed!"
EOF

chmod +x dev_workflow.sh

# Use workflow
./dev_workflow.sh medical_records local
```

### Hot Reload Development

```bash
# Create hot reload script
cat > hot_reload.sh <<'EOF'
#!/bin/bash

CONTRACT_NAME=$1
NETWORK=${2:-local}
IDENTITY=${3:-dev-admin}

echo "Hot reload for $CONTRACT_NAME"

# Watch for changes and redeploy
while true; do
  # Check if WASM file changed
  current_hash=$(sha256sum target/wasm32-unknown-unknown/release/$CONTRACT_NAME.wasm)
  
  if [ "$current_hash" != "$last_hash" ]; then
    echo "Changes detected, redeploying..."
    
    # Deploy new version
    NEW_CONTRACT_ID=$(./scripts/deploy_enhanced.sh $CONTRACT_NAME $NETWORK --identity $IDENTITY)
    
    # Migrate data if needed
    if [ -n "$CONTRACT_ID" ]; then
      ./scripts/migrate_contract.sh $CONTRACT_ID $NEW_CONTRACT_ID $NETWORK
    fi
    
    CONTRACT_ID=$NEW_CONTRACT_ID
    last_hash=$current_hash
    
    echo "Contract redeployed: $CONTRACT_ID"
  fi
  
  sleep 2
done
EOF

chmod +x hot_reload.sh

# Start hot reload
./hot_reload.sh medical_records local
```

## Integration Testing

### End-to-End Testing

```bash
# Create comprehensive test suite
cat > e2e_test.sh <<'EOF'
#!/bin/bash

NETWORK=$1
CONTRACTS=${2:-"medical_records,identity_registry,payment_router"}

echo "Running E2E tests on $NETWORK"

# Convert contracts to array
IFS=',' read -ra CONTRACT_ARRAY <<< "$CONTRACTS"
CONTRACT_IDS=()

# Deploy all contracts
echo "Deploying contracts..."
for contract in "${CONTRACT_ARRAY[@]}"; do
  contract_id=$(./scripts/deploy_enhanced.sh $contract $NETWORK --identity test-admin)
  CONTRACT_IDS+=("$contract_id")
  echo "Deployed $contract: $contract_id"
done

# Test contract interactions
echo "Testing contract interactions..."

# Test medical records
medical_id=${CONTRACT_IDS[0]}
soroban contract invoke \
  --id $medical_id \
  --function add_record \
  --arg-patient "patient123" \
  --arg-doctor "doctor456" \
  --arg-diagnosis "Test diagnosis" \
  --source test-admin \
  --network $NETWORK

# Test identity registry
identity_id=${CONTRACT_IDS[1]}
soroban contract invoke \
  --id $identity_id \
  --function register_identity \
  --arg-address "$(soroban config identity address test-admin)" \
  --arg-identity-type "doctor" \
  --source test-admin \
  --network $NETWORK

# Test payment router
payment_id=${CONTRACT_IDS[2]}
soroban contract invoke \
  --id $payment_id \
  --function create_payment \
  --arg-recipient "$(soroban config identity address test-admin)" \
  --arg-amount "1000" \
  --source test-admin \
  --network $NETWORK

echo "E2E tests completed successfully!"
EOF

chmod +x e2e_test.sh

# Run E2E tests
./e2e_test.sh local
```

### Contract Migration Testing

```bash
# Test contract upgrade migration
cat > test_migration.sh <<'EOF'
#!/bin/bash

OLD_CONTRACT=$1
NEW_CONTRACT=$2
NETWORK=$3

echo "Testing migration from $OLD_CONTRACT to $NEW_CONTRACT"

# Deploy old contract
OLD_ID=$(./scripts/deploy_enhanced.sh $OLD_CONTRACT $NETWORK --identity test-admin)
echo "Old contract: $OLD_ID"

# Setup test data
soroban contract invoke \
  --id $OLD_ID \
  --function setup_migration_test \
  --source test-admin \
  --network $NETWORK

# Deploy new contract
NEW_ID=$(./scripts/deploy_enhanced.sh $NEW_CONTRACT $NETWORK --identity test-admin)
echo "New contract: $NEW_ID"

# Test migration
soroban contract invoke \
  --id $NEW_ID \
  --function migrate_from \
  --arg-old-contract $OLD_ID \
  --source test-admin \
  --network $NETWORK

# Verify migration
result=$(soroban contract invoke \
  --id $NEW_ID \
  --function verify_migration \
  --source test-admin \
  --network $NETWORK \
  --dry-run)

if echo "$result" | grep -q "true"; then
  echo "Migration successful!"
else
  echo "Migration failed!"
  exit 1
fi
EOF

chmod +x test_migration.sh

# Test migration
./test_migration.sh medical_records_v1 medical_records_v2 local
```

## Continuous Integration

### GitHub Actions Workflow

```yaml
# .github/workflows/soroban-ci.yml
name: Soroban CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: wasm32-unknown-unknown
          
      - name: Install Soroban CLI
        run: cargo install --locked soroban-cli
        
      - name: Start Local Network
        run: |
          docker run --rm -d -p 8000:8000 \
            stellar/soroban-rpc:latest \
            --port 8000
          sleep 10
          
      - name: Setup Identities
        run: |
          soroban config identity generate ci-admin
          soroban keys fund ci-admin --network local
          
      - name: Build Contracts
        run: make build-opt
        
      - name: Run Tests
        run: cargo test
        
      - name: Deploy Contracts
        run: |
          for contract in medical_records identity_registry payment_router; do
            ./scripts/deploy_enhanced.sh $contract local --identity ci-admin
          done
          
      - name: Run Integration Tests
        run: make test-integration
        
      - name: Performance Tests
        run: |
          ./scripts/profile_functions.sh local performance_report.csv
          
      - name: Upload Reports
        uses: actions/upload-artifact@v3
        with:
          name: performance-reports
          path: performance_report.csv
```

### Local CI Simulation

```bash
# Create local CI script
cat > local_ci.sh <<'EOF'
#!/bin/bash

echo "Running local CI simulation..."

# 1. Environment setup
echo "1. Setting up environment..."
make start-local
soroban config identity generate ci-admin
soroban keys fund ci-admin --network local

# 2. Build and test
echo "2. Building and testing..."
make clean
make build-opt
cargo test

# 3. Deploy contracts
echo "3. Deploying contracts..."
contracts=("medical_records" "identity_registry" "payment_router")
for contract in "${contracts[@]}"; do
  ./scripts/deploy_enhanced.sh $contract local --identity ci-admin
done

# 4. Integration tests
echo "4. Running integration tests..."
make test-integration

# 5. Performance profiling
echo "5. Running performance profiling..."
./scripts/profile_functions.sh local ci_performance.csv

# 6. Cleanup
echo "6. Cleaning up..."
make stop-local

echo "Local CI completed successfully!"
echo "Performance report: ci_performance.csv"
EOF

chmod +x local_ci.sh

# Run local CI
./local_ci.sh
```

### Quality Gates

```bash
# Create quality gate script
cat > quality_gate.sh <<'EOF'
#!/bin/bash

CONTRACT_NAME=$1
NETWORK=$2

echo "Running quality gates for $CONTRACT_NAME..."

# Gate 1: Build success
echo "Gate 1: Build check..."
if ! make build-opt; then
  echo "❌ Build failed"
  exit 1
fi
echo "✅ Build passed"

# Gate 2: Unit tests
echo "Gate 2: Unit tests..."
if ! cargo test -p $CONTRACT_NAME; then
  echo "❌ Unit tests failed"
  exit 1
fi
echo "✅ Unit tests passed"

# Gate 3: WASM size check
echo "Gate 3: WASM size check..."
wasm_size=$(stat -c%s target/wasm32-unknown-unknown/release/$CONTRACT_NAME.wasm)
max_size=$((1024 * 1024))  # 1MB limit

if [ $wasm_size -gt $max_size ]; then
  echo "❌ WASM size too large: ${wasm_size} bytes (max: ${max_size} bytes)"
  exit 1
fi
echo "✅ WASM size OK: ${wasm_size} bytes"

# Gate 4: Deployment test
echo "Gate 4: Deployment test..."
if ! ./scripts/deploy_enhanced.sh $CONTRACT_NAME $NETWORK --identity test-admin --dry-run; then
  echo "❌ Deployment test failed"
  exit 1
fi
echo "✅ Deployment test passed"

# Gate 5: Performance check
echo "Gate 5: Performance check..."
CONTRACT_ID=$(./scripts/deploy_enhanced.sh $CONTRACT_NAME $NETWORK --identity test-admin)
resource_fee=$(soroban contract invoke \
  --id $CONTRACT_ID \
  --function get_config \
  --source test-admin \
  --network $NETWORK \
  --dry-run \
  | jq -r '.simulation.transaction_resource_fee')

max_fee=1000000
if [ "$resource_fee" -gt "$max_fee" ]; then
  echo "❌ Resource fee too high: $resource_fee (max: $max_fee)"
  exit 1
fi
echo "✅ Performance OK: $resource_fee"

echo "🎉 All quality gates passed!"
EOF

chmod +x quality_gate.sh

# Run quality gates
./quality_gate.sh medical_records local
```

---

This development guide provides comprehensive workflows for Soroban contract development. For deployment instructions, see the [Deployment Guide](./SOROBAN_CLI_DEPLOYMENT.md), and for interaction patterns, see the [Interaction Guide](./SOROBAN_CLI_INTERACTION.md).
