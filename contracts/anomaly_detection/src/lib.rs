#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct AnomalyDetectionContract;

#[contractimpl]
impl AnomalyDetectionContract {
    pub fn init(_env: Env) {}
}