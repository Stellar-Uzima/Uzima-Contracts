use crate::types::{ForensicEvidence, ThreatLevel};
use soroban_sdk::{Address, Bytes, BytesN, Env};

pub struct ForensicsLib;

impl ForensicsLib {
    /// Hashes specific evidence components into a unique identifier
    pub fn compute_evidence_hash(
        env: &Env,
        actor: Address,
        location: BytesN<32>,
        data: Bytes,
    ) -> BytesN<32> {
        let mut combined = [0u8; 96];
        // Actor's address is 32 bytes usually in Stellar?
        // Let's use crypto hashing.
        let mut buffer = soroban_sdk::Bytes::new(env);
        buffer.append(&actor.to_xdr(env));
        buffer.append(&location.to_xdr(env));
        buffer.append(&data);

        env.crypto().sha256(&buffer).into()
    }

    /// Evaluates if threat level requires immediate action/escalation
    pub fn is_critical_threat(threat: ThreatLevel) -> bool {
        match threat {
            ThreatLevel::Critical | ThreatLevel::High => true,
            _ => false,
        }
    }
}
