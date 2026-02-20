#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Map, String, Vec, Symbol,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum AuditAction {
    RecordAccess,
    RecordUpdate,
    RecordDelete,
    PermissionGrant,
    PermissionRevoke,
    RecordCreated,
    AnomalyDetected,
    ComplianceReportGenerated,
    AlertTriggered,
}

#[derive(Clone)]
#[contracttype]
pub struct AuditEntry {
    pub id: u64,
    pub timestamp: u64,
    pub actor: Address,
    pub action: AuditAction,
    pub record_id: Option<u64>,
    pub details_hash: BytesN<32>, // Hash of detailed log data (stored off-chain)
    pub metadata: Map<String, String>,
}

#[derive(Clone)]
#[contracttype]
pub struct ForensicReport {
    pub target_id: u64, // record_id or user_id
    pub entries: Vec<AuditEntry>,
    pub summary: String,
    pub detected_anomalies: Vec<u64>, // anomaly IDs
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    NextAuditId,
    AuditEntry(u64),
    UserAudits(Address),  // user -> list of audit IDs
    RecordAudits(u64),    // record -> list of audit IDs
    AlertThresholds(Symbol),
}

#[contract]
pub struct AuditForensicsContract;

#[contractimpl]
impl AuditForensicsContract {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::NextAuditId, &0u64);
    }

    pub fn log_event(
        env: Env,
        actor: Address,
        action: AuditAction,
        record_id: Option<u64>,
        details_hash: BytesN<32>,
        metadata: Map<String, String>,
    ) -> u64 {
        actor.require_auth();
        
        let id = Self::get_next_id(&env);
        let entry = AuditEntry {
            id,
            timestamp: env.ledger().timestamp(),
            actor: actor.clone(),
            action,
            record_id,
            details_hash,
            metadata,
        };

        // Store global entry
        env.storage().persistent().set(&DataKey::AuditEntry(id), &entry);

        // Map to actor
        let mut user_audits: Vec<u64> = env.storage().persistent()
            .get(&DataKey::UserAudits(actor.clone()))
            .unwrap_or(Vec::new(&env));
        user_audits.push_back(id);
        env.storage().persistent().set(&DataKey::UserAudits(actor), &user_audits);

        // Map to record (if applicable)
        if let Some(rid) = record_id {
            let mut record_audits: Vec<u64> = env.storage().persistent()
                .get(&DataKey::RecordAudits(rid))
                .unwrap_or(Vec::new(&env));
            record_audits.push_back(id);
            env.storage().persistent().set(&DataKey::RecordAudits(rid), &record_audits);
        }

        env.storage().instance().set(&DataKey::NextAuditId, &(id + 1));
        
        // Emit event for real-time monitoring
        env.events().publish(
            (symbol_short!("AUDIT"), symbol_short!("LOG")),
            (id, entry.timestamp, entry.action),
        );

        id
    }

    pub fn analyze_timeline(env: Env, record_id: u64) -> Vec<AuditEntry> {
        let audit_ids: Vec<u64> = env.storage().persistent()
            .get(&DataKey::RecordAudits(record_id))
            .unwrap_or(Vec::new(&env));
        
        let mut result = Vec::new(&env);
        for id in audit_ids.iter() {
            if let Some(entry) = env.storage().persistent().get::<DataKey, AuditEntry>(&DataKey::AuditEntry(id)) {
                result.push_back(entry);
            }
        }
        result
    }

    pub fn investigate_user(env: Env, user: Address) -> Vec<AuditEntry> {
        let audit_ids: Vec<u64> = env.storage().persistent()
            .get(&DataKey::UserAudits(user))
            .unwrap_or(Vec::new(&env));
        
        let mut result = Vec::new(&env);
        for id in audit_ids.iter() {
            if let Some(entry) = env.storage().persistent().get::<DataKey, AuditEntry>(&DataKey::AuditEntry(id)) {
                result.push_back(entry);
            }
        }
        result
    }

    pub fn generate_compliance_report(env: Env, start_time: u64, end_time: u64) -> Map<AuditAction, u32> {
        let next_id = Self::get_next_id(&env);
        let mut report = Map::new(&env);
        
        for i in 0..next_id {
            if let Some(entry) = env.storage().persistent().get::<DataKey, AuditEntry>(&DataKey::AuditEntry(i)) {
                if entry.timestamp >= start_time && entry.timestamp <= end_time {
                    let count = report.get(entry.action).unwrap_or(0);
                    report.set(entry.action, count + 1);
                }
            }
        }
        
        // Log report generation
        Self::log_internal(&env, env.current_contract_address(), AuditAction::ComplianceReportGenerated, None);
        
        report
    }

    pub fn set_alert_threshold(env: Env, admin: Address, action: AuditAction, threshold: u32) {
        admin.require_auth();
        Self::require_admin(&env, &admin);
        
        let key = match action {
            AuditAction::RecordAccess => symbol_short!("THR_ACC"),
            AuditAction::RecordUpdate => symbol_short!("THR_UPD"),
            AuditAction::RecordDelete => symbol_short!("THR_DEL"),
            AuditAction::AnomalyDetected => symbol_short!("THR_ANOM"),
            _ => panic!("Unsupported action for alert"),
        };
        
        env.storage().instance().set(&DataKey::AlertThresholds(key), &threshold);
    }

    pub fn compress_logs(env: Env, admin: Address, before_timestamp: u64) -> BytesN<32> {
        admin.require_auth();
        Self::require_admin(&env, &admin);
        
        let next_id = Self::get_next_id(&env);
        let mut last_hash = BytesN::from_array(&env, &[0u8; 32]);
        let mut count = 0;
        
        for i in 0..next_id {
            if let Some(entry) = env.storage().persistent().get::<DataKey, AuditEntry>(&DataKey::AuditEntry(i)) {
                if entry.timestamp < before_timestamp {
                    // Update hash (simplified running hash: hash(last_hash + current_details_hash))
                    let mut combined = last_hash.to_array().to_vec();
                    combined.extend_from_slice(&entry.details_hash.to_array());
                    
                    let bytes = soroban_sdk::Bytes::from_slice(&env, &combined);
                    last_hash = env.crypto().sha256(&bytes).into();
                    
                    // Delete the entry to save space
                    env.storage().persistent().remove(&DataKey::AuditEntry(i));
                    count += 1;
                }
            }
        }
        
        // Log compression event
        Self::log_internal(&env, admin, AuditAction::AlertTriggered, None); 
        
        env.events().publish(
            (symbol_short!("AUDIT"), symbol_short!("COMPRESS")),
            (before_timestamp, count, last_hash.clone()),
        );
        
        last_hash
    }

    pub fn archive_logs(env: Env, admin: Address, archive_ref: String) {
        admin.require_auth();
        Self::require_admin(&env, &admin);
        
        // This function typically records that logs have been moved to off-chain storage (e.g. IPFS)
        env.events().publish(
            (symbol_short!("AUDIT"), symbol_short!("ARCHIVE")),
            archive_ref,
        );
    }

    pub fn sync_audit_cross_chain(env: Env, admin: Address, target_chain: String, audit_root: BytesN<32>) {
        admin.require_auth();
        Self::require_admin(&env, &admin);
        
        env.events().publish(
            (symbol_short!("AUDIT"), symbol_short!("XCSYNC")),
            (target_chain, audit_root),
        );
    }

    pub fn share_audit_with_regulator(
        env: Env,
        admin: Address,
        regulator: Address,
        filter_start: u64,
        filter_end: u64,
        proof_ref: String, // Reference to a ZK-proof or redacted log set
    ) {
        admin.require_auth();
        Self::require_admin(&env, &admin);
        
        env.events().publish(
            (symbol_short!("AUDIT"), symbol_short!("SHARE")),
            (regulator, filter_start, filter_end, proof_ref),
        );
        
        Self::log_internal(&env, admin, AuditAction::AlertTriggered, None); // Log sharing action
    }

    fn check_alerts(env: &Env, action: AuditAction) {
        let key = match action {
            AuditAction::RecordAccess => Some(symbol_short!("THR_ACC")),
            AuditAction::RecordUpdate => Some(symbol_short!("THR_UPD")),
            AuditAction::RecordDelete => Some(symbol_short!("THR_DEL")),
            AuditAction::AnomalyDetected => Some(symbol_short!("THR_ANOM")),
            _ => None,
        };

        if let Some(k) = key {
            if let Some(threshold) = env.storage().instance().get::<DataKey, u32>(&DataKey::AlertThresholds(k.clone())) {
                // Simplified: check if count in last 1 hour exceeds threshold
                // In a production contract, we'd use a rolling window or bucketed counts
                let now = env.ledger().timestamp();
                let hour_ago = now.saturating_sub(3600);
                
                let mut count = 0;
                let next_id = Self::get_next_id(env);
                // Search backwards for efficiency
                for i in (0..next_id).rev() {
                    if let Some(entry) = env.storage().persistent().get::<DataKey, AuditEntry>(&DataKey::AuditEntry(i)) {
                        if entry.timestamp < hour_ago { break; }
                        if entry.action == action {
                            count += 1;
                        }
                    }
                    if count >= threshold {
                        env.events().publish(
                            (symbol_short!("ALERT"), k),
                            (action, count),
                        );
                        break;
                    }
                }
            }
        }
    }

    fn require_admin(env: &Env, actor: &Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != *actor {
            panic!("Not authorized");
        }
    }

    fn log_internal(env: &Env, actor: Address, action: AuditAction, record_id: Option<u64>) {
        // Internal logging without auth requirements for system events
        let id = Self::get_next_id(env);
        let entry = AuditEntry {
            id,
            timestamp: env.ledger().timestamp(),
            actor,
            action,
            record_id,
            details_hash: BytesN::from_array(env, &[0u8; 32]),
            metadata: Map::new(env),
        };
        env.storage().persistent().set(&DataKey::AuditEntry(id), &entry);
        env.storage().instance().set(&DataKey::NextAuditId, &(id + 1));
    }

    fn get_next_id(env: &Env) -> u64 {
        env.storage().instance().get(&DataKey::NextAuditId).unwrap_or(0)
    }
}
