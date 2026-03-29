# Pharmaceutical Supply Chain Tracking System

## Overview

The Pharmaceutical Supply Chain Tracking System is a comprehensive blockchain-based solution for tracking medications from manufacturer to patient. It ensures authenticity, prevents counterfeiting, monitors storage conditions, manages recalls, and maintains regulatory compliance throughout the entire pharmaceutical supply chain.

## Table of Contents

1. [Features](#features)
2. [Architecture](#architecture)
3. [Core Components](#core-components)
4. [Integration Guide](#integration-guide)
5. [Regulatory Compliance](#regulatory-compliance)
6. [API Reference](#api-reference)
7. [Use Cases](#use-cases)
8. [Security Considerations](#security-considerations)

## Features

### End-to-End Tracking
- **Complete Supply Chain Visibility**: Track medications through all stages from manufacturing to patient dispensation
- **Real-time Location Tracking**: Monitor medication location throughout the supply chain
- **Batch-level Traceability**: Track individual batches with unique identifiers and lot numbers

### Anti-Counterfeiting
- **Cryptographic Authentication**: Each batch receives a unique authentication hash
- **Blockchain Anchoring**: Immutable records prevent tampering
- **Verification API**: Instant authenticity verification at any point in the supply chain
- **Counterfeit Detection**: Automatic logging of verification failures

### IoT Integration
- **Temperature Monitoring**: Real-time temperature tracking for cold chain medications
- **Humidity Control**: Monitor humidity levels during storage and transport
- **GPS Location**: Track shipment location with latitude/longitude coordinates
- **Automated Alerts**: Instant notifications when conditions violate specifications
- **Device Authentication**: IoT device verification to prevent data manipulation

### Recall Management
- **Multi-level Recalls**: Support for Class 1, 2, and 3 recalls
- **Automated Patient Notification**: Track affected patients for immediate notification
- **Batch Isolation**: Instantly flag recalled batches to prevent distribution
- **Recovery Tracking**: Monitor units recovered during recall process
- **Regulatory Reporting**: Generate recall reports for regulatory authorities

### Prescription Verification
- **Electronic Prescriptions**: Digital prescription creation and management
- **Medication Matching**: Verify dispensed medication matches prescription
- **Refill Tracking**: Monitor prescription refills and prevent abuse
- **Medical Record Linkage**: Optional integration with electronic medical records
- **Verification Codes**: Unique codes for dispensation verification

### Regulatory Compliance
- **Controlled Substance Tracking**: Special monitoring for Schedule I-V substances
- **Audit Trail**: Complete audit log of all supply chain actions
- **Regulatory Reporting**: Generate reports for FDA, DEA, and other authorities
- **Expiry Management**: Track expiration dates and prevent dispensation of expired medications
- **Quality Certificates**: Store and verify quality control documentation

### Analytics
- **Supply Chain Metrics**: Track delivery times, condition violations, and efficiency
- **Adverse Event Correlation**: Link adverse events to specific batches for pattern detection
- **Counterfeit Attempts**: Monitor and analyze authentication failures
- **Controlled Substance Statistics**: Track controlled substance dispensations
- **Recall Effectiveness**: Measure recall response and recovery rates

## Architecture

### Data Model

```
┌─────────────────┐
│  Manufacturer   │
└────────┬────────┘
         │ creates
         ▼
┌─────────────────┐
│   Medication    │
└────────┬────────┘
         │ produces
         ▼
┌─────────────────┐     ┌──────────────┐
│ Medication Batch│────▶│  Shipment    │
└────────┬────────┘     └──────┬───────┘
         │                     │
         │                     ▼
         │              ┌──────────────┐
         │              │ Condition Log│
         │              └──────────────┘
         ▼
┌─────────────────┐
│  Prescription   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Dispensation   │
└─────────────────┘
```

### Supply Chain Stages

1. **Manufacturing**: Initial production and quality control
2. **Quality Control**: Testing and certification
3. **Packaging**: Final packaging and labeling
4. **Warehousing**: Storage at manufacturing facility
5. **Distribution**: Transport to distributors
6. **Wholesale**: Wholesale distribution centers
7. **Pharmacy**: Retail pharmacies and hospitals
8. **Hospital**: Hospital pharmacies and dispensaries
9. **Clinic Dispensary**: Clinic-based dispensing
10. **Patient**: Final delivery to patient

### Security Model

- **Multi-signature Authorization**: Critical operations require multiple signatures
- **Role-based Access Control**: Different permissions for manufacturers, distributors, pharmacies
- **Cryptographic Verification**: SHA-256 hashing for authentication
- **Immutable Records**: Blockchain storage prevents tampering
- **Audit Trail**: All actions logged for regulatory review

## Core Components

### 1. Manufacturer Management

**Purpose**: Register and manage pharmaceutical manufacturers

**Key Functions**:
- `register_manufacturer()`: Register a new manufacturer with licenses and certifications
- `deactivate_manufacturer()`: Deactivate a manufacturer (regulatory action)

**Data Stored**:
- License numbers
- Certifications (GMP, ISO, etc.)
- Country of operation
- Active status

### 2. Medication Registration

**Purpose**: Register medications with specifications

**Key Functions**:
- `register_medication()`: Register new medication with complete specifications

**Data Stored**:
- NDC (National Drug Code)
- Medication type (Controlled Substance, Prescription, OTC, Biologic, Vaccine, Chemotherapy)
- DEA Schedule (for controlled substances)
- Storage requirements (temperature, humidity)
- Shelf life
- Active ingredients

### 3. Batch Creation & Tracking

**Purpose**: Create medication batches with anti-counterfeiting measures

**Key Functions**:
- `create_batch()`: Create new batch with authentication hash
- `verify_batch_authenticity()`: Verify batch authenticity
- `get_batch()`: Retrieve batch information
- `is_batch_expired()`: Check expiration status

**Authentication**:
Each batch receives a unique cryptographic hash based on:
- Batch ID
- Medication ID
- Lot number
- Manufacturing date
- Production facility

### 4. Shipment Management

**Purpose**: Track medication movement through supply chain

**Key Functions**:
- `create_shipment()`: Create new shipment with IoT device assignment
- `log_condition_data()`: Log temperature, humidity, location from IoT devices
- `complete_shipment()`: Complete delivery and update batch location

**IoT Integration**:
- Device authentication
- Real-time condition monitoring
- Automated violation detection
- GPS tracking

### 5. Prescription System

**Purpose**: Manage prescriptions and dispensations

**Key Functions**:
- `create_prescription()`: Create electronic prescription
- `dispense_medication()`: Dispense medication with verification
- Automatic refill tracking
- Medical record integration

**Verification**:
- Prescription validity checks
- Medication matching
- Expiry verification
- Recall checking

### 6. Recall Management

**Purpose**: Manage medication recalls

**Key Functions**:
- `initiate_recall()`: Start recall process
- `update_recall_recovery()`: Track recovered units

**Recall Levels**:
- **Class 1**: Life-threatening situation
- **Class 2**: Serious adverse health effects
- **Class 3**: Minor adverse effects

### 7. Adverse Event Reporting

**Purpose**: Track and correlate adverse events

**Key Functions**:
- `report_adverse_event()`: Report adverse reaction
- Automatic correlation with batches
- Severity tracking (1-5 scale)

## Integration Guide

### Setting Up the Contract

```rust
// Initialize the contract
let admin = Address::from_string("ADMIN_ADDRESS");
pharma_contract.initialize(&admin);
```

### Registering a Manufacturer

```rust
let certifications = Vec::from_array(&env, [
    String::from_str(&env, "GMP"),
    String::from_str(&env, "ISO9001"),
    String::from_str(&env, "FDA_APPROVED"),
]);

pharma_contract.register_manufacturer(
    &String::from_str(&env, "MFG001"),
    &manufacturer_address,
    &String::from_str(&env, "PharmaCorp International"),
    &String::from_str(&env, "FDA-LIC-12345"),
    &certifications,
    &String::from_str(&env, "USA"),
);
```

### Creating a Medication

```rust
let active_ingredients = Vec::from_array(&env, [
    String::from_str(&env, "Atorvastatin Calcium"),
]);

pharma_contract.register_medication(
    &String::from_str(&env, "MED001"),
    &String::from_str(&env, "Lipitor 20mg"),
    &String::from_str(&env, "Atorvastatin"),
    &String::from_str(&env, "0071-0156-23"), // NDC code
    &MedicationType::Prescription,
    &ControlledSubstanceSchedule::NotControlled,
    &String::from_str(&env, "MFG001"),
    &false, // does not require cold chain
    &15,    // min temp celsius
    &30,    // max temp celsius
    &60,    // max humidity %
    &730,   // shelf life days (2 years)
    &String::from_str(&env, "Tablet"),
    &String::from_str(&env, "20mg"),
    &active_ingredients,
);
```

### Creating a Batch with Anti-Counterfeiting

```rust
let auth_hash = pharma_contract.create_batch(
    &String::from_str(&env, "BATCH-2024-001"),
    &String::from_str(&env, "MED001"),
    &100000, // quantity
    &env.ledger().timestamp(),
    &String::from_str(&env, "LOT-2024-Q1-001"),
    &String::from_str(&env, "Manufacturing Facility A - Building 3"),
    &String::from_str(&env, "QC-CERT-2024-001"),
);

// Store auth_hash securely - it's needed for verification
```

### Verifying Batch Authenticity

```rust
// At any point in supply chain, verify authenticity
let is_authentic = pharma_contract.verify_batch_authenticity(
    &String::from_str(&env, "BATCH-2024-001"),
    &stored_auth_hash,
);

if !is_authentic {
    // Alert: Potential counterfeit detected
    // Log incident for regulatory investigation
}
```

### Creating a Shipment with IoT Monitoring

```rust
pharma_contract.create_shipment(
    &String::from_str(&env, "SHIP-2024-001"),
    &String::from_str(&env, "BATCH-2024-001"),
    &50000, // quantity
    &distributor_address,
    &SupplyChainStage::Distribution,
    &(env.ledger().timestamp() + 86400), // 24 hours
    &Some(String::from_str(&env, "IOT-SENSOR-12345")),
);
```

### Logging IoT Condition Data

```rust
// IoT device sends data every 15 minutes
pharma_contract.log_condition_data(
    &String::from_str(&env, "LOG-001"),
    &String::from_str(&env, "SHIP-2024-001"),
    &23, // temperature celsius
    &45, // humidity percent
    &Some(40750000),  // latitude (NYC example)
    &Some(-73980000), // longitude (NYC example)
    &String::from_str(&env, "IOT-SENSOR-12345"),
);

// System automatically detects violations and updates shipment status
```

### Completing a Shipment

```rust
pharma_contract.complete_shipment(
    &String::from_str(&env, "SHIP-2024-001"),
    &true, // conditions verified
);

// Batch location and stage automatically updated
```

### Creating a Prescription

```rust
pharma_contract.create_prescription(
    &String::from_str(&env, "RX-2024-001"),
    &patient_address,
    &String::from_str(&env, "MED001"),
    &30, // quantity (30 tablets)
    &String::from_str(&env, "Take 1 tablet daily with food"),
    &3, // 3 refills allowed
    &(env.ledger().timestamp() + 31536000), // valid for 1 year
    &Some(String::from_str(&env, "EMR-RECORD-123456")),
);
```

### Dispensing Medication

```rust
pharma_contract.dispense_medication(
    &String::from_str(&env, "DISP-2024-001"),
    &String::from_str(&env, "RX-2024-001"),
    &String::from_str(&env, "BATCH-2024-001"),
    &30, // quantity dispensed
    &String::from_str(&env, "VERIFY-CODE-ABC123"),
);

// System automatically:
// - Verifies prescription validity
// - Checks batch authenticity
// - Verifies medication matches prescription
// - Checks for recalls
// - Checks expiration
// - Updates refill count
// - Tracks controlled substances if applicable
```

### Initiating a Recall

```rust
let affected_batches = Vec::from_array(&env, [
    String::from_str(&env, "BATCH-2024-001"),
    String::from_str(&env, "BATCH-2024-002"),
]);

pharma_contract.initiate_recall(
    &String::from_str(&env, "RECALL-2024-001"),
    &affected_batches,
    &String::from_str(&env, "MED001"),
    &RecallLevel::Class2,
    &String::from_str(&env, "Potential contamination detected during routine testing"),
);

// System automatically:
// - Marks batches as recalled
// - Identifies affected patients
// - Prepares notification list
// - Updates analytics
```

### Reporting Adverse Events

```rust
pharma_contract.report_adverse_event(
    &String::from_str(&env, "AE-2024-001"),
    &patient_address,
    &String::from_str(&env, "MED001"),
    &String::from_str(&env, "BATCH-2024-001"),
    &String::from_str(&env, "DISP-2024-001"),
    &3, // severity (1-5 scale)
    &String::from_str(&env, "Patient experienced mild nausea and headache"),
);

// Enables correlation of adverse events with specific batches
```

### Retrieving Analytics

```rust
let analytics = pharma_contract.get_analytics();

println!("Total Batches: {}", analytics.total_batches);
println!("Active Shipments: {}", analytics.active_shipments);
println!("Total Recalls: {}", analytics.total_recalls);
println!("Adverse Events: {}", analytics.total_adverse_events);
println!("Avg Delivery Time: {} seconds", analytics.avg_delivery_time);
println!("Condition Violations: {}", analytics.condition_violations);
println!("Counterfeit Attempts: {}", analytics.counterfeit_attempts);
println!("Controlled Substance Dispensations: {}", analytics.controlled_substance_dispensations);
```

## Regulatory Compliance

### FDA Requirements

**Drug Supply Chain Security Act (DSCSA)**:
- ✅ Batch-level traceability
- ✅ Electronic product verification
- ✅ Transaction information, history, and statement
- ✅ Suspect product quarantine (recalls)
- ✅ Illegitimate product notification

**21 CFR Part 11 (Electronic Records)**:
- ✅ Audit trails
- ✅ Digital signatures
- ✅ Record retention
- ✅ System validation

### DEA Requirements (Controlled Substances)

**Controlled Substances Act Compliance**:
- ✅ Schedule tracking (Schedule I-V)
- ✅ Dispensation logging
- ✅ Refill restrictions for Schedule II
- ✅ Prescription requirements
- ✅ Audit trail for regulatory inspection

### International Standards

**WHO Guidelines**:
- ✅ Good Manufacturing Practice (GMP)
- ✅ Good Distribution Practice (GDP)
- ✅ Temperature monitoring for cold chain
- ✅ Counterfeit prevention

**ISO Standards**:
- ✅ ISO 9001 (Quality Management)
- ✅ ISO 13485 (Medical Devices)
- ✅ Cold chain standards

## API Reference

### Initialization
- `initialize(admin: Address)` - Initialize contract with admin

### Manufacturer Management
- `register_manufacturer(...)` - Register new manufacturer
- `deactivate_manufacturer(id: String)` - Deactivate manufacturer

### Medication Management
- `register_medication(...)` - Register new medication
- `get_medication(id: String)` - Get medication details

### Batch Management
- `create_batch(...)` - Create new batch with authentication
- `verify_batch_authenticity(batch_id, hash)` - Verify authenticity
- `get_batch(id: String)` - Get batch information
- `is_batch_expired(id: String)` - Check expiration

### Shipment Management
- `create_shipment(...)` - Create shipment with IoT device
- `log_condition_data(...)` - Log IoT sensor data
- `complete_shipment(id, verified)` - Complete delivery
- `get_shipment(id: String)` - Get shipment status

### Prescription Management
- `create_prescription(...)` - Create prescription
- `dispense_medication(...)` - Dispense with verification

### Recall Management
- `initiate_recall(...)` - Initiate recall
- `update_recall_recovery(id, units)` - Update recovery status
- `get_recall(id: String)` - Get recall information

### Adverse Events
- `report_adverse_event(...)` - Report adverse reaction

### Analytics
- `get_analytics()` - Get supply chain analytics

## Use Cases

### Use Case 1: Cold Chain Vaccine Distribution

**Scenario**: COVID-19 vaccine requiring ultra-cold storage (-70°C)

**Implementation**:
1. Manufacturer creates batch with cold chain requirements
2. IoT sensors monitor temperature continuously during shipment
3. System alerts if temperature exceeds -60°C
4. Shipment automatically flagged for review if violation occurs
5. Pharmacist verifies temperature logs before accepting delivery

**Benefits**:
- Ensures vaccine efficacy
- Regulatory compliance
- Reduces waste from spoiled vaccines

### Use Case 2: Controlled Substance Tracking

**Scenario**: Oxycodone (Schedule II) distribution and dispensing

**Implementation**:
1. Medication registered as Schedule II controlled substance
2. Prescriptions created with no refills (DEA requirement)
3. Every dispensation logged for DEA reporting
4. Audit trail maintained for 5 years
5. Analytics track controlled substance patterns

**Benefits**:
- DEA compliance
- Prevents prescription drug abuse
- Enables pattern detection

### Use Case 3: Counterfeit Prevention

**Scenario**: High-value cancer medication counterfeiting attempt

**Implementation**:
1. Legitimate batch created with authentication hash
2. Counterfeiters produce fake medication
3. Pharmacy scans batch and attempts verification
4. System detects invalid authentication hash
5. Incident logged and regulators notified

**Benefits**:
- Patient safety
- Brand protection
- Regulatory reporting

### Use Case 4: Medication Recall

**Scenario**: Blood pressure medication contaminated with carcinogen

**Implementation**:
1. Manufacturer discovers contamination
2. Recall initiated for affected batches
3. System identifies all dispensations from those batches
4. Patients automatically flagged for notification
5. Pharmacies blocked from dispensing recalled batches
6. Recovery progress tracked

**Benefits**:
- Rapid patient notification
- Prevents further distribution
- Regulatory compliance

### Use Case 5: Adverse Event Investigation

**Scenario**: Multiple patients report similar side effects

**Implementation**:
1. Healthcare providers report adverse events
2. System correlates events with specific batch
3. Pattern detected: all from same manufacturing run
4. Investigation launched into manufacturing process
5. Potential recall initiated if necessary

**Benefits**:
- Early problem detection
- Patient safety
- Quality improvement

## Security Considerations

### Cryptographic Security

**Authentication Hashes**:
- SHA-256 hashing for batch authentication
- Collision-resistant
- Deterministic verification

**Digital Signatures**:
- Shipment signatures prevent tampering
- Pharmacist signatures for dispensations
- Multi-party verification

### Access Control

**Role-based Permissions**:
- Admin: Full contract control
- Manufacturer: Create medications and batches
- Distributor: Create and complete shipments
- Pharmacy: Dispense medications
- IoT Devices: Log condition data only

### Data Integrity

**Immutable Records**:
- Blockchain storage prevents modification
- Audit trail cannot be deleted
- Historical data preserved

**Validation**:
- Input validation on all functions
- Business rule enforcement
- Regulatory compliance checks

### Privacy Considerations

**Patient Privacy**:
- Patient addresses pseudonymous
- Medical records referenced, not stored
- HIPAA compliance considerations

**Proprietary Information**:
- Manufacturing details protected
- Quality certificates can be hashed
- Selective disclosure to regulators

## Testing

Run the comprehensive test suite:

```bash
cd contracts/pharma_supply_chain
cargo test
```

### Test Coverage

- ✅ Manufacturer registration
- ✅ Medication registration (all types)
- ✅ Batch creation and authentication
- ✅ Counterfeit detection
- ✅ Shipment lifecycle
- ✅ IoT condition monitoring
- ✅ Condition violations
- ✅ Prescription creation
- ✅ Medication dispensation
- ✅ Controlled substance tracking
- ✅ Recall management
- ✅ Adverse event reporting
- ✅ Expiry checking
- ✅ Supply chain transparency
- ✅ Analytics

## Deployment

### Prerequisites
- Soroban SDK 22.0.0+
- Rust toolchain
- Stellar network access

### Build
```bash
cargo build --release --target wasm32-unknown-unknown
```

### Deploy
```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/pharma_supply_chain.wasm \
  --network testnet
```

## Future Enhancements

### Planned Features
- AI-powered demand forecasting
- Blockchain interoperability for cross-chain verification
- Mobile app for patient medication verification
- Real-time regulatory reporting dashboard
- Machine learning for counterfeit detection
- Integration with hospital EMR systems
- Automated reordering based on inventory levels
- Patient adherence tracking

### Integration Opportunities
- HL7 FHIR for healthcare interoperability
- GS1 standards for product identification
- EPCIS for supply chain events
- FDA's National Drug Code Directory
- DEA ARCOS for controlled substances

## Support

For technical support, regulatory questions, or integration assistance:
- Documentation: `/docs/PHARMA_SUPPLY_CHAIN.md`
- Issues: GitHub Issues
- Email: support@uzima-healthcare.com

## License

Copyright © 2024 Uzima Healthcare. All rights reserved.

---

**Version**: 1.0.0
**Last Updated**: March 6, 2026
**Compliance**: FDA DSCSA, DEA CSA, 21 CFR Part 11, WHO GDP, ISO 9001
