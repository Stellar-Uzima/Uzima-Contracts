# Contract Naming Conventions

Standardized naming conventions for Uzima smart contracts to ensure consistency across the codebase.

## Table of Contents
1. [Contract Naming](#contract-naming)
2. [Function Naming](#function-naming)
3. [Variable Naming](#variable-naming)
4. [Constant Naming](#constant-naming)
5. [Type Naming](#type-naming)
6. [Event Naming](#event-naming)
7. [Storage Key Naming](#storage-key-naming)
8. [Error Naming](#error-naming)

---

## 1. Contract Naming

Use **snake_case** for all contract directories and files.

| Component | Convention | Example |
|-----------|------------|---------|
| Contract directory | `snake_case` | `medical_records`, `identity_registry` |
| Contract source file | `snake_case` | `lib.rs`, `errors.rs`, `types.rs` |
| Contract module | `snake_case` | `mod medical_records` |

### Directory Structure

```
contracts/
├── medical_records/
│   ├── src/
│   │   ├── lib.rs
│   │   ├── errors.rs
│   │   ├── events.rs
│   │   └── types.rs
│   └── Cargo.toml
```

---

## 2. Function Naming

Use **snake_case** for all function names.

### Public Functions

| Purpose | Convention | Example |
|---------|------------|---------|
| Initialization | `initialize` | `initialize(env, admin)` |
| CRUD operations | verb_noun | `add_record`, `get_record`, `update_record`, `delete_record` |
| User management | manage_entity | `manage_user`, `manage_role` |
| Access control | submit_proof | `submit_zk_access_proof` |
| State queries | get_entity | `get_user`, `get_record` |
| State setters | set_config | `set_rate_limit_config`, `set_zk_enforced` |
| Validation | require_* | `require_initialized`, `require_not_paused` |
| Rate limiting | check_and_update_* | `check_and_update_rate_limit` |

### Private Helper Functions

Use descriptive verbs that indicate the action:

```rust
fn validate_patient_id(env: &Env, patient: &Address) -> Result<(), Error>
fn compute_record_hash(env: &Env, data: &RecordData) -> BytesN<32>
fn check_permissions(env: &Env, caller: &Address, action: Action) -> bool
```

---

## 3. Variable Naming

Use **snake_case** for all variable names.

### General Variables

```rust
let admin = Address::from_string(&admin_str);
let record_id = env.storage().get(&RECORD)?;
let caller = env.current_contract_address();
let is_confidential = false;
let patient_records = Vec::new(env);
```

### Loop Variables

```rust
for user in users.iter() {
    for record in records.iter() {
        // ...
    }
}
```

---

## 4. Constant Naming

Use **SCREAMING_SNAKE_CASE** for all constant names.

### Storage Keys

```rust
const ADMIN: Symbol = symbol_short!("ADMIN");
const VERSION: Symbol = symbol_short!("VERSION");
const USER: Symbol = symbol_short!("USER");
const RECORD: Symbol = symbol_short!("RECORD");
const PAUSED: Symbol = symbol_short!("PAUSED");
```

### Configuration Constants

```rust
const MAX_RATE_LIMIT: u32 = 1000;
const DEFAULT_EXPIRY: u64 = 86400;
const MIN_CONFIDENTIALITY_LEVEL: u8 = 1;
```

---

## 5. Type Naming

Use **PascalCase** for all type names.

### Structs

```rust
#[derive(Clone)]
#[contracttype]
pub struct RecordMetadata {
    pub record_id: u64,
    pub patient_id: Address,
    pub timestamp: u64,
    pub category: String,
    pub is_confidential: bool,
}
```

### Enums

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum Role {
    Admin,
    Doctor,
    Patient,
    None,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
#[repr(u32)]
pub enum Permission {
    ManageUsers = 1,
    ManageSystem = 2,
    CreateRecord = 10,
    ReadRecord = 11,
}
```

### Cross-Chain Types

```rust
#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum ChainId {
    Stellar,
    Ethereum,
    Polygon,
    Avalanche,
    Custom(u32),
}
```

---

## 6. Event Naming

Use **PascalCase** for event types and operation categories.

### Event Type Enum

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum EventType {
    UserCreated,
    UserRoleUpdated,
    UserDeactivated,
    UserActivated,
    RecordCreated,
    RecordAccessed,
    RecordUpdated,
    RecordDeleted,
    AccessRequested,
    AccessGranted,
    AccessDenied,
    AccessRevoked,
    EmergencyAccessGranted,
    ContractPaused,
    ContractUnpaused,
    RecoveryProposed,
    RecoveryApproved,
    MetadataUpdated,
}
```

### Operation Category Enum

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum OperationCategory {
    UserManagement,
    RecordOperations,
    AccessControl,
    EmergencyAccess,
    SystemManagement,
}
```

---

## 7. Storage Key Naming

Use **SCREAMING_SNAKE_CASE** for storage key symbols.

### Common Storage Keys

```rust
const USER: Symbol = symbol_short!("USER");
const ROLE: Symbol = symbol_short!("ROLE");
const RECORD: Symbol = symbol_short!("RECORD");
const PAUSED: Symbol = symbol_short!("PAUSED");
const ADMIN: Symbol = symbol_short!("ADMIN");
const VERSION: Symbol = symbol_short!("VERSION");
const TIMESTAMP: Symbol = symbol_short!("TIMESTAMP");
const METADATA: Symbol = symbol_short!("METADATA");
const CONFIG: Symbol = symbol_short!("CONFIG");
const RATE_LIMIT: Symbol = symbol_short!("RATE_LIMIT");
```

### Composite Keys

```rust
fn user_key(user: &Address) -> Symbol {
    Symbol::new(&env, &format!("USER_{}", user))
}

fn record_key(record_id: u64) -> Symbol {
    Symbol::new(&env, &format!("RECORD_{}", record_id))
}
```

---

## 8. Error Naming

Use **PascalCase** for error variants, prefixed with the error context.

### Error Enum

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[contracttype]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    ContractPaused = 2,
    InvalidCredential = 3,
    RecordNotFound = 4,
    RateLimitExceeded = 5,
    EncryptionRequired = 6,
    InvalidTimestamp = 7,
    DuplicateRecord = 8,
    InvalidRole = 9,
    UserNotFound = 10,
    InsufficientPermissions = 11,
    InvalidAddress = 12,
    StorageError = 13,
}
```

### Error Usage

```rust
if !is_admin {
    return Err(Error::NotAuthorized);
}

if env.storage().get::<_, bool>(&PAUSED).unwrap_or(false) {
    return Err(Error::ContractPaused);
}

match env.storage().get(&USER_PREFIX, &user) {
    Some(_) => return Err(Error::DuplicateUser),
    None => (),
}
```

---

## Quick Reference

| Element | Convention | Example |
|---------|------------|---------|
| Contract | snake_case | `medical_records` |
| Function | snake_case | `initialize`, `add_record` |
| Variable | snake_case | `admin`, `record_id` |
| Constant | SCREAMING_SNAKE_CASE | `ADMIN`, `VERSION` |
| Type | PascalCase | `Role`, `Permission` |
| Enum Variant | PascalCase | `UserCreated`, `RecordCreated` |
| Storage Key | SCREAMING_SNAKE_CASE | `USER`, `RECORD` |
| Error Variant | PascalCase | `NotAuthorized`, `RecordNotFound` |

---

## Related Documentation

- [Developer Guide](./DEVELOPER_GUIDE.md)
- [Event System](./EVENT_SYSTEM.md)
- [Events](./EVENTS.md)