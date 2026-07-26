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
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::NotInitialized => write!(f, "not initialized"),
            Error::AlreadyInitialized => write!(f, "already initialized"),
            Error::NotAuthorized => write!(f, "not authorized"),
            Error::ModuleNotFound => write!(f, "module not found"),
            Error::ModuleAlreadyExists => write!(f, "module already exists"),
            Error::ReviewRouteNotFound => write!(f, "review route not found"),
            Error::InvalidOwnerCount => write!(f, "invalid owner count"),
        }
    }
}
