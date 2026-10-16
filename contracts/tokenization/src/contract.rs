use crate::errors::Error;
use crate::events;
use crate::storage::*;
use crate::types::{Asset, DataKey};
use crate::validation::*;

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

        events::initialized(&env, &admin, &treasury);
        Ok(())
    }

    /// # Administration

    /// Hand administration to a new address.
    pub fn transfer_admin(env: Env, new_admin: Address) -> Result<(), Error> {
        let admin = admin_of(&env)?;
        admin.require_auth();

        set_admin(&env, &new_admin);
        events::admin_transferred(&env, &admin, &new_admin);
        Ok(())
    }

    /// Set the treasury address that receives asset issuance fees.
    pub fn set_treasury_address(env: Env, treasury: Address) -> Result<(), Error> {
        let admin = admin_of(&env)?;
        admin.require_auth();

        set_treasury(&env, &treasury);
        events::treasury_updated(&env, &treasury);
        Ok(())
    }

    /// Halt every state-changing operation.
    pub fn pause(env: Env) -> Result<(), Error> {
        let admin = admin_of(&env)?;
        admin.require_auth();

        set_paused(&env, true);
        events::pause_state_changed(&env, true);
        Ok(())
    }

    /// Resume state-changing operation.
    pub fn unpause(env: Env) -> Result<(), Error> {
        let admin = admin_of(&env)?;
        admin.require_auth();

        set_paused(&env, false);
        events::pause_state_changed(&env, false);
        Ok(())
    }

    /// # Asset lifecycle

    /// Register a new tokenized asset with a fixed maximum supply.
    pub fn register_asset(
        env: Env,
        asset_id: String,
        issuer: Address,
        supply_cap: u128,
        decimals: u32,
        metadata_uri: String,
    ) -> Result<(), Error> {
        validate_asset_id(&asset_id)?;
        validate_decimals(decimals)?;
        validate_metadata_uri(&metadata_uri)?;
        if supply_cap == 0 {
            return Err(Error::ZeroAmount);
        }

        require_admin(&env)?;
        ensure_running(&env)?;

        if has_asset(&env, &asset_id) {
            return Err(Error::AssetAlreadyExists);
        }

        // One ledger read for the admin lookup, reused by `require_admin`.
        let asset = Asset {
            asset_id: asset_id.clone(),
            issuer: issuer.clone(),
            total_supply: 0,
            decimals,
            metadata_uri,
            created_at: env.ledger().timestamp(),
            // Held alongside the record so `mint` never has to re-read it.
            supply_cap,
        };
        set_asset(&env, &asset);

        set_asset_count(
            &env,
            get_asset_count(&env)
                .checked_add(1)
                .ok_or(Error::ArithmeticOverflow)?,
        );

        events::asset_registered(&env, &asset_id, &issuer, supply_cap, decimals);
        Ok(())
    }

    /// Mint `amount` units of `asset_id` to `to`, bounded by the registered cap.
    pub fn mint(env: Env, asset_id: String, to: Address, amount: u128) -> Result<(), Error> {
        validate_amount(amount)?;

        require_admin(&env)?;
        ensure_running(&env)?;

        // One asset read serves both the existence check, the cap check and the
        // write-back; a second read here would double the ledger cost of mint.
        let mut asset = asset_of(&env, &asset_id)?;
        asset.issuer.require_auth();

        validate_supply_cap(asset.total_supply, amount, asset.supply_cap)?;

        let new_supply = asset
            .total_supply
            .checked_add(amount)
            .ok_or(Error::ArithmeticOverflow)?;
        let balance = get_balance(&env, &asset_id, &to);
        let new_balance = balance
            .checked_add(amount)
            .ok_or(Error::ArithmeticOverflow)?;

        asset.total_supply = new_supply;
        set_asset(&env, &asset);
        set_balance(&env, &asset_id, &to, new_balance);

        events::tokens_minted(&env, &asset_id, &to, amount, new_supply, new_balance);
        Ok(())
    }

    /// Burn `amount` units held by `from`, reducing total supply.
    pub fn burn(env: Env, asset_id: String, from: Address, amount: u128) -> Result<(), Error> {
        validate_amount(amount)?;

        require_admin(&env)?;
        ensure_running(&env)?;

        let mut asset = asset_of(&env, &asset_id)?;
        asset.issuer.require_auth();

        let balance = get_balance(&env, &asset_id, &from);
        validate_debit(balance, amount)?;

        let new_balance = balance
            .checked_sub(amount)
            .ok_or(Error::ArithmeticUnderflow)?;
        let new_supply = asset
            .total_supply
            .checked_sub(amount)
            .ok_or(Error::ArithmeticUnderflow)?;

        asset.total_supply = new_supply;
        set_asset(&env, &asset);
        set_balance(&env, &asset_id, &from, new_balance);

        events::tokens_burned(&env, &asset_id, &from, amount, new_supply, new_balance);
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
        validate_amount(amount)?;
        validate_transfer_parties(&from, &to)?;
        ensure_running(&env)?;

        from.require_auth();
        let _ = asset_of(&env, &asset_id)?;

        let from_balance = get_balance(&env, &asset_id, &from);
        validate_debit(from_balance, amount)?;

        let new_from = from_balance
            .checked_sub(amount)
            .ok_or(Error::ArithmeticUnderflow)?;
        let to_balance = get_balance(&env, &asset_id, &to);
        let new_to = to_balance
            .checked_add(amount)
            .ok_or(Error::ArithmeticOverflow)?;

        // Total supply is untouched, so the asset record is not rewritten.
        set_balance(&env, &asset_id, &from, new_from);
        set_balance(&env, &asset_id, &to, new_to);

        events::tokens_transferred(
            &env, &asset_id, &from, &to, amount, new_from, new_to,
        );
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

    /// Return the registered supply cap of `asset_id`.
    pub fn supply_cap(env: Env, asset_id: String) -> Result<u128, Error> {
        Ok(asset_of(&env, &asset_id)?.supply_cap)
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
    admin_of(env)?.require_auth();
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