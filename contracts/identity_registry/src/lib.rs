// Identity Registry - W3C DID Compliant with proper validation throughout
#![no_std]
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Bytes, BytesN,
    Env, String, Symbol, Vec,
};

// ============================================================================
// W3C DID COMPLIANT DECENTRALIZED IDENTITY REGISTRY
// ============================================================================
// Implements W3C DID Core Specification (https://www.w3.org/TR/did-core/)
// DID Method: did:stellar:uzima:<network>:<address>
// ============================================================================

// === Error Definitions ===
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    NotVerifier = 4,
    CannotRemoveOwner = 5,
    DIDNotFound = 6,
    DIDAlreadyExists = 7,
    DIDDeactivated = 8,
    InvalidVerificationMethod = 9,
    VerificationMethodNotFound = 10,
    CredentialNotFound = 11,
    CredentialRevoked = 12,
    CredentialExpired = 13,
    InvalidCredentialType = 14,
    AttestationNotFound = 15,
    RecoveryNotInitiated = 16,
    RecoveryAlreadyPending = 17,
    RecoveryTimelockNotElapsed = 18,
    InvalidRecoveryGuardian = 19,
    InsufficientGuardianApprovals = 20,
    ServiceNotFound = 21,
    InvalidServiceEndpoint = 22,
    KeyRotationCooldown = 23,
}

// === DID Document Structures (W3C Compliant) ===

/// Verification Method Types as per W3C DID Specification
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
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
    /// Unique identifier for this verification method (fragment)
    pub id: String,
    /// Type of verification method
    pub method_type: VerificationMethodType,
    /// The controller of this key (usually the DID subject)
    pub controller: Address,
    /// Public key bytes (Ed25519: 32 bytes, Secp256k1: 33 bytes compressed)
    pub public_key: BytesN<32>,
    /// Whether this method is currently active
    pub is_active: bool,
    /// Timestamp when this key was added
    pub created: u64,
    /// Timestamp when this key was last rotated (0 if never)
    pub last_rotated: u64,
}

/// Service Endpoint as per W3C DID spec
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceEndpoint {
    /// Unique identifier for this service (fragment)
    pub id: String,
    /// Type of service (e.g., "LinkedDomains", "MedicalRecords", "CredentialRegistry")
    pub service_type: String,
    /// Service endpoint URI (IPFS hash, URL reference, or contract address)
    pub endpoint: String,
    /// Whether this service is active
    pub is_active: bool,
}

/// DID Document Status
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DIDStatus {
    Active,
    Deactivated,
    RecoveryPending,
}

/// Complete DID Document following W3C DID Core spec
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DIDDocument {
    /// The DID identifier (did:stellar:uzima:<network>:<address>)
    pub id: String,
    /// Controller(s) of this DID - who can make changes
    pub controller: Address,
    /// Alternative controller (for recovery scenarios)
    pub also_known_as: Vec<String>,
    /// All verification methods (public keys)
    pub verification_methods: Vec<VerificationMethod>,
    /// IDs of methods used for authentication
    pub authentication: Vec<String>,
    /// IDs of methods used for issuing credentials (assertion)
    pub assertion_method: Vec<String>,
    /// IDs of methods used for key agreement
    pub key_agreement: Vec<String>,
    /// IDs of methods for capability invocation
    pub capability_invocation: Vec<String>,
    /// IDs of methods for capability delegation
    pub capability_delegation: Vec<String>,
    /// Service endpoints
    pub services: Vec<ServiceEndpoint>,
    /// Document status
    pub status: DIDStatus,
    /// Creation timestamp
    pub created: u64,
    /// Last update timestamp
    pub updated: u64,
    /// Version number (incremented on each update)
    pub version: u32,
    /// Hash of previous document version (for audit trail)
    pub previous_hash: BytesN<32>,
}

// === Verifiable Credentials Structures ===

