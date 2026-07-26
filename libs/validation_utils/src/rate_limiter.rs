#![no_std]
//! rate_limiter - Rate limiting and throttling safeguards for public entrypoints.

use soroban_sdk::{contracttype, symbol_short, Address, Env};

#[derive(Clone, Copy, Debug)]
#[contracttype]
pub struct RateLimitConfig {
    pub max_calls: u32,
    pub window_ledgers: u32,
}

impl RateLimitConfig {
    pub const fn conservative() -> Self { Self { max_calls: 10, window_ledgers: 720 } }
    pub const fn standard() -> Self { Self { max_calls: 100, window_ledgers: 720 } }
    pub const fn permissive() -> Self { Self { max_calls: 1000, window_ledgers: 720 } }
}

#[derive(Clone)]
#[contracttype]
pub struct RateLimitKey {
    pub caller: Address,
    pub namespace: soroban_sdk::Symbol,
}

#[derive(Clone, Debug)]
#[contracttype]
struct RateLimitState {
    pub count: u32,
    pub window_start: u32,
}

pub struct RateLimiter;

impl RateLimiter {
    pub fn check_and_consume(
        env: &Env,
        caller: &Address,
        namespace: soroban_sdk::Symbol,
        config: &RateLimitConfig,
    ) -> Result<(), RateLimitError> {
        let key = RateLimitKey { caller: caller.clone(), namespace };
        let current = env.ledger().sequence();
        let mut state: RateLimitState = env.storage().temporary().get(&key)
            .unwrap_or(RateLimitState { count: 0, window_start: current });

        if current >= state.window_start + config.window_ledgers {
            state = RateLimitState { count: 0, window_start: current };
        }
        if state.count >= config.max_calls {
            env.events().publish(
                (symbol_short!("rl"), symbol_short!("exceeded")),
                (caller, state.count),
            );
            return Err(RateLimitError::Exceeded);
        }
        state.count += 1;
        env.storage().temporary().set(&key, &state);
        env.storage().temporary().extend_ttl(&key, 0, config.window_ledgers);
        Ok(())
    }

    pub fn remaining(env: &Env, caller: &Address, namespace: soroban_sdk::Symbol, config: &RateLimitConfig) -> u32 {
        let key = RateLimitKey { caller: caller.clone(), namespace };
        let current = env.ledger().sequence();
        let state: RateLimitState = env.storage().temporary().get(&key)
            .unwrap_or(RateLimitState { count: 0, window_start: current });
        if current >= state.window_start + config.window_ledgers {
            return config.max_calls;
        }
        config.max_calls.saturating_sub(state.count)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[soroban_sdk::contracterror]
#[repr(u32)]
pub enum RateLimitError {
    Exceeded = 6,
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::{Address as _, Ledger as _}, symbol_short, Env};

    #[test]
    fn test_within_limit_passes() {
        let env = Env::default();
        env.ledger().set_sequence_number(100);
        let caller = soroban_sdk::Address::generate(&env);
        let config = RateLimitConfig { max_calls: 3, window_ledgers: 100 };
        let ns = symbol_short!("test");
        assert!(RateLimiter::check_and_consume(&env, &caller, ns.clone(), &config).is_ok());
        assert!(RateLimiter::check_and_consume(&env, &caller, ns.clone(), &config).is_ok());
        assert!(RateLimiter::check_and_consume(&env, &caller, ns, &config).is_ok());
    }

    #[test]
    fn test_exceeding_limit_fails() {
        let env = Env::default();
        env.ledger().set_sequence_number(100);
        let caller = soroban_sdk::Address::generate(&env);
        let config = RateLimitConfig { max_calls: 2, window_ledgers: 100 };
        let ns = symbol_short!("test");
        RateLimiter::check_and_consume(&env, &caller, ns.clone(), &config).unwrap();
        RateLimiter::check_and_consume(&env, &caller, ns.clone(), &config).unwrap();
        assert_eq!(RateLimiter::check_and_consume(&env, &caller, ns, &config).unwrap_err(), RateLimitError::Exceeded);
    }

    #[test]
    fn test_window_reset_allows_new_calls() {
        let env = Env::default();
        env.ledger().set_sequence_number(100);
        let caller = soroban_sdk::Address::generate(&env);
        let config = RateLimitConfig { max_calls: 1, window_ledgers: 50 };
        let ns = symbol_short!("test");
        RateLimiter::check_and_consume(&env, &caller, ns.clone(), &config).unwrap();
        env.ledger().set_sequence_number(200);
        assert!(RateLimiter::check_and_consume(&env, &caller, ns, &config).is_ok());
    }
}
