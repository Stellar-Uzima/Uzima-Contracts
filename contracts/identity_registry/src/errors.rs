use soroban_sdk::{contracterror, symbol_short, Symbol};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // --- Access Control (100–199) ---
    Unauthorized = 100,
    NotVerifier = 110,
    CannotRemoveOwner = 111,
    InvalidRecoveryGuardian = 120,
    InsufficientGuardianApprovals = 121,

    // --- Input Validation (200–299) ---
    InvalidInput = 200,
    InputTooLong = 201,
    InvalidVerificationMethod = 250,
    InvalidCredentialType = 251,
    InvalidServiceEndpoint = 252,

    // --- Lifecycle & State (300–399) ---
    NotInitialized = 300,
    AlreadyInitialized = 301,
    ContractPaused = 302,
    RecoveryNotInitiated = 360,
    RecoveryAlreadyPending = 361,
    RecoveryTimelockNotElapsed = 362,

    // --- Entity Existence (400–499) ---
    VerificationMethodNotFound = 450,
    CredentialNotFound = 460,
    AttestationNotFound = 461,
    ServiceNotFound = 462,
    DIDNotFound = 470,
    DIDAlreadyExists = 471,
    DIDDeactivated = 472,

    // --- Cryptography (600–699) ---
    CredentialExpired = 605,
    CredentialRevoked = 606,
    KeyRotationCooldown = 603,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::Unauthorized => write!(f, "unauthorized"),
            Error::NotVerifier => write!(f, "not verifier"),
            Error::CannotRemoveOwner => write!(f, "cannot remove owner"),
            Error::InvalidRecoveryGuardian => write!(f, "invalid recovery guardian"),
            Error::InsufficientGuardianApprovals => write!(f, "insufficient guardian approvals"),
            Error::InvalidInput => write!(f, "invalid input"),
            Error::InputTooLong => write!(f, "input too long"),
            Error::InvalidVerificationMethod => write!(f, "invalid verification method"),
            Error::InvalidCredentialType => write!(f, "invalid credential type"),
            Error::InvalidServiceEndpoint => write!(f, "invalid service endpoint"),
            Error::NotInitialized => write!(f, "not initialized"),
            Error::AlreadyInitialized => write!(f, "already initialized"),
            Error::ContractPaused => write!(f, "contract paused"),
            Error::RecoveryNotInitiated => write!(f, "recovery not initiated"),
            Error::RecoveryAlreadyPending => write!(f, "recovery already pending"),
            Error::RecoveryTimelockNotElapsed => write!(f, "recovery timelock not elapsed"),
            Error::VerificationMethodNotFound => write!(f, "verification method not found"),
            Error::CredentialNotFound => write!(f, "credential not found"),
            Error::AttestationNotFound => write!(f, "attestation not found"),
            Error::ServiceNotFound => write!(f, "service not found"),
            Error::DIDNotFound => write!(f, "d i d not found"),
            Error::DIDAlreadyExists => write!(f, "d i d already exists"),
            Error::DIDDeactivated => write!(f, "d i d deactivated"),
            Error::CredentialExpired => write!(f, "credential expired"),
            Error::CredentialRevoked => write!(f, "credential revoked"),
            Error::KeyRotationCooldown => write!(f, "key rotation cooldown"),
        }
    }
}

pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::Unauthorized | Error::NotVerifier | Error::CannotRemoveOwner => {
            symbol_short!("CHK_AUTH")
        },
        Error::NotInitialized => symbol_short!("INIT_CTR"),
        Error::AlreadyInitialized => symbol_short!("ALREADY"),
        Error::ContractPaused => symbol_short!("RE_TRY_L"),
        Error::DIDNotFound
        | Error::CredentialNotFound
        | Error::AttestationNotFound
        | Error::ServiceNotFound
        | Error::VerificationMethodNotFound => symbol_short!("CHK_ID"),
        Error::CredentialExpired | Error::CredentialRevoked => symbol_short!("CONTACT"),
        Error::KeyRotationCooldown => symbol_short!("RE_TRY_L"),
        _ => symbol_short!("CONTACT"),
    }
}
