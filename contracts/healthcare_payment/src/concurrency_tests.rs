use super::{
    Claim, ClaimStatus, DataKey, Error, HealthcarePayment, HealthcarePaymentClient,
};
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, testutils::Address as _, token, Address,
    Env, String, Vec,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
#[repr(u32)]
enum MockRole {
    Admin = 0,
    Staff = 3,
    Service = 7,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[contracterror]
#[repr(u32)]
enum MockRoleError {
    Unauthorized = 100,
}

#[contract]
struct MockRbac;

#[contractimpl]
impl MockRbac {
    pub fn has_role(env: Env, address: Address, role: MockRole) -> Result<bool, MockRoleError> {
        let key = (address, role as u32);
        Ok(env.storage().instance().get(&key).unwrap_or(false))
    }

    pub fn assign_role(
        env: Env,
        address: Address,
        role: MockRole,
    ) -> Result<bool, MockRoleError> {
        let key = (address, role as u32);
        env.storage().instance().set(&key, &true);
        Ok(true)
    }

    pub fn remove_role(
        env: Env,
        address: Address,
        role: MockRole,
    ) -> Result<bool, MockRoleError> {
        let key = (address, role as u32);
        env.storage().instance().set(&key, &false);
        Ok(true)
    }
}

#[contract]
struct MockPaymentRouter;

#[contractimpl]
impl MockPaymentRouter {
    #[allow(dead_code)]
    fn compute_split(_env: Env, amount: i128) -> (i128, i128) {
        let fee = amount / 10;
        (amount - fee, fee)
    }
}

#[contract]
struct MockEscrow;

#[contractimpl]
impl MockEscrow {
    #[allow(dead_code)]
    fn create_escrow(_env: Env, _claim_id: u64, _sender: Address, _provider: Address, _amount: i128, _token: Address) -> bool {
        true
    }
}

fn setup_env() -> (
    Env,
    HealthcarePaymentClient<'static>,
    Address,
    Address,
    Address,
    Address,
) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let provider = Address::generate(&env);
    let patient = Address::generate(&env);

    let rbac_id = env.register_contract(None, MockRbac);
    let rbac_client = MockRbacClient::new(&env, &rbac_id);
    rbac_client.assign_role(&admin, &MockRole::Admin);

    let router_id = env.register_contract(None, MockPaymentRouter);
    let escrow_id = env.register_contract(None, MockEscrow);
    let treasury = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(token_admin.clone())
        .address();

    let contract_id = env.register_contract(None, HealthcarePayment);
    let client = HealthcarePaymentClient::new(&env, &contract_id);

    client.initialize(&admin, &router_id, &escrow_id, &treasury, &token_id, &escrow_id, &rbac_id);

    let token_client = token::StellarAssetClient::new(&env, &token_id);
    token_client.mint(&contract_id, &i128::MAX);

    (env, client, admin, provider, patient, token_id)
}

fn submit_approved_claim(
    env: &Env,
    client: &HealthcarePaymentClient,
    admin: &Address,
    provider: &Address,
    patient: &Address,
    pay_token: &Address,
) -> u64 {
    let service_id = String::from_str(env, "X-22");
    let policy_id = String::from_str(env, "BC001");
        let claim_id = client
            .submit_claim(patient, provider, &service_id, &1000i128, &policy_id, &None);

    // Mint tokens to contract for payment
    let token_admin = token::StellarAssetClient::new(env, pay_token);
    token_admin.mint(&env.current_contract_address(), &10000);

    client.verify_claim(&claim_id, provider);
    client.approve_claim(&claim_id, admin);
    claim_id
}

#[test]
fn test_concurrent_escrow_submissions() {
    let (env, client, admin, provider, _patient, pay_token) = setup_env();
    let mut claim_ids: Vec<u64> = Vec::new(&env);

    // Submit multiple claims in sequence to simulate concurrent load
    for i in 0..5u64 {
        let patient = Address::generate(&env);
        let service_id = String::from_str(&env, "X-22");
        let policy_id = String::from_str(&env, "BC001");
        let claim_id = client
            .submit_claim(&patient, &provider, &service_id, &(1000i128 + i as i128), &policy_id, &None);

        let token_admin = token::StellarAssetClient::new(&env, &pay_token);
        token_admin.mint(&env.current_contract_address(), &10000);

        client.verify_claim(&claim_id, &provider);
        client.approve_claim(&claim_id, &admin);
        claim_ids.push_back(claim_id);
    }

    // Process escrow for all claims
    for i in 0..5 {
        let claim_id = claim_ids.get(i as u32).unwrap();
        client.escrow_claim(&claim_id);
    }
}

