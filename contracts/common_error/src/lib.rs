#![no_std]
//! common_error - Healthcare smart contract on Stellar blockchain.

use soroban_sdk::{contracterror, symbol_short, Symbol};

pub const COMMON_ERROR_MAX: u32 = 99;
pub const MEDICAL_RECORDS_ERROR_BASE: u32 = 1000;
pub const RBAC_ERROR_BASE: u32 = 2000;
pub type CommonResult<T> = Result<T, CommonError>;

#[contracterror(export = false)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum CommonError {
    Unknown = 0,
    Unauthorized = 1,
    NotInitialized = 2,
    AlreadyInitialized = 3,
    ContractPaused = 4,
    DeadlineExceeded = 5,
    RateLimitExceeded = 6,
    InsufficientFunds = 7,
    InvalidInput = 8,
    InvalidState = 9,
    NotFound = 10,
    AccessDenied = 11,
    Timeout = 12,
    InvalidArgument = 13,
    ExternalContractNotSet = 14,
    InvalidData = 15,
    InvalidPayload = 16,
    DuplicateSubmission = 17,
    UnauthorizedCaller = 18,
}

pub fn is_common_error_code(code: u32) -> bool {
    code <= COMMON_ERROR_MAX
}

pub fn get_suggestion(error: CommonError) -> Symbol {
    match error {
        CommonError::ContractPaused | CommonError::RateLimitExceeded => symbol_short!("RE_TRY_L"),
        CommonError::Unauthorized | CommonError::UnauthorizedCaller => symbol_short!("CHK_AUTH"),
        CommonError::NotInitialized => symbol_short!("INIT_CTR"),
        CommonError::AlreadyInitialized => symbol_short!("ALREADY"),
        CommonError::InvalidInput | CommonError::InvalidArgument | CommonError::InvalidData => {
            symbol_short!("CHK_DATA")
        },
        CommonError::NotFound => symbol_short!("CHK_ID"),
        CommonError::InsufficientFunds => symbol_short!("ADD_FUND"),
        CommonError::Timeout => symbol_short!("RE_TRY_L"),
        _ => symbol_short!("CONTACT"),
    }
}
