use soroban_sdk::{Address, Env, String, Vec};

use crate::types::{DataKey, Error, OracleNode, SourceType};
use crate::utils;

pub fn register_oracle(
    env: Env,
    operator: Address,
    endpoint: String,
    source_type: SourceType,
) -> Result<(), Error> {
    operator.require_auth();
    utils::require_initialized(&env)?;

    if endpoint.len() == 0 {
        return Err(Error::InvalidData);
    }

    if env
        .storage()
        .persistent()
        .has(&DataKey::Oracle(operator.clone()))
    {
        return Err(Error::OracleAlreadyRegistered);
    }

    let mut oracles: Vec<Address> = env
        .storage()
        .instance()
        .get(&DataKey::OracleList)
        .unwrap_or(Vec::new(&env));
    oracles.push_back(operator.clone());

    let node = OracleNode {
        operator: operator.clone(),
        endpoint,
        source_type,
        verified: false,
        active: true,
        reputation: 50,
        submissions: 0,
        disputes: 0,
        last_seen: env.ledger().timestamp(),
    };

    env.storage()
        .persistent()
        .set(&DataKey::Oracle(operator), &node);
    env.storage().instance().set(&DataKey::OracleList, &oracles);
    Ok(())
}

pub fn verify_oracle(
    env: Env,
    admin: Address,
    operator: Address,
    verified: bool,
    active: bool,
) -> Result<(), Error> {
    utils::require_admin(&env, admin)?;

    let mut node = utils::read_oracle(&env, operator.clone())?;
    node.verified = verified;
    node.active = active;
    node.last_seen = env.ledger().timestamp();
    env.storage()
        .persistent()
        .set(&DataKey::Oracle(operator), &node);
    Ok(())
}

pub fn update_oracle_endpoint(
    env: Env,
    operator: Address,
    endpoint: String,
) -> Result<(), Error> {
    operator.require_auth();

    if endpoint.len() == 0 {
        return Err(Error::InvalidData);
    }

    let mut node = utils::read_oracle(&env, operator.clone())?;
    node.endpoint = endpoint;
    node.last_seen = env.ledger().timestamp();
    env.storage()
        .persistent()
        .set(&DataKey::Oracle(operator), &node);
    Ok(())
}

pub fn get_oracle(env: Env, operator: Address) -> Option<OracleNode> {
    env.storage().persistent().get(&DataKey::Oracle(operator))
}
