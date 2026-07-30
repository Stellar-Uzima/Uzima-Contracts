use soroban_sdk::contracterror;

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
    InvalidTTL = 480,
    InvalidNotificationWindow = 481,
    ErasureRequestExists = 500,
    ErasureRequestNotFound = 501,
    ErasureRequestNotPending = 502,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::Unauthorized => write!(f, "unauthorized"),
            Error::InvalidPatient => write!(f, "invalid patient"),
            Error::InvalidProvider => write!(f, "invalid provider"),
            Error::NotInitialized => write!(f, "not initialized"),
            Error::AlreadyInitialized => write!(f, "already initialized"),
            Error::ContractPaused => write!(f, "contract paused"),
            Error::ConsentNotFound => write!(f, "consent not found"),
            Error::ConsentAlreadyExists => write!(f, "consent already exists"),
            Error::InvalidExpiry => write!(f, "invalid expiry"),
            Error::InvalidTTL => write!(f, "invalid ttl"),
            Error::InvalidNotificationWindow => write!(f, "invalid notification window"),
            Error::ErasureRequestExists => write!(f, "erasure request exists"),
            Error::ErasureRequestNotFound => write!(f, "erasure request not found"),
            Error::ErasureRequestNotPending => write!(f, "erasure request not pending"),
        }
    }
}


#[cfg(test)]
pub fn get_suggestion(error: Error) -> soroban_sdk::Symbol {
    match error {
        Error::Unauthorized => soroban_sdk::symbol_short!("CHK_AUTH"),
        Error::NotInitialized => soroban_sdk::symbol_short!("INIT_CTR"),
        Error::AlreadyInitialized => soroban_sdk::symbol_short!("ALREADY"),
        Error::ContractPaused => soroban_sdk::symbol_short!("PAUSED"),
        Error::InvalidPatient | Error::InvalidProvider => soroban_sdk::symbol_short!("CHK_ID"),
        Error::ConsentNotFound => soroban_sdk::symbol_short!("CHK_ID"),
        Error::ConsentAlreadyExists => soroban_sdk::symbol_short!("ALREADY"),
        Error::InvalidExpiry => soroban_sdk::symbol_short!("BAD_EXP"),
        Error::InvalidTTL => soroban_sdk::symbol_short!("BAD_TTL"),
        Error::InvalidNotificationWindow => soroban_sdk::symbol_short!("BAD_WIN"),
        Error::ErasureRequestExists => soroban_sdk::symbol_short!("ER_EXIST"),
        Error::ErasureRequestNotFound => soroban_sdk::symbol_short!("ER_NFOUND"),
        Error::ErasureRequestNotPending => soroban_sdk::symbol_short!("ER_STATE"),
    }
}