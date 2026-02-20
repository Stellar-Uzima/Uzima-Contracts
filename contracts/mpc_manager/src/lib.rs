#![no_std]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Bytes, BytesN, Env,
    String, Symbol, Vec,
};

// =============================================================================
// Types
// =============================================================================

/// Shamir's Secret Sharing share structure
#[derive(Clone)]
#[contracttype]
pub struct SecretShare {
    /// Share identifier (x coordinate in polynomial)
    pub share_id: u32,
    /// Share value (y coordinate)
    pub share_value: Bytes,
    /// Commitment to the share for verification
    pub commitment: BytesN<32>,
    /// Timestamp when share was created
    pub created_at: u64,
}

/// Verifiable computation proof
#[derive(Clone)]
#[contracttype]
pub struct ComputationProof {
    /// Type of computation (statistical, ML, etc.)
    pub computation_type: String,
    /// Input commitments hash
    pub input_commitment: BytesN<32>,
    /// Output hash
    pub output_hash: BytesN<32>,
    /// Zero-knowledge proof or SNARK
    pub proof_data: Bytes,
    /// Verification key hash
    pub verification_key_hash: BytesN<32>,
    /// Gas used for computation
    pub gas_used: u64,
    /// Timestamp of proof generation
    pub created_at: u64,
}

/// MPC computation types
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum ComputationType {
    StatisticalAnalysis,
    SecureAggregation,
    PrivacyPreservingML,
    DiagnosticAnalysis,
    DrugDiscovery,
}

/// Audit trail entry for MPC operations
#[derive(Clone)]
#[contracttype]
pub struct AuditEntry {
    /// Participant address
    pub participant: Address,
    /// Operation type
    pub operation: String,
    /// Session ID
    pub session_id: BytesN<32>,
    /// Timestamp
    pub timestamp: u64,
    /// Gas consumed
    pub gas_used: u64,
    /// Additional metadata
    pub metadata: Bytes,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum SessionStatus {
    Initiated,
    CommitPhase,
    RevealPhase,
    Finalized,
    Aborted,
    Expired,
}

#[derive(Clone)]
#[contracttype]
pub struct ShareReveal {
    pub share_ref: String,
    pub share_hash: BytesN<32>,
    pub revealed_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct MPCSession {
    pub session_id: BytesN<32>,
    pub initiator: Address,
    pub participants: Vec<Address>,
    pub threshold: u32,
    pub purpose: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub status: SessionStatus,
    pub commits: u32,
    pub reveals: u32,
    /// Result reference; empty string means "no result yet".
    pub result_ref: String,
    /// Result hash; all-zero means "no result yet".
    pub result_hash: BytesN<32>,
    /// Optional proof reference; empty string means "no proof".
    pub proof_ref: String,
    /// Optional proof hash; all-zero means "no proof".
    pub proof_hash: BytesN<32>,
    /// Computation type for this session
    pub computation_type: ComputationType,
    /// Total gas used by all participants
    pub total_gas_used: u64,
    /// Number of audit entries
    pub audit_entries: u32,
}

#[contracttype]
pub enum DataKey {
    Initialized,
    Admin,
    Session(BytesN<32>),
    Commit(BytesN<32>, Address),
    Reveal(BytesN<32>, Address),
    SecretShare(BytesN<32>, Address, u32),
    ComputationProof(BytesN<32>),
    AuditEntry(u32),
    AuditCounter,
    GasTracker(BytesN<32>, Address),
}

const ADMIN: Symbol = symbol_short!("ADMIN");

// =============================================================================
// Errors
// =============================================================================

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    InvalidInput = 4,
    SessionNotFound = 5,
    SessionExpired = 6,
    InvalidState = 7,
    DuplicateCommit = 8,
    DuplicateReveal = 9,
    ThresholdNotMet = 10,
    InvalidShare = 11,
    ComputationFailed = 12,
    ProofVerificationFailed = 13,
    GasLimitExceeded = 14,
    InsufficientParticipants = 15,
}

// =============================================================================
// Contract
// =============================================================================

#[contract]
pub struct MPCManager;

#[contractimpl]
impl MPCManager {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&ADMIN, &admin);
        env.events()
            .publish((symbol_short!("mpc"), symbol_short!("init")), admin);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn start_session(
        env: Env,
        initiator: Address,
        session_id: BytesN<32>,
        participants: Vec<Address>,
        threshold: u32,
        purpose: String,
        ttl_secs: u64,
        computation_type: ComputationType,
    ) -> Result<(), Error> {
        initiator.require_auth();
        Self::require_initialized(&env)?;

        if participants.is_empty() {
            return Err(Error::InvalidInput);
        }
        if threshold == 0 || threshold > participants.len() {
            return Err(Error::InvalidInput);
        }
        if ttl_secs == 0 {
            return Err(Error::InvalidInput);
        }
        if env
            .storage()
            .persistent()
            .has(&DataKey::Session(session_id.clone()))
        {
            return Err(Error::InvalidInput);
        }

        let now = env.ledger().timestamp();
        let empty = String::from_str(&env, "");
        let zero_hash = BytesN::from_array(&env, &[0u8; 32]);
        let session = MPCSession {
            session_id: session_id.clone(),
            initiator: initiator.clone(),
            participants,
            threshold,
            purpose,
            created_at: now,
            expires_at: now.saturating_add(ttl_secs),
            status: SessionStatus::CommitPhase,
            commits: 0,
            reveals: 0,
            result_ref: empty.clone(),
            result_hash: zero_hash.clone(),
            proof_ref: empty,
            proof_hash: zero_hash,
            computation_type,
            total_gas_used: 0,
            audit_entries: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Session(session_id.clone()), &session);
        env.events().publish(
            (symbol_short!("mpc"), symbol_short!("start")),
            (initiator, session_id),
        );
        Ok(())
    }

