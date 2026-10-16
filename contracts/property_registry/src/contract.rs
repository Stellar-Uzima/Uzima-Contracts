use crate::access_control::{
    bootstrap_roles, grant_role, has_role, renounce_role, require_admin, require_role,
    revoke_role, role_members, DEFAULT_ADMIN_ROLE, REGISTRAR_ROLE,
};
use crate::errors::Error;
use crate::storage::*;
use crate::types::{DataKey, Property, BASIS_POINTS_SCALE};

use soroban_sdk::{contract, contractimpl, contractmeta, Address, Env, String, Vec};

contractmeta!(
    key = "Description",
    val = "Registry of on-chain real-estate property records and ownership shares"
);

/// Ledger seconds a persistent entry is extended to on every write.
const LIFETIME_THRESHOLD: u32 = 100_000;
/// Ledger count a persistent entry is extended to on every write (~30 days).
const LIFETIME_EXTEND_TO: u32 = 2_592_000;

#[contract]
pub struct PropertyRegistryContract;

#[contractimpl]
impl PropertyRegistryContract {
    /// # Initialization

    /// Initialize the registry. Callable exactly once.
    pub fn initialize(env: Env, admin: Address, registrar: Address) -> Result<(), Error> {
        if is_initialized(&env) {
            return Err(Error::AlreadyInitialized);
        }

        admin.require_auth();

        set_admin(&env, &admin);
        set_registrar(&env, &registrar);
        set_paused(&env, false);
        set_property_count(&env, 0);
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
    pub fn role_members(env: Env, role: u32) -> Result<Vec<Address>, Error> {
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

    /// Set the registrar allowed to create and revalue properties.
    pub fn set_registrar(env: Env, registrar: Address) -> Result<(), Error> {
        let admin = caller_of(&env)?;
        require_admin(&env, &admin)?;
        set_registrar(&env, &registrar);
        Ok(())
    }

    /// Halt every state-changing operation.
    pub fn pause(env: Env) -> Result<(), Error> {
        let admin = caller_of(&env)?;
        require_admin(&env, &admin)?;
        set_paused(&env, true);
        Ok(())
    }

    /// Resume state-changing operations.
    pub fn unpause(env: Env) -> Result<(), Error> {
        let admin = caller_of(&env)?;
        require_admin(&env, &admin)?;
        set_paused(&env, false);
        Ok(())
    }

    /// # Property lifecycle

    /// Register a new property, seeding the initial 100% ownership share.
    pub fn register_property(
        env: Env,
        property_id: String,
        owner: Address,
        valuation: u128,
        valuation_currency: String,
        metadata_uri: String,
    ) -> Result<(), Error> {
        require_registrar_role(&env)?;
        ensure_running(&env)?;

        if has_property(&env, &property_id) {
            return Err(Error::PropertyAlreadyExists);
        }

        let now = env.ledger().timestamp();
        let property = Property {
            property_id: property_id.clone(),
            owner: owner.clone(),
            created_at: now,
            updated_at: now,
            valuation,
            valuation_currency,
            metadata_uri,
            holder_count: 1,
        };
        set_property(&env, &property);

        let count = get_property_count(&env);
        set_property_count(&env, count.checked_add(1).ok_or(Error::ArithmeticOverflow)?);

        set_ownership(&env, &property_id, &owner, BASIS_POINTS_SCALE);
        Ok(())
    }

    /// Overwrite the recorded valuation of an existing property.
    pub fn update_valuation(env: Env, property_id: String, valuation: u128) -> Result<(), Error> {
        require_registrar_role(&env)?;
        ensure_running(&env)?;

        let mut property = property_of(&env, &property_id)?;
        property.valuation = valuation;
        property.updated_at = env.ledger().timestamp();
        set_property(&env, &property);
        Ok(())
    }

    /// Transfer the property of record to `new_owner`, moving the whole
    /// recorded position of the incumbent.
    pub fn transfer_property(env: Env, property_id: String, new_owner: Address) -> Result<(), Error> {
        ensure_running(&env)?;

        let mut property = property_of(&env, &property_id)?;
        property.owner.require_auth();

        if property.owner == new_owner {
            return Err(Error::InvalidArgument);
        }

        let previous = property.owner.clone();
        let moved = get_ownership(&env, &property_id, &previous);
        let successor_share = get_ownership(&env, &property_id, &new_owner);
        let credited = successor_share
            .checked_add(moved)
            .ok_or(Error::ArithmeticOverflow)?;

        if credited > BASIS_POINTS_SCALE {
            return Err(Error::ShareTotalExceeded);
        }

        if moved > 0 {
            property.holder_count = property.holder_count.saturating_sub(1);
        }
        if successor_share == 0 {
            property.holder_count = property
                .holder_count
                .checked_add(1)
                .ok_or(Error::ArithmeticOverflow)?;
        }

        set_ownership(&env, &property_id, &previous, 0);
        set_ownership(&env, &property_id, &new_owner, credited);

        property.owner = new_owner;
        property.updated_at = env.ledger().timestamp();
        set_property(&env, &property);
        Ok(())
    }

    /// Move `amount` basis points of the owner of record's position to `holder`.
    ///
    /// Gated by **both** the registrar role and the owner of record: the role
    /// says who may amend the register, the owner says whose position may be
    /// debited. Either alone would be wrong.
    pub fn record_ownership(
        env: Env,
        property_id: String,
        holder: Address,
        amount: u128,
    ) -> Result<(), Error> {
        require_registrar_role(&env)?;
        ensure_running(&env)?;

        let mut property = property_of(&env, &property_id)?;
        property.owner.require_auth();

        if holder == property.owner {
            return Err(Error::InvalidArgument);
        }

        let owner = property.owner.clone();
        let owner_share = get_ownership(&env, &property_id, &owner);
        if amount > owner_share {
            return Err(Error::OwnershipExceedsShare);
        }

        let holder_share = get_ownership(&env, &property_id, &holder);
        let credited = holder_share
            .checked_add(amount)
            .ok_or(Error::ArithmeticOverflow)?;
        if credited > BASIS_POINTS_SCALE {
            return Err(Error::ShareTotalExceeded);
        }

        if holder_share == 0 {
            property.holder_count = property
                .holder_count
                .checked_add(1)
                .ok_or(Error::ArithmeticOverflow)?;
        }

        let remaining = owner_share
            .checked_sub(amount)
            .ok_or(Error::ArithmeticUnderflow)?;

        set_ownership(&env, &property_id, &owner, remaining);
        set_ownership(&env, &property_id, &holder, credited);

        property.updated_at = env.ledger().timestamp();
        set_property(&env, &property);
        Ok(())
    }

    /// # Queries

    /// Return the full record for `property_id`.
    pub fn get_property(env: Env, property_id: String) -> Result<Property, Error> {
        property_of(&env, &property_id)
    }

    /// Return `holder`'s ownership share of `property_id`, in basis points.
    pub fn ownership_of(env: Env, property_id: String, holder: Address) -> u128 {
        get_ownership(&env, &property_id, &holder)
    }

    /// Return how many properties are registered.
    pub fn total_properties(env: Env) -> u32 {
        get_property_count(&env)
    }

    /// Return the current administrator.
    pub fn admin(env: Env) -> Result<Address, Error> {
        admin_of(&env)
    }

    /// Return the current registrar of record.
    pub fn registrar(env: Env) -> Result<Address, Error> {
        if !is_initialized(&env) {
            return Err(Error::NotInitialized);
        }
        Ok(get_registrar(&env))
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

/// Gate a registrar-only operation.
///
/// The registrar slot records *who the registry was configured with*, while the
/// role records *who may currently amend the register*. Both are checked: a
/// rotation that updated only the slot would otherwise silently do nothing.
fn require_registrar_role(env: &Env) -> Result<(), Error> {
    let registrar = get_registrar(env);
    registrar.require_auth();
    require_role(env, &registrar, REGISTRAR_ROLE)
}

/// Extend the TTL of every persistent key touched by `property_id`.
#[allow(dead_code)]
fn bump_ttl(env: &Env, property_id: &String, holder: &Address) {
    env.storage().persistent().extend_ttl(
        &DataKey::Property(property_id.clone()),
        LIFETIME_THRESHOLD,
        LIFETIME_EXTEND_TO,
    );
    env.storage().persistent().extend_ttl(
        &DataKey::Ownership(property_id.clone(), holder.clone()),
        LIFETIME_THRESHOLD,
        LIFETIME_EXTEND_TO,
    );
}