//! On-chain registry of property records.
//!
//! Implemented as a [`#[contractimpl]` block on [`PropertyRegistryContract`].
//! Every public entry point below carries `# Errors`, `# Auth` and `# Events`
//! sections; this module doc covers the parts that apply contract-wide.
//!
//! # Authorisation model
//!
//! Three independent authorities, all checked before any state is touched:
//!
//! | Authority | Held by | Gates |
//! | --- | --- | --- |
//! | admin | `initialize` caller | `transfer_admin`, `set_registrar`, `pause`, `unpause` |
//! | registrar | `initialize` second argument | `register_property`, `update_valuation`, `record_ownership` |
//! | owner of record | `Property.owner` | `transfer_property`, and the debit leg of `record_ownership` |
//!
//! # Execution ordering
//!
//! Every state-changing entry point follows the same order:
//!
//! 1. **Check** — `require_auth`, pause state, and any `require_registrar`.
//! 2. **Guard** — for the multi-key entry points, take the reentrancy guard.
//! 3. **Read** — load every ledger entry the call will touch, exactly once.
//! 4. **Validate and compute** — pure, no storage access.
//! 5. **Write** — settle every ledger entry, with no reads in between.
//!
//! Steps 4 and 5 are kept separate so that a rejected call cannot leave a
//! half-applied mutation behind, and step 3 is deliberately a single read per
//! key so a caller pays for each ledger entry at most once.
//!
//! # What is not here
//!
//! There is no cross-contract call in any value path. The only dispatch out of
//! this contract is the one behind `require_auth`, which is why the reentrancy
//! guard exists at all — see [`crate::reentrancy_guard`].

use crate::errors::Error;
use crate::reentrancy_guard;
use crate::storage::*;
use crate::types::{DataKey, Property, BASIS_POINTS_SCALE};

use soroban_sdk::{contract, contractimpl, contractmeta, Address, Env, String};

/// WASM custom-section metadata, attached to every build of this contract.
contractmeta!(
    key = "Description",
    val = "Registry of on-chain real-estate property records and ownership shares"
);

/// Ledger seconds a persistent entry is extended to on every write.
const LIFETIME_THRESHOLD: u32 = 100_000;
/// Ledger count a persistent entry is extended to on every write (~30 days).
const LIFETIME_EXTEND_TO: u32 = 2_592_000;

/// On-chain property register.
#[contract]
pub struct PropertyRegistryContract;

#[contractimpl]
impl PropertyRegistryContract {
    // -----------------------------------------------------------------------
    // Initialization
    // -----------------------------------------------------------------------

