use crate::types::{Asset, DataKey};
use soroban_sdk::{Address, Env, String};

pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

pub fn get_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .expect("admin missing")
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_treasury(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::Treasury)
        .expect("treasury missing")
}

pub fn set_treasury(env: &Env, treasury: &Address) {
    env.storage().instance().set(&DataKey::Treasury, treasury);
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

pub fn get_asset_count(env: &Env) -> u32 {
    env.storage().instance().get(&DataKey::AssetCount).unwrap_or(0)
}

pub fn set_asset_count(env: &Env, count: u32) {
    env.storage().instance().set(&DataKey::AssetCount, &count);
}

pub fn has_asset(env: &Env, asset_id: &String) -> bool {
    env.storage().persistent().has(&DataKey::Asset(asset_id.clone()))
}

pub fn get_asset(env: &Env, asset_id: &String) -> Option<Asset> {
    env.storage()
        .persistent()
        .get(&DataKey::Asset(asset_id.clone()))
}

pub fn set_asset(env: &Env, asset: &Asset) {
    env.storage()
        .persistent()
        .set(&DataKey::Asset(asset.asset_id.clone()), asset);
}

pub fn get_balance(env: &Env, asset_id: &String, holder: &Address) -> u128 {
    env.storage()
        .persistent()
        .get(&DataKey::Balance(asset_id.clone(), holder.clone()))
        .unwrap_or(0)
}

pub fn set_balance(env: &Env, asset_id: &String, holder: &Address, amount: u128) {
    env.storage().persistent().set(
        &DataKey::Balance(asset_id.clone(), holder.clone()),
        &amount,
    );
}