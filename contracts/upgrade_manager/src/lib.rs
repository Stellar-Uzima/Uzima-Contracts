#![no_std]
#![forbid(alloc)]
//! upgrade_manager - Healthcare smart contract on Stellar blockchain.

pub mod errors;
pub use errors::Error;
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Map, Symbol, Vec,
};
use upgradeability::UpgradeValidation;
// Shared multi-sig helpers (Phase 4 migration: see issue #830)
use governance_commons::{multi_sig, ApprovalStatus};

fn emit_governance_telemetry(env: &Env, actor: &Address, operation: &'static str) {
    use contract_monitoring::telemetry::{
        build_event, derive_trace_id, emit_telemetry_event, TelemetryEventType, TelemetrySeverity,
    };

    let trace_id = derive_trace_id(env, actor);
    let event = build_event(
        env,
        "upgrade_manager",
        "0.1.0",
        TelemetryEventType::StateTransition,
        TelemetrySeverity::Info,
        operation,
        "governance state transition",
        trace_id.clone(),
        trace_id,
    );
    emit_telemetry_event(env, &event);
}

#[cfg(test)]
mod test;

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct UpgradeProposal {
    pub target: Address,
    pub new_wasm_hash: BytesN<32>,
    pub new_version: u32,
    pub description: Symbol,
    pub proposer: Address,
    pub created_at: u64,
    pub executable_at: u64,
    pub executed: bool,
    pub canceled: bool,
    pub approvals: Vec<Address>,
    pub is_emergency: bool,
    // Storage gap — reserved for future upgrades. Never read, write, reorder or
    // remove these fields. To add a field later, append it before the gap and
    // consume one slot (decrement STORAGE_GAP_SLOTS in the same commit).
    pub storage_version: u32,
    pub reserved_1: u64,
    pub reserved_2: u64,
    pub reserved_3: u64,
    pub reserved_4: u64,
    pub reserved_5: u64,
}

#[contract]
pub struct UpgradeManager;

const PROPOSALS: Symbol = symbol_short!("PROPS");
const CONFIG: Symbol = symbol_short!("CONFIG");
const MIN_DELAY: u64 = 86400; // 24 hours
const REQUIRED_APPROVALS: u32 = 3;

/// Bumped only when the on-chain layout of a persisted state type changes.
pub const STORAGE_SCHEMA_VERSION: u32 = 1;
/// Number of reserved slots kept in persisted state for additive upgrades.
/// May only shrink as slots are consumed, never grow.
pub const STORAGE_GAP_SLOTS: u32 = 5;

// TTL constants for persistent storage management
const PERSISTENT_TTL_THRESHOLD: u32 = 100;
const PERSISTENT_TTL_EXTEND_TO: u32 = 10000;

#[contracttype]
pub struct Config {
    pub admin: Address,
    pub min_delay: u64,
    pub required_approvals: u32,
    pub validators: Vec<Address>,
    pub emergency_approvals: u32,
    // Storage gap — see UpgradeProposal. Append-only; consume slots on upgrade.
    pub storage_version: u32,
    pub reserved_1: u64,
    pub reserved_2: u64,
    pub reserved_3: u64,
    pub reserved_4: u64,
    pub reserved_5: u64,
}

// Minimal interface for target contracts
#[soroban_sdk::contractclient(name = "TargetContractClient")]
pub trait TargetContract {
    fn upgrade(env: Env, new_wasm_hash: BytesN<32>);
    fn validate_upgrade(env: Env, new_wasm_hash: BytesN<32>) -> UpgradeValidation;
}

#[contractimpl]
impl UpgradeManager {
    pub fn initialize(env: Env, admin: Address, validators: Vec<Address>) -> Result<(), Error> {
        governance_commons::try_init_guard(&env).map_err(|_| Error::AlreadyInitialized)?;
        let config = Config {
            admin,
            min_delay: MIN_DELAY,
            required_approvals: REQUIRED_APPROVALS,
            validators: validators.clone(),
            emergency_approvals: validators.len(), // Default to all validators for emergency
            storage_version: STORAGE_SCHEMA_VERSION,
            reserved_1: 0,
            reserved_2: 0,
            reserved_3: 0,
            reserved_4: 0,
            reserved_5: 0,
        };
        env.storage().instance().set(&CONFIG, &config);
        Ok(())
    }

