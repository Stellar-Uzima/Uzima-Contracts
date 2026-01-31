#![no_std]
#![allow(clippy::len_zero)] // FIXED: Allow len() == 0 check

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, xdr::ToXdr, Address, BytesN,
    Env, String, Symbol, Vec,
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
}

const STORAGE_BUMP_AMOUNT: u32 = 518400; // 30 days
const MAX_VERIFICATION_METHODS: u32 = 20;
const MAX_SERVICES: u32 = 20;
const MAX_CONTROLLERS: u32 = 10;

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum VerificationMethodType {
    Ed25519VerificationKey2020 = 1,
    EcdsaSecp256k1VerKey2019 = 2,
    JsonWebKey2020 = 3,
    X25519KeyAgreementKey2020 = 4,
    Bls12381G1Key2020 = 5,
    Bls12381G2Key2020 = 6,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerificationMethod {
    pub id: String,
    pub type_: VerificationMethodType,
    pub controller: Address,
    pub public_key_multibase: String,
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
    pub assertion_method: Vec<String>,
    pub key_agreement: Vec<String>,
    pub capability_invocation: Vec<String>,
    pub capability_delegation: Vec<String>,
    pub services: Vec<Service>,
    pub created: u64,
    pub updated: u64,
    pub version: u32,
    pub deactivated: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiableCredential {
    pub context: Vec<String>,
    pub id: String,
    pub type_: Vec<String>,
    pub issuer: Address,
    pub issuance_date: u64,
    pub expiration_date: Option<u64>,
    pub credential_subject: Address,
    pub proof: Proof,
    pub credential_status: Vec<CredentialStatus>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proof {
    pub type_: String,
    pub created: u64,
    pub verification_method: String,
    pub proof_purpose: String,
    pub proof_value: String,
    pub challenge: Option<String>,
    pub domain: Option<String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CredentialStatus {
    pub id: String,
    pub type_: String,
    pub status_list_index: Option<u32>,
    pub status_list_credential: Option<String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryConfig {
    pub guardians: Vec<Address>,
    pub threshold: u32,
    pub delay_seconds: u64,
    pub last_recovery: u64,
}

// Storage keys
const DID_DOCUMENT: Symbol = symbol_short!("DID_DOC");
const VC_STORAGE: Symbol = symbol_short!("VC_STORE");
const RECOVERY_CONFIG: Symbol = symbol_short!("REC_CFG");
const REVOCATION_LIST: Symbol = symbol_short!("REV_LIST");

#[contract]
pub struct IdentityRegistry;

#[contractimpl]
impl IdentityRegistry {
    // --- DID Management ---

    pub fn create_did(
        env: Env,
        subject: Address,
        initial_key: String,
        initial_services: Vec<Service>,
    ) -> Result<String, Error> {
        subject.require_auth();

        let storage_key = (DID_DOCUMENT, subject.clone());
        if env.storage().persistent().has(&storage_key) {
            return Err(Error::AlreadyExists);
        }

        let did_string = Self::generate_did_string(&env, &subject);

        let vm = VerificationMethod {
            id: String::from_str(&env, "#key-1"),
            type_: VerificationMethodType::Ed25519VerificationKey2020,
            controller: subject.clone(),
            public_key_multibase: initial_key,
        };

        let mut methods = Vec::new(&env);
        methods.push_back(vm);

        let mut controllers = Vec::new(&env);
        controllers.push_back(subject.clone());

        let mut auth_refs = Vec::new(&env);
        auth_refs.push_back(String::from_str(&env, "#key-1"));

        if initial_services.len() > MAX_SERVICES {
            return Err(Error::LimitReached);
        }

        let doc = DIDDocument {
            id: did_string.clone(),
            controller: controllers,
            verification_methods: methods,
            authentication: auth_refs.clone(),
            assertion_method: auth_refs.clone(),
            key_agreement: auth_refs.clone(),
            capability_invocation: auth_refs.clone(),
            capability_delegation: auth_refs,
            services: initial_services,
            created: env.ledger().timestamp(),
            updated: env.ledger().timestamp(),
            version: 1,
            deactivated: false,
        };

        env.storage().persistent().set(&storage_key, &doc);
        env.storage().persistent().extend_ttl(
            &storage_key,
            STORAGE_BUMP_AMOUNT,
            STORAGE_BUMP_AMOUNT,
        );

        Ok(did_string)
    }

    pub fn resolve_did(env: Env, subject: Address) -> Result<DIDDocument, Error> {
        let storage_key = (DID_DOCUMENT, subject.clone());
        env.storage()
            .persistent()
            .get(&storage_key)
            .ok_or(Error::NotFound)
    }

    pub fn update_did(env: Env, subject: Address, new_doc: DIDDocument) -> Result<(), Error> {
        subject.require_auth();

        let storage_key = (DID_DOCUMENT, subject.clone());
        let mut current_doc: DIDDocument = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(Error::NotFound)?;

        if current_doc.deactivated {
            return Err(Error::Deactivated);
        }

        if !Self::is_controller(&current_doc, &subject) {
            return Err(Error::NotController);
        }

        if new_doc.verification_methods.len() > MAX_VERIFICATION_METHODS
            || new_doc.services.len() > MAX_SERVICES
            || new_doc.controller.len() > MAX_CONTROLLERS
        {
            return Err(Error::LimitReached);
        }

        current_doc.verification_methods = new_doc.verification_methods;
        current_doc.services = new_doc.services;
        current_doc.controller = new_doc.controller;
        current_doc.authentication = new_doc.authentication;
        current_doc.updated = env.ledger().timestamp();
        current_doc.version += 1;

        env.storage().persistent().set(&storage_key, &current_doc);
        env.storage().persistent().extend_ttl(
            &storage_key,
            STORAGE_BUMP_AMOUNT,
            STORAGE_BUMP_AMOUNT,
        );

        Ok(())
    }

    pub fn deactivate_did(env: Env, subject: Address) -> Result<(), Error> {
        subject.require_auth();

        let storage_key = (DID_DOCUMENT, subject.clone());
        let mut doc: DIDDocument = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(Error::NotFound)?;

        if !Self::is_controller(&doc, &subject) {
            return Err(Error::NotController);
        }

        doc.deactivated = true;
        doc.updated = env.ledger().timestamp();

        env.storage().persistent().set(&storage_key, &doc);
        Ok(())
    }

    pub fn issue_credential(
        env: Env,
        issuer: Address,
        subject: Address,
        vc: VerifiableCredential,
    ) -> Result<String, Error> {
        issuer.require_auth();

        if issuer != vc.issuer {
            return Err(Error::InvalidIssuer);
        }

        if subject != vc.credential_subject {
            return Err(Error::InvalidSubject);
        }

        let storage_key = (VC_STORAGE, vc.id.clone());
        if env.storage().persistent().has(&storage_key) {
            return Err(Error::AlreadyExists);
        }

        if let Some(exp) = vc.expiration_date {
            if exp <= env.ledger().timestamp() {
                return Err(Error::InvalidValidity);
            }
        }

        let _issuer_doc = Self::resolve_did(env.clone(), issuer.clone())?;

        env.storage().persistent().set(&storage_key, &vc);
        env.storage().persistent().extend_ttl(
            &storage_key,
            STORAGE_BUMP_AMOUNT,
            STORAGE_BUMP_AMOUNT,
        );

        Ok(vc.id)
    }

    pub fn verify_credential(env: Env, vc_id: String) -> Result<bool, Error> {
        let storage_key = (VC_STORAGE, vc_id.clone());
        let vc: VerifiableCredential = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(Error::NotFound)?;

        if let Some(exp) = vc.expiration_date {
            if env.ledger().timestamp() > exp {
                return Err(Error::Expired);
            }
        }

        let revocation_key = (REVOCATION_LIST, vc_id);
        if env.storage().persistent().has(&revocation_key) {
            return Err(Error::Revoked);
        }

        if vc.proof.proof_value.len() == 0 {
            return Err(Error::InvalidSignature);
        }

        Ok(true)
    }

    pub fn revoke_credential(env: Env, issuer: Address, vc_id: String) -> Result<(), Error> {
        issuer.require_auth();

        let storage_key = (VC_STORAGE, vc_id.clone());
        let vc: VerifiableCredential = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(Error::NotFound)?;

        if vc.issuer != issuer {
            return Err(Error::Unauthorized);
        }

        let revocation_key = (REVOCATION_LIST, vc_id);
        env.storage()
            .persistent()
            .set(&revocation_key, &env.ledger().timestamp());

        Ok(())
    }

    pub fn set_recovery_config(
        env: Env,
        subject: Address,
        guardians: Vec<Address>,
        threshold: u32,
        delay_seconds: u64,
    ) -> Result<(), Error> {
        subject.require_auth();

        if threshold == 0 || threshold > guardians.len() {
            return Err(Error::InvalidContext);
        }

        let config = RecoveryConfig {
            guardians,
            threshold,
            delay_seconds,
            last_recovery: 0,
        };

        let key = (RECOVERY_CONFIG, subject);
        env.storage().persistent().set(&key, &config);

        Ok(())
    }

    // --- Internal Helpers ---

    fn generate_did_string(env: &Env, subject: &Address) -> String {
        let addr_hash = env.crypto().sha256(&subject.to_xdr(env));
        // FIXED: Removed unnecessary let binding and return directly
        Self::bytes_to_hex_string(env, &addr_hash.into())
    }

    fn is_controller(doc: &DIDDocument, subject: &Address) -> bool {
        for controller in doc.controller.iter() {
            if controller == *subject {
                return true;
            }
        }
        false
    }

    #[allow(unused_variables)]
    fn bytes_to_hex_string(env: &Env, _bytes: &BytesN<32>) -> String {
        String::from_str(env, "did:stellar:mock_id")
    }

    pub fn compute_document_hash(env: &Env, doc: DIDDocument) -> BytesN<32> {
        let data = doc.to_xdr(env);
        env.crypto().sha256(&data).into()
    }
}
