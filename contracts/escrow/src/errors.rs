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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Error::Unauthorized => "Unauthorized",
            Error::NotAdmin => "Not Admin",
            Error::InsufficientApprovals => "Insufficient Approvals",
            Error::InvalidAmount => "Invalid Amount",
            Error::InvalidFeeBps => "Invalid Fee Bps",
            Error::FeeNotSet => "Fee Not Set",
            Error::ReentrancyGuard => "Reentrancy Guard",
            Error::InvalidStateTransition => "Invalid State Transition",
            Error::EscrowExists => "Escrow Exists",
            Error::EscrowNotFound => "Escrow Not Found",
            Error::AlreadySettled => "Already Settled",
            Error::NoBasisToRefund => "No Basis To Refund",
            Error::NoCredit => "No Credit",
            Error::Overflow => "Overflow",
        };
        f.write_str(message)
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
