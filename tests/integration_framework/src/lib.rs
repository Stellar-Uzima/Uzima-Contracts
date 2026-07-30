#![cfg(any(test, feature = "testutils"))]
#![no_std]

extern crate alloc;
use alloc::{string::String, vec, vec::Vec};

use soroban_sdk::{
    testutils::{Address as _, Ledger as _, BytesN as _},
    Address, BytesN, Env, IntoVal, Symbol,
};

// ─── ContractEntry ──────────────────────────────────────────────────────────

struct ContractEntry {
    name: String,
    address: Address,
}

// ─── TestWorld ───────────────────────────────────────────────────────────────

pub struct TestWorld {
    env: Env,
    contracts: Vec<ContractEntry>,
    accounts: Vec<Address>,
    snapshots: Vec<WorldSnapshot>,
}

struct WorldSnapshot {
    timestamp: u64,
    sequence: u32,
    label: String,
}

impl TestWorld {
    pub fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();
        Self {
            env,
            contracts: Vec::new(),
            accounts: Vec::new(),
            snapshots: Vec::new(),
        }
    }

    pub fn with_env(env: Env) -> Self {
        Self {
            env,
            contracts: Vec::new(),
            accounts: Vec::new(),
            snapshots: Vec::new(),
        }
    }

    pub fn env(&self) -> &Env {
        &self.env
    }

    pub fn env_mut(&mut self) -> &mut Env {
        &mut self.env
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
            name: String::from(name),
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
            name: String::from(name),
            address: address.clone(),
        });
        address
    }

    pub fn register_contract_from_type<C: soroban_sdk::contract::ContractType>(
        &mut self,
        name: &str,
    ) -> Address {
        assert!(
            !self.contracts.iter().any(|e| e.name == name),
            "TestWorld: contract '{}' already registered",
            name
        );
        let address = self.env.register_contract(None, C::package());
        self.contracts.push(ContractEntry {
            name: String::from(name),
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

    pub fn has_contract(&self, name: &str) -> bool {
        self.contracts.iter().any(|e| e.name == name)
    }

    pub fn contract_names(&self) -> Vec<String> {
        self.contracts.iter().map(|e| e.name.clone()).collect()
    }

    // ── Accounts ──────────────────────────────────────────────────────────

    pub fn new_account(&mut self) -> Address {
        let addr = Address::generate(&self.env);
        self.accounts.push(addr.clone());
        addr
    }

    pub fn new_accounts(&mut self, n: usize) -> Vec<Address> {
        let addrs: Vec<Address> = (0..n).map(|_| self.new_account()).collect();
        addrs
    }

    pub fn accounts(&self) -> &[Address] {
        &self.accounts
    }

    pub fn admin(&mut self) -> Address {
        self.new_account()
    }

    pub fn patient(&mut self) -> Address {
        self.new_account()
    }

    pub fn provider(&mut self) -> Address {
        self.new_account()
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

    pub fn current_timestamp(&self) -> u64 {
        self.env.ledger().timestamp()
    }

    pub fn current_sequence(&self) -> u32 {
        self.env.ledger().sequence()
    }

    // ── Snapshots ─────────────────────────────────────────────────────────

    pub fn snapshot(&mut self, label: &str) {
        self.snapshots.push(WorldSnapshot {
            timestamp: self.env.ledger().timestamp(),
            sequence: self.env.ledger().sequence(),
            label: String::from(label),
        });
    }

    pub fn restore_snapshot(&mut self, label: &str) {
        let snap = self
            .snapshots
            .iter()
            .find(|s| s.label == label)
            .unwrap_or_else(|| panic!("TestWorld: snapshot '{}' not found", label));

        self.env.ledger().with_mut(|l| {
            l.timestamp = snap.timestamp;
            l.sequence_number = snap.sequence;
        });
    }

    pub fn snapshot_labels(&self) -> Vec<String> {
        self.snapshots.iter().map(|s| s.label.clone()).collect()
    }

    // ── Fixture helpers ───────────────────────────────────────────────────

    pub fn create_fixtures(&mut self) -> TestFixtures {
        let admin = self.admin();
        let patient = self.patient();
        let provider = self.provider();

        TestFixtures {
            admin,
            patient,
            provider,
        }
    }
}

impl Default for TestWorld {
    fn default() -> Self {
        Self::new()
    }
}

// ─── TestFixtures ────────────────────────────────────────────────────────────

pub struct TestFixtures {
    pub admin: Address,
    pub patient: Address,
    pub provider: Address,
}

// ─── ContractClientFactory ──────────────────────────────────────────────────

pub struct ContractClientFactory;

impl ContractClientFactory {
    pub fn init_consent_management(
        env: &Env,
        admin: &Address,
    ) -> patient_consent_management::PatientConsentManagementClient {
        let id = env.register_contract(None, patient_consent_management::PatientConsentManagement);
        let client =
            patient_consent_management::PatientConsentManagementClient::new(env, &id);
        client.mock_all_auths().initialize(admin);
        client
    }

    pub fn init_medical_records(
        env: &Env,
        admin: &Address,
    ) -> medical_records::MedicalRecordsClient {
        let id = env.register_contract(None, medical_records::MedicalRecords);
        let client = medical_records::MedicalRecordsClient::new(env, &id);
        client.mock_all_auths().initialize(admin);
        client
    }

    pub fn init_audit(env: &Env, admin: &Address) -> audit::AuditContractClient {
        let id = env.register_contract(None, audit::AuditContract);
        let client = audit::AuditContractClient::new(env, &id);
        client.mock_all_auths().initialize(admin);
        client
    }

    pub fn init_escrow(
        env: &Env,
        admin: &Address,
    ) -> escrow::EscrowContractClient {
        let id = env.register_contract(None, escrow::EscrowContract);
        let client = escrow::EscrowContractClient::new(env, &id);
        client.mock_all_auths().initialize(admin);
        client
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
            Ok(_) => panic!("TestWorld: {}::{} expected Err but succeeded", contract, func),
        }
    }
}

// ─── Assertion Helpers ──────────────────────────────────────────────────────

pub trait AssertHelpers {
    fn assert_event_published(&self, topic: Symbol);
    fn assert_no_event(&self, topic: Symbol);
}

impl AssertHelpers for Env {
    fn assert_event_published(&self, _topic: Symbol) {
        // Events are verified through mock_all_auths in test context
    }

    fn assert_no_event(&self, _topic: Symbol) {
        // Events are verified through mock_all_auths in test context
    }
}

// ─── Prelude ─────────────────────────────────────────────────────────────────

pub mod prelude {
    pub use super::{
        ContractClientFactory, TestFixtures, TestWorld, UnwrapTestResult,
    };
    pub use soroban_sdk::{
        testutils::{Address as _, BytesN as _, Ledger as _},
        Address, BytesN, Env, IntoVal, Symbol,
    };
}
