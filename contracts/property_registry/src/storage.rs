use crate::types::{DataKey, Property, BASIS_POINTS_SCALE};
use soroban_sdk::{Address, Env, String};

/// Whether `initialize` has run.
///
/// Checking `Admin` rather than a dedicated flag means a half-initialized
/// contract can never look initialized.
pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

/// The admin of record. Panics if called before `initialize`; every caller
/// goes through `admin_of` in `contract.rs`, which returns `NotInitialized`.
pub fn get_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .expect("admin missing")
}

/// Write the admin of record. Used only by `initialize` and `transfer_admin`.
pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

/// The registrar of record. Panics before `initialize`.
pub fn get_registrar(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::Registrar)
        .expect("registrar missing")
}

/// Write the registrar of record. Used only by `initialize` and `set_registrar`.
pub fn set_registrar(env: &Env, registrar: &Address) {
    env.storage().instance().set(&DataKey::Registrar, registrar);
}

/// Whether state-changing operations are halted. Defaults to `false`.
pub fn is_paused(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::Paused)
        .unwrap_or(false)
}

/// Write the pause switch.
pub fn set_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&DataKey::Paused, &paused);
}

/// How many properties are registered. Defaults to `0`.
pub fn get_property_count(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::PropertyCount)
        .unwrap_or(0)
}

/// Write the property counter.
pub fn set_property_count(env: &Env, count: u32) {
    env.storage().instance().set(&DataKey::PropertyCount, &count);
}

/// Whether a property exists under `property_id`.
///
/// A `has` rather than a `get`, so the duplicate check on `register_property`
/// costs one read instead of one read plus an `Option` match.
pub fn has_property(env: &Env, property_id: &String) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Property(property_id.clone()))
}

/// The property record, or `None` when `property_id` is unregistered.
pub fn get_property(env: &Env, property_id: &String) -> Option<Property> {
    env.storage()
        .persistent()
        .get(&DataKey::Property(property_id.clone()))
}

/// Write the property record.
pub fn set_property(env: &Env, property: &Property) {
    env.storage()
        .persistent()
        .set(&DataKey::Property(property.property_id.clone()), property);
}

/// A holder's share in basis points. Defaults to `0`, so an unknown holder is
/// indistinguishable from one holding nothing — which is what
/// `ownership_of` documents.
pub fn get_ownership(env: &Env, property_id: &String, holder: &Address) -> u128 {
    env.storage()
        .persistent()
        .get(&DataKey::Ownership(property_id.clone(), holder.clone()))
        .unwrap_or(0)
}

/// Write a holder's share. Writing `0` is a real write, not a removal: the entry
/// stays addressable so an indexer can see that the holder once had a position.
pub fn set_ownership(env: &Env, property_id: &String, holder: &Address, amount: u128) {
    env.storage().persistent().set(
        &DataKey::Ownership(property_id.clone(), holder.clone()),
        &amount,
    );
}