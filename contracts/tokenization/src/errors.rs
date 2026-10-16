use soroban_sdk::contracterror;

/// Typed errors for every rejection path in the contract.
///
/// The discriminant is part of the contract interface: these values are what
/// clients match on, so existing variants are never renumbered and new ones
/// are only ever appended.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    AssetNotFound = 4,
    AssetAlreadyExists = 5,

    // ---- input validation -------------------------------------------------
    /// `asset_id` was empty, longer than [`crate::validation::MAX_ASSET_ID_LEN`],
    /// or contained a character outside `[A-Za-z0-9_-]`.
    InvalidAssetId = 6,
    /// `decimals` exceeded [`crate::validation::MAX_DECIMALS`] or
    /// [`crate::validation::MIN_DECIMALS`].
    DecimalsOutOfRange = 7,
    /// `metadata_uri` was empty or longer than
    /// [`crate::validation::MAX_METADATA_URI_LEN`].
    InvalidMetadataUri = 8,
    /// A mint, burn or transfer was for `amount == 0`.
    ZeroAmount = 9,
    /// `from` and `to` were the same address on a transfer.
    TransferToSelf = 10,
    /// The holder does not hold enough units for the requested debit.
    InsufficientBalance = 11,
    /// The mint would push the asset past the supply registered for it.
    SupplyCapExceeded = 12,

    // ---- arithmetic -------------------------------------------------------
    /// A `u128` addition or multiplication would have wrapped.
    ArithmeticOverflow = 13,
    /// A `u128` subtraction would have wrapped past zero.
    ArithmeticUnderflow = 14,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::NotInitialized => write!(f, "not initialized"),
            Error::AlreadyInitialized => write!(f, "already initialized"),
            Error::Unauthorized => write!(f, "unauthorized"),
            Error::AssetNotFound => write!(f, "asset not found"),
            Error::AssetAlreadyExists => write!(f, "asset already exists"),
            Error::InvalidAssetId => write!(f, "invalid asset id"),
            Error::DecimalsOutOfRange => write!(f, "decimals out of range"),
            Error::InvalidMetadataUri => write!(f, "invalid metadata uri"),
            Error::ZeroAmount => write!(f, "amount must be non-zero"),
            Error::TransferToSelf => write!(f, "transfer to self"),
            Error::InsufficientBalance => write!(f, "insufficient balance"),
            Error::SupplyCapExceeded => write!(f, "supply cap exceeded"),
            Error::ArithmeticOverflow => write!(f, "arithmetic overflow"),
            Error::ArithmeticUnderflow => write!(f, "arithmetic underflow"),
        }
    }
}