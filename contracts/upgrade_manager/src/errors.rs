use soroban_sdk::{contracterror, symbol_short, Symbol};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // --- Access Control (100–199) ---
    NotAValidator = 110,
    NotEnoughApprovals = 120,

    // --- Lifecycle & State (300–399) ---
    AlreadyInitialized = 301,
    InvalidState = 304,
    TimelockNotExpired = 376,
    ConfigNotFound = 390,

    // --- Entity Existence (400–499) ---
    ProposalNotFound = 450,
    AlreadyApproved = 451,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Error::NotAValidator => "Not A Validator",
            Error::NotEnoughApprovals => "Not Enough Approvals",
            Error::AlreadyInitialized => "Already Initialized",
            Error::InvalidState => "Invalid State",
            Error::TimelockNotExpired => "Timelock Not Expired",
            Error::ConfigNotFound => "Config Not Found",
            Error::ProposalNotFound => "Proposal Not Found",
            Error::AlreadyApproved => "Already Approved",
        };
        f.write_str(message)
    }
}

pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::NotAValidator | Error::NotEnoughApprovals => symbol_short!("CHK_AUTH"),
        Error::AlreadyInitialized | Error::AlreadyApproved => symbol_short!("ALREADY"),
        Error::ProposalNotFound | Error::ConfigNotFound => symbol_short!("CHK_ID"),
        Error::TimelockNotExpired => symbol_short!("RE_TRY_L"),
        _ => symbol_short!("CONTACT"),
    }
}
