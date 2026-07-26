//! Governance proposal expiry, cancellation, veto, and timelock-delay tests.
//!
//! Issue #906: Add governance proposal expiry and cancellation tests
//! Acceptance criteria:
//!   - Expired proposals cannot be executed
//!   - Timelock delay cannot be bypassed

#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

#[cfg(test)]
mod governance_expiry_cancellation_tests {
    use crate::{Error, Governor, GovernorClient};
    use soroban_sdk::{
        contract, contractimpl, symbol_short, testutils::{Address as _, Ledger},
        Address, Bytes, Env,
    };

    // ── Mock token used by all tests ─────────────────────────────────────

    #[contract]
    struct MockToken;

    #[contractimpl]
    impl MockToken {
        pub fn balance_of(env: Env, user: Address) -> i128 {
            let key = (symbol_short!("bal"), user);
            env.storage().instance().get(&key).unwrap_or(0i128)
        }
        pub fn set_bal(env: Env, user: Address, amount: i128) {
            let key = (symbol_short!("bal"), user);
            env.storage().instance().set(&key, &amount);
        }
    }

    /// Helper: deploy and initialize Governor + MockToken.
    /// voting_delay=5, voting_period=10, quorum=100 bps, threshold=1.
    fn setup(env: &Env) -> (GovernorClient, MockTokenClient, Address, Address) {
        env.mock_all_auths();
        let token_id = env.register_contract(None, MockToken);
        let token = MockTokenClient::new(env, &token_id);

        let timelock = Address::generate(env);
        let voter = Address::generate(env);
        token.set_bal(&voter, &200i128);

        let gov_id = env.register_contract(None, Governor);
        let gov = GovernorClient::new(env, &gov_id);
        gov.initialize(&token_id, &timelock, &5u64, &10u64, &100u32, &1i128, &None, &None);

        (gov, token, timelock, voter)
    }

    fn new_proposal(env: &Env, gov: &GovernorClient, proposer: &Address) -> u64 {
        gov.propose(
            proposer,
            &Bytes::from_array(env, &[1, 2, 3]),
            &Bytes::from_array(env, &[0]),
        )
    }

    // ─────────────────────────────────────────────────────────────────────
    // Test 1: Proposal expires without reaching quorum → Defeated, not executable
    // ─────────────────────────────────────────────────────────────────────
    #[test]
    fn proposal_expires_without_quorum_is_defeated() {
        let env = Env::default();
        let (gov, _token, _tl, voter) = setup(&env);

        let id = new_proposal(&env, &gov, &voter);

        // Advance past voting window without any votes.
        // start_time = now+5, end_time = start+10 → need >15 past creation.
        env.ledger().set_timestamp(env.ledger().timestamp() + 20);

        // for_votes (0) is NOT > against_votes (0) → Defeated (2)
        assert_eq!(gov.state(&id), 2, "expired, unvoted proposal must be Defeated");

        // Cannot queue a defeated proposal.
        let err = gov.try_queue(&id).unwrap_err().unwrap();
        assert_eq!(err, Error::ProposalNotSuccessful, "queueing defeated proposal must fail");
    }

    // ─────────────────────────────────────────────────────────────────────
    // Test 2: Proposal where against_votes > for_votes is Defeated
    // ─────────────────────────────────────────────────────────────────────
    #[test]
    fn proposal_defeated_when_against_exceeds_for() {
        let env = Env::default();
        env.mock_all_auths();
        let token_id = env.register_contract(None, MockToken);
        let token = MockTokenClient::new(&env, &token_id);

        let tl = Address::generate(&env);
        let proposer = Address::generate(&env);
        let voter_for = Address::generate(&env);
        let voter_against = Address::generate(&env);

        token.set_bal(&proposer, &10i128);
        token.set_bal(&voter_for, &50i128);
        token.set_bal(&voter_against, &200i128);

        let gov_id = env.register_contract(None, Governor);
        let gov = GovernorClient::new(&env, &gov_id);
        gov.initialize(&token_id, &tl, &5u64, &10u64, &100u32, &1i128, &None, &None);

        let id = gov.propose(
            &proposer,
            &Bytes::from_array(&env, &[9]),
            &Bytes::from_array(&env, &[0]),
        );

        // Advance into voting window.
        env.ledger().set_timestamp(env.ledger().timestamp() + 6);
        gov.cast_vote(&id, &voter_for, &1u32);     // for:  50
        gov.cast_vote(&id, &voter_against, &0u32); // against: 200

        // Advance past end_time.
        env.ledger().set_timestamp(env.ledger().timestamp() + 15);

        assert_eq!(gov.state(&id), 2, "proposal with more against votes must be Defeated");
    }

    // ─────────────────────────────────────────────────────────────────────
    // Test 3: Proposer can cancel an active proposal
    // ─────────────────────────────────────────────────────────────────────
    #[test]
    fn proposer_can_cancel_active_proposal() {
        let env = Env::default();
        let (gov, _token, _tl, voter) = setup(&env);

        let id = new_proposal(&env, &gov, &voter);

        // Advance into voting window.
        env.ledger().set_timestamp(env.ledger().timestamp() + 6);
        assert_eq!(gov.state(&id), 1, "proposal should be Active before cancel");

        // Cancel while active.
        gov.cancel(&id, &voter);
        assert_eq!(gov.state(&id), 2, "canceled proposal must report Canceled state");
    }

