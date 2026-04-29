use soroban_sdk::{contracterror, symbol_short, Symbol};

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
    InsufficientFunds = 500,
    TokenTransferFailed = 501,
    DoubleWithdrawal = 505,
}

pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::Unauthorized | Error::OnlyPatientCanRefund | Error::OnlyProviderCanConfirm => {
            symbol_short!("CHK_AUTH")
        },
        Error::NotInitialized => symbol_short!("INIT_CTR"),
        Error::AlreadyInitialized => symbol_short!("ALREADY"),
        Error::InvalidAmount | Error::InvalidPatient | Error::InvalidProvider => {
            symbol_short!("CHK_LEN")
        },
        Error::AppointmentNotFound => symbol_short!("CHK_ID"),
        Error::InsufficientFunds => symbol_short!("ADD_FUND"),
        _ => symbol_short!("CONTACT"),
    }
}
