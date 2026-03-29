#![no_std]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Map, String, Symbol,
    Vec,
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
    pub details_hash: BytesN<32>,
    pub metadata: Map<String, String>,
}

#[derive(Clone)]
#[contracttype]
pub struct ForensicReport {
    pub target_id: u64,
    pub entries: Vec<AuditEntry>,
    pub summary: String,
    pub detected_anomalies: Vec<u64>,
}

#[derive(Clone)]
#[contracttype]
pub struct AuditRule {
    pub id: u64,
    pub name: String,
    pub applies_to_language: String,
    pub severity_bps: u32,
    pub enabled: bool,
    pub pattern_ref: String,
    pub remediation: String,
}

#[derive(Clone)]
#[contracttype]
pub struct VulnerabilityFinding {
    pub id: u64,
    pub execution_id: u64,
    pub rule_id: u64,
    pub contract_hash: BytesN<32>,
    pub title: String,
    pub severity_bps: u32,
    pub confidence_bps: u32,
    pub language: String,
    pub analysis_mode: String,
    pub remediation: String,
    pub detected_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct AnalysisExecution {
    pub id: u64,
    pub contract_hash: BytesN<32>,
    pub language: String,
    pub analysis_mode: String,
    pub finding_ids: Vec<u64>,
    pub started_at: u64,
    pub completed_at: u64,
    pub duration_minutes: u32,
    pub passed: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct FormalVerificationSummary {
    pub execution_id: u64,
    pub property_name: String,
    pub proved: bool,
    pub proof_ref: String,
    pub checked_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    NextAuditId,
    AuditEntry(u64),
    UserAudits(Address),
    RecordAudits(u64),
    AlertThresholds(Symbol),
    NextRuleId,
    Rule(u64),
    NextExecutionId,
    Execution(u64),
    NextFindingId,
    Finding(u64),
    FindingsByExecution(u64),
    FormalSummary(u64),
}

#[contract]
pub struct AuditForensicsContract;

#[contractimpl]
impl AuditForensicsContract {
    #[allow(clippy::panic)]
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::NextAuditId, &0u64);
        env.storage().instance().set(&DataKey::NextRuleId, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::NextExecutionId, &0u64);
        env.storage().instance().set(&DataKey::NextFindingId, &0u64);
    }

    pub fn configure_audit_rule(
        env: Env,
        admin: Address,
        name: String,
        applies_to_language: String,
        severity_bps: u32,
        pattern_ref: String,
        remediation: String,
    ) -> u64 {
        admin.require_auth();
        Self::require_admin(&env, &admin);

        let rule_id = Self::next_counter(&env, &DataKey::NextRuleId);
        let rule = AuditRule {
            id: rule_id,
            name,
            applies_to_language,
            severity_bps: severity_bps.min(10_000),
            enabled: true,
            pattern_ref,
            remediation,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Rule(rule_id), &rule);
        env.events()
            .publish((symbol_short!("AUDIT"), symbol_short!("RULE")), rule_id);

        rule_id
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

        env.storage()
            .persistent()
            .set(&DataKey::AuditEntry(id), &entry);

        let mut user_audits: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::UserAudits(actor.clone()))
            .unwrap_or(Vec::new(&env));
        user_audits.push_back(id);
        env.storage()
            .persistent()
            .set(&DataKey::UserAudits(actor), &user_audits);

        if let Some(rid) = record_id {
            let mut record_audits: Vec<u64> = env
                .storage()
                .persistent()
                .get(&DataKey::RecordAudits(rid))
                .unwrap_or(Vec::new(&env));
            record_audits.push_back(id);
            env.storage()
                .persistent()
                .set(&DataKey::RecordAudits(rid), &record_audits);
        }

        env.storage()
            .instance()
            .set(&DataKey::NextAuditId, &id.saturating_add(1));

        env.events().publish(
            (symbol_short!("AUDIT"), symbol_short!("LOG")),
            (id, entry.timestamp, entry.action),
        );

