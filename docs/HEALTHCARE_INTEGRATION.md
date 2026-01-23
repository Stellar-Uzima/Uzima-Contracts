# Healthcare Integration Guide

## Overview

This document provides comprehensive guidance for integrating with Uzima's healthcare system, including FHIR/HL7 compliance, EMR/EHR integration, and healthcare provider onboarding.

## Table of Contents

1. [FHIR Integration](#fhir-integration)
2. [EMR/EHR System Integration](#emreehr-system-integration)
3. [Provider Onboarding](#provider-onboarding)
4. [Healthcare Standards](#healthcare-standards)
5. [Data Format Conversion](#data-format-conversion)
6. [Healthcare Network Services](#healthcare-network-services)
7. [API Reference](#api-reference)
8. [Testing & Compliance](#testing--compliance)

---

## FHIR Integration

### Overview

Uzima supports HL7 FHIR (Fast Healthcare Interoperability Resources) standard for healthcare data exchange. This enables seamless integration with modern healthcare systems.

### Supported FHIR Resources

The FHIR Integration contract supports the following resource types:

| Resource Type | Use Case | Supported Operations |
|---|---|---|
| **Patient** | Patient demographics and identifiers | Create, Read, Update |
| **Observation** | Vital signs, lab results, measurements | Create, Read, Query |
| **Condition** | Diagnoses and clinical conditions | Create, Read, Update |
| **Medication Statement** | Medication history and current medications | Create, Read |
| **Procedure** | Medical procedures performed | Create, Read |
| **AllergyIntolerance** | Patient allergies and adverse reactions | Create, Read, Update |
| **CareTeam** | Care coordination and team members | Create, Read |
| **Encounter** | Clinical encounters and visits | Create, Read |
| **DiagnosticReport** | Diagnostic reports and conclusions | Create, Read |
| **Immunization** | Vaccination records | Create, Read |
| **DocumentReference** | Document references and metadata | Create, Read |

### FHIR Coding Systems

Uzima supports multiple healthcare coding systems for standardization:

```rust
pub enum CodingSystem {
    ICD10,          // International Classification of Diseases, 10th Edition
    ICD9,           // Legacy diagnosis codes
    CPT,            // Current Procedural Terminology (procedures)
    SNOMEDCT,       // SNOMED Clinical Terms (comprehensive clinical coding)
    LOINC,          // Laboratory codes
    RxNorm,         // Medication names and identifiers
    Custom(String), // Custom coding systems
}
```

### Storing FHIR Observations

```rust
let observation = FHIRObservation {
    identifier: "obs-12345".to_string(),
    status: "final".to_string(),
    category: FHIRCode {
        system: CodingSystem::LOINC,
        code: "8480-6".to_string(),
        display: "Systolic Blood Pressure".to_string(),
    },
    code: FHIRCode {
        system: CodingSystem::LOINC,
        code: "8480-6".to_string(),
        display: "Systolic Blood Pressure".to_string(),
    },
    subject_reference: "Patient/patient-123".to_string(),
    effective_datetime: "2024-01-15T10:30:00Z".to_string(),
    value_quantity_value: 120,
    value_quantity_unit: "mmHg".to_string(),
    interpretation: Some(FHIRCode {
        system: CodingSystem::Custom("hl7.org/fhir".to_string()),
        code: "L".to_string(),
        display: "Low".to_string(),
    }),
    reference_range: "< 120 mmHg".to_string(),
};

client.store_observation(&provider, &observation)?;
```

### Storing FHIR Conditions

```rust
let condition = FHIRCondition {
    identifier: "cond-56789".to_string(),
    clinical_status: "active".to_string(),
    code: FHIRCode {
        system: CodingSystem::ICD10,
        code: "E11.9".to_string(),
        display: "Type 2 diabetes mellitus without complications".to_string(),
    },
    subject_reference: "Patient/patient-123".to_string(),
    onset_date_time: "2023-06-01T00:00:00Z".to_string(),
    recorded_date: "2023-06-15T10:30:00Z".to_string(),
    severity: Some(FHIRCode {
        system: CodingSystem::SNOMEDCT,
        code: "255604002".to_string(),
        display: "Mild".to_string(),
    }),
};

client.store_condition(&provider, &condition)?;
```

---

## EMR/EHR System Integration

### Overview

The EMR Integration contract manages integration between Uzima and Electronic Medical Record (EMR) and Electronic Health Record (EHR) systems.

### Supported EMR Systems

Uzima supports integration with major EMR vendors:

- **Epic Systems**
- **Cerner Corporation**
- **Athena Health**
- **NextGen Healthcare**
- **Practice Fusion**
- **Allscripts**
- **eClinicalWorks**
- **Medidata**
- **Custom Systems** (via FHIR API)

### EMR System Registration

```rust
client.register_emr_system(
    &admin,
    "epic-healthcare-abc",           // System ID
    "Epic Systems Corporation",      // Vendor name
    "admin@epic-healthcare.com",     // Vendor contact
    "2023.3.1",                      // System version
    vec![
        "HL7 v2",
        "HL7 FHIR R4",
        "CDA Release 2",
    ],
    vec![
        "https://fhir.epic-healthcare.com/api/FHIR/R4",
        "https://hl7v2.epic-healthcare.com/api/v2",
    ],
)?;
```

### Provider Onboarding Flow

The provider onboarding process consists of several steps:

#### Step 1: Initiate Onboarding

```rust
client.initiate_onboarding(
    &provider_address,
    "onboard-001",                           // Unique onboarding ID
    "provider-np-2024-001",                  // Provider ID
    "Dr. Jane Smith",                        // Provider name
    "jane.smith@hospital.org",               // Provider email
    "Community Medical Center",              // Facility name
    "1234567890",                            // NPI (10 digits)
    "epic-healthcare-abc",                   // EMR system ID
    vec![
        "Submit credentials".to_string(),
        "Background check".to_string(),
        "License verification".to_string(),
        "DEA registration".to_string(),
        "Malpractice insurance verification".to_string(),
        "HIPAA training".to_string(),
    ],
)?;
```

#### Step 2: Complete Onboarding with Verification

```rust
client.complete_onboarding(
    &admin,
    "onboard-001",                          // Onboarding ID
    "verify-001",                           // Verification ID
    "NM-123456",                            // License number
    "NM",                                   // License state
    "2026-12-31",                           // License expiration
    vec![
        "Board Certified Internal Medicine".to_string(),
        "Board Certified Pediatrics".to_string(),
    ],
    "Policy #: INS-2024-567890",            // Malpractice insurance
    "BG-CHECK-2024-001",                    // Background check ID
)?;
```

---

## Healthcare Standards

### HL7 FHIR R4 Compliance

Uzima implements HL7 FHIR Release 4 (R4) standard, which includes:

- **RESTful API**: HTTP-based resource access
- **FHIR XML & JSON**: Multiple serialization formats
- **Bundles**: Batch operations and transactional support
- **Search Parameters**: Standard FHIR search and filtering
- **Conformance**: Profile-based validation

### HL7 v2 Support

For legacy systems, Uzima supports HL7 v2.x messaging:

- **Message Types**: ADT, ORM, ORU, RGV, etc.
- **Segment Types**: MSH, PID, OBX, OBR, etc.
- **Encoding Characters**: Standard HL7 v2 delimiters
- **Acknowledgments**: ACK messages

### CDA Support

Clinical Document Architecture (CDA) support for:

- **Continuity of Care Documents (CCD)**: Patient summary documents
- **Referral Letters**: Provider-to-provider communication
- **Discharge Summaries**: Hospital discharge documentation

---

## Data Format Conversion

### Supported Conversions

Uzima supports conversion between multiple healthcare data formats:

| Source | Target | Status |
|--------|--------|--------|
| FHIR JSON | FHIR XML | Supported |
| FHIR | HL7 v2 | Supported |
| FHIR | CDA | Supported |
| HL7 v2 | FHIR | Supported |
| CDA | FHIR | Supported |

### Registering Data Mapping

```rust
let mapping = DataMapping {
    source_system: "hospital-legacy-emr".to_string(),
    source_field: "DIAGNOSIS_CODE".to_string(),
    target_system: "uzima-fhir".to_string(),
    target_field: "Condition.code.coding.code".to_string(),
    transformation_rule: "ICD9 to ICD10 conversion with mapping table lookup".to_string(),
    status: "active".to_string(),
};

client.register_data_mapping(&admin, mapping)?;
```

### Retrieving Data Mapping

```rust
let mapping = client.get_data_mapping(
    "hospital-legacy-emr".to_string(),
    "DIAGNOSIS_CODE".to_string(),
)?;

println!("Transform {} to {}", mapping.source_field, mapping.target_field);
```

---

## Healthcare Network Services

### Healthcare Provider Directory

The healthcare network services provide a distributed directory of healthcare providers and facilities:

```rust
let node = NetworkNode {
    node_id: "hospital-nyu-001".to_string(),
    provider_id: "provider-np-2024-001".to_string(),
    node_type: "hospital".to_string(),
    network_name: "NYU Healthcare Network".to_string(),
    geographic_region: "New York, NY".to_string(),
    specialties: vec![
        "Emergency Medicine".to_string(),
        "Cardiology".to_string(),
        "Neurology".to_string(),
    ],
    bed_capacity: 750,
    operating_hours: "24/7".to_string(),
    emergency_services: true,
    telemedicine_enabled: true,
    coordinates: "40.7128,-74.0060".to_string(), // Latitude,Longitude
    connectivity_score: 98, // 0-100
};

client.register_network_node(&admin, node)?;
```

### Interoperability Agreements

Create formal agreements between healthcare providers for data sharing:

```rust
let agreement = InteroperabilityAgreement {
    agreement_id: "interop-nyu-columbia".to_string(),
    initiating_provider: "provider-np-2024-001".to_string(),
    receiving_provider: "provider-np-2024-002".to_string(),
    effective_date: "2024-01-01".to_string(),
    expiration_date: "2026-12-31".to_string(),
    supported_data_types: vec![
        "Patient Demographics".to_string(),
        "Lab Results".to_string(),
        "Vital Signs".to_string(),
        "Problem List".to_string(),
        "Medications".to_string(),
    ],
    access_level: "read-write".to_string(),
    audit_requirement: "quarterly".to_string(),
    data_encryption: "TLS 1.3 with end-to-end encryption".to_string(),
    status: "active".to_string(),
};

client.register_interop_agreement(&admin, agreement)?;
```

### Interoperability Testing

Test and validate interoperability between healthcare systems:

```rust
let test = InteroperabilityTest {
    test_id: "interop-test-001".to_string(),
    test_date: env.ledger().timestamp(),
    provider_a: "provider-np-2024-001".to_string(),
    provider_b: "provider-np-2024-002".to_string(),
    test_type: "data-exchange".to_string(),
    result_status: "passed".to_string(),
    success_rate: 100,
    data_exchanged: 2_500_000,  // 2.5 MB
    latency_ms: 245,
    error_details: "".to_string(),
    tester_address: tester_address.clone(),
};

client.record_interop_test(&tester, test)?;
```

---

## API Reference

### FHIR Integration Contract

#### `initialize`

Initialize the FHIR integration contract.

```rust
pub fn initialize(
    env: Env,
    admin: Address,
    medical_records_contract: Address,
) -> Result<bool, Error>
```

#### `register_provider`

Register a healthcare provider with EMR system details.

```rust
pub fn register_provider(
    env: Env,
    admin: Address,
    provider_id: String,
    name: String,
    facility_type: String,
    npi: String,
    tax_id: String,
    address: String,
    contact_point: String,
    emr_system: String,
    fhir_endpoint: String,
) -> Result<bool, Error>
```

#### `verify_provider`

Verify a healthcare provider's credentials and complete onboarding.

```rust
pub fn verify_provider(
    env: Env,
    admin: Address,
    provider_id: String,
    credential_id: BytesN<32>,
) -> Result<bool, Error>
```

#### `store_observation`

Store a FHIR observation (vital signs, lab results, etc.).

```rust
pub fn store_observation(
    env: Env,
    provider: Address,
    observation: FHIRObservation,
) -> Result<bool, Error>
```

#### `store_condition`

Store a FHIR condition (diagnosis).

```rust
pub fn store_condition(
    env: Env,
    provider: Address,
    condition: FHIRCondition,
) -> Result<bool, Error>
```

#### `store_medication`

Store a FHIR medication statement.

```rust
pub fn store_medication(
    env: Env,
    provider: Address,
    medication: FHIRMedicationStatement,
) -> Result<bool, Error>
```

#### `store_procedure`

Store a FHIR procedure record.

```rust
pub fn store_procedure(
    env: Env,
    provider: Address,
    procedure: FHIRProcedure,
) -> Result<bool, Error>
```

#### `store_allergy`

Store a FHIR allergy intolerance record.

```rust
pub fn store_allergy(
    env: Env,
    provider: Address,
    allergy: FHIRAllergyIntolerance,
) -> Result<bool, Error>
```

### EMR Integration Contract

#### `register_emr_system`

Register an EMR system vendor.

```rust
pub fn register_emr_system(
    env: Env,
    admin: Address,
    system_id: String,
    vendor_name: String,
    vendor_contact: String,
    system_version: String,
    supported_standards: Vec<String>,
    api_endpoints: Vec<String>,
) -> Result<bool, Error>
```

#### `initiate_onboarding`

Start provider onboarding process.

```rust
pub fn initiate_onboarding(
    env: Env,
    provider: Address,
    onboarding_id: String,
    provider_id: String,
    provider_name: String,
    provider_email: String,
    facility_name: String,
    npi: String,
    emr_system_id: String,
    compliance_checklist: Vec<String>,
) -> Result<bool, Error>
```

#### `complete_onboarding`

Complete provider onboarding with verification.

```rust
pub fn complete_onboarding(
    env: Env,
    admin: Address,
    onboarding_id: String,
    verification_id: String,
    license_number: String,
    license_state: String,
    license_expiration: String,
    board_certifications: Vec<String>,
    malpractice_insurance: String,
    background_check_id: String,
) -> Result<bool, Error>
```

#### `register_network_node`

Register a healthcare facility in the network directory.

```rust
pub fn register_network_node(
    env: Env,
    admin: Address,
    node: NetworkNode,
) -> Result<bool, Error>
```

#### `register_interop_agreement`

Register an interoperability agreement between providers.

```rust
pub fn register_interop_agreement(
    env: Env,
    admin: Address,
    agreement: InteroperabilityAgreement,
) -> Result<bool, Error>
```

#### `record_interop_test`

Record interoperability test results.

```rust
pub fn record_interop_test(
    env: Env,
    tester: Address,
    test: InteroperabilityTest,
) -> Result<bool, Error>
```

---

## Testing & Compliance

### Running Integration Tests

```bash
# Run all healthcare integration tests
./scripts/healthcare_integration_test.sh

# Run specific test suites
cargo test --package fhir_integration --release
cargo test --package emr_integration --release
```

### HIPAA Compliance

Uzima's healthcare integration includes:

- **Data Encryption**: TLS 1.3 for all data in transit
- **Access Controls**: Role-based access control (RBAC)
- **Audit Logging**: Comprehensive audit trails for all data access
- **Data Integrity**: Digital signatures and hash verification
- **User Authentication**: Multi-factor authentication support

### Data Privacy Requirements

- All patient data must be encrypted at rest and in transit
- Provider verification must include background checks
- Compliance checklists must be completed before provider activation
- Regular interoperability testing must be performed
- Audit logs must be retained for regulatory compliance

---

## Best Practices

### Provider Onboarding

1. **Verify Credentials**: Obtain and verify licenses from appropriate state boards
2. **Background Checks**: Conduct thorough background checks per state requirements
3. **HIPAA Training**: Ensure HIPAA training completion before data access
4. **Malpractice Insurance**: Verify active malpractice coverage
5. **Regular Re-verification**: Establish periodic credential re-verification schedule

### Data Exchange

1. **Use FHIR Standards**: Prefer FHIR over legacy formats for new integrations
2. **Encryption**: Always encrypt data in transit and at rest
3. **Validation**: Validate data against FHIR schemas before storage
4. **Testing**: Test data exchange before going live
5. **Monitoring**: Monitor integration performance and error rates

### Interoperability

1. **Clear Agreements**: Document all data sharing agreements formally
2. **Audit Requirements**: Establish audit requirements in agreements
3. **Testing**: Perform regular interoperability testing
4. **Performance**: Monitor API latency and throughput
5. **Documentation**: Maintain documentation of all integrations

---

## Support & Resources

For questions or support regarding healthcare integration:

- **Documentation**: See `docs/` directory
- **Examples**: See `contracts/` for example implementations
- **Tests**: See `tests/` for integration test examples
- **Issues**: Report integration issues in the project issue tracker

## Glossary

- **FHIR**: Fast Healthcare Interoperability Resources
- **HL7**: Health Level Seven, healthcare messaging standards
- **EMR**: Electronic Medical Record
- **EHR**: Electronic Health Record
- **NPI**: National Provider Identifier
- **ICD**: International Classification of Diseases
- **CPT**: Current Procedural Terminology
- **SNOMED CT**: Systematized Nomenclature of Medicine Clinical Terms
- **LOINC**: Logical Observation Identifiers Names and Codes
- **RxNorm**: National standard nomenclature for clinical drugs
- **CDA**: Clinical Document Architecture
- **HIPAA**: Health Insurance Portability and Accountability Act

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2024-01-23 | Initial release with FHIR, EMR, and provider onboarding |

---

**Last Updated**: January 23, 2024
