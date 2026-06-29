use soroban_sdk::{contracterror};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    Unauthorized = 100,
    OnlyPatientCanRefund = 110,
    OnlyProviderCanConfirm = 111,
    InvalidAmount = 205,
    InvalidPatient = 210,
    InvalidProvider = 211,
    NotInitialized = 300,
    AlreadyInitialized = 301,
    InvalidState = 304,
    AppointmentNotFound = 410,
    AppointmentAlreadyConfirmed = 411,
    AppointmentAlreadyRefunded = 412,
    AppointmentNoShow = 413,
    InsufficientFunds = 500,
    TokenTransferFailed = 501,
    DoubleWithdrawal = 505,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::Unauthorized => write!(f, "unauthorized"),
            Error::OnlyPatientCanRefund => write!(f, "only patient can refund"),
            Error::OnlyProviderCanConfirm => write!(f, "only provider can confirm"),
            Error::InvalidAmount => write!(f, "invalid amount"),
            Error::InvalidPatient => write!(f, "invalid patient"),
            Error::InvalidProvider => write!(f, "invalid provider"),
            Error::NotInitialized => write!(f, "not initialized"),
            Error::AlreadyInitialized => write!(f, "already initialized"),
            Error::InvalidState => write!(f, "invalid state"),
            Error::AppointmentNotFound => write!(f, "appointment not found"),
            Error::AppointmentAlreadyConfirmed => write!(f, "appointment already confirmed"),
            Error::AppointmentAlreadyRefunded => write!(f, "appointment already refunded"),
            Error::AppointmentNoShow => write!(f, "appointment no show"),
            Error::InsufficientFunds => write!(f, "insufficient funds"),
            Error::TokenTransferFailed => write!(f, "token transfer failed"),
            Error::DoubleWithdrawal => write!(f, "double withdrawal"),
        }
    }
}

#[doc(hidden)] /// Reserved for ABI consistency; not currently called from contract code.
#[cfg(test)]
pub fn get_suggestion(error: Error) -> soroban_sdk::Symbol {
    match error {
        Error::Unauthorized | Error::OnlyPatientCanRefund | Error::OnlyProviderCanConfirm => {
            soroban_sdk::symbol_short!("CHK_AUTH")
        },
        Error::NotInitialized => soroban_sdk::symbol_short!("INIT_CTR"),
        Error::AlreadyInitialized => soroban_sdk::symbol_short!("ALREADY"),
        Error::InvalidAmount | Error::InvalidPatient | Error::InvalidProvider => {
            soroban_sdk::symbol_short!("CHK_LEN")
        },
        Error::AppointmentNotFound => soroban_sdk::symbol_short!("CHK_ID"),
        Error::InsufficientFunds => soroban_sdk::symbol_short!("ADD_FUND"),
        _ => soroban_sdk::symbol_short!("CONTACT"),
    }
}