    pub fn commit_share(
        env: Env,
        participant: Address,
        session_id: BytesN<32>,
        commitment_hash: BytesN<32>,
    ) -> Result<(), Error> {
        participant.require_auth();
        Self::require_initialized(&env)?;

        let mut session: MPCSession = env
            .storage()
            .persistent()
            .get(&DataKey::Session(session_id.clone()))
            .ok_or(Error::SessionNotFound)?;
        Self::require_not_expired(&env, &session)?;
        if session.status != SessionStatus::CommitPhase {
            return Err(Error::InvalidState);
        }
        if !session.participants.contains(&participant) {
            return Err(Error::NotAuthorized);
        }

        let commit_key = DataKey::Commit(session_id.clone(), participant.clone());
        if env.storage().persistent().has(&commit_key) {
            return Err(Error::DuplicateCommit);
        }
        env.storage()
            .persistent()
            .set(&commit_key, &commitment_hash);

        session.commits = session.commits.saturating_add(1);

        // Automatically move to reveal phase when threshold commits met.
        if session.commits >= session.threshold {
            session.status = SessionStatus::RevealPhase;
        }
        env.storage()
            .persistent()
            .set(&DataKey::Session(session_id.clone()), &session);

        env.events().publish(
            (symbol_short!("mpc"), symbol_short!("commit")),
            (participant, session_id),
        );
        Ok(())
    }

    pub fn reveal_share(
        env: Env,
        participant: Address,
        session_id: BytesN<32>,
        share_ref: String,
        share_hash: BytesN<32>,
    ) -> Result<(), Error> {
        participant.require_auth();
        Self::require_initialized(&env)?;

        let mut session: MPCSession = env
            .storage()
            .persistent()
            .get(&DataKey::Session(session_id.clone()))
            .ok_or(Error::SessionNotFound)?;
        Self::require_not_expired(&env, &session)?;
        if session.status != SessionStatus::RevealPhase {
            return Err(Error::InvalidState);
        }
        if !session.participants.contains(&participant) {
            return Err(Error::NotAuthorized);
        }
        if share_ref.is_empty() {
            return Err(Error::InvalidInput);
        }

        let reveal_key = DataKey::Reveal(session_id.clone(), participant.clone());
        if env.storage().persistent().has(&reveal_key) {
            return Err(Error::DuplicateReveal);
        }

        let reveal = ShareReveal {
            share_ref,
            share_hash,
            revealed_at: env.ledger().timestamp(),
        };
        env.storage().persistent().set(&reveal_key, &reveal);

        session.reveals = session.reveals.saturating_add(1);
        env.storage()
            .persistent()
            .set(&DataKey::Session(session_id.clone()), &session);

        env.events().publish(
            (symbol_short!("mpc"), symbol_short!("reveal")),
            (participant, session_id),
        );
        Ok(())
    }

