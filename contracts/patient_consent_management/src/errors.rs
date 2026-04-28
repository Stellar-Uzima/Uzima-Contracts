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
    ConsentNotFound = 406,
    ConsentAlreadyExists = 460,
}

#[allow(dead_code)]
pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::Unauthorized => symbol_short!("CHK_AUTH"),
        Error::NotInitialized => symbol_short!("INIT_CTR"),
        Error::AlreadyInitialized => symbol_short!("ALREADY"),
        Error::InvalidPatient | Error::InvalidProvider => symbol_short!("CHK_ID"),
        Error::ConsentNotFound => symbol_short!("CHK_ID"),
        Error::ConsentAlreadyExists => symbol_short!("ALREADY"),
    }
}
