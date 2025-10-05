#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Bytes, Env, Map, Symbol,
};

#[derive(Clone)]
#[contracttype]
pub struct GovernorConfig {
    pub voting_delay: u32,
    pub voting_period: u32,
    pub quorum: i128,
    pub timelock: Address,
}

#[derive(Clone)]
#[contracttype]
pub struct Proposal {
    pub proposer: Address,
    pub description_ref: Bytes, // e.g., IPFS CID bytes or text ref
    pub start: u64,
    pub end: u64,
    pub for_votes: i128,
    pub against_votes: i128,
    pub abstain_votes: i128,
    pub queued: bool,
    pub executed: bool,
}

const CFG: Symbol = symbol_short!("cfg");
const PROPS: Symbol = symbol_short!("props");
const WEIGHTS: Symbol = symbol_short!("weights");

#[contract]
pub struct Governor;

fn now(env: &Env) -> u64 {
    env.ledger().timestamp().into()
}

#[contractimpl]
impl Governor {
    pub fn initialize(
        env: Env,
        timelock: Address,
        voting_delay: u32,
        voting_period: u32,
        quorum: i128,
    ) {
        if env.storage().persistent().has(&CFG) {
            panic!("init");
        }
        let cfg = GovernorConfig {
            timelock,
            voting_delay,
            voting_period,
            quorum,
        };
        env.storage().persistent().set(&CFG, &cfg);
    }

    pub fn set_weight(env: Env, voter: Address, weight: i128) {
        // In production, weights come from snapshot ERC-20 voting power via oracle or interface.
        let mut w: Map<Address, i128> = env
            .storage()
            .persistent()
            .get(&WEIGHTS)
            .unwrap_or(Map::new(&env));
        w.set(voter, weight);
        env.storage().persistent().set(&WEIGHTS, &w);
    }

    pub fn propose(env: Env, id: u64, proposer: Address, description_ref: Bytes) {
        let cfg: GovernorConfig = env
            .storage()
            .persistent()
            .get(&CFG)
            .unwrap_or_else(|| panic!("init"));
        let mut props: Map<u64, Proposal> = env
            .storage()
            .persistent()
            .get(&PROPS)
            .unwrap_or(Map::new(&env));
        if props.contains_key(id) {
            panic!("exists");
        }
        let start = now(&env) + (cfg.voting_delay as u64);
        let end = start + (cfg.voting_period as u64);
        let p = Proposal {
            proposer,
            description_ref,
            start,
            end,
            for_votes: 0,
            against_votes: 0,
            abstain_votes: 0,
            queued: false,
            executed: false,
        };
        props.set(id, p);
        env.storage().persistent().set(&PROPS, &props);
        env.events()
            .publish((symbol_short!("Propose"), id), (start, end));
    }

    pub fn cast_vote(env: Env, id: u64, voter: Address, support: u32) {
        let mut props: Map<u64, Proposal> = env
            .storage()
            .persistent()
            .get(&PROPS)
            .unwrap_or(Map::new(&env));
        let mut p = props.get(id).unwrap_or_else(|| panic!("no prop"));
        let t = now(&env);
        if t < p.start || t >= p.end {
            panic!("not active");
        }
        let w: Map<Address, i128> = env
            .storage()
            .persistent()
            .get(&WEIGHTS)
            .unwrap_or(Map::new(&env));
        let weight = w.get(voter).unwrap_or(0);
        if weight <= 0 {
            panic!("no weight");
        }
        match support {
            // 0=against,1=for,2=abstain
            0 => p.against_votes += weight,
            1 => p.for_votes += weight,
            2 => p.abstain_votes += weight,
            _ => panic!("bad support"),
        }
        props.set(id, p.clone());
        env.storage().persistent().set(&PROPS, &props);
        env.events()
            .publish((symbol_short!("Vote"), id), (support, weight));
    }

    pub fn state(env: Env, id: u64) -> u32 {
        let cfg: GovernorConfig = env
            .storage()
            .persistent()
            .get(&CFG)
            .unwrap_or_else(|| panic!("init"));
        let props: Map<u64, Proposal> = env
            .storage()
            .persistent()
            .get(&PROPS)
            .unwrap_or(Map::new(&env));
        let p = props.get(id).unwrap_or_else(|| panic!("no prop"));
        let t = now(&env);
        if p.executed {
            return 4;
        } // executed
        if p.queued {
            return 3;
        } // queued
        if t < p.start {
            return 0;
        } // proposed
        if t < p.end {
            return 1;
        } // active
          // succeeded if for_votes >= quorum and > against
        if p.for_votes >= cfg.quorum && p.for_votes > p.against_votes {
            2
        } else {
            5
        } // 2=succeeded,5=failed
    }

    pub fn queue(env: Env, id: u64) {
        let cfg: GovernorConfig = env
            .storage()
            .persistent()
            .get(&CFG)
            .unwrap_or_else(|| panic!("init"));
        let mut props: Map<u64, Proposal> = env
            .storage()
            .persistent()
            .get(&PROPS)
            .unwrap_or(Map::new(&env));
        let mut p = props.get(id).unwrap_or_else(|| panic!("no prop"));
        // must be succeeded
        if Governor::state(env.clone(), id) != 2 {
            panic!("not succeeded");
        }
        p.queued = true;
        props.set(id, p.clone());
        env.storage().persistent().set(&PROPS, &props);
        // Hook to timelock.queue would go here; we emit event for now.
        env.events()
            .publish((symbol_short!("Queue"), id), (cfg.timelock,));
    }

    pub fn execute(env: Env, id: u64) {
        let mut props: Map<u64, Proposal> = env
            .storage()
            .persistent()
            .get(&PROPS)
            .unwrap_or(Map::new(&env));
        let mut p = props.get(id).unwrap_or_else(|| panic!("no prop"));
        if !p.queued {
            panic!("not queued");
        }
        p.executed = true;
        props.set(id, p.clone());
        env.storage().persistent().set(&PROPS, &props);
        env.events().publish((symbol_short!("Exec"), id), ());
    }
}

#[cfg(all(test, feature = "testutils"))]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Bytes, Env, LedgerInfo};

    #[test]
    fn lifecycle_succeeds() {
        let env = Env::default();
        env.mock_all_auths();
        let tl = Address::random(&env);
        Governor::initialize(env.clone(), tl, 5, 10, 100);
        let voter = Address::random(&env);
        Governor::set_weight(env.clone(), voter.clone(), 200);
        Governor::propose(
            env.clone(),
            1,
            voter.clone(),
            Bytes::from_array(&env, &[1, 2, 3]),
        );

        // Move into active
        env.ledger().set(LedgerInfo {
            timestamp: env.ledger().timestamp() + 6,
            ..Default::default()
        });
        Governor::cast_vote(env.clone(), 1, voter.clone(), 1);
        // Move past end
        env.ledger().set(LedgerInfo {
            timestamp: env.ledger().timestamp() + 20,
            ..Default::default()
        });
        assert_eq!(Governor::state(env.clone(), 1), 2);
        Governor::queue(env.clone(), 1);
        Governor::execute(env.clone(), 1);
        assert_eq!(Governor::state(env.clone(), 1), 4);
    }
}
