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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Error::Unauthorized => "Unauthorized",
            Error::NotVerifier => "Not Verifier",
            Error::CannotRemoveOwner => "Cannot Remove Owner",
            Error::InvalidRecoveryGuardian => "Invalid Recovery Guardian",
            Error::InsufficientGuardianApprovals => "Insufficient Guardian Approvals",
            Error::InvalidInput => "Invalid Input",
            Error::InputTooLong => "Input Too Long",
            Error::InvalidVerificationMethod => "Invalid Verification Method",
            Error::InvalidCredentialType => "Invalid Credential Type",
            Error::InvalidServiceEndpoint => "Invalid Service Endpoint",
            Error::NotInitialized => "Not Initialized",
            Error::AlreadyInitialized => "Already Initialized",
            Error::ContractPaused => "Contract Paused",
            Error::RecoveryNotInitiated => "Recovery Not Initiated",
            Error::RecoveryAlreadyPending => "Recovery Already Pending",
            Error::RecoveryTimelockNotElapsed => "Recovery Timelock Not Elapsed",
            Error::VerificationMethodNotFound => "Verification Method Not Found",
            Error::CredentialNotFound => "Credential Not Found",
            Error::AttestationNotFound => "Attestation Not Found",
            Error::ServiceNotFound => "Service Not Found",
            Error::DIDNotFound => "D I D Not Found",
            Error::DIDAlreadyExists => "D I D Already Exists",
            Error::DIDDeactivated => "D I D Deactivated",
            Error::CredentialExpired => "Credential Expired",
            Error::CredentialRevoked => "Credential Revoked",
            Error::KeyRotationCooldown => "Key Rotation Cooldown",
        };
        f.write_str(message)
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
