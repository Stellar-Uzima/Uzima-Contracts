#![allow(dead_code)] // Suppress unused function warnings

use soroban_sdk::{contracttype, Address, Env, Symbol};

// ============================================================================
// CONSTANTS - Rate Limit Configuration
// ============================================================================

const WINDOW_SIZE: u32 = 100;
const LIMIT_REGULAR_USER: u32 = 5;
const LIMIT_DOCTOR: u32 = 20;
const LIMIT_ADMIN: u32 = u32::MAX;

// ============================================================================
// DATA STRUCTURES
// ============================================================================

#[derive(Clone)]
#[contracttype]
pub struct RateLimitData {
    pub window_start: u32,
    pub attempt_count: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum UserRole {
    RegularUser = 0,
    Doctor = 1,
    Admin = 2,
}

// ============================================================================
// STORAGE KEY HELPERS
// ============================================================================

fn rate_limit_key(env: &Env, _user: &Address) -> Symbol {
    Symbol::new(env, "RateLimit")
}

// ============================================================================
// CORE RATE LIMITING FUNCTIONS
// ============================================================================

pub fn check_rate_limit(env: &Env, user: &Address, role: UserRole) -> bool {
    if role == UserRole::Admin {
        return true;
    }

    let current_ledger = env.ledger().sequence();
    let limit = get_limit_for_role(role);

    let key = rate_limit_key(env, user);
    let data: Option<RateLimitData> = env.storage().persistent().get(&key);

    match data {
        None => true,
        Some(rate_data) => {
            let window_expired = current_ledger >= rate_data.window_start + WINDOW_SIZE;
            if window_expired {
                true
            } else {
                rate_data.attempt_count < limit
            }
        }
    }
}

pub fn update_rate_limit(env: &Env, user: &Address, role: UserRole) {
    if role == UserRole::Admin {
        return;
    }

    let current_ledger = env.ledger().sequence();
    let key = rate_limit_key(env, user);
    let data: Option<RateLimitData> = env.storage().persistent().get(&key);

    let new_data = match data {
        None => RateLimitData {
            window_start: current_ledger,
            attempt_count: 1,
        },
        Some(mut rate_data) => {
            if current_ledger >= rate_data.window_start + WINDOW_SIZE {
                RateLimitData {
                    window_start: current_ledger,
                    attempt_count: 1,
                }
            } else {
                rate_data.attempt_count += 1;
                rate_data
            }
        }
    };

    env.storage().persistent().set(&key, &new_data);
    env.storage()
        .persistent()
        .extend_ttl(&key, WINDOW_SIZE * 2, WINDOW_SIZE * 2);
}

pub fn enforce_rate_limit(env: &Env, user: &Address, role: UserRole) -> Result<(), RateLimitError> {
    if check_rate_limit(env, user, role) {
        update_rate_limit(env, user, role);
        Ok(())
    } else {
        Err(RateLimitError::LimitExceeded)
    }
}

pub fn is_admin_bypass(role: UserRole) -> bool {
    role == UserRole::Admin
}

fn get_limit_for_role(role: UserRole) -> u32 {
    match role {
        UserRole::RegularUser => LIMIT_REGULAR_USER,
        UserRole::Doctor => LIMIT_DOCTOR,
        UserRole::Admin => LIMIT_ADMIN,
    }
}

pub fn reset_rate_limit(env: &Env, user: &Address) {
    let key = rate_limit_key(env, user);
    env.storage().persistent().remove(&key);
}

pub fn get_user_attempt_count(env: &Env, user: &Address) -> Option<u32> {
    let key = rate_limit_key(env, user);
    let data: Option<RateLimitData> = env.storage().persistent().get(&key);
    data.map(|d| d.attempt_count)
}

pub fn get_user_window_start(env: &Env, user: &Address) -> Option<u32> {
    let key = rate_limit_key(env, user);
    let data: Option<RateLimitData> = env.storage().persistent().get(&key);
    data.map(|d| d.window_start)
}

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum RateLimitError {
    LimitExceeded = 1,
}

#[cfg(test)]
pub mod test_helpers {
    use super::*;
    use soroban_sdk::testutils::Ledger;

    pub fn advance_ledger(env: &Env, count: u32) {
        let current = env.ledger().sequence();
        env.ledger().set_sequence_number(current + count);
    }
}
