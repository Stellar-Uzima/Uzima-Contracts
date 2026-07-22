//! Re-initialization guard for cloneable / upgradeable contracts.
//!
//! # Why this exists
//!
//! A contract whose `initialize` (or `init`) entry point can be called more
//! than once can have its ownership stolen: an attacker simply calls
//! `initialize` again and installs themselves as admin. This is a well-known
//! attack class for cloneable contracts and has been exploited repeatedly in
//! the wild.
//!
//! `SECURITY_CHECKLIST.md` (Item 4 — State Management) requires every contract
//! to be idempotent or guarded against re-initialization. This module is the
//! **single, standardized** way to satisfy that requirement so the semantics
//! are identical across all 100+ contracts in the workspace.
//!
//! # Semantics (the contract this module guarantees)
//!
//! * **One-shot initialization.** The first call to [`init_guard`] /
//!   [`try_init_guard`] marks the contract initialized and succeeds. Every
//!   subsequent call **fails**: [`init_guard`] panics, [`try_init_guard`]
//!   returns [`GovernanceError::AlreadyInitialized`]. The guard is flipped
//!   *before* the contract writes its admin/config, so a re-init can never
//!   reach the ownership-mutating code.
//!
//! * **Admin transfer is a separate, independent operation.** This guard does
//!   **not** lock ownership forever. Rotating the admin must be done through a
//!   dedicated `transfer_admin`-style function that authorizes the *current*
//!   admin (`require_auth`) and overwrites the admin key. That path never
//!   touches the init flag, so transferring the admin does not "re-open"
//!   initialization, and re-initialization can never be used as a backdoor
//!   admin transfer. The two concerns are deliberately decoupled.
//!
//! * **Dedicated, namespaced flag.** The guard owns its own instance-storage
//!   key ([`INIT_GUARD_KEY`]) rather than inferring initialization from the
//!   presence of an admin/config entry. This makes the semantics uniform
//!   regardless of how a given contract lays out its own state, and avoids the
//!   inconsistent "is the admin set?" heuristics that previously varied
//!   contract to contract.
//!
//! # Usage
//!
//! For contracts whose `initialize` returns `Result<_, E>` (the common case):
//!
//! ```ignore
//! pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
//!     governance_commons::try_init_guard(&env).map_err(|_| Error::AlreadyInitialized)?;
//!     admin.require_auth();
//!     env.storage().instance().set(&DataKey::Admin, &admin);
//!     Ok(())
//! }
//! ```
//!
//! For contracts whose `initialize` returns `()` and signals errors by
//! panicking:
//!
//! ```ignore
//! pub fn initialize(env: Env, admin: Address) {
//!     governance_commons::init_guard(&env);
//!     admin.require_auth();
//!     env.storage().instance().set(&DataKey::Admin, &admin);
//! }
//! ```

use crate::errors::GovernanceError;
use soroban_sdk::{symbol_short, Env, Symbol};

/// Instance-storage key under which the one-shot initialization flag is kept.
///
/// This key is namespaced to the guard and must not be reused by contract
/// state. Storing it in *instance* storage ties its lifetime to the contract
/// instance (the same place admin/config live), so it is bundled into the same
/// archival/TTL unit and cannot be independently expired to bypass the guard.
pub const INIT_GUARD_KEY: Symbol = symbol_short!("INIT_GD");

/// Returns `true` if the contract has already been initialized.
///
/// Reads the dedicated guard flag from instance storage. Useful for read-only
/// status checks; the mutating guard lives in [`try_init_guard`] / [`init_guard`].
pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&INIT_GUARD_KEY)
}

/// One-shot initialization guard (non-panicking).
///
/// On the **first** call this marks the contract initialized and returns
/// `Ok(())`. On every **subsequent** call it returns
/// [`GovernanceError::AlreadyInitialized`] and leaves storage unchanged.
///
/// Call this as the very first statement of `initialize`, before any
/// ownership/admin writes, so a re-initialization attempt is rejected before it
/// can mutate state. See the [module docs](self) for the full semantics,
/// including why admin transfer is handled separately.
pub fn try_init_guard(env: &Env) -> Result<(), GovernanceError> {
    if is_initialized(env) {
        return Err(GovernanceError::AlreadyInitialized);
    }
    env.storage().instance().set(&INIT_GUARD_KEY, &true);
    Ok(())
}

