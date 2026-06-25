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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Error::Unauthorized => "Unauthorized",
            Error::InvalidInput => "Invalid Input",
            Error::NotInitialized => "Not Initialized",
            Error::AlreadyInitialized => "Already Initialized",
            Error::VersionNotFound => "Version Not Found",
            Error::InvalidProof => "Invalid Proof",
            Error::VerificationFailed => "Verification Failed",
        };
        f.write_str(message)
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
