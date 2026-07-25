#![no_std]
//! bridge_dispute_mediation - Dispute mediation state machine for cross-chain bridge failures.
//!
//! # State Machine Overview
//!
//! A dispute goes through the following states:
//!
//! ```
//! Open ──► UnderReview ──► Resolved (Upheld | Rejected)
//!   │            │
//!   │            └──► Escalated ──► Resolved
//!   │
//!   └──► Withdrawn (before mediator assignment)
//! ```
//!
//! Mediators are appointed by the admin and must be active to accept cases.
//! Quorum (`min_votes`) determines how many mediator votes are needed to resolve.
#![allow(clippy::too_many_arguments)]
#![allow(dead_code)]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, BytesN, Env, String, Symbol, Vec,
};

// ==================== Error Codes ====================

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // Lifecycle
    AlreadyInitialized = 1,
    NotInitialized     = 2,
    ContractPaused     = 3,

    // Auth / access
    Unauthorized       = 10,
    NotMediator        = 11,
    MediatorNotActive  = 12,
    MediatorExists     = 13,

    // Dispute state
    DisputeNotFound    = 20,
    InvalidTransition  = 21,
    AlreadyVoted       = 22,
    QuorumNotReached   = 23,
    DisputeNotOpen     = 24,

    // Arithmetic
    Overflow           = 90,
}

// ==================== Core Types ====================

/// Chains mirrored from cross_chain_bridge for dispute context.
#[derive(Clone, PartialEq, Eq, Debug)]
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

/// The failure category that prompted the dispute.
#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub enum BridgeFailureKind {
    /// A cross-chain message was lost or never relayed.
    MessageLost,
    /// An atomic transaction was partially executed across chains.
    AtomicTxPartial,
    /// A record sync failed mid-way, leaving chains inconsistent.
    RecordSyncFailure,
    /// Token transfer was debited on source but never credited on dest.
    TokenTransferStuck,
    /// An oracle reported conflicting data for the same block.
    OracleConflict,
    /// Validator misbehaved (e.g. double-signing).
    ValidatorMisbehavior,
    /// Any other bridge failure not covered above.
    Other,
}

/// The verdict that mediators vote towards.
#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub enum DisputeVerdict {
    /// The dispute claim is upheld; remediation should proceed.
    Upheld,
    /// The dispute claim is rejected; no remediation required.
    Rejected,
}

/// Lifecycle states of a dispute.
#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub enum DisputeState {
    /// Newly filed, awaiting mediator assignment.
    Open,
    /// A mediator has accepted the case and review is in progress.
    UnderReview,
    /// Escalated to full mediator panel after initial review stalls.
    Escalated,
    /// Resolved with a final verdict.
    Resolved,
    /// Withdrawn by the claimant before a mediator was assigned.
    Withdrawn,
}

/// A single mediator vote cast during resolution.
#[derive(Clone)]
#[contracttype]
pub struct MediatorVote {
    pub mediator:   Address,
    pub verdict:    DisputeVerdict,
    pub rationale:  String,
    pub voted_at:   u64,
}

/// Full dispute record stored on-chain.
#[derive(Clone)]
#[contracttype]
pub struct DisputeRecord {
    pub dispute_id:     BytesN<32>,
    pub claimant:       Address,
    pub source_chain:   ChainId,
    pub dest_chain:     ChainId,
    pub failure_kind:   BridgeFailureKind,
    /// ID of the bridge operation (message, tx, etc.) being disputed.
    pub operation_id:   BytesN<32>,
    pub description:    String,
    pub state:          DisputeState,
    pub assigned_to:    Option<Address>,
    pub filed_at:       u64,
    pub updated_at:     u64,
    pub resolved_at:    u64,
    pub final_verdict:  Option<DisputeVerdict>,
    pub votes:          Vec<MediatorVote>,
}

/// Mediator profile.
#[derive(Clone)]
#[contracttype]
pub struct MediatorInfo {
    pub address:          Address,
    pub is_active:        bool,
    pub cases_resolved:   u32,
    pub registered_at:    u64,
}

// ==================== Storage Keys ====================

#[contracttype]
pub enum DataKey {
    Admin,
    Paused,
    MinVotes,
    DisputeCount,
    MediatorCount,
    Dispute(BytesN<32>),
    Mediator(Address),
}

