#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct AnomalyDetector;

#[contractimpl]
impl AnomalyDetector {
    pub fn hello(_env: Env) -> Symbol {
        // We use symbol_short! because the string is 32 characters or fewer
        symbol_short!("Anomaly")
    }
}
