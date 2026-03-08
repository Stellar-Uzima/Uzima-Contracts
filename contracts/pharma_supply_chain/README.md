# Pharmaceutical Supply Chain Contract

A sophisticated blockchain-based pharmaceutical supply chain tracking system that ensures medication authenticity, monitors storage conditions, manages recalls, and maintains regulatory compliance.

## Quick Start

```bash
# Build the contract
cargo build --release --target wasm32-unknown-unknown

# Run tests
cargo test

# Deploy to testnet
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/pharma_supply_chain.wasm \
  --network testnet
```

## Features

✅ **End-to-end pharmaceutical tracking** from manufacturer to patient  
✅ **Anti-counterfeiting** with cryptographic verification  
✅ **IoT integration** for temperature and humidity monitoring  
✅ **Automated recall management** with patient notification  
✅ **Prescription-to-medication verification**  
✅ **Supply chain transparency** for regulators  
✅ **Medication expiry tracking**  
✅ **Controlled substance monitoring** (DEA Schedule I-V)  
✅ **Adverse event correlation**  
✅ **Supply chain optimization and analytics**

## Key Components

- **Manufacturer Registry**: Register and manage pharmaceutical manufacturers
- **Medication Database**: Comprehensive medication specifications
- **Batch Tracking**: Cryptographic authentication for each batch
- **Shipment Management**: Real-time tracking with IoT integration
- **Prescription System**: Electronic prescriptions with refill management
- **Recall System**: Multi-level recalls with patient notification
- **Adverse Events**: Report and correlate adverse reactions
- **Analytics Dashboard**: Supply chain metrics and insights

## Regulatory Compliance

- FDA Drug Supply Chain Security Act (DSCSA)
- DEA Controlled Substances Act (CSA)
- 21 CFR Part 11 (Electronic Records)
- WHO Good Distribution Practice (GDP)
- ISO 9001 Quality Management

## Documentation

For complete documentation, see [PHARMA_SUPPLY_CHAIN.md](../../docs/PHARMA_SUPPLY_CHAIN.md)

## Test Coverage

```
test_initialize ... ok
test_register_manufacturer ... ok
test_register_medication ... ok
test_create_batch_with_authentication ... ok
test_verify_batch_authenticity ... ok
test_create_shipment_with_iot ... ok
test_condition_monitoring_with_violations ... ok
test_complete_shipment ... ok
test_prescription_and_dispensation ... ok
test_controlled_substance_tracking ... ok
test_recall_management ... ok
test_adverse_event_reporting ... ok
test_expiry_checking ... ok
test_supply_chain_transparency ... ok
```

## Architecture

```
Manufacturer → Medication → Batch → Shipment → Prescription → Dispensation
                                ↓
                          Condition Logs
                                ↓
                          IoT Monitoring
```

## Example Usage

### Create a Batch
```rust
let auth_hash = contract.create_batch(
    &String::from_str(&env, "BATCH-001"),
    &String::from_str(&env, "MED-001"),
    &100000, // quantity
    &env.ledger().timestamp(),
    &String::from_str(&env, "LOT-2024-001"),
    &String::from_str(&env, "Facility A"),
    &String::from_str(&env, "QC-CERT-001"),
);
```

### Verify Authenticity
```rust
let is_authentic = contract.verify_batch_authenticity(
    &String::from_str(&env, "BATCH-001"),
    &auth_hash,
);
```

### Track with IoT
```rust
contract.log_condition_data(
    &String::from_str(&env, "LOG-001"),
    &String::from_str(&env, "SHIP-001"),
    &4,  // 4°C temperature
    &50, // 50% humidity
    &Some(40750000),  // latitude
    &Some(-73980000), // longitude
    &String::from_str(&env, "IOT-DEVICE-001"),
);
```

## Integration Points

- **Medical Records**: Links to EMR systems
- **IoT Devices**: Temperature/humidity sensors
- **Pharmacy Systems**: Dispensing verification
- **Regulatory Systems**: FDA, DEA reporting
- **Notification Services**: Patient alerts

## License

Copyright © 2024 Uzima Healthcare
