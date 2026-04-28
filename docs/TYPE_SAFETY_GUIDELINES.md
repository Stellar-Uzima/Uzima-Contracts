# Type Safety Guidelines for Soroban Smart Contracts

This guide outlines best practices for maintaining type safety in the Uzima smart contracts ecosystem, focusing on Rust-based Soroban development. Ensuring strict type safety is critical for preventing vulnerabilities and ensuring contract reliability.

## 1. Type System Overview

Soroban uses a strictly typed system where data is passed between the host and the VM using `ScVal`. In Rust contracts, we work with high-level wrappers provided by the `soroban-sdk`.

### Core Types
- **Primitive Types**: `u32`, `i32`, `u64`, `i64`, `u128`, `i128`, `bool`.
- **Soroban Types**:
    - `Address`: Represents a Stellar account or contract ID.
    - `Symbol`: Short strings (up to 32 characters) stored as 64-bit integers.
    - `Bytes` / `BytesN<N>`: Arbitrary data buffers.
    - `String`: High-level string wrapper.
    - `Vec<T>` / `Map<K, V>`: Host-managed collections.

## 2. Custom Type Patterns

Use the `#[contracttype]` attribute to define custom structs and enums that can be serialized and stored in the contract's state.

### Structs
Use named fields for clarity and avoid excessively large structs to stay within memory limits.

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PatientRecord {
    pub id: u64,
    pub patient: Address,
    pub doctor: Address,
    pub encrypted_data: Bytes,
    pub timestamp: u64,
}
```

### Enums
Use enums for role-based access control and state management.

```rust
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserRole {
    Patient = 0,
    Doctor = 1,
    Admin = 2,
}
```

## 3. Validation Strategies

Type safety is not just about types, but also about ensuring data integrity during execution.

### Auth Checks
Always verify the caller's identity before performing sensitive operations.

```rust
pub fn update_record(env: Env, caller: Address, record_id: u64) {
    caller.require_auth();
    // Proceed with logic...
}
```

### Bounds and Range Checking
Validate that input values fall within expected ranges to prevent overflows or logic errors.

```rust
pub fn set_data(env: Env, value: u32) {
    if value > 1000 {
        panic_with_error!(&env, Error::ValueExceedsLimit);
    }
}
```

### Sanitization
Always check the length of `Bytes` and `String` inputs.

```rust
pub fn store_metadata(env: Env, data: Bytes) {
    if data.len() > 256 {
        panic!("Metadata too long");
    }
}
```

## 4. Error Type Hierarchies

Define a central `Error` enum using `#[contracterror]` to provide meaningful feedback to users and frontends.

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    InvalidId = 2,
    AlreadyInitialized = 3,
    LimitExceeded = 4,
}
```

## 5. Generic Type Usage

While Rust supports generics, be mindful of their impact on WASM binary size. Use traits for shared behavior instead of complex generic structures where possible.

```rust
pub trait AccessControlled {
    fn check_access(&self, user: Address) -> bool;
}

impl AccessControlled for PatientRecord {
    fn check_access(&self, user: Address) -> bool {
        self.patient == user || self.doctor == user
    }
}
```

## 6. Type Conversion Best Practices

Safe conversions are essential for moving data between host types and contract logic.

### into_val and from_val
Use `.into_val(env)` and `.from_val(env)` for conversions between primitive and host types.

```rust
let my_val: u32 = 42;
let host_val: Val = my_val.into_val(&env);
let recovered: u32 = u32::from_val(&env, &host_val);
```

### Safer Conversions
Prefer `try_from` or explicit checks when converting between numeric types of different sizes.

```rust
let large_val: u128 = 1000;
let small_val: u32 = large_val.try_into().map_err(|_| Error::ConversionError)?;
```

---

## Team Onboarding Integration
New developers should review these guidelines before contributing to ensure all new contract code adheres to our safety standards. Use `cargo clippy` to catch common type-related pitfalls.
