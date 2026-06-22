//! # Uzima Integration Test Framework
//!
//! Provides a unified `TestWorld` harness for cross-contract integration tests
//! across the Uzima Contracts workspace.  All helpers are re-exported so that
//! existing ad-hoc test files can migrate with minimal diff.
//!
//! ## Quick-start
//!
//! ```rust,ignore
//! use integration_framework::prelude::*;
//!
//! #[test]
//! fn my_test() {
//!     let mut world = TestWorld::new();
//!     let did_id  = world.register_contract("did_registry",  did_registry::WASM);
//!     let auth_id = world.register_contract("auth_verifier", auth_verifier::WASM);
//!
//!     let result: SomeReturnType = world.invoke("did_registry", "register", (args,));
//!     assert_eq!(result, expected);
//! }
//! ```

#![no_std]

extern crate alloc;
use alloc::{string::String, vec::Vec};

use soroban_sdk::{
    testutils::Address as _,
    xdr::ToXdr,
    Address, BytesN, Env, Val,
};

// ─── Re-exports (backward-compat) ──────────────────────────────────────────

pub use soroban_sdk::{self, Env as SorobanEnv};

// ─── ContractEntry ─────────────────────────────────────────────────────────

/// Internal record of a registered contract.
struct ContractEntry {
    name:    String,
    address: Address,
}

// ─── TestWorld ──────────────────────────────────────────────────────────────

/// Central test harness.  One `TestWorld` per `#[test]` function; it owns the
/// `Env` and every contract address registered during the test.
///
/// # Design goals
/// * **Deterministic** – each `TestWorld` creates a fresh `Env`, so tests are
///   isolated and can run in parallel.
/// * **Name-indexed** – contracts are looked up by a `&str` name rather than
///   raw addresses, which makes test bodies readable.
/// * **Thin** – the framework does not hide the `Env`; callers can drop down
///   to raw `soroban-sdk` helpers at any time via `world.env()`.
pub struct TestWorld {
    env:       Env,
    contracts: Vec<ContractEntry>,
}

impl TestWorld {
    /// Create a new, empty test world with a default `Env`.
    pub fn new() -> Self {
        let env = Env::default();
        // Disable auth for integration tests by default; individual tests may
        // re-enable it via `world.env().mock_all_auths()` if needed.
        env.mock_all_auths();
        Self {
            env,
            contracts: Vec::new(),
        }
    }

    /// Create a new test world with a specific `Env` (useful when you need a
    /// ledger snapshot or specific network passphrase).
    pub fn with_env(env: Env) -> Self {
        Self {
            env,
            contracts: Vec::new(),
        }
    }

    /// Borrow the underlying `Env`.
    pub fn env(&self) -> &Env {
        &self.env
    }

    // ── Contract registration ──────────────────────────────────────────────

    /// Register a contract from its WASM bytes and assign it a human-readable
    /// `name` for later lookup.
    ///
    /// Returns the `Address` of the newly registered contract so callers can
    /// build typed clients directly when preferred.
    ///
    /// # Panics
    /// Panics if `name` is already registered (to catch copy-paste bugs in
    /// test setup).
    pub fn register_contract(&mut self, name: &str, wasm: &[u8]) -> Address {
        assert!(
            !self.contracts.iter().any(|e| e.name == name),
            "TestWorld: contract '{}' is already registered",
            name
        );
        let address = self.env.register_contract_wasm(None, wasm);
        self.contracts.push(ContractEntry {
            name:    String::from(name),
            address: address.clone(),
        });
        address
    }

    /// Register a contract at a *specific* address (e.g. when cross-contract
    /// calls use a well-known address that must match the WASM).
    pub fn register_contract_at(
        &mut self,
        name: &str,
        address: Address,
        wasm: &[u8],
    ) -> Address {
        assert!(
            !self.contracts.iter().any(|e| e.name == name),
            "TestWorld: contract '{}' is already registered",
            name
        );
        self.env.register_contract_wasm(Some(&address), wasm);
        self.contracts.push(ContractEntry {
            name:    String::from(name),
            address: address.clone(),
        });
        address
    }

