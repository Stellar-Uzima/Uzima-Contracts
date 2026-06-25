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
    AppointmentNoShow = 413,
    InsufficientFunds = 500,
    TokenTransferFailed = 501,
    DoubleWithdrawal = 505,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Error::Unauthorized => "Unauthorized",
            Error::OnlyPatientCanRefund => "Only Patient Can Refund",
            Error::OnlyProviderCanConfirm => "Only Provider Can Confirm",
            Error::InvalidAmount => "Invalid Amount",
            Error::InvalidPatient => "Invalid Patient",
            Error::InvalidProvider => "Invalid Provider",
            Error::NotInitialized => "Not Initialized",
            Error::AlreadyInitialized => "Already Initialized",
            Error::InvalidState => "Invalid State",
            Error::AppointmentNotFound => "Appointment Not Found",
            Error::AppointmentAlreadyConfirmed => "Appointment Already Confirmed",
            Error::AppointmentAlreadyRefunded => "Appointment Already Refunded",
            Error::AppointmentNoShow => "Appointment No Show",
            Error::InsufficientFunds => "Insufficient Funds",
            Error::TokenTransferFailed => "Token Transfer Failed",
            Error::DoubleWithdrawal => "Double Withdrawal",
        };
        f.write_str(message)
    }
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
