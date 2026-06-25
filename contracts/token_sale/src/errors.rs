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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Error::AlreadyInitialized => "Already Initialized",
            Error::InvalidArgument => "Invalid Argument",
            Error::Overflow => "Overflow",
            Error::PhaseNotFound => "Phase Not Found",
            Error::PhaseClosed => "Phase Closed",
            Error::CapExceeded => "Cap Exceeded",
            Error::NotFinalized => "Not Finalized",
            Error::AlreadyClaimed => "Already Claimed",
            Error::RefundsNotEnabled => "Refunds Not Enabled",
            Error::Paused => "Paused",
        };
        f.write_str(message)
    }
}
