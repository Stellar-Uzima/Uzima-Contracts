use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    NotAuthorized = 3,
    CheckNotFound = 4,
    CheckAlreadyExists = 5,
    CheckNotActive = 6,
    InvalidSeverity = 7,
    InvalidResourceLimit = 8,
    ResourceLimitExceeded = 9,
    ViolationNotFound = 10,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Error::NotInitialized => "Not Initialized",
            Error::AlreadyInitialized => "Already Initialized",
            Error::NotAuthorized => "Not Authorized",
            Error::CheckNotFound => "Check Not Found",
            Error::CheckAlreadyExists => "Check Already Exists",
            Error::CheckNotActive => "Check Not Active",
            Error::InvalidSeverity => "Invalid Severity",
            Error::InvalidResourceLimit => "Invalid Resource Limit",
            Error::ResourceLimitExceeded => "Resource Limit Exceeded",
            Error::ViolationNotFound => "Violation Not Found",
        };
        f.write_str(message)
    }
}