/// One-shot initialization guard (panicking).
///
/// Equivalent to [`try_init_guard`] but **panics** if the contract has already
/// been initialized, per the documented "panic if init called twice"
/// semantics. Prefer this for contracts whose `initialize` returns `()`; use
/// [`try_init_guard`] when you want to fold the failure into your own `Result`
/// error type.
///
/// # Panics
///
/// Panics with [`GovernanceError::AlreadyInitialized`] if called more than once.
pub fn init_guard(env: &Env) {
    assert!(
        try_init_guard(env).is_ok(),
        "governance_commons::init_guard: already initialized",
    );
}

/// Asserts that the contract **has** been initialized, returning
/// [`GovernanceError::NotInitialized`] otherwise.
///
/// Useful at the top of post-init entry points that must not run on an
/// uninitialized contract.
pub fn require_initialized(env: &Env) -> Result<(), GovernanceError> {
    if is_initialized(env) {
        Ok(())
    } else {
        Err(GovernanceError::NotInitialized)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{
        contract, symbol_short, testutils::Address as _, Address, Env, Symbol,
    };

    // A minimal host contract so we have an instance-storage context to run the
    // guard inside (instance storage is only available within a contract).
    #[contract]
    struct GuardTestContract;

    const ADMIN_KEY: Symbol = symbol_short!("ADMIN");

    /// init succeeds the first time and records the initialized state.
    #[test]
    fn init_succeeds_once() {
        let env = Env::default();
        let id = env.register_contract(None, GuardTestContract);
        env.as_contract(&id, || {
            assert!(!is_initialized(&env), "fresh contract must be uninitialized");
            assert!(try_init_guard(&env).is_ok(), "first init must succeed");
            assert!(is_initialized(&env), "flag must be set after init");
            assert!(require_initialized(&env).is_ok());
        });
    }

    /// init fails (returns AlreadyInitialized) on the second call — try_ variant.
    #[test]
    fn init_fails_second_time() {
        let env = Env::default();
        let id = env.register_contract(None, GuardTestContract);
        env.as_contract(&id, || {
            assert!(try_init_guard(&env).is_ok());
            assert_eq!(
                try_init_guard(&env),
                Err(GovernanceError::AlreadyInitialized),
                "second init must be rejected",
            );
        });
    }

    /// init panics on the second call — panicking variant.
    #[test]
    #[should_panic(expected = "already initialized")]
    fn init_guard_panics_second_time() {
        let env = Env::default();
        let id = env.register_contract(None, GuardTestContract);
        env.as_contract(&id, || {
            init_guard(&env);
            init_guard(&env); // must panic
        });
    }

    /// require_initialized rejects an uninitialized contract.
    #[test]
    fn require_initialized_rejects_uninitialized() {
        let env = Env::default();
        let id = env.register_contract(None, GuardTestContract);
        env.as_contract(&id, || {
            assert_eq!(
                require_initialized(&env),
                Err(GovernanceError::NotInitialized),
            );
        });
    }

    /// Admin transfer works independently of the init guard: rotating the admin
    /// neither flips the init flag nor re-opens initialization. This models the
    /// "one-shot admin transfer allowed separately" semantics.
    #[test]
    fn admin_transfer_is_independent_of_init_guard() {
        let env = Env::default();
        let id = env.register_contract(None, GuardTestContract);
        let admin = Address::generate(&env);
        let new_admin = Address::generate(&env);

        env.as_contract(&id, || {
            // initialize once, storing an admin alongside the guard flag
            try_init_guard(&env).unwrap();
            env.storage().instance().set(&ADMIN_KEY, &admin);

            // a separate admin-transfer path overwrites the admin key only
            env.storage().instance().set(&ADMIN_KEY, &new_admin);

            // transfer changed the admin...
            let stored: Address = env.storage().instance().get(&ADMIN_KEY).unwrap();
            assert_eq!(stored, new_admin, "admin transfer must update the admin");

            // ...but did not touch the guard: still initialized, still one-shot.
            assert!(is_initialized(&env), "transfer must not clear init flag");
            assert_eq!(
                try_init_guard(&env),
                Err(GovernanceError::AlreadyInitialized),
                "re-init must remain blocked after an admin transfer",
            );
        });
    }
}
