#![cfg(test)]

extern crate std;

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, BytesN, Env, String,
};

use crate::{
    BridgeDisputeMediationContract, BridgeDisputeMediationContractClient, BridgeFailureKind,
    ChainId, DisputeState, DisputeVerdict, Error,
};

// ==================== Test Helpers ====================

fn setup_env() -> (Env, Address, BridgeDisputeMediationContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, BridgeDisputeMediationContract);
    let client = BridgeDisputeMediationContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    (env, admin, client)
}

fn init(
    env: &Env,
    client: &BridgeDisputeMediationContractClient,
    admin: &Address,
    min_votes: u32,
) {
    client.initialize(admin, &min_votes);
    let _ = env; // suppress unused-variable warning
}

fn make_op_id(env: &Env, seed: u8) -> BytesN<32> {
    BytesN::from_array(env, &[seed; 32])
}

fn file_basic_dispute(
    env: &Env,
    client: &BridgeDisputeMediationContractClient,
    claimant: &Address,
) -> BytesN<32> {
    client.file_dispute(
        claimant,
        &ChainId::Stellar,
        &ChainId::Ethereum,
        &BridgeFailureKind::MessageLost,
        &make_op_id(env, 1),
        &String::from_str(env, "test dispute"),
    )
}

// ==================== Initialisation ====================

#[test]
fn test_initialize_ok() {
    let (env, admin, client) = setup_env();
    init(&env, &client, &admin, 2);
    assert_eq!(client.min_votes(), 2);
    assert_eq!(client.dispute_count(), 0);
}

#[test]
fn test_initialize_twice_fails() {
    let (env, admin, client) = setup_env();
    init(&env, &client, &admin, 1);
    let result = client.try_initialize(&admin, &1);
    assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
}

// ==================== Mediator Management ====================

#[test]
fn test_add_mediator_ok() {
    let (env, admin, client) = setup_env();
    init(&env, &client, &admin, 1);

    let mediator = Address::generate(&env);
    client.add_mediator(&admin, &mediator);

    let info = client.get_mediator(&mediator);
    assert!(info.is_active);
    assert_eq!(info.cases_resolved, 0);
}

#[test]
fn test_add_mediator_duplicate_fails() {
    let (env, admin, client) = setup_env();
    init(&env, &client, &admin, 1);

    let mediator = Address::generate(&env);
    client.add_mediator(&admin, &mediator);
    let result = client.try_add_mediator(&admin, &mediator);
    assert_eq!(result, Err(Ok(Error::MediatorExists)));
}

#[test]
fn test_add_mediator_non_admin_fails() {
    let (env, admin, client) = setup_env();
    init(&env, &client, &admin, 1);

    let not_admin = Address::generate(&env);
    let mediator = Address::generate(&env);
    let result = client.try_add_mediator(&not_admin, &mediator);
    assert_eq!(result, Err(Ok(Error::Unauthorized)));
}

#[test]
fn test_deactivate_mediator() {
    let (env, admin, client) = setup_env();
    init(&env, &client, &admin, 1);

    let mediator = Address::generate(&env);
    client.add_mediator(&admin, &mediator);
    client.deactivate_mediator(&admin, &mediator);

    let info = client.get_mediator(&mediator);
    assert!(!info.is_active);
}

// ==================== Filing and Withdrawing ====================

#[test]
fn test_file_dispute_creates_record() {
    let (env, admin, client) = setup_env();
    init(&env, &client, &admin, 1);

    let claimant = Address::generate(&env);
    let dispute_id = file_basic_dispute(&env, &client, &claimant);

    let record = client.get_dispute(&dispute_id);
    assert_eq!(record.state, DisputeState::Open);
    assert_eq!(record.claimant, claimant);
    assert_eq!(client.dispute_count(), 1);
}

#[test]
fn test_file_multiple_disputes_unique_ids() {
    let (env, admin, client) = setup_env();
    init(&env, &client, &admin, 1);

    let claimant = Address::generate(&env);
    let id1 = file_basic_dispute(&env, &client, &claimant);
    let id2 = file_basic_dispute(&env, &client, &claimant);
    assert_ne!(id1, id2);
    assert_eq!(client.dispute_count(), 2);
}

#[test]
fn test_withdraw_dispute_ok() {
    let (env, admin, client) = setup_env();
    init(&env, &client, &admin, 1);

    let claimant = Address::generate(&env);
    let id = file_basic_dispute(&env, &client, &claimant);

    client.withdraw_dispute(&claimant, &id);
    assert_eq!(client.get_dispute_state(&id), DisputeState::Withdrawn);
}

#[test]
fn test_withdraw_by_non_claimant_fails() {
    let (env, admin, client) = setup_env();
    init(&env, &client, &admin, 1);

    let claimant = Address::generate(&env);
    let other = Address::generate(&env);
    let id = file_basic_dispute(&env, &client, &claimant);

    let result = client.try_withdraw_dispute(&other, &id);
    assert_eq!(result, Err(Ok(Error::Unauthorized)));
}

