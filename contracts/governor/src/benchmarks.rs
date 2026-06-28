//! Storage-read regression benchmarks for governor.
#![allow(clippy::unwrap_used)]
extern crate std;

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{Address, Bytes, Env};

#[contract]
pub struct BenchToken;

#[contractimpl]
impl BenchToken {
    pub fn balance_of(env: Env, user: Address) -> i128 {
        let key = (symbol_short!("bal"), user);
        env.storage().instance().get(&key).unwrap_or(0i128)
    }

    pub fn set_bal(env: Env, user: Address, amount: i128) {
        let key = (symbol_short!("bal"), user);
        env.storage().instance().set(&key, &amount);
    }
}

fn measure_cpu<F: FnOnce()>(env: &Env, f: F) -> u64 {
    env.budget().reset_unlimited();
    f();
    env.budget().cpu_instruction_cost()
}

fn print_delta(name: &str, before: u64, after: u64) {
    let saved = before.saturating_sub(after);
    let reduction_pct = if before == 0 {
        0.0
    } else {
        (saved as f64 * 100.0) / before as f64
    };
    std::println!(
        "[STORAGE-BENCH] {} before={} after={} saved={} reduction_pct={:.2}",
        name,
        before,
        after,
        saved,
        reduction_pct
    );
}

fn old_state(env: &Env, proposal_id: u64) -> Result<u32, Error> {
    let cfg = get_cfg(env)?;
    let props: Map<u64, Proposal> = env
        .storage()
        .persistent()
        .get(&PROPS)
        .unwrap_or(Map::new(env));
    let p = props.get(proposal_id).ok_or(Error::ProposalNotFound)?;
    let t = now(env);

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
        let args = vec![env, proposal_id.into_val(env)];
        let is_disputed: bool =
            env.invoke_contract(&dispute_addr, &Symbol::new(env, "is_disputed"), args);
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

fn new_state(env: &Env, proposal_id: u64) -> Result<u32, Error> {
    let cfg = get_cfg(env)?;
    let props = get_props(env);
    let p = props.get(proposal_id).ok_or(Error::ProposalNotFound)?;
    Ok(proposal_state(env, &cfg, proposal_id, &p))
}

fn old_queue(env: &Env, proposal_id: u64) -> Result<(), Error> {
    let state = old_state(env, proposal_id)?;
    if state != 3 {
        return Err(Error::ProposalNotSuccessful);
    }

    let mut props: Map<u64, Proposal> = env
        .storage()
        .persistent()
        .get(&PROPS)
        .unwrap_or(Map::new(env));
    let mut p = props.get(proposal_id).ok_or(Error::ProposalNotFound)?;
    p.queued = true;
    props.set(proposal_id, p);
    env.storage().persistent().set(&PROPS, &props);
    Ok(())
}

fn new_queue(env: &Env, proposal_id: u64) -> Result<(), Error> {
    let cfg = get_cfg(env)?;
    let mut props = get_props(env);
    let mut p = props.get(proposal_id).ok_or(Error::ProposalNotFound)?;

    if proposal_state(env, &cfg, proposal_id, &p) != 3 {
        return Err(Error::ProposalNotSuccessful);
    }

    p.queued = true;
    props.set(proposal_id, p);
    env.storage().persistent().set(&PROPS, &props);
    Ok(())
}

fn new_execute(env: &Env, proposal_id: u64) -> Result<(), Error> {
    let mut props = get_props(env);
    let mut p = props.get(proposal_id).ok_or(Error::ProposalNotFound)?;

    if !p.queued {
        return Err(Error::NotQueued);
    }
    if p.executed {
        return Err(Error::AlreadyExecuted);
    }

    let cfg = get_cfg(env)?;
    if let Some(dispute_addr) = &cfg.dispute_contract {
        let args = vec![env, proposal_id.into_val(env)];
        let is_disputed: bool =
            env.invoke_contract(dispute_addr, &Symbol::new(env, "is_disputed"), args);
        if is_disputed {
            return Err(Error::ProposalDisputed);
        }
    }

    p.executed = true;
    props.set(proposal_id, p.clone());
    env.storage().persistent().set(&PROPS, &props);
    Ok(())
}

fn old_execute(env: &Env, proposal_id: u64) -> Result<(), Error> {
    let mut props: Map<u64, Proposal> = env
        .storage()
        .persistent()
        .get(&PROPS)
        .unwrap_or(Map::new(env));
    let mut p = props.get(proposal_id).ok_or(Error::ProposalNotFound)?;

    if !p.queued {
        return Err(Error::NotQueued);
    }
    if p.executed {
        return Err(Error::AlreadyExecuted);
    }

    let cfg = get_cfg(env)?;
    if let Some(dispute_addr) = cfg.dispute_contract {
        let args = vec![env, proposal_id.into_val(env)];
        let is_disputed: bool =
            env.invoke_contract(&dispute_addr, &Symbol::new(env, "is_disputed"), args);
        if is_disputed {
            return Err(Error::ProposalDisputed);
        }
    }

    p.executed = true;
    props.set(proposal_id, p.clone());
    env.storage().persistent().set(&PROPS, &props);
    Ok(())
}

fn setup_governor(
    env: &Env,
) -> (
    GovernorClient<'_>,
    BenchTokenClient<'_>,
    Address,
    Address,
    Address,
) {
    env.mock_all_auths();
    let token_id = env.register_contract(None, BenchToken);
    let token_client = BenchTokenClient::new(env, &token_id);

    let gov_id = env.register_contract(None, Governor);
    let gov_client = GovernorClient::new(env, &gov_id);
    let timelock = Address::generate(env);
    let voter = Address::generate(env);

    gov_client.initialize(&token_id, &timelock, &5, &10, &100, &1, &None, &None);
    token_client.set_bal(&voter, &200);

    (gov_client, token_client, gov_id, timelock, voter)
}

fn successful_proposal(env: &Env) -> (GovernorClient<'_>, Address, u64, Address) {
    let (gov_client, _token_client, gov_id, _timelock, voter) = setup_governor(env);
    let prop_id = gov_client.propose(
        &voter,
        &Bytes::from_array(env, &[1, 2, 3]),
        &Bytes::from_array(env, &[0]),
    );
    env.ledger().set_timestamp(env.ledger().timestamp() + 6);
    gov_client.cast_vote(&prop_id, &voter, &1);
    env.ledger().set_timestamp(env.ledger().timestamp() + 20);
    (gov_client, gov_id, prop_id, voter)
}

fn queued_proposal(env: &Env) -> (GovernorClient<'_>, Address, u64) {
    let (gov_client, gov_id, prop_id, _voter) = successful_proposal(env);
    gov_client.queue(&prop_id);
    (gov_client, gov_id, prop_id)
}

#[test]
fn bench_storage_state_lookup() {
    let env_before = Env::default();
    let (_client_before, gov_before, prop_before, _voter_before) = successful_proposal(&env_before);
    let before = measure_cpu(&env_before, || {
        env_before.as_contract(&gov_before, || {
            old_state(&env_before, prop_before).unwrap();
        });
    });

    let env_after = Env::default();
    let (_client_after, gov_after, prop_after, _voter_after) = successful_proposal(&env_after);
    let after = measure_cpu(&env_after, || {
        env_after.as_contract(&gov_after, || {
            new_state(&env_after, prop_after).unwrap();
        });
    });

    print_delta("governor::state", before, after);
    assert!(after > 0);
}

#[test]
fn bench_storage_queue() {
    let env_before = Env::default();
    let (_client_before, gov_before, prop_before, _voter_before) = successful_proposal(&env_before);
    let before = measure_cpu(&env_before, || {
        env_before.as_contract(&gov_before, || {
            old_queue(&env_before, prop_before).unwrap();
        });
    });

    let env_after = Env::default();
    let (_client_after, gov_after, prop_after, _voter_after) = successful_proposal(&env_after);
    let after = measure_cpu(&env_after, || {
        env_after.as_contract(&gov_after, || {
            new_queue(&env_after, prop_after).unwrap();
        });
    });

    print_delta("governor::queue", before, after);
    assert!(after > 0);
}

#[test]
fn bench_storage_execute() {
    let env_before = Env::default();
    let (_client_before, gov_before, prop_before) = queued_proposal(&env_before);
    let before = measure_cpu(&env_before, || {
        env_before.as_contract(&gov_before, || {
            old_execute(&env_before, prop_before).unwrap();
        });
    });

    let env_after = Env::default();
    let (_client_after, gov_after, prop_after) = queued_proposal(&env_after);
    let after = measure_cpu(&env_after, || {
        env_after.as_contract(&gov_after, || {
            new_execute(&env_after, prop_after).unwrap();
        });
    });

    print_delta("governor::execute", before, after);
    assert!(after > 0);
}
