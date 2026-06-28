# Uzima Contracts Coding Standards

## Overview
This document defines the coding standards and naming conventions for all Uzima smart contracts. Consistent naming patterns improve code readability, maintainability, and developer experience.

## Naming Conventions

### 1. Functions
- **Format**: `snake_case`
- **Examples**: 
  - ✅ `initialize`, `get_record`, `submit_update`
  - ❌ `Initialize`, `getRecord`, `SubmitUpdate`
- **Prefix Guidelines**:
  - `get_` for retrieval functions
  - `set_` for configuration functions  
  - `is_`/`has_` for boolean checks
  - `submit_` for proof/transaction submission
  - `validate_` for validation logic

### 2. Types (Structs, Enums)
- **Format**: `PascalCase`
- **Examples**:
  - ✅ `MedicalRecord`, `AccessRequest`, `Error`
  - ❌ `medical_record`, `access_request`, `error`
- **Note**: Always use `Error` for error enums, never `Err`

### 3. Constants
- **Format**: `SCREAMING_SNAKE_CASE`
- **Examples**:
  - ✅ `APPROVAL_THRESHOLD`, `MAX_RETRY_COUNT`, `DEFAULT_TIMEOUT`
  - ❌ `approval_threshold`, `maxRetryCount`, `DefaultTimeout`
- **Scope**: Use for true constants, not configuration values

### 4. Modules
- **Format**: `snake_case`
- **Examples**:
  - ✅ `detection`, `enforcement`, `monitoring`
  - ❌ `Detection`, `Enforcement`, `Monitoring`

### 5. Variables and Parameters
- **Format**: `snake_case`
- **Examples**:
  - ✅ `record_id`, `patient_address`, `access_level`
  - ❌ `recordId`, `patientAddress`, `accessLevel`

### 6. Error Enum Variants
- **Format**: `PascalCase` (following Rust enum convention)
- **Examples**:
  - ✅ `NotAuthorized`, `RecordNotFound`, `InvalidInput`
  - ❌ `not_authorized`, `record_not_found`, `invalid_input`

## Code Organization

### File Structure
```
contracts/
├── contract_name/
│   ├── src/
│   │   ├── lib.rs          # Main contract implementation
│   │   ├── errors.rs       # Error definitions
│   │   ├── types.rs        # Type definitions (optional)
│   │   ├── events.rs       # Event definitions (optional)
│   │   └── modules/        # Additional modules
│   └── Cargo.toml
```

### Module Organization
- Keep related functionality together
- Split large modules (>500 lines) into submodules
- Use `pub mod` for public modules, `mod` for private

## Rust Specific Guidelines

### Imports
```rust
// Group imports logically
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype,
    Address, BytesN, Env, String, Symbol, Vec,
};
```

### Error Handling
- Always use `Result<T, Error>` for fallible functions
- Document error conditions in function docstrings
- Use descriptive error variant names

### Documentation
- Use `///` for public API documentation
- Include examples for complex functions
- Document preconditions and postconditions

## Examples

### Good Example
```rust
const MAX_RETRY_COUNT: u32 = 3;
const DEFAULT_TIMEOUT_SECS: u64 = 30;

#[derive(Clone)]
#[contracttype]
pub struct MedicalRecord {
    pub record_id: u64,
    pub patient_address: Address,
    pub diagnosis: String,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    RecordNotFound = 2,
    InvalidInput = 3,
}

pub fn get_record(env: Env, record_id: u64) -> Result<MedicalRecord, Error> {
    // Implementation
}
```

### Bad Example (Violations)
```rust
const maxRetryCount: u32 = 3;  // Should be SCREAMING_SNAKE_CASE
const default_timeout: u64 = 30;  // Should be SCREAMING_SNAKE_CASE

#[derive(Clone)]
#[contracttype]
pub struct medical_record {  // Should be PascalCase
    pub recordId: u64,  // Should be snake_case
    pub patientAddress: Address,  // Should be snake_case
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Err {  // Should be Error
    not_authorized = 1,  // Should be PascalCase
    record_not_found = 2,  // Should be PascalCase
}

pub fn GetRecord(env: Env, recordId: u64) -> Result<medical_record, Err> {  // Multiple violations
    // Implementation
}
```

## Enforcement
- Use clippy with strict naming lints
- Run `cargo clippy -- -D warnings` in CI/CD
- Review code against this guide during PR reviews

## Updates
This document should be updated as patterns evolve. Major changes require team consensus.