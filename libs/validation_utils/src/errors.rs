//! Error types for validation

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum ValidationError {
    // String errors
    EmptyString = 1,
    StringTooShort = 2,
    StringTooLong = 3,
    InvalidCharset = 4,

    // Address errors
    InvalidAddress = 5,
    ZeroAddress = 6,
    AddressesMustDiffer = 7,

    // Numeric errors
    NegativeValueNotAllowed = 8,
    ValueTooSmall = 9,
    ValueTooLarge = 10,
    ZeroValueNotAllowed = 11,

    // Collection errors
    CollectionEmpty = 12,
    CollectionTooLarge = 13,
    DuplicateEntry = 14,

    // General errors
    InvalidInput = 15,
    ValidationFailed = 16,

    // Timestamp errors
    InvalidTimestamp = 17,
    TimestampInFuture = 18,
}

impl ValidationError {
    pub fn as_u32(self) -> u32 {
        self as u32
    }
}
