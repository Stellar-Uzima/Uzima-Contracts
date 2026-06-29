use soroban_sdk::{contracterror, symbol_short, Symbol};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    Unauthorized = 100,
    InvalidInput = 200,
    NotInitialized = 300,
    AlreadyInitialized = 301,
    VersionNotFound = 430,
    InvalidProof = 600,
    VerificationFailed = 601,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::Unauthorized => write!(f, "unauthorized"),
            Error::InvalidInput => write!(f, "invalid input"),
            Error::NotInitialized => write!(f, "not initialized"),
            Error::AlreadyInitialized => write!(f, "already initialized"),
            Error::VersionNotFound => write!(f, "version not found"),
            Error::InvalidProof => write!(f, "invalid proof"),
            Error::VerificationFailed => write!(f, "verification failed"),
        }
    }
}

pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::Unauthorized => symbol_short!("CHK_AUTH"),
        Error::NotInitialized => symbol_short!("INIT_CTR"),
        Error::AlreadyInitialized => symbol_short!("ALREADY"),
        Error::InvalidInput => symbol_short!("CHK_LEN"),
        Error::VersionNotFound => symbol_short!("CHK_ID"),
        Error::InvalidProof | Error::VerificationFailed => symbol_short!("CONTACT"),
    }
}
