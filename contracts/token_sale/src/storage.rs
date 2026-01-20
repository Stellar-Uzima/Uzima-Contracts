use crate::types::*;
use soroban_sdk::{Address, Env};

pub fn get_config(env: &Env) -> SaleConfig {
    env.storage().instance().get(&DataKey::Config).unwrap()
}

pub fn set_config(env: &Env, config: &SaleConfig) {
    env.storage().instance().set(&DataKey::Config, config);
}

pub fn get_owner(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Owner).unwrap()
}

pub fn set_owner(env: &Env, owner: &Address) {
    env.storage().instance().set(&DataKey::Owner, owner);
}

pub fn is_paused(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::Paused)
        .unwrap_or(false)
}

pub fn set_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&DataKey::Paused, &paused);
}

pub fn get_total_raised(env: &Env) -> u128 {
    env.storage()
        .instance()
        .get(&DataKey::TotalRaised)
        .unwrap_or(0)
}

pub fn set_total_raised(env: &Env, amount: u128) {
    env.storage().instance().set(&DataKey::TotalRaised, &amount);
}

pub fn get_phase_count(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::PhaseCount)
        .unwrap_or(0)
}

pub fn set_phase_count(env: &Env, count: u32) {
    env.storage().instance().set(&DataKey::PhaseCount, &count);
}

pub fn get_sale_phase(env: &Env, phase_id: u32) -> Option<SalePhase> {
    env.storage().persistent().get(&DataKey::Phase(phase_id))
}

pub fn set_sale_phase(env: &Env, phase_id: u32, phase: &SalePhase) {
    env.storage()
        .persistent()
        .set(&DataKey::Phase(phase_id), phase);
}

pub fn get_contribution(env: &Env, user: &Address) -> Option<Contribution> {
    env.storage()
        .persistent()
        .get(&DataKey::Contribution(user.clone()))
}

pub fn set_contribution(env: &Env, user: &Address, contribution: &Contribution) {
    env.storage()
        .persistent()
        .set(&DataKey::Contribution(user.clone()), contribution);
}

pub fn get_phase_contribution(env: &Env, user: &Address, phase_id: u32) -> u128 {
    env.storage()
        .persistent()
        .get(&DataKey::PhaseContribution(user.clone(), phase_id))
        .unwrap_or(0)
}

pub fn set_phase_contribution(env: &Env, user: &Address, phase_id: u32, amount: u128) {
    env.storage()
        .persistent()
        .set(&DataKey::PhaseContribution(user.clone(), phase_id), &amount);
}

pub fn is_supported_token(env: &Env, token: &Address) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::SupportedToken(token.clone()))
        .unwrap_or(false)
}

pub fn set_supported_token(env: &Env, token: &Address, supported: bool) {
    env.storage()
        .persistent()
        .set(&DataKey::SupportedToken(token.clone()), &supported);
}

pub fn get_vesting_schedule(env: &Env, beneficiary: &Address) -> Option<VestingSchedule> {
    env.storage()
        .persistent()
        .get(&DataKey::VestingSchedule(beneficiary.clone()))
}

pub fn set_vesting_schedule(env: &Env, beneficiary: &Address, schedule: &VestingSchedule) {
    env.storage()
        .persistent()
        .set(&DataKey::VestingSchedule(beneficiary.clone()), schedule);
}

pub fn get_vesting_contract(_env: &Env) -> Option<Address> {
    _env.storage().instance().get(&DataKey::VestingContract)
}

#[allow(dead_code)]
pub fn set_vesting_contract(_env: &Env, _contract: &Address) {
    _env.storage()
        .instance()
        .set(&DataKey::VestingContract, _contract);
}
