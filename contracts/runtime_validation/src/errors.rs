use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    NotAuthorized = 3,
    CheckNotFound = 4,
    CheckAlreadyExists = 5,
    CheckNotActive = 6,
    InvalidSeverity = 7,
    InvalidResourceLimit = 8,
    ResourceLimitExceeded = 9,
    ViolationNotFound = 10,
}