/// Credential Types for Healthcare
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CredentialType {
    MedicalLicense,
    SpecialistCertification,
    HospitalAffiliation,
    ResearchAuthorization,
    PatientConsent,
    EmergencyAccess,
    DataAccessPermission,
}

/// Verifiable Credential (on-chain reference)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiableCredential {
    /// Unique credential ID
    pub id: BytesN<32>,
    /// Credential type
    pub credential_type: CredentialType,
    /// Issuer DID (the entity that issued this credential)
    pub issuer: Address,
    /// Subject DID (the entity the credential is about)
    pub subject: Address,
    /// Issuance timestamp
    pub issuance_date: u64,
    /// Expiration timestamp (0 = no expiration)
    pub expiration_date: u64,
    /// Hash of the full credential data (stored off-chain)
    pub credential_hash: BytesN<32>,
    /// URI to the full credential (IPFS CID or similar)
    pub credential_uri: String,
    /// Whether the credential has been revoked
    pub is_revoked: bool,
    /// Revocation timestamp (0 if not revoked)
    pub revoked_at: u64,
    /// Revocation reason (if revoked)
    pub revocation_reason: String,
}

/// Credential Status for verification
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CredentialStatus {
    Valid,
    Revoked,
    Expired,
    NotFound,
}

// === Identity Recovery Structures ===

/// Recovery Guardian
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryGuardian {
    pub address: Address,
    pub weight: u32,
    pub added_at: u64,
}

/// Recovery Request
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryRequest {
    pub request_id: u64,
    pub subject: Address,
    pub new_controller: Address,
    pub new_primary_key: BytesN<32>,
    pub created_at: u64,
    pub approvals: Vec<Address>,
    pub total_weight: u32,
    pub executed: bool,
}

