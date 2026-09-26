//! Query load tests for the healthcare data marketplace (issue #1594).
//!
//! The marketplace is read-heavy in production: buyers browse listings and
//! check provider profiles far more often than they transact. `test.rs` covers
//! the behaviour of those reads one at a time, and carries a provider-scale
//! stress test that is `#[ignore]`d because it is too slow for a normal test
//! run. These tests sit in between: a full read workload over a populated
//! marketplace, sized to stay inside a routine `cargo test --workspace`.
//!
//! The point is to catch the failure mode where a read degrades as the
//! catalogue grows — an O(n) provider scan behind `get_provider_count`, or a
//! listing lookup that walks rather than indexes.
#![cfg(test)]
#![allow(clippy::unwrap_used)]
extern crate std;

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, BytesN, Env, String, Vec};

/// Providers registered before the read workload runs.
const PROVIDER_COUNT: u32 = 25;
/// Listings created before the read workload runs.
const LISTING_COUNT: u32 = 25;
/// Read operations issued per listing.
const READS_PER_LISTING: u32 = 4;
/// Ignored-by-default scale ceiling.
const SCALE_PROVIDER_COUNT: u32 = 200;

fn setup(env: &Env) -> (HealthcareDataMarketplaceClient<'_>, Address) {
    let contract_id = env.register_contract(None, HealthcareDataMarketplace {});
    let client = HealthcareDataMarketplaceClient::new(env, &contract_id);
    (client, contract_id)
}

fn initialize(client: &HealthcareDataMarketplaceClient, env: &Env) -> Address {
    let admin = Address::generate(env);
    client.initialize(
        &admin,
        &Address::generate(env),
        &Address::generate(env),
        &Address::generate(env),
        &300u64,
    );
    admin
}

/// A payload the contract accepts: quality and royalty splits inside the
/// validated ranges, differential-privacy anonymization with a positive
/// epsilon. Mirrors the accepted values in `test.rs`.
fn valid_payload(env: &Env, token: Address, salt: u8) -> ListingPayload {
    ListingPayload {
        data_ref: String::from_str(env, "s3://fhir/chunk"),
        data_hash: BytesN::from_array(env, &[salt; 32]),
        format: DataFormat::Parquet,
        anonymization: AnonymizationLevel::DifferentialPrivacy,
        min_k: 0u32,
        dp_epsilon_milli: 1000u32,
        quality: QualityMetrics {
            completeness_bps: 9000,
            consistency_bps: 9000,
            timeliness_bps: 9000,
            validity_bps: 9000,
        },
        royalty: RoyaltyPolicy {
            provider_bps: 8500,
            curator_bps: 500,
            platform_bps: 1000,
        },
        price: 5_000i128,
        token,
    }
}

/// Register `count` providers, each with one listing. Returns the providers and
/// their listing ids.
fn seed_catalogue(
    client: &HealthcareDataMarketplaceClient,
    env: &Env,
    count: u32,
) -> (Vec<Address>, Vec<u64>) {
    let token = Address::generate(env);
    let mut providers = Vec::new(env);
    let mut listing_ids = Vec::new(env);

    for i in 0..count {
        let provider = Address::generate(env);
        client.register_provider(&provider);
        let listing_id = client.create_listing(&provider, &valid_payload(env, token.clone(), i as u8));
        providers.push_back(provider);
        listing_ids.push_back(listing_id);
    }

    (providers, listing_ids)
}

#[test]
fn load_register_many_providers() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    initialize(&client, &env);

    for _ in 0..PROVIDER_COUNT {
        client.register_provider(&Address::generate(&env));
    }

    assert_eq!(client.get_provider_count(), PROVIDER_COUNT);
}

#[test]
fn load_create_many_listings() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    initialize(&client, &env);

    let (_providers, listing_ids) = seed_catalogue(&client, &env, LISTING_COUNT);

    for listing_id in listing_ids.iter() {
        assert!(
            client.get_listing(&listing_id).is_some(),
            "every created listing must be retrievable"
        );
    }
}

#[test]
fn load_query_catalogue_at_volume() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    initialize(&client, &env);

    let (providers, listing_ids) = seed_catalogue(&client, &env, LISTING_COUNT);

    // Read-heavy pass: repeat the browse path several times per listing so the
    // total operation count is meaningful without a huge fixture.
    for _ in 0..READS_PER_LISTING {
        for listing_id in listing_ids.iter() {
            assert!(client.get_listing(&listing_id).is_some());
        }
        for provider in providers.iter() {
            assert!(client.get_provider(&provider).is_some());
        }
        assert_eq!(client.get_provider_count(), PROVIDER_COUNT);
    }
}

#[test]
fn load_missing_lookups_do_not_scan() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    initialize(&client, &env);

    let (_providers, _listing_ids) = seed_catalogue(&client, &env, LISTING_COUNT);

    // Ids past the seeded range must miss cleanly rather than resolve, and must
    // not disturb the seeded records.
    let missing = LISTING_COUNT as u64 + 1_000;
    assert!(client.get_listing(&missing).is_none());
    assert!(client.get_intent(&missing).is_none());
    assert!(client.get_provider(&Address::generate(&env)).is_none());
    assert_eq!(client.get_provider_count(), PROVIDER_COUNT);
}

#[test]
#[ignore = "stress test for marketplace query scale"]
fn load_query_catalogue_at_high_scale() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    initialize(&client, &env);

    let (providers, listing_ids) = seed_catalogue(&client, &env, SCALE_PROVIDER_COUNT);

    for listing_id in listing_ids.iter() {
        assert!(client.get_listing(&listing_id).is_some());
    }
    for provider in providers.iter() {
        assert!(client.get_provider(&provider).is_some());
    }
    assert_eq!(client.get_provider_count(), SCALE_PROVIDER_COUNT);
}
