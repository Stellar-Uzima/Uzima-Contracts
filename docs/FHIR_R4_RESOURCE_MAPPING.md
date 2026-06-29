# FHIR R4 Resource Mapping for `fhir_integration` Contract

This document describes how HL7 FHIR R4 resources map to fields and functions in the `fhir_integration` Soroban smart contract. It covers bidirectional conversion examples, limitations, unsupported fields, and validation requirements.

---

## Overview

The `fhir_integration` contract stores and retrieves healthcare data using structures that mirror core FHIR R4 resource types. All data is stored on the Stellar blockchain and accessed through Soroban contract invocations.

**FHIR Version**: R4 (4.0.1)  
**Contract**: `contracts/fhir_integration/src/lib.rs`  
**Supported Resources**: Patient, Observation, Condition, MedicationStatement, Procedure, AllergyIntolerance

---

## Resource Mapping Tables

### 1. Patient (`FHIRPatient`)

Maps to: [`FHIR R4 Patient`](https://www.hl7.org/fhir/R4/patient.html)

| FHIR R4 Field | Contract Field | Type | Notes |
|---|---|---|---|
| `Patient.identifier[].system` | `FHIRIdentifier.system` | `String` | e.g. `"urn:mrn:hospital-a"` |
| `Patient.identifier[].value` | `FHIRIdentifier.value` | `String` | Actual ID value |
| `Patient.identifier[].use` | `FHIRIdentifier.use_type` | `String` | `"official"`, `"usual"`, `"secondary"` |
| `Patient.name[0].given[0]` | `FHIRPatient.given_name` | `String` | First given name only |
| `Patient.name[0].family` | `FHIRPatient.family_name` | `String` | Family name |
| `Patient.birthDate` | `FHIRPatient.birth_date` | `String` | `YYYY-MM-DD` format |
| `Patient.gender` | `FHIRPatient.gender` | `String` | `"male"`, `"female"`, `"other"`, `"unknown"` |
| `Patient.telecom[0].value` | `FHIRPatient.contact_point` | `String` | Single contact (email or phone) |
| `Patient.address[0].text` | `FHIRPatient.address` | `String` | Free-text address |
| `Patient.communication[].language.coding[0].code` | `FHIRPatient.communication` | `Vec<String>` | Language codes e.g. `["en", "sw"]` |
| `Patient.maritalStatus.coding[0].code` | `FHIRPatient.marital_status` | `String` | `"M"`, `"S"`, `"U"`, etc. |

**Bidirectional Conversion Example**:

```
FHIR R4 JSON → Contract:
{
  "resourceType": "Patient",
  "identifier": [{"system": "urn:mrn:nairobi-hosp", "value": "P-00123", "use": "official"}],
  "name": [{"given": ["Amina"], "family": "Wanjiku"}],
  "birthDate": "1985-03-22",
  "gender": "female",
  "telecom": [{"value": "+254700000000"}],
  "address": [{"text": "Nairobi, Kenya"}],
  "communication": [{"language": {"coding": [{"code": "sw"}]}}],
  "maritalStatus": {"coding": [{"code": "M"}]}
}

→ FHIRPatient {
    identifiers: [FHIRIdentifier { system: "urn:mrn:nairobi-hosp", value: "P-00123", use_type: "official" }],
    given_name: "Amina",
    family_name: "Wanjiku",
    birth_date: "1985-03-22",
    gender: "female",
    contact_point: "+254700000000",
    address: "Nairobi, Kenya",
    communication: ["sw"],
    marital_status: "M",
}

Contract → FHIR R4 JSON:
Reverse the mapping above. Each field maps 1:1.
```

**Limitations / Unsupported Fields**:
- Multiple given names: only `given[0]` is stored
- Multiple addresses: only a single free-text address is supported
- `Patient.deceased`, `Patient.photo`, `Patient.contact`, `Patient.generalPractitioner`, `Patient.managingOrganization` — not supported
- `Patient.link` (patient merging) — not supported

---

### 2. Observation (`FHIRObservation`)

Maps to: [`FHIR R4 Observation`](https://www.hl7.org/fhir/R4/observation.html)

| FHIR R4 Field | Contract Field | Type | Notes |
|---|---|---|---|
| `Observation.identifier[0].value` | `FHIRObservation.identifier` | `String` | Single identifier |
| `Observation.status` | `FHIRObservation.status` | `String` | `"registered"`, `"preliminary"`, `"final"`, `"amended"`, `"cancelled"` |
| `Observation.category[0]` | `FHIRObservation.category` | `FHIRCode` | Coding system + code + display |
| `Observation.code` | `FHIRObservation.code` | `FHIRCode` | LOINC / SNOMED code |
| `Observation.subject.reference` | `FHIRObservation.subject_reference` | `String` | e.g. `"Patient/P-00123"` |
| `Observation.effectiveDateTime` | `FHIRObservation.effective_datetime` | `String` | ISO 8601 timestamp |
| `Observation.valueQuantity.value` | `FHIRObservation.value_quantity_value` | `i64` | Scaled integer (× 1000 for decimals) |
| `Observation.valueQuantity.unit` | `FHIRObservation.value_quantity_unit` | `String` | UCUM unit e.g. `"mmHg"` |
| `Observation.interpretation[]` | `FHIRObservation.interpretation` | `Vec<FHIRCode>` | e.g. `"H"` (High), `"L"` (Low), `"N"` (Normal) |
| `Observation.referenceRange[0].text` | `FHIRObservation.reference_range` | `String` | Human-readable range |

**Bidirectional Conversion Example** (Blood Pressure):

```
FHIR R4 JSON → Contract:
{
  "resourceType": "Observation",
  "identifier": [{"value": "obs-bp-001"}],
  "status": "final",
  "category": [{"coding": [{"system": "http://terminology.hl7.org/CodeSystem/observation-category", "code": "vital-signs", "display": "Vital Signs"}]}],
  "code": {"coding": [{"system": "http://loinc.org", "code": "8867-4", "display": "Heart rate"}]},
  "subject": {"reference": "Patient/P-00123"},
  "effectiveDateTime": "2025-06-01T09:30:00Z",
  "valueQuantity": {"value": 72, "unit": "/min"},
  "interpretation": [{"coding": [{"code": "N", "display": "Normal"}]}],
  "referenceRange": [{"text": "60-100 /min"}]
}

→ FHIRObservation {
    identifier: "obs-bp-001",
    status: "final",
    category: FHIRCode { system: LOINC, code: "vital-signs", display: "Vital Signs" },
    code: FHIRCode { system: LOINC, code: "8867-4", display: "Heart rate" },
    subject_reference: "Patient/P-00123",
    effective_datetime: "2025-06-01T09:30:00Z",
    value_quantity_value: 72000,   // stored as × 1000 to preserve decimals
    value_quantity_unit: "/min",
    interpretation: [FHIRCode { system: Custom, code: "N", display: "Normal" }],
    reference_range: "60-100 /min",
}
```

**Limitations / Unsupported Fields**:
- `Observation.component` (multi-component observations like systolic + diastolic BP) — not supported; store as separate Observations
- `Observation.valueCodeableConcept`, `valueString`, `valueBoolean`, `valueInteger`, `valueRange`, `valueRatio`, `valueSampledData`, `valueTime`, `valueDateTime`, `valuePeriod` — only `valueQuantity` is supported
- `Observation.note`, `Observation.device`, `Observation.specimen` — not supported
- `Observation.focus`, `Observation.encounter`, `Observation.performer` — not supported
- Decimal values: multiply by 1000 before storing in `value_quantity_value` (i64)

---

### 3. Condition (`FHIRCondition`)

Maps to: [`FHIR R4 Condition`](https://www.hl7.org/fhir/R4/condition.html)

| FHIR R4 Field | Contract Field | Type | Notes |
|---|---|---|---|
| `Condition.identifier[0].value` | `FHIRCondition.identifier` | `String` | Single identifier |
| `Condition.clinicalStatus.coding[0].code` | `FHIRCondition.clinical_status` | `String` | `"active"`, `"recurrence"`, `"remission"`, `"inactive"` |
| `Condition.code` | `FHIRCondition.code` | `FHIRCode` | ICD-10 or SNOMED code |
| `Condition.subject.reference` | `FHIRCondition.subject_reference` | `String` | e.g. `"Patient/P-00123"` |
| `Condition.onsetDateTime` | `FHIRCondition.onset_date_time` | `String` | ISO 8601 |
| `Condition.recordedDate` | `FHIRCondition.recorded_date` | `String` | ISO 8601 |
| `Condition.severity.coding[]` | `FHIRCondition.severity` | `Vec<FHIRCode>` | e.g. SNOMED `"24484000"` (Severe) |

**Bidirectional Conversion Example** (Hypertension):

```
FHIR R4 JSON → Contract:
{
  "resourceType": "Condition",
  "identifier": [{"value": "cond-htn-001"}],
  "clinicalStatus": {"coding": [{"code": "active"}]},
  "code": {"coding": [{"system": "http://hl7.org/fhir/sid/icd-10", "code": "I10", "display": "Essential (primary) hypertension"}]},
  "subject": {"reference": "Patient/P-00123"},
  "onsetDateTime": "2020-01-15",
  "recordedDate": "2020-01-20",
  "severity": {"coding": [{"system": "http://snomed.info/sct", "code": "24484000", "display": "Severe"}]}
}

→ FHIRCondition {
    identifier: "cond-htn-001",
    clinical_status: "active",
    code: FHIRCode { system: ICD10, code: "I10", display: "Essential (primary) hypertension" },
    subject_reference: "Patient/P-00123",
    onset_date_time: "2020-01-15",
    recorded_date: "2020-01-20",
    severity: [FHIRCode { system: SNOMEDCT, code: "24484000", display: "Severe" }],
}
```

**Limitations / Unsupported Fields**:
- `Condition.verificationStatus` — not stored (assume all conditions are confirmed before recording)
- `Condition.category`, `Condition.bodySite`, `Condition.stage` — not supported
- `Condition.note`, `Condition.encounter`, `Condition.asserter`, `Condition.recorder` — not supported
- `Condition.evidence` — not supported
- `Condition.abatement*` — not stored

---

## Coding System Mapping

The `CodingSystem` enum maps to standard FHIR coding system URIs:

| Contract Enum | FHIR System URI |
|---|---|
| `ICD10` | `http://hl7.org/fhir/sid/icd-10` |
| `ICD9` | `http://hl7.org/fhir/sid/icd-9-cm` |
| `CPT` | `http://www.ama-assn.org/go/cpt` |
| `SNOMEDCT` | `http://snomed.info/sct` |
| `LOINC` | `http://loinc.org` |
| `RxNorm` | `http://www.nlm.nih.gov/research/umls/rxnorm` |
| `Custom` | Provider-specific URI stored in `code.code` field |

---

## Validation Rules

All FHIR-formatted inputs to the contract are validated as follows:

| Field | Validation |
|---|---|
| `NPI` | Must be exactly 10 characters (numeric) |
| `Tax ID` | 1–20 characters, non-empty |
| `birth_date` | Must match `YYYY-MM-DD` pattern |
| `gender` | Must be one of: `"male"`, `"female"`, `"other"`, `"unknown"` |
| `Observation.status` | Must be one of: `"registered"`, `"preliminary"`, `"final"`, `"amended"`, `"cancelled"` |
| `Condition.clinical_status` | Must be one of: `"active"`, `"recurrence"`, `"remission"`, `"inactive"` |
| `FHIR version (EMR config)` | Must be `"R4"` or `"R5"` |
| `data_format` | Must be `"json"` or `"xml"` |

---

## Unsupported Features

The following FHIR R4 features are **not supported** in the current contract version:

1. **Extensions** (`extension[]`, `modifierExtension[]`): FHIR extensions cannot be stored. Custom data must use existing contract fields.
2. **Contained resources** (`contained[]`): Inline embedded resources are not supported.
3. **Text narratives** (`text.div`): No HTML narrative support.
4. **Resource versioning** (`meta.versionId`, `meta.lastUpdated`): Use blockchain ledger timestamps instead.
5. **Bundle processing**: While `FHIRBundle` metadata is stored, bundle-level operations (transactions, batches) are not executed atomically on-chain. Each resource must be stored individually.
6. **Multi-component Observations** (e.g., blood pressure with systolic and diastolic components): Store each component as a separate Observation.
7. **Decimal precision**: All quantity values are stored as `i64` multiplied by 1000. Maximum precision is 3 decimal places.
8. **Medication.Ingredient, Dosage.doseAndRate**: Complex dosage structures not supported; use `FHIRMedicationStatement.dosage` as a free-text string.

---

## FHIR Input Validation Tests

The following test scenarios validate FHIR-compliant input and output:

| Test | Description | Expected Result |
|---|---|---|
| Valid Patient registration | Register patient with all required FHIR fields | `Ok(true)` |
| Invalid NPI (9 digits) | NPI shorter than 10 characters | `Err(InvalidNPI)` |
| Invalid gender value | Gender not in allowed set | `Err(InvalidFHIRData)` |
| Valid Observation (final status) | Observation with LOINC code and quantity | Stored and retrievable |
| Invalid Observation status | Status value not in allowed set | `Err(InvalidFHIRData)` |
| Valid Condition (ICD-10 code) | Condition with active clinical status | Stored and retrievable |
| Invalid clinical_status | Value not in allowed set | `Err(InvalidFHIRData)` |
| Observation decimal value | Value stored as `i64` × 1000 | Round-trips correctly |
| Provider not verified | Storing FHIR data without verification | `Err(ProviderNotVerified)` |
| Bundle metadata storage | Store bundle header with valid type | `Ok(true)` |

See `tests/integration/ihe_fhir_integration_tests.rs` for executable test cases.

---

## Integration Flow

```
Off-chain EHR System (FHIR R4 JSON)
    │
    │  1. Parse FHIR JSON
    │  2. Map fields to contract types (see tables above)
    │  3. Validate all required fields
    │
    ▼
fhir_integration Contract
    │
    │  register_provider() → verify_provider() → store_observation() / store_condition()
    │
    ▼
Stellar Ledger (immutable, auditable)
    │
    │  get_observation() / get_condition()
    │  Map contract types back to FHIR R4 JSON
    │
    ▼
Off-chain Consumer (FHIR R4 JSON)
```

---

## Related Files

- Contract source: `contracts/fhir_integration/src/lib.rs`
- IHE/FHIR integration tests: `tests/integration/ihe_fhir_integration_tests.rs`
- EMR integration guide: `docs/EMR_INTEGRATION.md`
- Healthcare integration: `docs/HEALTHCARE_INTEGRATION.md`