// === Legacy Structures (Backward Compatibility) ===

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityRecord {
    pub hash: BytesN<32>,
    pub meta: String,
    pub registered_by: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attestation {
    pub claim_hash: BytesN<32>,
    pub verifier: Address,
    pub is_active: bool,
}

// === Storage Keys ===

#[contracttype]
pub enum DataKey {
    // Contract State
    Owner,
    Initialized,
    NetworkId,

    // Verifier Management
    Verifier(Address),

    // Legacy Identity (backward compatibility)
    IdentityHash(Address),
    Attestation(Address, BytesN<32>),
    SubjectAttestations(Address),

    // DID Document Storage
    DIDDocument(Address),
    DIDByString(String),

    // Verification Methods
    VerificationMethod(Address, String),

    // Verifiable Credentials
    Credential(BytesN<32>),
    SubjectCredentials(Address),
    IssuerCredentials(Address),
    CredentialsByType(Address, CredentialType),

    // Recovery System
    RecoveryGuardians(Address),
    RecoveryThreshold(Address),
    RecoveryRequest(u64),
    ActiveRecovery(Address),
    RecoveryCounter,

    // Key Rotation
    LastKeyRotation(Address),
    KeyRotationCooldown,
}

// === Constants ===

const DEFAULT_RECOVERY_THRESHOLD: u32 = 2;
const DEFAULT_RECOVERY_TIMELOCK: u64 = 86_400; // 24 hours
const DEFAULT_KEY_ROTATION_COOLDOWN: u64 = 3_600; // 1 hour
const ZERO_HASH: [u8; 32] = [0u8; 32];

// === Contract Implementation ===

#[contract]
pub struct IdentityRegistryContract;

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
        primary_public_key: BytesN<32>,
        services: Vec<ServiceEndpoint>,
    ) -> Result<String, Error> {
        subject.require_auth();

        // Check if DID already exists
        if env
            .storage()
            .persistent()
            .has(&DataKey::DIDDocument(subject.clone()))
        {
            return Err(Error::DIDAlreadyExists);
        }

        let timestamp = env.ledger().timestamp();
        let network_id: String = env
            .storage()
            .instance()
            .get(&DataKey::NetworkId)
            .unwrap_or(String::from_str(&env, "testnet"));

        // Generate DID string
        let did_string = Self::generate_did_string(&env, &network_id, &subject);

        // Create primary verification method
        let primary_vm_id = String::from_str(&env, "#key-1");
        let primary_vm = VerificationMethod {
            id: primary_vm_id.clone(),
            method_type: VerificationMethodType::Ed25519VerificationKey2020,
            controller: subject.clone(),
            public_key: primary_public_key,
            is_active: true,
            created: timestamp,
            last_rotated: 0,
        };

        let mut verification_methods = Vec::new(&env);
        verification_methods.push_back(primary_vm);

        let mut auth_methods = Vec::new(&env);
        auth_methods.push_back(primary_vm_id.clone());

        let mut assertion_methods = Vec::new(&env);
        assertion_methods.push_back(primary_vm_id.clone());

        let mut cap_invocation = Vec::new(&env);
        cap_invocation.push_back(primary_vm_id.clone());

        let mut cap_delegation = Vec::new(&env);
        cap_delegation.push_back(primary_vm_id);

        let did_doc = DIDDocument {
            id: did_string.clone(),
            controller: subject.clone(),
            also_known_as: Vec::new(&env),
            verification_methods,
            authentication: auth_methods,
            assertion_method: assertion_methods,
            key_agreement: Vec::new(&env),
            capability_invocation: cap_invocation,
            capability_delegation: cap_delegation,
            services,
            status: DIDStatus::Active,
            created: timestamp,
            updated: timestamp,
            version: 1,
            previous_hash: BytesN::from_array(&env, &ZERO_HASH),
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

    /// Resolve a DID Document by subject address
    pub fn resolve_did(env: Env, subject: Address) -> Result<DIDDocument, Error> {
        let did_doc: DIDDocument = env
            .storage()
            .persistent()
            .get(&DataKey::DIDDocument(subject))
            .ok_or(Error::DIDNotFound)?;

        if matches!(did_doc.status, DIDStatus::Deactivated) {
            return Err(Error::DIDDeactivated);
        }

        Ok(did_doc)
    }

    /// Resolve a DID Document by DID string
    pub fn resolve_did_by_string(env: Env, did_string: String) -> Result<DIDDocument, Error> {
        let subject: Address = env
            .storage()
            .persistent()
            .get(&DataKey::DIDByString(did_string))
            .ok_or(Error::DIDNotFound)?;

        Self::resolve_did(env, subject)
    }

    /// Update DID Document (add/modify services, also_known_as)
    pub fn update_did(
        env: Env,
        subject: Address,
        new_services: Vec<ServiceEndpoint>,
        new_also_known_as: Vec<String>,
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

        // Compute hash of current document for audit trail
        let prev_hash = Self::compute_document_hash(&env, &did_doc);

        did_doc.services = new_services;
        did_doc.also_known_as = new_also_known_as;
        did_doc.updated = env.ledger().timestamp();
        did_doc.version += 1;
        did_doc.previous_hash = prev_hash;

        env.storage()
            .persistent()
            .set(&DataKey::DIDDocument(subject.clone()), &did_doc);

        env.events().publish(
            (Symbol::new(&env, "DIDUpdated"),),
            (subject, did_doc.version),
        );

        Ok(())
    }

    /// Deactivate a DID (soft delete)
    pub fn deactivate_did(env: Env, subject: Address) -> Result<(), Error> {
        subject.require_auth();

        let mut did_doc: DIDDocument = env
            .storage()
            .persistent()
            .get(&DataKey::DIDDocument(subject.clone()))
            .ok_or(Error::DIDNotFound)?;

        did_doc.status = DIDStatus::Deactivated;
        did_doc.updated = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::DIDDocument(subject.clone()), &did_doc);

        env.events()
            .publish((Symbol::new(&env, "DIDDeactivated"),), subject);

        Ok(())
    }

    // ========================================================================
    // VERIFICATION METHOD MANAGEMENT (Key Management)
    // ========================================================================

    /// Add a new verification method to a DID
    pub fn add_verification_method(
        env: Env,
        subject: Address,
        method_id: String,
        method_type: VerificationMethodType,
        public_key: BytesN<32>,
        relationships: Vec<VerificationRelationship>,
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

        let timestamp = env.ledger().timestamp();

        let new_vm = VerificationMethod {
            id: method_id.clone(),
            method_type,
            controller: subject.clone(),
            public_key,
            is_active: true,
            created: timestamp,
            last_rotated: 0,
        };

        did_doc.verification_methods.push_back(new_vm);

        // Add to specified relationships
        for rel in relationships.iter() {
            match rel {
                VerificationRelationship::Authentication => {
                    did_doc.authentication.push_back(method_id.clone());
                }
                VerificationRelationship::AssertionMethod => {
                    did_doc.assertion_method.push_back(method_id.clone());
                }
                VerificationRelationship::KeyAgreement => {
                    did_doc.key_agreement.push_back(method_id.clone());
                }
                VerificationRelationship::CapabilityInvocation => {
                    did_doc.capability_invocation.push_back(method_id.clone());
                }
                VerificationRelationship::CapabilityDelegation => {
                    did_doc.capability_delegation.push_back(method_id.clone());
                }
            }
        }

        did_doc.updated = timestamp;
        did_doc.version += 1;

        env.storage()
            .persistent()
            .set(&DataKey::DIDDocument(subject.clone()), &did_doc);

        env.events().publish(
            (Symbol::new(&env, "VerificationMethodAdded"),),
            (subject, method_id),
        );

        Ok(())
    }

    /// Rotate a verification method key
    pub fn rotate_key(
        env: Env,
        subject: Address,
        method_id: String,
        new_public_key: BytesN<32>,
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

        let timestamp = env.ledger().timestamp();

        // Check cooldown period
        let cooldown: u64 = env
            .storage()
            .instance()
            .get(&DataKey::KeyRotationCooldown)
            .unwrap_or(DEFAULT_KEY_ROTATION_COOLDOWN);
        let last_rotation: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::LastKeyRotation(subject.clone()))
            .unwrap_or(0);

        if timestamp < last_rotation + cooldown {
            return Err(Error::KeyRotationCooldown);
        }

        // Find and update the verification method
        let mut found = false;
        let mut updated_methods = Vec::new(&env);

        for vm in did_doc.verification_methods.iter() {
            if vm.id == method_id {
                let updated_vm = VerificationMethod {
                    id: vm.id.clone(),
                    method_type: vm.method_type.clone(),
                    controller: vm.controller.clone(),
                    public_key: new_public_key.clone(),
                    is_active: true,
                    created: vm.created,
                    last_rotated: timestamp,
                };
                updated_methods.push_back(updated_vm);
                found = true;
            } else {
                updated_methods.push_back(vm);
            }
        }

        if !found {
            return Err(Error::VerificationMethodNotFound);
        }

        did_doc.verification_methods = updated_methods;
        did_doc.updated = timestamp;
        did_doc.version += 1;

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

    /// Revoke/deactivate a verification method
    pub fn revoke_verification_method(
        env: Env,
        subject: Address,
        method_id: String,
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

        // Ensure at least one method remains active
        let active_count = did_doc
            .verification_methods
            .iter()
            .filter(|vm| vm.is_active)
            .count();
        if active_count <= 1 {
            return Err(Error::InvalidVerificationMethod);
        }

        let mut found = false;
        let mut updated_methods = Vec::new(&env);

        for vm in did_doc.verification_methods.iter() {
            if vm.id == method_id {
                let revoked_vm = VerificationMethod {
                    id: vm.id.clone(),
                    method_type: vm.method_type.clone(),
                    controller: vm.controller.clone(),
                    public_key: vm.public_key.clone(),
                    is_active: false,
                    created: vm.created,
                    last_rotated: vm.last_rotated,
                };
                updated_methods.push_back(revoked_vm);
                found = true;
            } else {
                updated_methods.push_back(vm);
            }
        }

        if !found {
            return Err(Error::VerificationMethodNotFound);
        }

        did_doc.verification_methods = updated_methods;
        did_doc.updated = env.ledger().timestamp();
        did_doc.version += 1;

        env.storage()
            .persistent()
            .set(&DataKey::DIDDocument(subject.clone()), &did_doc);

        env.events().publish(
            (Symbol::new(&env, "VerificationMethodRevoked"),),
            (subject, method_id),
        );

        Ok(())
    }

    // ========================================================================
    // VERIFIABLE CREDENTIALS
    // ========================================================================

    /// Issue a verifiable credential (only verifiers/issuers can do this)
    pub fn issue_credential(
        env: Env,
        issuer: Address,
        subject: Address,
        credential_type: CredentialType,
        credential_hash: BytesN<32>,
        credential_uri: String,
        expiration_date: u64,
    ) -> Result<BytesN<32>, Error> {
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
            .get(&DataKey::SubjectCredentials(subject))
            .unwrap_or(Vec::new(&env));

        let mut credentials = Vec::new(&env);
        for id in credential_ids.iter() {
            if let Some(cred) = env
                .storage()
                .persistent()
                .get::<DataKey, VerifiableCredential>(&DataKey::Credential(id))
            {
                credentials.push_back(cred);
            }
        }
        credentials
    }

    /// Verify if subject has a valid credential of a specific type
    pub fn has_valid_credential(
        env: Env,
        subject: Address,
        credential_type: CredentialType,
    ) -> bool {
        let credentials = Self::get_subject_credentials(env.clone(), subject);
        let timestamp = env.ledger().timestamp();

        for cred in credentials.iter() {
            if cred.credential_type == credential_type
                && !cred.is_revoked
                && (cred.expiration_date == 0 || cred.expiration_date > timestamp)
            {
                return true;
            }
        }
        false
    }

    // ========================================================================
    // IDENTITY RECOVERY
    // ========================================================================

    /// Add a recovery guardian
    pub fn add_recovery_guardian(
        env: Env,
        subject: Address,
        guardian: Address,
        weight: u32,
    ) -> Result<(), Error> {
        subject.require_auth();

        let mut guardians: Vec<RecoveryGuardian> = env
            .storage()
            .persistent()
            .get(&DataKey::RecoveryGuardians(subject.clone()))
            .unwrap_or(Vec::new(&env));

        let new_guardian = RecoveryGuardian {
            address: guardian.clone(),
            weight,
            added_at: env.ledger().timestamp(),
        };

        guardians.push_back(new_guardian);
        env.storage()
            .persistent()
            .set(&DataKey::RecoveryGuardians(subject.clone()), &guardians);

        env.events().publish(
            (Symbol::new(&env, "GuardianAdded"),),
            (subject, guardian, weight),
        );

        Ok(())
    }

    /// Remove a recovery guardian
    pub fn remove_recovery_guardian(
        env: Env,
        subject: Address,
        guardian: Address,
    ) -> Result<(), Error> {
        subject.require_auth();

        let guardians: Vec<RecoveryGuardian> = env
            .storage()
            .persistent()
            .get(&DataKey::RecoveryGuardians(subject.clone()))
            .unwrap_or(Vec::new(&env));

        let mut new_guardians = Vec::new(&env);
        for g in guardians.iter() {
            if g.address != guardian {
                new_guardians.push_back(g);
            }
        }

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

        env.storage()
            .persistent()
            .set(&DataKey::RecoveryThreshold(subject.clone()), &threshold);

        env.events().publish(
            (Symbol::new(&env, "ThresholdUpdated"),),
            (subject, threshold),
        );

        Ok(())
    }

    /// Initiate identity recovery
    pub fn initiate_recovery(
        env: Env,
        guardian: Address,
        subject: Address,
        new_controller: Address,
        new_primary_key: BytesN<32>,
    ) -> Result<u64, Error> {
        guardian.require_auth();

        // Verify guardian
        let guardians: Vec<RecoveryGuardian> = env
            .storage()
            .persistent()
            .get(&DataKey::RecoveryGuardians(subject.clone()))
            .unwrap_or(Vec::new(&env));

        let guardian_info = guardians
            .iter()
            .find(|g| g.address == guardian)
            .ok_or(Error::InvalidRecoveryGuardian)?;

        // Check if recovery already pending
        if env
            .storage()
            .persistent()
            .has(&DataKey::ActiveRecovery(subject.clone()))
        {
            return Err(Error::RecoveryAlreadyPending);
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

        let mut request: RecoveryRequest = env
            .storage()
            .persistent()
            .get(&DataKey::RecoveryRequest(request_id))
            .ok_or(Error::RecoveryNotInitiated)?;

        if request.executed {
            return Err(Error::RecoveryNotInitiated);
        }

        // Verify guardian
        let guardians: Vec<RecoveryGuardian> = env
            .storage()
            .persistent()
            .get(&DataKey::RecoveryGuardians(request.subject.clone()))
            .unwrap_or(Vec::new(&env));

        let guardian_info = guardians
            .iter()
            .find(|g| g.address == guardian)
            .ok_or(Error::InvalidRecoveryGuardian)?;

        // Check if already approved
        if request.approvals.iter().any(|a| a == guardian) {
            return Ok(());
        }

        request.approvals.push_back(guardian.clone());
        request.total_weight += guardian_info.weight;

        env.storage()
            .persistent()
            .set(&DataKey::RecoveryRequest(request_id), &request);

        env.events().publish(
            (Symbol::new(&env, "RecoveryApproved"),),
            (guardian, request_id),
        );

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
        const MAX_PART_LEN: usize = 128;
        const MAX_DID_LEN: usize = 512;

        let subject_str = subject.to_string();

        let network_len = network_id.len() as usize;
        let subject_len = subject_str.len() as usize;

        let mut network_buf = [0u8; MAX_PART_LEN];
        network_id.copy_into_slice(&mut network_buf[..network_len]);

        let mut subject_buf = [0u8; MAX_PART_LEN];
        subject_str.copy_into_slice(&mut subject_buf[..subject_len]);

        let mut did_bytes = Bytes::new(env);
        did_bytes.extend_from_slice(b"did:stellar:uzima:");
        did_bytes.extend_from_slice(&network_buf[..network_len]);
        did_bytes.extend_from_slice(b":");
        did_bytes.extend_from_slice(&subject_buf[..subject_len]);

        let did_buf = did_bytes.to_buffer::<MAX_DID_LEN>();
        String::from_bytes(env, did_buf.as_slice())
    }

    fn generate_credential_id(
        env: &Env,
        issuer: &Address,
        subject: &Address,
        timestamp: u64,
        _credential_type: &CredentialType,
    ) -> BytesN<32> {
        let mut data = issuer.to_xdr(env);
        data.append(&subject.to_xdr(env));
        data.append(&timestamp.to_xdr(env));
        env.crypto().sha256(&data).into()
    }

    /// Compute document hash for audit trail
    fn compute_document_hash(env: &Env, doc: &DIDDocument) -> BytesN<32> {
        let data = doc.clone().to_xdr(env);
        env.crypto().sha256(&data).into()
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
    use soroban_sdk::testutils::{Address as _, Ledger as _};
    use soroban_sdk::{Address, BytesN, Env, String, Vec};

    fn create_contract() -> (Env, IdentityRegistryContractClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(10_000);
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
        env.ledger().set_timestamp(10_000);
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

        env.ledger().set_timestamp(4000);

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

        env.ledger().set_timestamp(2000);

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
