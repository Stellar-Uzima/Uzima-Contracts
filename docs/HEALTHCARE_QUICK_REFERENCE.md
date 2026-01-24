# Healthcare Integration Quick Reference

## Quick Links

- 📚 [Full Healthcare Integration Guide](HEALTHCARE_INTEGRATION.md)
- 🏥 [EMR/EHR Integration Guide](EMR_INTEGRATION.md)
- 📋 [Implementation Summary](HEALTHCARE_INTEGRATION_SUMMARY.md)

## Contracts Overview

### 1. FHIR Integration Contract
**Purpose**: Manage FHIR-compliant healthcare data  
**Location**: `contracts/fhir_integration/src/lib.rs`

```rust
// Store patient observation (vital signs)
let observation = FHIRObservation {
    identifier: "obs-12345".to_string(),
    status: "final".to_string(),
    code: FHIRCode {
        system: CodingSystem::LOINC,
        code: "8480-6".to_string(),
        display: "Systolic Blood Pressure".to_string(),
    },
    // ... more fields
};
client.store_observation(&provider, &observation)?;
```

### 2. EMR Integration Contract
**Purpose**: Manage EMR system integration and provider onboarding  
**Location**: `contracts/emr_integration/src/lib.rs`

```rust
// Initiate provider onboarding
client.initiate_onboarding(
    &provider_address,
    "onboard-001",
    "provider-np-001",
    "Dr. Jane Smith",
    "jane@hospital.org",
    "General Hospital",
    "1234567890",  // NPI
    "epic-prod-001",
    compliance_checklist,
)?;
```

### 3. Healthcare Data Conversion Contract
**Purpose**: Convert between healthcare data formats  
**Location**: `contracts/healthcare_data_conversion/src/lib.rs`

```rust
// Register conversion rule
let rule = ConversionRule {
    rule_id: "fhir-to-hl7v2".to_string(),
    source_format: DataFormat::FHIRJSON,
    target_format: DataFormat::HL7v2,
    // ... more fields
};
client.register_conversion_rule(&admin, rule)?;
```

---

## Common Tasks

### Register a Healthcare Provider

```bash
soroban contract invoke \
    --id "$FHIR_CONTRACT" \
    --source-account "$ADMIN" \
    -- register_provider \
    --provider_id "np-2024-001" \
    --name "Community Medical Center" \
    --facility_type "hospital" \
    --npi "1234567890" \
    --tax_id "12-3456789" \
    --address "123 Main St, New York, NY" \
    --contact_point "contact@hospital.org" \
    --emr_system "epic-prod-001" \
    --fhir_endpoint "https://fhir.epic.com/api/FHIR/R4"
```

### Store a Patient Observation

```bash
curl -X POST \
  "https://fhir-api.uzima.health/Observation" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/fhir+json" \
  -d '{
    "resourceType": "Observation",
    "status": "final",
    "code": {
      "coding": [{
        "system": "http://loinc.org",
        "code": "8480-6",
        "display": "Systolic Blood Pressure"
      }]
    },
    "valueQuantity": {
      "value": 120,
      "unit": "mmHg"
    }
  }'
```

### Initialize Provider Onboarding

```bash
soroban contract invoke \
    --id "$EMR_CONTRACT" \
    --source-account "$PROVIDER" \
    -- initiate_onboarding \
    --onboarding_id "onboard-001" \
    --provider_id "np-2024-001" \
    --provider_name "Dr. John Smith" \
    --provider_email "john@hospital.org" \
    --facility_name "General Hospital" \
    --npi "1234567890" \
    --emr_system_id "epic-prod-001" \
    --compliance_checklist '["License", "DEA", "Background Check"]'
```

### Complete Provider Verification

```bash
soroban contract invoke \
    --id "$EMR_CONTRACT" \
    --source-account "$ADMIN" \
    -- complete_onboarding \
    --onboarding_id "onboard-001" \
    --verification_id "verify-001" \
    --license_number "MD-123456" \
    --license_state "NY" \
    --license_expiration "2026-12-31" \
    --board_certifications '["Board Certified"]' \
    --malpractice_insurance "Policy #123" \
    --background_check_id "BGC-001"
```

### Register Coding Mapping

