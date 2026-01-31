#![no_std]
use soroban_sdk::{contract, contractimpl};

#[contract]
pub struct AnomalyDetector;

#[contractimpl]
impl AnomalyDetector {
    pub fn hello(env: soroban_sdk::Env) -> soroban_sdk::String {
        soroban_sdk::String::from_str(&env, "Anomaly Detector Placeholder")
    }
}
