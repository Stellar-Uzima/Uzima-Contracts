use soroban_sdk::{contracterror, symbol_short, Symbol};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    Unauthorized = 100,
    InvalidPatient = 210,
    InvalidProvider = 211,
    NotInitialized = 300,
    AlreadyInitialized = 301,
    ContractPaused = 302,
    ConsentNotFound = 406,
    ConsentAlreadyExists = 460,
    InvalidExpiry = 470,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Error::Unauthorized => "Unauthorized",
            Error::InvalidPatient => "Invalid Patient",
            Error::InvalidProvider => "Invalid Provider",
            Error::NotInitialized => "Not Initialized",
            Error::AlreadyInitialized => "Already Initialized",
            Error::ContractPaused => "Contract Paused",
            Error::ConsentNotFound => "Consent Not Found",
            Error::ConsentAlreadyExists => "Consent Already Exists",
            Error::InvalidExpiry => "Invalid Expiry",
        };
        f.write_str(message)
    }
}

#[allow(dead_code)]
pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::Unauthorized => symbol_short!("CHK_AUTH"),
        Error::NotInitialized => symbol_short!("INIT_CTR"),
        Error::AlreadyInitialized => symbol_short!("ALREADY"),
        Error::ContractPaused => symbol_short!("PAUSED"),
        Error::InvalidPatient | Error::InvalidProvider => symbol_short!("CHK_ID"),
        Error::ConsentNotFound => symbol_short!("CHK_ID"),
        Error::ConsentAlreadyExists => symbol_short!("ALREADY"),
        Error::InvalidExpiry => symbol_short!("BAD_EXP"),
    }
}
