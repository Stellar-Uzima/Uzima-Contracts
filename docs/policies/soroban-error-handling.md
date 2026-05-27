# Soroban Error Handling Policy

Purpose
- Provide a small, consistent policy for propagating and asserting errors from Soroban contract utilities used in tests and examples.

Policy
- Use a shared `ContractError` type for utility code: `Result<T, ContractError>`.
- Convert low-level numeric codes (e.g. `i32`) into `ContractError::Code(i32)` at the boundary.
- For human-readable failures use `ContractError::Message(String)`.
- Do not expose internal or sensitive state in error messages.

Logging
- Log non-sensitive diagnostic messages in tests only. Contract production code should avoid verbose logs.

Guidance for utility authors
- Export the shared `ContractError` from `tests/utils/contract_error.rs` and use `ContractResult<T> = Result<T, ContractError>`.
- Provide helper functions to assert error codes and convert legacy `i32` results to `ContractError` where needed.

Example
```rust
pub type ContractResult<T> = Result<T, ContractError>;

pub enum ContractError {
    Code(i32),
    Message(String),
}
```

Reference
- Include a brief reference to this policy in `CONTRIBUTING.md` and follow it when adding or updating contract utilities.
