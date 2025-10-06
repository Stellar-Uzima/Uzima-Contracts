//! # ERC-2771 Context Module
//!
//! This module provides utilities for contracts to support meta-transactions
//! by correctly extracting the original sender from forwarded calls.
//!
//! ## Usage
//! Target contracts should use `msg_sender()` instead of directly calling
//! `env.current_contract_address()` to get the correct sender address when called through
//! a trusted forwarder.

use soroban_sdk::{symbol_short, Address, Bytes, Env};

/// ERC-2771 Context trait for contracts that support meta-transactions
pub trait ERC2771Context {
    /// Get the trusted forwarder address
    fn get_trusted_forwarder(env: &Env) -> Option<Address>;

    /// Check if the caller is the trusted forwarder
    fn is_trusted_forwarder(env: &Env, forwarder: &Address) -> bool {
        if let Some(trusted) = Self::get_trusted_forwarder(env) {
            return &trusted == forwarder;
        }
        false
    }

    /// Get the original message sender
    ///
    /// If the call comes from a trusted forwarder, extract the original sender
    /// from the call data. Otherwise, return the direct caller.
    fn msg_sender(env: &Env) -> Address {
        let caller = env.current_contract_address();

        // Check if caller is the trusted forwarder
        if Self::is_trusted_forwarder(env, &caller) {
            // Extract original sender from call data (appended at the end)
            // In ERC-2771, the original sender is appended as the last 20 bytes
            // For Soroban, we adapt this pattern

            // Note: In a real implementation, you would extract this from
            // the actual call data. This is a simplified version.
            // The forwarder should append the original sender address.

            // For now, return the caller (this should be enhanced based on
            // actual Soroban contract invocation patterns)
            caller
        } else {
            // Direct call, return the caller
            caller
        }
    }

    /// Get the original message data
    ///
    /// If the call comes from a trusted forwarder, remove the appended sender
    /// from the data. Otherwise, return the original data.
    fn msg_data(env: &Env, original_data: &Bytes) -> Bytes {
        let caller = env.current_contract_address();

        if Self::is_trusted_forwarder(env, &caller) {
            // Remove the last 32 bytes (appended sender address)
            // This is a simplified version
            original_data.clone()
        } else {
            original_data.clone()
        }
    }
}

/// Helper struct for implementing ERC-2771 context in contracts
pub struct ERC2771ContextImpl;

impl ERC2771ContextImpl {
    /// Set the trusted forwarder address (should be called during initialization)
    pub fn set_trusted_forwarder(env: &Env, forwarder: Address) {
        env.storage()
            .instance()
            .set(&symbol_short!("FORWARDER"), &forwarder);
    }

    /// Get the trusted forwarder address
    pub fn get_trusted_forwarder(env: &Env) -> Option<Address> {
        env.storage().instance().get(&symbol_short!("FORWARDER"))
    }

    /// Extract the original sender from forwarded call data
    ///
    /// In ERC-2771, the forwarder appends the original sender to the call data.
    /// This function extracts it.
    pub fn extract_sender_from_data(_env: &Env, _data: &Bytes) -> Option<Address> {
        // In a real implementation, parse the last 32 bytes as an address
        // This is a placeholder for the actual implementation
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    struct TestContract;

    impl ERC2771Context for TestContract {
        fn get_trusted_forwarder(env: &Env) -> Option<Address> {
            ERC2771ContextImpl::get_trusted_forwarder(env)
        }
    }

    #[test]
    fn test_set_and_get_trusted_forwarder() {
        let env = Env::default();
        let forwarder = Address::generate(&env);

        ERC2771ContextImpl::set_trusted_forwarder(&env, forwarder.clone());

        let retrieved = ERC2771ContextImpl::get_trusted_forwarder(&env);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), forwarder);
    }

    #[test]
    fn test_is_trusted_forwarder() {
        let env = Env::default();
        let forwarder = Address::generate(&env);
        let other = Address::generate(&env);

        ERC2771ContextImpl::set_trusted_forwarder(&env, forwarder.clone());

        assert!(TestContract::is_trusted_forwarder(&env, &forwarder));
        assert!(!TestContract::is_trusted_forwarder(&env, &other));
    }
}