    pub fn finalize_session(
        env: Env,
        initiator: Address,
        session_id: BytesN<32>,
        result_ref: String,
        result_hash: BytesN<32>,
        proof_ref: String,
        proof_hash: BytesN<32>,
    ) -> Result<(), Error> {
        initiator.require_auth();
        Self::require_initialized(&env)?;

        let mut session: MPCSession = env
            .storage()
            .persistent()
            .get(&DataKey::Session(session_id.clone()))
            .ok_or(Error::SessionNotFound)?;
        Self::require_not_expired(&env, &session)?;
        if session.initiator != initiator {
            return Err(Error::NotAuthorized);
        }
        if session.status != SessionStatus::RevealPhase {
            return Err(Error::InvalidState);
        }
        if result_ref.is_empty() {
            return Err(Error::InvalidInput);
        }
        let zero_hash = BytesN::from_array(&env, &[0u8; 32]);
        if result_hash == zero_hash {
            return Err(Error::InvalidInput);
        }
        if session.reveals < session.threshold {
            return Err(Error::ThresholdNotMet);
        }

        if proof_ref.is_empty() {
            if proof_hash != zero_hash {
                return Err(Error::InvalidInput);
            }
        } else if proof_hash == zero_hash {
            return Err(Error::InvalidInput);
        }

        session.status = SessionStatus::Finalized;
        session.result_ref = result_ref;
        session.result_hash = result_hash;
        session.proof_ref = proof_ref;
        session.proof_hash = proof_hash;
        env.storage()
            .persistent()
            .set(&DataKey::Session(session_id.clone()), &session);

        env.events().publish(
            (symbol_short!("mpc"), symbol_short!("final")),
            (initiator, session_id),
        );
        Ok(())
    }

    pub fn get_session(env: Env, session_id: BytesN<32>) -> Result<Option<MPCSession>, Error> {
        Self::require_initialized(&env)?;
        Ok(env
            .storage()
            .persistent()
            .get(&DataKey::Session(session_id)))
    }

    pub fn get_commitment(
        env: Env,
        session_id: BytesN<32>,
        participant: Address,
    ) -> Result<Option<BytesN<32>>, Error> {
        Self::require_initialized(&env)?;
        Ok(env
            .storage()
            .persistent()
            .get(&DataKey::Commit(session_id, participant)))
    }

    pub fn get_reveal(
        env: Env,
        session_id: BytesN<32>,
        participant: Address,
    ) -> Result<Option<ShareReveal>, Error> {
        Self::require_initialized(&env)?;
        Ok(env
            .storage()
            .persistent()
            .get(&DataKey::Reveal(session_id, participant)))
    }

    /// Create Shamir's Secret Sharing shares for medical record encryption keys
    #[allow(clippy::too_many_arguments)]
    pub fn create_secret_shares(
        env: Env,
        participant: Address,
        session_id: BytesN<32>,
        secret: Bytes,
        num_shares: u32,
        threshold: u32,
    ) -> Result<Vec<SecretShare>, Error> {
        participant.require_auth();
        Self::require_initialized(&env)?;

        let session: MPCSession = env
            .storage()
            .persistent()
            .get(&DataKey::Session(session_id.clone()))
            .ok_or(Error::SessionNotFound)?;

        Self::require_not_expired(&env, &session)?;
        if session.status != SessionStatus::CommitPhase {
            return Err(Error::InvalidState);
        }
        if !session.participants.contains(&participant) {
            return Err(Error::NotAuthorized);
        }
        if num_shares < threshold || threshold == 0 {
            return Err(Error::InvalidInput);
        }

        let mut shares = Vec::new(&env);
        let now = env.ledger().timestamp();

        // Generate shares using simplified Shamir's Secret Sharing
        // In production, this would use proper finite field arithmetic
        for i in 1..=num_shares {
            let share_value = Self::generate_share_value(&env, &secret, i, threshold);
            let commitment = env.crypto().sha256(&share_value);

            let share = SecretShare {
                share_id: i,
                share_value,
                commitment: commitment.into(),
                created_at: now,
            };
            shares.push_back(share.clone());

            // Store share for participant
            let share_key = DataKey::SecretShare(session_id.clone(), participant.clone(), i);
            env.storage().persistent().set(&share_key, &share);
        }

        // Track gas usage
        Self::track_gas_usage(&env, &session_id, &participant, 15000);

        // Create audit entry
        Self::create_audit_entry(
            &env,
            &participant,
            String::from_str(&env, "create_shares"),
            &session_id,
            15000,
            Bytes::from_slice(&env, &num_shares.to_be_bytes()),
        );

        Ok(shares)
    }

