use crate::access_control::{
    bootstrap_roles, grant_role, has_role, renounce_role, require_admin, require_role,
    revoke_role, role_members, DEFAULT_ADMIN_ROLE, ISSUER_ROLE,
};
use crate::errors::Error;
use crate::storage::*;
use crate::types::{Asset, DataKey};

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
        bootstrap_roles(&env, &admin);

        Ok(())
    }

    /// # Access control

    /// Grant `role` to `account`. Requires the default admin role.
    pub fn grant_role(env: Env, account: Address, role: u32) -> Result<(), Error> {
        let admin = caller_of(&env)?;
        require_admin(&env, &admin)?;
        grant_role(&env, &admin, &account, role)
    }

    /// Revoke `role` from `account`. Requires the default admin role.
    pub fn revoke_role(env: Env, account: Address, role: u32) -> Result<(), Error> {
        let admin = caller_of(&env)?;
        require_admin(&env, &admin)?;
        revoke_role(&env, &admin, &account, role)
    }

    /// Give up every role the caller holds.
    pub fn renounce_role(env: Env) -> Result<(), Error> {
        let caller = caller_of(&env)?;
        caller.require_auth();
        renounce_role(&env, &caller);
        Ok(())
    }

    /// Whether `account` holds `role`.
    pub fn has_role(env: Env, account: Address, role: u32) -> Result<bool, Error> {
        if !is_initialized(&env) {
            return Err(Error::NotInitialized);
        }
        Ok(has_role(&env, &account, role))
    }

    /// List every address holding `role`.
    pub fn role_members(env: Env, role: u32) -> Result<soroban_sdk::Vec<Address>, Error> {
        if !is_initialized(&env) {
            return Err(Error::NotInitialized);
        }
        Ok(role_members(&env, role))
    }

    /// # Administration

    /// Hand administration to a new address.
    ///
    /// Rotates the `DEFAULT_ADMIN_ROLE` holder rather than writing the admin
    /// slot directly, so the role index and the `admin()` mirror cannot drift.
    pub fn transfer_admin(env: Env, new_admin: Address) -> Result<(), Error> {
        let admin = caller_of(&env)?;
        require_admin(&env, &admin)?;

        grant_role(&env, &admin, &new_admin, DEFAULT_ADMIN_ROLE)?;

        let previous = get_admin(&env);
        if previous != new_admin {
            revoke_role(&env, &admin, &previous, DEFAULT_ADMIN_ROLE)?;
        }
        set_admin(&env, &new_admin);
        Ok(())
    }

    /// Set the treasury address that receives asset issuance fees.
    pub fn set_treasury_address(env: Env, treasury: Address) -> Result<(), Error> {
        require_admin_role(&env)?;
        set_treasury(&env, &treasury);
        Ok(())
    }

    /// Halt every state-changing operation.
    pub fn pause(env: Env) -> Result<(), Error> {
        require_admin_role(&env)?;
        set_paused(&env, true);
        Ok(())
    }

    /// Resume state-changing operations.
    pub fn unpause(env: Env) -> Result<(), Error> {
        require_admin_role(&env)?;
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
        require_admin_role(&env)?;
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
        set_asset_count(&env, count + 1);

        Ok(())
    }

    /// Mint `amount` units of `asset_id` to `to`.
    ///
    /// Requires the `ISSUER` role **and** that the caller is the asset's own
    /// issuer. The role check is what makes minting a governance-gated action;
    /// the asset check is what stops a contract-wide issuer from minting an
    /// asset it does not own.
    pub fn mint(env: Env, asset_id: String, to: Address, amount: u128) -> Result<(), Error> {
        ensure_running(&env)?;

        let mut asset = asset_of(&env, &asset_id)?;
        only_asset_issuer(&env, &asset)?;

        let balance = get_balance(&env, &asset_id, &to);
        let new_supply = asset
            .total_supply
            .checked_add(amount)
            .ok_or(Error::ArithmeticOverflow)?;
        let new_balance = balance
            .checked_add(amount)
            .ok_or(Error::ArithmeticOverflow)?;

        asset.total_supply = new_supply;
        set_asset(&env, &asset);
        set_balance(&env, &asset_id, &to, new_balance);

        Ok(())
    }

    /// Burn `amount` units held by `from`, reducing total supply.
    pub fn burn(env: Env, asset_id: String, from: Address, amount: u128) -> Result<(), Error> {
        ensure_running(&env)?;

        let mut asset = asset_of(&env, &asset_id)?;
        only_asset_issuer(&env, &asset)?;

        let balance = get_balance(&env, &asset_id, &from);
        if balance < amount {
            return Err(Error::InsufficientBalance);
        }

        asset.total_supply = asset
            .total_supply
            .checked_sub(amount)
            .ok_or(Error::ArithmeticUnderflow)?;
        set_asset(&env, &asset);
        set_balance(
            &env,
            &asset_id,
            &from,
            balance
                .checked_sub(amount)
                .ok_or(Error::ArithmeticUnderflow)?,
        );

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
        let _ = asset_of(&env, &asset_id)?;

        let from_balance = get_balance(&env, &asset_id, &from);
        if from_balance < amount {
            return Err(Error::InsufficientBalance);
        }
        let to_balance = get_balance(&env, &asset_id, &to);

        set_balance(
            &env,
            &asset_id,
            &from,
            from_balance
                .checked_sub(amount)
                .ok_or(Error::ArithmeticUnderflow)?,
        );
        set_balance(
            &env,
            &asset_id,
            &to,
            to_balance
                .checked_add(amount)
                .ok_or(Error::ArithmeticOverflow)?,
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

/// The authenticated caller, resolved from the admin slot's authorization.
///
/// Soroban does not expose "who called this" directly, so admin-gated entry
/// points authenticate the admin address and then confirm it actually holds the
/// default admin role. That double check is deliberate: it is what stops the
/// `admin()` mirror slot and the role index from silently disagreeing.
fn caller_of(env: &Env) -> Result<Address, Error> {
    let admin = admin_of(env)?;
    admin.require_auth();
    Ok(admin)
}

fn require_admin_role(env: &Env) -> Result<(), Error> {
    let admin = caller_of(env)?;
    require_admin(env, &admin)
}

/// Gate `mint` / `burn` on the asset's own issuer *and* on that issuer holding
/// the contract-wide `ISSUER_ROLE`.
///
/// The two checks are not redundant. `Asset.issuer` says who owns the asset;
/// `ISSUER_ROLE` says who is allowed to create supply on this contract at all.
/// Without the second, registering an asset with an arbitrary issuer address
/// would hand that address unbounded mint power over the contract.
fn only_asset_issuer(env: &Env, asset: &Asset) -> Result<(), Error> {
    let issuer = asset.issuer.clone();
    issuer.require_auth();
    require_role(env, &issuer, ISSUER_ROLE)
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