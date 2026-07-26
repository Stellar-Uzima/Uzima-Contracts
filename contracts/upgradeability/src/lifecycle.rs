#![no_std]
//! lifecycle - Shared contract lifecycle state machine for upgradeable Soroban contracts.
//!
//! This module provides a formal state machine that all upgradeable contracts
//! in the Uzima workspace can adopt to ensure consistent lifecycle management.
//!
//! ## State Transitions
//!
//! ```text
//!  ┌─────────────────────────────────────────────────────────────┐
//!  │                    UNINITIALIZED                           │
//!  │   Contract deployed but initialize() not yet called        │
//!  └──────────────────────────┬──────────────────────────────────┘
//!                             │ initialize()
//!                             ▼
//!  ┌─────────────────────────────────────────────────────────────┐
//!  │                      ACTIVE                                │
//!  │   Normal operation — all entrypoints available             │
//!  └──────┬───────────────────┬──────────────────────────────────┘
//!         │ pause()           │ begin_upgrade()
//!         ▼                   ▼
//!  ┌────────────────┐  ┌─────────────────────────────────────────┐
//!  │    PAUSED      │  │            UPGRADING                   │
//!  │ Emergency stop │  │ Upgrade in progress – reads allowed,   │
//!  └──────┬─────────┘  │ writes blocked except migration        │
//!         │ resume()   └──────────────────┬──────────────────────┘
//!         │                               │ complete_upgrade() / abort_upgrade()
//!         ▼                               ▼
//!  ┌─────────────────────────────────────────────────────────────┐
//!  │                      ACTIVE                                │
//!  └─────────────────────────────────────────────────────────────┘
//!
//!  Any state → DEPRECATED  (admin only, one-way)
//! ```

use soroban_sdk::{contracttype, symbol_short, Env, Symbol};

// ──────────────────────────────────────────────────────────────────────────────
// State enum
// ──────────────────────────────────────────────────────────────────────────────

/// Lifecycle states for an upgradeable contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[contracttype]
pub enum ContractLifecycleState {
    /// Contract deployed but not yet initialised.
    Uninitialized = 0,
    /// Normal operation — all public entrypoints available.
    Active = 1,
    /// Emergency pause — read-only entrypoints still permitted.
    Paused = 2,
    /// Upgrade in progress — writes blocked except migration path.
    Upgrading = 3,
    /// Contract has been permanently retired; no entrypoints should be called.
    Deprecated = 4,
}

impl ContractLifecycleState {
    /// Returns `true` if the state allows normal write operations.
    #[inline]
    pub fn allows_writes(&self) -> bool {
        matches!(self, ContractLifecycleState::Active)
    }

    /// Returns `true` if the state allows read operations.
    #[inline]
    pub fn allows_reads(&self) -> bool {
        !matches!(self, ContractLifecycleState::Deprecated)
    }

    /// Returns `true` if an upgrade may be initiated from this state.
    #[inline]
    pub fn can_begin_upgrade(&self) -> bool {
        matches!(self, ContractLifecycleState::Active)
    }

