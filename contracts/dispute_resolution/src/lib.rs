#![no_std]
#![forbid(alloc)]
//! # Dispute Resolution Contract
//!
//! Structured dispute lifecycle for healthcare payment claims.
//! Supports filing, evidence submission, arbiter deliberation, and resolution
//! with timeout-based escalation.
//!
//! ## Lifecycle
//! ```text
//! Filed → UnderReview → EvidencePhase → Deliberation → Resolved | Escalated | Closed
//! ```
//!
//! ## Roles
//! - **Filer**: Patient, provider, or payer who initiates the dispute
//! - **Arbiter**: Neutral party assigned to investigate and resolve
//! - **Admin**: Contract administrator who manages arbiters

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Map, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    NotArbiter = 2,
    DisputeNotFound = 3,
    InvalidStateTransition = 4,
    Unauthorized = 5,
    DeadlineNotElapsed = 6,
    AlreadyDisputed = 7,
    EvidenceLimitExceeded = 8,
    InvalidOutcome = 9,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::NotInitialized => write!(f, "not initialized"),
            Error::NotArbiter => write!(f, "not arbiter"),
            Error::DisputeNotFound => write!(f, "dispute not found"),
            Error::InvalidStateTransition => write!(f, "invalid state transition"),
            Error::Unauthorized => write!(f, "unauthorized"),
            Error::DeadlineNotElapsed => write!(f, "deadline not elapsed"),
            Error::AlreadyDisputed => write!(f, "already disputed"),
            Error::EvidenceLimitExceeded => write!(f, "evidence limit exceeded"),
            Error::InvalidOutcome => write!(f, "invalid outcome"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum DisputeStatus {
    Filed = 0,
    UnderReview = 1,
    EvidencePhase = 2,
    Deliberation = 3,
    ResolvedInFavorOfFiler = 4,
    ResolvedAgainstFiler = 5,
    Escalated = 6,
    ClosedWithoutResolution = 7,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum DisputeOutcome {
    InFavorOfFiler = 0,
    AgainstFiler = 1,
    Withdrawn = 2,
    Dismissed = 3,
}

#[derive(Clone)]
#[contracttype]
pub struct Dispute {
    pub dispute_id: u64,
    pub claim_id: u64,
    pub filer: Address,
    pub respondent: Address,
    pub filed_at: u64,
    pub deadline: u64,
    pub status: DisputeStatus,
    pub assigned_arbiter: Option<Address>,
    pub evidence_count: u32,
    pub max_evidence: u32,
    pub resolution_notes: Option<String>,
    pub outcome: Option<DisputeOutcome>,
    pub resolved_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct EvidenceEntry {
    pub submitter: Address,
    pub evidence_type: String,
    pub content_hash: String,
    pub submitted_at: u64,
    pub notes: String,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Initialized,
    Admin,
    Arbiters,
    NextDisputeId,
    Dispute(u64),
    DisputeEvidence(u64),
    ClaimDispute(u64),
    EvidenceCount(u64),
}

const MAX_EVIDENCE_PER_DISPUTE: u32 = 20;
const DEFAULT_DISPUTE_DEADLINE_SECS: u64 = 2_592_000; // 30 days

#[contract]
pub struct DisputeResolution;

#[contractimpl]
impl DisputeResolution {
    /// Initialize with an admin and list of arbiters.
    pub fn initialize(env: Env, admin: Address, arbiters: Vec<Address>) {
        governance_commons::init_guard(&env);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::Arbiters, &arbiters);
        env.storage()
            .instance()
            .set(&DataKey::Initialized, &true);
        env.storage()
            .instance()
            .set(&DataKey::NextDisputeId, &1u64);
    }

    /// File a new dispute for a healthcare payment claim.
    pub fn file_dispute(
        env: Env,
        filer: Address,
        claim_id: u64,
        respondent: Address,
    ) -> Result<u64, Error> {
        filer.require_auth();
        Self::require_initialized(&env)?;

        // Check no active dispute for this claim
        let existing: Option<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::ClaimDispute(claim_id));
        if existing.is_some() {
            return Err(Error::AlreadyDisputed);
        }

        let dispute_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextDisputeId)
            .unwrap_or(1);
        let now = env.ledger().timestamp();

        let dispute = Dispute {
            dispute_id,
            claim_id,
            filer: filer.clone(),
            respondent: respondent.clone(),
            filed_at: now,
            deadline: now.saturating_add(DEFAULT_DISPUTE_DEADLINE_SECS),
            status: DisputeStatus::Filed,
            assigned_arbiter: None,
            evidence_count: 0,
            max_evidence: MAX_EVIDENCE_PER_DISPUTE,
            resolution_notes: None,
            outcome: None,
            resolved_at: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Dispute(dispute_id), &dispute);
        env.storage()
            .persistent()
            .set(&DataKey::ClaimDispute(claim_id), &dispute_id);
        env.storage()
            .persistent()
            .set(&DataKey::EvidenceCount(dispute_id), &0u32);
        env.storage()
            .instance()
            .set(&DataKey::NextDisputeId, &dispute_id.saturating_add(1));

        env.events().publish(
            (symbol_short!("DISP"), symbol_short!("FILED")),
            (dispute_id, claim_id, filer, respondent, now),
        );

        Ok(dispute_id)
    }

    /// Move dispute to UnderReview and assign an arbiter.
    pub fn assign_arbiter(
        env: Env,
        arbiter: Address,
        dispute_id: u64,
    ) -> Result<(), Error> {
        arbiter.require_auth();
        Self::require_initialized(&env)?;
        Self::require_arbiter(&env, &arbiter)?;

        let mut dispute = Self::get_dispute(&env, dispute_id)?;

        if dispute.status != DisputeStatus::Filed {
            return Err(Error::InvalidStateTransition);
        }

        dispute.status = DisputeStatus::UnderReview;
        dispute.assigned_arbiter = Some(arbiter.clone());

        env.storage()
            .persistent()
            .set(&DataKey::Dispute(dispute_id), &dispute);

        env.events().publish(
            (symbol_short!("DISP"), symbol_short!("REVIEW")),
            (dispute_id, arbiter),
        );

        Ok(())
    }

    /// Move dispute to evidence submission phase.
    pub fn open_evidence_phase(
        env: Env,
        arbiter: Address,
        dispute_id: u64,
    ) -> Result<(), Error> {
        arbiter.require_auth();
        Self::require_initialized(&env)?;
        Self::require_arbiter(&env, &arbiter)?;

        let mut dispute = Self::get_dispute(&env, dispute_id)?;
        Self::require_assigned_arbiter(&dispute, &arbiter)?;

        if dispute.status != DisputeStatus::UnderReview {
            return Err(Error::InvalidStateTransition);
        }

        dispute.status = DisputeStatus::EvidencePhase;
        env.storage()
            .persistent()
            .set(&DataKey::Dispute(dispute_id), &dispute);

        env.events().publish(
            (symbol_short!("DISP"), symbol_short!("EVIDENCE")),
            (dispute_id,),
        );

        Ok(())
    }

    /// Submit evidence for a dispute in EvidencePhase.
    pub fn submit_evidence(
        env: Env,
        submitter: Address,
        dispute_id: u64,
        evidence_type: String,
        content_hash: String,
        notes: String,
    ) -> Result<u32, Error> {
        submitter.require_auth();
        Self::require_initialized(&env)?;

        let mut dispute = Self::get_dispute(&env, dispute_id)?;

        if dispute.status != DisputeStatus::EvidencePhase {
            return Err(Error::InvalidStateTransition);
        }

        if submitter != dispute.filer && submitter != dispute.respondent {
            return Err(Error::Unauthorized);
        }

        if dispute.evidence_count >= dispute.max_evidence {
            return Err(Error::EvidenceLimitExceeded);
        }

        let entry = EvidenceEntry {
            submitter: submitter.clone(),
            evidence_type,
            content_hash,
            submitted_at: env.ledger().timestamp(),
            notes,
        };

        let mut evidence: Vec<EvidenceEntry> = env
            .storage()
            .persistent()
            .get(&DataKey::DisputeEvidence(dispute_id))
            .unwrap_or(Vec::new(&env));
        evidence.push_back(entry);
        dispute.evidence_count = dispute.evidence_count.saturating_add(1);

        env.storage()
            .persistent()
            .set(&DataKey::DisputeEvidence(dispute_id), &evidence);
        env.storage()
            .persistent()
            .set(&DataKey::Dispute(dispute_id), &dispute);

        env.events().publish(
            (symbol_short!("DISP"), symbol_short!("EVD_SUB")),
            (dispute_id, submitter),
        );

        Ok(dispute.evidence_count)
    }

    /// Move dispute to deliberation phase.
    pub fn begin_deliberation(
        env: Env,
        arbiter: Address,
        dispute_id: u64,
    ) -> Result<(), Error> {
        arbiter.require_auth();
        Self::require_initialized(&env)?;
        Self::require_arbiter(&env, &arbiter)?;

        let mut dispute = Self::get_dispute(&env, dispute_id)?;
        Self::require_assigned_arbiter(&dispute, &arbiter)?;

        if dispute.status != DisputeStatus::EvidencePhase {
            return Err(Error::InvalidStateTransition);
        }

        dispute.status = DisputeStatus::Deliberation;
        env.storage()
            .persistent()
            .set(&DataKey::Dispute(dispute_id), &dispute);

        env.events().publish(
            (symbol_short!("DISP"), symbol_short!("DELIB")),
            (dispute_id,),
        );

        Ok(())
    }

    /// Resolve a dispute with a structured outcome.
    pub fn resolve_dispute(
        env: Env,
        arbiter: Address,
        dispute_id: u64,
        outcome: DisputeOutcome,
        notes: String,
    ) -> Result<(), Error> {
        arbiter.require_auth();
        Self::require_initialized(&env)?;
        Self::require_arbiter(&env, &arbiter)?;

        let mut dispute = Self::get_dispute(&env, dispute_id)?;
        Self::require_assigned_arbiter(&dispute, &arbiter)?;

        if dispute.status != DisputeStatus::Deliberation
            && dispute.status != DisputeStatus::EvidencePhase
        {
            return Err(Error::InvalidStateTransition);
        }

        let now = env.ledger().timestamp();
        dispute.resolution_notes = Some(notes);
        dispute.outcome = Some(outcome);
        dispute.resolved_at = now;

        dispute.status = match outcome {
            DisputeOutcome::InFavorOfFiler => DisputeStatus::ResolvedInFavorOfFiler,
            DisputeOutcome::AgainstFiler => DisputeStatus::ResolvedAgainstFiler,
            DisputeOutcome::Withdrawn | DisputeOutcome::Dismissed => {
                DisputeStatus::ClosedWithoutResolution
            }
        };

        env.storage()
            .persistent()
            .set(&DataKey::Dispute(dispute_id), &dispute);

        env.events().publish(
            (symbol_short!("DISP"), symbol_short!("RESOLVED")),
            (dispute_id, outcome as u32, now),
        );

        Ok(())
    }

    /// Escalate a dispute (e.g., if deadline passed without resolution).
    pub fn escalate_dispute(
        env: Env,
        caller: Address,
        dispute_id: u64,
    ) -> Result<(), Error> {
        caller.require_auth();
        Self::require_initialized(&env)?;

        let mut dispute = Self::get_dispute(&env, dispute_id)?;

        // Only filer, respondent, or admin can escalate
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if caller != dispute.filer && caller != dispute.respondent && caller != admin {
            return Err(Error::Unauthorized);
        }

        let now = env.ledger().timestamp();

        // Can escalate if deadline passed or dispute is in review/deliberation
        if dispute.status != DisputeStatus::Filed
            && dispute.status != DisputeStatus::UnderReview
            && dispute.status != DisputeStatus::Deliberation
            && now < dispute.deadline
        {
            return Err(Error::InvalidStateTransition);
        }

        dispute.status = DisputeStatus::Escalated;
        dispute.resolved_at = now;
        env.storage()
            .persistent()
            .set(&DataKey::Dispute(dispute_id), &dispute);

        env.events().publish(
            (symbol_short!("DISP"), symbol_short!("ESCAL")),
            (dispute_id, caller, now),
        );

        Ok(())
    }

    /// Close a dispute without resolution (by admin or mutual agreement).
    pub fn close_dispute(
        env: Env,
        caller: Address,
        dispute_id: u64,
    ) -> Result<(), Error> {
        caller.require_auth();
        Self::require_initialized(&env)?;

        let mut dispute = Self::get_dispute(&env, dispute_id)?;

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;

        if caller != dispute.filer && caller != dispute.respondent && caller != admin {
            return Err(Error::Unauthorized);
        }

        if dispute.status as u32 >= DisputeStatus::ResolvedInFavorOfFiler as u32 {
            return Err(Error::InvalidStateTransition);
        }

        dispute.status = DisputeStatus::ClosedWithoutResolution;
        dispute.resolved_at = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::Dispute(dispute_id), &dispute);

        env.events().publish(
            (symbol_short!("DISP"), symbol_short!("CLOSED")),
            (dispute_id, caller),
        );

        Ok(())
    }

    /// Check if a claim is currently disputed.
    pub fn is_disputed(env: Env, claim_id: u64) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::ClaimDispute(claim_id))
            .is_some()
    }

    /// Get dispute details.
    pub fn get_dispute(env: Env, dispute_id: u64) -> Result<Dispute, Error> {
        Self::require_initialized(&env)?;
        Self::get_dispute_inner(&env, dispute_id)
    }

    /// Get evidence for a dispute.
    pub fn get_evidence(env: Env, dispute_id: u64) -> Result<Vec<EvidenceEntry>, Error> {
        Self::require_initialized(&env)?;
        let _ = Self::get_dispute_inner(&env, dispute_id)?;
        Ok(env
            .storage()
            .persistent()
            .get(&DataKey::DisputeEvidence(dispute_id))
            .unwrap_or(Vec::new(&env)))
    }

    /// Get the dispute ID for a claim.
    pub fn get_claim_dispute(env: Env, claim_id: u64) -> Option<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::ClaimDispute(claim_id))
    }

    // ── Internal helpers ────────────────────────────────────────────────

    fn require_initialized(env: &Env) -> Result<(), Error> {
        if env
            .storage()
            .instance()
            .get::<DataKey, bool>(&DataKey::Initialized)
            .unwrap_or(false)
        {
            Ok(())
        } else {
            Err(Error::NotInitialized)
        }
    }

    fn require_arbiter(env: &Env, arbiter: &Address) -> Result<(), Error> {
        let arbiters: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Arbiters)
            .ok_or(Error::NotInitialized)?;
        if arbiters.contains(arbiter) {
            Ok(())
        } else {
            Err(Error::NotArbiter)
        }
    }

    fn get_dispute_inner(env: &Env, dispute_id: u64) -> Result<Dispute, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Dispute(dispute_id))
            .ok_or(Error::DisputeNotFound)
    }

    fn require_assigned_arbiter(dispute: &Dispute, arbiter: &Address) -> Result<(), Error> {
        match &dispute.assigned_arbiter {
            Some(a) if a == arbiter => Ok(()),
            _ => Err(Error::Unauthorized),
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use soroban_sdk::testutils::Address as _;

    fn setup() -> (Env, Address, Address, Address) {
        let env = Env::default();
        let admin = Address::generate(&env);
        let arbiter = Address::generate(&env);
        let filer = Address::generate(&env);
        env.mock_all_auths();

        let arbiters = Vec::from_array(&env, [arbiter.clone()]);
        env.as_contract(&env.register_contract(None, DisputeResolution), || {
            governance_commons::init_guard(&env);
            env.storage().instance().set(&DataKey::Admin, &admin);
            env.storage().instance().set(&DataKey::Arbiters, &arbiters);
            env.storage().instance().set(&DataKey::Initialized, &true);
            env.storage().instance().set(&DataKey::NextDisputeId, &1u64);
        });

        (env, admin, arbiter, filer)
    }

    #[test]
    fn test_full_lifecycle() {
        let (env, _admin, arbiter, filer) = setup();
        let contract_id = env.register_contract(None, DisputeResolution);
        let respondent = Address::generate(&env);

        let dispute_id = DisputeResolution::file_dispute(
            env.clone(),
            filer.clone(),
            100,
            respondent.clone(),
        )
        .unwrap();
        assert_eq!(dispute_id, 1);

        assert!(DisputeResolution::is_disputed(env.clone(), 100));

        DisputeResolution::assign_arbiter(env.clone(), arbiter.clone(), dispute_id).unwrap();

        DisputeResolution::open_evidence_phase(env.clone(), arbiter.clone(), dispute_id).unwrap();

        let count = DisputeResolution::submit_evidence(
            env.clone(),
            filer.clone(),
            dispute_id,
            String::from_str(&env, "document"),
            String::from_str(&env, "hash1"),
            String::from_str(&env, "billing record"),
        )
        .unwrap();
        assert_eq!(count, 1);

        DisputeResolution::begin_deliberation(env.clone(), arbiter.clone(), dispute_id).unwrap();

        DisputeResolution::resolve_dispute(
            env.clone(),
            arbiter.clone(),
            dispute_id,
            DisputeOutcome::InFavorOfFiler,
            String::from_str(&env, "Valid claim"),
        )
        .unwrap();

        let d = DisputeResolution::get_dispute(env.clone(), dispute_id).unwrap();
        assert_eq!(d.status, DisputeStatus::ResolvedInFavorOfFiler);
    }
}