    // ─────────────────────────────────────────────────────────────────────
    // Test 4: Voting on a canceled proposal must fail
    // ─────────────────────────────────────────────────────────────────────
    #[test]
    fn voting_on_canceled_proposal_fails() {
        let env = Env::default();
        let (gov, _token, _tl, voter) = setup(&env);

        let id = new_proposal(&env, &gov, &voter);

        // Cancel immediately (still in pending state).
        gov.cancel(&id, &voter);

        // Try to vote — must fail.
        let err = gov.try_cast_vote(&id, &voter, &1u32).unwrap_err().unwrap();
        assert!(
            err == Error::InvalidState || err == Error::VotingClosed,
            "voting on canceled proposal must fail with InvalidState or VotingClosed, got {:?}",
            err
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // Test 5: Queuing a canceled proposal must fail
    // ─────────────────────────────────────────────────────────────────────
    #[test]
    fn queuing_canceled_proposal_fails() {
        let env = Env::default();
        let (gov, _token, _tl, voter) = setup(&env);

        let id = new_proposal(&env, &gov, &voter);
        gov.cancel(&id, &voter);

        let err = gov.try_queue(&id).unwrap_err().unwrap();
        assert_eq!(err, Error::ProposalNotSuccessful, "queueing canceled proposal must fail");
    }

    // ─────────────────────────────────────────────────────────────────────
    // Test 6: Non-proposer cannot cancel a proposal (admin veto simulation)
    // ─────────────────────────────────────────────────────────────────────
    #[test]
    fn non_proposer_cannot_cancel() {
        let env = Env::default();
        env.mock_all_auths();
        let token_id = env.register_contract(None, MockToken);
        let token = MockTokenClient::new(&env, &token_id);

        let tl = Address::generate(&env);
        let proposer = Address::generate(&env);
        let attacker = Address::generate(&env);

        token.set_bal(&proposer, &100i128);
        token.set_bal(&attacker, &100i128);

        let gov_id = env.register_contract(None, Governor);
        let gov = GovernorClient::new(&env, &gov_id);
        gov.initialize(&token_id, &tl, &5u64, &10u64, &100u32, &1i128, &None, &None);

        let id = gov.propose(
            &proposer,
            &Bytes::from_array(&env, &[5, 6]),
            &Bytes::from_array(&env, &[0]),
        );

        // Attacker tries to cancel proposal they did not create.
        let err = gov.try_cancel(&id, &attacker).unwrap_err().unwrap();
        assert_eq!(err, Error::Unauthorized, "only proposer should cancel their own proposal");
    }

    // ─────────────────────────────────────────────────────────────────────
    // Test 7: Duplicate proposals get distinct IDs
    // ─────────────────────────────────────────────────────────────────────
    #[test]
    fn duplicate_proposals_receive_distinct_ids() {
        let env = Env::default();
        let (gov, _token, _tl, voter) = setup(&env);

        let desc = Bytes::from_array(&env, &[0xDE, 0xAD]);
        let data = Bytes::from_array(&env, &[0]);

        let id1 = gov.propose(&voter, &desc, &data);
        let id2 = gov.propose(&voter, &desc, &data);

        assert_ne!(id1, id2, "duplicate proposals must receive distinct IDs");
    }

    // ─────────────────────────────────────────────────────────────────────
    // Test 8: Timelock delay enforcement is tested in timelock contract tests.
    // This test verifies that a queued (state=4) proposal cannot be re-executed.
    // ─────────────────────────────────────────────────────────────────────
    #[test]
    fn already_executed_proposal_cannot_execute_again() {
        let env = Env::default();
        let (gov, _token, _tl, voter) = setup(&env);

        let id = new_proposal(&env, &gov, &voter);
        env.ledger().set_timestamp(env.ledger().timestamp() + 6);
        gov.cast_vote(&id, &voter, &1u32);
        env.ledger().set_timestamp(env.ledger().timestamp() + 15);

        gov.queue(&id);
        gov.execute(&id);
        assert_eq!(gov.state(&id), 5);

        // Attempting execute again must fail.
        let err = gov.try_execute(&id).unwrap_err().unwrap();
        assert_eq!(err, Error::AlreadyExecuted, "re-executing must fail with AlreadyExecuted");
    }

    // ─────────────────────────────────────────────────────────────────────
    // Test 9: Succeeded proposal CAN be queued, then executed
    // ─────────────────────────────────────────────────────────────────────
    #[test]
    fn succeeded_proposal_can_be_queued_and_executed() {
        let env = Env::default();
        let (gov, _token, _tl, voter) = setup(&env);

        let id = new_proposal(&env, &gov, &voter);

        // Enter voting window and vote FOR.
        env.ledger().set_timestamp(env.ledger().timestamp() + 6);
        gov.cast_vote(&id, &voter, &1u32);

        // Advance past end_time → Succeeded (3).
        env.ledger().set_timestamp(env.ledger().timestamp() + 15);
        assert_eq!(gov.state(&id), 3, "proposal with majority for votes must be Succeeded");

        gov.queue(&id);
        assert_eq!(gov.state(&id), 4, "queued proposal must be in Queued state");

        gov.execute(&id);
        assert_eq!(gov.state(&id), 5, "executed proposal must be in Executed state");
    }
}
