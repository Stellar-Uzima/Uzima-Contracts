// Cross-Chain Identity Contract - Identity verification across blockchains
#![no_std]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::unnecessary_map_or)]
#![allow(clippy::arithmetic_side_effects)]
#![allow(dead_code)]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
    String, Symbol, Vec,
};

// ==================== Existing Types ====================

#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub enum VerificationStatus {
    Unverified,
    Pending,
    Verified,
    Revoked,
    Expired,
}

#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum ChainId {
    Stellar,
    Ethereum,
    Polygon,
    Avalanche,
    BinanceSmartChain,
    Arbitrum,
    Optimism,
    Custom(u32),
}

#[derive(Clone)]
#[contracttype]
pub struct CrossChainIdentity {
    pub stellar_address: Address,
    pub external_chain: ChainId,
    pub external_address: String,
    pub verification_status: VerificationStatus,
    pub verified_at: u64,
    pub expires_at: u64,
    pub attestations: u32,
    pub metadata_hash: BytesN<32>,
}

#[derive(Clone)]
#[contracttype]
pub struct VerificationRequest {
    pub request_id: u64,
    pub stellar_address: Address,
    pub external_chain: ChainId,
    pub external_address: String,
    pub proof: BytesN<64>,
    pub created_at: u64,
    pub status: RequestStatus,
    pub validator_attestations: Vec<Address>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub enum RequestStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}

#[derive(Clone)]
#[contracttype]
pub struct Attestation {
    pub validator: Address,
    pub stellar_address: Address,
    pub external_chain: ChainId,
    pub attested_at: u64,
    pub is_valid: bool,
    pub signature: BytesN<64>,
}

#[derive(Clone)]
#[contracttype]
pub struct IdentityValidator {
    pub address: Address,
    pub name: String,
    pub public_key: BytesN<32>,
    pub is_active: bool,
    pub trust_score: u32,
    pub total_attestations: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct IdentitySync {
    pub stellar_address: Address,
    pub source_chain: ChainId,
    pub dest_chain: ChainId,
    pub sync_timestamp: u64,
    pub sync_status: SyncStatus,
    pub sync_proof: BytesN<32>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub enum SyncStatus {
    Initiated,
    InProgress,
    Completed,
    Failed,
}

// ==================== Storage Keys (DataKey Enum) ====================
// BUG FIX: identity_key always returned "id_key" and attestation_key always
// returned "att_key", causing all identities and attestations to overwrite
// each other. Now uses typed per-item storage keys.

#[contracttype]
pub enum DataKey {
    // Core config
    Admin,
    Bridge,
    Paused,
    RequestCount,
    SyncCount,
    MinAttestations,
    IdentityTtl,
    // Per-item storage (BUG FIX)
    Validator(Address),
    Request(u64),
    Identity(Address, ChainId), // BUG FIX: was always "id_key"
    Attestation(u64, Address),  // BUG FIX: was always "att_key"
    Sync(u64),
}

// Constants
const DEFAULT_MIN_ATTESTATIONS: u32 = 2;
const DEFAULT_IDENTITY_TTL: u64 = 31_536_000; // 1 year
const REQUEST_EXPIRY: u64 = 86_400; // 24 hours

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    ContractPaused = 2,
    AlreadyInitialized = 3,
    IdentityNotFound = 4,
    IdentityAlreadyExists = 5,
    IdentityExpired = 6,
    IdentityRevoked = 7,
    RequestNotFound = 8,
    RequestExpired = 9,
    RequestAlreadyProcessed = 10,
    ValidatorNotFound = 11,
    ValidatorNotActive = 12,
    DuplicateAttestation = 13,
    InsufficientAttestations = 14,
    InvalidProof = 15,
    InvalidChain = 16,
    SyncNotFound = 17,
    SyncFailed = 18,
}

#[contract]
pub struct CrossChainIdentityContract;

#[contractimpl]
impl CrossChainIdentityContract {
    pub fn initialize(env: Env, admin: Address, bridge_contract: Address) -> Result<bool, Error> {
        admin.require_auth();

        if env.storage().persistent().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage()
            .persistent()
            .set(&DataKey::Bridge, &bridge_contract);
        env.storage().persistent().set(&DataKey::Paused, &false);
        env.storage()
            .persistent()
            .set(&DataKey::RequestCount, &0u64);
        env.storage().persistent().set(&DataKey::SyncCount, &0u64);
        env.storage()
            .persistent()
            .set(&DataKey::MinAttestations, &DEFAULT_MIN_ATTESTATIONS);
        env.storage()
            .persistent()
            .set(&DataKey::IdentityTtl, &DEFAULT_IDENTITY_TTL);

        env.events().publish(
            (Symbol::new(&env, "IdentityContractInitialized"),),
            (admin.clone(),),
        );

        Ok(true)
    }

