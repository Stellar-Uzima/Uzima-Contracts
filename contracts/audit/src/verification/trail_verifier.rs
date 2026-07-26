use crate::types::{AuditLog, DataKey};
use soroban_sdk::{xdr::ToXdr, BytesN, Env};

pub struct TrailVerifier;

impl TrailVerifier {
    /// Recomputes the rolling hash over all AuditLog entries and returns it.
    ///
    /// Uses the per-entry `prev_hash` field (see Issue #999) for a consistent
    /// verification path with `verify_chain`. Each entry's `prev_hash` should
    /// equal the hash of the previous entry's data.
    ///
    /// Compare against the stored `RollingHash` to detect tampering.
    pub fn verify_log_integrity(env: &Env) -> BytesN<32> {
        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::LogCount)
            .unwrap_or(0u64);
        let mut rolling = BytesN::from_array(env, &[0u8; 32]);

        for i in 1..=count {
            if let Some(log) = env
                .storage()
                .persistent()
                .get::<DataKey, AuditLog>(&DataKey::Log(i))
            {
                // Verify the stored prev_hash matches our computed expectation
                if log.prev_hash != rolling {
                    // Chain is broken at this entry
                    return rolling;
                }

                // Compute the next expected hash from current log data
                let mut buffer = soroban_sdk::Bytes::new(env);
                buffer.append(&rolling.to_xdr(env));
                buffer.append(&log.id.to_xdr(env));
                buffer.append(&log.timestamp.to_xdr(env));
                let action_disc = log.action as u32;
                buffer.append(&action_disc.to_xdr(env));
                buffer.append(&log.target.clone().to_xdr(env));
                rolling = env.crypto().sha256(&buffer).into();
            }
        }
        rolling
    }

    /// Returns true if the AuditLog chain has been tampered with.
    pub fn is_log_chain_tampered(env: &Env, expected: BytesN<32>) -> bool {
        Self::verify_log_integrity(env) != expected
    }
}
