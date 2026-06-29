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

// ── Property-Based Tests (Issue #832) ─────────────────────────

// Property 1: Listings count increases monotonically
#[test]
fn proptest_listing_count_monotonicity() {
    use proptest::proptest;
    proptest!(|(listing_count in 1usize..=30)| {
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

        for i in 0..listing_count {
            let payload = ListingPayload {
                data_ref: String::from_str(&env, "ipfs://data"),
                data_hash: BytesN::from_array(&env, &[i as u8; 32]),
                format: DataFormat::FhirJson,
                anonymization: AnonymizationLevel::KAnonymity,
                min_k: 3u32,
                dp_epsilon_milli: 0u32,
                quality,
                royalty,
                price: 1_000i128,
                token: Address::generate(&env),
            };
            let _ = client.create_listing(&provider, &payload);
        }

        let final_id = client.get_next_listing_id();
        prop_assert_eq!(final_id as usize, listing_count, 
            "Listing count must equal number of created listings");
    });
}

// Property 2: Price must be positive
#[test]
fn proptest_price_validation() {
    use proptest::proptest;
    proptest!(|(price in 1i128..=i128::MAX)| {
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
            data_ref: String::from_str(&env, "ipfs://data"),
            data_hash: BytesN::from_array(&env, &[1u8; 32]),
            format: DataFormat::FhirJson,
            anonymization: AnonymizationLevel::KAnonymity,
            min_k: 3u32,
            dp_epsilon_milli: 0u32,
            quality,
            royalty,
            price,
            token: Address::generate(&env),
        };

        // Should succeed with positive price
        let result = client.try_create_listing(&provider, &payload);
        prop_assert!(result.is_ok(), "Positive price {} should be valid", price);
    });
}

// Property 3: Provider count monotonicity
#[test]
fn proptest_provider_count_monotonicity() {
    use proptest::proptest;
    proptest!(|(provider_count in 1usize..=50)| {
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

        let mut prev_count = 0u64;
        for _ in 0..provider_count {
            let provider = Address::generate(&env);
            let res = client.try_register_provider(&provider);
            if res.is_ok() {
                let count = client.get_provider_count();
                prop_assert_eq!(count, prev_count + 1,
                    "Provider count must increase by 1 after registration");
                prev_count = count;
            }
        }
    });
}

// Property 4: Settlement window must be valid (1-300 seconds)
#[test]
fn proptest_settlement_window_validation() {
    use proptest::proptest;
    proptest!(|(window_secs in 1u64..=300)| {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _) = setup(&env);
        let admin = Address::generate(&env);

        let result = client.try_initialize(
            &admin,
            &Address::generate(&env),
            &Address::generate(&env),
            &Address::generate(&env),
            &window_secs,
        );
        prop_assert!(result.is_ok(), 
            "Settlement window {} should be valid (1-300)", window_secs);
    });
}

// Property 5: Intent ID counter monotonicity
#[test]
fn proptest_intent_id_monotonicity() {
    use proptest::proptest;
    proptest!(|(intent_count in 1usize..=30)| {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _) = setup(&env);
        let admin = Address::generate(&env);
        let payment_router = env.register_contract(None, MockPaymentRouter {});
        let escrow = env.register_contract(None, MockEscrow {});
        
        client.initialize(&admin, &payment_router, &escrow, &Address::generate(&env), &300u64);

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
            data_ref: String::from_str(&env, "ipfs://data"),
            data_hash: BytesN::from_array(&env, &[1u8; 32]),
            format: DataFormat::FhirJson,
            anonymization: AnonymizationLevel::KAnonymity,
            min_k: 3u32,
            dp_epsilon_milli: 0u32,
            quality,
            royalty,
            price: 1_000i128,
            token: Address::generate(&env),
        };

        let listing_id = client.create_listing(&provider, &payload);
        
        for _ in 0..intent_count {
            let buyer = Address::generate(&env);
            let _ = client.reserve_purchase(&buyer, &listing_id);
        }

        let next_id = client.get_next_intent_id();
        prop_assert_eq!(next_id as usize, intent_count,
            "Intent ID counter must equal number of created intents");
    });
}

// Property 6: Royalty percentages sum constraint (must be <= 10000 bps)
#[test]
fn proptest_royalty_sum_constraint() {
    use proptest::proptest;
    proptest!(|(provider_bps in 0u32..=8000, curator_bps in 0u32..=1500, platform_bps in 0u32..=1500)| {
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

        let provider = Address::generate(&env);
        client.register_provider(&provider);

        let total = (provider_bps as u64) + (curator_bps as u64) + (platform_bps as u64);
        
        if total <= 10000 {
            let quality = QualityMetrics {
                completeness_bps: 9000,
                consistency_bps: 9000,
                timeliness_bps: 9000,
                validity_bps: 9000,
            };
            let royalty = RoyaltyPolicy {
                provider_bps,
                curator_bps,
                platform_bps,
            };

            let payload = ListingPayload {
                data_ref: String::from_str(&env, "ipfs://data"),
                data_hash: BytesN::from_array(&env, &[1u8; 32]),
                format: DataFormat::FhirJson,
                anonymization: AnonymizationLevel::KAnonymity,
                min_k: 3u32,
                dp_epsilon_milli: 0u32,
                quality,
                royalty,
                price: 1_000i128,
                token: Address::generate(&env),
            };

            let result = client.try_create_listing(&provider, &payload);
            prop_assert!(result.is_ok(), 
                "Royalty split {} bps should be valid", total);
        }
    });
}
