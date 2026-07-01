//! Error types for the Audit Forensics contract.
//!
//! All contract errors live here for consistency with the standard
//! module layout (lib.rs + storage.rs + errors.rs + events.rs + types.rs).

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AuditForensicsError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    RuleNotFound = 4,
    ExecutionNotFound = 5,
    FindingNotFound = 6,
    UnsupportedAction = 7,
}
