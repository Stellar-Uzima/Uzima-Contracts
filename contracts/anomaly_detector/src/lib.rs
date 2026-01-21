#![no_std]

use soroban_sdk::{contract, contractimpl};

#[contract]
pub struct AnomalyDetector;

#[contractimpl]
impl AnomalyDetector {
    pub fn hello(_env: soroban_sdk::Env) -> &'static str {
        "Anomaly Detector Placeholder"
    }
}
