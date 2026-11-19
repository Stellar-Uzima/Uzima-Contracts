//! Reentrancy regression tests for dispute-mediation callbacks (issue #1951).
//! A `VoteReentryProbe` casts a vote and immediately tries a second vote in the
//! same frame. The inner call is refused and the tests assert exactly one vote
//! is recorded — i.e. state is updated before events are published.
#![cfg(test)]
#![allow(clippy::unwrap_used)]

use crate::{
    BridgeDisputeMediationContract, BridgeDisputeMediationContractClient, BridgeFailureKind,
    ChainId, DisputeState, DisputeVerdict,
};
use soroban_sdk::{contract, contractimpl, testutils::Address as _, Address, BytesN, Env, String};

#[contract]
struct VoteReentryProbe;

#[contractimpl]
impl VoteReentryProbe {
    pub fn probe_vote(
        env: Env,
        mediation: Address,
        mediator: Address,
        dispute_id: BytesN<32>,
    ) -> (bool, bool) {
        let client = BridgeDisputeMediationContractClient::new(&env, &mediation);
        let why = String::from_str(&env, "probe");
        let outer = client
            .try_cast_vote(&mediator, &dispute_id, &DisputeVerdict::Upheld, &why)
            .is_ok();
        let inner = client
            .try_cast_vote(&mediator, &dispute_id, &DisputeVerdict::Upheld, &why)
            .is_err();
        (outer, inner)
    }
}

#[test]
fn cast_vote_cannot_be_double_counted_by_reentrant_callback() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let claimant = Address::generate(&env);
    let mediator = Address::generate(&env);
    let id = env.register_contract(None, BridgeDisputeMediationContract);
    let client = BridgeDisputeMediationContractClient::new(&env, &id);

    client.initialize(&admin, &2);
    client.add_mediator(&admin, &mediator);
    let dispute_id = client.file_dispute(
        &claimant,
        &ChainId::Stellar,
        &ChainId::Ethereum,
        &BridgeFailureKind::MessageLost,
        &BytesN::from_array(&env, &[1u8; 32]),
        &String::from_str(&env, "probe dispute"),
    );
    client.accept_dispute(&mediator, &dispute_id);

    let probe_id = env.register_contract(None, VoteReentryProbe);
    let probe = VoteReentryProbeClient::new(&env, &probe_id);
    let (outer_ok, inner_refused) = probe.probe_vote(&id, &mediator, &dispute_id);

    assert!(outer_ok, "outer cast_vote should succeed");
    assert!(inner_refused, "re-entrant cast_vote must be refused");
    assert_eq!(
        client.get_votes(&dispute_id).unwrap().len(),
        1,
        "exactly one vote recorded"
    );
    assert_eq!(
        client.get_dispute_state(&dispute_id).unwrap(),
        DisputeState::UnderReview
    );
}
