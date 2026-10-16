use crate::types::{DataKey, Property, STORAGE_GAP_SLOTS, STORAGE_SCHEMA_VERSION};
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

pub fn get_registrar(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::Registrar)
        .expect("registrar missing")
}

pub fn set_registrar(env: &Env, registrar: &Address) {
    env.storage().instance().set(&DataKey::Registrar, registrar);
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

pub fn get_property_count(env: &Env) -> u32 {
    env.storage().instance().get(&DataKey::PropertyCount).unwrap_or(0)
}

pub fn set_property_count(env: &Env, count: u32) {
    env.storage().instance().set(&DataKey::PropertyCount, &count);
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
pub fn is_reserved_gap_slot(slot: u32) -> bool {
    slot < STORAGE_GAP_SLOTS
}

// ---------------------------------------------------------------------------
// Property records and ownership shares
// ---------------------------------------------------------------------------

pub fn has_property(env: &Env, property_id: &String) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Property(property_id.clone()))
}

pub fn get_property(env: &Env, property_id: &String) -> Option<Property> {
    env.storage()
        .persistent()
        .get(&DataKey::Property(property_id.clone()))
}

pub fn set_property(env: &Env, property: &Property) {
    env.storage()
        .persistent()
        .set(&DataKey::Property(property.property_id.clone()), property);
}

pub fn get_ownership(env: &Env, property_id: &String, holder: &Address) -> u128 {
    env.storage()
        .persistent()
        .get(&DataKey::Ownership(property_id.clone(), holder.clone()))
        .unwrap_or(0)
}

pub fn set_ownership(env: &Env, property_id: &String, holder: &Address, amount: u128) {
    env.storage().persistent().set(
        &DataKey::Ownership(property_id.clone(), holder.clone()),
        &amount,
    );
}