    /// Read-only view of the persisted-state schema version. Zero before init.
    pub fn storage_version(env: Env) -> u32 {
        env.storage()
            .instance()
            .get::<_, Config>(&CONFIG)
            .map(|c| c.storage_version)
            .unwrap_or(0)
    }

    /// Number of reserved gap slots in upgradeable state types.
    pub fn storage_gap_slots(_env: Env) -> u32 {
        STORAGE_GAP_SLOTS
    }

    /// True exactly for the reserved gap range `0..STORAGE_GAP_SLOTS`.
    pub fn is_reserved_gap_slot(_env: Env, slot: u32) -> bool {
        slot < STORAGE_GAP_SLOTS
    }

    #[allow(clippy::too_many_arguments)]
    pub fn propose_upgrade(
        env: Env,
        proposer: Address,
        target: Address,
        new_wasm_hash: BytesN<32>,
        new_version: u32,
        description: Symbol,
        is_emergency: bool,
    ) -> Result<u64, Error> {
        proposer.require_auth();
        let config: Config = env
            .storage()
            .instance()
            .get(&CONFIG)
            .ok_or(Error::ConfigNotFound)?;

        let mut proposals: Map<u64, UpgradeProposal> = env
            .storage()
            .persistent()
            .get(&PROPOSALS)
            .unwrap_or(Map::new(&env));

        let id = proposals.len() as u64;
        let executable_at = if is_emergency {
            env.ledger().timestamp()
        } else {
            env.ledger()
                .timestamp()
                .checked_add(config.min_delay)
                .ok_or(Error::InvalidState)?
        };

        let proposal = UpgradeProposal {
            target,
            new_wasm_hash,
            new_version,
            description,
            proposer: proposer.clone(),
            created_at: env.ledger().timestamp(),
            executable_at,
            executed: false,
            canceled: false,
            approvals: Vec::new(&env),
            is_emergency,
            storage_version: STORAGE_SCHEMA_VERSION,
            reserved_1: 0,
            reserved_2: 0,
            reserved_3: 0,
            reserved_4: 0,
            reserved_5: 0,
        };

        proposals.set(id, proposal);
        env.storage().persistent().set(&PROPOSALS, &proposals);
        env.storage().persistent().extend_ttl(
            &PROPOSALS,
            PERSISTENT_TTL_THRESHOLD,
            PERSISTENT_TTL_EXTEND_TO,
        );

        emit_governance_telemetry(&env, &proposer, "upgrade_propose");
        env.events()
            .publish((symbol_short!("proposed"), id), proposer);
        Ok(id)
    }

