//! Type documentation for the Audit Forensics contract.
//!
//! All primary types (AuditAction, AuditEntry, ForensicReport, etc.) are
//! defined in `lib.rs` alongside the contract implementation for ABI
//! compatibility. This module exists to satisfy the standard contract
//! structure convention (lib.rs + storage.rs + errors.rs + events.rs + types.rs).
//!
//! See `lib.rs` for:
//! - `AuditAction` — actions that can be audited
//! - `AuditEntry` — individual audit log entries
//! - `ForensicReport` — forensic analysis reports
//! - `AuditRule` — automated audit rules
//! - `VulnerabilityFinding` — detected security findings
//! - `AnalysisExecution` — automated analysis runs
//! - `FormalVerificationSummary` — formal verification results
//! - `DataKey` — persistent storage keys
