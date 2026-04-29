use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    InvalidArgument = 2,
    Overflow = 3,
    PhaseNotFound = 4,
    PhaseClosed = 5,
    CapExceeded = 6,
    NotFinalized = 7,
    AlreadyClaimed = 8,
    RefundsNotEnabled = 9,
    Paused = 10,
}
