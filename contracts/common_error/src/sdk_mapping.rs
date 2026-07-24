//! sdk_mapping - SDK-facing error taxonomy and code mapping.
//!
//! This module centralises the mapping from Soroban contract error discriminants
//! to SDK-friendly error objects that client applications can consume.  Every
//! on-chain error code has exactly one entry in this table.
//!
//! ## Design
//!
//! Soroban contract errors are returned as `u32` discriminants over XDR.
//! Client SDKs (TypeScript, Python, mobile) need human-readable codes,
//! HTTP-style status categories, and remediation hints to surface useful
//! messages to end users.
//!
//! This module provides:
//! - `SdkError` — the canonical SDK-facing error representation
//! - `error_from_code()` — maps a `u32` discriminant to an `SdkError`
//! - `FULL_TAXONOMY` — the complete error taxonomy as a static slice

use soroban_sdk::{contracttype, String as SorobanString, Symbol};

// ──────────────────────────────────────────────────────────────────────────────
// SDK Error type
// ──────────────────────────────────────────────────────────────────────────────

/// Category of an SDK error (maps to HTTP-like status codes for SDK consumers).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ErrorCategory {
    /// Client-side input or auth error (4xx equivalent).
    ClientError = 1,
    /// Contract state conflict (409 equivalent).
    ConflictError = 2,
    /// Rate limiting or temporary availability (429/503 equivalent).
    TransientError = 3,
    /// Contract-level internal or unknown error (5xx equivalent).
    InternalError = 4,
    /// Resource not found (404 equivalent).
    NotFound = 5,
    /// Forbidden / access denied (403 equivalent).
    Forbidden = 6,
}

/// The canonical SDK-facing error representation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SdkError {
    /// The on-chain `u32` discriminant.
    pub code: u32,
    /// Short machine-readable identifier (snake_case).
    pub slug: &'static str,
    /// Human-readable message for end-user display.
    pub message: &'static str,
    /// SDK-facing error category.
    pub category: ErrorCategory,
    /// Optional remediation hint for the SDK consumer.
    pub hint: Option<&'static str>,
    /// Link to documentation for this error.
    pub doc_link: &'static str,
}

// ──────────────────────────────────────────────────────────────────────────────
// Full error taxonomy
// ──────────────────────────────────────────────────────────────────────────────

