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

### Clippy Lint Configuration

Clippy is enforced in CI with both `clippy::pedantic` and `clippy::nursery` lint groups enabled. The CI command (`.github/workflows/ci.yml`) runs:

```shell
cargo clippy --workspace --all-targets -- \
    -D warnings \
    -W clippy::pedantic \
    -W clippy::nursery \
    -A clippy::bool_to_int_with_if \
    -A clippy::cast_lossless \
    -A clippy::cast_possible_truncation \
    -A clippy::cast_possible_wrap \
    -A clippy::cast_precision_loss \
    -A clippy::cast_sign_loss \
    -A dead_code \
    -A clippy::derive_partial_eq_without_eq \
    -A clippy::doc_markdown \
    -A clippy::double_must_use \
    -A clippy::duplicated_attributes \
    -A clippy::explicit_iter_loop \
    -A clippy::fn_params_excessive_bools \
    -A clippy::format_push_string \
    -A clippy::ignored_unit_patterns \
    -A clippy::inconsistent_struct_constructor \
    -A clippy::items_after_statements \
    -A clippy::let_and_return \
    -A clippy::manual_assert \
    -A clippy::manual_let_else \
    -A clippy::map_unwrap_or \
    -A clippy::match_same_arms \
    -A clippy::match_wildcard_for_single_variants \
    -A mismatched_lifetime_syntaxes \
    -A clippy::missing_const_for_fn \
    -A clippy::missing_errors_doc \
    -A clippy::missing_panics_doc \
    -A clippy::must_use_candidate \
    -A clippy::needless_pass_by_value \
    -A clippy::needless_raw_string_hashes \
    -A clippy::option_if_let_else \
    -A clippy::or_fun_call \
    -A clippy::redundant_closure_for_method_calls \
    -A clippy::redundant_clone \
    -A clippy::result_unit_err \
    -A clippy::similar_names \
    -A clippy::single_match \
    -A clippy::struct_excessive_bools \
    -A clippy::struct_field_names \
    -A clippy::suboptimal_flops \
    -A clippy::too_long_first_doc_paragraph \
    -A clippy::too_many_lines \
    -A clippy::trivially_copy_pass_by_ref \
    -A clippy::uninlined_format_args \
    -A clippy::unnecessary_cast \
    -A clippy::unnecessary_semicolon \
    -A clippy::unnecessary_wraps \
    -A clippy::unnested_or_patterns \
    -A clippy::unreadable_literal \
    -A clippy::unused_async \
    -A clippy::use_self \
    -A clippy::used_underscore_binding \
    -A clippy::useless_let_if_seq \
    -A clippy::wildcard_imports
```

A PR **fails** if it introduces any new `clippy::pedantic` or `clippy::nursery`
warning that is not in the explicit allowlist above. The allowed exceptions are
documented with rationale in `clippy.toml`.

### Adding or removing allowlisted lints

1. Update the `-A <lint>` list in `.github/workflows/ci.yml`.
2. Add or update the corresponding rationale comment in `clippy.toml`.
3. Update this section to reflect the change.

### Local development

Run the exact CI command locally before pushing:

```shell
cargo clippy --workspace --all-targets -- \
    -D warnings \
    -W clippy::pedantic \
    -W clippy::nursery \
    -A clippy::bool_to_int_with_if \
    -A clippy::cast_lossless \
    -A clippy::cast_possible_truncation \
    -A clippy::cast_possible_wrap \
    -A clippy::cast_precision_loss \
    -A clippy::cast_sign_loss \
    -A dead_code \
    -A clippy::derive_partial_eq_without_eq \
    -A clippy::doc_markdown \
    -A clippy::double_must_use \
    -A clippy::duplicated_attributes \
    -A clippy::explicit_iter_loop \
    -A clippy::fn_params_excessive_bools \
    -A clippy::format_push_string \
    -A clippy::ignored_unit_patterns \
    -A clippy::inconsistent_struct_constructor \
    -A clippy::items_after_statements \
    -A clippy::let_and_return \
    -A clippy::manual_assert \
    -A clippy::manual_let_else \
    -A clippy::map_unwrap_or \
    -A clippy::match_same_arms \
    -A clippy::match_wildcard_for_single_variants \
    -A mismatched_lifetime_syntaxes \
    -A clippy::missing_const_for_fn \
    -A clippy::missing_errors_doc \
    -A clippy::missing_panics_doc \
    -A clippy::must_use_candidate \
    -A clippy::needless_pass_by_value \
    -A clippy::needless_raw_string_hashes \
    -A clippy::option_if_let_else \
    -A clippy::or_fun_call \
    -A clippy::redundant_closure_for_method_calls \
    -A clippy::redundant_clone \
    -A clippy::result_unit_err \
    -A clippy::similar_names \
    -A clippy::single_match \
    -A clippy::struct_excessive_bools \
    -A clippy::struct_field_names \
    -A clippy::suboptimal_flops \
    -A clippy::too_long_first_doc_paragraph \
    -A clippy::too_many_lines \
    -A clippy::trivially_copy_pass_by_ref \
    -A clippy::uninlined_format_args \
    -A clippy::unnecessary_cast \
    -A clippy::unnecessary_semicolon \
    -A clippy::unnecessary_wraps \
    -A clippy::unnested_or_patterns \
    -A clippy::unreadable_literal \
    -A clippy::unused_async \
    -A clippy::use_self \
    -A clippy::used_underscore_binding \
    -A clippy::useless_let_if_seq \
    -A clippy::wildcard_imports
```

## Updates
This document should be updated as patterns evolve. Major changes require team consensus.