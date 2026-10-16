use crate::checked_math::{checked_add, checked_sub};
use crate::errors::Error;
use crate::storage::*;
use crate::types::{DataKey, Property, BASIS_POINTS_SCALE, STORAGE_GAP_SLOTS, STORAGE_SCHEMA_VERSION};

use soroban_sdk::{contract, contractimpl, contractmeta, Address, Env, String};

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

    /// Set the registrar allowed to create and revalue properties.
    pub fn set_registrar(env: Env, registrar: Address) -> Result<(), Error> {
        let admin = admin_of(&env)?;
        admin.require_auth();

        set_registrar(&env, &registrar);
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
        require_registrar(&env)?;
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
        require_registrar(&env)?;
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

        let previous = property.owner.clone();
        if previous == new_owner {
            return Err(Error::InvalidArgument);
        }

        let moved = get_ownership(&env, &property_id, &previous);
        let new_owner_share = get_ownership(&env, &property_id, &new_owner);

        // Credit before debiting, so an over-commitment is rejected while the
        // incumbent's position is still intact.
        let credited = checked_add(new_owner_share, moved)?;
        if credited > BASIS_POINTS_SCALE {
            return Err(Error::ShareTotalExceeded);
        }

        if moved > 0 {
            property.holder_count = property.holder_count.saturating_sub(1);
        }
        if new_owner_share == 0 {
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
    /// The two legs always move together, so the total across all holders stays
    /// at 100% and a rejected call leaves the split untouched.
    pub fn record_ownership(
        env: Env,
        property_id: String,
        holder: Address,
        amount: u128,
    ) -> Result<(), Error> {
        require_registrar(&env)?;
        ensure_running(&env)?;

        let mut property = property_of(&env, &property_id)?;
        property.owner.require_auth();

        if holder == property.owner {
            return Err(Error::InvalidArgument);
        }
        if amount > BASIS_POINTS_SCALE {
            return Err(Error::ShareTotalExceeded);
        }

        let owner_share = get_ownership(&env, &property_id, &property.owner);
        if amount > owner_share {
            return Err(Error::OwnershipExceedsShare);
        }

        let holder_share = get_ownership(&env, &property_id, &holder);
        if holder_share == 0 && amount > 0 {
            property.holder_count = property
                .holder_count
                .checked_add(1)
                .ok_or(Error::ArithmeticOverflow)?;
        }

        let remaining = checked_sub(owner_share, amount)?;
        let credited = checked_add(holder_share, amount)?;

        set_ownership(&env, &property_id, &property.owner, remaining);
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

    /// Return the current registrar.
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

fn require_registrar(env: &Env) -> Result<(), Error> {
    let registrar = get_registrar(env);
    registrar.require_auth();
    Ok(())
}

fn ensure_running(env: &Env) -> Result<(), Error> {
    if is_paused(env) {
        return Err(Error::Unauthorized);
    }
    Ok(())
}

fn property_of(env: &Env, property_id: &String) -> Result<Property, Error> {
    if !is_initialized(env) {
        return Err(Error::NotInitialized);
    }
    get_property(env, property_id).ok_or(Error::PropertyNotFound)
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