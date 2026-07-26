#![cfg(test)]

use crate::{CommonError, get_suggestion, is_common_error_code, COMMON_ERROR_MAX};
use soroban_sdk::{symbol_short, Env};

/// Verifies every variant discriminant matches the golden (canonical) values.
/// If this test fails after adding a new variant, update both the enum and this
/// test to lock in the new discriminant.
#[test]
fn test_golden_error_discriminants() {
    assert_eq!(CommonError::Unknown as u32, 0);
    assert_eq!(CommonError::Unauthorized as u32, 1);
    assert_eq!(CommonError::NotInitialized as u32, 2);
    assert_eq!(CommonError::AlreadyInitialized as u32, 3);
    assert_eq!(CommonError::ContractPaused as u32, 4);
    assert_eq!(CommonError::DeadlineExceeded as u32, 5);
    assert_eq!(CommonError::RateLimitExceeded as u32, 6);
    assert_eq!(CommonError::InsufficientFunds as u32, 7);
    assert_eq!(CommonError::InvalidInput as u32, 8);
    assert_eq!(CommonError::InvalidState as u32, 9);
    assert_eq!(CommonError::NotFound as u32, 10);
    assert_eq!(CommonError::AccessDenied as u32, 11);
    assert_eq!(CommonError::Timeout as u32, 12);
    assert_eq!(CommonError::InvalidArgument as u32, 13);
    assert_eq!(CommonError::ExternalContractNotSet as u32, 14);
    assert_eq!(CommonError::InvalidData as u32, 15);
    assert_eq!(CommonError::InvalidPayload as u32, 16);
    assert_eq!(CommonError::DuplicateSubmission as u32, 17);
    assert_eq!(CommonError::UnauthorizedCaller as u32, 18);
}

/// Golden test for the suggestion mapping.
#[test]
fn test_golden_suggestion_mappings() {
    let env = Env::default();

    // Each (error, expected_hint) pair must match what `get_suggestion` returns.
    let cases: [(CommonError, soroban_sdk::Symbol); 19] = [
        (CommonError::Unknown, symbol_short!("CONTACT")),
        (CommonError::Unauthorized, symbol_short!("CHK_AUTH")),
        (CommonError::NotInitialized, symbol_short!("INIT_CTR")),
        (CommonError::AlreadyInitialized, symbol_short!("ALREADY")),
        (CommonError::ContractPaused, symbol_short!("RE_TRY_L")),
        (CommonError::DeadlineExceeded, symbol_short!("CONTACT")),
        (CommonError::RateLimitExceeded, symbol_short!("RE_TRY_L")),
        (CommonError::InsufficientFunds, symbol_short!("ADD_FUND")),
        (CommonError::InvalidInput, symbol_short!("CHK_DATA")),
        (CommonError::InvalidState, symbol_short!("CONTACT")),
        (CommonError::NotFound, symbol_short!("CHK_ID")),
        (CommonError::AccessDenied, symbol_short!("CONTACT")),
        (CommonError::Timeout, symbol_short!("RE_TRY_L")),
        (CommonError::InvalidArgument, symbol_short!("CHK_DATA")),
        (CommonError::ExternalContractNotSet, symbol_short!("CONTACT")),
        (CommonError::InvalidData, symbol_short!("CHK_DATA")),
        (CommonError::InvalidPayload, symbol_short!("CHK_DATA")),
        (CommonError::DuplicateSubmission, symbol_short!("CONTACT")),
        (CommonError::UnauthorizedCaller, symbol_short!("CHK_AUTH")),
    ];

    for (error, expected) in cases {
        assert_eq!(
            get_suggestion(error),
            expected,
            "Suggestion mismatch for {:?}",
            error,
        );
    }
}

/// Ensures every variant discriminant is within the reserved range.
#[test]
fn test_all_discriminants_within_range() {
    let variants: [CommonError; 19] = [
        CommonError::Unknown,
        CommonError::Unauthorized,
        CommonError::NotInitialized,
        CommonError::AlreadyInitialized,
        CommonError::ContractPaused,
        CommonError::DeadlineExceeded,
        CommonError::RateLimitExceeded,
        CommonError::InsufficientFunds,
        CommonError::InvalidInput,
        CommonError::InvalidState,
        CommonError::NotFound,
        CommonError::AccessDenied,
        CommonError::Timeout,
        CommonError::InvalidArgument,
        CommonError::ExternalContractNotSet,
        CommonError::InvalidData,
        CommonError::InvalidPayload,
        CommonError::DuplicateSubmission,
        CommonError::UnauthorizedCaller,
    ];

    for variant in variants {
        let code = variant as u32;
        assert!(
            is_common_error_code(code),
            "Variant {:?} with code {} exceeds COMMON_ERROR_MAX ({}))",
            variant,
            code,
            COMMON_ERROR_MAX,
        );
    }
}

/// Golden test for the suggestion fallback: variants that map to the default
/// suggestion should still produce a valid Symbol.
#[test]
fn test_golden_suggestion_never_panics() {
    let variants: [CommonError; 19] = [
        CommonError::Unknown,
        CommonError::Unauthorized,
        CommonError::NotInitialized,
        CommonError::AlreadyInitialized,
        CommonError::ContractPaused,
        CommonError::DeadlineExceeded,
        CommonError::RateLimitExceeded,
        CommonError::InsufficientFunds,
        CommonError::InvalidInput,
        CommonError::InvalidState,
        CommonError::NotFound,
        CommonError::AccessDenied,
        CommonError::Timeout,
        CommonError::InvalidArgument,
        CommonError::ExternalContractNotSet,
        CommonError::InvalidData,
        CommonError::InvalidPayload,
        CommonError::DuplicateSubmission,
        CommonError::UnauthorizedCaller,
    ];

    for variant in variants {
        let _hint = get_suggestion(variant);
    }
}
