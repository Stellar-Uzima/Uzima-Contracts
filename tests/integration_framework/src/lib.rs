#![no_std]

extern crate alloc;
use alloc::{string::String, vec::Vec};

use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, Env,
};

// ─── ContractEntry ──────────────────────────────────────────────────────────

struct ContractEntry {
    name:    String,
    address: Address,
}

// ─── TestWorld ───────────────────────────────────────────────────────────────

pub struct TestWorld {
    env:       Env,
    contracts: Vec<ContractEntry>,
}

impl TestWorld {
    pub fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();
        Self { env, contracts: Vec::new() }
    }

    pub fn with_env(env: Env) -> Self {
        Self { env, contracts: Vec::new() }
    }

    pub fn env(&self) -> &Env {
        &self.env
    }

    // ── Registration ─────────────────────────────────────────────────────

    pub fn register_contract(&mut self, name: &str, wasm: &[u8]) -> Address {
        assert!(
            !self.contracts.iter().any(|e| e.name == name),
            "TestWorld: contract '{}' already registered",
            name
        );
        let address = self.env.register_contract_wasm(None, wasm);
        self.contracts.push(ContractEntry {
            name:    String::from(name),
            address: address.clone(),
        });
        address
    }

    pub fn register_contract_at(
        &mut self,
        name: &str,
        address: Address,
        wasm: &[u8],
    ) -> Address {
        assert!(
            !self.contracts.iter().any(|e| e.name == name),
            "TestWorld: contract '{}' already registered",
            name
        );
        self.env.register_contract_wasm(Some(&address), wasm);
        self.contracts.push(ContractEntry {
            name:    String::from(name),
            address: address.clone(),
        });
        address
    }

    pub fn address_of(&self, name: &str) -> Address {
        self.contracts
            .iter()
            .find(|e| e.name == name)
            .unwrap_or_else(|| {
                panic!(
                    "TestWorld: no contract '{}'. Registered: [{}]",
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

    // ── Accounts ──────────────────────────────────────────────────────────

    pub fn new_account(&self) -> Address {
        Address::generate(&self.env)
    }

    pub fn new_accounts(&self, n: usize) -> Vec<Address> {
        (0..n).map(|_| Address::generate(&self.env)).collect()
    }

    // ── Ledger helpers ────────────────────────────────────────────────────

    pub fn advance_time(&self, seconds: u64) {
        self.env.ledger().with_mut(|l| l.timestamp += seconds);
    }

    pub fn set_time(&self, timestamp: u64) {
        self.env.ledger().with_mut(|l| l.timestamp = timestamp);
    }

    pub fn advance_sequence(&self, n: u32) {
        self.env.ledger().with_mut(|l| l.sequence_number += n);
    }
}

impl Default for TestWorld {
    fn default() -> Self {
        Self::new()
    }
}

// ─── UnwrapTestResult ────────────────────────────────────────────────────────

pub trait UnwrapTestResult<T> {
    fn expect_ok(self, contract: &str, func: &str) -> T;
    fn expect_err(self, contract: &str, func: &str) -> soroban_sdk::Error;
}

impl<T> UnwrapTestResult<T> for Result<T, soroban_sdk::Error> {
    fn expect_ok(self, contract: &str, func: &str) -> T {
        self.unwrap_or_else(|e| {
            panic!("TestWorld: {}::{} expected Ok, got Err({:?})", contract, func, e)
        })
    }

    fn expect_err(self, contract: &str, func: &str) -> soroban_sdk::Error {
        match self {
            Err(e) => e,
            Ok(_)  => panic!("TestWorld: {}::{} expected Err but succeeded", contract, func),
        }
    }
}

// ─── Prelude ─────────────────────────────────────────────────────────────────

pub mod prelude {
    pub use super::{TestWorld, UnwrapTestResult};
    pub use soroban_sdk::{
        testutils::{Address as _, Ledger as _},
        Address, BytesN, Env, Symbol,
    };
}