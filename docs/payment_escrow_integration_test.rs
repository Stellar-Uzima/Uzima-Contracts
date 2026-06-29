#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env, Map,
};
use token::Client as TokenClient;
use token::StellarAssetClient as TokenAdminClient;

// Import the compiled WASM files for integration testing.
// Ensure these contracts are built before running the tests.
mod payment_router {
    soroban_sdk::contractimport!(
        file = "../target/wasm32-unknown-unknown/release/payment_router.wasm"
    );
}

mod escrow {
    soroban_sdk::contractimport!(
        file = "../target/wasm32-unknown-unknown/release/appointment_booking_escrow.wasm"
    );
}

fn setup_env() -> (Env, Address, Address, TokenClient<'static>, TokenAdminClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract(token_admin.clone());
    let token = TokenClient::new(&env, &token_contract);
    let token_admin_client = TokenAdminClient::new(&env, &token_contract);
    
    // Mint 10,000,000 tokens to the user for testing
    token_admin_client.mint(&user, &10_000_000);
    
    (env, admin, user, token, token_admin_client)
}

#[test]
fn test_payment_flow_with_successful_escrow_release() {
    let (env, admin, patient, token, _) = setup_env();
    let provider = Address::generate(&env);
    
    // Register contracts
    let escrow_id = env.register_contract_wasm(None, escrow::WASM);
    let escrow_client = escrow::Client::new(&env, &escrow_id);
    
    let router_id = env.register_contract_wasm(None, payment_router::WASM);
    let router_client = payment_router::Client::new(&env, &router_id);
    
    // Initialize escrow and router with a 2% platform fee (200 basis points)
    escrow_client.initialize(&admin, &token.address);
    router_client.initialize(&admin, &token.address, &escrow_id, &200);
    
    let amount = 100_000;
    
    // Route payment to escrow
    let payment_id = router_client.route_payment(&patient, &provider, &amount);
    
    assert_eq!(token.balance(&patient), 9_900_000);
    assert_eq!(token.balance(&escrow_id), amount);
    
    // Successful release by the provider
    router_client.release_payment(&provider, &payment_id);
    
    // Verify 2% fee to admin, 98% to provider
    assert_eq!(token.balance(&escrow_id), 0);
    assert_eq!(token.balance(&provider), 98_000);
    assert_eq!(token.balance(&admin), 2_000);
}

#[test]
fn test_payment_refund_on_dispute() {
    let (env, admin, patient, token, _) = setup_env();
    let provider = Address::generate(&env);
    
    let escrow_id = env.register_contract_wasm(None, escrow::WASM);
    let router_id = env.register_contract_wasm(None, payment_router::WASM);
    let router_client = payment_router::Client::new(&env, &router_id);
    
    escrow::Client::new(&env, &escrow_id).initialize(&admin, &token.address);
    router_client.initialize(&admin, &token.address, &escrow_id, &200);
    
    let amount = 100_000;
    let payment_id = router_client.route_payment(&patient, &provider, &amount);
    
    // Dispute and refund initiated by admin/patient
    router_client.refund_payment(&admin, &payment_id);
    
    // Full amount returned to patient, no fees taken
    assert_eq!(token.balance(&patient), 10_000_000);
    assert_eq!(token.balance(&provider), 0);
    assert_eq!(token.balance(&admin), 0);
    assert_eq!(token.balance(&escrow_id), 0);
}

#[test]
fn test_multi_party_payment_splitting() {
    let (env, admin, patient, token, _) = setup_env();
    let provider1 = Address::generate(&env);
    let provider2 = Address::generate(&env);
    let hospital = Address::generate(&env);
    
    let escrow_id = env.register_contract_wasm(None, escrow::WASM);
    let router_id = env.register_contract_wasm(None, payment_router::WASM);
    let router_client = payment_router::Client::new(&env, &router_id);
    
    escrow::Client::new(&env, &escrow_id).initialize(&admin, &token.address);
    router_client.initialize(&admin, &token.address, &escrow_id, &200);
    
    let amount = 100_000;
    let payment_id = router_client.route_payment(&patient, &hospital, &amount);
    
    // Map to split payments between hospital and sub-providers
    let mut splits = Map::new(&env);
    splits.set(provider1.clone(), 40_000);
    splits.set(provider2.clone(), 30_000);
    splits.set(hospital.clone(), 28_000); // Leaves 2,000 for platform fees
    
    router_client.execute_split_payment(&hospital, &payment_id, &splits);
    
    assert_eq!(token.balance(&provider1), 40_000);
    assert_eq!(token.balance(&provider2), 30_000);
    assert_eq!(token.balance(&hospital), 28_000);
    assert_eq!(token.balance(&admin), 2_000);
}

