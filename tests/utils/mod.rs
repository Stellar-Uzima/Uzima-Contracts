/// Testing utilities module for contract testing
/// This module provides helper functions and utilities for testing Soroban contracts

pub mod contract_utils;
pub mod test_fixtures;
pub mod test_data;
pub mod assertions;
pub mod performance;

pub use contract_utils::*;
pub use test_fixtures::*;
pub use test_data::*;
pub use assertions::*;
pub use performance::*;

/// Common test constants
pub mod constants {
    use soroban_sdk::Duration;

    pub const INITIAL_BALANCE: u128 = 1_000_000 * 10_u128.pow(7); // 10M tokens
    pub const MIN_BALANCE: u128 = 1_000 * 10_u128.pow(7); // 1k tokens
    pub const MAX_PAGES: u32 = 1000;
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_millis(5000);
}
