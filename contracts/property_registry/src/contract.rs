use crate::errors::Error;
use crate::events;
use crate::storage::*;
use crate::types::{DataKey, Property};
use crate::validation::*;

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

        events::initialized(&env, &admin, &registrar);
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

    /// Set the registrar allowed to create and revalue properties.
    pub fn set_registrar(env: Env, registrar: Address) -> Result<(), Error> {
        let admin = admin_of(&env)?;
        admin.require_auth();

        set_registrar(&env, &registrar);
        events::registrar_updated(&env, &registrar);
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

    /// Resume state-changing operations.
    pub fn unpause(env: Env) -> Result<(), Error> {
        let admin = admin_of(&env)?;
        admin.require_auth();

        set_paused(&env, false);
        events::pause_state_changed(&env, false);
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
        validate_property_id(&property_id)?;
        validate_valuation(valuation)?;
        validate_currency(&valuation_currency)?;
        validate_metadata_uri(&metadata_uri)?;

        require_registrar(&env)?;
        ensure_running(&env)?;

        if has_property(&env, &property_id) {
            return Err(Error::PropertyAlreadyExists);
        }

        // One ledger timestamp read serves both timestamps on the new record.
        let now = env.ledger().timestamp();
        let property = Property {
            property_id: property_id.clone(),
            owner: owner.clone(),
            created_at: now,
            updated_at: now,
            valuation,
            valuation_currency: valuation_currency.clone(),
            metadata_uri,
            holder_count: 1,
        };
        set_property(&env, &property);

        set_property_count(
            &env,
            get_property_count(&env)
                .checked_add(1)
                .ok_or(Error::ArithmeticOverflow)?,
        );

        set_ownership(&env, &property_id, &owner, BASIS_POINTS_SCALE);

        events::property_registered(&env, &property_id, &owner, valuation, &valuation_currency);
        Ok(())
    }

    /// Overwrite the recorded valuation of an existing property.
    pub fn update_valuation(env: Env, property_id: String, valuation: u128) -> Result<(), Error> {
        validate_valuation(valuation)?;

        require_registrar(&env)?;
        ensure_running(&env)?;

        // A single read serves the existence check and the write-back.
        let mut property = property_of(&env, &property_id)?;
        let previous = property.valuation;

        property.valuation = valuation;
        property.updated_at = env.ledger().timestamp();
        set_property(&env, &property);

        events::valuation_updated(&env, &property_id, previous, valuation);
        Ok(())
    }

    /// Transfer the property of record to `new_owner`, moving the whole
    /// recorded position of the incumbent.
    pub fn transfer_property(env: Env, property_id: String, new_owner: Address) -> Result<(), Error> {
        ensure_running(&env)?;

        let mut property = property_of(&env, &property_id)?;
        property.owner.require_auth();
        validate_transfer_parties(&property.owner, &new_owner)?;

        let previous = property.owner.clone();
        let moved = get_ownership(&env, &property_id, &previous);
        let successor_share = get_ownership(&env, &property_id, &new_owner);
        let credited = successor_share
            .checked_add(moved)
            .ok_or(Error::ArithmeticOverflow)?;

        // Credit before debit: rejecting here leaves the incumbent's position
        // intact, so a refused transfer cannot half-apply.
        validate_share_total(credited)?;

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

        property.owner = new_owner.clone();
        property.updated_at = env.ledger().timestamp();
        set_property(&env, &property);

        events::property_transferred(&env, &property_id, &previous, &new_owner, moved);
        Ok(())
    }

    /// Move `amount` basis points of the owner of record's position to `holder`.
    ///
    /// The two legs always move together, so the total across all holders stays
    /// at `BASIS_POINTS_SCALE`.
    pub fn record_ownership(
        env: Env,
        property_id: String,
        holder: Address,
        amount: u128,
    ) -> Result<(), Error> {
        validate_share(amount)?;

        require_registrar(&env)?;
        ensure_running(&env)?;

        let mut property = property_of(&env, &property_id)?;
        property.owner.require_auth();
        validate_holder(&holder, &property.owner)?;

        let owner = property.owner.clone();
        let owner_share = get_ownership(&env, &property_id, &owner);
        validate_debit(owner_share, amount)?;

        let holder_share = get_ownership(&env, &property_id, &holder);
        let credited = holder_share
            .checked_add(amount)
            .ok_or(Error::ArithmeticOverflow)?;
        validate_share_total(credited)?;

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

        events::ownership_recorded(&env, &property_id, &holder, amount, remaining, credited);
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

    /// Return `holder`'s ownership of `property_id` as whole percent.
    pub fn ownership_percent(env: Env, property_id: String, holder: Address) -> u128 {
        get_ownership(&env, &property_id, &holder) / (BASIS_POINTS_SCALE / 100)
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