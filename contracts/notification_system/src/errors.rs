use soroban_sdk::{contracterror};

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

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::Unauthorized => write!(f, "unauthorized"),
            Error::SenderNotAuthorized => write!(f, "sender not authorized"),
            Error::BatchTooLarge => write!(f, "batch too large"),
            Error::RecipientsEmpty => write!(f, "recipients empty"),
            Error::TitleTooLong => write!(f, "title too long"),
            Error::MessageTooLong => write!(f, "message too long"),
            Error::NameTooLong => write!(f, "name too long"),
            Error::LocaleTooLong => write!(f, "locale too long"),
            Error::InvalidNotifType => write!(f, "invalid notif type"),
            Error::TooManyEnabledTypes => write!(f, "too many enabled types"),
            Error::NotInitialized => write!(f, "not initialized"),
            Error::AlreadyInitialized => write!(f, "already initialized"),
            Error::RateLimitExceeded => write!(f, "rate limit exceeded"),
            Error::AlreadyRead => write!(f, "already read"),
            Error::AlreadyArchived => write!(f, "already archived"),
            Error::NotificationNotFound => write!(f, "notification not found"),
            Error::AlertRuleNotFound => write!(f, "alert rule not found"),
            Error::TemplateNotFound => write!(f, "template not found"),
            Error::SenderNotFound => write!(f, "sender not found"),
            Error::MaxSendersReached => write!(f, "max senders reached"),
            Error::MaxRulesReached => write!(f, "max rules reached"),
            Error::MaxNotificationsReached => write!(f, "max notifications reached"),
            Error::MaxTemplatesReached => write!(f, "max templates reached"),
        }
    }
}

#[doc(hidden)] /// Reserved for ABI consistency; not currently called from contract code.
#[cfg(test)]
pub fn get_suggestion(error: Error) -> soroban_sdk::Symbol {
    match error {
        Error::Unauthorized | Error::SenderNotAuthorized => soroban_sdk::symbol_short!("CHK_AUTH"),
        Error::RateLimitExceeded => soroban_sdk::symbol_short!("RE_TRY_L"),
        Error::TitleTooLong | Error::MessageTooLong | Error::NameTooLong => {
            soroban_sdk::symbol_short!("SHORTEN")
        },
        Error::NotificationNotFound | Error::AlertRuleNotFound | Error::TemplateNotFound => {
            soroban_sdk::symbol_short!("CHK_ID")
        },
        Error::MaxSendersReached
        | Error::MaxRulesReached
        | Error::MaxNotificationsReached
        | Error::MaxTemplatesReached => soroban_sdk::symbol_short!("CLN_OLD"),
        Error::BatchTooLarge | Error::TooManyEnabledTypes => soroban_sdk::symbol_short!("REDUCE"),
        Error::NotInitialized => soroban_sdk::symbol_short!("INIT_CTR"),
        Error::AlreadyInitialized | Error::AlreadyRead | Error::AlreadyArchived => {
            soroban_sdk::symbol_short!("ALREADY")
        },
        Error::RecipientsEmpty => soroban_sdk::symbol_short!("ADD_TEXT"),
        Error::LocaleTooLong => soroban_sdk::symbol_short!("FIX_LANG"),
        _ => soroban_sdk::symbol_short!("CONTACT"),
    }
}
