#![no_std]
use soroban_sdk::{contracterror, contracttype, Address, Env, Symbol};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u32)]
pub enum PauseError {
    ContractIsPaused = 100,
    ContractNotPaused = 101,
    UnauthorizedAdmin = 102,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PauseState {
    pub is_paused: bool,
    pub paused_by: Address,
    pub paused_at: u64,
}

pub struct PausableControl;

impl PausableControl {
    /// Ensures execution is allowed; panics/fails if contract is currently paused
    pub fn require_not_paused(env: &Env) -> Result<(), PauseError> {
        if Self::is_paused(env) {
            return Err(PauseError::ContractIsPaused);
        }
        Ok(())
    }

    /// Reads current contract pause status from instance storage
    pub fn is_paused(env: &Env) -> bool {
        env.storage()
            .instance()
            .get(&Symbol::new(env, "PAUSED"))
            .unwrap_or(false)
    }

    /// Sets contract paused state to true with admin authorization
    pub fn pause(env: &Env, admin: Address) -> Result<(), PauseError> {
        admin.require_auth();

        if Self::is_paused(env) {
            return Err(PauseError::ContractIsPaused);
        }

        let state = PauseState {
            is_paused: true,
            paused_by: admin,
            paused_at: env.ledger().timestamp(),
        };

        env.storage().instance().set(&Symbol::new(env, "PAUSED"), &true);
        env.storage().instance().set(&Symbol::new(env, "PAUSE_STATE"), &state);

        env.events().publish(
            (Symbol::new(env, "pausable"), Symbol::new(env, "paused")),
            state,
        );

        Ok(())
    }

    /// Restores operational state to unpaused
    pub fn unpause(env: &Env, admin: Address) -> Result<(), PauseError> {
        admin.require_auth();

        if !Self::is_paused(env) {
            return Err(PauseError::ContractNotPaused);
        }

        env.storage().instance().set(&Symbol::new(env, "PAUSED"), &false);

        env.events().publish(
            (Symbol::new(env, "pausable"), Symbol::new(env, "unpaused")),
            env.ledger().timestamp(),
        );

        Ok(())
    }
}