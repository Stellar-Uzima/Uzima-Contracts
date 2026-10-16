use crate::checked_math::{checked_add, checked_sub};
use crate::errors::Error;
use crate::storage::*;
use crate::types::{Asset, DataKey, STORAGE_GAP_SLOTS, STORAGE_SCHEMA_VERSION};

use soroban_sdk::{contract, contractimpl, contractmeta, Address, Env, String};

contractmeta!(
    key = "Description",
    val = "Tokenization registry for fractional ownership of on-chain assets"
);

/// Ledger seconds a persistent entry is extended to on every write.
const LIFETIME_THRESHOLD: u32 = 100_000;
/// Ledger count a persistent entry is extended to on every write (~30 days).
const LIFETIME_EXTEND_TO: u32 = 2_592_000;

#[contract]
pub struct TokenizationContract;

#[contractimpl]
impl TokenizationContract {
    /// # Initialization

    /// Initialize the registry. Callable exactly once.
    pub fn initialize(env: Env, admin: Address, treasury: Address) -> Result<(), Error> {
        if is_initialized(&env) {
            return Err(Error::AlreadyInitialized);
        }

        admin.require_auth();

        set_admin(&env, &admin);
        set_treasury(&env, &treasury);
        set_paused(&env, false);
        set_asset_count(&env, 0);
        set_storage_version(&env);

        Ok(())
    }

    /// # Administration

    /// Hand administration to a new address.
    pub fn transfer_admin(env: Env, new_admin: Address) -> Result<(), Error> {
        let admin = admin_of(&env)?;
        admin.require_auth();

        set_admin(&env, &new_admin);
        Ok(())
    }

    /// Set the treasury address that receives asset issuance fees.
    pub fn set_treasury_address(env: Env, treasury: Address) -> Result<(), Error> {
        let admin = admin_of(&env)?;
        admin.require_auth();

        set_treasury(&env, &treasury);
        Ok(())
    }

    /// Halt every state-changing operation.
    pub fn pause(env: Env) -> Result<(), Error> {
        let admin = admin_of(&env)?;
        admin.require_auth();

        set_paused(&env, true);
        Ok(())
    }

    /// Resume state-changing operations.
    pub fn unpause(env: Env) -> Result<(), Error> {
        let admin = admin_of(&env)?;
        admin.require_auth();

        set_paused(&env, false);
        Ok(())
    }

    /// # Asset lifecycle

    /// Register a new tokenized asset with a fixed maximum supply.
    pub fn register_asset(
        env: Env,
        asset_id: String,
        issuer: Address,
        total_supply: u128,
        decimals: u32,
        metadata_uri: String,
    ) -> Result<(), Error> {
        require_admin(&env)?;
        ensure_running(&env)?;

        if has_asset(&env, &asset_id) {
            return Err(Error::AssetAlreadyExists);
        }

        let asset = Asset {
            asset_id: asset_id.clone(),
            issuer,
            total_supply,
            decimals,
            metadata_uri,
            created_at: env.ledger().timestamp(),
        };
        set_asset(&env, &asset);

        let count = get_asset_count(&env);
        set_asset_count(&env, count.checked_add(1).ok_or(Error::ArithmeticOverflow)?);

        Ok(())
    }

    /// Mint `amount` units of `asset_id` to `to`, bounded by the registered cap.
    pub fn mint(env: Env, asset_id: String, to: Address, amount: u128) -> Result<(), Error> {
        require_admin(&env)?;
        ensure_running(&env)?;

        let mut asset = asset_of(&env, &asset_id)?;
        asset.issuer.require_auth();

        asset.total_supply = checked_add(asset.total_supply, amount)?;
        set_asset(&env, &asset);

        let balance = get_balance(&env, &asset_id, &to);
        set_balance(&env, &asset_id, &to, checked_add(balance, amount)?);

        Ok(())
    }

    /// Burn `amount` units held by `from`, reducing total supply.
    pub fn burn(env: Env, asset_id: String, from: Address, amount: u128) -> Result<(), Error> {
        require_admin(&env)?;
        ensure_running(&env)?;

        let mut asset = asset_of(&env, &asset_id)?;
        asset.issuer.require_auth();

        let balance = get_balance(&env, &asset_id, &from);
        if balance < amount {
            return Err(Error::InsufficientBalance);
        }
        set_balance(&env, &asset_id, &from, checked_sub(balance, amount)?);

        asset.total_supply = checked_sub(asset.total_supply, amount)?;
        set_asset(&env, &asset);

        Ok(())
    }

