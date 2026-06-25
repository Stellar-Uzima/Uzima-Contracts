use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    NotAuthorized = 3,
    ModuleNotFound = 4,
    ModuleAlreadyExists = 5,
    ReviewRouteNotFound = 6,
    InvalidOwnerCount = 7,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Error::NotInitialized => "Not Initialized",
            Error::AlreadyInitialized => "Already Initialized",
            Error::NotAuthorized => "Not Authorized",
            Error::ModuleNotFound => "Module Not Found",
            Error::ModuleAlreadyExists => "Module Already Exists",
            Error::ReviewRouteNotFound => "Review Route Not Found",
            Error::InvalidOwnerCount => "Invalid Owner Count",
        };
        f.write_str(message)
    }
}
