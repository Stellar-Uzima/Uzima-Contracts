//! Events emitted on every state change.
//!
//! Topics are `symbol_short!` so they stay inside the 9-byte topic limit, and
//! each state-changing entry point publishes exactly one event carrying the full
//! before/after delta. An off-chain indexer can therefore reconstruct asset and
//! balance state from the event stream alone, without replaying transactions.

use soroban_sdk::{symbol_short, Address, Env, String};

/// Emitted once, by `initialize`.
pub fn initialized(env: &Env, admin: &Address, treasury: &Address) {
    env.events()
        .publish((symbol_short!("init"),), (admin.clone(), treasury.clone()));
}

/// Emitted by `transfer_admin`.
pub fn admin_transferred(env: &Env, previous: &Address, current: &Address) {
    env.events().publish(
        (symbol_short!("adm_xfer"),),
        (previous.clone(), current.clone()),
    );
}

/// Emitted by `set_treasury_address`.
pub fn treasury_updated(env: &Env, treasury: &Address) {
    env.events()
        .publish((symbol_short!("treasury"),), (treasury.clone(),));
}

/// Emitted by `pause` / `unpause`; `paused` carries the resulting state.
pub fn pause_state_changed(env: &Env, paused: bool) {
    env.events().publish((symbol_short!("pause"),), paused);
}

/// Emitted by `register_asset` with the supply cap the asset was created with.
pub fn asset_registered(
    env: &Env,
    asset_id: &String,
    issuer: &Address,
    supply_cap: u128,
    decimals: u32,
) {
    env.events().publish(
        (symbol_short!("asset"), symbol_short!("reg")),
        (
            asset_id.clone(),
            issuer.clone(),
            supply_cap,
            decimals,
            env.ledger().timestamp(),
        ),
    );
}

/// Emitted by `mint` with the post-mint supply and the credited balance.
pub fn tokens_minted(
    env: &Env,
    asset_id: &String,
    to: &Address,
    amount: u128,
    new_supply: u128,
    new_balance: u128,
) {
    env.events().publish(
        (symbol_short!("asset"), symbol_short!("mint")),
        (
            asset_id.clone(),
            to.clone(),
            amount,
            new_supply,
            new_balance,
            env.ledger().timestamp(),
        ),
    );
}

/// Emitted by `burn` with the post-burn supply and the debited balance.
pub fn tokens_burned(
    env: &Env,
    asset_id: &String,
    from: &Address,
    amount: u128,
    new_supply: u128,
    new_balance: u128,
) {
    env.events().publish(
        (symbol_short!("asset"), symbol_short!("burn")),
        (
            asset_id.clone(),
            from.clone(),
            amount,
            new_supply,
            new_balance,
            env.ledger().timestamp(),
        ),
    );
}

/// Emitted by `transfer` with both sides of the resulting balance delta.
pub fn tokens_transferred(
    env: &Env,
    asset_id: &String,
    from: &Address,
    to: &Address,
    amount: u128,
    from_balance: u128,
    to_balance: u128,
) {
    env.events().publish(
        (symbol_short!("asset"), symbol_short!("xfer")),
        (
            asset_id.clone(),
            from.clone(),
            to.clone(),
            amount,
            from_balance,
            to_balance,
            env.ledger().timestamp(),
        ),
    );
}