// ==================== TTL Constants ====================

const PERSISTENT_TTL_THRESHOLD: u32 = 100;
const PERSISTENT_TTL_EXTEND_TO: u32 = 10_000;

// ==================== Contract ====================

#[contract]
pub struct BridgeDisputeMediationContract;

#[contractimpl]
impl BridgeDisputeMediationContract {
    // ----------------------------------------------------------------
    // Storage helpers
    // ----------------------------------------------------------------

    fn persistent_set<T: soroban_sdk::IntoVal<Env, soroban_sdk::Val> + Clone>(
        env: &Env,
        key: &DataKey,
        val: &T,
    ) {
        env.storage().persistent().set(key, val);
        env.storage()
            .persistent()
            .extend_ttl(key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
    }

    fn persistent_get<T: soroban_sdk::TryFromVal<Env, soroban_sdk::Val> + Clone>(
        env: &Env,
        key: &DataKey,
    ) -> Option<T> {
        let val: Option<T> = env.storage().persistent().get(key);
        if val.is_some() {
            env.storage()
                .persistent()
                .extend_ttl(key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
        }
        val
    }

    // ----------------------------------------------------------------
    // Guards
    // ----------------------------------------------------------------

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if *caller != admin {
            return Err(Error::Unauthorized);
        }
        Ok(())
    }

    fn require_not_paused(env: &Env) -> Result<(), Error> {
        let paused: bool = env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false);
        if paused {
            return Err(Error::ContractPaused);
        }
        Ok(())
    }

    fn get_active_mediator(env: &Env, addr: &Address) -> Result<MediatorInfo, Error> {
        let key = DataKey::Mediator(addr.clone());
        let info: MediatorInfo =
            Self::persistent_get(env, &key).ok_or(Error::NotMediator)?;
        if !info.is_active {
            return Err(Error::MediatorNotActive);
        }
        Ok(info)
    }

    // ----------------------------------------------------------------
    // Initialise
    // ----------------------------------------------------------------

    /// Initialise the contract. Must be called once by the deployer.
    ///
    /// * `admin`     – sole account that can add/remove mediators and pause.
    /// * `min_votes` – minimum mediator votes required to resolve a dispute.
    pub fn initialize(env: Env, admin: Address, min_votes: u32) -> Result<(), Error> {
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage().instance().set(&DataKey::MinVotes, &min_votes);
        env.storage().instance().set(&DataKey::DisputeCount, &0u64);
        env.storage().instance().set(&DataKey::MediatorCount, &0u32);

        env.events()
            .publish((Symbol::new(&env, "Initialized"),), (admin,));
        Ok(())
    }

    // ----------------------------------------------------------------
    // Admin functions
    // ----------------------------------------------------------------

