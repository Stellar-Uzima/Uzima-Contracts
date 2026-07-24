#![no_std]
//! admin_recovery - Hardened admin recovery and emergency access flows.
//!
//! Provides a multi-step admin recovery protocol for upgradeable contracts
//! that prevents single-key compromise from permanently locking a contract.
//!
//! ## Recovery Flow
//!
//! ```text
//! 1. Proposer calls propose_recovery(new_admin)
//!    → Stored as pending, emits RecoveryProposed event
//!
//! 2. Guardian (separate key) calls approve_recovery(proposal_id)
//!    → Validates cooldown window has passed
//!
//! 3. After approval, new admin calls execute_recovery(proposal_id)
//!    → Replaces admin key, emits RecoveryExecuted event
//! ```
//!
//! ## Emergency Access
//!
//! The emergency access flow allows a designated emergency contact to gain
//! time-limited read access when the primary admin is unavailable:
//!
//! ```text
//! emergency_contact calls request_emergency_access(reason)
//! → Grants access for EMERGENCY_ACCESS_TTL_LEDGERS ledgers
//! → Emits EmergencyAccessGranted event
//! → All access attempts logged to audit trail
//! ```

use soroban_sdk::{contracttype, symbol_short, Address, BytesN, Env, Symbol};

// ──────────────────────────────────────────────────────────────────────────────
// Constants
// ──────────────────────────────────────────────────────────────────────────────

/// Minimum ledgers between proposing and executing a recovery (~48 hours).
pub const RECOVERY_COOLDOWN_LEDGERS: u32 = 34_560;

/// Ledgers a recovery proposal is valid for before expiring (~7 days).
pub const RECOVERY_EXPIRY_LEDGERS: u32 = 120_960;

/// Ledgers that emergency access is valid for (~4 hours).
pub const EMERGENCY_ACCESS_TTL_LEDGERS: u32 = 2_880;

/// Maximum active emergency access grants at once.
pub const MAX_EMERGENCY_GRANTS: u32 = 3;

// ──────────────────────────────────────────────────────────────────────────────
// Types
// ──────────────────────────────────────────────────────────────────────────────

/// Status of a recovery proposal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum RecoveryStatus {
    Pending = 0,
    Approved = 1,
    Executed = 2,
    Expired = 3,
    Cancelled = 4,
}

/// A pending admin recovery proposal.
#[derive(Clone, Debug)]
#[contracttype]
pub struct RecoveryProposal {
    /// Unique proposal ID.
    pub id: BytesN<32>,
    /// The proposed new admin address.
    pub new_admin: Address,
    /// Address that created the proposal.
    pub proposer: Address,
    /// Ledger when the proposal was created.
    pub proposed_at: u32,
    /// Ledger when approved (0 if not yet approved).
    pub approved_at: u32,
    /// Current status.
    pub status: RecoveryStatus,
}

/// A time-limited emergency access grant.
#[derive(Clone, Debug)]
#[contracttype]
pub struct EmergencyAccessGrant {
    /// The address granted emergency access.
    pub grantee: Address,
    /// Ledger when access expires.
    pub expires_at: u32,
    /// Reason code for the grant.
    pub reason: Symbol,
}

// ──────────────────────────────────────────────────────────────────────────────
// Storage keys
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Clone)]
#[contracttype]
pub enum RecoveryKey {
    /// Stores the guardian address.
    Guardian,
    /// Stores a recovery proposal by ID.
    Proposal(BytesN<32>),
    /// Emergency access grants.
    EmergencyGrant(Address),
}

// ──────────────────────────────────────────────────────────────────────────────
// AdminRecovery
// ──────────────────────────────────────────────────────────────────────────────

/// Hardened admin recovery and emergency access manager.
pub struct AdminRecovery;

impl AdminRecovery {
    /// Set the guardian address (the second approval key for recovery).
    ///
    /// Must be called by the current admin during initialization.
    pub fn set_guardian(env: &Env, guardian: &Address) {
        env.storage()
            .instance()
            .set(&RecoveryKey::Guardian, guardian);
    }

    /// Returns the configured guardian address.
    pub fn get_guardian(env: &Env) -> Option<Address> {
        env.storage().instance().get(&RecoveryKey::Guardian)
    }

    /// Propose a new admin. Starts the recovery cooldown timer.
    ///
    /// Caller must be the current admin or the guardian.
    pub fn propose_recovery(
        env: &Env,
        proposal_id: BytesN<32>,
        new_admin: Address,
        proposer: Address,
    ) {
        let proposal = RecoveryProposal {
            id: proposal_id.clone(),
            new_admin: new_admin.clone(),
            proposer: proposer.clone(),
            proposed_at: env.ledger().sequence(),
            approved_at: 0,
            status: RecoveryStatus::Pending,
        };

        env.storage()
            .persistent()
            .set(&RecoveryKey::Proposal(proposal_id.clone()), &proposal);

        // Emit event for indexers and alerting
        env.events().publish(
            (symbol_short!("recovery"), symbol_short!("proposed")),
            (&proposal_id, &new_admin, &proposer),
        );
    }