    /// Submit a computation proof for verification
    pub fn submit_computation_proof(
        env: Env,
        participant: Address,
        session_id: BytesN<32>,
        proof: ComputationProof,
    ) -> Result<(), Error> {
        participant.require_auth();
        Self::require_initialized(&env)?;

        let mut session: MPCSession = env
            .storage()
            .persistent()
            .get(&DataKey::Session(session_id.clone()))
            .ok_or(Error::SessionNotFound)?;

        Self::require_not_expired(&env, &session)?;
        if session.status != SessionStatus::RevealPhase {
            return Err(Error::InvalidState);
        }
        if !session.participants.contains(&participant) {
            return Err(Error::NotAuthorized);
        }

        // Verify proof structure
        if proof.proof_data.is_empty() {
            return Err(Error::InvalidInput);
        }

        // Store proof
        env.storage()
            .persistent()
            .set(&DataKey::ComputationProof(session_id.clone()), &proof);

        // Update session gas tracking
        session.total_gas_used += proof.gas_used;
        env.storage()
            .persistent()
            .set(&DataKey::Session(session_id.clone()), &session);

        // Create audit entry
        Self::create_audit_entry(
            &env,
            &participant,
            String::from_str(&env, "submit_proof"),
            &session_id,
            proof.gas_used,
            Bytes::new(&env),
        );

        env.events().publish(
            (symbol_short!("mpc"), symbol_short!("proof")),
            (participant, session_id),
        );

        Ok(())
    }

    /// Perform privacy-preserving statistical analysis
    #[allow(clippy::too_many_arguments)]
    pub fn perform_statistical_analysis(
        env: Env,
        participant: Address,
        session_id: BytesN<32>,
        _analysis_type: String,
        encrypted_data: Bytes,
    ) -> Result<BytesN<32>, Error> {
        participant.require_auth();
        Self::require_initialized(&env)?;

        let session: MPCSession = env
            .storage()
            .persistent()
            .get(&DataKey::Session(session_id.clone()))
            .ok_or(Error::SessionNotFound)?;

        Self::require_not_expired(&env, &session)?;
        if session.computation_type != ComputationType::StatisticalAnalysis {
            return Err(Error::InvalidState);
        }
        if !session.participants.contains(&participant) {
            return Err(Error::NotAuthorized);
        }

        // Simulate statistical computation on encrypted data
        // In production, this would use homomorphic encryption or secure aggregation
        let result_hash = env.crypto().sha256(&encrypted_data);

        // Track gas usage (target: < 50,000 gas)
        Self::track_gas_usage(&env, &session_id, &participant, 35000);

        // Create audit entry
        Self::create_audit_entry(
            &env,
            &participant,
            String::from_str(&env, "statistical_analysis"),
            &session_id,
            35000,
            encrypted_data,
        );

        env.events().publish(
            (symbol_short!("mpc"), symbol_short!("stats")),
            (
                participant,
                session_id,
                BytesN::from_array(&env, &result_hash.to_array()),
            ),
        );

        Ok(result_hash.into())
    }

    /// Train machine learning model on encrypted data
    #[allow(clippy::too_many_arguments)]
    pub fn train_secure_ml_model(
        env: Env,
        participant: Address,
        session_id: BytesN<32>,
        model_params: Bytes,
        training_data: Bytes,
    ) -> Result<BytesN<32>, Error> {
        participant.require_auth();
        Self::require_initialized(&env)?;

        let session: MPCSession = env
            .storage()
            .persistent()
            .get(&DataKey::Session(session_id.clone()))
            .ok_or(Error::SessionNotFound)?;

        Self::require_not_expired(&env, &session)?;
        if session.computation_type != ComputationType::PrivacyPreservingML {
            return Err(Error::InvalidState);
        }
        if !session.participants.contains(&participant) {
            return Err(Error::NotAuthorized);
        }

        // Simulate secure ML training
        let mut combined_data = Bytes::new(&env);
        combined_data.append(&model_params);
        combined_data.append(&training_data);
        let model_hash = env.crypto().sha256(&combined_data);

        // Track gas usage
        Self::track_gas_usage(&env, &session_id, &participant, 45000);

        // Create audit entry
        Self::create_audit_entry(
            &env,
            &participant,
            String::from_str(&env, "ml_training"),
            &session_id,
            45000,
            combined_data,
        );

        env.events().publish(
            (symbol_short!("mpc"), symbol_short!("ml")),
            (
                participant,
                session_id,
                BytesN::from_array(&env, &model_hash.to_array()),
            ),
        );

        Ok(model_hash.into())
    }

