//! Events emitted on every state change.
//!
//! Topics are `symbol_short!` so they stay inside the 9-byte topic limit, and
//! each state-changing entry point publishes exactly one event carrying the full
//! before/after delta. An off-chain indexer can therefore reconstruct the
//! property register and the ownership split from the event stream alone,
//! without replaying transactions.

use soroban_sdk::{symbol_short, Address, Env, String};

/// Emitted once, by `initialize`.
pub fn initialized(env: &Env, admin: &Address, registrar: &Address) {
    env.events()
        .publish((symbol_short!("init"),), (admin.clone(), registrar.clone()));
}

/// Emitted by `transfer_admin`.
pub fn admin_transferred(env: &Env, previous: &Address, current: &Address) {
    env.events().publish(
        (symbol_short!("adm_xfer"),),
        (previous.clone(), current.clone()),
    );
}

/// Emitted by `set_registrar`.
pub fn registrar_updated(env: &Env, registrar: &Address) {
    env.events()
        .publish((symbol_short!("registrar"),), (registrar.clone(),));
}

/// Emitted by `pause` / `unpause`; `paused` carries the resulting state.
pub fn pause_state_changed(env: &Env, paused: bool) {
    env.events().publish((symbol_short!("pause"),), paused);
}

/// Emitted by `register_property` with the seeded valuation and holder count.
pub fn property_registered(
    env: &Env,
    property_id: &String,
    owner: &Address,
    valuation: u128,
    currency: &String,
) {
    env.events().publish(
        (symbol_short!("prop"), symbol_short!("reg")),
        (
            property_id.clone(),
            owner.clone(),
            valuation,
            currency.clone(),
            env.ledger().timestamp(),
        ),
    );
}

/// Emitted by `update_valuation` with both the old and the new valuation.
pub fn valuation_updated(env: &Env, property_id: &String, previous: u128, current: u128) {
    env.events().publish(
        (symbol_short!("prop"), symbol_short!("value")),
        (
            property_id.clone(),
            previous,
            current,
            env.ledger().timestamp(),
        ),
    );
}

/// Emitted by `transfer_property` with the outgoing and incoming positions.
pub fn property_transferred(
    env: &Env,
    property_id: &String,
    previous: &Address,
    current: &Address,
    moved_share: u128,
) {
    env.events().publish(
        (symbol_short!("prop"), symbol_short!("xfer")),
        (
            property_id.clone(),
            previous.clone(),
            current.clone(),
            moved_share,
            env.ledger().timestamp(),
        ),
    );
}

/// Emitted by `record_ownership` with the resulting share on both legs.
pub fn ownership_recorded(
    env: &Env,
    property_id: &String,
    holder: &Address,
    amount: u128,
    owner_share: u128,
    holder_share: u128,
) {
    env.events().publish(
        (symbol_short!("prop"), symbol_short!("share")),
        (
            property_id.clone(),
            holder.clone(),
            amount,
            owner_share,
            holder_share,
            env.ledger().timestamp(),
        ),
    );
}