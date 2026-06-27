use soroban_sdk::{contracterror, symbol_short, Symbol};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // --- Access Control (100–199) ---
    Unauthorized = 100,
    NotAdmin = 102,
    InsufficientApprovals = 120,

    // --- Input Validation (200–299) ---
    InvalidAmount = 205,
    InvalidFeeBps = 260,

    // --- Lifecycle & State (300–399) ---
    FeeNotSet = 380,
    ReentrancyGuard = 381,
    InvalidStateTransition = 382,

    // --- Entity Existence (400–499) ---
    EscrowExists = 480,
    EscrowNotFound = 481,
    AlreadySettled = 482,

    // --- Financial & Resource (500–599) ---
    NoBasisToRefund = 560,
    NoCredit = 561,
    Overflow = 562,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::Unauthorized => write!(f, "unauthorized"),
            Error::NotAdmin => write!(f, "not admin"),
            Error::InsufficientApprovals => write!(f, "insufficient approvals"),
            Error::InvalidAmount => write!(f, "invalid amount"),
            Error::InvalidFeeBps => write!(f, "invalid fee bps"),
            Error::FeeNotSet => write!(f, "fee not set"),
            Error::ReentrancyGuard => write!(f, "reentrancy guard"),
            Error::InvalidStateTransition => write!(f, "invalid state transition"),
            Error::EscrowExists => write!(f, "escrow exists"),
            Error::EscrowNotFound => write!(f, "escrow not found"),
            Error::AlreadySettled => write!(f, "already settled"),
            Error::NoBasisToRefund => write!(f, "no basis to refund"),
            Error::NoCredit => write!(f, "no credit"),
            Error::Overflow => write!(f, "overflow"),
        }
    }
}

pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::Unauthorized | Error::NotAdmin | Error::InsufficientApprovals => {
            symbol_short!("CHK_AUTH")
        },
        Error::InvalidAmount | Error::InvalidFeeBps => symbol_short!("CHK_LEN"),
        Error::EscrowNotFound => symbol_short!("CHK_ID"),
        Error::AlreadySettled | Error::EscrowExists => symbol_short!("ALREADY"),
        _ => symbol_short!("CONTACT"),
    }
}
