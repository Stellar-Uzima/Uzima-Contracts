# Health Data Access Logging Contract

## Overview

The Health Data Access Logging Contract (#299) is a Soroban smart contract designed to provide complete traceability and auditing of all patient health data access. This contract ensures transparency and compliance by logging every access attempt with immutable records that cannot be modified or deleted.

## Problem Statement

Healthcare systems face critical challenges with data access transparency:
- **Lack of Traceability**: No clear audit trail of who accessed sensitive patient data
- **Compliance Issues**: Regulatory requirements (HIPAA, GDPR) demand comprehensive audit logs
- **Security Concerns**: Unauthorized access could go undetected without proper logging
- **Accountability Gap**: Difficult to identify insider threats or data misuse

## Solution

The Health Data Access Logging Contract provides:

✅ **Immutable Audit Trail**: All access logs are stored in persistent storage and cannot be modified  
✅ **Complete Traceability**: Every access captures accessor, timestamp, patient ID, and access type  
✅ **Queryable History**: Efficient retrieval of logs by various criteria  
✅ **Integrity Verification**: Rolling hash mechanism to detect tampering  
✅ **Role-Based Access**: Only authorized parties can query their own records  
✅ **Off-Chain Indexing**: Event emissions for external systems to track and index

## Features

### Core Functions

#### 1. `log_access(patient, accessor, access_type, metadata)`
Logs a single access event to patient health data.

**Arguments:**
- `patient_id`: Address/ID of the patient whose data was accessed
- `accessor_address`: Address of the person/system accessing the data
- `access_type`: Type of access (e.g., "read", "write", "export", "delete")
- `metadata`: Optional map of additional context (reason, IP, location, etc.)

**Returns:** Log entry ID (u64)

**Events Emitted:** ACCESS/LOG event with full details

**Immutability:** Once logged, the entry cannot be modified or deleted

```rust
let log_id = contract.log_access(
    patient_address,
    accessor_address,
    "read",
    metadata_map,
);
```

#### 2. `get_access_logs(patient_id)`
Retrieves all access logs for a specific patient.

**Authorization:** Requires auth from the patient themselves

**Returns:** Vector of `AccessLogEntry` structs

**Use Case:** Patient reviewing who has accessed their records

```rust
let logs = contract.get_access_logs(patient_address);
```

### Advanced Queries

#### 3. `get_access_logs_in_range(patient_id, start_timestamp, end_timestamp)`
Retrieves logs within a specific time window.

**Returns:** Filtered vector of logs

**Use Case:** Investigating access patterns during a specific period

```rust
let logs = contract.get_access_logs_in_range(
    patient_address,
    1234567890,
    1234571490,
);
```

#### 4. `get_logs_by_accessor(patient_id, accessor)`
Retrieves all access logs from a specific accessor.

**Returns:** Filtered vector of logs

**Use Case:** Audit trail of a specific doctor or system

```rust
let logs = contract.get_logs_by_accessor(patient_address, doctor_address);
```

#### 5. `get_latest_access_logs(patient_id, limit)`
Retrieves the N most recent access logs.

**Returns:** Last N logs in chronological order

**Use Case:** Quick review of recent activity

```rust
let recent_logs = contract.get_latest_access_logs(patient_address, 10);
```

#### 6. `get_access_log_summary(patient_id)`
Generates aggregated statistics about access patterns.

**Returns:** `AccessLogSummary` with:
- Total number of accesses
- Time of first and last access
- Number of unique accessors
- Summary integrity hash

**Use Case:** Compliance reporting, anomaly detection

```rust
let summary = contract.get_access_log_summary(patient_address);
```

#### 7. `get_unique_accessors_count(patient_id)`
Returns the count of unique individuals/systems that accessed the patient's data.

**Use Case:** Understanding data exposure scope

```rust
let count = contract.get_unique_accessors_count(patient_address);
```

#### 8. `get_unique_accessors(patient_id)`
Returns all unique accessor addresses.

**Use Case:** Identifying all parties with access to patient data

```rust
let accessors = contract.get_unique_accessors(patient_address);
```

### Integrity & Verification

#### 9. `verify_logs_integrity()`
Returns the rolling hash of all log entries.

**Purpose:** Tamper detection - if logs are modified, the hash changes

**Use Case:** Periodic integrity checks by compliance officers

```rust
let hash = contract.verify_logs_integrity();
```

### Administration

#### 10. `initialize(admin, config)`
Initializes the contract with admin and configuration.

**One-time setup** required before any logging can occur.

```rust
let config = LoggingConfig {
    max_logs_per_patient: 1000,
    allow_public_queries: false,
    retention_period: 0,
};
contract.initialize(admin_address, config);
```

#### 11. `update_config(config)` (Admin only)
Updates logging configuration.

**Use Case:** Adjusting retention policies, access controls

```rust
contract.update_config(new_config);
```

#### 12. `get_config()`
Retrieves current logging configuration.

```rust
let config = contract.get_config();
```

## Data Structures

### AccessLogEntry
```rust
pub struct AccessLogEntry {
    pub id: u64,                          // Unique log ID
    pub patient_id: Address,              // Patient whose data was accessed
    pub accessor_address: Address,        // Who accessed the data
    pub timestamp: u64,                   // When access occurred (ledger timestamp)
    pub access_type: String,              // Type of access ("read", "write", etc.)
    pub metadata: Map<String, String>,    // Additional context
    pub entry_hash: BytesN<32>,          // SHA256 hash for integrity
}
```

### AccessLogSummary
```rust
pub struct AccessLogSummary {
    pub patient_id: Address,              // Patient ID
    pub total_accesses: u64,              // Total number of accesses
    pub first_access_timestamp: u64,      // First access time
    pub last_access_timestamp: u64,       // Most recent access time
    pub unique_accessors_count: u32,      // Count of unique accessors
    pub summary_hash: BytesN<32>,        // Hash for integrity
}
```

### LoggingConfig
```rust
pub struct LoggingConfig {
    pub max_logs_per_patient: u32,        // Maximum logs per patient
    pub allow_public_queries: bool,       // Public query permission
    pub retention_period: u64,            // Retention time in seconds (0 = no limit)
}
```

## Acceptance Criteria ✅

- [x] **Every access triggers a log entry** - `log_access()` creates immutable entry
- [x] **Logs are immutable** - Stored in persistent contract storage, cannot be modified
- [x] **Retrieval works efficiently** - Multiple query options with optimized storage indexes
- [x] **Log accessor address** - Captured in `accessor_address` field
- [x] **Log patient ID** - Captured in `patient_id` field
- [x] **Log timestamp** - Captured in `timestamp` field (ledger-based)
- [x] **Provide queryable audit history** - 8 different query functions
- [x] **Ensure logs cannot be modified** - Immutable persistent storage + integrity hashing

## Authorization Model

- **Patients**: Can only view their own access logs (require auth from patient)
- **Accessors**: Can log their own access (require auth from accessor)
- **Admin**: Can update configuration (require auth from admin)

## Storage Optimization

The contract uses efficient storage patterns:

1. **Persistent Storage**: All logs use persistent storage for immutability
2. **Indexed Queries**: Patient access logs indexed by patient ID for fast retrieval
3. **Unique Accessors Tracking**: Separate index to track unique accessors without duplicates
4. **Rolling Hash**: Single hash value updated with each entry (space-efficient)

## Events

The contract emits two types of events:

1. **INIT/HDL**: When contract is initialized
2. **ACCESS/LOG**: When a new access is logged
3. **CONFIG/UPDATE**: When configuration is updated

## Security Considerations

1. **Immutability**: All logs are stored in persistent storage and cannot be modified
2. **Authorization**: Soroban's `require_auth()` ensures only authorized parties can perform actions
3. **Integrity Hashing**: Rolling hash detects any tampering attempts
4. **No Deletion**: Contract doesn't provide delete functionality to prevent audit trail destruction
5. **Timestamp**: Uses ledger timestamp (cannot be manipulated by users)

## Compliance Features

✅ **HIPAA Compliance**: Complete audit trail of PHI access  
✅ **GDPR Compliance**: Right to access audit logs showing data processing  
✅ **SOC 2 Compliance**: Immutable audit trails for security controls  
✅ **HITECH ACT**: Breach notification requirements supported through query functions

## Usage Example

```rust
// Initialize the contract
let admin = Address::random(&env);
let config = LoggingConfig {
    max_logs_per_patient: 1000,
    allow_public_queries: false,
    retention_period: 0,
};
HealthDataAccessLogging::initialize(env, admin.clone(), config);

// Log a patient data access
let patient = Address::random(&env);
let doctor = Address::random(&env);
let mut metadata = Map::new(&env);
metadata.set(String::from_slice(&env, "reason"), String::from_slice(&env, "routine_checkup"));

let log_id = HealthDataAccessLogging::log_access(
    env.clone(),
    patient.clone(),
    doctor.clone(),
    String::from_slice(&env, "read"),
    metadata,
);

// Patient views their access logs
let logs = HealthDataAccessLogging::get_access_logs(env.clone(), patient.clone());

// Get summary for compliance report
let summary = HealthDataAccessLogging::get_access_log_summary(env.clone(), patient.clone());

// Verify integrity
let integrity_hash = HealthDataAccessLogging::verify_logs_integrity(env);
```

## Testing

The contract includes comprehensive tests covering:

- ✅ Contract initialization
- ✅ Single and multiple access logs
- ✅ Log immutability verification
- ✅ Query filtering (by accessor, time range, latest)
- ✅ Unique accessor tracking
- ✅ Summary generation
- ✅ Integrity verification
- ✅ Configuration updates
- ✅ Authorization checks

Run tests with:
```bash
cargo test -p health_data_access_logging --lib
```

## Future Enhancements

1. **Retention Policies**: Implement automatic log archival/deletion based on retention_period
2. **Batch Operations**: Log multiple accesses in a single transaction
3. **Advanced Analytics**: Anomaly detection for unusual access patterns
4. **Encrypted Logs**: Optional encryption for sensitive metadata
5. **Cross-Chain Bridges**: Sync logs with other blockchains
6. **Identity Management**: Integration with DID system for accessor verification

## Files Overview

```
contracts/health_data_access_logging/
├── Cargo.toml                 # Package manifest
└── src/
    ├── lib.rs                 # Main contract implementation
    ├── types.rs               # Data structures and types
    ├── storage.rs             # Storage operations
    ├── queries.rs             # Query functions
    └── test.rs                # Comprehensive tests
```

## Contribution

To extend this contract:

1. Add new query functions in `queries.rs`
2. Add storage helpers in `storage.rs`
3. Expose new contract methods in `lib.rs`
4. Add tests in `test.rs`

## License

MIT License - See repository for details
