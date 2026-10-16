use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    AssetNotFound = 4,
    AssetAlreadyExists = 5,
    InvalidArgument = 6,
    ArithmeticOverflow = 7,
    ArithmeticUnderflow = 8,
    InsufficientBalance = 9,
    /// Revoking `DEFAULT_ADMIN_ROLE` from its last holder would leave the
    /// contract permanently unadministrable.
    CannotRevokeLastAdmin = 10,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::NotInitialized => write!(f, "not initialized"),
            Error::AlreadyInitialized => write!(f, "already initialized"),
            Error::Unauthorized => write!(f, "unauthorized"),
            Error::AssetNotFound => write!(f, "asset not found"),
            Error::AssetAlreadyExists => write!(f, "asset already exists"),
            Error::InvalidArgument => write!(f, "invalid argument"),
            Error::ArithmeticOverflow => write!(f, "arithmetic overflow"),
            Error::ArithmeticUnderflow => write!(f, "arithmetic underflow"),
            Error::InsufficientBalance => write!(f, "insufficient balance"),
            Error::CannotRevokeLastAdmin => write!(f, "cannot revoke the last admin"),
        }
    }
}