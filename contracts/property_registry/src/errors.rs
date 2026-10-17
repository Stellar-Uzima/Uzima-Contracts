use soroban_sdk::contracterror;

/// Typed errors for every rejection path.
///
/// # Stability
///
/// The `u32` discriminants are part of the contract interface: clients match on
/// them, so existing variants are never renumbered and new ones are only ever
/// appended at the end of the enum.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    /// The contract has not been initialized yet.
    NotInitialized = 1,
    /// `initialize` was called on a contract that is already initialized.
    AlreadyInitialized = 2,
    /// The caller's authorization or role check failed, or the contract is
    /// paused.
    ///
    /// This one variant covers both "not permitted" and "temporarily halted"
    /// because both mean the same thing to a caller: retry later or ask the
    /// admin. Callers that need to tell them apart should consult `paused()`.
    Unauthorized = 3,
    /// No property is registered under the supplied `property_id`.
    PropertyNotFound = 4,
    /// A property is already registered under the supplied `property_id`.
    PropertyAlreadyExists = 5,
    /// An argument was structurally unacceptable — currently only the
    /// self-transfer and owner-as-holder cases, which are both no-ops written
    /// twice.
    InvalidArgument = 6,
    /// A `u128` addition or multiplication would have wrapped.
    ArithmeticOverflow = 7,
    /// A `u128` subtraction would have wrapped past zero.
    ArithmeticUnderflow = 8,
    /// The owner of record does not hold the share being moved.
    OwnershipExceedsShare = 9,
    /// The resulting shares for a property would exceed 100%.
    ShareTotalExceeded = 10,
    /// A guarded entry point was re-entered while already active.
    ///
    /// Only reachable by an address whose `SorobanAuthorization` contract is
    /// under attacker control; see `src/reentrancy_guard.rs` for why the guard
    /// exists at all.
    ReentrantCall = 11,
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
            Error::ReentrantCall => write!(f, "reentrant call"),
        }
    }
}