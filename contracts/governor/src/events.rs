//! # Governor Contract Events Module
//!
//! Comprehensive event emission for governance operations.
//! Critical for tracking proposals, votes, and execution.

use soroban_sdk::{contracttype, symbol_short, Address, Env};

// ── Event Type Definitions ─────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum EventType {
    /// Contract initialized
    Initialized,
    /// Proposal created
    ProposalCreated,
    /// Vote cast on proposal
    VoteCast,
    /// Proposal queued for execution
    ProposalQueued,
    /// Proposal executed
    ProposalExecuted,
    /// Proposal canceled
    ProposalCanceled,
    /// Proposal disputed
    ProposalDisputed,
    /// Voting period ended
    VotingEnded,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum OperationCategory {
    RecordOperations,
    Administrative,
    System,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum VoteType {
    Against = 0,
    For = 1,
    Abstain = 2,
}

// ── Event Data Structures ──────────────────────────────────────────────────

#[derive(Clone)]
#[contracttype]
pub struct GovernorEventData {
    /// Proposal ID
    pub proposal_id: u64,
    /// Proposer address
    pub proposer: Option<Address>,
    /// Voter address (for vote events)
    pub voter: Option<Address>,
    /// Vote type (0=Against, 1=For, 2=Abstain)
    pub vote_type: Option<u32>,
    /// Voting power weight
    pub weight: Option<i128>,
    /// Current vote counts
    pub for_votes: Option<i128>,
    pub against_votes: Option<i128>,
    pub abstain_votes: Option<i128>,
}

#[derive(Clone)]
#[contracttype]
pub struct GovernorEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: GovernorEventData,
}

// ── Event Emission Functions ───────────────────────────────────────────────

/// Emit Initialized event
pub fn emit_initialized(env: &Env, caller: Address) {
    let event = GovernorEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller,
        block_height: env.ledger().sequence() as u64,
        data: GovernorEventData {
            proposal_id: 0,
            proposer: None,
            voter: None,
            vote_type: None,
            weight: None,
            for_votes: None,
            against_votes: None,
            abstain_votes: None,
        },
    };
    env.events()
        .publish((symbol_short!("GOV"), symbol_short!("INIT")), event);
}

/// Emit ProposalCreated event
pub fn emit_proposal_created(env: &Env, proposal_id: u64, proposer: Address, voting_delay: u64) {
    let event = GovernorEvent {
        event_type: EventType::ProposalCreated,
        category: OperationCategory::RecordOperations,
        timestamp: env.ledger().timestamp(),
        user_id: proposer.clone(),
        block_height: env.ledger().sequence() as u64,
        data: GovernorEventData {
            proposal_id,
            proposer: Some(proposer),
            voter: None,
            vote_type: None,
            weight: Some(voting_delay as i128),
            for_votes: None,
            against_votes: None,
            abstain_votes: None,
        },
    };
    env.events()
        .publish((symbol_short!("GOV"), symbol_short!("PROP")), event);
}

/// Emit VoteCast event
pub fn emit_vote_cast(
    env: &Env,
    proposal_id: u64,
    voter: Address,
    vote_type: u32,
    weight: i128,
) {
    let event = GovernorEvent {
        event_type: EventType::VoteCast,
        category: OperationCategory::RecordOperations,
        timestamp: env.ledger().timestamp(),
        user_id: voter.clone(),
        block_height: env.ledger().sequence() as u64,
        data: GovernorEventData {
            proposal_id,
            proposer: None,
            voter: Some(voter),
            vote_type: Some(vote_type),
            weight: Some(weight),
            for_votes: None,
            against_votes: None,
            abstain_votes: None,
        },
    };
    env.events()
        .publish((symbol_short!("GOV"), symbol_short!("VOTE")), event);
}

/// Emit ProposalQueued event
pub fn emit_proposal_queued(env: &Env, proposal_id: u64, caller: Address) {
    let event = GovernorEvent {
        event_type: EventType::ProposalQueued,
        category: OperationCategory::RecordOperations,
        timestamp: env.ledger().timestamp(),
        user_id: caller,
        block_height: env.ledger().sequence() as u64,
        data: GovernorEventData {
            proposal_id,
            proposer: None,
            voter: None,
            vote_type: None,
            weight: None,
            for_votes: None,
            against_votes: None,
            abstain_votes: None,
        },
    };
    env.events()
        .publish((symbol_short!("GOV"), symbol_short!("QUEUE")), event);
}

/// Emit ProposalExecuted event
pub fn emit_proposal_executed(env: &Env, proposal_id: u64, caller: Address) {
    let event = GovernorEvent {
        event_type: EventType::ProposalExecuted,
        category: OperationCategory::RecordOperations,
        timestamp: env.ledger().timestamp(),
        user_id: caller,
        block_height: env.ledger().sequence() as u64,
        data: GovernorEventData {
            proposal_id,
            proposer: None,
            voter: None,
            vote_type: None,
            weight: None,
            for_votes: None,
            against_votes: None,
            abstain_votes: None,
        },
    };
    env.events()
        .publish((symbol_short!("GOV"), symbol_short!("EXEC")), event);
}

/// Emit ProposalCanceled event
pub fn emit_proposal_canceled(env: &Env, proposal_id: u64, caller: Address) {
    let event = GovernorEvent {
        event_type: EventType::ProposalCanceled,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller,
        block_height: env.ledger().sequence() as u64,
        data: GovernorEventData {
            proposal_id,
            proposer: None,
            voter: None,
            vote_type: None,
            weight: None,
            for_votes: None,
            against_votes: None,
            abstain_votes: None,
        },
    };
    env.events()
        .publish((symbol_short!("GOV"), symbol_short!("CANC")), event);
}

/// Emit ProposalDisputed event
pub fn emit_proposal_disputed(env: &Env, proposal_id: u64, caller: Address) {
    let event = GovernorEvent {
        event_type: EventType::ProposalDisputed,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller,
        block_height: env.ledger().sequence() as u64,
        data: GovernorEventData {
            proposal_id,
            proposer: None,
            voter: None,
            vote_type: None,
            weight: None,
            for_votes: None,
            against_votes: None,
            abstain_votes: None,
        },
    };
    env.events()
        .publish((symbol_short!("GOV"), symbol_short!("DISP")), event);
}

/// Emit VotingEnded event with final tallies
pub fn emit_voting_ended(
    env: &Env,
    proposal_id: u64,
    for_votes: i128,
    against_votes: i128,
    abstain_votes: i128,
) {
    let event = GovernorEvent {
        event_type: EventType::VotingEnded,
        category: OperationCategory::System,
        timestamp: env.ledger().timestamp(),
        user_id: Address::from_contract_id(&env, &env.current_contract_address()),
        block_height: env.ledger().sequence() as u64,
        data: GovernorEventData {
            proposal_id,
            proposer: None,
            voter: None,
            vote_type: None,
            weight: None,
            for_votes: Some(for_votes),
            against_votes: Some(against_votes),
            abstain_votes: Some(abstain_votes),
        },
    };
    env.events()
        .publish((symbol_short!("GOV"), symbol_short!("VEND")), event);
}