    /// Pause or unpause the contract.
    pub fn set_paused(env: Env, caller: Address, paused: bool) -> Result<(), Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        env.storage().instance().set(&DataKey::Paused, &paused);
        env.events()
            .publish((Symbol::new(&env, "PauseToggled"),), (paused,));
        Ok(())
    }

    /// Register a new mediator. Only the admin may call this.
    pub fn add_mediator(env: Env, caller: Address, mediator: Address) -> Result<(), Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        Self::require_not_paused(&env)?;

        let key = DataKey::Mediator(mediator.clone());
        if Self::persistent_get::<MediatorInfo>(&env, &key).is_some() {
            return Err(Error::MediatorExists);
        }

        let info = MediatorInfo {
            address: mediator.clone(),
            is_active: true,
            cases_resolved: 0,
            registered_at: env.ledger().timestamp(),
        };
        Self::persistent_set(&env, &key, &info);

        let count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::MediatorCount)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::MediatorCount, &count.saturating_add(1));

        env.events()
            .publish((Symbol::new(&env, "MediatorAdded"),), (mediator,));
        Ok(())
    }

    /// Deactivate an existing mediator.
    pub fn deactivate_mediator(env: Env, caller: Address, mediator: Address) -> Result<(), Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        let key = DataKey::Mediator(mediator.clone());
        let mut info: MediatorInfo =
            Self::persistent_get(&env, &key).ok_or(Error::NotMediator)?;
        info.is_active = false;
        Self::persistent_set(&env, &key, &info);

        env.events()
            .publish((Symbol::new(&env, "MediatorDeactivated"),), (mediator,));
        Ok(())
    }

    /// Query a mediator's profile.
    pub fn get_mediator(env: Env, mediator: Address) -> Result<MediatorInfo, Error> {
        Self::persistent_get(&env, &DataKey::Mediator(mediator)).ok_or(Error::NotMediator)
    }

    // ----------------------------------------------------------------
    // Dispute lifecycle — Open / Withdraw
    // ----------------------------------------------------------------

    /// File a new dispute. Caller must be the claimant.
    ///
    /// Returns the `dispute_id` stored on-chain.
    pub fn file_dispute(
        env: Env,
        claimant: Address,
        source_chain: ChainId,
        dest_chain: ChainId,
        failure_kind: BridgeFailureKind,
        operation_id: BytesN<32>,
        description: String,
    ) -> Result<BytesN<32>, Error> {
        claimant.require_auth();
        Self::require_not_paused(&env)?;

        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::DisputeCount)
            .unwrap_or(0);
        let new_count = count.checked_add(1).ok_or(Error::Overflow)?;

        // Derive a deterministic dispute_id from (operation_id, count, timestamp).
        let now = env.ledger().timestamp();
        let mut id_seed = operation_id.to_array();
        // XOR the low 8 bytes with the count so each filing is unique.
        let count_bytes = new_count.to_be_bytes();
        for (i, b) in count_bytes.iter().enumerate() {
            id_seed[i] ^= b;
        }
        let dispute_id: BytesN<32> =
            BytesN::from_array(&env, &id_seed);

        let record = DisputeRecord {
            dispute_id: dispute_id.clone(),
            claimant: claimant.clone(),
            source_chain,
            dest_chain,
            failure_kind,
            operation_id,
            description,
            state: DisputeState::Open,
            assigned_to: None,
            filed_at: now,
            updated_at: now,
            resolved_at: 0,
            final_verdict: None,
            votes: Vec::new(&env),
        };

        Self::persistent_set(&env, &DataKey::Dispute(dispute_id.clone()), &record);
        env.storage()
            .instance()
            .set(&DataKey::DisputeCount, &new_count);

        env.events().publish(
            (Symbol::new(&env, "DisputeFiled"),),
            (dispute_id.clone(), claimant),
        );
        Ok(dispute_id)
    }

    /// Withdraw an open dispute. Only the claimant may withdraw, and only
    /// while the dispute is still in `Open` state (no mediator yet).
    pub fn withdraw_dispute(
        env: Env,
        claimant: Address,
        dispute_id: BytesN<32>,
    ) -> Result<(), Error> {
        claimant.require_auth();
        Self::require_not_paused(&env)?;

        let key = DataKey::Dispute(dispute_id.clone());
        let mut record: DisputeRecord =
            Self::persistent_get(&env, &key).ok_or(Error::DisputeNotFound)?;

        if record.claimant != claimant {
            return Err(Error::Unauthorized);
        }
        if record.state != DisputeState::Open {
            return Err(Error::InvalidTransition);
        }

        record.state = DisputeState::Withdrawn;
        record.updated_at = env.ledger().timestamp();
        Self::persistent_set(&env, &key, &record);

        env.events().publish(
            (Symbol::new(&env, "DisputeWithdrawn"),),
            (dispute_id, claimant),
        );
        Ok(())
    }

    // ----------------------------------------------------------------
    // Dispute lifecycle — UnderReview / Escalated
    // ----------------------------------------------------------------

    /// A mediator accepts an open dispute, moving it to `UnderReview`.
    /// Only one mediator can be assigned at a time; subsequent mediators
    /// participate via `cast_vote` once the dispute is escalated.
    pub fn accept_dispute(
        env: Env,
        mediator: Address,
        dispute_id: BytesN<32>,
    ) -> Result<(), Error> {
        mediator.require_auth();
        Self::require_not_paused(&env)?;
        Self::get_active_mediator(&env, &mediator)?;

        let key = DataKey::Dispute(dispute_id.clone());
        let mut record: DisputeRecord =
            Self::persistent_get(&env, &key).ok_or(Error::DisputeNotFound)?;

        if record.state != DisputeState::Open {
            return Err(Error::InvalidTransition);
        }

        record.state = DisputeState::UnderReview;
        record.assigned_to = Some(mediator.clone());
        record.updated_at = env.ledger().timestamp();
        Self::persistent_set(&env, &key, &record);

        env.events().publish(
            (Symbol::new(&env, "DisputeAccepted"),),
            (dispute_id, mediator),
        );
        Ok(())
    }

    /// Escalate a dispute that is `UnderReview` to the full mediator panel.
    /// Either the assigned mediator or the admin may escalate.
    pub fn escalate_dispute(
        env: Env,
        caller: Address,
        dispute_id: BytesN<32>,
    ) -> Result<(), Error> {
        caller.require_auth();
        Self::require_not_paused(&env)?;

        let key = DataKey::Dispute(dispute_id.clone());
        let mut record: DisputeRecord =
            Self::persistent_get(&env, &key).ok_or(Error::DisputeNotFound)?;

        if record.state != DisputeState::UnderReview {
            return Err(Error::InvalidTransition);
        }

        // Only the assigned mediator or admin may escalate.
        let is_admin = env
            .storage()
            .instance()
            .get::<DataKey, Address>(&DataKey::Admin)
            .map(|a| a == caller)
            .unwrap_or(false);
        let is_assigned = record
            .assigned_to
            .as_ref()
            .map(|a| *a == caller)
            .unwrap_or(false);

        if !is_admin && !is_assigned {
            return Err(Error::Unauthorized);
        }

        record.state = DisputeState::Escalated;
        record.updated_at = env.ledger().timestamp();
        Self::persistent_set(&env, &key, &record);

        env.events().publish(
            (Symbol::new(&env, "DisputeEscalated"),),
            (dispute_id, caller),
        );
        Ok(())
    }

    // ----------------------------------------------------------------
    // Voting and resolution
    // ----------------------------------------------------------------

    /// Cast a mediator vote on a dispute that is `UnderReview` or `Escalated`.
    ///
    /// Each active mediator may vote exactly once per dispute. When the vote
    /// tally for a single verdict reaches `min_votes`, the dispute is
    /// automatically resolved.
    pub fn cast_vote(
        env: Env,
        mediator: Address,
        dispute_id: BytesN<32>,
        verdict: DisputeVerdict,
        rationale: String,
    ) -> Result<DisputeState, Error> {
        mediator.require_auth();
        Self::require_not_paused(&env)?;
        Self::get_active_mediator(&env, &mediator)?;

        let key = DataKey::Dispute(dispute_id.clone());
        let mut record: DisputeRecord =
            Self::persistent_get(&env, &key).ok_or(Error::DisputeNotFound)?;

        // Voting is allowed while UnderReview or Escalated.
        match record.state {
            DisputeState::UnderReview | DisputeState::Escalated => {},
            _ => return Err(Error::InvalidTransition),
        }

        // Prevent double-voting.
        for v in record.votes.iter() {
            if v.mediator == mediator {
                return Err(Error::AlreadyVoted);
            }
        }

        let vote = MediatorVote {
            mediator: mediator.clone(),
            verdict: verdict.clone(),
            rationale,
            voted_at: env.ledger().timestamp(),
        };
        record.votes.push_back(vote);
        record.updated_at = env.ledger().timestamp();

        // Tally votes for each verdict.
        let (upheld_count, rejected_count) = Self::tally_votes(&record.votes);

        let min_votes: u32 = env
            .storage()
            .instance()
            .get(&DataKey::MinVotes)
            .unwrap_or(1);

        // Auto-resolve when quorum is reached.
        if upheld_count >= min_votes {
            Self::apply_resolution(
                &env,
                &mut record,
                DisputeVerdict::Upheld,
                &mediator,
                &dispute_id,
            )?;
        } else if rejected_count >= min_votes {
            Self::apply_resolution(
                &env,
                &mut record,
                DisputeVerdict::Rejected,
                &mediator,
                &dispute_id,
            )?;
        }

        Self::persistent_set(&env, &key, &record);

        env.events().publish(
            (Symbol::new(&env, "VoteCast"),),
            (dispute_id, mediator, verdict),
        );

        Ok(record.state)
    }

    /// Manually resolve a dispute. The admin may call this to force-close a
    /// dispute that has enough votes but was not auto-resolved, or in
    /// exceptional administrative circumstances.
    pub fn resolve_dispute(
        env: Env,
        caller: Address,
        dispute_id: BytesN<32>,
        verdict: DisputeVerdict,
    ) -> Result<(), Error> {
        caller.require_auth();
        Self::require_not_paused(&env)?;
        Self::require_admin(&env, &caller)?;

        let key = DataKey::Dispute(dispute_id.clone());
        let mut record: DisputeRecord =
            Self::persistent_get(&env, &key).ok_or(Error::DisputeNotFound)?;

        match record.state {
            DisputeState::UnderReview | DisputeState::Escalated => {},
            _ => return Err(Error::InvalidTransition),
        }

        Self::apply_resolution(&env, &mut record, verdict, &caller, &dispute_id)?;
        Self::persistent_set(&env, &key, &record);
        Ok(())
    }

    // ----------------------------------------------------------------
    // Internal helpers
    // ----------------------------------------------------------------

    /// Count Upheld and Rejected votes in the current tally.
    fn tally_votes(votes: &Vec<MediatorVote>) -> (u32, u32) {
        let mut upheld: u32 = 0;
        let mut rejected: u32 = 0;
        for v in votes.iter() {
            match v.verdict {
                DisputeVerdict::Upheld   => upheld   = upheld.saturating_add(1),
                DisputeVerdict::Rejected => rejected = rejected.saturating_add(1),
            }
        }
        (upheld, rejected)
    }

    /// Transition a dispute record to `Resolved` and update mediator stats.
    fn apply_resolution(
        env: &Env,
        record: &mut DisputeRecord,
        verdict: DisputeVerdict,
        resolver: &Address,
        dispute_id: &BytesN<32>,
    ) -> Result<(), Error> {
        record.state = DisputeState::Resolved;
        record.final_verdict = Some(verdict.clone());
        let now = env.ledger().timestamp();
        record.resolved_at = now;
        record.updated_at = now;

        // Increment cases_resolved for the resolver (if they are a mediator).
        let med_key = DataKey::Mediator(resolver.clone());
        if let Some(mut info) = Self::persistent_get::<MediatorInfo>(env, &med_key) {
            info.cases_resolved = info.cases_resolved.saturating_add(1);
            Self::persistent_set(env, &med_key, &info);
        }

        env.events().publish(
            (Symbol::new(&env, "DisputeResolved"),),
            (dispute_id.clone(), verdict),
        );
        Ok(())
    }

    // ----------------------------------------------------------------
    // Queries
    // ----------------------------------------------------------------

    /// Fetch a dispute record by its ID.
    pub fn get_dispute(env: Env, dispute_id: BytesN<32>) -> Result<DisputeRecord, Error> {
        Self::persistent_get(&env, &DataKey::Dispute(dispute_id))
            .ok_or(Error::DisputeNotFound)
    }

    /// Return the current state of a dispute.
    pub fn get_dispute_state(env: Env, dispute_id: BytesN<32>) -> Result<DisputeState, Error> {
        let record: DisputeRecord =
            Self::persistent_get(&env, &DataKey::Dispute(dispute_id))
                .ok_or(Error::DisputeNotFound)?;
        Ok(record.state)
    }

    /// Return all votes recorded against a dispute.
    pub fn get_votes(env: Env, dispute_id: BytesN<32>) -> Result<Vec<MediatorVote>, Error> {
        let record: DisputeRecord =
            Self::persistent_get(&env, &DataKey::Dispute(dispute_id))
                .ok_or(Error::DisputeNotFound)?;
        Ok(record.votes)
    }

    /// Return the final verdict, or None if not yet resolved.
    pub fn get_verdict(env: Env, dispute_id: BytesN<32>) -> Result<Option<DisputeVerdict>, Error> {
        let record: DisputeRecord =
            Self::persistent_get(&env, &DataKey::Dispute(dispute_id))
                .ok_or(Error::DisputeNotFound)?;
        Ok(record.final_verdict)
    }

    /// Return the total number of disputes ever filed.
    pub fn dispute_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::DisputeCount)
            .unwrap_or(0)
    }

    /// Return the minimum votes required for resolution.
    pub fn min_votes(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::MinVotes)
            .unwrap_or(0)
    }
}
