#![no_std]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
    String, Symbol, Vec,
};

// =============================================================================
// Types
// =============================================================================

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
}

#[contracttype]
pub enum DataKey {
    Initialized,
    Admin,
    Session(BytesN<32>),
    Commit(BytesN<32>, Address),
    Reveal(BytesN<32>, Address),
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

    pub fn start_session(
        env: Env,
        initiator: Address,
        session_id: BytesN<32>,
        participants: Vec<Address>,
        threshold: u32,
        purpose: String,
        ttl_secs: u64,
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
}
