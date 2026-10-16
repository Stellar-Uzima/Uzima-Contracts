use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    PropertyNotFound = 4,
    PropertyAlreadyExists = 5,
    InvalidArgument = 6,
    ArithmeticOverflow = 7,
    /// A `u128` subtraction would have wrapped past zero.
    ArithmeticUnderflow = 8,
    /// The requested ownership share is larger than the recorded share.
    OwnershipExceedsShare = 9,
    /// The resulting shares for a property would exceed 100%.
    ShareTotalExceeded = 10,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::NotInitialized => write!(f, "not initialized"),
            Error::AlreadyInitialized => write!(f, "already initialized"),
            Error::Unauthorized => write!(f, "unauthorized"),
            Error::PropertyNotFound => write!(f, "property not found"),
            Error::PropertyAlreadyExists => write!(f, "property already exists"),
            Error::InvalidArgument => write!(f, "invalid argument"),
            Error::ArithmeticOverflow => write!(f, "arithmetic overflow"),
            Error::ArithmeticUnderflow => write!(f, "arithmetic underflow"),
            Error::OwnershipExceedsShare => write!(f, "ownership exceeds recorded share"),
            Error::ShareTotalExceeded => write!(f, "share total exceeds 100 percent"),
        }
    }
}