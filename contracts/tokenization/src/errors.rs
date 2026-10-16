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
    /// A `u128` subtraction would have wrapped past zero.
    ArithmeticUnderflow = 8,
    /// The holder does not hold enough units for the requested debit.
    InsufficientBalance = 9,
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
        }
    }
}