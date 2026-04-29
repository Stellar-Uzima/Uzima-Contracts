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
    InvalidVerificationMethod = 250,
    InvalidCredentialType = 251,
    InvalidServiceEndpoint = 252,

    // --- Lifecycle & State (300–399) ---
    NotInitialized = 300,
    AlreadyInitialized = 301,
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

pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::Unauthorized | Error::NotVerifier | Error::CannotRemoveOwner => {
            symbol_short!("CHK_AUTH")
        },
        Error::NotInitialized => symbol_short!("INIT_CTR"),
        Error::AlreadyInitialized => symbol_short!("ALREADY"),
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
