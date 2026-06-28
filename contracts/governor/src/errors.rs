use soroban_sdk::{contracterror, symbol_short, Symbol};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // --- Authorization (100–199) ---
    Unauthorized = 100,

    // --- Lifecycle & State (300–399) ---
    NotInitialized = 300,
    AlreadyInitialized = 301,
    InvalidState = 304,
    VotingClosed = 370,
    AlreadyVoted = 371,
    NotQueued = 372,
    ProposalDisputed = 373,
    CleanupEmpty = 374,

    // --- Entity Existence (400–499) ---
    ProposalNotFound = 450,
    ProposalNotSuccessful = 451,
    AlreadyExecuted = 452,

    // --- Financial & Resource (500–599) ---
    ProposalThresholdNotMet = 530,
    NoVotingPower = 531,
    Overflow = 580,

    // --- Input Validation (200–299) ---
    InvalidVoteType = 280,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::Unauthorized => write!(f, "unauthorized"),
            Error::NotInitialized => write!(f, "not initialized"),
            Error::AlreadyInitialized => write!(f, "already initialized"),
            Error::InvalidState => write!(f, "invalid state"),
            Error::VotingClosed => write!(f, "voting closed"),
            Error::AlreadyVoted => write!(f, "already voted"),
            Error::NotQueued => write!(f, "not queued"),
            Error::ProposalDisputed => write!(f, "proposal disputed"),
            Error::ProposalNotFound => write!(f, "proposal not found"),
            Error::ProposalNotSuccessful => write!(f, "proposal not successful"),
            Error::AlreadyExecuted => write!(f, "already executed"),
            Error::ProposalThresholdNotMet => write!(f, "proposal threshold not met"),
            Error::NoVotingPower => write!(f, "no voting power"),
            Error::Overflow => write!(f, "overflow"),
            Error::InvalidVoteType => write!(f, "invalid vote type"),
        }
    }
}

pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::NotInitialized => symbol_short!("INIT_CTR"),
        Error::AlreadyInitialized | Error::AlreadyVoted | Error::AlreadyExecuted => {
            symbol_short!("ALREADY")
        },
        Error::ProposalNotFound | Error::ProposalNotSuccessful => symbol_short!("CHK_ID"),
        Error::VotingClosed | Error::NotQueued => symbol_short!("RE_TRY_L"),
        _ => symbol_short!("CONTACT"),
    }
}
