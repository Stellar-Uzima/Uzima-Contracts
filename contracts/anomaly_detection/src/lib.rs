#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct AnomalyDetector;

#[contractimpl]
impl AnomalyDetector {
    pub fn init(_env: Env) {}
}