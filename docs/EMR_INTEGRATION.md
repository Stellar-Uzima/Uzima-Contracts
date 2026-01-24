# EMR/EHR System Integration Guide

## Overview

This guide provides detailed information about integrating Electronic Medical Records (EMR) and Electronic Health Records (EHR) systems with Uzima's healthcare platform.

## Table of Contents

1. [EMR System Support](#emr-system-support)
2. [Integration Workflow](#integration-workflow)
3. [Provider Onboarding Process](#provider-onboarding-process)
4. [API Integration](#api-integration)
5. [Security & Compliance](#security--compliance)
6. [Healthcare Network Services](#healthcare-network-services)
7. [Troubleshooting](#troubleshooting)

---

## EMR System Support

### Supported EMR Vendors

Uzima provides native support for the following EMR system vendors:

| Vendor | Product | Version | Status |
|--------|---------|---------|--------|
| **Epic Systems** | EHR Suite | 2023.3+ | Certified |
| **Cerner** | HealtheLife | 9.x+ | Certified |
| **Athena Health** | athenaOne | Current | Certified |
| **NextGen Healthcare** | NextGen Office | 24.x+ | Certified |
| **eClinicalWorks** | eCW Cloud | 13.x+ | Certified |
| **Allscripts** | Professional EHR | 20.0+ | Certified |
| **Practice Fusion** | Cloud EHR | Current | Certified |
| **Custom Systems** | Any FHIR-compliant | R4 | Supported |

### Integration Standards

Each EMR integration supports:

- **HL7 FHIR R4**: Primary modern standard
- **HL7 v2.5.1**: Legacy systems support
- **CDA Release 2**: Document exchange
- **Direct Protocol**: Secure messaging
- **SFTP/FTPS**: Legacy file exchange

---

## Integration Workflow

### Phase 1: EMR System Registration

Register the EMR system in Uzima:

```bash
# Using the Soroban CLI
soroban contract invoke \
    --id "$EMR_CONTRACT" \
    --source-account "$ADMIN" \
    --network testnet \
    -- register_emr_system \
    --system_id "epic-prod-001" \
    --vendor_name "Epic Systems Corporation" \
    --vendor_contact "integration@epic.com" \
    --system_version "2023.3.1" \
    --supported_standards '["HL7 v2.5.1", "HL7 FHIR R4", "CDA Release 2"]' \
    --api_endpoints '["https://fhir.epic.com/api/FHIR/R4", "https://hl7v2.epic.com/api"]'
```

### Phase 2: Provider Onboarding

Initiate provider onboarding process:

```bash
soroban contract invoke \
    --id "$EMR_CONTRACT" \
    --source-account "$PROVIDER_ADDRESS" \
    --network testnet \
    -- initiate_onboarding \
    --onboarding_id "onboard-epic-001" \
    --provider_id "NPI-1234567890" \
    --provider_name "Dr. John Smith" \
    --provider_email "jsmith@hospital.org" \
    --facility_name "General Hospital" \
    --npi "1234567890" \
    --emr_system_id "epic-prod-001" \
    --compliance_checklist '["License Verification", "DEA Registration", "Background Check", "HIPAA Training"]'
```

### Phase 3: Provider Verification

Administrator verifies provider credentials:

```bash
soroban contract invoke \
    --id "$EMR_CONTRACT" \
    --source-account "$ADMIN" \
    --network testnet \
    -- complete_onboarding \
    --onboarding_id "onboard-epic-001" \
    --verification_id "verify-epic-001" \
    --license_number "MD-123456" \
    --license_state "NY" \
    --license_expiration "2026-12-31" \
    --board_certifications '["Board Certified Internal Medicine"]' \
    --malpractice_insurance "Policy #INS-123456" \
    --background_check_id "BGC-123456"
```

### Phase 4: EMR Configuration

Configure EMR system integration settings:

```bash
soroban contract invoke \
    --id "$FHIR_CONTRACT" \
    --source-account "$ADMIN" \
    --network testnet \
    -- configure_emr \
    --provider_id "NPI-1234567890" \
    --fhir_version "R4" \
    --supported_resources '[0, 1, 2, 3, 4]' \
    --authentication_type "oauth2" \
    --oauth_endpoint "https://auth.epic.com/oauth2/authorize" \
    --data_format "json" \
    --batch_size 100 \
    --retry_policy "exponential_backoff"
```

---

## Provider Onboarding Process

### Documentation Requirements

Providers must submit the following documentation:

1. **Medical License**
   - Current state medical license
   - License number and expiration date
   - State board verification

2. **DEA Registration**
   - DEA Certificate of Registration
   - DEA number
   - Prescribing authority verification

3. **Board Certification**
   - Official board certification documents
   - Specialty area
   - Certification expiration date

4. **Malpractice Insurance**
   - Active insurance certificate
   - Coverage amount
   - Policy expiration date

5. **Background Check**
   - Criminal background check report
   - Verification of credentials
   - Conflict of interest disclosure

6. **HIPAA Training**
   - Completion certificate
   - Training date
   - Renewal schedule

### Verification Process

The verification process includes:

1. **Document Review**
   - Validate submitted documentation
   - Verify digital signatures
   - Cross-reference with authoritative sources

2. **Credential Verification**
   - Check state medical board
   - Verify DEA registration
   - Confirm board certification

3. **Background Check**
   - Criminal history review
   - License disciplinary actions
   - Sanction history

4. **Risk Assessment**
   - Calculate risk score
   - Identify red flags
   - Determine trust level

5. **Approval**
   - Final approval decision
   - Credential ID issuance
   - System access activation

### Timeline

Typical onboarding timeline:

| Step | Duration | Notes |
|------|----------|-------|
| Documentation Submission | 3-5 days | Provider gathers documents |
| Document Review | 2-3 days | Initial validation |
| Credential Verification | 3-7 days | External verification |
| Background Check | 5-10 days | Third-party review |
| Final Approval | 1-2 days | Admin approval |
| **Total** | **14-27 days** | Varies by completeness |

---

## API Integration

### FHIR API Endpoints

Standard FHIR RESTful endpoints:

```
GET    /Patient/{id}                    # Get patient
POST   /Patient                          # Create patient
PUT    /Patient/{id}                    # Update patient
GET    /Patient/{id}/Observation        # Get observations
POST   /Observation                      # Create observation
GET    /Condition?patient={id}          # Get conditions
POST   /Condition                        # Create condition
GET    /Medication?patient={id}         # Get medications
POST   /MedicationStatement             # Create medication
```

### Authentication

Uzima supports multiple authentication methods:

```rust
pub enum AuthenticationMethod {
    OAuth2,              // OAuth 2.0 with PKCE
    APIKey,             // Static API key
    MutualTLS,          // Mutual TLS certificate
    SAML2,              // SAML 2.0 assertion
    JWT,                // JSON Web Token
}
```

### Example: Retrieve Patient Data

```bash
# Get patient with FHIR
curl -X GET \
  "https://fhir-api.uzima.health/Patient/patient-123" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Accept: application/fhir+json"
```

### Example: Store Observation

```bash
# Store vital signs observation
curl -X POST \
  "https://fhir-api.uzima.health/Observation" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/fhir+json" \
  -d '{
    "resourceType": "Observation",
    "identifier": [{
      "system": "urn:system:uzima",
      "value": "obs-12345"
    }],
    "status": "final",
    "category": [{
      "coding": [{
        "system": "http://terminology.hl7.org/CodeSystem/observation-category",
        "code": "vital-signs"
      }]
    }],
    "code": {
      "coding": [{
        "system": "http://loinc.org",
        "code": "8480-6",
        "display": "Systolic Blood Pressure"
      }]
    },
    "subject": {
      "reference": "Patient/patient-123"
    },
    "valueQuantity": {
      "value": 120,
      "unit": "mmHg",
      "system": "http://unitsofmeasure.org",
      "code": "mm[Hg]"
    }
  }'
```

---

## Security & Compliance

### Data Encryption

All data in transit and at rest uses encryption:

- **In Transit**: TLS 1.3 minimum
- **At Rest**: AES-256 encryption
- **Keys**: HSM-backed key management

### Access Control

Role-based access control (RBAC):

```rust
pub enum Role {
    Admin,              // Full administrative access
    Provider,           // Can access their own patient data
    Staff,              // Can access assigned patient data
    Auditor,            // Read-only audit access
    System,             // Automated system access
}
```

### Audit Logging

Comprehensive audit logging tracks:

- **Who**: User/system accessing data
- **What**: Specific data accessed
- **When**: Timestamp of access
- **Why**: Purpose of access
- **Where**: System/location of access
- **Result**: Success/failure of operation

### HIPAA Compliance

Uzima maintains HIPAA compliance through:

1. **Business Associate Agreement (BAA)**
   - Required for all integrations
   - Covers all covered entities

2. **Access Controls**
   - Minimum necessary access principle
   - Role-based access control
   - Multi-factor authentication

3. **Audit Controls**
   - Comprehensive activity logging
   - Integrity checking
   - Regular audit reviews

4. **Transmission Security**
   - Encryption in transit
   - Secure protocols
   - Certificate validation

---

## Healthcare Network Services

### Provider Directory

Maintain a distributed directory of healthcare providers:

```rust
pub struct NetworkNode {
    pub node_id: String,
    pub provider_id: String,
    pub node_type: String,       // hospital, clinic, lab, etc.
    pub network_name: String,
    pub geographic_region: String,
    pub specialties: Vec<String>,
    pub bed_capacity: u32,
    pub operating_hours: String,
    pub emergency_services: bool,
    pub telemedicine_enabled: bool,
    pub coordinates: String,     // lat,long
    pub connectivity_score: u32, // 0-100
}
```

### Network Discovery

Discover providers and facilities:

```bash
# Query for cardiology specialists
curl -X GET \
  "https://network.uzima.health/query?specialty=Cardiology&region=New%20York" \
  -H "Authorization: Bearer $ACCESS_TOKEN"

# Response includes all certified providers with specialties
```

### Interoperability Agreements

Formal agreements govern data sharing:

```rust
pub struct InteroperabilityAgreement {
    pub agreement_id: String,
    pub initiating_provider: String,
    pub receiving_provider: String,
    pub effective_date: String,
    pub expiration_date: String,
    pub supported_data_types: Vec<String>,
    pub access_level: String,    // read-only, read-write, limited
    pub audit_requirement: String,
    pub data_encryption: String,
    pub status: String,
}
```

---

## Troubleshooting

### Common Issues and Solutions

#### Issue: Provider Verification Fails

**Symptoms**: Onboarding stuck at verification step

**Diagnosis**:
1. Check NPI format (must be 10 digits)
2. Verify medical license is active
3. Confirm background check completion

**Solution**:
```bash
# Resubmit with corrected information
soroban contract invoke \
    --id "$EMR_CONTRACT" \
    --source-account "$ADMIN" \
    -- complete_onboarding \
    --onboarding_id "onboard-epic-001" \
    # ... corrected details
```

#### Issue: Data Exchange Fails

**Symptoms**: Observations not being stored

**Diagnosis**:
1. Check EMR system registration
2. Verify FHIR endpoint connectivity
3. Confirm authentication credentials

**Solution**:
```bash
# Test FHIR endpoint connectivity
curl -X GET \
  "https://fhir.epic.com/api/FHIR/R4/metadata" \
  -H "Authorization: Bearer $TOKEN"
```

#### Issue: Slow Data Synchronization

**Symptoms**: Data takes too long to sync between systems

**Diagnosis**:
1. Check network connectivity
2. Review batch size configuration
3. Monitor EMR system performance

**Solution**:
```bash
# Adjust batch size
soroban contract invoke \
    --id "$FHIR_CONTRACT" \
    -- configure_emr \
    --batch_size 50  # Reduce if too large
```

### Performance Optimization

For optimal performance:

1. **Batch Operations**: Use batch size of 50-100 records
2. **Connection Pooling**: Maintain persistent connections
3. **Caching**: Cache frequently accessed data
4. **Compression**: Enable GZIP compression for large payloads

### Monitoring and Metrics

Monitor integration health:

```bash
# Check integration status
soroban contract invoke \
    --id "$EMR_CONTRACT" \
    -- get_emr_system \
    --system_id "epic-prod-001"

# Review interoperability test results
soroban contract invoke \
    --id "$EMR_CONTRACT" \
    -- get_interop_test \
    --test_id "interop-test-001"
```

---

## Support

For technical support:

- **Email**: integration-support@uzima.health
- **Phone**: +1-800-UZIMA-HELP
- **Portal**: https://support.uzima.health
- **Status Page**: https://status.uzima.health

---

**Last Updated**: January 23, 2024