    // ==================== Admin Functions ====================

    pub fn add_validator(
        env: Env,
        caller: Address,
        validator_address: Address,
        name: String,
        public_key: BytesN<32>,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        Self::require_not_paused(&env)?;

        let validator = IdentityValidator {
            address: validator_address.clone(),
            name,
            public_key,
            is_active: true,
            trust_score: 50,
            total_attestations: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Validator(validator_address.clone()), &validator);

        env.events()
            .publish((Symbol::new(&env, "ValidatorAdded"),), (validator_address,));

        Ok(true)
    }

    pub fn deactivate_validator(
        env: Env,
        caller: Address,
        validator_address: Address,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        let key = DataKey::Validator(validator_address.clone());
        if let Some(mut validator) = env
            .storage()
            .persistent()
            .get::<DataKey, IdentityValidator>(&key)
        {
            validator.is_active = false;
            env.storage().persistent().set(&key, &validator);

            env.events().publish(
                (Symbol::new(&env, "ValidatorDeactivated"),),
                (validator_address,),
            );

            Ok(true)
        } else {
            Err(Error::ValidatorNotFound)
        }
    }

    pub fn update_trust_score(
        env: Env,
        caller: Address,
        validator_address: Address,
        trust_score: u32,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        let key = DataKey::Validator(validator_address.clone());
        if let Some(mut validator) = env
            .storage()
            .persistent()
            .get::<DataKey, IdentityValidator>(&key)
        {
            validator.trust_score = trust_score.min(100);
            env.storage().persistent().set(&key, &validator);
            Ok(true)
        } else {
            Err(Error::ValidatorNotFound)
        }
    }

    pub fn set_min_attestations(
        env: Env,
        caller: Address,
        min_attestations: u32,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        env.storage()
            .persistent()
            .set(&DataKey::MinAttestations, &min_attestations);
        Ok(true)
    }

    pub fn pause(env: Env, caller: Address) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        env.storage().persistent().set(&DataKey::Paused, &true);

        env.events().publish(
            (symbol_short!("Paused"),),
            (caller, env.ledger().timestamp()),
        );

        Ok(true)
    }

    pub fn unpause(env: Env, caller: Address) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        env.storage().persistent().set(&DataKey::Paused, &false);

        env.events().publish(
            (symbol_short!("Unpaused"),),
            (caller, env.ledger().timestamp()),
        );