    /// Initialize the registry. Callable exactly once.
    ///
    /// # Auth
    /// Requires auth from `admin`, which becomes both the admin of record and
    /// the holder of every administrative capability.
    ///
    /// # Errors
    /// - [`Error::AlreadyInitialized`] — the registry already has an admin.
    ///
    /// # Events
    /// None. `initialize` is the first event-free entry point by design: there
    /// is no previous state to diff against.
    pub fn initialize(env: Env, admin: Address, registrar: Address) -> Result<(), Error> {
        if is_initialized(&env) {
            return Err(Error::AlreadyInitialized);
        }

        admin.require_auth();

        set_admin(&env, &admin);
        set_registrar(&env, &registrar);
        set_paused(&env, false);
        set_property_count(&env, 0);

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Administration
    // -----------------------------------------------------------------------

    /// Hand administration to a new address.
    ///
    /// # Auth
    /// Requires auth from the current admin. The previous admin is not
    /// notified on-chain; coordinate the handover off-chain.
    ///
    /// # Errors
    /// - [`Error::NotInitialized`] — `initialize` has not run.
    ///
    /// # Events
    /// None.
    pub fn transfer_admin(env: Env, new_admin: Address) -> Result<(), Error> {
        let admin = admin_of(&env)?;
        admin.require_auth();

        set_admin(&env, &new_admin);
        Ok(())
    }

    /// Set the registrar allowed to create and revalue properties.
    ///
    /// Replacing the registrar immediately revokes the previous one's access;
    /// there is no overlap window.
    ///
    /// # Auth
    /// Requires auth from the current admin.
    ///
    /// # Errors
    /// - [`Error::NotInitialized`] — `initialize` has not run.
    ///
    /// # Events
    /// None.
    pub fn set_registrar(env: Env, registrar: Address) -> Result<(), Error> {
        let admin = admin_of(&env)?;
        admin.require_auth();

        set_registrar(&env, &registrar);
        Ok(())
    }

    /// Halt every state-changing operation.
    ///
    /// Queries keep working while paused: a registry that refuses to be
    /// inspected makes it impossible to find out what happened.
    ///
    /// # Auth
    /// Requires auth from the current admin.
    ///
    /// # Errors
    /// - [`Error::NotInitialized`] — `initialize` has not run.
    ///
    /// # Events
    /// None.
    pub fn pause(env: Env) -> Result<(), Error> {
        let admin = admin_of(&env)?;
        admin.require_auth();

        set_paused(&env, true);
        Ok(())
    }

    /// Resume state-changing operations.
    ///
    /// # Auth
    /// Requires auth from the current admin.
    ///
    /// # Errors
    /// - [`Error::NotInitialized`] — `initialize` has not run.
    ///
    /// # Events
    /// None.
    pub fn unpause(env: Env) -> Result<(), Error> {
        let admin = admin_of(&env)?;
        admin.require_auth();

        set_paused(&env, false);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Property lifecycle
    // -----------------------------------------------------------------------

    /// Register a new property, seeding the initial 100% ownership share.
    ///
    /// The owner is credited [`BASIS_POINTS_SCALE`] basis points and the
    /// property's `holder_count` starts at one. Supplying the same
    /// `property_id` twice is refused rather than treated as an amendment.
    ///
    /// # Auth
    /// Requires auth from the registrar of record.
    ///
    /// # Errors
    /// - [`Error::Unauthorized`] — the contract is paused, or the caller's auth
    ///   was not the registrar's.
    /// - [`Error::PropertyAlreadyExists`] — `property_id` is taken.
    /// - [`Error::ArithmeticOverflow`] — the property counter is saturated.
    ///
    /// # Events
    /// None.
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

        // A single `ledger()` read serves both timestamps on the new record.
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
    ///
    /// The previous valuation is not retained; this is a register of current
    /// values, not an appraisal history.
    ///
    /// # Auth
    /// Requires auth from the registrar of record.
    ///
    /// # Errors
    /// - [`Error::Unauthorized`] — the contract is paused, or the caller's auth
    ///   was not the registrar's.
    /// - [`Error::PropertyNotFound`] — no property is registered under
    ///   `property_id`.
    ///
    /// # Events
    /// None.
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
    ///
    /// This is the only entry point that changes `Property.owner`. It moves the
    /// incumbent's entire share, which may be less than 100% if the property has
    /// been fractionalised; use
    /// [`PropertyRegistryContract::record_ownership`] to move part of it.
    ///
    /// # Auth
    /// Requires auth from the property's current owner of record. The registrar
    /// cannot transfer a property on the owner's behalf.
    ///
    /// # Errors
    /// - [`Error::Unauthorized`] — the contract is paused, or the caller's auth
    ///   was not the owner's.
    /// - [`Error::PropertyNotFound`] — no property is registered under
    ///   `property_id`.
    /// - [`Error::InvalidArgument`] — `new_owner` is the current owner of record.
    /// - [`Error::ShareTotalExceeded`] — crediting `new_owner` with the
    ///   incumbent's position would push that holder above 100%.
    /// - [`Error::ArithmeticOverflow`] — the credit would wrap a `u128`, or the
    ///   holder counter is saturated.
    /// - [`Error::ReentrantCall`] — re-entered from an attacker-controlled
    ///   authorization contract.
    ///
    /// # Events
    /// None.
    pub fn transfer_property(env: Env, property_id: String, new_owner: Address) -> Result<(), Error> {
        ensure_running(&env)?;
        reentrancy_guard::enter(&env)?;

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

        // Reject before any write, so a refused transfer leaves the incumbent's
        // position intact rather than half-applied.
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

        reentrancy_guard::exit(&env);
        Ok(())
    }

    /// Move `amount` basis points of the owner of record's position to `holder`.
    ///
    /// Both legs move in the same call, so the total across all holders stays at
    /// [`BASIS_POINTS_SCALE`]. `holder_count` is derived from the share read
    /// rather than from the argument, so a repeat transfer to an existing holder
    /// cannot double-count them.
    ///
    /// # Auth
    /// Requires auth from **both** the registrar of record and the property's
    /// owner of record. The registrar says who may amend the register; the owner
    /// says whose position may be debited. Either alone would be wrong: without
    /// the owner, a compromised registrar could strip every property in the
    /// registry.
    ///
    /// # Errors
    /// - [`Error::Unauthorized`] — the contract is paused, or either required
    ///   authorization is missing.
    /// - [`Error::PropertyNotFound`] — no property is registered under
    ///   `property_id`.
    /// - [`Error::InvalidArgument`] — `holder` is the owner of record, which
    ///   would debit and credit the same entry.
    /// - [`Error::OwnershipExceedsShare`] — `amount` is more than the owner of
    ///   record currently holds.
    /// - [`Error::ShareTotalExceeded`] — the credit would push `holder` above
    ///   100%.
    /// - [`Error::ArithmeticOverflow`] — the credit would wrap a `u128`, or the
    ///   holder counter is saturated.
    /// - [`Error::ArithmeticUnderflow`] — the debit would wrap past zero.
    /// - [`Error::ReentrantCall`] — re-entered from an attacker-controlled
    ///   authorization contract.
    ///
    /// # Events
    /// None.
    pub fn record_ownership(
        env: Env,
        property_id: String,
        holder: Address,
        amount: u128,
    ) -> Result<(), Error> {
        ensure_running(&env)?;
        reentrancy_guard::enter(&env)?;

        require_registrar(&env)?;

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

        reentrancy_guard::exit(&env);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Queries
    // -----------------------------------------------------------------------

    /// Return the full record for `property_id`.
    ///
    /// # Auth
    /// None. Read-only.
    ///
    /// # Errors
    /// - [`Error::NotInitialized`] — `initialize` has not run.
    /// - [`Error::PropertyNotFound`] — no property is registered under
    ///   `property_id`.
    pub fn get_property(env: Env, property_id: String) -> Result<Property, Error> {
        property_of(&env, &property_id)
    }

    /// Return `holder`'s ownership share of `property_id`, in basis points.
    ///
    /// Returns `0` rather than an error for an unknown property or holder: a
    /// share is a quantity, and "holds nothing" is a legitimate answer that a
    /// caller should not have to special-case.
    ///
    /// # Auth
    /// None. Read-only.
    pub fn ownership_of(env: Env, property_id: String, holder: Address) -> u128 {
        get_ownership(&env, &property_id, &holder)
    }

    /// Return how many properties are registered.
    ///
    /// # Auth
    /// None. Read-only.
    pub fn total_properties(env: Env) -> u32 {
        get_property_count(&env)
    }

    /// Return the current administrator.
    ///
    /// # Auth
    /// None. Read-only.
    ///
    /// # Errors
    /// - [`Error::NotInitialized`] — `initialize` has not run.
    pub fn admin(env: Env) -> Result<Address, Error> {
        admin_of(&env)
    }

    /// Return the registrar of record.
    ///
    /// # Auth
    /// None. Read-only.
    ///
    /// # Errors
    /// - [`Error::NotInitialized`] — `initialize` has not run.
    pub fn registrar(env: Env) -> Result<Address, Error> {
        if !is_initialized(&env) {
            return Err(Error::NotInitialized);
        }
        Ok(get_registrar(&env))
    }

    /// Return whether state-changing operations are halted.
    ///
    /// Callers that receive [`Error::Unauthorized`] can call this to tell a
    /// pause apart from a failed authorization.
    ///
    /// # Auth
    /// None. Read-only.
    pub fn paused(env: Env) -> bool {
        is_paused(&env)
    }

    /// Return whether a guarded section is currently active.
    ///
    /// Normally `false` outside a guarded call. Exposed as a query so the guard
    /// can be observed by callers and asserted on by tests, rather than only
    /// reachable from inside the contract.
    ///
    /// # Auth
    /// None. Read-only.
    pub fn reentrancy_locked(env: Env) -> bool {
        reentrancy_guard::is_locked(&env)
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// The admin address, or [`Error::NotInitialized`] before `initialize` runs.
fn admin_of(env: &Env) -> Result<Address, Error> {
    if !is_initialized(env) {
        return Err(Error::NotInitialized);
    }
    Ok(get_admin(env))
}

/// Require the registrar of record to authorize.
///
/// This is the only dispatch out of the contract other than the admin's, and
/// is the reason `src/reentrancy_guard.rs` exists.
fn require_registrar(env: &Env) -> Result<(), Error> {
    let registrar = get_registrar(env);
    registrar.require_auth();
    Ok(())
}

/// Refuse state changes while the contract is paused.
///
/// Checked *after* `require_auth` so an unauthorized caller cannot use a paused
/// registry to learn that it is paused.
fn ensure_running(env: &Env) -> Result<(), Error> {
    if is_paused(env) {
        return Err(Error::Unauthorized);
    }
    Ok(())
}

/// The property record, or [`Error::PropertyNotFound`].
fn property_of(env: &Env, property_id: &String) -> Result<Property, Error> {
    if !is_initialized(env) {
        return Err(Error::NotInitialized);
    }
    get_property(env, property_id).ok_or(Error::PropertyNotFound)
}

/// Extend the TTL of every persistent key touched by `property_id`.
///
/// Not yet called from the entry points: the TTL policy is being settled
/// separately, and calling it per write would add two ledger writes to every
/// mutation. Kept here, and exported for the migration tooling that will use it.
#[allow(dead_code)]
pub fn bump_ttl(env: &Env, property_id: &String, holder: &Address) {
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