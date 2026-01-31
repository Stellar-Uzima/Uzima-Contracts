#![no_std]

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

#[contractimpl]
impl Governor {
    #[allow(clippy::too_many_arguments)]
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
    ) {
        if env.storage().instance().has(&CFG) {
            panic!("already initialized");
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
    }

    pub fn propose(
        env: Env,
        proposer: Address,
        description_hash: Bytes,
        execution_data: Bytes,
    ) -> u64 {
        proposer.require_auth();
        let cfg: GovernorConfig = env.storage().instance().get(&CFG).unwrap();

        // Check Proposal Threshold
        let voting_power = Self::get_power(&env, &cfg, &proposer);
        if voting_power < cfg.prop_threshold {
            panic!("below proposal threshold");
        }

        let id = env.storage().instance().get(&P_COUNT).unwrap_or(0u64) + 1;
        let start = now(&env) + cfg.voting_delay;
        let end = start + cfg.voting_period;

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
        id
    }

    pub fn cast_vote(env: Env, proposal_id: u64, voter: Address, support: u32) {
        voter.require_auth();
        let cfg: GovernorConfig = env.storage().instance().get(&CFG).unwrap();
        let mut props: Map<u64, Proposal> = env.storage().persistent().get(&PROPS).unwrap();
        let mut p = props.get(proposal_id).unwrap();

        let t = now(&env);
        if t < p.start_time || t > p.end_time {
            panic!("voting closed");
        }
        if p.canceled || p.executed || p.queued {
            panic!("invalid state");
        }

        let vote_key = (proposal_id, voter.clone());
        let mut votes: Map<(u64, Address), u32> = env
            .storage()
            .persistent()
            .get(&VOTES)
            .unwrap_or(Map::new(&env));
        if votes.contains_key(vote_key.clone()) {
            panic!("already voted");
        }

        let weight = Self::get_power(&env, &cfg, &voter);
        if weight == 0 {
            panic!("no voting power");
        }

        match support {
            0 => p.against_votes += weight,
            1 => p.for_votes += weight,
            2 => p.abstain_votes += weight,
            _ => panic!("bad vote type"),
        }

        votes.set(vote_key, support);
        env.storage().persistent().set(&VOTES, &votes);
        props.set(proposal_id, p);
        env.storage().persistent().set(&PROPS, &props);

        env.events().publish(
            (symbol_short!("Vote"), proposal_id),
            (voter, support, weight),
        );
    }

    pub fn state(env: Env, proposal_id: u64) -> u32 {
        let cfg: GovernorConfig = env.storage().instance().get(&CFG).unwrap();
        let props: Map<u64, Proposal> = env.storage().persistent().get(&PROPS).unwrap();
        let p = props.get(proposal_id).unwrap();
        let t = now(&env);

        if p.canceled {
            return 2;
        }
        if p.executed {
            return 5;
        }
        if p.queued {
            return 4;
        }

        if let Some(dispute_addr) = cfg.dispute_contract {
            let args = vec![&env, proposal_id.into_val(&env)];
            let is_disputed: bool =
                env.invoke_contract(&dispute_addr, &Symbol::new(&env, "is_disputed"), args);
            if is_disputed {
                return 6;
            } // Disputed
        }

        if t < p.start_time {
            return 0;
        }
        if t <= p.end_time {
            return 1;
        }

        if p.for_votes > p.against_votes {
            return 3;
        } // Succeeded

        2 // Defeated (Fixed unneeded return)
    }

    pub fn queue(env: Env, proposal_id: u64) {
        let state = Self::state(env.clone(), proposal_id);
        if state != 3 {
            panic!("proposal not successful");
        }

        let mut props: Map<u64, Proposal> = env.storage().persistent().get(&PROPS).unwrap();
        let mut p = props.get(proposal_id).unwrap();
        p.queued = true;
        props.set(proposal_id, p);
        env.storage().persistent().set(&PROPS, &props);

        env.events()
            .publish((symbol_short!("Queue"), proposal_id), ());
    }

    pub fn execute(env: Env, proposal_id: u64) {
        let mut props: Map<u64, Proposal> = env.storage().persistent().get(&PROPS).unwrap();
        let mut p = props.get(proposal_id).unwrap();

        if !p.queued {
            panic!("not queued");
        }
        if p.executed {
            panic!("already executed");
        }

        let cfg: GovernorConfig = env.storage().instance().get(&CFG).unwrap();
        if let Some(dispute_addr) = cfg.dispute_contract {
            let args = vec![&env, proposal_id.into_val(&env)];
            let is_disputed: bool =
                env.invoke_contract(&dispute_addr, &Symbol::new(&env, "is_disputed"), args);
            if is_disputed {
                panic!("proposal disputed");
            }
        }

        p.executed = true;
        props.set(proposal_id, p.clone());
        env.storage().persistent().set(&PROPS, &props);

        env.events()
            .publish((symbol_short!("Execute"), proposal_id), ());
    }

    // --- Helpers ---
    fn get_power(env: &Env, cfg: &GovernorConfig, voter: &Address) -> i128 {
        let token_args = vec![env, voter.into_val(env)];
        // Fixed needless borrow: &env -> env
        let balance: i128 =
            env.invoke_contract(&cfg.token, &Symbol::new(env, "balance_of"), token_args);

        // Reputation
        let rep: i128 = if let Some(rep_addr) = &cfg.rep_contract {
            let rep_args = vec![env, voter.into_val(env)];
            // Fixed needless borrow: &env -> env
            env.invoke_contract(rep_addr, &Symbol::new(env, "get_score"), rep_args)
        } else {
            0
        };

        balance + rep
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};

    // MOCK TOKEN
    #[contract]
    pub struct MockToken;
    #[contractimpl]
    impl MockToken {
        pub fn balance_of(env: Env, user: Address) -> i128 {
            let key = (symbol_short!("bal"), user);
            env.storage().instance().get(&key).unwrap_or(0i128)
        }

        // Helper to set balance for testing
        pub fn set_bal(env: Env, user: Address, amount: i128) {
            let key = (symbol_short!("bal"), user);
            env.storage().instance().set(&key, &amount);
        }
    }

    #[test]
    fn lifecycle_succeeds() {
        let env = Env::default();
        env.mock_all_auths();

        // 1. Setup Mocks
        let token_id = env.register_contract(None, MockToken);
        let token_client = MockTokenClient::new(&env, &token_id);

        let tl = Address::generate(&env);
        let voter = Address::generate(&env);

        // 2. Initialize Governor
        let gov_id = env.register_contract(None, Governor);
        let gov_client = GovernorClient::new(&env, &gov_id);

        gov_client.initialize(
            &token_id, &tl, &5,    // voting_delay
            &10,   // voting_period
            &100,  // quorum_bps
            &1,    // proposal_threshold
            &None, // no reputation contract
            &None, // no dispute contract
        );

        // 3. Give Voter Weight
        // We use our helper 'set_bal' to simulate the user having tokens
        token_client.set_bal(&voter, &200);

        // 4. Propose
        let prop_id = gov_client.propose(
            &voter,
            &Bytes::from_array(&env, &[1, 2, 3]), // Description Hash
            &Bytes::from_array(&env, &[0]),       // Execution Data
        );

        // 5. Move Time -> Active
        env.ledger().set_timestamp(env.ledger().timestamp() + 6);
        assert_eq!(gov_client.state(&prop_id), 1); // 1 = Active

        // 6. Vote
        gov_client.cast_vote(&prop_id, &voter, &1); // 1 = For

        // 7. Move Time -> Ended
        env.ledger().set_timestamp(env.ledger().timestamp() + 20);

        // 8. Queue & Execute
        assert_eq!(gov_client.state(&prop_id), 3); // 3 = Succeeded

        gov_client.queue(&prop_id);
        assert_eq!(gov_client.state(&prop_id), 4); // 4 = Queued

        gov_client.execute(&prop_id);
        assert_eq!(gov_client.state(&prop_id), 5); // 5 = Executed
    }
}
