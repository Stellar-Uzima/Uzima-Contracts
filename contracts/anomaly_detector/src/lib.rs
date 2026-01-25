#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Symbol, Env};

#[contract]
pub struct AnomalyDetector;

#[contractimpl]
impl AnomalyDetector {
    // Fixed: Returns a Symbol instead of an invalid &str
    pub fn hello(_env: Env) -> Symbol {
        symbol_short!("hello")
    }
}
