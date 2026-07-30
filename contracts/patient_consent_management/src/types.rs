//! Re-export of public types from the Patient Consent Management contract.
//!
//! All primary types (ConsentRecord, ConsentLog, DataKey, etc.) are defined
//! in `lib.rs` alongside the contract implementation for ABI compatibility.
//! This module exists to satisfy the standard contract structure convention
//! (lib.rs + storage.rs + errors.rs + events.rs + types.rs).
//!
//! See `lib.rs` for:
//! - `ConsentRecord` — patient consent grant record
//! - `ConsentLog` — accumulated consent history
//! - `DataKey` — persistent storage keys
//! - `ProxyScope`, `ProxyRecord`, `ProxyKey` — proxy delegation types
