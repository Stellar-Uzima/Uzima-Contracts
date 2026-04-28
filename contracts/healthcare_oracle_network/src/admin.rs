use soroban_sdk::{Address, Env, Vec};

use crate::types::{Config, DataKey, Error};
use crate::utils;

pub fn initialize(
    env: Env,
    admin: Address,
    arbiters: Vec<Address>,
    min_submissions: u32,
) -> Result<(), Error> {
    if env.storage().instance().has(&DataKey::Config) {
        return Err(Error::AlreadyInitialized);
    }

    if min_submissions == 0 {
        return Err(Error::InvalidData);
    }

    let config = Config {
        admin,
        arbiters,
        min_submissions,
        min_reputation: 0,
        max_drug_price_minor: 1_000_000_000,
        max_availability_units: 5_000_000,
    };

    env.storage().instance().set(&DataKey::Config, &config);
    env.storage()
        .instance()
        .set(&DataKey::OracleList, &Vec::<Address>::new(&env));
    env.storage().instance().set(&DataKey::DisputeCount, &0u64);
    Ok(())
}

pub fn update_config(
    env: Env,
    admin: Address,
    min_submissions: u32,
    min_reputation: i128,
    max_drug_price_minor: i128,
    max_availability_units: u32,
) -> Result<(), Error> {
    utils::require_admin(&env, admin)?;

    if min_submissions == 0 || max_drug_price_minor <= 0 || max_availability_units == 0 {
        return Err(Error::InvalidData);
    }

    let mut config: Config = env
        .storage()
        .instance()
        .get(&DataKey::Config)
        .ok_or(Error::NotInitialized)?;

    config.min_submissions = min_submissions;
    config.min_reputation = min_reputation;
    config.max_drug_price_minor = max_drug_price_minor;
    config.max_availability_units = max_availability_units;

    env.storage().instance().set(&DataKey::Config, &config);
    Ok(())
}

pub fn add_arbiter(env: Env, admin: Address, arbiter: Address) -> Result<(), Error> {
    utils::require_admin(&env, admin)?;

    let mut config: Config = env
        .storage()
        .instance()
        .get(&DataKey::Config)
        .ok_or(Error::NotInitialized)?;

    if config.arbiters.contains(&arbiter) {
        return Err(Error::ArbiterExists);
    }

    config.arbiters.push_back(arbiter);
    env.storage().instance().set(&DataKey::Config, &config);
    Ok(())
}

pub fn get_config(env: Env) -> Option<Config> {
    env.storage().instance().get(&DataKey::Config)
}
