use soroban_sdk::{contracterror, symbol_short, Symbol};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // --- Lifecycle & State (300–399) ---
    NotInitialized = 300,
    AlreadyInitialized = 301,
    InvalidState = 304,
    VotingClosed = 370,
    AlreadyVoted = 371,
    NotQueued = 372,
    ProposalDisputed = 373,

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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Error::NotInitialized => "Not Initialized",
            Error::AlreadyInitialized => "Already Initialized",
            Error::InvalidState => "Invalid State",
            Error::VotingClosed => "Voting Closed",
            Error::AlreadyVoted => "Already Voted",
            Error::NotQueued => "Not Queued",
            Error::ProposalDisputed => "Proposal Disputed",
            Error::ProposalNotFound => "Proposal Not Found",
            Error::ProposalNotSuccessful => "Proposal Not Successful",
            Error::AlreadyExecuted => "Already Executed",
            Error::ProposalThresholdNotMet => "Proposal Threshold Not Met",
            Error::NoVotingPower => "No Voting Power",
            Error::Overflow => "Overflow",
            Error::InvalidVoteType => "Invalid Vote Type",
        };
        f.write_str(message)
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
