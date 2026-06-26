# Input Validation Standards for Smart Contracts

## Overview

This document establishes comprehensive input validation requirements for all smart contracts in the Uzima platform. Consistent validation across all contracts ensures data integrity, prevents invalid states, and improves security.

## Issue Reference

- **Issue #771**: Input Validation is Inconsistent — Some Contracts Validate, Others Don't
- **Status**: In Progress
- **Key Contracts Updated**: 
  - `contracts/reputation/src/lib.rs`
  - `contracts/clinical_trial/src/lib.rs`

## Shared ValidationUtils Module

A new shared library has been created to provide common validation utilities:

**Location**: `libs/validation_utils/`

**Features**:
- String validation (length bounds, character sets)
- Address validation
- Numeric validation (ranges, positive/negative checks)
- Collection validation (size bounds, emptiness)

### Using ValidationUtils

```rust
use validation_utils::{
    validate_string_length,
    validate_non_negative,
    validate_u64_range,
};

// Validate string length
validate_string_length(&my_string, 1, 100)?;

// Validate numeric value is non-negative
validate_non_negative(amount, false)?; // false means don't allow zero

// Validate numeric value is within range
validate_u64_range(max_participants, 1, 10_000_000)?;
```

## Validation Categories

### 1. String Validation

All string inputs must be validated for:
- **Length bounds**: Minimum and maximum character length
- **Non-empty requirement**: Critical fields must not be empty
- **Character sets**: Where applicable, restrict to specific character types

#### Standard Length Bounds

| Field Type | Min | Max | Example |
|------------|-----|-----|---------|
| Title/Name | 2-3 | 128-256 | Protocol title: 3-256 chars |
| Description/Reference | 10 | 256-512 | IPFS hash, metadata ref |
| Short identifier | 1 | 50 | Category, tag |
| Address/DID | 10 | 200 | Decentralized identifier |

#### Implementation Pattern

```rust
fn validate_title(title: &String) -> Result<(), Error> {
    let len = title.len();
    if len == 0 || len < 3 || len > 256 {
        return Err(Error::InvalidTitle);
    }
    Ok(())
}

pub fn create_item(env: Env, title: String) -> Result<(), Error> {
    validate_title(&title)?;
    // ... rest of logic
}
```

### 2. Address Validation

All address parameters should be validated to ensure:
- Address is properly constructed (Soroban SDK handles most validation)
- Addresses are different when required (no self-referential operations)
- Address uniqueness in collections when needed

#### Implementation Pattern

```rust
use validation_utils::validate_addresses_different;

pub fn transfer(env: Env, from: Address, to: Address) -> Result<(), Error> {
    validate_addresses_different(&from, &to)?;
    // ... rest of logic
}
```

### 3. Numeric Validation

All numeric inputs must validate:
- **Sign correctness**: Amount fields should not accept negative values
- **Range bounds**: Values within expected operational ranges
- **Zero handling**: Some operations require non-zero values

#### Standard Ranges

| Field Type | Min | Max | Example |
|------------|-----|-----|---------|
| Reputation amount | 0 | i128::MAX | Non-negative only |
| Participant count | 1 | 10,000,000 | Must be positive |
| Percentage | 0 | 100 | Or 0-10,000 for basis points |
| Severity level | 0 | 10 | Application-specific range |
| Timestamp | > 0 | current + 1 day | Not zero, not far future |

#### Implementation Pattern

```rust
use validation_utils::{validate_non_negative, validate_u64_range};

pub fn mint_reputation(env: Env, amount: i128) -> Result<(), Error> {
    // Ensure non-negative and non-zero
    validate_non_negative(amount, false)?; // false = don't allow zero
    
    // Or with specific range
    validate_i128_range(amount, 1, 1_000_000)?;
    
    // ... rest of logic
}

pub fn set_severity(env: Env, severity: u32) -> Result<(), Error> {
    validate_u32_range(severity, 0, 10)?;
    // ... rest of logic
}
```

### 4. Collection Validation

Collections (vectors, maps) should validate:
- **Size bounds**: Minimum and maximum number of elements
- **Non-empty requirement**: When elements are required
- **Uniqueness**: When duplicates are not allowed

#### Implementation Pattern

```rust
use validation_utils::{validate_vector_not_empty, validate_vector_size};

pub fn process_batch(env: Env, items: Vec<Item>) -> Result<(), Error> {
    validate_vector_not_empty(&items)?;
    validate_vector_size(&items, 1, 1000)?;
    // ... rest of logic
}
```

### 5. Enum Validation

All enum-like selections must validate:
- Value is within acceptable enum range
- Only valid status transitions occur

#### Implementation Pattern

```rust
pub fn validate_severity(severity: u32) -> Result<(), Error> {
    if severity < MIN_SEVERITY || severity > MAX_SEVERITY {
        return Err(Error::InvalidSeverity);
    }
    Ok(())
}
```

## Minimum Validation Checklist

Every public function in a contract should validate:

- [ ] **String parameters**: Length bounds checked
- [ ] **Address parameters**: Proper format (SDK validates construction)
- [ ] **Numeric parameters**: Range checked, sign validated if relevant
- [ ] **Collection parameters**: Size validated, non-empty if required
- [ ] **Enum parameters**: Within valid range
- [ ] **Reference parameters**: Length validated, format checked

## Contract-Specific Guidelines

### Reputation System (`contracts/reputation/`)

**Functions**: `initialize()`, `mint()`, `slash()`

| Function | Parameter | Validation |
|----------|-----------|-----------|
| mint | amount | Must be positive (>0), max i128::MAX |
| slash | amount | Must be positive (>0), max i128::MAX |