#[test]
fn test_withdraw_under_review_fails() {
    let (env, admin, client) = setup_env();
    init(&env, &client, &admin, 1);

    let claimant = Address::generate(&env);
    let mediator = Address::generate(&env);
    client.add_mediator(&admin, &mediator);

    let id = file_basic_dispute(&env, &client, &claimant);
    client.accept_dispute(&mediator, &id);

    let result = client.try_withdraw_dispute(&claimant, &id);
    assert_eq!(result, Err(Ok(Error::InvalidTransition)));
}

// ==================== State Transitions ====================

#[test]
fn test_accept_dispute_transitions_to_under_review() {
    let (env, admin, client) = setup_env();
    init(&env, &client, &admin, 1);

    let claimant = Address::generate(&env);
    let mediator = Address::generate(&env);
    client.add_mediator(&admin, &mediator);

    let id = file_basic_dispute(&env, &client, &claimant);
    client.accept_dispute(&mediator, &id);

    let record = client.get_dispute(&id);
    assert_eq!(record.state, DisputeState::UnderReview);
    assert_eq!(record.assigned_to, Some(mediator));
}

#[test]
fn test_accept_already_accepted_dispute_fails() {
    let (env, admin, client) = setup_env();
    init(&env, &client, &admin, 1);

    let claimant = Address::generate(&env);
    let m1 = Address::generate(&env);
    let m2 = Address::generate(&env);
    client.add_mediator(&admin, &m1);
    client.add_mediator(&admin, &m2);

    let id = file_basic_dispute(&env, &client, &claimant);
    client.accept_dispute(&m1, &id);

    let result = client.try_accept_dispute(&m2, &id);
    assert_eq!(result, Err(Ok(Error::InvalidTransition)));
}

#[test]
fn test_escalate_by_assigned_mediator() {
    let (env, admin, client) = setup_env();
    init(&env, &client, &admin, 2);

    let claimant = Address::generate(&env);
    let mediator = Address::generate(&env);
    client.add_mediator(&admin, &mediator);

    let id = file_basic_dispute(&env, &client, &claimant);
    client.accept_dispute(&mediator, &id);
    client.escalate_dispute(&mediator, &id);

    assert_eq!(client.get_dispute_state(&id), DisputeState::Escalated);
}

#[test]
fn test_escalate_by_admin() {
    let (env, admin, client) = setup_env();
    init(&env, &client, &admin, 2);

    let claimant = Address::generate(&env);
    let mediator = Address::generate(&env);
    client.add_mediator(&admin, &mediator);

    let id = file_basic_dispute(&env, &client, &claimant);
    client.accept_dispute(&mediator, &id);
    client.escalate_dispute(&admin, &id);

    assert_eq!(client.get_dispute_state(&id), DisputeState::Escalated);
}

#[test]
fn test_escalate_from_open_fails() {
    let (env, admin, client) = setup_env();
    init(&env, &client, &admin, 1);

    let claimant = Address::generate(&env);
    let id = file_basic_dispute(&env, &client, &claimant);

    let result = client.try_escalate_dispute(&admin, &id);
    assert_eq!(result, Err(Ok(Error::InvalidTransition)));
}

// ==================== Voting & Auto-resolution ====================

#[test]
fn test_cast_vote_auto_resolves_on_quorum() {
    let (env, admin, client) = setup_env();
    init(&env, &client, &admin, 1); // quorum = 1

    let claimant = Address::generate(&env);
    let mediator = Address::generate(&env);
    client.add_mediator(&admin, &mediator);

    let id = file_basic_dispute(&env, &client, &claimant);
    client.accept_dispute(&mediator, &id);

    let final_state = client.cast_vote(
        &mediator,
        &id,
        &DisputeVerdict::Upheld,
        &String::from_str(&env, "evidence is clear"),
    );

    assert_eq!(final_state, DisputeState::Resolved);
    assert_eq!(client.get_verdict(&id), Some(DisputeVerdict::Upheld));
}

#[test]
fn test_cast_vote_does_not_resolve_below_quorum() {
    let (env, admin, client) = setup_env();
    init(&env, &client, &admin, 2); // quorum = 2

    let claimant = Address::generate(&env);
    let m1 = Address::generate(&env);
    let m2 = Address::generate(&env);
    client.add_mediator(&admin, &m1);
    client.add_mediator(&admin, &m2);

    let id = file_basic_dispute(&env, &client, &claimant);
    client.accept_dispute(&m1, &id);
    client.escalate_dispute(&m1, &id);

    // First vote – not yet at quorum.
    let state = client.cast_vote(
        &m1,
        &id,
        &DisputeVerdict::Rejected,
        &String::from_str(&env, "no evidence"),
    );
    assert_eq!(state, DisputeState::Escalated);

    // Second vote – reaches quorum.
    let state2 = client.cast_vote(
        &m2,
        &id,
        &DisputeVerdict::Rejected,
        &String::from_str(&env, "agreed"),
    );
    assert_eq!(state2, DisputeState::Resolved);
    assert_eq!(client.get_verdict(&id), Some(DisputeVerdict::Rejected));
}

