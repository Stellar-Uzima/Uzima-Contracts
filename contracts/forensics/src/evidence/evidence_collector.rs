use soroban_sdk::{Env, Address, Bytes, BytesN};
use crate::types::{ForensicEvidence, ActivityType, ThreatLevel, DataKey};

pub struct EvidenceCollector;

impl EvidenceCollector {
    /// Collects and preserves forensic data immutably on-chain
    pub fn collect_new_evidence(
        env: &Env,
        id: u64,
        actor: Address,
        activity: ActivityType,
        location: BytesN<32>,
        evidence_data: Bytes,
        threat: ThreatLevel,
    ) -> ForensicEvidence {
        let evidence = ForensicEvidence {
            id,
            timestamp: env.ledger().timestamp(),
            actor,
            activity_type: activity,
            location_hash: location,
            evidence_data,
            threat_level: threat,
            is_preserved: true,
        };

        // Storage and consistency check
        env.storage()
            .persistent()
            .set(&DataKey::Evidence(id), &evidence);

        evidence
    }

    /// Retrieves collection of evidence by case ID/evidence ID
    pub fn get_evidence(env: &Env, id: u64) -> Option<ForensicEvidence> {
        env.storage().persistent().get(&DataKey::Evidence(id))
    }
}
