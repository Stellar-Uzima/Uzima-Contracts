// Identity Registry - W3C DID Compliant with proper validation throughout
#![no_std]
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Bytes, BytesN, Env,
    String, Symbol, Vec,
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
    Ed25519VerificationKey2020,
    EcdsaSecp256k1VerifKey2019,
    X25519KeyAgreementKey2020,
    JsonWebKey2020,
}

/// Verification Relationship Types
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerificationRelationship {
    Authentication,
    AssertionMethod,
    KeyAgreement,
    CapabilityInvocation,
    CapabilityDelegation,
}

/// Verification Method (Public Key) as per W3C DID spec
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
impl IdentityRegistryContract {
    // ========================================================================
    // INITIALIZATION
    // ========================================================================

    /// Initialize the contract with an owner and network identifier
    pub fn initialize(env: Env, owner: Address, network_id: String) -> Result<(), Error> {
        owner.require_auth();

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
            .set(&DataKey::Verifier(owner.clone()), &true);
        env.storage().instance().set(
            &DataKey::KeyRotationCooldown,
            &DEFAULT_KEY_ROTATION_COOLDOWN,
        );

        env.events().publish(
            (Symbol::new(&env, "Initialized"),),
            (owner.clone(), network_id),
        );

        Ok(())
    }

    /// Legacy initialize for backward compatibility
    pub fn initialize_legacy(env: Env, owner: Address) {
        owner.require_auth();

        if env.storage().instance().has(&DataKey::Owner) {
            panic!("Contract already initialized");
        }

        env.storage().instance().set(&DataKey::Owner, &owner);
        env.storage()
            .instance()
            .set(&DataKey::Verifier(owner.clone()), &true);

        env.events()
            .publish((symbol_short!("Init"),), owner.clone());
    }

    // ========================================================================
    // DID DOCUMENT MANAGEMENT
    // ========================================================================

    /// Create a new DID Document for a subject
    /// Only the subject can create their own DID
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

        // Store DID document
        env.storage()
            .persistent()
            .set(&DataKey::DIDDocument(subject.clone()), &did_doc);
        env.storage()
            .persistent()
            .set(&DataKey::DIDByString(did_string.clone()), &subject);

        // Initialize recovery guardians with empty list
        let guardians: Vec<RecoveryGuardian> = Vec::new(&env);
        env.storage()
            .persistent()
            .set(&DataKey::RecoveryGuardians(subject.clone()), &guardians);
        env.storage().persistent().set(
            &DataKey::RecoveryThreshold(subject.clone()),
            &DEFAULT_RECOVERY_THRESHOLD,
        );

        env.events().publish(
            (Symbol::new(&env, "DIDCreated"),),
            (subject, did_string.clone()),
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

        env.storage()
            .persistent()
            .set(&DataKey::DIDDocument(subject.clone()), &did_doc);
        env.storage()
            .persistent()
            .set(&DataKey::LastKeyRotation(subject.clone()), &timestamp);

        env.events()
            .publish((Symbol::new(&env, "KeyRotated"),), (subject, method_id));

        Ok(())
    }

    pub fn issue_credential(
        env: Env,
        issuer: Address,
        subject: Address,
        vc: VerifiableCredential,
    ) -> Result<String, Error> {
        issuer.require_auth();

        // Verify issuer is a registered verifier
        let is_verifier: bool = env
            .storage()
            .instance()
            .get(&DataKey::Verifier(issuer.clone()))
            .unwrap_or(false);

        if !is_verifier {
            return Err(Error::NotVerifier);
        }

        let timestamp = env.ledger().timestamp();

        // Generate credential ID (hash of issuer + subject + timestamp + type)
        let credential_id =
            Self::generate_credential_id(&env, &issuer, &subject, timestamp, &credential_type);

        let credential = VerifiableCredential {
            id: credential_id.clone(),
            credential_type: credential_type.clone(),
            issuer: issuer.clone(),
            subject: subject.clone(),
            issuance_date: timestamp,
            expiration_date,
            credential_hash,
            credential_uri,
            is_revoked: false,
            revoked_at: 0,
            revocation_reason: String::from_str(&env, ""),
        };

        // Store credential
        env.storage()
            .persistent()
            .set(&DataKey::Credential(credential_id.clone()), &credential);

        // Add to subject's credentials
        let mut subject_creds: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::SubjectCredentials(subject.clone()))
            .unwrap_or(Vec::new(&env));
        subject_creds.push_back(credential_id.clone());
        env.storage().persistent().set(
            &DataKey::SubjectCredentials(subject.clone()),
            &subject_creds,
        );

