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
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::NotInitialized => write!(f, "not initialized"),
            Error::AlreadyInitialized => write!(f, "already initialized"),
            Error::NotAuthorized => write!(f, "not authorized"),
            Error::CheckNotFound => write!(f, "check not found"),
            Error::CheckAlreadyExists => write!(f, "check already exists"),
            Error::CheckNotActive => write!(f, "check not active"),
            Error::InvalidSeverity => write!(f, "invalid severity"),
            Error::InvalidResourceLimit => write!(f, "invalid resource limit"),
            Error::ResourceLimitExceeded => write!(f, "resource limit exceeded"),
            Error::ViolationNotFound => write!(f, "violation not found"),
        }
    }
}
