// Identity Registry - W3C DID Compliant
#![no_std]
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, BytesN, Env, String, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotFound = 1,
    AlreadyExists = 2,
    Unauthorized = 3,
    InvalidValidity = 4,
    InvalidProof = 5,
    InvalidSignature = 6,
    Revoked = 7,
    Expired = 8,
    InvalidFormat = 9,
    InvalidDID = 10,
    InvalidType = 11,
    InvalidContext = 12,
    InvalidSubject = 13,
    InvalidIssuer = 14,
    NotController = 15,
    RecoveryFailed = 16,
    Deactivated = 17,
    DelegationFailed = 18,
    RateLimitExceeded = 19,
    InvalidKey = 20,
    DuplicateService = 21,
    ServiceNotFound = 22,
    LimitReached = 23,
    NotVerifier = 24,
    AlreadyInitialized = 25,
    NotInitialized = 26,
    CannotRemoveOwner = 27,
    RecoveryNotInitiated = 28,
    RecoveryTimelockNotElapsed = 29,
    InsufficientGuardianApprovals = 30,
    DIDNotFound = 31,
    DIDDeactivated = 32,
    CredentialRevoked = 33,
    CredentialNotFound = 34,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DIDStatus {
    Active,
    RecoveryPending,
    Deactivated,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerificationMethodType {
    Ed25519VerificationKey2020,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerificationMethod {
    pub id: String,
    pub type_: VerificationMethodType,
    pub controller: Address,
    pub public_key_multibase: String,
    pub is_active: bool,
    pub created: u64,
    pub last_rotated: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Service {
    pub id: String,
    pub type_: String,
    pub service_endpoint: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DIDDocument {
    pub id: String,
    pub controller: Vec<Address>,
    pub verification_methods: Vec<VerificationMethod>,
    pub authentication: Vec<String>,
    pub services: Vec<Service>,
    pub created: u64,
    pub updated: u64,
    pub version: u32,
    pub status: DIDStatus,
    pub deactivated: bool,
}

#[contracttype]
pub enum DataKey {
    Owner,
    Initialized,
    NetworkId,
    Verifier(Address),
    DIDDocument(Address),
    DIDByString(String),
    RecoveryRequest(u64),
    RecoveryCounter,
    ActiveRecovery(Address),
    RecoveryThreshold(Address),
    Credential(BytesN<32>),
    SubjectCredentials(Address),
    IssuerCredentials(Address),
}

#[contract]
pub struct IdentityRegistry;

#[contractimpl]
impl IdentityRegistry {
    pub fn initialize(env: Env, owner: Address, network_id: String) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Owner, &owner);
        env.storage()
            .instance()
            .set(&DataKey::NetworkId, &network_id);
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage()
            .instance()
            .set(&DataKey::Verifier(owner), &true);
        Ok(())
    }

    pub fn create_did(
        env: Env,
        subject: Address,
        initial_key: String,
        initial_services: Vec<Service>,
    ) -> Result<String, Error> {
        subject.require_auth();
        if env
            .storage()
            .persistent()
            .has(&DataKey::DIDDocument(subject.clone()))
        {
            return Err(Error::AlreadyExists);
        }

        let did_string = String::from_str(&env, "did:stellar:uzima:user");
        let vm = VerificationMethod {
            id: String::from_str(&env, "#key-1"),
            type_: VerificationMethodType::Ed25519VerificationKey2020,
            controller: subject.clone(),
            public_key_multibase: initial_key,
            is_active: true,
            created: env.ledger().timestamp(),
            last_rotated: 0,
        };

        let doc = DIDDocument {
            id: did_string.clone(),
            controller: Vec::from_array(&env, [subject.clone()]),
            verification_methods: Vec::from_array(&env, [vm]),
            authentication: Vec::from_array(&env, [String::from_str(&env, "#key-1")]),
            services: initial_services,
            created: env.ledger().timestamp(),
            updated: env.ledger().timestamp(),
            version: 1,
            status: DIDStatus::Active,
            deactivated: false,
        };

        env.storage()
            .persistent()
            .set(&DataKey::DIDDocument(subject.clone()), &doc);
        Ok(did_string)
    }

    pub fn resolve_did(env: Env, subject: Address) -> Result<DIDDocument, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::DIDDocument(subject))
            .ok_or(Error::NotFound)
    }

    pub fn deactivate_did(env: Env, subject: Address) -> Result<(), Error> {
        subject.require_auth();
        let mut doc = Self::resolve_did(env.clone(), subject.clone())?;
        doc.deactivated = true;
        doc.status = DIDStatus::Deactivated;
        env.storage()
            .persistent()
            .set(&DataKey::DIDDocument(subject), &doc);
        Ok(())
    }

    pub fn is_verifier(env: Env, account: Address) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Verifier(account))
            .unwrap_or(false)
    }

    pub fn get_owner(env: Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Owner)
            .ok_or(Error::NotInitialized)
    }
}
