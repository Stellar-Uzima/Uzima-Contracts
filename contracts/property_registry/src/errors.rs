use soroban_sdk::contracterror;

/// Typed errors for every rejection path in the contract.
///
/// The discriminant is part of the contract interface: these values are what
/// clients match on, so existing variants are never renumbered and new ones are
/// only ever appended.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    PropertyNotFound = 4,
    PropertyAlreadyExists = 5,

    // ---- input validation -------------------------------------------------
    /// `property_id` was empty, longer than
    /// [`crate::validation::MAX_PROPERTY_ID_LEN`], or contained a character
    /// outside `[A-Za-z0-9_-]`.
    InvalidPropertyId = 6,
    /// `valuation_currency` was empty or longer than
    /// [`crate::validation::MAX_CURRENCY_LEN`].
    InvalidCurrency = 7,
    /// `metadata_uri` was empty or longer than
    /// [`crate::validation::MAX_METADATA_URI_LEN`].
    InvalidMetadataUri = 8,
    /// A recorded valuation or share of zero was supplied where a positive
    /// value is required.
    ZeroValue = 9,
    /// `record_ownership` was asked to credit the owner of record, which would
    /// be a no-op written twice.
    HolderIsOwner = 10,
    /// `transfer_property` was asked to transfer the property to its current
    /// owner of record.
    TransferToSelf = 11,

    // ---- share accounting -------------------------------------------------
    /// The resulting shares for a property would exceed 100%.
    ShareTotalExceeded = 12,
    /// The requested share is larger than the recorded share.
    OwnershipExceedsShare = 13,
    /// A share that could not be represented in basis points was supplied.
    ShareTooLarge = 14,

    // ---- arithmetic -------------------------------------------------------
    /// A `u128` addition or multiplication would have wrapped.
    ArithmeticOverflow = 15,
    /// A `u128` subtraction would have wrapped past zero.
    ArithmeticUnderflow = 16,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::NotInitialized => write!(f, "not initialized"),
            Error::AlreadyInitialized => write!(f, "already initialized"),
            Error::Unauthorized => write!(f, "unauthorized"),
            Error::PropertyNotFound => write!(f, "property not found"),
            Error::PropertyAlreadyExists => write!(f, "property already exists"),
            Error::InvalidPropertyId => write!(f, "invalid property id"),
            Error::InvalidCurrency => write!(f, "invalid valuation currency"),
            Error::InvalidMetadataUri => write!(f, "invalid metadata uri"),
            Error::ZeroValue => write!(f, "value must be non-zero"),
            Error::HolderIsOwner => write!(f, "holder is the owner of record"),
            Error::TransferToSelf => write!(f, "transfer to self"),
            Error::ShareTotalExceeded => write!(f, "share total exceeds 100 percent"),
            Error::OwnershipExceedsShare => write!(f, "ownership exceeds recorded share"),
            Error::ShareTooLarge => write!(f, "share too large"),
            Error::ArithmeticOverflow => write!(f, "arithmetic overflow"),
            Error::ArithmeticUnderflow => write!(f, "arithmetic underflow"),
        }
    }
}