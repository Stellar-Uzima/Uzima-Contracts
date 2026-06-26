use soroban_sdk::{
    testutils::Address as _, Address, BytesN, Env, String, Vec,
};

use crate::{
    IdentityRegistryContract, IdentityRegistryContractClient, MockRbac, MockRbacClient,
    RbacRole, ServiceEndpoint,
};

/// A fully initialized test environment with pre-configured contracts.
pub struct TestEnv {
    pub env: Env,
    pub client: IdentityRegistryContractClient<'static>,
    pub owner: Address,
}

/// A DID subject with a freshly created DID document.
pub struct TestSubject {
    pub address: Address,
    pub did_string: String,
}

/// Create a standard test environment.
pub fn create_test_env() -> TestEnv {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let rbac_id = env.register_contract(None, MockRbac);
    let rbac_client = MockRbacClient::new(&env, &rbac_id);
    rbac_client.assign_role(&owner, &RbacRole::Admin);

    let contract_id = env.register_contract(None, IdentityRegistryContract);
    let client = IdentityRegistryContractClient::new(&env, &contract_id);
    client.initialize(
        &owner,
        &String::from_str(&env, "testnet"),
        &rbac_id,
    );

    TestEnv { env, client, owner }
}

/// Create a DID subject with a basic DID document.
pub fn create_did_subject(env: &TestEnv) -> TestSubject {
    let address = Address::generate(&env.env);
    let public_key = BytesN::<32>::from_array(&env.env, &[1u8; 32]);
    let services: Vec<ServiceEndpoint> = Vec::new(&env.env);

    let did_string = env.client.create_did(
        &address,
        &public_key,
        &services,
    );

    TestSubject { address, did_string }
}

/// Resolve a DID and assert it matches the expected subject.
pub fn assert_resolve_did(env: &TestEnv, subject: &TestSubject) {
    let doc = env.client.resolve_did(&subject.address);
    assert_eq!(doc.id, subject.did_string);
}

/// Create a subject with an attached service endpoint.
pub fn create_subject_with_service(
    env: &TestEnv,
    service_id: &str,
    service_type: &str,
    endpoint: &str,
) -> TestSubject {
    let subject = create_did_subject(env);
    env.client.add_service(
        &subject.address,
        &String::from_str(&env.env, service_id),
        &String::from_str(&env.env, service_type),
        &String::from_str(&env.env, endpoint),
    );
    subject
}
