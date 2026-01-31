#![no_std]
#[contract]
pub struct AnomalyDetector;

#[contractimpl]
impl AnomalyDetector {
    pub fn hello(env: Env) -> String {
        String::from_str(&env, "Anomaly Detector Placeholder")
    }
}
