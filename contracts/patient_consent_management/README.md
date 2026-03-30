# Patient Consent Management Contract

A Soroban smart contract for managing patient consent for sharing medical data. Patients explicitly grant and revoke access to healthcare providers or third-party applications with full audit trails.

## Overview

The Patient Consent Management contract ensures HIPAA-compliant, on-chain consent management by:
- Enabling patients to explicitly grant consent to specific addresses
- Allowing revocation of consent at any time
- Maintaining timestamped audit logs of all consent changes
- Preventing unauthorized access through require_auth()

### Key Features

- **Patient-Controlled Access**: Only patients can grant/revoke their own consent
- **Revocable Anytime**: Patients can revoke consent at any point
- **Audit Trail**: Complete timestamped history of all consent events
- **Multiple Providers**: Patients can manage consent for multiple providers independently
- **Verification**: Third parties can verify consent status (read-only)
- **Event Tracking**: All actions emit events for off-chain monitoring and compliance

## Contract Functions

### Core Operations

#### `initialize(env: Env, admin: Address) -> Result<(), Error>`
Initializes the contract with an administrator address.

**Parameters:**
- `admin`: The address to be set as the contract administrator

**Returns:** Success or `AlreadyInitialized` error if already initialized

**Requirements:**
- `admin` must have authorization (require_auth)

**Events:** Publishes `INIT` event

---

#### `grant_consent(env: Env, patient: Address, provider: Address) -> Result<(), Error>`
Grants consent for a provider to access the patient's medical data.

**Parameters:**
- `patient`: The patient's address (must match caller for authorization)
- `provider`: The provider/app address to grant access to

**Returns:** Success or one of the following errors:
- `NotInitialized`: Contract not yet initialized
- `InvalidProvider`: Patient attempted to grant consent to themselves
- `ConsentAlreadyExists`: Active consent already exists for this patient-provider pair
- `UnauthorizedAccess`: Caller is not the patient

**Requirements:**
- `patient` must have authorization (only patient can grant their own consent)
- `provider` must be different from `patient`

**Acceptance Criteria Met:**
- ✅ Only patient wallet can grant consent
- ✅ Unauthorized access attempts fail (require_auth)
- ✅ Consent state persists correctly

**Events:** Publishes `GRANT` event with (patient, provider, timestamp)

---

#### `revoke_consent(env: Env, patient: Address, provider: Address) -> Result<(), Error>`
Revokes patient's consent for a provider to access their medical data.

**Parameters:**
- `patient`: The patient's address (must match caller for authorization)
- `provider`: The provider's address to revoke access from

**Returns:** Success or one of the following errors:
- `NotInitialized`: Contract not yet initialized
- `ConsentNotFound`: Consent does not exist or is already revoked
- `UnauthorizedAccess`: Caller is not the patient

**Requirements:**
- `patient` must have authorization (only patient can revoke their own consent)
- Consent must exist and be active

**Events:** Publishes `REVOKE` event with (patient, provider, timestamp)

---

#### `check_consent(env: Env, patient: Address, provider: Address) -> Result<bool, Error>`
Checks if a provider has active consent from a patient. Read-only operation, no authorization required.

**Parameters:**
- `patient`: The patient's address
- `provider`: The provider's address to check

**Returns:** `Ok(true)` if consent is active, `Ok(false)` if not found or revoked

**Usage:** Third parties can call this to verify they have consent before accessing patient data

**Events:** Publishes `CHECK` event with (patient, provider, has_consent)

---

### Query Operations

#### `get_patient_consents(env: Env, patient: Address) -> Option<ConsentLog>`
Retrieves all consent records (both active and revoked) for a patient.

**Parameters:**
- `patient`: The patient's address

**Returns:** `Some(ConsentLog)` containing all consent records with metadata, or `None` if no records exist

**Data Structure:**
```rust
pub struct ConsentLog {
    pub records: Vec<ConsentRecord>,
    pub record_count: u32,
}
```

---

#### `get_active_consent_count(env: Env, patient: Address) -> u32`
Gets the count of currently active consents for a patient.

**Parameters:**
- `patient`: The patient's address

**Returns:** Number of active consent grants (0 if none)

---

#### `verify_consent_with_audit(env: Env, patient: Address, provider: Address) -> Result<(bool, u64, u64), Error>`
Comprehensive consent verification with complete audit trail.

**Parameters:**
- `patient`: The patient's address
- `provider`: The provider's address

**Returns:** Tuple of (has_consent, granted_timestamp, revoked_timestamp) or `ConsentNotFound` error

**Usage:** For compliance reporting and detailed audit trails

**Data Elements:**
- `has_consent`: true if currently active, false if revoked
- `granted_timestamp`: Block timestamp when consent was granted
- `revoked_timestamp`: Block timestamp when revoked (0 if not revoked)

---

#### `get_admin(env: Env) -> Result<Address, Error>`
Retrieves the current contract administrator.

**Returns:** Admin address or `NotInitialized` error

---

## Data Structures

### ConsentRecord
```rust
pub struct ConsentRecord {
    pub patient: Address,           // Patient's address
    pub provider: Address,          // Provider/app address
    pub granted_at: u64,            // Block timestamp when granted
    pub revoked_at: u64,            // Block timestamp when revoked (0 if active)
    pub active: bool,               // Current consent status
}
```

### ConsentLog
```rust
pub struct ConsentLog {
    pub records: Vec<ConsentRecord>, // All consent records for this patient
    pub record_count: u32,          // Total number of records
}
```

## Error Codes

