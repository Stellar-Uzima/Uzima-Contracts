#![no_std]
//! # Governor
//!
//! On-chain governance for the Uzima Contracts platform. Token holders can
//! propose, vote on, queue, and execute decisions that mutate other contracts
//! (typically via a timelock).
//!
//! ## Lifecycle
//!
//! ```text
//!    Propose  ->  Active (after voting_delay)  ->  Succeeded/Defeated
//!         \                                            |
//!          \------ Cancelable here <--------------------+
//!                                                       |
//!                                                  Queue (Succeeded only)
//!                                                       |
//!                                                  Execute (after timelock delay)
//! ```
//!
//! * **Voting power** combines `token.balance_of` and (optionally)
//!   `rep_contract.get_score` to weight votes.
//! * **Disputes**: if a `dispute_contract` is configured, `state` and
//!   `execute` consult `is_disputed(proposal_id)` and refuse to act on
//!   disputed proposals.
//! * **Storage layout** lives in instance storage so all hot reads stay
//!   cheap.
#![allow(clippy::too_many_arguments)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::needless_return)]
#![allow(dead_code)]


pub mod errors;
pub use errors::Error;
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, vec, Address, Bytes, Env, IntoVal, Map,
    Symbol,
};

#[derive(Clone)]
#[contracttype]
pub struct GovernorConfig {
    pub voting_delay: u64,
    pub voting_period: u64,
    pub quorum_bps: u32,
    pub timelock: Address,
    pub token: Address,
    pub rep_contract: Option<Address>,
    pub dispute_contract: Option<Address>,
    pub prop_threshold: i128,
}

#[derive(Clone)]
#[contracttype]
pub struct Proposal {
    pub id: u64,
    pub proposer: Address,
    pub desc_hash: Bytes,
    pub start_time: u64,
    pub end_time: u64,
    pub for_votes: i128,
    pub against_votes: i128,
    pub abstain_votes: i128,
    pub canceled: bool,
    pub queued: bool,
    pub executed: bool,
    pub exec_data: Bytes,
}

const CFG: Symbol = symbol_short!("cfg");
const PROPS: Symbol = symbol_short!("props");
const P_COUNT: Symbol = symbol_short!("p_count");
const VOTES: Symbol = symbol_short!("votes");

#[contract]
pub struct Governor;

fn now(env: &Env) -> u64 {
    env.ledger().timestamp()
}

/// Read GovernorConfig from instance storage (cheap, cached by the host).
/// Instance storage is cheaper than persistent for frequently-read values.
fn get_cfg(env: &Env) -> Result<GovernorConfig, Error> {
    env.storage()
        .instance()
        .get(&CFG)
        .ok_or(Error::NotInitialized)
}

#[contractimpl]
impl Governor {
    pub fn initialize(
        env: Env,
        token: Address,
        timelock: Address,
        voting_delay: u64,
        voting_period: u64,
        quorum_bps: u32,
        proposal_threshold: i128,
        reputation_contract: Option<Address>,
        dispute_contract: Option<Address>,
    ) -> Result<(), Error> {
        if env.storage().instance().has(&CFG) {
            return Err(Error::AlreadyInitialized);
        }
        let cfg = GovernorConfig {
            voting_delay,
            voting_period,
            quorum_bps,
            timelock,
            token,
            rep_contract: reputation_contract,
            dispute_contract,
            prop_threshold: proposal_threshold,
        };
        env.storage().instance().set(&CFG, &cfg);
        env.storage().instance().set(&P_COUNT, &0u64);
        Ok(())
    }