        Ok(true)
    }

    // ==================== Identity Verification Functions ====================

    pub fn request_verification(
        env: Env,
        stellar_address: Address,
        external_chain: ChainId,
        external_address: String,
        proof: BytesN<64>,
    ) -> Result<u64, Error> {
        stellar_address.require_auth();
        Self::require_not_paused(&env)?;

        // BUG FIX: check the correct per-identity key
        let identity_key = DataKey::Identity(stellar_address.clone(), external_chain.clone());
        if let Some(existing) = env
            .storage()
            .persistent()
            .get::<DataKey, CrossChainIdentity>(&identity_key)
        {
            if existing.verification_status == VerificationStatus::Verified {
                return Err(Error::IdentityAlreadyExists);
            }
        }

        let request_id = Self::get_and_increment_request_count(&env);
        let now = env.ledger().timestamp();

        let request = VerificationRequest {
            request_id,
            stellar_address: stellar_address.clone(),
            external_chain: external_chain.clone(),
            external_address,
            proof,
            created_at: now,
            status: RequestStatus::Pending,
            validator_attestations: Vec::new(&env),
        };

        env.storage()
            .persistent()
            .set(&DataKey::Request(request_id), &request);

        env.events().publish(
            (Symbol::new(&env, "VerificationRequested"),),
            (stellar_address, external_chain, request_id),
        );

        Ok(request_id)
    }

    /// Validator attests to a verification request
    /// BUG FIX: each attestation stored per (request_id, validator) — was "att_key"
    pub fn attest_verification(
        env: Env,
        validator: Address,
        request_id: u64,
        is_valid: bool,
        signature: BytesN<64>,
    ) -> Result<bool, Error> {
        validator.require_auth();
        Self::require_not_paused(&env)?;
        Self::require_active_validator(&env, &validator)?;

        let req_key = DataKey::Request(request_id);
        let mut request = env
            .storage()
            .persistent()
            .get::<DataKey, VerificationRequest>(&req_key)
            .ok_or(Error::RequestNotFound)?;

        if request.status != RequestStatus::Pending {
            return Err(Error::RequestAlreadyProcessed);
        }

        let now = env.ledger().timestamp();
        if now > request.created_at + REQUEST_EXPIRY {
            request.status = RequestStatus::Expired;
            env.storage().persistent().set(&req_key, &request);
            return Err(Error::RequestExpired);
        }

        if request.validator_attestations.contains(&validator) {
            return Err(Error::DuplicateAttestation);
        }

        // BUG FIX: unique key per (request_id, validator) — was always "att_key"
        let attestation = Attestation {
            validator: validator.clone(),
            stellar_address: request.stellar_address.clone(),
            external_chain: request.external_chain.clone(),
            attested_at: now,
            is_valid,
            signature,
        };

        env.storage().persistent().set(
            &DataKey::Attestation(request_id, validator.clone()),
            &attestation,
        );

        request.validator_attestations.push_back(validator.clone());

        Self::increment_validator_attestations(&env, &validator);

        let min_attestations: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::MinAttestations)
            .unwrap_or(DEFAULT_MIN_ATTESTATIONS);

        if is_valid && request.validator_attestations.len() as u32 >= min_attestations {
            request.status = RequestStatus::Approved;

            Self::create_verified_identity(&env, &request)?;

            env.events().publish(
                (Symbol::new(&env, "VerificationApproved"),),
                (
                    request.stellar_address.clone(),
                    request.external_chain.clone(),
                    request_id,
                ),
            );
        }

        env.storage().persistent().set(&req_key, &request);

        env.events().publish(
            (Symbol::new(&env, "AttestationAdded"),),
            (validator, request_id, is_valid),
        );

        Ok(true)
    }

    pub fn revoke_identity(
        env: Env,
        caller: Address,
        stellar_address: Address,
        external_chain: ChainId,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_not_paused(&env)?;

        let is_admin = Self::is_admin(&env, &caller);
        if !is_admin && caller != stellar_address {
            return Err(Error::NotAuthorized);
        }

        let identity_key = DataKey::Identity(stellar_address.clone(), external_chain.clone());
        if let Some(mut identity) = env
            .storage()
            .persistent()
            .get::<DataKey, CrossChainIdentity>(&identity_key)
        {
            identity.verification_status = VerificationStatus::Revoked;
            env.storage().persistent().set(&identity_key, &identity);

            env.events().publish(
                (Symbol::new(&env, "IdentityRevoked"),),
                (stellar_address, external_chain),
            );

            Ok(true)
        } else {
            Err(Error::IdentityNotFound)
        }
    }

    // ==================== Identity Sync Functions ====================

    pub fn initiate_sync(
        env: Env,
        stellar_address: Address,
        source_chain: ChainId,
        dest_chain: ChainId,
    ) -> Result<u64, Error> {
        stellar_address.require_auth();
        Self::require_not_paused(&env)?;

        // BUG FIX: check identity using correct per-identity key
        let identity_key = DataKey::Identity(stellar_address.clone(), source_chain.clone());
        let identity = env
            .storage()
            .persistent()
            .get::<DataKey, CrossChainIdentity>(&identity_key)
            .ok_or(Error::IdentityNotFound)?;

        if identity.verification_status != VerificationStatus::Verified {
            return Err(Error::IdentityNotFound);
        }

        let now = env.ledger().timestamp();
        if now > identity.expires_at {
            return Err(Error::IdentityExpired);
        }

        let sync_id = Self::get_and_increment_sync_count(&env);
        let sync_proof = BytesN::from_array(&env, &[0u8; 32]);

        let sync = IdentitySync {
            stellar_address: stellar_address.clone(),
            source_chain: source_chain.clone(),
            dest_chain: dest_chain.clone(),
            sync_timestamp: now,
            sync_status: SyncStatus::Initiated,
            sync_proof,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Sync(sync_id), &sync);

        env.events().publish(
            (Symbol::new(&env, "SyncInitiated"),),
            (stellar_address, source_chain, dest_chain, sync_id),
        );

        Ok(sync_id)
    }

    pub fn update_sync_status(
        env: Env,
        validator: Address,
        sync_id: u64,
        status: SyncStatus,
        proof: BytesN<32>,
    ) -> Result<bool, Error> {
        validator.require_auth();
        Self::require_not_paused(&env)?;
        Self::require_active_validator(&env, &validator)?;

        let sync_key = DataKey::Sync(sync_id);
        let mut sync = env
            .storage()
            .persistent()
            .get::<DataKey, IdentitySync>(&sync_key)
            .ok_or(Error::SyncNotFound)?;

        sync.sync_status = status.clone();
        sync.sync_proof = proof;
        sync.sync_timestamp = env.ledger().timestamp();

        env.storage().persistent().set(&sync_key, &sync);

        env.events()
            .publish((Symbol::new(&env, "SyncStatusUpdated"),), (sync_id, status));

        Ok(true)
    }

    // ==================== Query Functions ====================

    /// Get identity by Stellar address and external chain
    /// BUG FIX: each (stellar_address, chain) has a unique storage entry
    pub fn get_identity(
        env: Env,
        stellar_address: Address,
        external_chain: ChainId,
    ) -> Option<CrossChainIdentity> {
        env.storage()
            .persistent()
            .get(&DataKey::Identity(stellar_address, external_chain))
    }

    pub fn verify_identity(env: Env, stellar_address: Address, external_chain: ChainId) -> bool {
        if let Some(identity) = Self::get_identity(env.clone(), stellar_address, external_chain) {
            let now = env.ledger().timestamp();
            identity.verification_status == VerificationStatus::Verified
                && now <= identity.expires_at
        } else {
            false
        }
    }

    pub fn get_request(env: Env, request_id: u64) -> Option<VerificationRequest> {
        env.storage()
            .persistent()
            .get(&DataKey::Request(request_id))
    }

    pub fn get_sync(env: Env, sync_id: u64) -> Option<IdentitySync> {
        env.storage().persistent().get(&DataKey::Sync(sync_id))
    }

    pub fn get_validator(env: Env, validator_address: Address) -> Option<IdentityValidator> {
        env.storage()
            .persistent()
            .get(&DataKey::Validator(validator_address))
    }

    pub fn get_attestation(env: Env, request_id: u64, validator: Address) -> Option<Attestation> {
        env.storage()
            .persistent()
            .get(&DataKey::Attestation(request_id, validator))
    }

    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    // ==================== Internal Helper Functions ====================

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .persistent()
            .get(&DataKey::Admin)
            .ok_or(Error::NotAuthorized)?;

        if caller != &admin {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    fn is_admin(env: &Env, caller: &Address) -> bool {
        let admin: Option<Address> = env.storage().persistent().get(&DataKey::Admin);
        admin.map_or(false, |a| &a == caller)
    }

    fn require_not_paused(env: &Env) -> Result<(), Error> {
        if env
            .storage()
            .persistent()
            .get(&DataKey::Paused)
            .unwrap_or(false)
        {
            return Err(Error::ContractPaused);
        }
        Ok(())
    }

    fn require_active_validator(env: &Env, validator: &Address) -> Result<(), Error> {
        match env
            .storage()
            .persistent()
            .get::<DataKey, IdentityValidator>(&DataKey::Validator(validator.clone()))
        {
            Some(v) if v.is_active => Ok(()),
            Some(_) => Err(Error::ValidatorNotActive),
            None => Err(Error::ValidatorNotFound),
        }
    }

    fn get_and_increment_request_count(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::RequestCount)
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::RequestCount, &(count + 1));
        count + 1
    }

    fn get_and_increment_sync_count(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::SyncCount)
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::SyncCount, &(count + 1));
        count + 1
    }

    fn create_verified_identity(env: &Env, request: &VerificationRequest) -> Result<(), Error> {
        let now = env.ledger().timestamp();
        let ttl: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::IdentityTtl)
            .unwrap_or(DEFAULT_IDENTITY_TTL);

        let identity = CrossChainIdentity {
            stellar_address: request.stellar_address.clone(),
            external_chain: request.external_chain.clone(),
            external_address: request.external_address.clone(),
            verification_status: VerificationStatus::Verified,
            verified_at: now,
            expires_at: now + ttl,
            attestations: request.validator_attestations.len(),
            metadata_hash: BytesN::from_array(&env, &[0u8; 32]),
        };

        // BUG FIX: store under unique (stellar_address, chain) key
        env.storage().persistent().set(
            &DataKey::Identity(
                request.stellar_address.clone(),
                request.external_chain.clone(),
            ),
            &identity,
        );

        env.events().publish(
            (Symbol::new(&env, "IdentityVerified"),),
            (
                request.stellar_address.clone(),
                request.external_chain.clone(),
            ),
        );

        Ok(())
    }

    fn increment_validator_attestations(env: &Env, validator: &Address) {
        let key = DataKey::Validator(validator.clone());
        if let Some(mut v) = env
            .storage()
            .persistent()
            .get::<DataKey, IdentityValidator>(&key)
        {
            v.total_attestations += 1;
            env.storage().persistent().set(&key, &v);
        }
    }
}
