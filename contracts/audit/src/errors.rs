use soroban_sdk::{contracterror, symbol_short, Symbol};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    Unauthorized = 100,
    NotAuthorized = 101,
    NotInitialized = 300,
    AlreadyInitialized = 301,
    RecordNotFound = 403,
    RetentionPolicyNotFound = 404,
    RetentionWindowTooShort = 405,
    ChainBroken = 406,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::Unauthorized => write!(f, "unauthorized"),
            Error::NotAuthorized => write!(f, "not authorized"),
            Error::NotInitialized => write!(f, "not initialized"),
            Error::AlreadyInitialized => write!(f, "already initialized"),
            Error::RecordNotFound => write!(f, "record not found"),
            Error::RetentionPolicyNotFound => write!(f, "retention policy not found"),
            Error::RetentionWindowTooShort => write!(f, "retention window too short"),
            Error::ChainBroken => write!(f, "chain broken"),
        }
    }
}

pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::Unauthorized | Error::NotAuthorized => symbol_short!("CHK_AUTH"),
        Error::NotInitialized => symbol_short!("INIT_CTR"),
        Error::AlreadyInitialized => symbol_short!("ALREADY"),
        Error::RecordNotFound => symbol_short!("CHK_ID"),
        Error::RetentionPolicyNotFound => symbol_short!("CHK_POL"),
        Error::RetentionWindowTooShort => symbol_short!("TSHORT"),
        Error::ChainBroken => symbol_short!("CHAIN_BRK"),
    }
}
