#![allow(clippy::unwrap_used)]
extern crate std;
use std::time::Instant;

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env, Symbol, Vec};

struct BenchResult {
    name: &'static str,
    cpu_instructions: u64,
    memory_bytes: u64,
    wall_us: u128,
}

impl BenchResult {
    fn print(&self) {
        std::println!(
            "[BENCH] {:40} cpu={:>12} insns  mem={:>10} bytes  wall={:>8}µs",
            self.name, self.cpu_instructions, self.memory_bytes, self.wall_us
        );
    }

    fn assert_cpu_reduction(&self, baseline: u64, min_reduction_pct: u64) {
        let allowed = baseline.saturating_mul(100 - min_reduction_pct) / 100;
        assert!(
            self.cpu_instructions <= allowed,
            "[BENCH] {} did not meet expected reduction: {} > {}",
            self.name,
            self.cpu_instructions,
            allowed,
        );
    }
}

fn measure<F: FnOnce()>(env: &Env, name: &'static str, f: F) -> BenchResult {
    env.budget().reset_unlimited();
    let t0 = Instant::now();
    f();
    BenchResult {
        name,
        cpu_instructions: env.budget().cpu_instruction_cost(),
        memory_bytes: env.budget().memory_bytes_cost(),
        wall_us: t0.elapsed().as_micros(),
    }
}

#[contract]
pub struct MockFeedProvider;

#[contractimpl]
impl MockFeedProvider {
    pub fn get_feed_payload(env: Env, _feed_id: String) -> FeedPayload {
        FeedPayload::DrugPrice(DrugPriceData {
            ndc_code: String::from_str(&env, "NDC-12345"),
            currency: String::from_str(&env, "USD"),
            price_minor: 123450,
            availability_units: 75,
            observed_at: env.ledger().timestamp(),
        })
    }
}

#[test]
fn bench_cached_cross_contract_call() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = Address::generate(&env);
    env.register_contract(&contract_id, HealthcareOracleNetwork);
    let client = HealthcareOracleNetworkClient::new(&env, &contract_id);

    let provider_id = Address::generate(&env);
    env.register_contract(&provider_id, MockFeedProvider);
    let feed_id = String::from_str(&env, "benchmark-drug-feed");

    let uncached = measure(&env, "healthcare_oracle_network::raw_external_calls", || {
        let args = Vec::from_array(&env, [feed_id.clone().into_val(&env)]);
        let _: FeedPayload = env.invoke_contract(
            &provider_id,
            &Symbol::new(&env, "get_feed_payload"),
            args.clone(),
        );
        let _: FeedPayload = env.invoke_contract(
            &provider_id,
            &Symbol::new(&env, "get_feed_payload"),
            args.clone(),
        );
        let _: FeedPayload = env.invoke_contract(
            &provider_id,
            &Symbol::new(&env, "get_feed_payload"),
            args,
        );
    });
    uncached.print();

    let cached = measure(&env, "healthcare_oracle_network::cached_external_calls", || {
        let _ = client.fetch_external_payload(&provider_id, &feed_id);
        let _ = client.fetch_external_payload(&provider_id, &feed_id);
        let _ = client.fetch_external_payload(&provider_id, &feed_id);
    });
    cached.print();

    cached.assert_cpu_reduction(uncached.cpu_instructions, 40);
}
