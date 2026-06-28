use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    NotAuthorized = 3,
    ModuleNotFound = 4,
    ModuleAlreadyExists = 5,
    ReviewRouteNotFound = 6,
    InvalidOwnerCount = 7,
}