        id
    }

    pub fn run_automated_audit(
        env: Env,
        caller: Address,
        contract_hash: BytesN<32>,
        language: String,
        analysis_mode: String,
        rule_ids: Vec<u64>,
        ml_confidence_bps: u32,
    ) -> u64 {
        caller.require_auth();
        Self::require_admin(&env, &caller);

        let execution_id = Self::next_counter(&env, &DataKey::NextExecutionId);
        let started_at = env.ledger().timestamp();
        let mut finding_ids = Vec::new(&env);
        let mut passed = true;

        for rule_id in rule_ids.iter() {
            let Some(rule): Option<AuditRule> =
                env.storage().persistent().get(&DataKey::Rule(rule_id))
            else {
                continue;
            };
            if !rule.enabled || rule.applies_to_language != language {
                continue;
            }

            let finding_id = Self::next_counter(&env, &DataKey::NextFindingId);
            let finding = VulnerabilityFinding {
                id: finding_id,
                execution_id,
                rule_id,
                contract_hash: contract_hash.clone(),
                title: rule.name.clone(),
                severity_bps: rule.severity_bps,
                confidence_bps: ml_confidence_bps.min(10_000),
                language: language.clone(),
                analysis_mode: analysis_mode.clone(),
                remediation: rule.remediation.clone(),
                detected_at: started_at,
            };

            env.storage()
                .persistent()
                .set(&DataKey::Finding(finding_id), &finding);
            finding_ids.push_back(finding_id);
            if finding.severity_bps >= 5_000 {
                passed = false;
            }
        }

        let duration_minutes = finding_ids.len().saturating_mul(10).saturating_add(15);
        let execution = AnalysisExecution {
            id: execution_id,
            contract_hash,
            language: language.clone(),
            analysis_mode,
            finding_ids: finding_ids.clone(),
            started_at,
            completed_at: started_at.saturating_add(60),
            duration_minutes,
            passed,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Execution(execution_id), &execution);
        env.storage()
            .persistent()
            .set(&DataKey::FindingsByExecution(execution_id), &finding_ids);

        Self::log_internal(&env, caller, AuditAction::AnomalyDetected, None);
        env.events().publish(
            (symbol_short!("AUDIT"), symbol_short!("RUN")),
            (execution_id, execution.duration_minutes, execution.passed),
        );

        execution_id
    }

    pub fn record_formal_verification(
        env: Env,
        admin: Address,
        execution_id: u64,
        property_name: String,
        proved: bool,
        proof_ref: String,
    ) -> bool {
        admin.require_auth();
        Self::require_admin(&env, &admin);

        let summary = FormalVerificationSummary {
            execution_id,
            property_name,
            proved,
            proof_ref,
            checked_at: env.ledger().timestamp(),
        };
        env.storage()
            .persistent()
            .set(&DataKey::FormalSummary(execution_id), &summary);

        if !proved {
            Self::log_internal(&env, admin, AuditAction::AlertTriggered, None);
        }

        true
    }

    pub fn get_execution(env: Env, execution_id: u64) -> Option<AnalysisExecution> {
        env.storage()
            .persistent()
            .get(&DataKey::Execution(execution_id))
    }

    pub fn get_finding(env: Env, finding_id: u64) -> Option<VulnerabilityFinding> {
        env.storage()
            .persistent()
            .get(&DataKey::Finding(finding_id))
    }

    pub fn get_findings_by_execution(env: Env, execution_id: u64) -> Vec<VulnerabilityFinding> {
        let finding_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::FindingsByExecution(execution_id))
            .unwrap_or(Vec::new(&env));
        let mut findings = Vec::new(&env);
        for finding_id in finding_ids.iter() {
            if let Some(finding) = env
                .storage()
                .persistent()
                .get::<DataKey, VulnerabilityFinding>(&DataKey::Finding(finding_id))
            {
                findings.push_back(finding);
            }
        }
        findings
    }

    pub fn get_formal_verification(
        env: Env,
        execution_id: u64,
    ) -> Option<FormalVerificationSummary> {
        env.storage()
            .persistent()
            .get(&DataKey::FormalSummary(execution_id))
    }

    pub fn generate_remediation_plan(env: Env, execution_id: u64) -> Vec<String> {
        let findings = Self::get_findings_by_execution(env.clone(), execution_id);
        let mut remediation = Vec::new(&env);
        for finding in findings.iter() {
            remediation.push_back(finding.remediation);
        }
        remediation
    }

    pub fn analyze_timeline(env: Env, record_id: u64) -> Vec<AuditEntry> {
        let audit_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::RecordAudits(record_id))
            .unwrap_or(Vec::new(&env));

        let mut result = Vec::new(&env);
        for id in audit_ids.iter() {
            if let Some(entry) = env
                .storage()
                .persistent()
                .get::<DataKey, AuditEntry>(&DataKey::AuditEntry(id))
            {
                result.push_back(entry);
            }
        }
        result
    }

    pub fn investigate_user(env: Env, user: Address) -> Vec<AuditEntry> {
        let audit_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::UserAudits(user))
            .unwrap_or(Vec::new(&env));

        let mut result = Vec::new(&env);
        for id in audit_ids.iter() {
            if let Some(entry) = env
                .storage()
                .persistent()
                .get::<DataKey, AuditEntry>(&DataKey::AuditEntry(id))
            {
                result.push_back(entry);
            }
        }
        result
    }

    pub fn generate_compliance_report(
        env: Env,
        start_time: u64,
        end_time: u64,
    ) -> Map<AuditAction, u32> {
        let next_id = Self::get_next_id(&env);
        let mut report = Map::new(&env);

        for i in 0..next_id {
            if let Some(entry) = env
                .storage()
                .persistent()
                .get::<DataKey, AuditEntry>(&DataKey::AuditEntry(i))
            {
                if entry.timestamp >= start_time && entry.timestamp <= end_time {
                    let count: u32 = report.get(entry.action).unwrap_or(0);
                    report.set(entry.action, count.saturating_add(1));
                }
            }
        }

        Self::log_internal(
            &env,
            env.current_contract_address(),
            AuditAction::ComplianceReportGenerated,
            None,
        );

        report
    }

    #[allow(clippy::panic)]
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

        env.storage()
            .instance()
            .set(&DataKey::AlertThresholds(key), &threshold);
    }

    pub fn compress_logs(env: Env, admin: Address, before_timestamp: u64) -> BytesN<32> {
        admin.require_auth();
        Self::require_admin(&env, &admin);

        let next_id = Self::get_next_id(&env);
        let mut last_hash = BytesN::from_array(&env, &[0u8; 32]);
        let mut count: u64 = 0;

        for i in 0..next_id {
            if let Some(entry) = env
                .storage()
                .persistent()
                .get::<DataKey, AuditEntry>(&DataKey::AuditEntry(i))
            {
                if entry.timestamp < before_timestamp {
                    let mut combined = [0u8; 64];
                    combined[..32].copy_from_slice(&last_hash.to_array());
                    combined[32..].copy_from_slice(&entry.details_hash.to_array());

                    let bytes = soroban_sdk::Bytes::from_slice(&env, &combined);
                    last_hash = env.crypto().sha256(&bytes).into();

                    env.storage().persistent().remove(&DataKey::AuditEntry(i));
                    count = count.saturating_add(1);
                }
            }
        }

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

        env.events().publish(
            (symbol_short!("AUDIT"), symbol_short!("ARCHIVE")),
            archive_ref,
        );
    }

    pub fn sync_audit_cross_chain(
        env: Env,
        admin: Address,
        target_chain: String,
        audit_root: BytesN<32>,
    ) {
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
        proof_ref: String,
    ) {
        admin.require_auth();
        Self::require_admin(&env, &admin);

        env.events().publish(
            (symbol_short!("AUDIT"), symbol_short!("SHARE")),
            (regulator, filter_start, filter_end, proof_ref),
        );

        Self::log_internal(&env, admin, AuditAction::AlertTriggered, None);
    }

    #[allow(dead_code)]
    fn check_alerts(env: &Env, action: AuditAction) {
        let key = match action {
            AuditAction::RecordAccess => Some(symbol_short!("THR_ACC")),
            AuditAction::RecordUpdate => Some(symbol_short!("THR_UPD")),
            AuditAction::RecordDelete => Some(symbol_short!("THR_DEL")),
            AuditAction::AnomalyDetected => Some(symbol_short!("THR_ANOM")),
            _ => None,
        };

        if let Some(k) = key {
            if let Some(threshold) = env
                .storage()
                .instance()
                .get::<DataKey, u32>(&DataKey::AlertThresholds(k.clone()))
            {
                let now = env.ledger().timestamp();
                let hour_ago = now.saturating_sub(3600);

                let mut count: u32 = 0;
                let next_id = Self::get_next_id(env);
                for i in (0..next_id).rev() {
                    if let Some(entry) = env
                        .storage()
                        .persistent()
                        .get::<DataKey, AuditEntry>(&DataKey::AuditEntry(i))
                    {
                        if entry.timestamp < hour_ago {
                            break;
                        }
                        if entry.action == action {
                            count = count.saturating_add(1);
                        }
                    }
                    if count >= threshold {
                        env.events()
                            .publish((symbol_short!("ALERT"), k), (action, count));
                        break;
                    }
                }
            }
        }
    }

    #[allow(clippy::panic, clippy::unwrap_used)]
    fn require_admin(env: &Env, actor: &Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != *actor {
            panic!("Not authorized");
        }
    }

    fn log_internal(env: &Env, actor: Address, action: AuditAction, record_id: Option<u64>) {
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
        env.storage()
            .persistent()
            .set(&DataKey::AuditEntry(id), &entry);
        env.storage()
            .instance()
            .set(&DataKey::NextAuditId, &id.saturating_add(1));
    }

    fn get_next_id(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::NextAuditId)
            .unwrap_or(0)
    }

    fn next_counter(env: &Env, key: &DataKey) -> u64 {
        let current: u64 = env.storage().instance().get(key).unwrap_or(0);
        let next = current.saturating_add(1);
        env.storage().instance().set(key, &next);
        next
    }
}