**Error Types Added**:
- `NegativeAmount = 3`: Amount cannot be negative
- `InvalidAmount = 4`: Amount must be non-zero

**Status**: ✅ Implemented

### Clinical Trial (`contracts/clinical_trial/`)

**Functions**: `create_protocol()`, `register_site()`, `record_consent()`, `report_adverse_event()`

| Function | Parameter | Validation |
|----------|-----------|-----------|
| create_protocol | title | 3-256 characters |
| create_protocol | metadata_ref | 10-256 characters (IPFS hash) |
| create_protocol | max_participants | Must be > 0 |
| register_site | name | 2-128 characters |
| record_consent | consent_ref | 10-256 characters |
| report_adverse_event | severity | 0-10 (valid severity level) |
| report_adverse_event | description_ref | 10-256 characters |

**Constants Defined**:
```rust
const MIN_TITLE_LENGTH: u32 = 3;
const MAX_TITLE_LENGTH: u32 = 256;
const MIN_NAME_LENGTH: u32 = 2;
const MAX_NAME_LENGTH: u32 = 128;
const MIN_REF_LENGTH: u32 = 10;
const MAX_REF_LENGTH: u32 = 256;
const MIN_SEVERITY: u32 = 0;
const MAX_SEVERITY: u32 = 10;
```

**Error Types Added**:
- `InvalidTitle = 3`
- `InvalidMetadataRef = 4`
- `InvalidName = 5`
- `InvalidConsentRef = 6`
- `InvalidMaxParticipants = 7`
- `InvalidDescriptionRef = 8`
- `InvalidSeverity = 9`

**Status**: ✅ Implemented

## Testing Strategy

### Unit Tests

Every validation function should have corresponding unit tests:

```rust
#[test]
fn test_validate_title_valid() {
    let title = String::from_str(&env, "Valid Protocol Title");
    assert!(validate_title(&title).is_ok());
}

#[test]
fn test_validate_title_too_short() {
    let title = String::from_str(&env, "AB");
    assert_eq!(validate_title(&title), Err(Error::InvalidTitle));
}
```

### Property-Based Tests

Validation functions should include property-based tests demonstrating:

1. **Monotonic Property**: If a value passes validation, all "better" values also pass
2. **Boundary Property**: Boundary values are handled correctly
3. **Consistency Property**: Validation always produces same result for same input
4. **Exclusive Property**: Invalid values are always rejected

**Example**:
```rust
#[test]
fn property_range_boundaries_inclusive() {
    let min = 100u64;
    let max = 200u64;
    
    assert!(validate_u64_range(min, min, max).is_ok());
    assert!(validate_u64_range(max, min, max).is_ok());
    assert!(validate_u64_range(min - 1, min, max).is_err());
}
```

**Location**: `libs/validation_utils/tests/validation_tests.rs`

## Best Practices

### 1. Validate Early

Perform all validation at the start of public functions before any state modifications:

```rust
pub fn create_protocol(env: Env, title: String) -> Result<(), Error> {
    // Validate first
    validate_title(&title)?;
    
    // Then proceed with business logic
    // ...
}
```

### 2. Provide Clear Error Types

Use specific error variants for different validation failures:

```rust
pub enum Error {
    InvalidTitle = 3,        // Too short/long/empty
    InvalidAmount = 4,       // Negative or zero
    InvalidMaxParticipants = 5, // Zero or too large
}
```

### 3. Use Constants for Bounds

Define validation constants at the module level:

```rust
const MIN_TITLE_LENGTH: u32 = 3;
const MAX_TITLE_LENGTH: u32 = 256;
```

### 4. Document Validation Requirements

Include validation constraints in function documentation:

```rust
/// Creates a new protocol
/// 
/// # Arguments
/// * `title` - Protocol title (3-256 characters)
/// * `max_participants` - Maximum participants (must be > 0)
///
/// # Returns
/// `Ok(id)` on success, error if validation fails
pub fn create_protocol(...) -> Result<u64, Error>
```

### 5. Handle Edge Cases

Always consider boundary conditions:
- Empty strings
- Zero values
- Maximum values
- Special addresses (though Soroban handles most)

## Migration Plan for Existing Contracts

For contracts without validation, migration should:

1. **Add error types** for validation failures
2. **Define validation constants** for bounds
3. **Create validation functions** or use `validation_utils`
4. **Update function signatures** to return `Result` if currently returning `void`
5. **Add validation calls** at start of public functions
6. **Add tests** for validation logic

### Example Migration

**Before**:
```rust
pub fn create_item(env: Env, name: String) -> u64 {
    // No validation
    let id = next_id(&env);
    // ... store item
    id
}
```

**After**:
```rust
pub fn create_item(env: Env, name: String) -> Result<u64, Error> {
    // Validate input
    validate_name(&name)?;
    
    let id = next_id(&env);
    // ... store item
    Ok(id)
}
```

## Maintenance

### Regular Audits

Perform quarterly audits to identify contracts needing validation:

```bash
# Search for contracts without Result error handling
grep -r "pub fn " contracts/ | grep -v "Result<" | grep -v "read-only"
```

### Update ValidationUtils

As new validation patterns emerge:

1. Add new validation function to `libs/validation_utils/`
2. Include comprehensive documentation
3. Add property-based tests
4. Update this guide

## References

- [Soroban SDK Documentation](https://soroban.stellar.org)
- [ValidationUtils Module](../../libs/validation_utils/src/)
- [Reputation Contract](../../contracts/reputation/src/lib.rs)
- [Clinical Trial Contract](../../contracts/clinical_trial/src/lib.rs)

## Conclusion

Consistent input validation is critical for contract security and reliability. By following these standards and using the shared `ValidationUtils` module, all contracts in the Uzima platform can maintain high-quality validation with minimal duplication.
