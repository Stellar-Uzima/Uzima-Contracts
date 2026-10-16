use crate::types::{Asset, DataKey, STORAGE_GAP_SLOTS, STORAGE_SCHEMA_VERSION};
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

// ---------------------------------------------------------------------------
// Upgrade-safety layout markers
// ---------------------------------------------------------------------------

/// Layout version stamped into instance storage at initialization.
pub fn set_storage_version(env: &Env) {
    env.storage()
        .instance()
        .set(&DataKey::StorageVersion, &STORAGE_SCHEMA_VERSION);
}

/// Layout version recorded on-chain.
///
/// Returns `0` for a deployment that predates the marker, which upgrade tooling
/// treats as "run every migration step".
pub fn get_storage_version(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::StorageVersion)
        .unwrap_or(0)
}

/// Whether `slot` falls inside the reserved upgrade gap.
///
/// Slots outside the gap are *not* free to claim without bumping
/// [`STORAGE_SCHEMA_VERSION`], because today they are either live keys or
/// outside the reserved range entirely.
pub fn is_reserved_gap_slot(slot: u32) -> bool {
    slot < STORAGE_GAP_SLOTS
}

// ---------------------------------------------------------------------------
// Asset records and balances
// ---------------------------------------------------------------------------

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