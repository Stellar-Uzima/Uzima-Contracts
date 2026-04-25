use crate::types::{AuthFactor, DataKey};
use soroban_sdk::{Address, Env, Vec};

pub struct FactorManager;

impl FactorManager {
    /// Retrieve all active factors for a specific user
    pub fn get_active_factors(env: &Env, user: &Address) -> Vec<AuthFactor> {
        let factors: Vec<AuthFactor> = env
            .storage()
            .persistent()
            .get(&DataKey::UserFactors(user.clone()))
            .unwrap_or(Vec::new(env));

        let mut active = Vec::new(env);
        for f in factors.iter() {
            if f.is_active {
                active.push_back(f);
            }
        }
        active
    }

    /// Deactivate a factor identified by factor_id
    pub fn deactivate_factor(env: &Env, user: &Address, factor_id: u64) -> bool {
        let mut factors: Vec<AuthFactor> = env
            .storage()
            .persistent()
            .get(&DataKey::UserFactors(user.clone()))
            .unwrap_or(Vec::new(env));

        for i in 0..factors.len() {
            if let Some(mut f) = factors.get(i) {
                if f.factor_id == factor_id {
                    f.is_active = false;
                    factors.set(i, f);
                    env.storage()
                        .persistent()
                        .set(&DataKey::UserFactors(user.clone()), &factors);
                    return true;
                }
            }
        }
        false
    }
}
