#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

#[cfg(test)]
mod test;

// --- Types ---

#[contracttype]
#[derive(Clone)]
pub struct SaleInfo {
    pub admin: Address,
    pub token: Address,
    pub treasury: Address,
    pub start_time: u64,
    pub end_time: u64,
    pub rate: i128, // tokens per unit of currency
    pub total_sold: i128,
}

// --- Storage Keys ---

const SALE_INFO: Symbol = symbol_short!("SALE");

// --- Contract ---

#[contract]
pub struct TokenSaleContract;

#[contractimpl]
impl TokenSaleContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        token: Address,
        treasury: Address,
        start_time: u64,
        end_time: u64,
        rate: i128,
    ) {
        if env.storage().persistent().has(&SALE_INFO) {
            panic!("Already initialized");
        }

        let info = SaleInfo {
            admin,
            token,
            treasury,
            start_time,
            end_time,
            rate,
            total_sold: 0,
        };

        env.storage().persistent().set(&SALE_INFO, &info);
    }

    pub fn buy_tokens(env: Env, buyer: Address, amount: i128) {
        buyer.require_auth();

        let mut info: SaleInfo = env
            .storage()
            .persistent()
            .get(&SALE_INFO)
            .expect("Not initialized");

        let now = env.ledger().timestamp();
        if now < info.start_time || now > info.end_time {
            panic!("Sale not active");
        }

        if amount <= 0 {
            panic!("Invalid amount");
        }

        // Mock transfer logic:
        // In a real contract, we would call the token contract here:
        // token::Client::new(&env, &info.token).transfer(&info.treasury, &buyer, &amount);

        info.total_sold += amount;
        env.storage().persistent().set(&SALE_INFO, &info);
    }

    pub fn get_sale_info(env: Env) -> SaleInfo {
        env.storage()
            .persistent()
            .get(&SALE_INFO)
            .expect("Not initialized")
    }
}
