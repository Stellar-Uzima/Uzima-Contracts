use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 3001,
    AlreadyInitialized = 3002,
    Unauthorized = 3003,
    InvalidAmount = 3004,
    AssetNotFound = 3005,
    InsufficientShares = 3006,
    SelfTransfer = 3007,
    NoListing = 3008,
    ExceedsListing = 3009,
    Overflow = 3010,
    NothingToClaim = 3011,
    PayoutTooSmall = 3012,
}