/// Complete error taxonomy mapping every contract discriminant range
/// to SDK-facing error metadata.
pub static FULL_TAXONOMY: &[SdkError] = &[
    // ── Common errors (0-99) ─────────────────────────────────────────────────
    SdkError {
        code: 0,
        slug: "unknown",
        message: "An unknown error occurred.",
        category: ErrorCategory::InternalError,
        hint: Some("Contact support if this persists."),
        doc_link: "https://docs.uzima.health/errors#unknown",
    },
    SdkError {
        code: 1,
        slug: "unauthorized",
        message: "The caller is not authorized to perform this action.",
        category: ErrorCategory::Forbidden,
        hint: Some("Check that you are using the correct key and have the required role."),
        doc_link: "https://docs.uzima.health/errors#unauthorized",
    },
    SdkError {
        code: 2,
        slug: "not_initialized",
        message: "The contract has not been initialized yet.",
        category: ErrorCategory::ClientError,
        hint: Some("Call initialize() before invoking other entrypoints."),
        doc_link: "https://docs.uzima.health/errors#not_initialized",
    },
    SdkError {
        code: 3,
        slug: "already_initialized",
        message: "The contract is already initialized.",
        category: ErrorCategory::ConflictError,
        hint: None,
        doc_link: "https://docs.uzima.health/errors#already_initialized",
    },
    SdkError {
        code: 4,
        slug: "contract_paused",
        message: "The contract is temporarily paused.",
        category: ErrorCategory::TransientError,
        hint: Some("Wait for the contract to resume or contact the administrator."),
        doc_link: "https://docs.uzima.health/errors#contract_paused",
    },
    SdkError {
        code: 5,
        slug: "deadline_exceeded",
        message: "The operation deadline has passed.",
        category: ErrorCategory::ClientError,
        hint: Some("Resubmit the transaction with a future deadline."),
        doc_link: "https://docs.uzima.health/errors#deadline_exceeded",
    },
    SdkError {
        code: 6,
        slug: "rate_limit_exceeded",
        message: "Rate limit exceeded. Too many requests.",
        category: ErrorCategory::TransientError,
        hint: Some("Wait before retrying. Use exponential backoff."),
        doc_link: "https://docs.uzima.health/errors#rate_limit_exceeded",
    },
    SdkError {
        code: 7,
        slug: "insufficient_funds",
        message: "Insufficient funds to complete the operation.",
        category: ErrorCategory::ClientError,
        hint: Some("Add more XLM to your account and retry."),
        doc_link: "https://docs.uzima.health/errors#insufficient_funds",
    },
    SdkError {
        code: 8,
        slug: "invalid_input",
        message: "One or more input parameters are invalid.",
        category: ErrorCategory::ClientError,
        hint: Some("Review the API documentation for valid input formats."),
        doc_link: "https://docs.uzima.health/errors#invalid_input",
    },
    SdkError {
        code: 9,
        slug: "invalid_state",
        message: "The contract is in an invalid state for this operation.",
        category: ErrorCategory::ConflictError,
        hint: Some("Check the current contract state before retrying."),
        doc_link: "https://docs.uzima.health/errors#invalid_state",
    },
    SdkError {
        code: 10,
        slug: "not_found",
        message: "The requested resource was not found.",
        category: ErrorCategory::NotFound,
        hint: Some("Verify the ID or address and try again."),
        doc_link: "https://docs.uzima.health/errors#not_found",
    },
    SdkError {
        code: 11,
        slug: "access_denied",
        message: "Access to this resource is denied.",
        category: ErrorCategory::Forbidden,
        hint: Some("Ensure you have the required permissions."),
        doc_link: "https://docs.uzima.health/errors#access_denied",
    },
    SdkError {
        code: 12,
        slug: "timeout",
        message: "The operation timed out.",
        category: ErrorCategory::TransientError,
        hint: Some("Retry the operation."),
        doc_link: "https://docs.uzima.health/errors#timeout",
    },
    SdkError {
        code: 13,
        slug: "invalid_argument",
        message: "An argument to this call is invalid.",
        category: ErrorCategory::ClientError,
        hint: Some("Check all argument types and values."),
        doc_link: "https://docs.uzima.health/errors#invalid_argument",
    },
    SdkError {
        code: 14,
        slug: "external_contract_not_set",
        message: "A required external contract address has not been configured.",
        category: ErrorCategory::InternalError,
        hint: Some("Configure the external contract address via the admin entrypoint."),
        doc_link: "https://docs.uzima.health/errors#external_contract_not_set",
    },
    SdkError {
        code: 15,
        slug: "invalid_data",
        message: "The supplied data is invalid or corrupted.",
        category: ErrorCategory::ClientError,
        hint: None,
        doc_link: "https://docs.uzima.health/errors#invalid_data",
    },
    SdkError {
        code: 16,
        slug: "invalid_payload",
        message: "The request payload is malformed.",
        category: ErrorCategory::ClientError,
        hint: Some("Validate your payload against the contract ABI."),
        doc_link: "https://docs.uzima.health/errors#invalid_payload",
    },
    SdkError {
        code: 17,
        slug: "duplicate_submission",
        message: "This request has already been submitted.",
        category: ErrorCategory::ConflictError,
        hint: Some("Check for idempotency keys and avoid re-submitting."),
        doc_link: "https://docs.uzima.health/errors#duplicate_submission",
    },
    SdkError {
        code: 18,
        slug: "unauthorized_caller",
        message: "The calling contract or account is not authorized.",
        category: ErrorCategory::Forbidden,
        hint: Some("Check cross-contract authorization configuration."),
        doc_link: "https://docs.uzima.health/errors#unauthorized_caller",
    },
    // ── Lifecycle errors (200-204) ────────────────────────────────────────────
    SdkError {
        code: 200,
        slug: "lifecycle_not_initialized",
        message: "Contract lifecycle: not initialized.",
        category: ErrorCategory::ClientError,
        hint: Some("Call initialize() first."),
        doc_link: "https://docs.uzima.health/errors#lifecycle_not_initialized",
    },
    SdkError {
        code: 201,
        slug: "lifecycle_paused",
        message: "Contract lifecycle: paused by administrator.",
        category: ErrorCategory::TransientError,
        hint: Some("Wait for the contract to resume."),
        doc_link: "https://docs.uzima.health/errors#lifecycle_paused",
    },
    SdkError {
        code: 202,
        slug: "lifecycle_upgrade_in_progress",
        message: "Contract lifecycle: upgrade in progress.",
        category: ErrorCategory::TransientError,
        hint: Some("Retry after the upgrade completes."),
        doc_link: "https://docs.uzima.health/errors#lifecycle_upgrade_in_progress",
    },
    SdkError {
        code: 203,
        slug: "lifecycle_deprecated",
        message: "Contract has been permanently deprecated.",
        category: ErrorCategory::InternalError,
        hint: Some("Migrate to the replacement contract."),
        doc_link: "https://docs.uzima.health/errors#lifecycle_deprecated",
    },
    SdkError {
        code: 204,
        slug: "lifecycle_invalid_transition",
        message: "Invalid lifecycle state transition requested.",
        category: ErrorCategory::ConflictError,
        hint: None,
        doc_link: "https://docs.uzima.health/errors#lifecycle_invalid_transition",
    },
    // ── Upgrade errors (100-108) ──────────────────────────────────────────────
    SdkError {
        code: 100,
        slug: "upgrade_not_authorized",
        message: "Not authorized to perform the upgrade.",
        category: ErrorCategory::Forbidden,
        hint: Some("Only the contract admin can perform upgrades."),
        doc_link: "https://docs.uzima.health/errors#upgrade_not_authorized",
    },
    SdkError {
        code: 101,
        slug: "invalid_wasm_hash",
        message: "The WASM hash provided is invalid.",
        category: ErrorCategory::ClientError,
        hint: Some("Provide a 32-byte SHA-256 hash of the new WASM binary."),
        doc_link: "https://docs.uzima.health/errors#invalid_wasm_hash",
    },
    SdkError {
        code: 107,
        slug: "integrity_check_failed",
        message: "Contract integrity check failed during upgrade.",
        category: ErrorCategory::InternalError,
        hint: None,
        doc_link: "https://docs.uzima.health/errors#integrity_check_failed",
    },
];