    pub fn approve(env: Env, validator: Address, proposal_id: u64) -> Result<(), Error> {
        validator.require_auth();
        let config: Config = env
            .storage()
            .instance()
            .get(&CONFIG)
            .ok_or(Error::ConfigNotFound)?;

        // Delegate validator-membership check to the shared multi-sig helper.
        multi_sig::validate_approver(&validator, &config.validators)
            .map_err(|_| Error::NotAValidator)?;

        let mut proposals: Map<u64, UpgradeProposal> = env
            .storage()
            .persistent()
            .get(&PROPOSALS)
            .ok_or(Error::ProposalNotFound)?;

        let mut proposal = proposals.get(proposal_id).ok_or(Error::ProposalNotFound)?;

        // add_approval returns true when the approver is newly added,
        // false when the approver already approved (idempotent).
        if !multi_sig::add_approval(validator.clone(), &mut proposal.approvals) {
            return Err(Error::AlreadyApproved);
        }

        emit_governance_telemetry(&env, &validator, "upgrade_approve");
        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&PROPOSALS, &proposals);
        env.storage().persistent().extend_ttl(
            &PROPOSALS,
            PERSISTENT_TTL_THRESHOLD,
            PERSISTENT_TTL_EXTEND_TO,
        );
        Ok(())
    }

    pub fn execute(env: Env, proposal_id: u64) -> Result<(), Error> {
        let mut proposals: Map<u64, UpgradeProposal> = env
            .storage()
            .persistent()
            .get(&PROPOSALS)
            .ok_or(Error::ProposalNotFound)?;

        let mut proposal = proposals.get(proposal_id).ok_or(Error::ProposalNotFound)?;
        let config: Config = env
            .storage()
            .instance()
            .get(&CONFIG)
            .ok_or(Error::ConfigNotFound)?;

        if proposal.executed || proposal.canceled {
            return Err(Error::InvalidState);
        }

        if env.ledger().timestamp() < proposal.executable_at {
            return Err(Error::TimelockNotExpired);
        }

        // Use shared approval status check rather than ad-hoc length compare.
        // `proposal.executed` is guaranteed false at this point because we
        // early-returned on `proposal.executed || proposal.canceled` above,
        // so we pass `false` explicitly for the executed flag.
        if multi_sig::check_approval_status(&proposal.approvals, config.required_approvals, false)
            != ApprovalStatus::Ready
        {
            return Err(Error::NotEnoughApprovals);
        }

        emit_governance_telemetry(&env, &proposal.proposer, "upgrade_execute");

        // Call target.upgrade(new_wasm_hash)
        let target_client = TargetContractClient::new(&env, &proposal.target);
        target_client.upgrade(&proposal.new_wasm_hash);

        proposal.executed = true;
        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&PROPOSALS, &proposals);
        env.storage().persistent().extend_ttl(
            &PROPOSALS,
            PERSISTENT_TTL_THRESHOLD,
            PERSISTENT_TTL_EXTEND_TO,
        );

        env.events()
            .publish((symbol_short!("executed"), proposal_id), ());
        Ok(())
    }

    pub fn execute_emergency(env: Env, proposal_id: u64) -> Result<(), Error> {
        let mut proposals: Map<u64, UpgradeProposal> = env
            .storage()
            .persistent()
            .get(&PROPOSALS)
            .ok_or(Error::ProposalNotFound)?;

        let mut proposal = proposals.get(proposal_id).ok_or(Error::ProposalNotFound)?;
        let config: Config = env
            .storage()
            .instance()
            .get(&CONFIG)
            .ok_or(Error::ConfigNotFound)?;

        if !proposal.is_emergency {
            return Err(Error::InvalidState);
        }

        if proposal.executed || proposal.canceled {
            return Err(Error::InvalidState);
        }

        // Emergency path requires unanimous validator approvals.
        // `proposal.executed` is guaranteed false (early-returned above).
        if multi_sig::check_approval_status(&proposal.approvals, config.emergency_approvals, false)
            != ApprovalStatus::Ready
        {
            return Err(Error::NotEnoughApprovals);
        }

        emit_governance_telemetry(&env, &proposal.proposer, "upgrade_execute_emergency");

        let target_client = TargetContractClient::new(&env, &proposal.target);
        target_client.upgrade(&proposal.new_wasm_hash);

        proposal.executed = true;
        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&PROPOSALS, &proposals);
        env.storage().persistent().extend_ttl(
            &PROPOSALS,
            PERSISTENT_TTL_THRESHOLD,
            PERSISTENT_TTL_EXTEND_TO,
        );

        env.events()
            .publish((symbol_short!("emergency"), proposal_id), ());
        Ok(())
    }

    pub fn validate_proposal(env: Env, proposal_id: u64) -> Result<UpgradeValidation, Error> {
        let proposals: Map<u64, UpgradeProposal> = env
            .storage()
            .persistent()
            .get(&PROPOSALS)
            .ok_or(Error::ProposalNotFound)?;

        let proposal = proposals.get(proposal_id).ok_or(Error::ProposalNotFound)?;

        let target_client = TargetContractClient::new(&env, &proposal.target);
        Ok(target_client.validate_upgrade(&proposal.new_wasm_hash))
    }
}
