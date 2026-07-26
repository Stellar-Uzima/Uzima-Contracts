//! Type documentation for the Medical Records contract.
//!
//! All primary types (MedicalRecord, FilteredRecord, DataCategory, etc.) are
//! defined in `lib.rs` alongside the contract implementation for ABI
//! compatibility. This module exists to satisfy the standard contract
//! structure convention (lib.rs + storage.rs + errors.rs + events.rs + types.rs).
//!
//! ## Key Types (defined in lib.rs)
//!
//! ### Core Record Types
//! - `MedicalRecord` — primary patient medical record
//! - `FilteredRecord` — HIPAA minimum-necessary filtered view
//! - `RecordMetadata` — versioned metadata
//!
//! ### Access Control
//! - `Role`, `RbacRole` — user role definitions
//! - `Permission`, `PermissionGrant` — granular access control
//! - `DataCategory` — HIPAA field-level access categories
//!
//! ### Cross-Chain
//! - `ChainId`, `CrossChainRecordRef` — cross-chain sync
//!
//! ### Cryptography
//! - `EncryptedRecord`, `KeyEnvelope`, `EnvelopeAlgorithm` — E2E encryption
//! - `ZkPublicInputs`, `ZkAccessGrant` — zero-knowledge proofs
//!
//! ### Data Quality
//! - `DataQualityScore`, `ValidationIssue`, `ValidationReport`
//! - `CorrectionWorkflow`, `CleanseResult`