    /// Human-readable name for event emission.
    pub fn as_symbol(&self) -> &'static str {
        match self {
            ContractLifecycleState::Uninitialized => "uninit",
            ContractLifecycleState::Active => "active",
            ContractLifecycleState::Paused => "paused",
            ContractLifecycleState::Upgrading => "upgrading",
            ContractLifecycleState::Deprecated => "deprecated",
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Storage key
// ──────────────────────────────────────────────────────────────────────────────

const LIFECYCLE_KEY: Symbol = symbol_short!("LIFECYCLE");

// ──────────────────────────────────────────────────────────────────────────────
// ContractLifecycle — stateful helper
// ──────────────────────────────────────────────────────────────────────────────

/// Provides lifecycle state management backed by contract storage.
///
/// # Usage
///
/// ```rust,ignore
/// // In your contract's initialize():
/// ContractLifecycle::transition(&env, ContractLifecycleState::Active);
///
/// // In a write entrypoint:
/// ContractLifecycle::require_active(&env)?;
///
/// // In an admin pause():
/// ContractLifecycle::transition(&env, ContractLifecycleState::Paused);
/// ```
pub struct ContractLifecycle;

impl ContractLifecycle {
    /// Returns the current lifecycle state, defaulting to `Uninitialized`.
    pub fn current(env: &Env) -> ContractLifecycleState {
        env.storage()
            .instance()
            .get(&LIFECYCLE_KEY)
            .unwrap_or(ContractLifecycleState::Uninitialized)
    }

    /// Persist a new lifecycle state and emit a diagnostic event.
    ///
    /// This is the **only** function that should mutate lifecycle state.
    /// Callers are responsible for authorisation checks before calling this.
    pub fn transition(env: &Env, new_state: ContractLifecycleState) {
        let old_state = Self::current(env);
        env.storage().instance().set(&LIFECYCLE_KEY, &new_state);
        Self::emit_transition(env, old_state, new_state);
    }

    /// Asserts the contract is in `Active` state.
    ///
    /// Returns `LifecycleError::NotActive` otherwise.
    pub fn require_active(env: &Env) -> Result<(), LifecycleError> {
        match Self::current(env) {
            ContractLifecycleState::Active => Ok(()),
            ContractLifecycleState::Paused => Err(LifecycleError::ContractPaused),
            ContractLifecycleState::Upgrading => Err(LifecycleError::UpgradeInProgress),
            ContractLifecycleState::Deprecated => Err(LifecycleError::ContractDeprecated),
            ContractLifecycleState::Uninitialized => Err(LifecycleError::NotInitialized),
        }
    }

    /// Asserts the contract has been initialised (state != Uninitialized).
    pub fn require_initialized(env: &Env) -> Result<(), LifecycleError> {
        if Self::current(env) == ContractLifecycleState::Uninitialized {
            return Err(LifecycleError::NotInitialized);
        }
        Ok(())
    }

    /// Asserts the contract is NOT deprecated.
    pub fn require_not_deprecated(env: &Env) -> Result<(), LifecycleError> {
        if Self::current(env) == ContractLifecycleState::Deprecated {
            return Err(LifecycleError::ContractDeprecated);
        }
        Ok(())
    }

    /// Validate a state transition against the allowed FSM edges.
    pub fn validate_transition(
        from: ContractLifecycleState,
        to: ContractLifecycleState,
    ) -> Result<(), LifecycleError> {
        use ContractLifecycleState::*;
        let allowed = matches!(
            (from, to),
            (Uninitialized, Active)
            | (Active, Paused)
            | (Active, Upgrading)
            | (Active, Deprecated)
            | (Paused, Active)
            | (Paused, Deprecated)
            | (Upgrading, Active)
            | (Upgrading, Deprecated)
        );
        if allowed {
            Ok(())
        } else {
            Err(LifecycleError::InvalidTransition)
        }
    }

    // ── private ───────────────────────────────────────────────────────────────

    fn emit_transition(
        env: &Env,
        old: ContractLifecycleState,
        new: ContractLifecycleState,
    ) {
        // Emit a Soroban diagnostic event so indexers can track state changes.
        let _ = old; // suppress unused warning — included in event topics
        let _ = new;
        env.events().publish(
            (symbol_short!("lifecycle"), symbol_short!("transition")),
            (old as u32, new as u32),
        );
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Errors
// ──────────────────────────────────────────────────────────────────────────────

/// Errors returned by lifecycle guard functions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[soroban_sdk::contracterror]
#[repr(u32)]
pub enum LifecycleError {
    /// `initialize()` has not been called yet.
    NotInitialized = 200,
    /// The contract is paused by an emergency stop.
    ContractPaused = 201,
    /// An upgrade is currently in progress.
    UpgradeInProgress = 202,
    /// The contract has been permanently deprecated.
    ContractDeprecated = 203,
    /// The requested state transition is not allowed by the FSM.
    InvalidTransition = 204,
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_allows_writes() {
        assert!(ContractLifecycleState::Active.allows_writes());
        assert!(!ContractLifecycleState::Paused.allows_writes());
        assert!(!ContractLifecycleState::Upgrading.allows_writes());
        assert!(!ContractLifecycleState::Deprecated.allows_writes());
        assert!(!ContractLifecycleState::Uninitialized.allows_writes());
    }

    #[test]
    fn test_state_allows_reads() {
        assert!(ContractLifecycleState::Active.allows_reads());
        assert!(ContractLifecycleState::Paused.allows_reads());
        assert!(ContractLifecycleState::Upgrading.allows_reads());
        assert!(!ContractLifecycleState::Deprecated.allows_reads());
    }

    #[test]
    fn test_valid_transitions() {
        use ContractLifecycleState::*;
        assert!(ContractLifecycle::validate_transition(Uninitialized, Active).is_ok());
        assert!(ContractLifecycle::validate_transition(Active, Paused).is_ok());
        assert!(ContractLifecycle::validate_transition(Active, Upgrading).is_ok());
        assert!(ContractLifecycle::validate_transition(Active, Deprecated).is_ok());
        assert!(ContractLifecycle::validate_transition(Paused, Active).is_ok());
        assert!(ContractLifecycle::validate_transition(Upgrading, Active).is_ok());
    }

    #[test]
    fn test_invalid_transitions() {
        use ContractLifecycleState::*;
        // Cannot go backwards from Deprecated
        assert!(ContractLifecycle::validate_transition(Deprecated, Active).is_err());
        // Cannot skip states
        assert!(ContractLifecycle::validate_transition(Uninitialized, Paused).is_err());
        assert!(ContractLifecycle::validate_transition(Uninitialized, Upgrading).is_err());
        // Cannot go from Upgrading to Paused directly
        assert!(ContractLifecycle::validate_transition(Upgrading, Paused).is_err());
    }
}
