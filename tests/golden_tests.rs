#![cfg(test)]

use soroban_sdk::testutils::{Address as _, Events, Ledger as _};
use soroban_sdk::{
    contract, contractimpl, Address, BytesN, Env, String, Symbol, Vec,
};

use escrow::EscrowContract;
use identity_registry::{IdentityRegistryContract, IdentityRegistryContractClient};
use timelock::{Timelock, TimelockClient};

// ---------------------------------------------------------------------------
// Mock RBAC contract for identity_registry
// ---------------------------------------------------------------------------

#[contract]
struct MockRbac;

#[contractimpl]
impl MockRbac {
    pub fn init_mock(_env: Env) {}

    pub fn has_role(env: Env, address: Address, role: u32) -> Result<bool, u32> {
        let key = (address, role);
        Ok(env.storage().instance().get(&key).unwrap_or(false))
    }

    pub fn assign_role(env: Env, address: Address, role: u32) -> Result<bool, u32> {
        let key = (address, role);
        env.storage().instance().set(&key, &true);
        Ok(true)
    }

    pub fn remove_role(env: Env, address: Address, role: u32) -> Result<bool, u32> {
        let key = (address, role);
        env.storage().instance().set(&key, &false);
        Ok(true)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn load_golden(name: &str) -> serde_json::Value {
    let path = format!(
        "{}/golden/events/{}.json",
        env!("CARGO_MANIFEST_DIR"),
        name
    );
    let content = std::fs::read_to_string(&path).expect("golden file not found");
    serde_json::from_str(&content).expect("invalid golden JSON")
}

fn get_event_name(env: &Env, topics: &soroban_sdk::Vec<soroban_sdk::Val>) -> Option<Symbol> {
    let topic = topics.get(0)?;
    Symbol::try_from_val(env, &topic).ok()
}

fn verify_golden_events(env: &Env, contract_name: &str) {
    let golden = load_golden(contract_name);
    assert_eq!(golden["contract"].as_str().unwrap(), contract_name);

    let events = env.events().all();
    let expected_events = golden["events"].as_array().unwrap();

    for expected in expected_events {
        let name = expected["name"].as_str().unwrap();
        let expected_data_len = expected["data"].as_object().unwrap().len();
        let matched = events.iter().any(|e| {
            let Some(sym) = get_event_name(env, &e.topics) else {
                return false;
            };
            if sym != Symbol::new(env, name) {
                return false;
            }
            e.data.len() == expected_data_len
        });
        assert!(
            matched,
            "Event '{name}' with {expected_data_len} data field(s) not emitted"
        );
    }
}

// ---------------------------------------------------------------------------
// identity_registry golden test
// ---------------------------------------------------------------------------

#[test]
fn test_identity_registry_golden() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(10_000);

    let rbac_id = env.register_contract(None, MockRbac);
    let rbac_client = MockRbacClient::new(&env, &rbac_id);
    let contract_id = env.register_contract(None, IdentityRegistryContract);
    let client = IdentityRegistryContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    rbac_client.assign_role(&owner, &0u32);
    let network_id = String::from_str(&env, "testnet");
    client.initialize(&owner, &network_id, &rbac_id);

    let subject = Address::generate(&env);
    let public_key = BytesN::from_array(&env, &[1u8; 32]);
    let services: Vec<identity_registry::ServiceEndpoint> = Vec::new(&env);
    client.create_did(&subject, &public_key, &services);

    env.ledger().set_timestamp(20_000);
    let new_key = BytesN::from_array(&env, &[2u8; 32]);
    let method_id = String::from_str(&env, "#key-1");
    client.rotate_key(&subject, &method_id, &new_key);

    let credential_hash = BytesN::from_array(&env, &[3u8; 32]);
    let credential_uri = String::from_str(&env, "ipfs://QmTest");
    client.issue_credential(
        &owner,
        &subject,
        &identity_registry::CredentialType::MedicalLicense,
        &credential_hash,
        &credential_uri,
        &0u64,
    );

    verify_golden_events(&env, "identity_registry");
}

// ---------------------------------------------------------------------------
// timelock golden test
// ---------------------------------------------------------------------------

#[test]
fn test_timelock_golden() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, Timelock);
    let client = TimelockClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &10u64);

    let target = Address::generate(&env);
    let call = BytesN::from_array(&env, &[0u8; 32]);
    client.queue(&1u64, &target, &call);

    env.ledger().set_timestamp(env.ledger().timestamp() + 15);
    client.execute(&1u64);

    verify_golden_events(&env, "timelock");
}

// ---------------------------------------------------------------------------
// escrow golden test
// ---------------------------------------------------------------------------

#[test]
fn test_escrow_golden() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, EscrowContract);
    let client = escrow::EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let payer = Address::generate(&env);
    let payee = Address::generate(&env);
    let token = Address::generate(&env);
    let fee_receiver = Address::generate(&env);

    client.initialize(&admin);
    client.set_fee_config(&admin, &fee_receiver, &250u32);

    client.create_escrow(&1u64, &payer, &payee, &1000i128, &token);
    client.mark_disputed(&payer, &1u64);

    let arbiter = Address::generate(&env);
    client.approve_release(&1u64, &payer);
    client.approve_release(&1u64, &arbiter);
    client.release_escrow(&1u64);

    client.create_escrow(&2u64, &payer, &payee, &500i128, &token);
    let reason = String::from_str(&env, "order cancelled");
    client.refund_escrow(&2u64, &reason);

    verify_golden_events(&env, "escrow");
}