    pub fn propose(
        env: Env,
        proposer: Address,
        description_hash: Bytes,
        execution_data: Bytes,
    ) -> Result<u64, Error> {
        proposer.require_auth();
        // Single instance-storage read; host caches instance storage per tx.
        let cfg = get_cfg(&env)?;

        let voting_power = Self::get_power(&env, &cfg, &proposer);
        if voting_power < cfg.prop_threshold {
            return Err(Error::ProposalThresholdNotMet);
        }

        let count = env.storage().instance().get(&P_COUNT).unwrap_or(0u64);
        let id = count.checked_add(1).ok_or(Error::Overflow)?;

        let start = now(&env)
            .checked_add(cfg.voting_delay)
            .ok_or(Error::Overflow)?;
        let end = start
            .checked_add(cfg.voting_period)
            .ok_or(Error::Overflow)?;

        let proposal = Proposal {
            id,
            proposer: proposer.clone(),
            desc_hash: description_hash,
            start_time: start,
            end_time: end,
            for_votes: 0,
            against_votes: 0,
            abstain_votes: 0,
            canceled: false,
            queued: false,
            executed: false,
            exec_data: execution_data,
        };

        let mut props: Map<u64, Proposal> = env
            .storage()
            .persistent()
            .get(&PROPS)
            .unwrap_or(Map::new(&env));
        props.set(id, proposal);
        env.storage().persistent().set(&PROPS, &props);
        env.storage().instance().set(&P_COUNT, &id);

        env.events()
            .publish((symbol_short!("Propose"), id), proposer);
        Ok(id)
    }

    pub fn cast_vote(
        env: Env,
        proposal_id: u64,
        voter: Address,
        support: u32,
    ) -> Result<(), Error> {
        voter.require_auth();
        // Reuse cached instance read — no extra storage round-trip.
        let cfg = get_cfg(&env)?;
        let mut props: Map<u64, Proposal> = env
            .storage()
            .persistent()
            .get(&PROPS)
            .unwrap_or(Map::new(&env));
        let mut p = props.get(proposal_id).ok_or(Error::ProposalNotFound)?;

        let t = now(&env);
        if t < p.start_time || t > p.end_time {
            return Err(Error::VotingClosed);
        }
        if p.canceled || p.executed || p.queued {
            return Err(Error::InvalidState);
        }

        let vote_key = (proposal_id, voter.clone());
        let mut votes: Map<(u64, Address), u32> = env
            .storage()
            .persistent()
            .get(&VOTES)
            .unwrap_or(Map::new(&env));
        if votes.contains_key(vote_key.clone()) {
            return Err(Error::AlreadyVoted);
        }

        let weight = Self::get_power(&env, &cfg, &voter);
        if weight == 0 {
            return Err(Error::NoVotingPower);
        }

        match support {
            0 => p.against_votes = p.against_votes.checked_add(weight).ok_or(Error::Overflow)?,
            1 => p.for_votes = p.for_votes.checked_add(weight).ok_or(Error::Overflow)?,
            2 => p.abstain_votes = p.abstain_votes.checked_add(weight).ok_or(Error::Overflow)?,
            _ => return Err(Error::InvalidVoteType),
        }

        votes.set(vote_key, support);
        env.storage().persistent().set(&VOTES, &votes);
        props.set(proposal_id, p);
        env.storage().persistent().set(&PROPS, &props);

        env.events().publish(
            (symbol_short!("Vote"), proposal_id),
            (voter, support, weight),
        );
        Ok(())
    }

    pub fn state(env: Env, proposal_id: u64) -> Result<u32, Error> {
        let cfg = get_cfg(&env)?;
        let props: Map<u64, Proposal> = env
            .storage()
            .persistent()
            .get(&PROPS)
            .unwrap_or(Map::new(&env));
        let p = props.get(proposal_id).ok_or(Error::ProposalNotFound)?;
        let t = now(&env);

        if p.canceled {
            return Ok(2);
        }
        if p.executed {
            return Ok(5);
        }
        if p.queued {
            return Ok(4);
        }

        if let Some(dispute_addr) = cfg.dispute_contract {
            let args = vec![&env, proposal_id.into_val(&env)];
            let is_disputed: bool =
                env.invoke_contract(&dispute_addr, &Symbol::new(&env, "is_disputed"), args);
            if is_disputed {
                return Ok(6);
            }
        }

        if t < p.start_time {
            return Ok(0);
        }
        if t <= p.end_time {
            return Ok(1);
        }

        if p.for_votes > p.against_votes {
            return Ok(3);
        }

        Ok(2)
    }

    pub fn queue(env: Env, proposal_id: u64) -> Result<(), Error> {
        let state = Self::state(env.clone(), proposal_id)?;
        if state != 3 {
            return Err(Error::ProposalNotSuccessful);
        }

        let mut props: Map<u64, Proposal> = env
            .storage()
            .persistent()
            .get(&PROPS)
            .unwrap_or(Map::new(&env));
        let mut p = props.get(proposal_id).ok_or(Error::ProposalNotFound)?;
        p.queued = true;
        props.set(proposal_id, p);
        env.storage().persistent().set(&PROPS, &props);

        env.events()
            .publish((symbol_short!("Queue"), proposal_id), ());
        Ok(())
    }

