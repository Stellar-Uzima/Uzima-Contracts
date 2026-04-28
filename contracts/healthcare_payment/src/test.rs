use super::{ClaimSubmissionStatus, Error, HealthcarePayment, HealthcarePaymentClient};
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

#[test]
fn test_insurance_eligibility_claim_submission_and_eob_flow() {
    let (env, client, admin, provider, patient, _, _, _) = setup_env_and_clients();

    let insurance_provider_id = client.register_insurance_provider(
        &admin,
        &String::from_str(&env, "Uzima Insurance"),
        &String::from_str(&env, "UZM001"),
        &true,
        &true,
    );

    let coverage_policy_id = client.register_coverage_policy(
        &patient,
        &patient,
        &insurance_provider_id,
        &String::from_str(&env, "POLICY-INS-1"),
        &String::from_str(&env, "MEMBER-77"),
        &String::from_str(&env, "GROUP-A"),
        &500i128,
        &25i128,
        &2000u32,
    );

    let eligibility_id = client.verify_insurance_eligibility(
        &provider,
        &coverage_policy_id,
        &String::from_str(&env, "CONSULT-01"),
        &8000u32,
        &String::from_str(&env, "271-ACK-001"),
    );
    let eligibility = client.get_eligibility_check(&eligibility_id);
    assert!(eligibility.eligible);
    assert_eq!(eligibility.deductible_remaining, 500);

    let claim_id = client.submit_claim(
        &patient,
        &provider,
        &String::from_str(&env, "CONSULT-01"),
        &1000i128,
        &String::from_str(&env, "POLICY-INS-1"),
        &None,
    );

    assert!(client.submit_insurance_claim(
        &provider,
        &claim_id,
        &coverage_policy_id,
        &String::from_str(&env, "PAYER-REF-001"),
        &String::from_str(&env, "837"),
    ));

    let enrollment_id = client.sync_coverage_enrollment(
        &admin,
        &coverage_policy_id,
        &String::from_str(&env, "ENROLL-ACK-001"),
        &String::from_str(&env, "834"),
    );
    let enrollment = client.get_coverage_enrollment(&enrollment_id);
    assert_eq!(enrollment.policy_id, coverage_policy_id);

    assert!(client.process_eob(
        &admin,
        &claim_id,
        &coverage_policy_id,
        &900i128,
        &700i128,
        &150i128,
        &String::from_str(&env, "Adjudicated successfully"),
        &String::from_str(&env, "835"),
    ));

    let submission = client.get_claim_submission(&claim_id);
    assert_eq!(submission.status, ClaimSubmissionStatus::Adjudicated);

    let eob = client.get_explanation_of_benefits(&claim_id);
    assert_eq!(eob.insurer_paid, 700);
    assert_eq!(eob.patient_responsibility, 225);

    let responsibility = client.get_patient_responsibility(&patient);
    assert!(responsibility.is_some());
    if let Some(responsibility) = responsibility {
        assert_eq!(responsibility.total_copay_tracked, 25);
        assert_eq!(responsibility.total_deductible_tracked, 150);
    }

    let policy = client.get_coverage_policy(&coverage_policy_id);
    assert_eq!(policy.deductible_met, 150);
}

#[test]
fn test_insurance_claim_requires_matching_policy() {
    let (env, client, admin, provider, patient, _, _, _) = setup_env_and_clients();

    let insurance_provider_id = client.register_insurance_provider(
        &admin,
        &String::from_str(&env, "Uzima Insurance"),
        &String::from_str(&env, "UZM002"),
        &true,
        &true,
    );

    let coverage_policy_id = client.register_coverage_policy(
        &patient,
        &patient,
        &insurance_provider_id,
        &String::from_str(&env, "POLICY-MATCH"),
        &String::from_str(&env, "MEMBER-88"),
        &String::from_str(&env, "GROUP-B"),
        &300i128,
        &10i128,
        &1000u32,
    );

    client.verify_insurance_eligibility(
        &provider,
        &coverage_policy_id,
        &String::from_str(&env, "LAB-01"),
        &9000u32,
        &String::from_str(&env, "271-ACK-XYZ"),
    );

    let claim_id = client.submit_claim(
        &patient,
        &provider,
        &String::from_str(&env, "LAB-01"),
        &400i128,
        &String::from_str(&env, "POLICY-OTHER"),
        &None,
    );

    let result = client.try_submit_insurance_claim(
        &provider,
        &claim_id,
        &coverage_policy_id,
        &String::from_str(&env, "PAYER-REF-999"),
        &String::from_str(&env, "837"),
    );
    assert_eq!(result, Err(Ok(Error::PolicyMismatch)));
}

#[test]
fn test_error_codes_are_stable() {
    assert_eq!(Error::Unauthorized as u32, 100);
    assert_eq!(Error::InvalidAmount as u32, 205);
    assert_eq!(Error::NotInitialized as u32, 300);
    assert_eq!(Error::AlreadyInitialized as u32, 301);
    assert_eq!(Error::ContractPaused as u32, 302);
    assert_eq!(Error::ClaimNotFound as u32, 480);
    assert_eq!(Error::InsufficientFunds as u32, 500);
}

#[test]
fn test_get_suggestion_returns_expected_hint() {
    use soroban_sdk::symbol_short;
    assert_eq!(
        super::errors::get_suggestion(Error::Unauthorized),
        symbol_short!("CHK_AUTH")
    );
    assert_eq!(
        super::errors::get_suggestion(Error::NotInitialized),
        symbol_short!("INIT_CTR")
    );
    assert_eq!(
        super::errors::get_suggestion(Error::AlreadyInitialized),
        symbol_short!("ALREADY")
    );
    assert_eq!(
        super::errors::get_suggestion(Error::ContractPaused),
        symbol_short!("RE_TRY_L")
    );
    assert_eq!(
        super::errors::get_suggestion(Error::InsufficientFunds),
        symbol_short!("ADD_FUND")
    );
    assert_eq!(
        super::errors::get_suggestion(Error::ClaimNotFound),
        symbol_short!("CHK_ID")
    );
}