// ──────────────────────────────────────────────────────────────────────────────
// Lookup function
// ──────────────────────────────────────────────────────────────────────────────

/// Returns the `SdkError` for the given on-chain error discriminant.
///
/// Returns `None` if the code is not registered in the taxonomy.
pub fn error_from_code(code: u32) -> Option<&'static SdkError> {
    FULL_TAXONOMY.iter().find(|e| e.code == code)
}

/// Returns the error category for the given discriminant.
///
/// Returns `ErrorCategory::InternalError` for unknown codes.
pub fn category_from_code(code: u32) -> ErrorCategory {
    error_from_code(code)
        .map(|e| e.category)
        .unwrap_or(ErrorCategory::InternalError)
}

/// Returns `true` if the error is transient and the client should retry.
pub fn is_retryable(code: u32) -> bool {
    matches!(
        category_from_code(code),
        ErrorCategory::TransientError
    )
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_common_errors() {
        let e = error_from_code(1).unwrap();
        assert_eq!(e.slug, "unauthorized");
        assert_eq!(e.category, ErrorCategory::Forbidden);
    }

    #[test]
    fn test_unknown_code_returns_none() {
        assert!(error_from_code(9999).is_none());
    }

    #[test]
    fn test_retryable_transient_errors() {
        assert!(is_retryable(4));  // contract_paused
        assert!(is_retryable(6));  // rate_limit_exceeded
        assert!(!is_retryable(1)); // unauthorized
        assert!(!is_retryable(10)); // not_found
    }

    #[test]
    fn test_all_taxonomy_codes_unique() {
        let mut codes: alloc::vec::Vec<u32> = FULL_TAXONOMY.iter().map(|e| e.code).collect();
        codes.sort_unstable();
        let original_len = codes.len();
        codes.dedup();
        assert_eq!(codes.len(), original_len, "Duplicate error codes found in taxonomy");
    }
}
