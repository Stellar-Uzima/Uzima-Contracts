use crate::types::{AuditRecord, DataKey};
use soroban_sdk::{xdr::ToXdr, Address, BytesN, Env};

pub struct TrailVerifier;

impl TrailVerifier {
    /// Recalculates the cumulative Hash to verify entire audit trail integrity.
    pub fn verify_full_integrity(env: &Env) -> BytesN<32> {
        let count = env
            .storage()
            .instance()
            .get(&DataKey::RecordCount)
            .unwrap_or(0u64);
        let mut rolling = BytesN::from_array(env, &[0u8; 32]);

        for i in 0..count {
            if let Some(record) = env
                .storage()
                .persistent()
                .get::<DataKey, AuditRecord>(&DataKey::Record(i))
            {
                let mut buffer = soroban_sdk::Bytes::new(env);
                buffer.append(&rolling.to_xdr(env));
                buffer.append(&record.id.to_xdr(env));
                buffer.append(&record.action_hash.to_xdr(env));
                rolling = env.crypto().sha256(&buffer).into();
            }
        }
        rolling
    }

    /// Verifies if a specific record's rolling hash matches the stored value.
    pub fn is_audit_tampered(env: &Env, stored_rolling: BytesN<32>) -> bool {
        let current_rolling: BytesN<32> = env
            .storage()
            .instance()
            .get(&DataKey::RollingHash)
            .unwrap_or(BytesN::from_array(env, &[0u8; 32]));
        current_rolling != stored_rolling
    }
}
