#![no_std]

use soroban_sdk::{
    contract, contractimpl, contractmeta, contracttype, token, Address, Env, String,
};

contractmeta!(
    key = "Description",
    val = "SUT Token - Stellar Utility Token for the ecosystem"
);

#[derive(Clone)]
#[contracttype]
pub struct TokenMetadata {
    pub decimal: u32,
    pub name: String,
    pub symbol: String,
}

#[contract]
pub struct SutToken;

#[contractimpl]
impl SutToken {
    pub fn initialize(env: Env, admin: Address, decimal: u32, name: String, symbol: String) {
        let metadata = TokenMetadata {
            decimal,
            name,
            symbol,
        };
        env.storage().instance().set(&"METADATA", &metadata);

        // Note: In Soroban SDK 20.5.0, token initialization is handled differently
        // This is a simplified version for demonstration
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        // Mint functionality - in production this would implement proper token minting
        // For now, this is a placeholder
    }

    pub fn burn(env: Env, from: Address, amount: i128) {
        // Burn functionality - in production this would implement proper token burning
        // For now, this is a placeholder
    }
}