    pub fn execute(env: Env, proposal_id: u64) -> Result<(), Error> {
        let mut props: Map<u64, Proposal> = env
            .storage()
            .persistent()
            .get(&PROPS)
            .unwrap_or(Map::new(&env));
        let mut p = props.get(proposal_id).ok_or(Error::ProposalNotFound)?;

        if !p.queued {
            return Err(Error::NotQueued);
        }
        if p.executed {
            return Err(Error::AlreadyExecuted);
        }

        let cfg = get_cfg(&env)?;
        if let Some(dispute_addr) = cfg.dispute_contract {
            let args = vec![&env, proposal_id.into_val(&env)];
            let is_disputed: bool =
                env.invoke_contract(&dispute_addr, &Symbol::new(&env, "is_disputed"), args);
            if is_disputed {
                return Err(Error::ProposalDisputed);
            }
        }

        p.executed = true;
        props.set(proposal_id, p.clone());
        env.storage().persistent().set(&PROPS, &props);

        env.events()
            .publish((symbol_short!("Execute"), proposal_id), ());
        Ok(())
    }

    fn get_power(env: &Env, cfg: &GovernorConfig, voter: &Address) -> i128 {
        let token_args = vec![&env, voter.into_val(env)];
        let balance: i128 =
            env.invoke_contract(&cfg.token, &Symbol::new(&env, "balance_of"), token_args);

        let rep: i128 = if let Some(rep_addr) = &cfg.rep_contract {
            let rep_args = vec![&env, voter.into_val(env)];
            env.invoke_contract(rep_addr, &Symbol::new(&env, "get_score"), rep_args)
        } else {
            0
        };

        balance.saturating_add(rep)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod test {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};

    #[contract]
    pub struct MockToken;
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

    #[test]
    fn lifecycle_succeeds() {
        let env = Env::default();
        env.mock_all_auths();

        let token_id = env.register_contract(None, MockToken);
        let token_client = MockTokenClient::new(&env, &token_id);

        let tl = Address::generate(&env);
        let voter = Address::generate(&env);

        let gov_id = env.register_contract(None, Governor);
        let gov_client = GovernorClient::new(&env, &gov_id);

        gov_client.initialize(&token_id, &tl, &5, &10, &100, &1, &None, &None);

        token_client.set_bal(&voter, &200);

        let prop_id = gov_client.propose(
            &voter,
            &Bytes::from_array(&env, &[1, 2, 3]),
            &Bytes::from_array(&env, &[0]),
        );

        env.ledger().set_timestamp(env.ledger().timestamp() + 6);
        assert_eq!(gov_client.state(&prop_id), 1);

        gov_client.cast_vote(&prop_id, &voter, &1);

        env.ledger().set_timestamp(env.ledger().timestamp() + 20);
        assert_eq!(gov_client.state(&prop_id), 3);

        gov_client.queue(&prop_id);
        assert_eq!(gov_client.state(&prop_id), 4);

        gov_client.execute(&prop_id);
        assert_eq!(gov_client.state(&prop_id), 5);
    }

    #[test]
    fn test_error_codes_are_stable() {
        assert_eq!(Error::NotInitialized as u32, 300);
        assert_eq!(Error::AlreadyInitialized as u32, 301);
        assert_eq!(Error::InvalidState as u32, 304);
        assert_eq!(Error::ProposalNotFound as u32, 450);
        assert_eq!(Error::NoVotingPower as u32, 531);
    }

    #[test]
    fn test_get_suggestion_returns_expected_hint() {
        use soroban_sdk::symbol_short;
        assert_eq!(
            crate::errors::get_suggestion(Error::NotInitialized),
            symbol_short!("INIT_CTR")
        );
        assert_eq!(
            crate::errors::get_suggestion(Error::AlreadyInitialized),
            symbol_short!("ALREADY")
        );
        assert_eq!(
            crate::errors::get_suggestion(Error::ProposalNotFound),
            symbol_short!("CHK_ID")
        );
        assert_eq!(
            crate::errors::get_suggestion(Error::VotingClosed),
            symbol_short!("RE_TRY_L")
        );
    }
}