#[test]
fn test_timeout_handling() {
    let (env, admin, patient, token, _) = setup_env();
    let provider = Address::generate(&env);
    
    let escrow_id = env.register_contract_wasm(None, escrow::WASM);
    let router_id = env.register_contract_wasm(None, payment_router::WASM);
    let router_client = payment_router::Client::new(&env, &router_id);
    
    escrow::Client::new(&env, &escrow_id).initialize(&admin, &token.address);
    router_client.initialize(&admin, &token.address, &escrow_id, &200);
    
    let amount = 100_000;
    // Route payment with a 24-hour timeout (86400 seconds)
    let payment_id = router_client.route_payment_with_timeout(&patient, &provider, &amount, &86400);
    
    // Fast forward ledger time beyond the timeout
    env.ledger().with_mut(|l| {
        l.timestamp += 86401;
    });
    
    // Claim timeout refund
    router_client.claim_timeout_refund(&patient, &payment_id);
    
    assert_eq!(token.balance(&patient), 10_000_000);
    assert_eq!(token.balance(&escrow_id), 0);
}

#[test]
fn test_fee_calculation_accuracy() {
    let (env, admin, patient, token, _) = setup_env();
    let provider = Address::generate(&env);
    
    let escrow_id = env.register_contract_wasm(None, escrow::WASM);
    let router_id = env.register_contract_wasm(None, payment_router::WASM);
    let router_client = payment_router::Client::new(&env, &router_id);
    
    escrow::Client::new(&env, &escrow_id).initialize(&admin, &token.address);
    
    // Initialize with a 0.5% fee (50 basis points)
    router_client.initialize(&admin, &token.address, &escrow_id, &50);
    
    // Test fee calculation on edge-case amounts
    let amounts = [100, 1_000, 50_500, 1_000_000];
    
    for amount in amounts {
        let prev_admin_bal = token.balance(&admin);
        let prev_provider_bal = token.balance(&provider);
        
        let payment_id = router_client.route_payment(&patient, &provider, &amount);
        router_client.release_payment(&provider, &payment_id);
        
        // Formula: (amount * fee_bps) / 10,000
        let expected_fee = (amount * 50) / 10_000;
        let expected_provider_amount = amount - expected_fee;
        
        assert_eq!(token.balance(&admin) - prev_admin_bal, expected_fee);
        assert_eq!(token.balance(&provider) - prev_provider_bal, expected_provider_amount);
    }
}

#[test]
#[should_panic(expected = "Amount below minimum threshold")]
fn test_minimum_amount_edge_case() {
    let (env, admin, patient, token, _) = setup_env();
    let provider = Address::generate(&env);
    
    let escrow_id = env.register_contract_wasm(None, escrow::WASM);
    let router_id = env.register_contract_wasm(None, payment_router::WASM);
    let router_client = payment_router::Client::new(&env, &router_id);
    
    escrow::Client::new(&env, &escrow_id).initialize(&admin, &token.address);
    router_client.initialize(&admin, &token.address, &escrow_id, &200);
    
    // Should panic if amount is too small to calculate fee reliably
    router_client.route_payment(&patient, &provider, &1);
}

#[test]
fn test_gas_optimization_verification() {
    let (env, admin, patient, token, _) = setup_env();
    let provider = Address::generate(&env);
    
    let escrow_id = env.register_contract_wasm(None, escrow::WASM);
    let router_id = env.register_contract_wasm(None, payment_router::WASM);
    let router_client = payment_router::Client::new(&env, &router_id);
    
    escrow::Client::new(&env, &escrow_id).initialize(&admin, &token.address);
    router_client.initialize(&admin, &token.address, &escrow_id, &200);
    
    // Reset budget to measure the true routing cost
    env.budget().reset_unlimited();
    
    let amount = 100_000;
    let payment_id = router_client.route_payment(&patient, &provider, &amount);
    
    // Verify gas/CPU instructions are within optimized bounds (e.g. < 2,000,000 instructions)
    let route_cpu_used = env.budget().cpu_instruction_cost();
    assert!(route_cpu_used < 2_000_000, "Routing gas usage exceeded optimization targets");
    
    env.budget().reset_unlimited();
    
    router_client.release_payment(&provider, &payment_id);
    
    let release_cpu_used = env.budget().cpu_instruction_cost();
    assert!(release_cpu_used < 2_000_000, "Release gas usage exceeded optimization targets");
}