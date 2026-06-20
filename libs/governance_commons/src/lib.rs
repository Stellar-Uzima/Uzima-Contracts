#![no_std]

//! # Governance Commons Library
//!
//! This library provides shared abstractions and utilities for governance-related contracts:
//! - **DisputeResolution**: Handles disputes on proposals via arbiters
//! - **Governor**: Handles voting and proposal lifecycle
//! - **UpgradeManager**: Handles contract upgrades with approvals
//! - **EmergencyAccessOverride**: Handles emergency access with multi-sig approvals
//!
//! ## Separation of Concerns
//!
//! Each contract has a single, well-defined responsibility:
//!
//! | Contract | Responsibility | Approval Type |
//! |----------|-----------------|--------------|
//! | DisputeResolution | Challenge proposals, resolution by arbiters | Arbiter consensus |
//! | Governor | Token-based voting, proposal lifecycle | Quorum + voting |
//! | UpgradeManager | Contract upgrades with validators | Validator multi-sig |
//! | EmergencyAccessOverride | Emergency medical access | Approver multi-sig |
//! | Timelock | Time-delay execution gate | Time passage |
//!
//! ## Decision Tree: Which Approval Mechanism to Use
//!
//! ```text
//! Does it involve VOTING on a PROPOSAL?
//!   ├─ YES → Use GOVERNOR
//!   │        (voting delay, voting period, quorum, voting power)
//!   │
//!   └─ NO → Does it need DISPUTE RESOLUTION?
//!            ├─ YES → Use DISPUTE_RESOLUTION
//!            │        (arbiters can override proposals)
//!            │
//!            └─ NO → Does it need TIME-DELAY execution gate?
//!                     ├─ YES → Use TIMELOCK
//!                     │        (required before sensitive operations)
//!                     │
//!                     └─ NO → Does it need MULTI-SIG approval from fixed set?
//!                              ├─ YES → Use MULTI_SIG module
//!                              │        (validators, approvers, etc.)
//!                              │
//!                              └─ NO → Use custom authorization
//!                                       (require_auth, specific roles)
//! ```

pub mod errors;
pub mod multi_sig;
pub mod types;

pub use errors::*;
pub use multi_sig::*;
pub use types::*;
