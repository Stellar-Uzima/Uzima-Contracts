#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Map, String, Vec, BytesN};

// Data structures
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

// Storage keys
#[contracttype]
pub enum DataKey {
    Owner,
    Verifier(Address),
    IdentityHash(Address),
    Attestation(Address, BytesN<32>),
    SubjectAttestations(Address),
}

#[contract]
pub struct IdentityRegistryContract;

#[contractimpl]
impl IdentityRegistryContract {
    /// Initialize the contract with an owner
    pub fn initialize(env: Env, owner: Address) {
        owner.require_auth();
        
        // Set the owner
        env.storage().instance().set(&DataKey::Owner, &owner);
        
        // Owner is automatically a verifier
        env.storage().instance().set(&DataKey::Verifier(owner.clone()), &true);
    }

    /// Register an identity hash with metadata
    pub fn register_identity_hash(
        env: Env,
        hash: BytesN<32>,
        subject: Address,
        meta: String,
    ) {
        let caller = env.current_contract_address();
        
        let identity_record = IdentityRecord {
            hash: hash.clone(),
            meta: meta.clone(),
            registered_by: caller,
        };
        
        env.storage().instance().set(&DataKey::IdentityHash(subject.clone()), &identity_record);
        
        // Emit event
        env.events().publish(
            ("IdentityRegistered",),
            (subject, hash, meta)
        );
    }

    /// Create an attestation (only verifiers can do this)
    pub fn attest(env: Env, subject: Address, claim_hash: BytesN<32>) {
        let verifier = env.current_contract_address();
        verifier.require_auth();
        
        // Check if caller is a verifier
        let is_verifier: bool = env.storage()
            .instance()
            .get(&DataKey::Verifier(verifier.clone()))
            .unwrap_or(false);
        
        if !is_verifier {
            panic!("Caller is not a verifier");
        }
        
        // Create attestation
        let attestation = Attestation {
            claim_hash: claim_hash.clone(),
            verifier: verifier.clone(),
            is_active: true,
        };
        
        env.storage().instance().set(
            &DataKey::Attestation(subject.clone(), claim_hash.clone()),
            &attestation
        );
        
        // Add to subject's attestation list
        let mut attestations: Vec<BytesN<32>> = env.storage()
            .instance()
            .get(&DataKey::SubjectAttestations(subject.clone()))
            .unwrap_or(Vec::new(&env));
        
        attestations.push_back(claim_hash.clone());
        env.storage().instance().set(&DataKey::SubjectAttestations(subject.clone()), &attestations);
        
        // Emit event
        env.events().publish(
            ("Attested",),
            (subject, verifier, claim_hash)
        );
    }

    /// Revoke an attestation (only verifiers can do this)
    pub fn revoke_attestation(env: Env, subject: Address, claim_hash: BytesN<32>) {
        let verifier = env.current_contract_address();
        verifier.require_auth();
        
        // Check if caller is a verifier
        let is_verifier: bool = env.storage()
            .instance()
            .get(&DataKey::Verifier(verifier.clone()))
            .unwrap_or(false);
        
        if !is_verifier {
            panic!("Caller is not a verifier");
        }
        
        // Get existing attestation
        let mut attestation: Attestation = env.storage()
            .instance()
            .get(&DataKey::Attestation(subject.clone(), claim_hash.clone()))
            .unwrap_or_else(|| panic!("Attestation not found"));
        
        // Revoke the attestation
        attestation.is_active = false;
        env.storage().instance().set(
            &DataKey::Attestation(subject.clone(), claim_hash.clone()),
            &attestation
        );
        
        // Emit event
        env.events().publish(
            ("Revoked",),
            (subject, verifier, claim_hash)
        );
    }

    /// Add a verifier (only owner can do this)
    pub fn add_verifier(env: Env, verifier: Address) {
        let owner: Address = env.storage()
            .instance()
            .get(&DataKey::Owner)
            .unwrap_or_else(|| panic!("Contract not initialized"));
        
        owner.require_auth();
        
        env.storage().instance().set(&DataKey::Verifier(verifier.clone()), &true);
        
        // Emit event
        env.events().publish(("VerifierAdded",), verifier);
    }

    /// Remove a verifier (only owner can do this)
    pub fn remove_verifier(env: Env, verifier: Address) {
        let owner: Address = env.storage()
            .instance()
            .get(&DataKey::Owner)
            .unwrap_or_else(|| panic!("Contract not initialized"));
        
        owner.require_auth();
        
        // Cannot remove owner as verifier
        if verifier == owner {
            panic!("Cannot remove owner as verifier");
        }
        
        env.storage().instance().set(&DataKey::Verifier(verifier.clone()), &false);
        
        // Emit event
        env.events().publish(("VerifierRemoved",), verifier);
    }

    /// Check if an address is a verifier
    pub fn is_verifier(env: Env, account: Address) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Verifier(account))
            .unwrap_or(false)
    }

    /// Get identity hash for a subject
    pub fn get_identity_hash(env: Env, subject: Address) -> Option<BytesN<32>> {
        let record: Option<IdentityRecord> = env.storage()
            .instance()
            .get(&DataKey::IdentityHash(subject));
        
        record.map(|r| r.hash)
    }

    /// Get identity metadata for a subject
    pub fn get_identity_meta(env: Env, subject: Address) -> Option<String> {
        let record: Option<IdentityRecord> = env.storage()
            .instance()
            .get(&DataKey::IdentityHash(subject));
        
        record.map(|r| r.meta)
    }

    /// Check if a specific attestation is active
    pub fn is_attested(env: Env, subject: Address, claim_hash: BytesN<32>) -> bool {
        let attestation: Option<Attestation> = env.storage()
            .instance()
            .get(&DataKey::Attestation(subject, claim_hash));
        
        attestation.map_or(false, |a| a.is_active)
    }

    /// Get all active attestations for a subject
    pub fn get_attestations(env: Env, subject: Address) -> Vec<BytesN<32>> {
        let all_attestations: Vec<BytesN<32>> = env.storage()
            .instance()
            .get(&DataKey::SubjectAttestations(subject.clone()))
            .unwrap_or(Vec::new(&env));
        
        let mut active_attestations = Vec::new(&env);
        
        for claim_hash in all_attestations.iter() {
            if let Some(attestation) = env.storage()
                .instance()
                .get::<DataKey, Attestation>(&DataKey::Attestation(subject.clone(), claim_hash.clone())) {
                if attestation.is_active {
                    active_attestations.push_back(claim_hash);
                }
            }
        }
        
        active_attestations
    }

    /// Get the contract owner
    pub fn get_owner(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Owner)
            .unwrap_or_else(|| panic!("Contract not initialized"))
    }
}