#[test]
fn test_double_vote_fails() {
    let (env, admin, client) = setup_env();
    init(&env, &client, &admin, 2);

    let claimant = Address::generate(&env);
    let mediator = Address::generate(&env);
    client.add_mediator(&admin, &mediator);

    let id = file_basic_dispute(&env, &client, &claimant);
    client.accept_dispute(&mediator, &id);
    client.cast_vote(
        &mediator,
        &id,
        &DisputeVerdict::Upheld,
        &String::from_str(&env, "first"),
    );

    let result = client.try_cast_vote(
        &mediator,
        &id,
        &DisputeVerdict::Rejected,
        &String::from_str(&env, "second"),
    );
    assert_eq!(result, Err(Ok(Error::AlreadyVoted)));
}

#[test]
fn test_inactive_mediator_cannot_vote() {
    let (env, admin, client) = setup_env();
    init(&env, &client, &admin, 1);

    let claimant = Address::generate(&env);
    let mediator = Address::generate(&env);
    client.add_mediator(&admin, &mediator);

    let id = file_basic_dispute(&env, &client, &claimant);
    client.accept_dispute(&mediator, &id);
    client.deactivate_mediator(&admin, &mediator);

    let result = client.try_cast_vote(
        &mediator,
        &id,
        &DisputeVerdict::Upheld,
        &String::from_str(&env, "late vote"),
    );
    assert_eq!(result, Err(Ok(Error::MediatorNotActive)));
}

// ==================== Admin Force-resolve ====================

#[test]
fn test_admin_resolve_dispute() {
    let (env, admin, client) = setup_env();
    init(&env, &client, &admin, 3); // high quorum, won't auto-resolve

    let claimant = Address::generate(&env);
    let mediator = Address::generate(&env);
    client.add_mediator(&admin, &mediator);

    let id = file_basic_dispute(&env, &client, &claimant);
    client.accept_dispute(&mediator, &id);

    client.resolve_dispute(&admin, &id, &DisputeVerdict::Upheld);
    assert_eq!(client.get_dispute_state(&id), DisputeState::Resolved);
    assert_eq!(client.get_verdict(&id), Some(DisputeVerdict::Upheld));
}

#[test]
fn test_non_admin_cannot_force_resolve() {
    let (env, admin, client) = setup_env();
    init(&env, &client, &admin, 3);

    let claimant = Address::generate(&env);
    let mediator = Address::generate(&env);
    client.add_mediator(&admin, &mediator);

    let id = file_basic_dispute(&env, &client, &claimant);
    client.accept_dispute(&mediator, &id);

    let result = client.try_resolve_dispute(&claimant, &id, &DisputeVerdict::Upheld);
    assert_eq!(result, Err(Ok(Error::Unauthorized)));
}

// ==================== Pausing ====================

#[test]
fn test_paused_contract_blocks_file_dispute() {
    let (env, admin, client) = setup_env();
    init(&env, &client, &admin, 1);
    client.set_paused(&admin, &true);

    let claimant = Address::generate(&env);
    let result = client.try_file_dispute(
        &claimant,
        &ChainId::Stellar,
        &ChainId::Ethereum,
        &BridgeFailureKind::MessageLost,
        &make_op_id(&env, 5),
        &String::from_str(&env, "should fail"),
    );
    assert_eq!(result, Err(Ok(Error::ContractPaused)));
}

#[test]
fn test_unpause_allows_operations() {
    let (env, admin, client) = setup_env();
    init(&env, &client, &admin, 1);
    client.set_paused(&admin, &true);
    client.set_paused(&admin, &false);

    let claimant = Address::generate(&env);
    // Should succeed after unpausing.
    file_basic_dispute(&env, &client, &claimant);
}

// ==================== Mediator cases_resolved counter ====================

#[test]
fn test_cases_resolved_increments_on_vote_resolution() {
    let (env, admin, client) = setup_env();
    init(&env, &client, &admin, 1);

    let claimant = Address::generate(&env);
    let mediator = Address::generate(&env);
    client.add_mediator(&admin, &mediator);

    let id = file_basic_dispute(&env, &client, &claimant);
    client.accept_dispute(&mediator, &id);
    client.cast_vote(
        &mediator,
        &id,
        &DisputeVerdict::Upheld,
        &String::from_str(&env, "ok"),
    );

    let info = client.get_mediator(&mediator);
    assert_eq!(info.cases_resolved, 1);
}

// ==================== Ledger timestamp in records ====================

#[test]
fn test_dispute_timestamps_are_set() {
    let (env, admin, client) = setup_env();
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);
    init(&env, &client, &admin, 1);

    let claimant = Address::generate(&env);
    let id = file_basic_dispute(&env, &client, &claimant);
    let record = client.get_dispute(&id);

    assert_eq!(record.filed_at, 1_000_000);
    assert_eq!(record.resolved_at, 0); // not yet resolved
}