    /// Get audit trail for a session
    pub fn get_audit_trail(env: Env, session_id: BytesN<32>) -> Result<Vec<AuditEntry>, Error> {
        Self::require_initialized(&env)?;

        let _session: MPCSession = env
            .storage()
            .persistent()
            .get(&DataKey::Session(session_id.clone()))
            .ok_or(Error::SessionNotFound)?;

        let mut audit_trail = Vec::new(&env);
        let audit_counter: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::AuditCounter)
            .unwrap_or(0);

        for i in 0..audit_counter {
            if let Some(entry) = env
                .storage()
                .persistent()
                .get::<DataKey, AuditEntry>(&DataKey::AuditEntry(i))
            {
                if entry.session_id == session_id {
                    audit_trail.push_back(entry);
                }
            }
        }

        Ok(audit_trail)
    }

    /// Get gas usage statistics for a session
    pub fn get_gas_stats(env: Env, session_id: BytesN<32>) -> Result<u64, Error> {
        Self::require_initialized(&env)?;

        let session: MPCSession = env
            .storage()
            .persistent()
            .get(&DataKey::Session(session_id))
            .ok_or(Error::SessionNotFound)?;

        Ok(session.total_gas_used)
    }

    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

    fn require_initialized(env: &Env) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Initialized) {
            Ok(())
        } else {
            Err(Error::NotInitialized)
        }
    }

    fn require_not_expired(env: &Env, session: &MPCSession) -> Result<(), Error> {
        let now = env.ledger().timestamp();
        if now > session.expires_at {
            Err(Error::SessionExpired)
        } else {
            Ok(())
        }
    }

    /// Generate share value using simplified Shamir's Secret Sharing
    fn generate_share_value(env: &Env, secret: &Bytes, share_id: u32, threshold: u32) -> Bytes {
        // Simplified implementation - in production use proper finite field arithmetic
        let mut share_data = Bytes::new(env);
        share_data.append(&Bytes::from_slice(env, &share_id.to_be_bytes()));
        share_data.append(&Bytes::from_slice(env, &threshold.to_be_bytes()));
        share_data.append(secret);

        // Apply simple transformation to create share
        let hash = env.crypto().sha256(&share_data);
        let mut result = Bytes::new(env);
        result.append(&Bytes::from_slice(env, &hash.to_array()));
        result.append(&Bytes::from_slice(env, &share_id.to_be_bytes()));
        result
    }

    /// Track gas usage for a participant in a session
    fn track_gas_usage(env: &Env, session_id: &BytesN<32>, participant: &Address, gas_used: u64) {
        let gas_key = DataKey::GasTracker(session_id.clone(), participant.clone());
        let current_gas: u64 = env.storage().persistent().get(&gas_key).unwrap_or(0);
        let total_gas = current_gas.saturating_add(gas_used);
        env.storage().persistent().set(&gas_key, &total_gas);
    }

    /// Create an audit entry for MPC operations
    fn create_audit_entry(
        env: &Env,
        participant: &Address,
        operation: String,
        session_id: &BytesN<32>,
        gas_used: u64,
        metadata: Bytes,
    ) {
        let audit_counter: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::AuditCounter)
            .unwrap_or(0);

        let entry = AuditEntry {
            participant: participant.clone(),
            operation,
            session_id: session_id.clone(),
            timestamp: env.ledger().timestamp(),
            gas_used,
            metadata,
        };

        env.storage()
            .persistent()
            .set(&DataKey::AuditEntry(audit_counter), &entry);
        env.storage()
            .persistent()
            .set(&DataKey::AuditCounter, &(audit_counter.saturating_add(1)));
    }
}
