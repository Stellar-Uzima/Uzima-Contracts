#![no_std]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Map,
    String, Symbol, Vec,
};

/// Represents the verification status of a cross-chain identity
#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub enum VerificationStatus {
    Unverified,
    Pending,
    Verified,
    Revoked,
    Expired,
}

/// Supported blockchain networks for identity
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

/// Cross-chain identity mapping
#[derive(Clone)]
#[contracttype]
pub struct CrossChainIdentity {
    pub stellar_address: Address,
    pub external_chain: ChainId,
    pub external_address: String, // Address on external chain
    pub verification_status: VerificationStatus,
    pub verified_at: u64,
    pub expires_at: u64,
    pub attestations: u32,
    pub metadata_hash: BytesN<32>, // Hash of identity metadata
}

/// Identity verification request
#[derive(Clone)]
#[contracttype]
pub struct VerificationRequest {
    pub request_id: u64,
    pub stellar_address: Address,
    pub external_chain: ChainId,
    pub external_address: String,
    pub proof: BytesN<64>, // Cryptographic proof of ownership
    pub created_at: u64,
    pub status: RequestStatus,
    pub validator_attestations: Vec<Address>,
}

/// Status of verification request
#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub enum RequestStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}

/// Identity attestation from a validator
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

/// Validator information for identity verification
#[derive(Clone)]
#[contracttype]
pub struct IdentityValidator {
    pub address: Address,
    pub name: String,
    pub public_key: BytesN<32>,
    pub is_active: bool,
    pub trust_score: u32, // 0-100
    pub total_attestations: u64,
}

/// Identity synchronization record
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

/// Synchronization status
#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub enum SyncStatus {
    Initiated,
    InProgress,
    Completed,
    Failed,
}

// Storage keys
const ADMIN: Symbol = symbol_short!("ADMIN");
const BRIDGE: Symbol = symbol_short!("BRIDGE");
const IDENTITIES: Symbol = symbol_short!("IDENTS");
const REQUESTS: Symbol = symbol_short!("REQUESTS");
const VALIDATORS: Symbol = symbol_short!("VALID");
const ATTESTATIONS: Symbol = symbol_short!("ATTEST");
const SYNCS: Symbol = symbol_short!("SYNCS");
const PAUSED: Symbol = symbol_short!("PAUSED");
const REQUEST_COUNT: Symbol = symbol_short!("REQ_CNT");
const MIN_ATTESTATIONS: Symbol = symbol_short!("MIN_ATT");
const IDENTITY_TTL: Symbol = symbol_short!("ID_TTL");