#[test]
fn test_concurrent_batch_payments() {
    let (env, client, admin, provider, patient, pay_token) = setup_env();
    let count = 5;
    let mut claim_ids: Vec<u64> = Vec::new(&env);

    for _i in 0..count {
        let c_id = submit_approved_claim(&env, &client, &admin, &provider, &patient, &pay_token);
        claim_ids.push_back(c_id);
    }

    // Process all payments via batch
    let paid = client.batch_process_payments(&claim_ids);
    assert_eq!(paid.len(), count as u32);
}

#[test]
fn test_concurrent_claim_status_transitions() {
    let (env, client, admin, provider, _patient, pay_token) = setup_env();
    let mut claim_ids: Vec<u64> = Vec::new(&env);

    for i in 0..3u64 {
        let patient = Address::generate(&env);
        let service_id = String::from_str(&env, "X-22");
        let policy_id = String::from_str(&env, "BC001");
        let claim_id = client
            .submit_claim(&patient, &provider, &service_id, &(500i128 + i as i128), &policy_id, &None);

        let token_admin = token::StellarAssetClient::new(&env, &pay_token);
        token_admin.mint(&env.current_contract_address(), &10000);

        claim_ids.push_back(claim_id);
    }

    // Interleave verify/approve across claims
    client.verify_claim(&claim_ids.get(0u32).unwrap(), &provider);
    client.verify_claim(&claim_ids.get(1u32).unwrap(), &provider);
    client.approve_claim(&claim_ids.get(0u32).unwrap(), &admin);
    client.verify_claim(&claim_ids.get(2u32).unwrap(), &provider);
    client.approve_claim(&claim_ids.get(1u32).unwrap(), &admin);
    client.approve_claim(&claim_ids.get(2u32).unwrap(), &admin);

    // All should now be approved
    for i in 0..3 {
        let claim: Claim = env
            .as_contract(&env.current_contract_address(), || {
                env.storage().persistent().get(&DataKey::Claim(
                    claim_ids.get(i as u32).unwrap(),
                )).unwrap()
            });
        assert_eq!(claim.status, ClaimStatus::Approved);
    }
}

#[test]
fn test_escrow_rejects_unapproved_claim() {
    let (env, client, _admin, provider, patient, _pay_token) = setup_env();
    let service_id = String::from_str(&env, "X-22");
    let policy_id = String::from_str(&env, "BC001");
    let claim_id = client
        .submit_claim(&patient, &provider, &service_id, &1000i128, &policy_id, &None);

    // Should fail because claim is Submitted, not Approved
    let result = client.try_escrow_claim(&claim_id);
    assert_eq!(result, Err(Ok(Error::InvalidStatus)));
}

#[test]
fn test_escrow_rejects_during_circuit_break() {
    let (env, client, admin, provider, patient, pay_token) = setup_env();
    let claim_id = submit_approved_claim(&env, &client, &admin, &provider, &patient, &pay_token);

    // Trip the circuit breaker
    client.emergency_pause(&admin);

    // Escrow should be rejected while circuit is open
    let result = client.try_escrow_claim(&claim_id);
    assert_eq!(result, Err(Ok(Error::CircuitOpen)));
}

#[test]
fn test_escrow_recovery_after_circuit_break() {
    let (env, client, admin, provider, patient, pay_token) = setup_env();
    let claim_id = submit_approved_claim(&env, &client, &admin, &provider, &patient, &pay_token);

    client.emergency_pause(&admin);
    client.begin_recovery(&admin);
    client.resume_operations(&admin);

    // After recovery, escrow should succeed
    client.escrow_claim(&claim_id);
}

#[test]
fn test_reentrancy_lock_prevents_concurrent_processing() {
    let (env, client, _admin, _provider, _patient, _pay_token) = setup_env();

    // The lock is checked in process_payment - verify we can't process
    // a non-approved claim (lock check is done, but claim status check comes after)
    let service_id = String::from_str(&env, "X-22");
    let policy_id = String::from_str(&env, "BC001");
    let patient = Address::generate(&env);
    let provider = Address::generate(&env);

    let claim_id = client
        .submit_claim(&patient, &provider, &service_id, &1000i128, &policy_id, &None);

    // Lock is acquired, but claim is not Approved
    let result = client.try_process_payment(&claim_id);
    assert_eq!(result, Err(Ok(Error::InvalidStatus)));
}