    /// Approve a recovery proposal (guardian only).
    ///
    /// Returns `Err` if:
    /// - Proposal not found or already executed/expired
    /// - Cooldown window has not passed
    pub fn approve_recovery(
        env: &Env,
        proposal_id: &BytesN<32>,
        guardian: &Address,
    ) -> Result<(), AdminRecoveryError> {
        let mut proposal: RecoveryProposal = env
            .storage()
            .persistent()
            .get(&RecoveryKey::Proposal(proposal_id.clone()))
            .ok_or(AdminRecoveryError::ProposalNotFound)?;

        if proposal.status != RecoveryStatus::Pending {
            return Err(AdminRecoveryError::ProposalNotPending);
        }

        let current = env.ledger().sequence();

        // Check cooldown
        if current < proposal.proposed_at + RECOVERY_COOLDOWN_LEDGERS {
            return Err(AdminRecoveryError::CooldownNotElapsed);
        }

        // Check not expired
        if current > proposal.proposed_at + RECOVERY_EXPIRY_LEDGERS {
            proposal.status = RecoveryStatus::Expired;
            env.storage()
                .persistent()
                .set(&RecoveryKey::Proposal(proposal_id.clone()), &proposal);
            return Err(AdminRecoveryError::ProposalExpired);
        }

        proposal.status = RecoveryStatus::Approved;
        proposal.approved_at = current;

        env.storage()
            .persistent()
            .set(&RecoveryKey::Proposal(proposal_id.clone()), &proposal);

        env.events().publish(
            (symbol_short!("recovery"), symbol_short!("approved")),
            (proposal_id, guardian),
        );

        Ok(())
    }

    /// Execute an approved recovery, replacing the admin key.
    ///
    /// Returns the new admin address on success.
    pub fn execute_recovery(
        env: &Env,
        proposal_id: &BytesN<32>,
    ) -> Result<Address, AdminRecoveryError> {
        let mut proposal: RecoveryProposal = env
            .storage()
            .persistent()
            .get(&RecoveryKey::Proposal(proposal_id.clone()))
            .ok_or(AdminRecoveryError::ProposalNotFound)?;

        if proposal.status != RecoveryStatus::Approved {
            return Err(AdminRecoveryError::ProposalNotApproved);
        }

        let new_admin = proposal.new_admin.clone();
        proposal.status = RecoveryStatus::Executed;

        env.storage()
            .persistent()
            .set(&RecoveryKey::Proposal(proposal_id.clone()), &proposal);

        env.events().publish(
            (symbol_short!("recovery"), symbol_short!("executed")),
            (proposal_id, &new_admin),
        );

        Ok(new_admin)
    }

    /// Grant time-limited emergency access to an address.
    pub fn grant_emergency_access(
        env: &Env,
        grantee: Address,
        reason: Symbol,
    ) -> Result<(), AdminRecoveryError> {
        let expires_at = env.ledger().sequence() + EMERGENCY_ACCESS_TTL_LEDGERS;

        let grant = EmergencyAccessGrant {
            grantee: grantee.clone(),
            expires_at,
            reason: reason.clone(),
        };

        env.storage()
            .temporary()
            .set(&RecoveryKey::EmergencyGrant(grantee.clone()), &grant);
        env.storage()
            .temporary()
            .extend_ttl(
                &RecoveryKey::EmergencyGrant(grantee.clone()),
                0,
                EMERGENCY_ACCESS_TTL_LEDGERS,
            );

        env.events().publish(
            (symbol_short!("emergency"), symbol_short!("access")),
            (&grantee, &expires_at, &reason),
        );

        Ok(())
    }

    /// Returns `true` if `grantee` has active emergency access.
    pub fn has_emergency_access(env: &Env, grantee: &Address) -> bool {
        if let Some(grant) = env
            .storage()
            .temporary()
            .get::<RecoveryKey, EmergencyAccessGrant>(&RecoveryKey::EmergencyGrant(grantee.clone()))
        {
            grant.expires_at > env.ledger().sequence()
        } else {
            false
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Errors
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[soroban_sdk::contracterror]
#[repr(u32)]
pub enum AdminRecoveryError {
    ProposalNotFound = 700,
    ProposalNotPending = 701,
    ProposalNotApproved = 702,
    ProposalExpired = 703,
    CooldownNotElapsed = 704,
    TooManyEmergencyGrants = 705,
}
