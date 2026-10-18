use soroban_sdk::contracterror;

/// Custom errors returned by `RentalYield` instead of panicking.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 3101,
    AlreadyInitialized = 3102,
    Unauthorized = 3103,
    InvalidAmount = 3104,
    Overflow = 3105,
    NothingToClaim = 3106,
    Reentrancy = 3107,
}
