# Uzima Contracts Integration Testing Framework

This framework provides a unified and simplified environment for testing complex interactions between multiple Soroban contracts in the Uzima ecosystem.

## Features

- **Unified Environment (`IntegrationTestEnv`)**: A wrapper around `soroban_sdk::Env` that pre-configures common settings and fixtures.
- **Healthcare Team Fixtures**: Automatically generates a complete set of test users (Admin, Doctors, Patients, etc.) using `HealthcareTeam`.
- **Time Control**: Easy-to-use methods for manipulating ledger time (`jump_time`, `set_time`).
- **Event Assertion**: Built-in helpers to verify that contracts are emitting the expected events (`assert_event_emitted`, `assert_event_topics`).
- **Multi-Contract Setup**: Streamlined process for registering and linking multiple contracts in a single test.
- **Registration Helpers**: Specialized methods for deploying and initializing common contracts like `MedicalRecords` and `SutToken`.

## Usage Guide

### 1. Initialize the Environment

```rust
use crate::utils::IntegrationTestEnv;

let test_env = IntegrationTestEnv::new();
let env = &test_env.env;
```

### 2. Access Test Users

```rust
let admin = &test_env.team.admin.address;
let doctor = &test_env.team.doctors[0].address;
let patient = &test_env.team.patients[0].address;
```

### 3. Deploy Contracts

You can deploy contracts manually or use the built-in helpers:

```rust
// Manual deployment
let medical_records_id = env.register_contract(None, MedicalRecordsContract);
let medical_records = MedicalRecordsContractClient::new(env, &medical_records_id);

// Using helpers (recommended)
let (records_id, records_client) = test_env.register_medical_records();
let (token_id, token_client) = test_env.register_token(&test_env.admin);
```

### 4. Control Time

```rust
// Advance time by 1 hour
test_env.jump_time(3600);

// Set to specific timestamp
test_env.set_time(2000000000);
```

### 5. Assert Events

```rust
// Verify that a specific event was emitted with certain topics
test_env.assert_event_topics(&contract_id, test_env.topics(&["EVENT", "REC_NEW"]));

// Verify full event data
test_env.assert_event_emitted(&contract_id, test_env.topics(&["EVENT", "REC_NEW"]), test_env.to_val(expected_data));
```

## Example Test

See `tests/integration/framework_tests.rs` for a complete demonstration of the framework in action.

## Integration with CI

The framework is integrated into the standard Rust test suite. You can run the integration tests using:

```bash
make test-integration
# or
cargo test --test integration
```

