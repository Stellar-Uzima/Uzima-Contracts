use soroban_sdk::{contracterror, symbol_short, Symbol};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    Unauthorized = 100,
    InvalidInput = 200,
    NotInitialized = 300,
    AlreadyInitialized = 301,
    RuleNotFound = 450,
    ZkProofRequired = 500,
    CategoryLimitExceeded = 510,
    OverallLimitExceeded = 520,
    ZkVerificationFailed = 600,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::Unauthorized => write!(f, "unauthorized"),
            Error::InvalidInput => write!(f, "invalid input"),
            Error::NotInitialized => write!(f, "not initialized"),
            Error::AlreadyInitialized => write!(f, "already initialized"),
            Error::RuleNotFound => write!(f, "rule not found"),
            Error::ZkProofRequired => write!(f, "ZK proof required"),
            Error::CategoryLimitExceeded => write!(f, "category limit exceeded"),
            Error::OverallLimitExceeded => write!(f, "overall limit exceeded"),
            Error::ZkVerificationFailed => write!(f, "ZK verification failed"),
        }
    }
}

pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::Unauthorized => symbol_short!("CHK_AUTH"),
        Error::NotInitialized => symbol_short!("INIT_CTR"),
        Error::AlreadyInitialized => symbol_short!("ALREADY"),
        Error::InvalidInput => symbol_short!("CHK_DATA"),
        Error::RuleNotFound => symbol_short!("SET_RULE"),
        Error::ZkProofRequired => symbol_short!("ADD_PROOF"),
        Error::CategoryLimitExceeded | Error::OverallLimitExceeded => {
            symbol_short!("REDUCE")
        }
        Error::ZkVerificationFailed => symbol_short!("CONTACT"),
    }
}