// Constants
const DEFAULT_MIN_ATTESTATIONS: u32 = 2;
const DEFAULT_IDENTITY_TTL: u64 = 31_536_000; // 1 year in seconds
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
    /// Initialize the identity contract
    pub fn initialize(env: Env, admin: Address, bridge_contract: Address) -> Result<bool, Error> {
        admin.require_auth();

        if env.storage().persistent().has(&ADMIN) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().persistent().set(&ADMIN, &admin);
        env.storage().persistent().set(&BRIDGE, &bridge_contract);
        env.storage().persistent().set(&PAUSED, &false);
        env.storage().persistent().set(&REQUEST_COUNT, &0u64);
        env.storage()
            .persistent()
            .set(&MIN_ATTESTATIONS, &DEFAULT_MIN_ATTESTATIONS);
        env.storage()
            .persistent()
            .set(&IDENTITY_TTL, &DEFAULT_IDENTITY_TTL);

        env.events().publish(
            (Symbol::new(&env, "IdentityContractInitialized"),),
            (admin.clone(),),
        );

        Ok(true)
    }

    // ==================== Admin Functions ====================

    /// Add a new identity validator
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
            trust_score: 50, // Default trust score
            total_attestations: 0,
        };

        let mut validators: Map<Address, IdentityValidator> = env
            .storage()
            .persistent()
            .get(&VALIDATORS)
            .unwrap_or(Map::new(&env));

        validators.set(validator_address.clone(), validator);
        env.storage().persistent().set(&VALIDATORS, &validators);

        env.events()
            .publish((Symbol::new(&env, "ValidatorAdded"),), (validator_address,));

        Ok(true)
    }

    /// Deactivate a validator
    pub fn deactivate_validator(
        env: Env,
        caller: Address,
        validator_address: Address,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        let mut validators: Map<Address, IdentityValidator> = env
            .storage()
            .persistent()
            .get(&VALIDATORS)
            .unwrap_or(Map::new(&env));

        if let Some(mut validator) = validators.get(validator_address.clone()) {
            validator.is_active = false;
            validators.set(validator_address.clone(), validator);
            env.storage().persistent().set(&VALIDATORS, &validators);

            env.events().publish(
                (Symbol::new(&env, "ValidatorDeactivated"),),
                (validator_address,),
            );

            Ok(true)
        } else {
            Err(Error::ValidatorNotFound)
        }
    }

    /// Update validator trust score
    pub fn update_trust_score(
        env: Env,
        caller: Address,
        validator_address: Address,
        trust_score: u32,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        let mut validators: Map<Address, IdentityValidator> = env
            .storage()
            .persistent()
            .get(&VALIDATORS)
            .unwrap_or(Map::new(&env));

        if let Some(mut validator) = validators.get(validator_address.clone()) {
            validator.trust_score = trust_score.min(100); // Cap at 100
            validators.set(validator_address.clone(), validator);
            env.storage().persistent().set(&VALIDATORS, &validators);
            Ok(true)
        } else {
            Err(Error::ValidatorNotFound)
        }
    }

    /// Set minimum attestations required
    pub fn set_min_attestations(
        env: Env,
        caller: Address,
        min_attestations: u32,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        env.storage()
            .persistent()
            .set(&MIN_ATTESTATIONS, &min_attestations);
        Ok(true)
    }

    /// Pause contract
    pub fn pause(env: Env, caller: Address) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        env.storage().persistent().set(&PAUSED, &true);

        env.events().publish(
            (symbol_short!("Paused"),),
            (caller, env.ledger().timestamp()),
        );

        Ok(true)
    }

    /// Unpause contract
    pub fn unpause(env: Env, caller: Address) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        env.storage().persistent().set(&PAUSED, &false);

        env.events().publish(
            (symbol_short!("Unpaused"),),
            (caller, env.ledger().timestamp()),
        );

        Ok(true)
    }

    // ==================== Identity Verification Functions ====================

    /// Request identity verification for an external chain address
    pub fn request_verification(
        env: Env,
        stellar_address: Address,
        external_chain: ChainId,
        external_address: String,
        proof: BytesN<64>,
    ) -> Result<u64, Error> {
        stellar_address.require_auth();
        Self::require_not_paused(&env)?;

        // Check if identity already exists and is verified
        let identity_key = Self::identity_key(&env, &stellar_address, &external_chain);
        let identities: Map<Symbol, CrossChainIdentity> = env
            .storage()
            .persistent()
            .get(&IDENTITIES)
            .unwrap_or(Map::new(&env));

        if let Some(existing) = identities.get(identity_key.clone()) {
            if existing.verification_status == VerificationStatus::Verified {
                return Err(Error::IdentityAlreadyExists);
            }
        }

        // Create verification request
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

        let mut requests: Map<u64, VerificationRequest> = env
            .storage()
            .persistent()
            .get(&REQUESTS)
            .unwrap_or(Map::new(&env));

        requests.set(request_id, request);
        env.storage().persistent().set(&REQUESTS, &requests);

        env.events().publish(
            (Symbol::new(&env, "VerificationRequested"),),
            (stellar_address, external_chain, request_id),
        );

        Ok(request_id)
    }

    /// Validator attests to a verification request
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

        let mut requests: Map<u64, VerificationRequest> = env
            .storage()
            .persistent()
            .get(&REQUESTS)
            .unwrap_or(Map::new(&env));

        let mut request = requests.get(request_id).ok_or(Error::RequestNotFound)?;

        // Check request status
        if request.status != RequestStatus::Pending {
            return Err(Error::RequestAlreadyProcessed);
        }

        // Check expiry
        let now = env.ledger().timestamp();
        if now > request.created_at + REQUEST_EXPIRY {
            request.status = RequestStatus::Expired;
            requests.set(request_id, request);
            env.storage().persistent().set(&REQUESTS, &requests);
            return Err(Error::RequestExpired);
        }

        // Check for duplicate attestation
        if request.validator_attestations.contains(&validator) {
            return Err(Error::DuplicateAttestation);
        }

        // Store attestation
        let attestation = Attestation {
            validator: validator.clone(),
            stellar_address: request.stellar_address.clone(),
            external_chain: request.external_chain.clone(),
            attested_at: now,
            is_valid,
            signature,
        };

        let attest_key = Self::attestation_key(&env, request_id, &validator);
        let mut attestations: Map<Symbol, Attestation> = env
            .storage()
            .persistent()
            .get(&ATTESTATIONS)
            .unwrap_or(Map::new(&env));

        attestations.set(attest_key, attestation);
        env.storage().persistent().set(&ATTESTATIONS, &attestations);

        // Add to request's validator list
        request.validator_attestations.push_back(validator.clone());

        // Increment validator's attestation count
        Self::increment_validator_attestations(&env, &validator);

        // Check if we have enough valid attestations
        let min_attestations: u32 = env
            .storage()
            .persistent()
            .get(&MIN_ATTESTATIONS)
            .unwrap_or(DEFAULT_MIN_ATTESTATIONS);

        if is_valid && request.validator_attestations.len() as u32 >= min_attestations {
            request.status = RequestStatus::Approved;

            // Create verified identity
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

        requests.set(request_id, request);
        env.storage().persistent().set(&REQUESTS, &requests);

        env.events().publish(
            (Symbol::new(&env, "AttestationAdded"),),
            (validator, request_id, is_valid),
        );

        Ok(true)
    }

    /// Revoke an identity
    pub fn revoke_identity(
        env: Env,
        caller: Address,
        stellar_address: Address,
        external_chain: ChainId,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_not_paused(&env)?;

        // Only admin or the identity owner can revoke
        let is_admin = Self::is_admin(&env, &caller);
        if !is_admin && caller != stellar_address {
            return Err(Error::NotAuthorized);
        }

        let identity_key = Self::identity_key(&env, &stellar_address, &external_chain);
        let mut identities: Map<Symbol, CrossChainIdentity> = env
            .storage()
            .persistent()
            .get(&IDENTITIES)
            .unwrap_or(Map::new(&env));

        if let Some(mut identity) = identities.get(identity_key.clone()) {
            identity.verification_status = VerificationStatus::Revoked;
            identities.set(identity_key, identity);
            env.storage().persistent().set(&IDENTITIES, &identities);

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

    /// Initiate identity synchronization to another chain
    pub fn initiate_sync(
        env: Env,
        stellar_address: Address,
        source_chain: ChainId,
        dest_chain: ChainId,
    ) -> Result<u64, Error> {
        stellar_address.require_auth();
        Self::require_not_paused(&env)?;

        // Verify identity exists and is verified
        let identity_key = Self::identity_key(&env, &stellar_address, &source_chain);
        let identities: Map<Symbol, CrossChainIdentity> = env
            .storage()
            .persistent()
            .get(&IDENTITIES)
            .unwrap_or(Map::new(&env));

        let identity = identities
            .get(identity_key)
            .ok_or(Error::IdentityNotFound)?;

        if identity.verification_status != VerificationStatus::Verified {
            return Err(Error::IdentityNotFound);
        }

        // Check if identity is expired
        let now = env.ledger().timestamp();
        if now > identity.expires_at {
            return Err(Error::IdentityExpired);
        }

        // Create sync record
        let sync_id = Self::get_and_increment_request_count(&env);
        let sync_proof = BytesN::from_array(&env, &[0u8; 32]); // Placeholder

        let sync = IdentitySync {
            stellar_address: stellar_address.clone(),
            source_chain: source_chain.clone(),
            dest_chain: dest_chain.clone(),
            sync_timestamp: now,
            sync_status: SyncStatus::Initiated,
            sync_proof,
        };

        let mut syncs: Map<u64, IdentitySync> = env
            .storage()
            .persistent()
            .get(&SYNCS)
            .unwrap_or(Map::new(&env));

        syncs.set(sync_id, sync);
        env.storage().persistent().set(&SYNCS, &syncs);

        env.events().publish(
            (Symbol::new(&env, "SyncInitiated"),),
            (stellar_address, source_chain, dest_chain, sync_id),
        );

        Ok(sync_id)
    }

    /// Update sync status (called by validators/bridge)
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

        let mut syncs: Map<u64, IdentitySync> = env
            .storage()
            .persistent()
            .get(&SYNCS)
            .unwrap_or(Map::new(&env));

        let mut sync = syncs.get(sync_id).ok_or(Error::SyncNotFound)?;

        sync.sync_status = status.clone();
        sync.sync_proof = proof;
        sync.sync_timestamp = env.ledger().timestamp();

        syncs.set(sync_id, sync.clone());
        env.storage().persistent().set(&SYNCS, &syncs);

        env.events()
            .publish((Symbol::new(&env, "SyncStatusUpdated"),), (sync_id, status));

        Ok(true)
    }

    // ==================== Query Functions ====================

    /// Get identity by Stellar address and chain
    pub fn get_identity(
        env: Env,
        stellar_address: Address,
        external_chain: ChainId,
    ) -> Option<CrossChainIdentity> {
        let identity_key = Self::identity_key(&env, &stellar_address, &external_chain);
        let identities: Map<Symbol, CrossChainIdentity> = env
            .storage()
            .persistent()
            .get(&IDENTITIES)
            .unwrap_or(Map::new(&env));

        identities.get(identity_key)
    }

    /// Verify if an identity is valid
    pub fn verify_identity(env: Env, stellar_address: Address, external_chain: ChainId) -> bool {
        if let Some(identity) = Self::get_identity(env.clone(), stellar_address, external_chain) {
            let now = env.ledger().timestamp();
            identity.verification_status == VerificationStatus::Verified
                && now <= identity.expires_at
        } else {
            false
        }
    }

    /// Get verification request
    pub fn get_request(env: Env, request_id: u64) -> Option<VerificationRequest> {
        let requests: Map<u64, VerificationRequest> = env
            .storage()
            .persistent()
            .get(&REQUESTS)
            .unwrap_or(Map::new(&env));

        requests.get(request_id)
    }

    /// Get sync record
    pub fn get_sync(env: Env, sync_id: u64) -> Option<IdentitySync> {
        let syncs: Map<u64, IdentitySync> = env
            .storage()
            .persistent()
            .get(&SYNCS)
            .unwrap_or(Map::new(&env));

        syncs.get(sync_id)
    }

    /// Get validator info
    pub fn get_validator(env: Env, validator_address: Address) -> Option<IdentityValidator> {
        let validators: Map<Address, IdentityValidator> = env
            .storage()
            .persistent()
            .get(&VALIDATORS)
            .unwrap_or(Map::new(&env));

        validators.get(validator_address)
    }

    /// Check if contract is paused
    pub fn is_paused(env: Env) -> bool {
        env.storage().persistent().get(&PAUSED).unwrap_or(false)
    }

    // ==================== Internal Helper Functions ====================

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN)
            .ok_or(Error::NotAuthorized)?;

        if caller != &admin {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    fn is_admin(env: &Env, caller: &Address) -> bool {
        let admin: Option<Address> = env.storage().persistent().get(&ADMIN);
        admin.map_or(false, |a| &a == caller)
    }

    fn require_not_paused(env: &Env) -> Result<(), Error> {
        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }
        Ok(())
    }

    fn require_active_validator(env: &Env, validator: &Address) -> Result<(), Error> {
        let validators: Map<Address, IdentityValidator> = env
            .storage()
            .persistent()
            .get(&VALIDATORS)
            .unwrap_or(Map::new(&env));

        match validators.get(validator.clone()) {
            Some(v) if v.is_active => Ok(()),
            Some(_) => Err(Error::ValidatorNotActive),
            None => Err(Error::ValidatorNotFound),
        }
    }

    fn get_and_increment_request_count(env: &Env) -> u64 {
        let count: u64 = env.storage().persistent().get(&REQUEST_COUNT).unwrap_or(0);
        env.storage().persistent().set(&REQUEST_COUNT, &(count + 1));
        count + 1
    }

    fn identity_key(_env: &Env, _stellar_address: &Address, _chain: &ChainId) -> Symbol {
        Symbol::new(&_env, "id_key")
    }

    fn attestation_key(_env: &Env, _request_id: u64, _validator: &Address) -> Symbol {
        Symbol::new(&_env, "att_key")
    }

    fn create_verified_identity(env: &Env, request: &VerificationRequest) -> Result<(), Error> {
        let now = env.ledger().timestamp();
        let ttl: u64 = env
            .storage()
            .persistent()
            .get(&IDENTITY_TTL)
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

        let identity_key =
            Self::identity_key(&env, &request.stellar_address, &request.external_chain);
        let mut identities: Map<Symbol, CrossChainIdentity> = env
            .storage()
            .persistent()
            .get(&IDENTITIES)
            .unwrap_or(Map::new(&env));

        identities.set(identity_key, identity);
        env.storage().persistent().set(&IDENTITIES, &identities);

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
        let mut validators: Map<Address, IdentityValidator> = env
            .storage()
            .persistent()
            .get(&VALIDATORS)
            .unwrap_or(Map::new(&env));

        if let Some(mut v) = validators.get(validator.clone()) {
            v.total_attestations += 1;
            validators.set(validator.clone(), v);
            env.storage().persistent().set(&VALIDATORS, &validators);
        }
    }
}