```bash
soroban contract invoke \
    --id "$CONVERSION_CONTRACT" \
    --source-account "$ADMIN" \
    -- register_coding_mapping \
    --mapping_id "icd9-to-icd10-001" \
    --source_code_system "ICD9" \
    --target_code_system "ICD10" \
    --source_code "250.00" \
    --target_code "E11.9" \
    --source_description "Type 2 diabetes" \
    --target_description "Type 2 diabetes without complications" \
    --confidence_score 95 \
    --effective_date "2023-01-01" \
    --end_date ""
```

---

## FHIR Resource Types

| Resource | Use Case | Code |
|----------|----------|------|
| **Patient** | Patient demographics | 0 |
| **Observation** | Vital signs, lab results | 1 |
| **Condition** | Diagnoses | 2 |
| **MedicationStatement** | Medications | 3 |
| **Procedure** | Medical procedures | 4 |
| **AllergyIntolerance** | Allergies | 5 |
| **CareTeam** | Care coordination | 6 |
| **Encounter** | Clinical visits | 7 |
| **DiagnosticReport** | Diagnostic results | 8 |
| **Immunization** | Vaccinations | 9 |
| **DocumentReference** | Documents | 10 |

---

## Healthcare Coding Systems

| Code System | Use |
|-------------|-----|
| **ICD-10** | Diagnoses (primary) |
| **ICD-9** | Diagnoses (legacy) |
| **CPT** | Procedures |
| **SNOMED CT** | Clinical terminology |
| **LOINC** | Laboratory codes |
| **RxNorm** | Medications |

---

## Supported EMR Vendors

- ✅ Epic Systems
- ✅ Cerner
- ✅ Athena Health
- ✅ NextGen Healthcare
- ✅ eClinicalWorks
- ✅ Allscripts
- ✅ Practice Fusion
- ✅ Custom FHIR-compliant systems

---

## Deployment

```bash
# Build contracts
cd contracts/fhir_integration && cargo build --release
cd contracts/emr_integration && cargo build --release
cd contracts/healthcare_data_conversion && cargo build --release

# Deploy to testnet
./scripts/deploy_healthcare_integration.sh testnet $ADMIN_ADDRESS

# Run tests
./scripts/healthcare_integration_test.sh
```

---

## Error Codes

| Error | Code | Meaning |
|-------|------|---------|
| NotAuthorized | 1 | User not authorized for operation |
| ContractPaused | 2 | Contract in paused state |
| ProviderNotFound | 3 | Provider doesn't exist |
| ProviderAlreadyExists | 4 | Provider already registered |
| ProviderNotVerified | 11 | Provider credentials not verified |
| InvalidNPI | 12 | NPI format invalid |
| InvalidTaxId | 13 | Tax ID format invalid |
| FormatNotSupported | 5 | Data format not supported |
| InvalidConversionRequest | 8 | Conversion request invalid |

---

## Performance Guidelines

| Operation | Max Records | Latency |
|-----------|-------------|---------|
| Store Observation | 100/batch | <100ms |
| Provider Query | 1000 | <500ms |
| Data Conversion | 50/batch | <200ms |
| Interop Test | 1/test | <1s |

---

## Security Checklist

- [ ] TLS 1.3 enabled for all APIs
- [ ] AES-256 encryption for data at rest
- [ ] Multi-factor authentication enabled
- [ ] HIPAA compliance verified
- [ ] Provider credentials verified
- [ ] Audit logging enabled
- [ ] Access controls configured
- [ ] Backup/recovery plan in place

---

## Testing

```bash
# Run all tests
./scripts/healthcare_integration_test.sh

# Run specific test suite
cargo test --package fhir_integration --release

# Run real-world scenarios
./tests/healthcare_real_world_scenarios.sh
```

---

## Support Resources

- **Full Documentation**: `docs/HEALTHCARE_INTEGRATION.md`
- **EMR Integration**: `docs/EMR_INTEGRATION.md`
- **Implementation Summary**: `docs/HEALTHCARE_INTEGRATION_SUMMARY.md`
- **Code Examples**: `contracts/*/src/lib.rs`
- **Tests**: `tests/` and `scripts/`

---

## Key Metrics to Monitor

```
✓ Provider registration success rate
✓ Data synchronization latency
✓ Interoperability agreement uptime
✓ Credential verification time
✓ API error rates
✓ System availability
✓ Audit log completeness
✓ Compliance violations
```

---

**Last Updated**: January 23, 2024
