# Health Data Access Logging Contract

A Soroban smart contract for immutable, transparent patient health data access logging.

## Quick Start

### Features

- 🔒 **Immutable Audit Trail** - All access logs stored in persistent storage
- 📊 **Advanced Querying** - Multiple ways to query access history
- 🔐 **Authorization-Based** - Only authorized parties can perform actions  
- ✅ **Integrity Verification** - Rolling hash to detect tampering
- 📝 **Complete Traceability** - Captures: accessor, patient, timestamp, access type

### Core Functions

| Function | Purpose | Authorization |
|----------|---------|---------------|
| `log_access()` | Create a new access log | Accessor |
| `get_access_logs()` | Retrieve all logs for patient | Patient |
| `get_access_logs_in_range()` | Logs within time window | Patient |
| `get_logs_by_accessor()` | Logs from specific accessor | Patient |
| `get_latest_access_logs()` | Most recent N logs | Patient |
| `get_access_log_summary()` | Statistics summary | Patient |
| `get_unique_accessors()` | All accessors for patient | Patient |
| `verify_logs_integrity()` | Get rolling hash | Admin/All |

### Initialization

```rust
let config = LoggingConfig {
    max_logs_per_patient: 1000,
    allow_public_queries: false,
    retention_period: 0,
};

HealthDataAccessLogging::initialize(env, admin, config);
```

### Logging Access

```rust
let log_id = HealthDataAccessLogging::log_access(
    env,
    patient_address,
    accessor_address,
    "read",
    metadata,
);
```

### Retrieving Logs

```rust
// Get all logs
let logs = HealthDataAccessLogging::get_access_logs(env, patient_address);

// Get recent logs
let recent = HealthDataAccessLogging::get_latest_access_logs(env, patient_address, 10);

// Get by accessor
let doctor_logs = HealthDataAccessLogging::get_logs_by_accessor(
    env,
    patient_address,
    doctor_address,
);

// Get summary
let summary = HealthDataAccessLogging::get_access_log_summary(env, patient_address);
```

## Module Structure

- **`lib.rs`** - Main contract implementation with all public functions
- **`types.rs`** - Data structures (`AccessLogEntry`, `LoggingConfig`, `AccessLogSummary`, `DataKey`)
- **`storage.rs`** - Low-level storage operations (persistent storage helpers)
- **`queries.rs`** - Query logic for filtering and retrieving logs
- **`test.rs`** - Comprehensive unit tests

## Building

```bash
cargo build -p health_data_access_logging --target wasm32-unknown-unknown --release
```

## Testing

```bash
cargo test -p health_data_access_logging --lib
```

## Key Design Decisions

1. **Immutability**: No delete or update functions - logs are permanent
2. **Authorization**: Every operation uses Soroban's `require_auth()` for security
3. **Indexed Storage**: Patient-specific indexes for efficient queries
4. **Rolling Hash**: Efficient integrity verification without storing all hashes
5. **Modular Design**: Separate modules for storage, queries, and types

## Data Flow

```
log_access()
    ↓
[Create hash of entry]
    ↓
[Store in persistent storage]
    ↓
[Update patient index]
    ↓
[Track unique accessors]
    ↓
[Update rolling hash]
    ↓
[Emit event]
    ↓
[Return log ID]
```

## Compliance

✅ HIPAA - Complete PHI access audit trail  
✅ GDPR - Data subject access audit logs  
✅ SOC 2 - Immutable audit controls  
✅ HITECH ACT - Breach notification support

## See Also

- Full documentation: [HEALTH_DATA_ACCESS_LOGGING.md](../docs/HEALTH_DATA_ACCESS_LOGGING.md)
- Audit contract: [../audit/](../audit/)
- Medical records: [../medical_records/](../medical_records/)