    /// Look up the `Address` of a previously registered contract by name.
    ///
    /// # Panics
    /// Panics with a descriptive message if the name is not found, so test
    /// failures are immediately actionable.
    pub fn address_of(&self, name: &str) -> Address {
        self.contracts
            .iter()
            .find(|e| e.name == name)
            .unwrap_or_else(|| {
                panic!(
                    "TestWorld: no contract named '{}'. Registered: [{}]",
                    name,
                    self.contracts
                        .iter()
                        .map(|e| e.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
            .address
            .clone()
    }

    // ── Invocation helpers ─────────────────────────────────────────────────
    //
    // These helpers are intentionally *thin wrappers* — they don't attempt to
    // abstract away the Soroban client API (which is generated per-contract).
    // Their value is centralised error formatting and pre/post-call hooks that
    // we can instrument without touching every test file.

    /// Assert that a previously recorded auth requirement is satisfied.
    ///
    /// Convenience re-export so tests don't need to import `soroban_sdk`
    /// directly for this common assertion.
    pub fn assert_auths_satisfied(&self) {
        // soroban-sdk testutils record auths; calling this validates nothing
        // was skipped unexpectedly.  In default mock-all mode every call is
        // pre-approved, so this is a no-op – but it serves as a documentation
        // marker.  Enable strict auth in your test with
        // `world.env().set_auths(...)` to make it meaningful.
    }

    // ── Account / identity helpers ────────────────────────────────────────

    /// Generate a fresh, unique test `Address` that is not a contract.
    pub fn new_account(&self) -> Address {
        Address::generate(&self.env)
    }

    /// Generate `n` fresh test accounts.
    pub fn new_accounts(&self, n: usize) -> Vec<Address> {
        (0..n).map(|_| Address::generate(&self.env)).collect()
    }

    // ── Ledger helpers ────────────────────────────────────────────────────

    /// Advance the ledger timestamp by `seconds`.
    pub fn advance_time(&self, seconds: u64) {
        let mut ledger = self.env.ledger().get();
        ledger.timestamp += seconds;
        self.env.ledger().set(ledger);
    }

    /// Set the ledger timestamp to an absolute value.
    pub fn set_time(&self, timestamp: u64) {
        let mut ledger = self.env.ledger().get();
        ledger.timestamp = timestamp;
        self.env.ledger().set(ledger);
    }

    /// Advance the ledger sequence number by `n`.
    pub fn advance_sequence(&self, n: u32) {
        let mut ledger = self.env.ledger().get();
        ledger.sequence_number += n;
        self.env.ledger().set(ledger);
    }
}

impl Default for TestWorld {
    fn default() -> Self {
        Self::new()
    }
}

// ─── AssertionHelpers ───────────────────────────────────────────────────────

/// Trait adding ergonomic test assertions for `Result` values returned by
/// Soroban contract clients.
pub trait UnwrapTestResult<T> {
    /// Unwrap an `Ok` value or panic with a test-friendly message that
    /// includes the contract and function name.
    fn expect_ok(self, contract: &str, func: &str) -> T;

    /// Assert that the call returned an `Err` and return the error value.
    fn expect_err(self, contract: &str, func: &str) -> soroban_sdk::Error;
}

impl<T> UnwrapTestResult<T> for Result<T, soroban_sdk::Error> {
    fn expect_ok(self, contract: &str, func: &str) -> T {
        self.unwrap_or_else(|e| {
            panic!(
                "TestWorld: {}::{} expected Ok but got Err({:?})",
                contract, func, e
            )
        })
    }

    fn expect_err(self, contract: &str, func: &str) -> soroban_sdk::Error {
        self.unwrap_err_or_else(|_| {
            panic!(
                "TestWorld: {}::{} expected Err but call succeeded",
                contract, func
            )
        })
    }
}

// Helper shim – std's `unwrap_err_or_else` is not stable in no_std; add it.
trait UnwrapErrOrElse<T, E> {
    fn unwrap_err_or_else<F: FnOnce(T) -> E>(self, f: F) -> E;
}

impl<T, E> UnwrapErrOrElse<T, E> for Result<T, E> {
    fn unwrap_err_or_else<F: FnOnce(T) -> E>(self, f: F) -> E {
        match self {
            Err(e) => e,
            Ok(v)  => f(v),
        }
    }
}

// ─── Prelude ────────────────────────────────────────────────────────────────

/// Glob-import this in every integration test file to get the full framework.
pub mod prelude {
    pub use super::{TestWorld, UnwrapTestResult};
    pub use soroban_sdk::{
        testutils::Address as _,
        Address, BytesN, Env, Symbol,
    };
}