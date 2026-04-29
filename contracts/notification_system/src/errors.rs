use soroban_sdk::{contracterror, symbol_short, Symbol};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // --- Authorization (100–129) ---
    Unauthorized = 100,
    SenderNotAuthorized = 120,

    // --- Input Validation (200–299) ---
    BatchTooLarge = 208,
    RecipientsEmpty = 209,
    TitleTooLong = 221,
    MessageTooLong = 222,
    NameTooLong = 223,
    LocaleTooLong = 224,
    InvalidNotifType = 241,
    TooManyEnabledTypes = 242,

    // --- Lifecycle (300–399) ---
    NotInitialized = 300,
    AlreadyInitialized = 301,
    RateLimitExceeded = 307,
    AlreadyRead = 330,
    AlreadyArchived = 331,

    // --- Entity Existence (400–499) ---
    NotificationNotFound = 450,
    AlertRuleNotFound = 451,
    TemplateNotFound = 452,
    SenderNotFound = 453,

    // --- Financial & Resource (500–599) ---
    MaxSendersReached = 510,
    MaxRulesReached = 511,
    MaxNotificationsReached = 512,
    MaxTemplatesReached = 513,
}

pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::Unauthorized | Error::SenderNotAuthorized => symbol_short!("CHK_AUTH"),
        Error::RateLimitExceeded => symbol_short!("RE_TRY_L"),
        Error::TitleTooLong | Error::MessageTooLong | Error::NameTooLong => {
            symbol_short!("SHORTEN")
        },
        Error::NotificationNotFound | Error::AlertRuleNotFound | Error::TemplateNotFound => {
            symbol_short!("CHK_ID")
        },
        Error::MaxSendersReached
        | Error::MaxRulesReached
        | Error::MaxNotificationsReached
        | Error::MaxTemplatesReached => symbol_short!("CLN_OLD"),
        Error::BatchTooLarge | Error::TooManyEnabledTypes => symbol_short!("REDUCE"),
        Error::NotInitialized => symbol_short!("INIT_CTR"),
        Error::AlreadyInitialized | Error::AlreadyRead | Error::AlreadyArchived => {
            symbol_short!("ALREADY")
        },
        Error::RecipientsEmpty => symbol_short!("ADD_TEXT"),
        Error::LocaleTooLong => symbol_short!("FIX_LANG"),
        _ => symbol_short!("CONTACT"),
    }
}
