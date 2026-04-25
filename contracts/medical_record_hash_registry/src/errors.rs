#[cfg(not(feature = "library"))]
pub use soroban_sdk::contracterror;

#[cfg(feature = "library")]
pub use soroban_sdk::contracterror;

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    NotAuthorized = 3,
    DuplicateRecord = 4,
    RecordNotFound = 5,
    InvalidPatientId = 6,
    InvalidRecordHash = 7,
    InsufficientFunds = 10,
    DeadlineExceeded = 11,
    InvalidSignature = 12,
    UnauthorizedCaller = 13,
    ContractPaused = 14,
    StorageFull = 15,
    CrossChainTimeout = 16,
}