| Error | Code | Description |
|-------|------|-------------|
| `NotInitialized` | 1 | Contract has not been initialized |
| `AlreadyInitialized` | 2 | Contract already initialized |
| `NotAuthorized` | 3 | Caller lacks required authorization |
| `InvalidPatient` | 4 | Invalid patient address provided |
| `InvalidProvider` | 5 | Invalid provider address (e.g., same as patient) |
| `ConsentNotFound` | 6 | Consent record does not exist or is revoked |
| `ConsentAlreadyExists` | 7 | Active consent already exists for this pair |
| `UnauthorizedAccess` | 8 | Attempt to access/modify another patient's consent |

## Events

The contract publishes the following events for compliance and monitoring:

- **`INIT`**: Contract initialization - emits admin address
- **`GRANT`**: Consent granted - emits (patient, provider, timestamp)
- **`REVOKE`**: Consent revoked - emits (patient, provider, timestamp)
- **`CHECK`**: Consent verified - emits (patient, provider, has_consent)
- **`UNAUTH`**: Unauthorized access attempt - emits (caller, patient, timestamp)

## Acceptance Criteria Fulfillment

✅ **Only patient wallet can grant/revoke consent**
- Functions call `patient.require_auth()` ensuring only the patient can operate on their consent
- Unauthorized addresses are rejected at the protocol level

✅ **Unauthorized access attempts fail**
- Soroban's `require_auth()` mechanism prevents unauthorized wallet access
- Invalid provider addresses (self-grants) are caught and rejected
- Attempts to access non-existent or revoked consents return appropriate errors

✅ **Consent state persists correctly**
- Records stored in persistent storage survive blockchain restarts
- Dual indexing (patient storage + provider index) ensures consistency
- Active flag tracks current consent state accurately

✅ **Events emitted for all actions**
- GRANT events on successful consent grant
- REVOKE events on successful consent revocation
- CHECK events on verification calls
- UNAUTH events on authorization failures

## Storage

- **Instance Storage**: 
  - Admin address and initialization flag (immutable)
- **Persistent Storage**: 
  - ConsentLog per patient (keyed by patient address)
  - Individual ConsentRecord for fast (patient, provider) lookups
  - Enables both historical queries and real-time consent validation

## Security Considerations

1. **Patient Authentication**: `require_auth()` ensures only the patient controls their consent
2. **Authorization on Writes**: All consent modifications require patient authorization
3. **Read Access Open**: `check_consent()` is read-only and doesn't require auth (enables privacy-preserving providers)
4. **Immutable History**: Revoked consents remain in history, creating audit trail
5. **Timestamp Authenticity**: All timestamps come from blockchain ledger (cannot be spoofed)
6. **No Delegation**: Patients cannot delegate consent grant/revoke to other addresses

## Workflow Examples

### Basic Consent Grant and Check
```rust
// Patient grants consent to healthcare provider
patient_consent.grant_consent(&patient, &provider)?;

// Provider verifies they have consent
let has_consent = patient_consent.check_consent(&patient, &provider)?;
if has_consent {
    // Provider can safely access patient data
}
```

### Consent Revocation
```rust
// Patient revokes provider access
patient_consent.revoke_consent(&patient, &provider)?;

// Provider's next check will return false
let has_consent = patient_consent.check_consent(&patient, &provider)?;
assert_eq!(has_consent, false);
```

### Multiple Provider Management
```rust
// Patient manages consent for multiple providers
patient_consent.grant_consent(&patient, &provider_a)?;
patient_consent.grant_consent(&patient, &provider_b)?;
patient_consent.grant_consent(&patient, &provider_c)?;

// Check active consent count
let count = patient_consent.get_active_consent_count(&patient);
assert_eq!(count, 3);

// Revoke one provider
patient_consent.revoke_consent(&patient, &provider_b)?;

// Updated count
let count = patient_consent.get_active_consent_count(&patient);
assert_eq!(count, 2);
```

### Audit Trail Verification
```rust
// Get complete consent audit trail
let (has_consent, granted_at, revoked_at) = 
    patient_consent.verify_consent_with_audit(&patient, &provider)?;

// Can reconstruct when consent was active
if revoked_at > 0 {
    println!("Consent was active from {} to {}", granted_at, revoked_at);
} else {
    println!("Consent has been active since {}", granted_at);
}
```

## Testing

Comprehensive tests are included in `src/test.rs`:

- Contract initialization and re-initialization prevention
- Consent grant functionality
- Consent check for granted, revoked, and non-existent consents
- Consent revocation functionality
- Duplicate grant prevention
- Self-grant prevention (patient cannot grant to themselves)
- Multiple providers per patient
- Grant/revoke/regrant workflows
- Consent history retrieval
- Audit trail verification
- Active consent counting with revocations

Run tests with:
```bash
cd contracts/patient_consent_management
cargo test --lib test::tests
```

## HIPAA Compliance Notes

1. **Patient Control**: Aligns with HIPAA requirement that patients control who can access their PHI
2. **Audit Trails**: Complete timestamped history enables compliance reporting
3. **Revocability**: Patients can revoke access immediately
4. **No Implicit Consent**: Requires explicit grant before access is allowed
5. **Non-Repudiation**: Blockchain timestamps prevent denial of actions

## Related Contracts

- `medical_records`: Comprehensive medical record management
- `medical_record_hash_registry`: Hash-based record integrity verification
- `healthcare_compliance`: Compliance verification and reporting

## Future Enhancements

- Conditional consent (time-limited, purpose-specific)
- Consent templates for common use cases (emergency, research, treatment)
- Delegation support (authorizing healthcare proxies)
- Integration with DID (Decentralized Identifiers) for patient identity
- Consent withdrawal notice requirements
- Provider-initiated access requests with patient approval
- Batch consent operations for multiple patients/providers
- Integration with zero-knowledge proofs for privacy-preserving verification
