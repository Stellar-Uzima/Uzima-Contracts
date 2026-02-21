use super::{Error, HealthcarePayment, HealthcarePaymentClient};
use soroban_sdk::{contract, contractimpl, testutils::Address as _, token, Address, Env, String};

#[contract]
struct MockPaymentRouter;

#[contractimpl]
impl MockPaymentRouter {
    pub fn compute_split(_env: Env, amount: i128) -> (i128, i128) {
        let fee = amount / 10;
        (amount.saturating_sub(fee), fee)
    }
}

#[contract]
struct MockEscrow;

#[contractimpl]
impl MockEscrow {
    pub fn create_escrow(
        _env: Env,
        _order_id: u64,
        _payer: Address,
        _payee: Address,
        _amount: i128,
        _token: Address,
    ) -> bool {
        true
    }
}

fn setup_env_and_clients() -> (
    Env,
    HealthcarePaymentClient<'static>,
    Address,
    Address,
    Address,
    Address,
    token::StellarAssetClient<'static>,
    token::Client<'static>,
) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let provider = Address::generate(&env);
    let patient = Address::generate(&env);
    let treasury = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let stellar_asset_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_id = stellar_asset_contract.address();

    let token_admin_client = token::StellarAssetClient::new(&env, &token_id);
    let token_client = token::Client::new(&env, &token_id);

    let router_id = env.register_contract(None, MockPaymentRouter);
    let escrow_id = env.register_contract(None, MockEscrow);

    let contract_id = env.register_contract(None, HealthcarePayment);
    let client = HealthcarePaymentClient::new(&env, &contract_id);

    client.initialize(&admin, &router_id, &escrow_id, &treasury, &token_id);

    token_admin_client.mint(&contract_id, &100_000_000);
    token_admin_client.mint(&patient, &100_000_000);

    (
        env,
        client,
        admin,
        provider,
        patient,
        treasury,
        token_admin_client,
        token_client,
    )
}

#[test]
fn test_submit_and_approve_claim() {
    let (env, client, admin, provider, patient, treasury, _, token_client) =
        setup_env_and_clients();

    let claim_id = client.submit_claim(
        &patient,
        &provider,
        &String::from_str(&env, "SERVICE-123"),
        &1000i128,
        &String::from_str(&env, "POLICY-XYZ"),
        &None,
    );

    assert_eq!(claim_id, 1);

    client.verify_claim(&claim_id, &admin);
    client.approve_claim(&claim_id, &admin);
    client.process_payment(&claim_id);

    assert_eq!(token_client.balance(&provider), 900);
    assert_eq!(token_client.balance(&treasury), 100);
}

#[test]
fn test_escrow_claim() {
    let (env, client, admin, provider, patient, _, _, _) = setup_env_and_clients();

    let claim_id = client.submit_claim(
        &patient,
        &provider,
        &String::from_str(&env, "SERVICE-456"),
        &2000i128,
        &String::from_str(&env, "POLICY-ABC"),
        &None,
    );

    client.verify_claim(&claim_id, &admin);
    client.approve_claim(&claim_id, &admin);

    client.escrow_claim(&claim_id);
}

#[test]
fn test_fraud_report() {
    let (env, client, admin, provider, patient, _, _, _) = setup_env_and_clients();

    let claim_id = client.submit_claim(
        &patient,
        &provider,
        &String::from_str(&env, "SERVICE-789"),
        &3000i128,
        &String::from_str(&env, "POLICY-DEF"),
        &None,
    );

    client.report_fraud(
        &claim_id,
        &admin,
        &String::from_str(&env, "Suspicious activity"),
    );

    let res = client.try_approve_claim(&claim_id, &admin);
    assert_eq!(res, Err(Ok(Error::FraudDetected)));
}

#[test]
fn test_payment_plan() {
    let (env, client, _, provider, patient, _, _, token_client) = setup_env_and_clients();

    token_client.approve(
        &patient,
        &client.address,
        &1000i128,
        &(env.ledger().sequence() + 1000),
    );

    let plan_id = client.create_payment_plan(&patient, &provider, &1000i128, &250i128, &86400u64);

    assert_eq!(plan_id, 1);

    client.pay_installment(&plan_id);

    assert_eq!(token_client.balance(&provider), 250);
}
