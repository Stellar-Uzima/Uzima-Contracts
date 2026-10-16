//! Reentrancy guard for state-changing entry points.
//!
//! # Why a guard is needed at all
//!
//! `PropertyRegistry` makes no explicit cross-contract calls in its value paths:
//! it reads and writes its own ledger entries and nothing else. That rules out
//! the textbook read-write-read pattern, because there is no external call to
//! re-enter *through*.
//!
//! There is, however, one genuine cross-contract invocation left:
//! `Address::require_auth()`. When an address is backed by a custom
//! `SorobanAuthorization` contract — which is the normal way to express
//! multi-sig, spending policies or session keys on Soroban — `require_auth`
//! dispatches to that contract. That dispatch is a real contract invocation
//! controlled by whoever chose the authenticating address, so an address backed
//! by a hostile authorization contract can re-enter the registry in the middle
//! of a `transfer_property` or `record_ownership`.
//!
//! # What the guard protects
//!
//! The entry points that touch more than one ledger key take the guard:
//! `transfer_property` (two ownership entries plus the record) and
//! `record_ownership` (the same). Single-key entry points are left unguarded
//! deliberately — a guard costs a ledger read and a write on every call, and on a
//! single-key path there is no interleaving to protect against.
//!
//! # Why entering first is correct here
//!
//! The guard is taken *before* any authorization dispatch, not after. That is
//! what closes the window: `require_auth()` is itself the dispatch an attacker
//! controls, so a guard taken after it would be taken after the attacker's code
//! has already run.
//!
//! Leaving the guard is done by [`exit`], which is called on every success path
//! only. That is sufficient rather than sloppy, because Soroban execution is
//! transactional: an entry point that returns `Err` rolls back every storage
//! write made in that frame, including the guard flag itself. So an early
//! `return Err(..)` after `enter` cannot leave the contract permanently locked —
//! the lock is released by the rollback that released everything else. The
//! `reentrancy_tests.rs` case `guard_is_released_when_a_guarded_call_fails` pins
//! that behaviour, because it is the assumption the whole design rests on.
//!
//! The reentrancy regression tests in `reentrancy_tests.rs` assert exactly this:
//! that the outer call succeeds, the inner one is refused with
//! [`Error::ReentrantCall`], and the lock is clear again afterwards.

use crate::errors::Error;
use crate::types::DataKey;
use soroban_sdk::Env;

/// Guard state meaning "not currently inside a guarded section".
pub const NOT_ENTERED: u32 = 1;
/// Guard state meaning "a guarded section is already active in this frame chain".
pub const ENTERED: u32 = 2;

/// Current guard state. Defaults to [`NOT_ENTERED`] for a deployment that has
/// never taken the guard.
pub fn status(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::ReentrancyStatus)
        .unwrap_or(NOT_ENTERED)
}

/// Take the guard, or refuse if it is already held.
pub fn enter(env: &Env) -> Result<(), Error> {
    if status(env) == ENTERED {
        return Err(Error::ReentrantCall);
    }
    env.storage()
        .instance()
        .set(&DataKey::ReentrancyStatus, &ENTERED);
    Ok(())
}

/// Release the guard. Called on every success path.
pub fn exit(env: &Env) {
    env.storage()
        .instance()
        .set(&DataKey::ReentrancyStatus, &NOT_ENTERED);
}

/// Whether the guard is currently held. Exposed for tests and for monitoring.
pub fn is_locked(env: &Env) -> bool {
    status(env) == ENTERED
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env() -> Env {
        Env::default()
    }

    #[test]
    fn guard_starts_clear() {
        let e = env();
        assert_eq!(status(&e), NOT_ENTERED);
        assert!(!is_locked(&e));
    }

    #[test]
    fn enter_then_exit_returns_to_clear() {
        let e = env();

        enter(&e).unwrap();
        assert!(is_locked(&e));

        exit(&e);
        assert!(!is_locked(&e));
    }

    #[test]
    fn nested_enter_is_refused() {
        let e = env();

        enter(&e).unwrap();
        assert_eq!(enter(&e), Err(Error::ReentrantCall));
    }

    #[test]
    fn guard_can_be_taken_again_after_exit() {
        let e = env();

        enter(&e).unwrap();
        exit(&e);
        enter(&e).unwrap();
        assert!(is_locked(&e));
    }
}