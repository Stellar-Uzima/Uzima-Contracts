use soroban_sdk::{contracterror, symbol_short, Symbol};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // --- Lifecycle (1–2) ---
    AlreadyInitialized = 1,
    NotInitialized = 2,

    // --- Authorization (3–4) ---
    NotAuthorized = 3,
    SenderNotAuthorized = 4,

    // --- Capacity limits (5–8) ---
    MaxSendersReached = 5,
    MaxRulesReached = 6,
    MaxNotificationsReached = 7,
    MaxTemplatesReached = 8,

    // --- Input validation (9–16) ---
    TitleTooLong = 9,
    MessageTooLong = 10,
    NameTooLong = 11,
    LocaleTooLong = 12,
    InvalidNotifType = 13,
    BatchTooLarge = 14,
    RecipientsEmpty = 15,
    TooManyEnabledTypes = 16,

    // --- Not found (17–20) ---
    NotificationNotFound = 17,
    AlertRuleNotFound = 18,
    TemplateNotFound = 19,
    SenderNotFound = 20,

    // --- State transitions (21–23) ---
    AlreadyRead = 21,
    AlreadyArchived = 22,
    RateLimitExceeded = 23,
}

/// Recovery hints surfaced to callers alongside an error.
pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::NotAuthorized | Error::SenderNotAuthorized => symbol_short!("CHK_AUTH"),
        Error::RateLimitExceeded => symbol_short!("RE_TRY_L"),
        Error::TitleTooLong | Error::MessageTooLong | Error::NameTooLong => {
            symbol_short!("SHORTEN")
        }
        Error::NotificationNotFound | Error::AlertRuleNotFound | Error::TemplateNotFound => {
            symbol_short!("CHK_ID")
        }
        Error::MaxSendersReached
        | Error::MaxRulesReached
        | Error::MaxNotificationsReached
        | Error::MaxTemplatesReached => symbol_short!("CLN_OLD"),
        Error::BatchTooLarge | Error::TooManyEnabledTypes => symbol_short!("REDUCE"),
        _ => symbol_short!("CONTACT"),
    }
}