    /// Move `amount` units from `from` to `to`.
    pub fn transfer(
        env: Env,
        asset_id: String,
        from: Address,
        to: Address,
        amount: u128,
    ) -> Result<(), Error> {
        ensure_running(&env)?;

        from.require_auth();
        asset_of(&env, &asset_id)?;

        let from_balance = get_balance(&env, &asset_id, &from);
        if from_balance < amount {
            return Err(Error::InsufficientBalance);
        }
        set_balance(
            &env,
            &asset_id,
            &from,
            checked_sub(from_balance, amount)?,
        );

        let to_balance = get_balance(&env, &asset_id, &to);
        set_balance(&env, &asset_id, &to, checked_add(to_balance, amount)?);

        Ok(())
    }

    /// # Queries

    /// Return the full record for `asset_id`.
    pub fn get_asset(env: Env, asset_id: String) -> Result<Asset, Error> {
        asset_of(&env, &asset_id)
    }

    /// Return `holder`'s balance of `asset_id`.
    pub fn balance_of(env: Env, asset_id: String, holder: Address) -> u128 {
        get_balance(&env, &asset_id, &holder)
    }

    /// Return the circulating supply of `asset_id`.
    pub fn total_supply(env: Env, asset_id: String) -> Result<u128, Error> {
        Ok(asset_of(&env, &asset_id)?.total_supply)
    }

    /// Return how many assets are registered.
    pub fn asset_count(env: Env) -> u32 {
        get_asset_count(&env)
    }

    /// Return the current administrator.
    pub fn admin(env: Env) -> Result<Address, Error> {
        admin_of(&env)
    }

    /// Return the current treasury address.
    pub fn treasury(env: Env) -> Result<Address, Error> {
        if !is_initialized(&env) {
            return Err(Error::NotInitialized);
        }
        Ok(get_treasury(&env))
    }

    /// Return whether state-changing operations are halted.
    pub fn paused(env: Env) -> bool {
        is_paused(&env)
    }

    /// # Upgrade safety

    /// Return the storage layout version stamped at initialization.
    ///
    /// `0` means the deployment predates the marker and every migration step
    /// must run before the next WASM is installed.
    pub fn storage_version(env: Env) -> u32 {
        get_storage_version(&env)
    }

    /// Return how many `StorageGap` slot indices are held back for upgrades.
    pub fn storage_gap_slots() -> u32 {
        STORAGE_GAP_SLOTS
    }

    /// Return whether `slot` is inside the reserved upgrade gap.
    pub fn is_reserved_gap_slot(_env: Env, slot: u32) -> bool {
        is_reserved_gap_slot(slot)
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn admin_of(env: &Env) -> Result<Address, Error> {
    if !is_initialized(env) {
        return Err(Error::NotInitialized);
    }
    Ok(get_admin(env))
}

fn require_admin(env: &Env) -> Result<(), Error> {
    let admin = admin_of(env)?;
    admin.require_auth();
    Ok(())
}

fn ensure_running(env: &Env) -> Result<(), Error> {
    if is_paused(env) {
        return Err(Error::Unauthorized);
    }
    Ok(())
}

fn asset_of(env: &Env, asset_id: &String) -> Result<Asset, Error> {
    if !is_initialized(env) {
        return Err(Error::NotInitialized);
    }
    get_asset(env, asset_id).ok_or(Error::AssetNotFound)
}

/// Extend the TTL of every persistent key touched by `asset_id`.
#[allow(dead_code)]
fn bump_ttl(env: &Env, asset_id: &String, holder: &Address) {
    env.storage().persistent().extend_ttl(
        &DataKey::Asset(asset_id.clone()),
        LIFETIME_THRESHOLD,
        LIFETIME_EXTEND_TO,
    );
    env.storage().persistent().extend_ttl(
        &DataKey::Balance(asset_id.clone(), holder.clone()),
        LIFETIME_THRESHOLD,
        LIFETIME_EXTEND_TO,
    );
}