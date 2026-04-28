use soroban_sdk::{contracterror, symbol_short, Symbol};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    Unauthorized = 100,
    NotInitialized = 300,
    AlreadyInitialized = 301,
    InvalidThreshold = 230,
    InvalidDuration = 231,
    RecordNotFound = 403,
}

pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::Unauthorized => symbol_short!("CHK_AUTH"),
        Error::NotInitialized => symbol_short!("INIT_CTR"),
        Error::AlreadyInitialized => symbol_short!("ALREADY"),
        Error::InvalidThreshold | Error::InvalidDuration => symbol_short!("CHK_LEN"),
        Error::RecordNotFound => symbol_short!("CHK_ID"),
    }
}
