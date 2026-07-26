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

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::AlreadyInitialized => write!(f, "already initialized"),
            Error::InvalidArgument => write!(f, "invalid argument"),
            Error::Overflow => write!(f, "overflow"),
            Error::PhaseNotFound => write!(f, "phase not found"),
            Error::PhaseClosed => write!(f, "phase closed"),
            Error::CapExceeded => write!(f, "cap exceeded"),
            Error::NotFinalized => write!(f, "not finalized"),
            Error::AlreadyClaimed => write!(f, "already claimed"),
            Error::RefundsNotEnabled => write!(f, "refunds not enabled"),
            Error::Paused => write!(f, "paused"),
        }
    }
}
