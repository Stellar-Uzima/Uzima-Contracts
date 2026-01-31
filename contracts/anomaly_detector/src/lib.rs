#![no_std]

use soroban_sdk::{contract, contractimpl, Env, String};

#[contract]
pub struct AnomalyDetector;

#[contractimpl]
impl AnomalyDetector {
    pub fn hello(env: Env) -> String {
        String::from_str(&env, "Anomaly Detector Placeholder")
    }
}
