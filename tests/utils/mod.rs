pub mod assertions;
/// Testing utilities module for contract testing
/// This module provides helper functions and utilities for testing Soroban contracts
pub mod contract_utils;
pub mod performance;
pub mod test_data;
pub mod test_fixtures;

#[cfg(test)]
pub mod contract_fixtures;
#[cfg(test)]
pub mod integration_framework;

pub use assertions::*;
pub use contract_utils::*;
pub use performance::*;
pub use test_data::*;
pub use test_fixtures::*;

#[cfg(test)]
pub use contract_fixtures::*;
#[cfg(test)]
pub use integration_framework::*;

/// Common test constants
pub mod constants {
    pub const INITIAL_BALANCE: u128 = 1_000_000 * 10_u128.pow(7); // 10M tokens
    pub const MIN_BALANCE: u128 = 1_000 * 10_u128.pow(7); // 1k tokens
    pub const MAX_PAGES: u32 = 1000;
    pub const DEFAULT_TIMEOUT_MS: u64 = 5000;
}
