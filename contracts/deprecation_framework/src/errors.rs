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
