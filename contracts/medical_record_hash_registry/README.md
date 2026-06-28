# Medical Record Hash Registry

A Soroban smart contract for storing, verifying, and managing cryptographic hashes of medical records to ensure data integrity and prevent tampering.

## Overview

The Medical Record Hash Registry contract addresses the critical need to protect medical data integrity by storing hashed medical records on-chain. This ensures that medical data stored off-chain cannot be altered without detection.

### Key Features

- **Hash Storage**: Securely store cryptographic hashes of medical records
- **Duplicate Prevention**: Automatically reject duplicate record submissions for the same patient
- **Verification**: Verify the authenticity and integrity of records
- **Immutability**: Once stored, records cannot be modified or deleted
- **Patient Organization**: Records are organized by patient ID for easy retrieval
- **Global Lookup**: Find patient IDs associated with specific record hashes

## Contract Functions

### Core Operations

#### `initialize(env: Env, admin: Address) -> Result<(), Error>`
Initializes the contract with an administrator address.

**Parameters:**
- `admin`: The address to be set as the contract administrator

**Returns:** Success or `AlreadyInitialized` error if already initialized

**Events:** Publishes `INIT` event

---

#### `store_record(env: Env, caller: Address, patient_id: Address, record_hash: BytesN<32>) -> Result<(), Error>`
Stores a new medical record hash for a patient with duplicate detection.

**Parameters:**
- `caller`: The address calling the function (must have authorization)
- `patient_id`: The ID of the patient (must be valid Address)
- `record_hash`: The 32-byte hash of the medical record

**Returns:** Success or one of the following errors:
- `NotInitialized`: Contract not yet initialized
- `DuplicateRecord`: Hash already exists for this patient
- `InvalidPatientId`: Invalid patient address provided

**Acceptance Criteria Met:**
- ✅ Prevents duplicate hash submissions
- ✅ Stores immutable record hashes
- ✅ Records are verified upon creation

**Events:** Publishes `STORE` event with patient, hash, and timestamp

---

#### `verify_record(env: Env, patient_id: Address, record_hash: BytesN<32>) -> Result<bool, Error>`
Verifies if a record hash exists and is valid for a specific patient.

**Parameters:**
- `patient_id`: The ID of the patient
- `record_hash`: The hash to verify

**Returns:** `Ok(true)` if record exists and verified, `Ok(false)` if not found

**Events:** Publishes `VERIFY` event with verification status

---

### Query Operations

#### `get_patient_by_hash(env: Env, record_hash: BytesN<32>) -> Option<Address>`
Retrieves the patient ID associated with a specific record hash.

**Parameters:**
- `record_hash`: The hash to look up

**Returns:** `Some(patient_id)` if found, `None` otherwise

---

#### `get_patient_records(env: Env, patient_id: Address) -> Option<PatientRecords>`
Retrieves all records for a specific patient.

**Parameters:**
- `patient_id`: The ID of the patient to query

**Returns:** `Some(PatientRecords)` containing all hashes and metadata, or `None` if no records exist

---

#### `get_record_count(env: Env, patient_id: Address) -> u32`
Gets the total number of records stored for a patient.

**Parameters:**
- `patient_id`: The ID of the patient

**Returns:** Number of records (0 if none exist)

---

#### `get_admin(env: Env) -> Result<Address, Error>`
Retrieves the current contract administrator.

**Returns:** Admin address or `NotInitialized` error

---

## Data Structures

### RecordEntry
```rust
pub struct RecordEntry {
    pub patient_id: Address,        // Patient identifier
    pub record_hash: BytesN<32>,    // SHA-256 hash of medical record
    pub timestamp: u64,              // Block timestamp when stored
    pub verified: bool,              // Verification status
}
```

### PatientRecords
```rust
pub struct PatientRecords {
    pub records: Vec<RecordEntry>,  // All records for the patient
    pub record_count: u32,           // Count of stored records
}
```

## Error Codes

| Error | Code | Description |
|-------|------|-------------|
| `NotInitialized` | 1 | Contract has not been initialized |
| `AlreadyInitialized` | 2 | Contract already initialized |
| `NotAuthorized` | 3 | Caller is not authorized |
| `DuplicateRecord` | 4 | Hash already exists for this patient |
| `RecordNotFound` | 5 | Record does not exist |
| `InvalidPatientId` | 6 | Invalid patient ID provided |
| `InvalidRecordHash` | 7 | Invalid record hash provided |

## Events

The contract publishes the following events:

- **`INIT`**: Contract initialization - emits admin address
- **`STORE`**: Record storage - emits (patient_id, record_hash, timestamp)
- **`VERIFY`**: Record verification - emits (patient_id, record_hash, verified_status)
- **`DUP`**: Duplicate rejection - emits (patient_id, record_hash)

## Acceptance Criteria Fulfillment

✅ **Duplicate hashes are rejected**
- The `store_record` function scans existing records and returns `DuplicateRecord` error

✅ **Verification returns correct status**
- The `verify_record` function returns `true` for stored records, `false` for non-existent ones

✅ **Data is immutable once stored**
- Records cannot be modified or deleted; attempting to store a duplicate hash fails
- Contract uses `persistent()` storage for durability

## Storage

- **Instance Storage**: Admin and initialization flag
- **Persistent Storage**: 
  - Patient record collections (keyed by patient address)
  - Hash index for global lookups (keyed by record hash)

## Security Considerations

1. **Authorization**: `store_record` requires caller authentication via `require_auth()`
2. **Immutability**: Records cannot be deleted or modified
3. **Duplicate Detection**: Prevents replay attacks via same hash
4. **Index Integrity**: Hash index maintains referential consistency
5. **Timestamp Authenticity**: Timestamps come from the blockchain ledger

## Usage Example

```rust
// Initialize
medical_registry.initialize(&admin)?;

// Store a medical record hash
let patient = Address::random(&env);
let record_hash = BytesN::from_array(&env, &[/* hash bytes */]);
medical_registry.store_record(&admin, &patient, &record_hash)?;

// Verify the record exists
let verified = medical_registry.verify_record(&patient, &record_hash)?;
assert_eq!(verified, true);

// Check for duplicates (will fail)
medical_registry.store_record(&admin, &patient, &record_hash)?; // Error: DuplicateRecord

// Query records
let count = medical_registry.get_record_count(&patient);
let records = medical_registry.get_patient_records(&patient);
```

## Testing

Comprehensive tests are included in `src/tests.rs`:

- Contract initialization
- Duplicate detection and rejection
- Record storage and retrieval
- Record verification accuracy
- Multiple records per patient
- Hash-to-patient lookup
- Immutability validation
- Error handling

Run tests with:
```bash
cd contracts/medical_record_hash_registry
cargo test --lib test::tests
```

## Related Contracts

- `medical_records`: Comprehensive medical record management
- `credential_registry`: Credential and credential root management
- `healthcare_compliance`: Compliance verification

## Future Enhancements

- Permission-based access control
- Record metadata (FHIR compliance markers)
- Batch operations for multiple records
- Record expiration/versioning
- Oracle integration for external validation