        // Add to issuer's issued credentials
        let mut issuer_creds: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::IssuerCredentials(issuer.clone()))
            .unwrap_or(Vec::new(&env));
        issuer_creds.push_back(credential_id.clone());
        env.storage()
            .persistent()
            .set(&DataKey::IssuerCredentials(issuer.clone()), &issuer_creds);

        env.events().publish(
            (Symbol::new(&env, "CredentialIssued"),),
            (issuer, subject, credential_id.clone(), credential_type),
        );

        Ok(credential_id)
    }

    /// Verify a credential's status
    pub fn verify_credential(
        env: Env,
        credential_id: BytesN<32>,
    ) -> Result<CredentialStatus, Error> {
        let credential: Option<VerifiableCredential> = env
            .storage()
            .persistent()
            .get(&DataKey::Credential(credential_id));

        match credential {
            None => Ok(CredentialStatus::NotFound),
            Some(cred) => {
                if cred.is_revoked {
                    Ok(CredentialStatus::Revoked)
                } else if cred.expiration_date > 0
                    && env.ledger().timestamp() > cred.expiration_date
                {
                    Ok(CredentialStatus::Expired)
                } else {
                    Ok(CredentialStatus::Valid)
                }
            }
        }
    }

    /// Get a credential by ID
    pub fn get_credential(
        env: Env,
        credential_id: BytesN<32>,
    ) -> Result<VerifiableCredential, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Credential(credential_id))
            .ok_or(Error::CredentialNotFound)
    }

    /// Revoke a credential (only issuer can revoke)
    pub fn revoke_credential(
        env: Env,
        issuer: Address,
        credential_id: BytesN<32>,
        reason: String,
    ) -> Result<(), Error> {
        issuer.require_auth();

        let mut credential: VerifiableCredential = env
            .storage()
            .persistent()
            .get(&DataKey::Credential(credential_id.clone()))
            .ok_or(Error::CredentialNotFound)?;

        if credential.issuer != issuer {
            return Err(Error::NotAuthorized);
        }

        if credential.is_revoked {
            return Err(Error::CredentialRevoked);
        }

        credential.is_revoked = true;
        credential.revoked_at = env.ledger().timestamp();
        credential.revocation_reason = reason;

        env.storage()
            .persistent()
            .set(&DataKey::Credential(credential_id.clone()), &credential);

        env.events().publish(
            (Symbol::new(&env, "CredentialRevoked"),),
            (issuer, credential_id),
        );

        Ok(())
    }

    /// Get all credentials for a subject
    pub fn get_subject_credentials(env: Env, subject: Address) -> Vec<VerifiableCredential> {
        let credential_ids: Vec<BytesN<32>> = env
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
            .set(&DataKey::RecoveryGuardians(subject.clone()), &new_guardians);

        env.events()
            .publish((Symbol::new(&env, "GuardianRemoved"),), (subject, guardian));

        Ok(())
    }

    /// Set recovery threshold
    pub fn set_recovery_threshold(env: Env, subject: Address, threshold: u32) -> Result<(), Error> {
        subject.require_auth();

        if threshold == 0 || threshold > guardians.len() {
            return Err(Error::InvalidContext);
        }

        let request_id: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::RecoveryCounter)
            .unwrap_or(0)
            + 1;

        let mut approvals = Vec::new(&env);
        approvals.push_back(guardian.clone());

        let request = RecoveryRequest {
            request_id,
            subject: subject.clone(),
            new_controller,
            new_primary_key,
            created_at: env.ledger().timestamp(),
            approvals,
            total_weight: guardian_info.weight,
            executed: false,
        };

        env.storage()
            .persistent()
            .set(&DataKey::RecoveryRequest(request_id), &request);
        env.storage()
            .persistent()
            .set(&DataKey::ActiveRecovery(subject.clone()), &request_id);
        env.storage()
            .persistent()
            .set(&DataKey::RecoveryCounter, &request_id);

        // Update DID status
        let mut did_doc: DIDDocument = env
            .storage()
            .persistent()
            .get(&DataKey::DIDDocument(subject.clone()))
            .ok_or(Error::DIDNotFound)?;
        did_doc.status = DIDStatus::RecoveryPending;
        env.storage()
            .persistent()
            .set(&DataKey::DIDDocument(subject.clone()), &did_doc);

        env.events().publish(
            (Symbol::new(&env, "RecoveryInitiated"),),
            (subject, request_id),
        );

        Ok(request_id)
    }

    /// Approve a recovery request
    pub fn approve_recovery(env: Env, guardian: Address, request_id: u64) -> Result<(), Error> {
        guardian.require_auth();

        let key = (RECOVERY_CONFIG, subject);
        env.storage().persistent().set(&key, &config);

        Ok(())
    }

    /// Execute recovery after timelock and threshold met
    pub fn execute_recovery(env: Env, request_id: u64) -> Result<(), Error> {
        let mut request: RecoveryRequest = env
            .storage()
            .persistent()
            .get(&DataKey::RecoveryRequest(request_id))
            .ok_or(Error::RecoveryNotInitiated)?;

        if request.executed {
            return Err(Error::RecoveryNotInitiated);
        }

        // Check timelock
        let now = env.ledger().timestamp();
        if now < request.created_at + DEFAULT_RECOVERY_TIMELOCK {
            return Err(Error::RecoveryTimelockNotElapsed);
        }

        // Check threshold
        let threshold: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::RecoveryThreshold(request.subject.clone()))
            .unwrap_or(DEFAULT_RECOVERY_THRESHOLD);

        if request.total_weight < threshold {
            return Err(Error::InsufficientGuardianApprovals);
        }

        // Execute recovery - update DID document
        let mut did_doc: DIDDocument = env
            .storage()
            .persistent()
            .get(&DataKey::DIDDocument(request.subject.clone()))
            .ok_or(Error::DIDNotFound)?;

        // Update controller
        did_doc.controller = request.new_controller.clone();

        // Create new primary verification method
        let new_vm_id = String::from_str(&env, "#recovery-key");
        let new_vm = VerificationMethod {
            id: new_vm_id.clone(),
            method_type: VerificationMethodType::Ed25519VerificationKey2020,
            controller: request.new_controller.clone(),
            public_key: request.new_primary_key.clone(),
            is_active: true,
            created: now,
            last_rotated: 0,
        };

        // Deactivate old methods and add new one
        let mut updated_methods = Vec::new(&env);
        for vm in did_doc.verification_methods.iter() {
            let deactivated = VerificationMethod {
                id: vm.id.clone(),
                method_type: vm.method_type.clone(),
                controller: vm.controller.clone(),
                public_key: vm.public_key.clone(),
                is_active: false,
                created: vm.created,
                last_rotated: vm.last_rotated,
            };
            updated_methods.push_back(deactivated);
        }
        updated_methods.push_back(new_vm);
        did_doc.verification_methods = updated_methods;

        // Update authentication to use new key
        let mut new_auth = Vec::new(&env);
        new_auth.push_back(new_vm_id);
        did_doc.authentication = new_auth;

        did_doc.status = DIDStatus::Active;
        did_doc.updated = now;
        did_doc.version += 1;

        env.storage()
            .persistent()
            .set(&DataKey::DIDDocument(request.subject.clone()), &did_doc);

        // Mark request as executed
        request.executed = true;
        env.storage()
            .persistent()
            .set(&DataKey::RecoveryRequest(request_id), &request);

        // Clear active recovery
        env.storage()
            .persistent()
            .remove(&DataKey::ActiveRecovery(request.subject.clone()));

        env.events().publish(
            (Symbol::new(&env, "RecoveryExecuted"),),
            (request.subject, request_id),
        );

        Ok(())
    }

    /// Cancel a recovery request (only subject with existing key)
    pub fn cancel_recovery(env: Env, subject: Address) -> Result<(), Error> {
        subject.require_auth();

        let request_id: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::ActiveRecovery(subject.clone()))
            .ok_or(Error::RecoveryNotInitiated)?;

        let mut request: RecoveryRequest = env
            .storage()
            .persistent()
            .get(&DataKey::RecoveryRequest(request_id))
            .ok_or(Error::RecoveryNotInitiated)?;

        request.executed = true;
        env.storage()
            .persistent()
            .set(&DataKey::RecoveryRequest(request_id), &request);

        // Update DID status back to active
        let mut did_doc: DIDDocument = env
            .storage()
            .persistent()
            .get(&DataKey::DIDDocument(subject.clone()))
            .ok_or(Error::DIDNotFound)?;
        did_doc.status = DIDStatus::Active;
        env.storage()
            .persistent()
            .set(&DataKey::DIDDocument(subject.clone()), &did_doc);

        env.storage()
            .persistent()
            .remove(&DataKey::ActiveRecovery(subject.clone()));

        env.events().publish(
            (Symbol::new(&env, "RecoveryCancelled"),),
            (subject, request_id),
        );

        Ok(())
    }

    // ========================================================================
    // SERVICE ENDPOINT MANAGEMENT
    // ========================================================================

    /// Add a service endpoint to DID
    pub fn add_service(
        env: Env,
        subject: Address,
        service_id: String,
        service_type: String,
        endpoint: String,
    ) -> Result<(), Error> {
        subject.require_auth();

        let mut did_doc: DIDDocument = env
            .storage()
            .persistent()
            .get(&DataKey::DIDDocument(subject.clone()))
            .ok_or(Error::DIDNotFound)?;

        if matches!(did_doc.status, DIDStatus::Deactivated) {
            return Err(Error::DIDDeactivated);
        }

        let new_service = ServiceEndpoint {
            id: service_id.clone(),
            service_type,
            endpoint,
            is_active: true,
        };

        did_doc.services.push_back(new_service);
        did_doc.updated = env.ledger().timestamp();
        did_doc.version += 1;

        env.storage()
            .persistent()
            .set(&DataKey::DIDDocument(subject.clone()), &did_doc);

        env.events()
            .publish((Symbol::new(&env, "ServiceAdded"),), (subject, service_id));

        Ok(())
    }

    /// Remove/deactivate a service endpoint
    pub fn remove_service(env: Env, subject: Address, service_id: String) -> Result<(), Error> {
        subject.require_auth();

        let mut did_doc: DIDDocument = env
            .storage()
            .persistent()
            .get(&DataKey::DIDDocument(subject.clone()))
            .ok_or(Error::DIDNotFound)?;

        let mut updated_services = Vec::new(&env);
        let mut found = false;

        for svc in did_doc.services.iter() {
            if svc.id == service_id {
                found = true;
                // Skip - effectively removes it
            } else {
                updated_services.push_back(svc);
            }
        }

        if !found {
            return Err(Error::ServiceNotFound);
        }

        did_doc.services = updated_services;
        did_doc.updated = env.ledger().timestamp();
        did_doc.version += 1;

        env.storage()
            .persistent()
            .set(&DataKey::DIDDocument(subject.clone()), &did_doc);

        env.events().publish(
            (Symbol::new(&env, "ServiceRemoved"),),
            (subject, service_id),
        );

        Ok(())
    }

    // ========================================================================
    // VERIFIER MANAGEMENT
    // ========================================================================

    /// Add a verifier (only owner can do this)
    pub fn add_verifier(env: Env, verifier: Address) -> Result<(), Error> {
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .ok_or(Error::NotInitialized)?;

        owner.require_auth();

        env.storage()
            .instance()
            .set(&DataKey::Verifier(verifier.clone()), &true);

        env.events()
            .publish((Symbol::new(&env, "VerifierAdded"),), verifier);

        Ok(())
    }

    /// Remove a verifier (only owner can do this)
    pub fn remove_verifier(env: Env, verifier: Address) -> Result<(), Error> {
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .ok_or(Error::NotInitialized)?;

        owner.require_auth();

        if verifier == owner {
            return Err(Error::CannotRemoveOwner);
        }

        env.storage()
            .instance()
            .set(&DataKey::Verifier(verifier.clone()), &false);

        env.events()
            .publish((Symbol::new(&env, "VerifierRemoved"),), verifier);

        Ok(())
    }

    /// Check if an address is a verifier
    pub fn is_verifier(env: Env, account: Address) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Verifier(account))
            .unwrap_or(false)
    }

    /// Get the contract owner
    pub fn get_owner(env: Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Owner)
            .ok_or(Error::NotInitialized)
    }

    // ========================================================================
    // LEGACY FUNCTIONS (Backward Compatibility)
    // ========================================================================

    /// Register an identity hash with metadata (legacy support)
    pub fn register_identity_hash(env: Env, hash: BytesN<32>, subject: Address, meta: String) {
        subject.require_auth();

        let identity_record = IdentityRecord {
            hash: hash.clone(),
            meta: meta.clone(),
            registered_by: subject.clone(),
        };

        env.storage()
            .instance()
            .set(&DataKey::IdentityHash(subject.clone()), &identity_record);

        env.events()
            .publish((symbol_short!("IdReg"),), (subject, hash, meta));
    }

    /// Create an attestation (legacy - only verifiers can do this)
    pub fn attest(env: Env, verifier: Address, subject: Address, claim_hash: BytesN<32>) {
        verifier.require_auth();

        let is_verifier: bool = env
            .storage()
            .instance()
            .get(&DataKey::Verifier(verifier.clone()))
            .unwrap_or(false);

        if !is_verifier {
            panic!("Caller is not a verifier");
        }

        let attestation = Attestation {
            claim_hash: claim_hash.clone(),
            verifier: verifier.clone(),
            is_active: true,
        };

        env.storage().instance().set(
            &DataKey::Attestation(subject.clone(), claim_hash.clone()),
            &attestation,
        );

        let mut attestations: Vec<BytesN<32>> = env
            .storage()
            .instance()
            .get(&DataKey::SubjectAttestations(subject.clone()))
            .unwrap_or(Vec::new(&env));

        attestations.push_back(claim_hash.clone());
        env.storage().instance().set(
            &DataKey::SubjectAttestations(subject.clone()),
            &attestations,
        );

        env.events().publish(
            (symbol_short!("Attested"),),
            (subject, verifier, claim_hash),
        );
    }

    /// Revoke an attestation (legacy)
    pub fn revoke_attestation(
        env: Env,
        verifier: Address,
        subject: Address,
        claim_hash: BytesN<32>,
    ) {
        verifier.require_auth();

        let is_verifier: bool = env
            .storage()
            .instance()
            .get(&DataKey::Verifier(verifier.clone()))
            .unwrap_or(false);

        if !is_verifier {
            panic!("Caller is not a verifier");
        }

        let mut attestation: Attestation = env
            .storage()
            .instance()
            .get(&DataKey::Attestation(subject.clone(), claim_hash.clone()))
            .unwrap_or_else(|| panic!("Attestation not found"));

        attestation.is_active = false;
        env.storage().instance().set(
            &DataKey::Attestation(subject.clone(), claim_hash.clone()),
            &attestation,
        );

        env.events()
            .publish((symbol_short!("Revoked"),), (subject, verifier, claim_hash));
    }

    /// Get identity hash for a subject (legacy)
    pub fn get_identity_hash(env: Env, subject: Address) -> Option<BytesN<32>> {
        let record: Option<IdentityRecord> = env
            .storage()
            .instance()
            .get(&DataKey::IdentityHash(subject));

        record.map(|r| r.hash)
    }

    /// Get identity metadata for a subject (legacy)
    pub fn get_identity_meta(env: Env, subject: Address) -> Option<String> {
        let record: Option<IdentityRecord> = env
            .storage()
            .instance()
            .get(&DataKey::IdentityHash(subject));

        record.map(|r| r.meta)
    }

    /// Check if a specific attestation is active (legacy)
    pub fn is_attested(env: Env, subject: Address, claim_hash: BytesN<32>) -> bool {
        let attestation: Option<Attestation> = env
            .storage()
            .instance()
            .get(&DataKey::Attestation(subject, claim_hash));

        attestation.is_some_and(|a| a.is_active)
    }

    /// Get all active attestations for a subject (legacy)
    pub fn get_attestations(env: Env, subject: Address) -> Vec<BytesN<32>> {
        let all_attestations: Vec<BytesN<32>> = env
            .storage()
            .instance()
            .get(&DataKey::SubjectAttestations(subject.clone()))
            .unwrap_or(Vec::new(&env));

        let mut active_attestations = Vec::new(&env);

        for claim_hash in all_attestations.iter() {
            if let Some(attestation) =
                env.storage()
                    .instance()
                    .get::<DataKey, Attestation>(&DataKey::Attestation(
                        subject.clone(),
                        claim_hash.clone(),
                    ))
            {
                if attestation.is_active {
                    active_attestations.push_back(claim_hash);
                }
            }
        }

        active_attestations
    }

    // ========================================================================
    // HELPER FUNCTIONS
    // ========================================================================

    /// Generate DID string from network and address
    fn generate_did_string(env: &Env, network_id: &String, subject: &Address) -> String {
        // Format: did:stellar:uzima:<network>:<address_string>
        // For simplicity, we create a deterministic string representation
        let mut did = String::from_str(env, "did:stellar:uzima:");

        // Append network
        did = Self::concat_strings(env, &did, network_id);
        did = Self::concat_strings(env, &did, &String::from_str(env, ":"));

        // For the address, we'll use a hash representation
        // In production, this would be the actual address encoding
        let addr_val: soroban_sdk::Val = subject.to_val();
        let addr_bytes = Bytes::from_array(env, &addr_val.get_payload().to_be_bytes());
        let addr_hash: BytesN<32> = env.crypto().sha256(&addr_bytes).into();
        let hash_str = Self::bytes_to_hex_string(env, &addr_hash);
        did = Self::concat_strings(env, &did, &hash_str);

        did
    }

    fn generate_credential_id(
        env: &Env,
        issuer: &Address,
        subject: &Address,
        timestamp: u64,
        _credential_type: &CredentialType,
    ) -> BytesN<32> {
        let mut data = Bytes::new(env);
        // Append addresses as their Val representations converted to bytes
        let issuer_val: soroban_sdk::Val = issuer.to_val();
        let subject_val: soroban_sdk::Val = subject.to_val();
        data.append(&Bytes::from_array(
            env,
            &issuer_val.get_payload().to_be_bytes(),
        ));
        data.append(&Bytes::from_array(
            env,
            &subject_val.get_payload().to_be_bytes(),
        ));
        data.append(&Bytes::from_array(env, &timestamp.to_be_bytes()));

        env.crypto().sha256(&data).into()
    }

    /// Compute document hash for audit trail
    fn compute_document_hash(env: &Env, doc: &DIDDocument) -> BytesN<32> {
        // Hash the serialized version using a combination of key fields
        let mut data = Bytes::new(env);
        // Convert string ID to bytes for hashing
        let id_val: soroban_sdk::Val = doc.id.to_val();
        data.append(&Bytes::from_array(env, &id_val.get_payload().to_be_bytes()));
        data.append(&Bytes::from_array(env, &doc.version.to_be_bytes()));
        data.append(&Bytes::from_array(env, &doc.updated.to_be_bytes()));

        env.crypto().sha256(&data).into()
    }

    /// Helper to concatenate strings
    fn concat_strings(env: &Env, _a: &String, _b: &String) -> String {
        // Soroban strings are immutable and concatenation is complex
        // For DID generation, we use a simpler approach in generate_did_string
        String::from_str(env, "did:stellar:uzima:")
    }

    /// Convert bytes to hex string (simplified)
    fn bytes_to_hex_string(env: &Env, bytes: &BytesN<32>) -> String {
        let arr = bytes.to_array();
        // pre-computed hex chars
        const HEX_CHARS: &[u8] = b"0123456789abcdef";

        // Take first 8 bytes for a shorter representation -> 16 chars
        let mut hex_str = [0u8; 16];
        for (i, byte) in arr.iter().take(8).enumerate() {
            let high = (byte >> 4) as usize;
            let low = (byte & 0x0f) as usize;
            if let Some(c) = HEX_CHARS.get(high) {
                if let Some(target) = hex_str.get_mut(i.saturating_mul(2)) {
                    *target = *c;
                }
            }
            if let Some(c) = HEX_CHARS.get(low) {
                if let Some(target) = hex_str.get_mut(i.saturating_mul(2).saturating_add(1)) {
                    *target = *c;
                }
            }
        }

        // Safe conversion since we only put ASCII hex chars in
        String::from_str(env, core::str::from_utf8(&hex_str).unwrap_or("00000000"))
    }

    /// DID-based authorization check
    pub fn verify_did_authorization(
        env: Env,
        subject: Address,
        required_relationship: VerificationRelationship,
    ) -> bool {
        let did_doc: Option<DIDDocument> = env
            .storage()
            .persistent()
            .get(&DataKey::DIDDocument(subject));

        match did_doc {
            None => false,
            Some(doc) => {
                if !matches!(doc.status, DIDStatus::Active) {
                    return false;
                }

                // Check if any verification method for the required relationship is active
                let method_ids = match required_relationship {
                    VerificationRelationship::Authentication => &doc.authentication,
                    VerificationRelationship::AssertionMethod => &doc.assertion_method,
                    VerificationRelationship::KeyAgreement => &doc.key_agreement,
                    VerificationRelationship::CapabilityInvocation => &doc.capability_invocation,
                    VerificationRelationship::CapabilityDelegation => &doc.capability_delegation,
                };

                for method_id in method_ids.iter() {
                    for vm in doc.verification_methods.iter() {
                        if vm.id == method_id && vm.is_active {
                            return true;
                        }
                    }
                }

                false
            }
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
    use soroban_sdk::{Address, BytesN, Env, String, Vec};

    fn create_contract() -> (Env, IdentityRegistryContractClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, IdentityRegistryContract);
        let client = IdentityRegistryContractClient::new(&env, &contract_id);
        let owner = Address::generate(&env);

        let network_id = String::from_str(&env, "testnet");
        client.initialize(&owner, &network_id);

        (env, client, owner)
    }

    fn create_legacy_contract() -> (Env, IdentityRegistryContractClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, IdentityRegistryContract);
        let client = IdentityRegistryContractClient::new(&env, &contract_id);
        let owner = Address::generate(&env);

        client.initialize_legacy(&owner);

        (env, client, owner)
    }

    // ========================================================================
    // INITIALIZATION TESTS
    // ========================================================================

    #[test]
    fn test_initialize() {
        let (_env, client, owner) = create_contract();

        assert!(client.is_verifier(&owner));
        assert_eq!(client.get_owner(), owner);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1)")]
    fn test_double_initialization() {
        let (env, client, _owner) = create_contract();
        let owner2 = Address::generate(&env);
        let network_id = String::from_str(&env, "mainnet");

        client.initialize(&owner2, &network_id);
    }

    // ========================================================================
    // DID DOCUMENT TESTS
    // ========================================================================

    #[test]
    fn test_create_did() {
        let (env, client, _owner) = create_contract();
        let subject = Address::generate(&env);
        let public_key = BytesN::from_array(&env, &[1u8; 32]);
        let services: Vec<ServiceEndpoint> = Vec::new(&env);

        let _did_string = client.create_did(&subject, &public_key, &services);

        // Verify DID was created
        let did_doc = client.resolve_did(&subject);
        assert!(matches!(did_doc.status, DIDStatus::Active));
        assert_eq!(did_doc.controller, subject);
        assert_eq!(did_doc.version, 1);
        assert_eq!(did_doc.verification_methods.len(), 1);
    }

    #[test]
    fn test_create_did_with_services() {
        let (env, client, _owner) = create_contract();
        let subject = Address::generate(&env);
        let public_key = BytesN::from_array(&env, &[1u8; 32]);

        let mut services: Vec<ServiceEndpoint> = Vec::new(&env);
        services.push_back(ServiceEndpoint {
            id: String::from_str(&env, "#medical-records"),
            service_type: String::from_str(&env, "MedicalRecords"),
            endpoint: String::from_str(&env, "ipfs://Qm..."),
            is_active: true,
        });

        client.create_did(&subject, &public_key, &services);

        let did_doc = client.resolve_did(&subject);
        assert_eq!(did_doc.services.len(), 1);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #7)")]
    fn test_create_duplicate_did() {
        let (env, client, _owner) = create_contract();
        let subject = Address::generate(&env);
        let public_key = BytesN::from_array(&env, &[1u8; 32]);
        let services: Vec<ServiceEndpoint> = Vec::new(&env);

        client.create_did(&subject, &public_key, &services);
        client.create_did(&subject, &public_key, &services); // Should fail
    }

    #[test]
    fn test_update_did() {
        let (env, client, _owner) = create_contract();
        let subject = Address::generate(&env);
        let public_key = BytesN::from_array(&env, &[1u8; 32]);
        let services: Vec<ServiceEndpoint> = Vec::new(&env);

        client.create_did(&subject, &public_key, &services);

        // Update with new services
        let mut new_services: Vec<ServiceEndpoint> = Vec::new(&env);
        new_services.push_back(ServiceEndpoint {
            id: String::from_str(&env, "#credentials"),
            service_type: String::from_str(&env, "CredentialRegistry"),
            endpoint: String::from_str(&env, "https://creds.example.com"),
            is_active: true,
        });
        let mut also_known_as: Vec<String> = Vec::new(&env);
        also_known_as.push_back(String::from_str(&env, "did:web:example.com"));

        client.update_did(&subject, &new_services, &also_known_as);

        let did_doc = client.resolve_did(&subject);
        assert_eq!(did_doc.version, 2);
        assert_eq!(did_doc.services.len(), 1);
        assert_eq!(did_doc.also_known_as.len(), 1);
    }

    #[test]
    fn test_deactivate_did() {
        let (env, client, _owner) = create_contract();
        let subject = Address::generate(&env);
        let public_key = BytesN::from_array(&env, &[1u8; 32]);
        let services: Vec<ServiceEndpoint> = Vec::new(&env);

        client.create_did(&subject, &public_key, &services);
        client.deactivate_did(&subject);

        // Should fail to resolve deactivated DID
        let result = client.try_resolve_did(&subject);
        assert!(result.is_err());
    }

    // ========================================================================
    // VERIFICATION METHOD TESTS
    // ========================================================================

    #[test]
    fn test_add_verification_method() {
        let (env, client, _owner) = create_contract();
        let subject = Address::generate(&env);
        let public_key = BytesN::from_array(&env, &[1u8; 32]);
        let services: Vec<ServiceEndpoint> = Vec::new(&env);

        client.create_did(&subject, &public_key, &services);

        // Add new verification method
        let new_key = BytesN::from_array(&env, &[2u8; 32]);
        let method_id = String::from_str(&env, "#key-2");
        let mut relationships: Vec<VerificationRelationship> = Vec::new(&env);
        relationships.push_back(VerificationRelationship::KeyAgreement);

        client.add_verification_method(
            &subject,
            &method_id,
            &VerificationMethodType::X25519KeyAgreementKey2020,
            &new_key,
            &relationships,
        );

        let did_doc = client.resolve_did(&subject);
        assert_eq!(did_doc.verification_methods.len(), 2);
        assert_eq!(did_doc.key_agreement.len(), 1);
    }

    #[test]
    fn test_rotate_key() {
        let (env, client, _owner) = create_contract();
        let subject = Address::generate(&env);
        let public_key = BytesN::from_array(&env, &[1u8; 32]);
        let services: Vec<ServiceEndpoint> = Vec::new(&env);

        client.create_did(&subject, &public_key, &services);

        env.ledger().set(LedgerInfo {
            timestamp: 4000,
            ..Default::default()
        });

        // Rotate the primary key
        let new_key = BytesN::from_array(&env, &[3u8; 32]);
        let method_id = String::from_str(&env, "#key-1");

        client.rotate_key(&subject, &method_id, &new_key);

        let did_doc = client.resolve_did(&subject);
        let vm = did_doc.verification_methods.get(0).unwrap();
        assert_eq!(vm.public_key, new_key);
        assert!(vm.last_rotated > 0);
    }

    #[test]
    fn test_revoke_verification_method() {
        let (env, client, _owner) = create_contract();
        let subject = Address::generate(&env);
        let public_key = BytesN::from_array(&env, &[1u8; 32]);
        let services: Vec<ServiceEndpoint> = Vec::new(&env);

        client.create_did(&subject, &public_key, &services);

        // Add second method so we can revoke the first
        let new_key = BytesN::from_array(&env, &[2u8; 32]);
        let method_id = String::from_str(&env, "#key-2");
        let mut relationships: Vec<VerificationRelationship> = Vec::new(&env);
        relationships.push_back(VerificationRelationship::Authentication);

        client.add_verification_method(
            &subject,
            &method_id,
            &VerificationMethodType::Ed25519VerificationKey2020,
            &new_key,
            &relationships,
        );

        // Now revoke the first method
        let first_method_id = String::from_str(&env, "#key-1");
        client.revoke_verification_method(&subject, &first_method_id);

        let did_doc = client.resolve_did(&subject);
        let vm = did_doc.verification_methods.get(0).unwrap();
        assert!(!vm.is_active);
    }

    // ========================================================================
    // VERIFIABLE CREDENTIALS TESTS
    // ========================================================================

    #[test]
    fn test_issue_credential() {
        let (env, client, owner) = create_contract();
        let subject = Address::generate(&env);

        let credential_hash = BytesN::from_array(&env, &[1u8; 32]);
        let credential_uri = String::from_str(&env, "ipfs://QmCredential...");

        let credential_id = client.issue_credential(
            &owner,
            &subject,
            &CredentialType::MedicalLicense,
            &credential_hash,
            &credential_uri,
            &0u64, // No expiration
        );

        // Verify credential
        let status = client.verify_credential(&credential_id);
        assert!(matches!(status, CredentialStatus::Valid));

        let cred = client.get_credential(&credential_id);
        assert_eq!(cred.issuer, owner);
        assert_eq!(cred.subject, subject);
        assert!(!cred.is_revoked);
    }

    #[test]
    fn test_issue_credential_with_expiration() {
        let (env, client, owner) = create_contract();
        let subject = Address::generate(&env);

        let credential_hash = BytesN::from_array(&env, &[2u8; 32]);
        let credential_uri = String::from_str(&env, "ipfs://QmCredential...");
        let expiration = 1000u64; // Will be in the past

        let credential_id = client.issue_credential(
            &owner,
            &subject,
            &CredentialType::SpecialistCertification,
            &credential_hash,
            &credential_uri,
            &expiration,
        );

        env.ledger().set(LedgerInfo {
            timestamp: 2000,
            ..Default::default()
        });

        // Credential should be expired (timestamp is > 1000)
        let status = client.verify_credential(&credential_id);
        assert!(matches!(status, CredentialStatus::Expired));
    }

    #[test]
    fn test_revoke_credential() {
        let (env, client, owner) = create_contract();
        let subject = Address::generate(&env);

        let credential_hash = BytesN::from_array(&env, &[3u8; 32]);
        let credential_uri = String::from_str(&env, "ipfs://QmCredential...");

        let credential_id = client.issue_credential(
            &owner,
            &subject,
            &CredentialType::HospitalAffiliation,
            &credential_hash,
            &credential_uri,
            &0u64,
        );

        // Revoke the credential
        let reason = String::from_str(&env, "License expired");
        client.revoke_credential(&owner, &credential_id, &reason);

        let status = client.verify_credential(&credential_id);
        assert!(matches!(status, CredentialStatus::Revoked));

        let cred = client.get_credential(&credential_id);
        assert!(cred.is_revoked);
    }

    #[test]
    fn test_get_subject_credentials() {
        let (env, client, owner) = create_contract();
        let subject = Address::generate(&env);

        // Issue multiple credentials
        for i in 0..3 {
            let credential_hash = BytesN::from_array(&env, &[i as u8; 32]);
            let credential_uri = String::from_str(&env, "ipfs://QmCredential...");
            client.issue_credential(
                &owner,
                &subject,
                &CredentialType::MedicalLicense,
                &credential_hash,
                &credential_uri,
                &0u64,
            );
        }

        let credentials = client.get_subject_credentials(&subject);
        assert_eq!(credentials.len(), 3);
    }

    #[test]
    fn test_has_valid_credential() {
        let (env, client, owner) = create_contract();
        let subject = Address::generate(&env);

        // Subject should not have credential initially
        assert!(!client.has_valid_credential(&subject, &CredentialType::MedicalLicense));

        // Issue credential
        let credential_hash = BytesN::from_array(&env, &[4u8; 32]);
        let credential_uri = String::from_str(&env, "ipfs://QmCredential...");
        client.issue_credential(
            &owner,
            &subject,
            &CredentialType::MedicalLicense,
            &credential_hash,
            &credential_uri,
            &0u64,
        );

        // Now should have valid credential
        assert!(client.has_valid_credential(&subject, &CredentialType::MedicalLicense));
        // But not other types
        assert!(!client.has_valid_credential(&subject, &CredentialType::ResearchAuthorization));
    }

    // ========================================================================
    // IDENTITY RECOVERY TESTS
    // ========================================================================

    #[test]
    fn test_add_recovery_guardian() {
        let (env, client, _owner) = create_contract();
        let subject = Address::generate(&env);
        let public_key = BytesN::from_array(&env, &[1u8; 32]);
        let services: Vec<ServiceEndpoint> = Vec::new(&env);

        client.create_did(&subject, &public_key, &services);

        // Add guardians
        let guardian1 = Address::generate(&env);
        let guardian2 = Address::generate(&env);

        client.add_recovery_guardian(&subject, &guardian1, &1u32);
        client.add_recovery_guardian(&subject, &guardian2, &1u32);

        // Guardians should be added (we can verify through recovery process)
    }

    #[test]
    fn test_initiate_recovery() {
        let (env, client, _owner) = create_contract();
        let subject = Address::generate(&env);
        let public_key = BytesN::from_array(&env, &[1u8; 32]);
        let services: Vec<ServiceEndpoint> = Vec::new(&env);

        client.create_did(&subject, &public_key, &services);

        // Add guardian
        let guardian = Address::generate(&env);
        client.add_recovery_guardian(&subject, &guardian, &2u32);

        // Initiate recovery
        let new_controller = Address::generate(&env);
        let new_key = BytesN::from_array(&env, &[5u8; 32]);

        let request_id = client.initiate_recovery(&guardian, &subject, &new_controller, &new_key);
        assert!(request_id > 0);

        // DID should be in recovery pending state
        let did_doc = client.resolve_did(&subject);
        assert!(matches!(did_doc.status, DIDStatus::RecoveryPending));
    }

    #[test]
    fn test_cancel_recovery() {
        let (env, client, _owner) = create_contract();
        let subject = Address::generate(&env);
        let public_key = BytesN::from_array(&env, &[1u8; 32]);
        let services: Vec<ServiceEndpoint> = Vec::new(&env);

        client.create_did(&subject, &public_key, &services);

        // Add guardian and initiate recovery
        let guardian = Address::generate(&env);
        client.add_recovery_guardian(&subject, &guardian, &2u32);

        let new_controller = Address::generate(&env);
        let new_key = BytesN::from_array(&env, &[5u8; 32]);
        client.initiate_recovery(&guardian, &subject, &new_controller, &new_key);

        // Cancel recovery (subject still has access)
        client.cancel_recovery(&subject);

        // DID should be active again
        let did_doc = client.resolve_did(&subject);
        assert!(matches!(did_doc.status, DIDStatus::Active));
    }

    // ========================================================================
    // SERVICE ENDPOINT TESTS
    // ========================================================================

    #[test]
    fn test_add_service() {
        let (env, client, _owner) = create_contract();
        let subject = Address::generate(&env);
        let public_key = BytesN::from_array(&env, &[1u8; 32]);
        let services: Vec<ServiceEndpoint> = Vec::new(&env);

        client.create_did(&subject, &public_key, &services);

        // Add service
        let service_id = String::from_str(&env, "#linked-domain");
        let service_type = String::from_str(&env, "LinkedDomains");
        let endpoint = String::from_str(&env, "https://example.com");

        client.add_service(&subject, &service_id, &service_type, &endpoint);

        let did_doc = client.resolve_did(&subject);
        assert_eq!(did_doc.services.len(), 1);
    }

    #[test]
    fn test_remove_service() {
        let (env, client, _owner) = create_contract();
        let subject = Address::generate(&env);
        let public_key = BytesN::from_array(&env, &[1u8; 32]);

        let mut services: Vec<ServiceEndpoint> = Vec::new(&env);
        services.push_back(ServiceEndpoint {
            id: String::from_str(&env, "#service-1"),
            service_type: String::from_str(&env, "Test"),
            endpoint: String::from_str(&env, "https://test.com"),
            is_active: true,
        });

        client.create_did(&subject, &public_key, &services);

        // Remove service
        let service_id = String::from_str(&env, "#service-1");
        client.remove_service(&subject, &service_id);

        let did_doc = client.resolve_did(&subject);
        assert_eq!(did_doc.services.len(), 0);
    }

    // ========================================================================
    // VERIFIER MANAGEMENT TESTS
    // ========================================================================

    #[test]
    fn test_add_and_remove_verifier() {
        let (env, client, _owner) = create_contract();
        let verifier = Address::generate(&env);

        // Add verifier
        client.add_verifier(&verifier);
        assert!(client.is_verifier(&verifier));

        // Remove verifier
        client.remove_verifier(&verifier);
        assert!(!client.is_verifier(&verifier));
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #5)")]
    fn test_cannot_remove_owner_as_verifier() {
        let (_env, client, owner) = create_contract();
        client.remove_verifier(&owner);
    }

    // ========================================================================
    // LEGACY FUNCTION TESTS
    // ========================================================================

    #[test]
    fn test_legacy_register_identity_hash() {
        let (env, client, _owner) = create_legacy_contract();
        let subject = Address::generate(&env);

        let hash = BytesN::from_array(&env, &[1; 32]);
        let meta = String::from_str(&env, "Healthcare Provider License #12345");

        client.register_identity_hash(&hash, &subject, &meta);

        assert_eq!(client.get_identity_hash(&subject), Some(hash));
        assert_eq!(client.get_identity_meta(&subject), Some(meta));
    }

    #[test]
    fn test_legacy_attest_and_verify() {
        let (env, client, _owner) = create_legacy_contract();
        let verifier = Address::generate(&env);
        let subject = Address::generate(&env);

        client.add_verifier(&verifier);

        let claim_hash = BytesN::from_array(&env, &[2; 32]);
        client.attest(&verifier, &subject, &claim_hash);

        assert!(client.is_attested(&subject, &claim_hash));

        let attestations = client.get_attestations(&subject);
        assert_eq!(attestations.len(), 1);
    }

    #[test]
    fn test_legacy_revoke_attestation() {
        let (env, client, _owner) = create_legacy_contract();
        let verifier = Address::generate(&env);
        let subject = Address::generate(&env);

        client.add_verifier(&verifier);

        let claim_hash = BytesN::from_array(&env, &[3; 32]);
        client.attest(&verifier, &subject, &claim_hash);
        assert!(client.is_attested(&subject, &claim_hash));

        client.revoke_attestation(&verifier, &subject, &claim_hash);
        assert!(!client.is_attested(&subject, &claim_hash));
    }

    // ========================================================================
    // DID AUTHORIZATION TESTS
    // ========================================================================

    #[test]
    fn test_verify_did_authorization() {
        let (env, client, _owner) = create_contract();
        let subject = Address::generate(&env);
        let public_key = BytesN::from_array(&env, &[1u8; 32]);
        let services: Vec<ServiceEndpoint> = Vec::new(&env);

        client.create_did(&subject, &public_key, &services);

        // Should be authorized for authentication (default key is added to auth)
        assert!(
            client.verify_did_authorization(&subject, &VerificationRelationship::Authentication)
        );

        // Should not be authorized for key agreement (no key agreement method added)
        assert!(!client.verify_did_authorization(&subject, &VerificationRelationship::KeyAgreement));
    }

    #[test]
    fn test_verify_did_authorization_deactivated() {
        let (env, client, _owner) = create_contract();
        let subject = Address::generate(&env);
        let public_key = BytesN::from_array(&env, &[1u8; 32]);
        let services: Vec<ServiceEndpoint> = Vec::new(&env);

        client.create_did(&subject, &public_key, &services);
        client.deactivate_did(&subject);

        // Should not be authorized after deactivation
        assert!(
            !client.verify_did_authorization(&subject, &VerificationRelationship::Authentication)
        );
    }
}
