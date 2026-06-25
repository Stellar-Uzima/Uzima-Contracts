use common_error::CommonError;
use soroban_sdk::{contracterror, symbol_short, Symbol};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    Unauthorized = CommonError::Unauthorized as u32,
    NotInitialized = CommonError::NotInitialized as u32,
    AlreadyInitialized = CommonError::AlreadyInitialized as u32,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Error::Unauthorized => "Unauthorized",
            Error::NotInitialized => "Not Initialized",
            Error::AlreadyInitialized => "Already Initialized",
        };
        f.write_str(message)
    }
}


pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::Unauthorized => symbol_short!("CHK_AUTH"),
        Error::NotInitialized => symbol_short!("INIT_CTR"),
        Error::AlreadyInitialized => symbol_short!("ALREADY"),
    }
}
