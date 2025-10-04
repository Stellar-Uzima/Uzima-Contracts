#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, String, Vec};

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

            // Initialization guard: panic if already initialized
            if env.storage().instance().has(&DataKey::Owner) {
                panic!("Contract already initialized");
            }

            // Set the owner
            env.storage().instance().set(&DataKey::Owner, &owner);

            // Owner is automatically a verifier
            env.storage()
                .instance()
                .set(&DataKey::Verifier(owner.clone()), &true);

            // Emit Initialized event
            env.events().publish(("Initialized",), owner.clone());
    }

    /// Register an identity hash with metadata
    /// Only the subject can register their own identity hash
    pub fn register_identity_hash(env: Env, hash: BytesN<32>, subject: Address, meta: String) {
        // Require authorization from the subject
        subject.require_auth();

        let identity_record = IdentityRecord {
            hash: hash.clone(),
            meta: meta.clone(),
            registered_by: subject.clone(),
        };

        env.storage()
            .instance()
            .set(&DataKey::IdentityHash(subject.clone()), &identity_record);

        // Emit event
        env.events()
            .publish(("IdentityRegistered",), (subject, hash, meta));
    }

    /// Create an attestation (only verifiers can do this)
    pub fn attest(env: Env, verifier: Address, subject: Address, claim_hash: BytesN<32>) {
        verifier.require_auth();

        // Check if caller is a verifier
        let is_verifier: bool = env
            .storage()
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
            &attestation,
        );

        // Add to subject's attestation list
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

        // Emit event
        env.events()
            .publish(("Attested",), (subject, verifier, claim_hash));
    }

    /// Revoke an attestation (only verifiers can do this)
    pub fn revoke_attestation(
        env: Env,
        verifier: Address,
        subject: Address,
        claim_hash: BytesN<32>,
    ) {
        verifier.require_auth();

        // Check if caller is a verifier
        let is_verifier: bool = env
            .storage()
            .instance()
            .get(&DataKey::Verifier(verifier.clone()))
            .unwrap_or(false);

        if !is_verifier {
            panic!("Caller is not a verifier");
        }

        // Get existing attestation
        let mut attestation: Attestation = env
            .storage()
            .instance()
            .get(&DataKey::Attestation(subject.clone(), claim_hash.clone()))
            .unwrap_or_else(|| panic!("Attestation not found"));

        // Revoke the attestation
        attestation.is_active = false;
        env.storage().instance().set(
            &DataKey::Attestation(subject.clone(), claim_hash.clone()),
            &attestation,
        );

        // Emit event
        env.events()
            .publish(("Revoked",), (subject, verifier, claim_hash));
    }

    /// Add a verifier (only owner can do this)
    pub fn add_verifier(env: Env, verifier: Address) {
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .unwrap_or_else(|| panic!("Contract not initialized"));

        owner.require_auth();

        env.storage()
            .instance()
            .set(&DataKey::Verifier(verifier.clone()), &true);

        // Emit event
        env.events().publish(("VerifierAdded",), verifier);
    }

    /// Remove a verifier (only owner can do this)
    pub fn remove_verifier(env: Env, verifier: Address) {
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .unwrap_or_else(|| panic!("Contract not initialized"));

        owner.require_auth();

        // Cannot remove owner as verifier
        if verifier == owner {
            panic!("Cannot remove owner as verifier");
        }

        env.storage()
            .instance()
            .set(&DataKey::Verifier(verifier.clone()), &false);

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
        let record: Option<IdentityRecord> = env
            .storage()
            .instance()
            .get(&DataKey::IdentityHash(subject));

        record.map(|r| r.hash)
    }

    /// Get identity metadata for a subject
    pub fn get_identity_meta(env: Env, subject: Address) -> Option<String> {
        let record: Option<IdentityRecord> = env
            .storage()
            .instance()
            .get(&DataKey::IdentityHash(subject));

        record.map(|r| r.meta)
    }

    /// Check if a specific attestation is active
    pub fn is_attested(env: Env, subject: Address, claim_hash: BytesN<32>) -> bool {
        let attestation: Option<Attestation> = env
            .storage()
            .instance()
            .get(&DataKey::Attestation(subject, claim_hash));

        attestation.map_or(false, |a| a.is_active)
    }

    /// Get all active attestations for a subject
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

    /// Get the contract owner
    pub fn get_owner(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Owner)
            .unwrap_or_else(|| panic!("Contract not initialized"))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[should_panic(expected = "Contract already initialized")]
    fn test_double_initialization_panics() {
        let env = Env::default();
        let contract_id = env.register_contract(None, IdentityRegistryContract);
        let client = IdentityRegistryContractClient::new(&env, &contract_id);
        let owner1 = Address::generate(&env);
        let owner2 = Address::generate(&env);

        // First initialization should succeed
        client.mock_all_auths().initialize(&owner1);

        // Second initialization should panic
        client.mock_all_auths().initialize(&owner2);
    }
    use super::*;
    use soroban_sdk::testutils::{Address as _, Events};
    use soroban_sdk::{Address, BytesN, Env, String};

    fn create_contract() -> (Env, IdentityRegistryContractClient<'static>, Address) {
        let env = Env::default();
        let contract_id = env.register_contract(None, IdentityRegistryContract);
        let client = IdentityRegistryContractClient::new(&env, &contract_id);
        let owner = Address::generate(&env);

        // Initialize the contract
        client.mock_all_auths().initialize(&owner);

        (env, client, owner)
    }

    #[test]
    fn test_initialize_and_owner_is_verifier() {
        let (_env, client, owner) = create_contract();

        // Owner should be a verifier by default
        assert!(client.is_verifier(&owner));

        // Owner should be retrievable
        assert_eq!(client.get_owner(), owner);
    }

    #[test]
    fn test_register_identity_hash() {
        let (env, client, _owner) = create_contract();
        let subject = Address::generate(&env);

        let hash = BytesN::from_array(&env, &[1; 32]);
        let meta = String::from_str(&env, "Healthcare Provider License #12345");

        // Register identity hash - subject must authorize
        client.mock_all_auths().register_identity_hash(&hash, &subject, &meta);

        // Verify storage
        assert_eq!(client.get_identity_hash(&subject), Some(hash));
        assert_eq!(client.get_identity_meta(&subject), Some(meta.clone()));

        // Verify event emission
        let events = env.events().all();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_register_identity_hash_with_correct_registrar() {
        let (env, client, _owner) = create_contract();
        let subject = Address::generate(&env);

        let hash = BytesN::from_array(&env, &[1; 32]);
        let meta = String::from_str(&env, "Healthcare Provider License #12345");

        // Register identity hash
        client.mock_all_auths().register_identity_hash(&hash, &subject, &meta);

        // Verify that registered_by is set to the subject (not the contract)
        let record_key = DataKey::IdentityHash(subject.clone());
        let record: IdentityRecord = env.storage().instance().get(&record_key).unwrap();

        // The registered_by field should be the subject, not the contract address
        assert_eq!(record.registered_by, subject);
        assert_eq!(record.hash, hash);
        assert_eq!(record.meta, meta);
    }

    #[test]
    fn test_add_and_remove_verifier() {
        let (env, client, _owner) = create_contract();
        let new_verifier = Address::generate(&env);

        // Add verifier
        client.mock_all_auths().add_verifier(&new_verifier);
        assert!(client.is_verifier(&new_verifier));

        // Remove verifier
        client.mock_all_auths().remove_verifier(&new_verifier);
        assert!(!client.is_verifier(&new_verifier));

        // Verify events
        let events = env.events().all();
        assert_eq!(events.len(), 2);
    }

    #[test]
    #[should_panic(expected = "Cannot remove owner as verifier")]
    fn test_cannot_remove_owner_as_verifier() {
        let (_env, client, owner) = create_contract();

        // Try to remove owner as verifier (should panic)
        client.mock_all_auths().remove_verifier(&owner);
    }

    #[test]
    fn test_attest_and_verify() {
        let (env, client, _owner) = create_contract();
        let verifier = Address::generate(&env);
        let subject = Address::generate(&env);

        // Add verifier
        client.mock_all_auths().add_verifier(&verifier);

        // Create attestation
        let claim_hash = BytesN::from_array(&env, &[2; 32]);
        client
            .mock_all_auths()
            .attest(&verifier, &subject, &claim_hash);

        // Verify attestation
        assert!(client.is_attested(&subject, &claim_hash));

        // Check attestations list
        let attestations = client.get_attestations(&subject);
        assert_eq!(attestations.len(), 1);
        assert_eq!(attestations.get(0).unwrap(), claim_hash);
    }

    #[test]
    #[should_panic(expected = "Caller is not a verifier")]
    fn test_attest_unauthorized() {
        let (env, client, _owner) = create_contract();
        let unauthorized = Address::generate(&env);
        let subject = Address::generate(&env);

        let claim_hash = BytesN::from_array(&env, &[3; 32]);

        // Try to attest without being a verifier (should panic)
        client
            .mock_all_auths()
            .attest(&unauthorized, &subject, &claim_hash);
    }

    #[test]
    fn test_revoke_attestation() {
        let (env, client, _owner) = create_contract();
        let verifier = Address::generate(&env);
        let subject = Address::generate(&env);

        // Add verifier
        client.mock_all_auths().add_verifier(&verifier);

        // Create attestation
        let claim_hash = BytesN::from_array(&env, &[4; 32]);
        client
            .mock_all_auths()
            .attest(&verifier, &subject, &claim_hash);

        // Verify attestation exists
        assert!(client.is_attested(&subject, &claim_hash));

        // Revoke attestation
        client
            .mock_all_auths()
            .revoke_attestation(&verifier, &subject, &claim_hash);

        // Verify attestation is revoked
        assert!(!client.is_attested(&subject, &claim_hash));

        // Check attestations list (should be empty after revocation)
        let attestations = client.get_attestations(&subject);
        assert_eq!(attestations.len(), 0);
    }

    #[test]
    #[should_panic(expected = "Caller is not a verifier")]
    fn test_revoke_attestation_unauthorized() {
        let (env, client, _owner) = create_contract();
        let unauthorized = Address::generate(&env);
        let subject = Address::generate(&env);

        let claim_hash = BytesN::from_array(&env, &[5; 32]);

        // Try to revoke without being a verifier (should panic)
        client
            .mock_all_auths()
            .revoke_attestation(&unauthorized, &subject, &claim_hash);
    }

    #[test]
    fn test_multiple_attestations() {
        let (env, client, _owner) = create_contract();
        let verifier1 = Address::generate(&env);
        let verifier2 = Address::generate(&env);
        let subject = Address::generate(&env);

        // Add verifiers
        client.mock_all_auths().add_verifier(&verifier1);
        client.mock_all_auths().add_verifier(&verifier2);

        // Create multiple attestations
        let claim_hash1 = BytesN::from_array(&env, &[6; 32]);
        let claim_hash2 = BytesN::from_array(&env, &[7; 32]);
        let claim_hash3 = BytesN::from_array(&env, &[8; 32]);

        client
            .mock_all_auths()
            .attest(&verifier1, &subject, &claim_hash1);
        client
            .mock_all_auths()
            .attest(&verifier1, &subject, &claim_hash2);
        client
            .mock_all_auths()
            .attest(&verifier2, &subject, &claim_hash3);

        // Verify all attestations
        assert!(client.is_attested(&subject, &claim_hash1));
        assert!(client.is_attested(&subject, &claim_hash2));
        assert!(client.is_attested(&subject, &claim_hash3));

        // Check attestations list
        let attestations = client.get_attestations(&subject);
        assert_eq!(attestations.len(), 3);

        // Revoke one attestation
        client
            .mock_all_auths()
            .revoke_attestation(&verifier1, &subject, &claim_hash1);

        // Verify partial revocation
        assert!(!client.is_attested(&subject, &claim_hash1));
        assert!(client.is_attested(&subject, &claim_hash2));
        assert!(client.is_attested(&subject, &claim_hash3));

        // Check updated attestations list
        let attestations = client.get_attestations(&subject);
        assert_eq!(attestations.len(), 2);
    }

    #[test]
    fn test_identity_record_persistence() {
        let (env, client, _owner) = create_contract();
        let subject1 = Address::generate(&env);
        let subject2 = Address::generate(&env);

        let hash1 = BytesN::from_array(&env, &[9; 32]);
        let hash2 = BytesN::from_array(&env, &[10; 32]);
        let meta1 = String::from_str(&env, "Doctor License");
        let meta2 = String::from_str(&env, "Clinic Registration");

        // Register multiple identities
        client.mock_all_auths().register_identity_hash(&hash1, &subject1, &meta1);
        client.mock_all_auths().register_identity_hash(&hash2, &subject2, &meta2);

        // Verify both are stored correctly
        assert_eq!(client.get_identity_hash(&subject1), Some(hash1));
        assert_eq!(client.get_identity_meta(&subject1), Some(meta1));
        assert_eq!(client.get_identity_hash(&subject2), Some(hash2));
        assert_eq!(client.get_identity_meta(&subject2), Some(meta2));

        // Verify non-existent subject returns None
        let non_existent = Address::generate(&env);
        assert_eq!(client.get_identity_hash(&non_existent), None);
        assert_eq!(client.get_identity_meta(&non_existent), None);
    }
}
