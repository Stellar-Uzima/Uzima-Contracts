use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    NotAuthorized = 3,
    ContractNotFound = 4,
    ContractAlreadyDeprecated = 5,
    InvalidTimeline = 6,
    InvalidPhaseTransition = 7,
    TimelineNotFound = 8,
    GuideNotFound = 9,
    ChecklistNotFound = 10,
    InvalidChecklistIndex = 11,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Error::NotInitialized => "Not Initialized",
            Error::AlreadyInitialized => "Already Initialized",
            Error::NotAuthorized => "Not Authorized",
            Error::ContractNotFound => "Contract Not Found",
            Error::ContractAlreadyDeprecated => "Contract Already Deprecated",
            Error::InvalidTimeline => "Invalid Timeline",
            Error::InvalidPhaseTransition => "Invalid Phase Transition",
            Error::TimelineNotFound => "Timeline Not Found",
            Error::GuideNotFound => "Guide Not Found",
            Error::ChecklistNotFound => "Checklist Not Found",
            Error::InvalidChecklistIndex => "Invalid Checklist Index",
        };
        f.write_str(message)
    }
}
