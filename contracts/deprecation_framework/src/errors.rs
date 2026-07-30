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
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::NotInitialized => write!(f, "not initialized"),
            Error::AlreadyInitialized => write!(f, "already initialized"),
            Error::NotAuthorized => write!(f, "not authorized"),
            Error::ContractNotFound => write!(f, "contract not found"),
            Error::ContractAlreadyDeprecated => write!(f, "contract already deprecated"),
            Error::InvalidTimeline => write!(f, "invalid timeline"),
            Error::InvalidPhaseTransition => write!(f, "invalid phase transition"),
            Error::TimelineNotFound => write!(f, "timeline not found"),
            Error::GuideNotFound => write!(f, "guide not found"),
            Error::ChecklistNotFound => write!(f, "checklist not found"),
            Error::InvalidChecklistIndex => write!(f, "invalid checklist index"),
        }
    }
}
