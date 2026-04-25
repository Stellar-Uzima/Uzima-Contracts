use crate::types::{DataKey, RecoveryVault};
use soroban_sdk::{Address, BytesN, Env};

pub struct RecoveryMechanism;

impl RecoveryMechanism {
    /// Commits a new recovery strategy for the user.
    pub fn setup_recovery(
        env: &Env,
        user: &Address,
        recovery_hashes: soroban_sdk::Vec<BytesN<32>>,
    ) {
        let vault = RecoveryVault {
            user: user.clone(),
            recovery_hashes,
            backup_address: None,
            unlock_at: 0,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Recovery(user.clone()), &vault);
    }

    /// Triggers factor recovery after time-lock expires.
    pub fn complete_recovery(env: &Env, user: &Address, code_hash: BytesN<32>) -> bool {
        let mut vault: RecoveryVault = env
            .storage()
            .persistent()
            .get(&DataKey::Recovery(user.clone()))
            .expect("Recovery not initiated");

        let now = env.ledger().timestamp();
        if vault.unlock_at > 0 && now >= vault.unlock_at {
            // Verify if the hash exists in recovery_hashes
            for h in vault.recovery_hashes.iter() {
                if h == code_hash {
                    return true;
                }
            }
        }
        false
    }
}
