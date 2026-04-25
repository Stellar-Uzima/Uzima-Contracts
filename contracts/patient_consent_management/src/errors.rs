use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    NotAuthorized = 3,
    InvalidPatient = 4,
    InvalidProvider = 5,
    ConsentNotFound = 6,
    ConsentAlreadyExists = 7,
    UnauthorizedAccess = 8,
}
