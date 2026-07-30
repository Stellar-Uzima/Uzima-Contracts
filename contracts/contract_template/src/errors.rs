use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum Error {
    /// Contract has not been initialized yet.
    NotInitialized = 1,
    /// Contract has already been initialized.
    AlreadyInitialized = 2,
    /// Caller is not authorized to perform this action.
    Unauthorized = 3,
    /// A string or bytes input exceeded the maximum allowed length.
    InputTooLong = 4,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::NotInitialized => write!(f, "not initialized"),
            Error::AlreadyInitialized => write!(f, "already initialized"),
            Error::Unauthorized => write!(f, "unauthorized"),
            Error::InputTooLong => write!(f, "input too long"),
        }
    }
}
