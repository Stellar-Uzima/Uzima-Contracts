#![allow(clippy::unwrap_used)]

use crate::{contract::TokenSaleContractClient, vesting::VestingContractClient};

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env,
};

fn create_token_contract<'a>(
    e: &Env,
    admin: &Address,
) -> (Address, token::Client<'a>, token::StellarAssetClient<'a>) {
    let contract_address = e
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    (
        contract_address.clone(),
        token::Client::new(e, &contract_address),
        token::StellarAssetClient::new(e, &contract_address),
    )
}

#[test]
fn test_sale_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TokenSaleContract);
    let client = TokenSaleContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let treasury = Address::generate(&env);

    // Initialize with 6 arguments: admin, token, treasury, start, end, rate
    client.initialize(
        &admin, &token, &treasury, &100u64,   // start_time
        &1000u64,  // end_time
        &2000i128, // rate
    );

    let _buyer = Address::generate(&env);

    // Verify state
    let info = client.get_sale_info();
    assert_eq!(info.rate, 2000);
    assert_eq!(info.start_time, 100);
    assert_eq!(info.total_sold, 0);
}
