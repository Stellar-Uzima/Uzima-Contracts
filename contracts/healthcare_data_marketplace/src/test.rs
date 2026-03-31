use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::testutils::Ledger as _;
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String};

#[contract]
struct MockPaymentRouter;

#[contractimpl]
impl MockPaymentRouter {
    pub fn compute_split(_env: Env, amount: i128) -> (i128, i128) {
        // 5% router fee for integration smoke test.
        let fee = amount / 20;
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

fn setup(env: &Env) -> (HealthcareDataMarketplaceClient<'_>, Address) {
    let contract_id = env.register_contract(None, HealthcareDataMarketplace {});
    let client = HealthcareDataMarketplaceClient::new(env, &contract_id);
    (client, contract_id)
}

#[test]
fn test_create_listing_requires_valid_anonymization_and_quality() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let admin = Address::generate(&env);
    let payment_router = Address::generate(&env);
    let escrow = Address::generate(&env);
    let treasury = Address::generate(&env);
    client.initialize(&admin, &payment_router, &escrow, &treasury, &300u64);

    let provider = Address::generate(&env);
    client.register_provider(&provider);

    let bad_quality = QualityMetrics {
        completeness_bps: 6500,
        consistency_bps: 9000,
        timeliness_bps: 9000,
        validity_bps: 9000,
    };
    let royalty = RoyaltyPolicy {
        provider_bps: 8000,
        curator_bps: 1000,
        platform_bps: 1000,
    };
    let payload = ListingPayload {
        data_ref: String::from_str(&env, "ipfs://dataset"),
        data_hash: BytesN::from_array(&env, &[1u8; 32]),
        format: DataFormat::FhirJson,
        anonymization: AnonymizationLevel::KAnonymity,
        min_k: 3u32,
        dp_epsilon_milli: 0u32,
        quality: bad_quality,
        royalty,
        price: 1_000i128,
        token: Address::generate(&env),
    };
    let result = client.try_create_listing(&provider, &payload);
    assert_eq!(result, Err(Ok(Error::InvalidAnonymization)));
}

#[test]
fn test_provider_counter_increments() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &admin,
        &Address::generate(&env),
        &Address::generate(&env),
        &Address::generate(&env),
        &300u64,
    );

    for _ in 0..25 {
        client.register_provider(&Address::generate(&env));
    }

    assert_eq!(client.get_provider_count(), 25);
}

#[test]
#[ignore = "stress test for provider scalability"]
fn test_provider_scale_to_1000_plus() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &admin,
        &Address::generate(&env),
        &Address::generate(&env),
        &Address::generate(&env),
        &300u64,
    );

    for _ in 0..1001 {
        client.register_provider(&Address::generate(&env));
    }

    assert_eq!(client.get_provider_count(), 1001);
}

#[test]
fn test_settlement_timeout_enforced_under_five_minutes() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let admin = Address::generate(&env);
    let payment_router = env.register_contract(None, MockPaymentRouter {});
    let escrow = env.register_contract(None, MockEscrow {});
    client.initialize(
        &admin,
        &payment_router,
        &escrow,
        &Address::generate(&env),
        &300u64,
    );
    let provider = Address::generate(&env);
    client.register_provider(&provider);

    let quality = QualityMetrics {
        completeness_bps: 9000,
        consistency_bps: 9000,
        timeliness_bps: 9000,
        validity_bps: 9000,
    };
    let royalty = RoyaltyPolicy {
        provider_bps: 8500,
        curator_bps: 500,
        platform_bps: 1000,
    };

    let payload = ListingPayload {
        data_ref: String::from_str(&env, "s3://fhir/chunk"),
        data_hash: BytesN::from_array(&env, &[7u8; 32]),
        format: DataFormat::Parquet,
        anonymization: AnonymizationLevel::DifferentialPrivacy,
        min_k: 0u32,
        dp_epsilon_milli: 1000u32,
        quality,
        royalty,
        price: 5_000i128,
        token: Address::generate(&env),
    };
    let listing_id = client.create_listing(&provider, &payload);
    let buyer = Address::generate(&env);
    let intent_id = client.reserve_purchase(&buyer, &listing_id);
    let _escrow_order_id = client.initiate_transaction(&buyer, &intent_id);

    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp.saturating_add(301);
    });
    let timeout_res = client.try_finalize_settlement(&buyer, &intent_id);
    assert_eq!(timeout_res, Err(Ok(Error::SettlementTimeout)));
}
