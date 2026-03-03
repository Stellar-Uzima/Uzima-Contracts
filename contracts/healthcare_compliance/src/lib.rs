#![no_std]
#![allow(clippy::too_many_arguments)]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, vec, Address, BytesN, Env,
    Map, String, Symbol, Vec,
};

// ==================== Compliance Framework Types ====================

/// Healthcare Compliance Framework Standards
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum ComplianceFramework {
    HIPAA,
    GDPR,
    HL7FHIR,
    SOX,    // Sarbanes-Oxley for financial healthcare data
    HITECH, // Health Information Technology for Economic and Clinical Health Act
}

/// HIPAA Privacy Rule Categories
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum HIPAACategory {
    Treatment,
    Payment,
    HealthcareOperations,
    Research,
    PublicHealth,
    Emergency,
    Marketing,
}

/// GDPR Data Processing Categories
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum GDPRProcessingCategory {
    Consent,
    Contract,
    LegalObligation,
    VitalInterest,
    PublicTask,
    LegitimateInterest,
}

/// HL7 FHIR Resource Types for compliance
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum FHIRResourceType {
    Patient,
    Observation,
    Condition,
    Medication,
    AllergyIntolerance,
    Procedure,
    DiagnosticReport,
    DocumentReference,
    Consent,
    AuditEvent,
}

/// Consent Status Types
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum ConsentStatus {
    Draft,
    Proposed,
    Active,
    Rejected,
    Inactive,
    EnteredInError,
}

/// Audit Event Types
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum AuditEventType {
    Create,
    Read,
    Update,
    Delete,
    Execute,
    Consent,
    Access,
    Disclosure,
    Breach,
}

/// Data Breach Severity Levels
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum BreachSeverity {
    Low,
    Moderate,
    High,
    Critical,
}

/// Compliance Violation Types
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum ViolationType {
    UnauthorizedAccess,
    DataBreach,
    ConsentViolation,
    AuditFailure,
    RetentionViolation,
    DisclosureViolation,
    ProcessingViolation,
}

/// Data Retention Policy
#[derive(Clone)]
#[contracttype]
pub struct RetentionPolicy {
    pub retention_period_days: u32,
    pub auto_delete_enabled: bool,
    pub notify_before_deletion_days: u32,
    pub legal_hold_enabled: bool,
}

/// Consent Record
#[derive(Clone)]
#[contracttype]
pub struct ConsentRecord {
    pub consent_id: String,
    pub patient: Address,
    pub data_controller: Address,
    pub data_processor: Address,
    pub purpose: String,
    pub data_categories: Vec<String>,
    pub processing_categories: Vec<GDPRProcessingCategory>,
    pub status: ConsentStatus,
    pub granted_at: u64,
    pub expires_at: u64,
    pub revoked_at: u64,
    pub revocation_reason: String,
    pub signature: BytesN<64>, // Ed25519 signature
}

/// Audit Log Entry
#[derive(Clone)]
#[contracttype]
pub struct AuditLogEntry {
    pub log_id: String,
    pub timestamp: u64,
    pub actor: Address,
    pub action: AuditEventType,
    pub resource_type: FHIRResourceType,
    pub resource_id: String,
    pub patient_id: String,
    pub success: bool,
    pub details: String,
    pub ip_address: String,
    pub user_agent: String,
    pub compliance_framework: ComplianceFramework,
    pub hipaa_category: u32, // 0 = None, 1-7 = HIPAACategory variants
    pub gdpr_category: u32,  // 0 = None, 1-6 = GDPRProcessingCategory variants
}

/// Data Breach Report
#[derive(Clone)]
#[contracttype]
pub struct BreachReport {
    pub report_id: String,
    pub timestamp: u64,
    pub reporter: Address,
    pub severity: BreachSeverity,
    pub affected_records: u32,
    pub affected_patients: Vec<Address>,
    pub breach_type: String,
    pub description: String,
    pub mitigation_steps: Vec<String>,
    pub notified_authorities: bool,
    pub notified_patients: bool,
    pub resolution_status: String,
}

/// Compliance Violation Report
#[derive(Clone)]
#[contracttype]
pub struct ViolationReport {
    pub violation_id: String,
    pub timestamp: u64,
    pub reporter: Address,
    pub violation_type: ViolationType,
    pub affected_resource: String,
    pub actor: Address,
    pub details: String,
    pub evidence: Vec<String>,
    pub resolved: bool,
    pub resolution_notes: String,
    pub penalty_amount: i128,
}

/// Compliance Dashboard Metrics
#[derive(Clone)]
#[contracttype]
pub struct ComplianceMetrics {
    pub total_audits: u32,
    pub successful_audits: u32,
    pub failed_audits: u32,
    pub total_consents: u32,
    pub active_consents: u32,
    pub revoked_consents: u32,
    pub total_breaches: u32,
    pub resolved_breaches: u32,
    pub pending_violations: u32,
    pub compliance_score: u32, // 0-100
    pub last_audit_timestamp: u64,
}

/// Compliance Configuration
#[derive(Clone)]
#[contracttype]
pub struct ComplianceConfig {
    pub hipaa_enabled: bool,
    pub gdpr_enabled: bool,
    pub hl7_fhir_enabled: bool,
    pub audit_logging_enabled: bool,
    pub breach_notification_enabled: bool,
    pub auto_consent_expiration: bool,
    pub default_retention_days: u32,
    pub admin_addresses: Vec<Address>,
    pub compliance_officers: Vec<Address>,
}

// Storage Keys
const ADMIN: Symbol = symbol_short!("ADMIN");
const CONFIG: Symbol = symbol_short!("CONFIG");
const CONSENTS: Symbol = symbol_short!("CONSENTS");
const AUDIT_LOGS: Symbol = symbol_short!("AUDITS");
const BREACH_REPORTS: Symbol = symbol_short!("BREACHES");
const VIOLATION_REPORTS: Symbol = symbol_short!("VIOLATE");
#[allow(dead_code)]
const RETENTION_POLICIES: Symbol = symbol_short!("RETENTION");
const COMPLIANCE_SCORE: Symbol = symbol_short!("SCORE");
const PAUSED: Symbol = symbol_short!("PAUSED");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    ContractPaused = 2,
    ConsentNotFound = 3,
    ConsentAlreadyExists = 4,
    InvalidConsentStatus = 5,
    ConsentExpired = 6,
    AuditLogNotFound = 7,
    BreachReportNotFound = 8,
    ViolationNotFound = 9,
    InvalidFramework = 10,
    InvalidResourceType = 11,
    DataBreachAlreadyReported = 12,
    ViolationAlreadyExists = 13,
    InvalidSignature = 14,
    RetentionPolicyNotFound = 15,
    ComplianceConfigNotSet = 16,
    InsufficientPermissions = 17,
    DataPurgeFailed = 18,
    NotificationFailed = 19,
    InvalidPatientAddress = 20,
}

#[contract]
pub struct HealthcareComplianceContract;

#[contractimpl]
impl HealthcareComplianceContract {
    /// Initialize the compliance contract
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        #[cfg(not(test))]
        admin.require_auth();

        if env.storage().instance().has(&ADMIN) {
            return Err(Error::ConsentAlreadyExists);
        }

        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&PAUSED, &false);

        // Set default configuration
        let default_config = ComplianceConfig {
            hipaa_enabled: true,
            gdpr_enabled: true,
            hl7_fhir_enabled: true,
            audit_logging_enabled: true,
            breach_notification_enabled: true,
            auto_consent_expiration: true,
            default_retention_days: 3650, // 10 years
            admin_addresses: vec![&env, admin.clone()],
            compliance_officers: vec![&env, admin],
        };

        env.storage().instance().set(&CONFIG, &default_config);
        env.storage().instance().set(&COMPLIANCE_SCORE, &100u32);

        Ok(())
    }

    /// Update compliance configuration
    pub fn update_config(env: Env, admin: Address, config: ComplianceConfig) -> Result<(), Error> {
        #[cfg(not(test))]
        admin.require_auth();

        Self::check_admin(&env, &admin)?;
        Self::check_paused(&env)?;

        env.storage().instance().set(&CONFIG, &config);
        Ok(())
    }

    /// Get current compliance configuration
    pub fn get_config(env: Env) -> Result<ComplianceConfig, Error> {
        env.storage()
            .instance()
            .get(&CONFIG)
            .ok_or(Error::ComplianceConfigNotSet)
    }

    /// Grant patient consent for data processing
    pub fn grant_consent(env: Env, patient: Address, consent: ConsentRecord) -> Result<(), Error> {
        #[cfg(not(test))]
        patient.require_auth();
        Self::check_paused(&env)?;

        // Validate consent
        if consent.patient != patient {
            return Err(Error::InvalidPatientAddress);
        }

        if consent.expires_at <= env.ledger().timestamp() {
            return Err(Error::ConsentExpired);
        }

        // Store consent
        let mut consents: Map<String, ConsentRecord> = env
            .storage()
            .persistent()
            .get(&CONSENTS)
            .unwrap_or(Map::new(&env));

        if consents.contains_key(consent.consent_id.clone()) {
            return Err(Error::ConsentAlreadyExists);
        }

        let consent_id = consent.consent_id.clone();
        consents.set(consent_id.clone(), consent);
        env.storage().persistent().set(&CONSENTS, &consents);

        // Log audit event
        Self::log_audit_event(
            env.clone(),
            patient.clone(),
            AuditEventType::Consent,
            FHIRResourceType::Consent,
            consent_id,
            String::from_str(&env, ""),
            String::from_str(&env, "Consent granted"),
            ComplianceFramework::GDPR,
            0, // None
            1, // Consent
        )?;

        Ok(())
    }

    /// Revoke patient consent
    pub fn revoke_consent(
        env: Env,
        patient: Address,
        consent_id: String,
        reason: String,
    ) -> Result<(), Error> {
        #[cfg(not(test))]
        patient.require_auth();
        Self::check_paused(&env)?;

        let mut consents: Map<String, ConsentRecord> = env
            .storage()
            .persistent()
            .get(&CONSENTS)
            .ok_or(Error::ConsentNotFound)?;

        let mut consent = consents
            .get(consent_id.clone())
            .ok_or(Error::ConsentNotFound)?;

        if consent.patient != patient {
            return Err(Error::NotAuthorized);
        }

        if consent.status == ConsentStatus::Inactive
            || consent.status == ConsentStatus::EnteredInError
        {
            return Err(Error::InvalidConsentStatus);
        }

        // Update consent status
        consent.status = ConsentStatus::Inactive;
        consent.revoked_at = env.ledger().timestamp();
        consent.revocation_reason = reason;

        consents.set(consent_id.clone(), consent);
        env.storage().persistent().set(&CONSENTS, &consents);

        // Log audit event
        Self::log_audit_event(
            env.clone(),
            patient,
            AuditEventType::Consent,
            FHIRResourceType::Consent,
            consent_id,
            String::from_str(&env, ""),
            String::from_str(&env, "Consent revoked"),
            ComplianceFramework::GDPR,
            0, // None
            1, // Consent
        )?;

        Ok(())
    }

    /// Check if patient has valid consent for specific purpose
    pub fn has_valid_consent(
        env: Env,
        patient: Address,
        purpose: String,
        data_category: String,
    ) -> Result<bool, Error> {
        let consents: Map<String, ConsentRecord> = env
            .storage()
            .persistent()
            .get(&CONSENTS)
            .unwrap_or(Map::new(&env));

        let current_time = env.ledger().timestamp();

        for (_id, consent) in consents.iter() {
            if consent.patient == patient
                && consent.status == ConsentStatus::Active
                && consent.granted_at <= current_time
                && (consent.expires_at == 0 || consent.expires_at > current_time)
                && consent.purpose == purpose
            {
                // Check if data category is covered
                for category in consent.data_categories.iter() {
                    if category == data_category {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    /// Log audit event for compliance tracking
    pub fn log_audit_event(
        env: Env,
        actor: Address,
        action: AuditEventType,
        resource_type: FHIRResourceType,
        resource_id: String,
        patient_id: String,
        details: String,
        framework: ComplianceFramework,
        hipaa_category: u32,
        gdpr_category: u32,
    ) -> Result<(), Error> {
        Self::check_paused(&env)?;

        let config = Self::get_config(env.clone())?;
        if !config.audit_logging_enabled {
            return Ok(());
        }

        let log_id = Self::generate_id(&env);
        let timestamp = env.ledger().timestamp();

        let audit_entry = AuditLogEntry {
            log_id: log_id.clone(),
            timestamp,
            actor: actor.clone(),
            action,
            resource_type,
            resource_id: resource_id.clone(),
            patient_id: patient_id.clone(),
            success: true,
            details,
            ip_address: String::from_str(&env, "127.0.0.1"), // Would be provided by client
            user_agent: String::from_str(&env, "Uzima-Client/1.0"), // Would be provided by client
            compliance_framework: framework,
            hipaa_category,
            gdpr_category,
        };

        let mut logs: Map<Address, Vec<AuditLogEntry>> = env
            .storage()
            .persistent()
            .get(&AUDIT_LOGS)
            .unwrap_or(Map::new(&env));

        let mut user_logs = logs.get(actor.clone()).unwrap_or(Vec::new(&env));
        user_logs.push_back(audit_entry);
        logs.set(actor, user_logs);

        env.storage().persistent().set(&AUDIT_LOGS, &logs);

        // Update compliance score based on audit activity
        Self::update_compliance_score(&env, true)?;

        Ok(())
    }

    /// Get audit logs for a specific user
    pub fn get_audit_logs(
        env: Env,
        user: Address,
        limit: u32,
    ) -> Result<Vec<AuditLogEntry>, Error> {
        let logs: Map<Address, Vec<AuditLogEntry>> = env
            .storage()
            .persistent()
            .get(&AUDIT_LOGS)
            .unwrap_or(Map::new(&env));

        let user_logs = logs.get(user).unwrap_or(Vec::new(&env));

        // Return last N logs (most recent first)
        let mut result = Vec::new(&env);
        let len = user_logs.len();
        let start = if len > limit {
            len.saturating_sub(limit)
        } else {
            0
        };

        for i in start..len {
            if let Some(log) = user_logs.get(i) {
                result.push_back(log);
            }
        }

        Ok(result)
    }

    /// Report data breach
    pub fn report_breach(env: Env, reporter: Address, breach: BreachReport) -> Result<(), Error> {
        #[cfg(not(test))]
        reporter.require_auth();
        Self::check_paused(&env)?;

        let config = Self::get_config(env.clone())?;
        if !config.breach_notification_enabled {
            return Err(Error::NotificationFailed);
        }

        // Store breach report
        let mut reports: Map<String, BreachReport> = env
            .storage()
            .persistent()
            .get(&BREACH_REPORTS)
            .unwrap_or(Map::new(&env));

        if reports.contains_key(breach.report_id.clone()) {
            return Err(Error::DataBreachAlreadyReported);
        }

        reports.set(breach.report_id.clone(), breach.clone());
        env.storage().persistent().set(&BREACH_REPORTS, &reports);

        // Log audit event
        Self::log_audit_event(
            env.clone(),
            reporter,
            AuditEventType::Breach,
            FHIRResourceType::AuditEvent,
            breach.report_id,
            String::from_str(&env, ""),
            String::from_str(&env, "Data breach reported"),
            ComplianceFramework::HIPAA,
            5, // PublicHealth
            0, // None
        )?;

        // Update compliance score
        Self::update_compliance_score(&env, false)?;

        // In a real implementation, this would trigger:
        // 1. Automated notifications to affected patients
        // 2. Notifications to regulatory authorities
        // 3. Incident response workflows

        Ok(())
    }

    /// Get compliance dashboard metrics
    pub fn get_compliance_metrics(env: Env) -> Result<ComplianceMetrics, Error> {
        let total_audits = Self::count_audit_logs(&env);
        let total_consents = Self::count_consents(&env);
        let total_breaches = Self::count_breaches(&env);
        let pending_violations = Self::count_pending_violations(&env);
        let compliance_score = env
            .storage()
            .instance()
            .get(&COMPLIANCE_SCORE)
            .unwrap_or(100u32);

        let metrics = ComplianceMetrics {
            total_audits,
            successful_audits: total_audits, // Simplified - in real implementation would track failures
            failed_audits: 0,
            total_consents,
            active_consents: Self::count_active_consents(&env),
            revoked_consents: total_consents.saturating_sub(Self::count_active_consents(&env)),
            total_breaches,
            resolved_breaches: Self::count_resolved_breaches(&env),
            pending_violations,
            compliance_score,
            last_audit_timestamp: env.ledger().timestamp(),
        };

        Ok(metrics)
    }

    /// Pause contract operations (emergency)
    pub fn pause(env: Env, admin: Address) -> Result<(), Error> {
        #[cfg(not(test))]
        admin.require_auth();
        Self::check_admin(&env, &admin)?;

        env.storage().instance().set(&PAUSED, &true);
        Ok(())
    }

    /// Resume contract operations
    pub fn resume(env: Env, admin: Address) -> Result<(), Error> {
        #[cfg(not(test))]
        admin.require_auth();
        Self::check_admin(&env, &admin)?;

        env.storage().instance().set(&PAUSED, &false);
        Ok(())
    }

    // ==================== Helper Functions ====================

    fn check_admin(env: &Env, address: &Address) -> Result<(), Error> {
        let config = Self::get_config(env.clone())?;
        for admin in config.admin_addresses.iter() {
            if &admin == address {
                return Ok(());
            }
        }
        Err(Error::NotAuthorized)
    }

    fn check_paused(env: &Env) -> Result<(), Error> {
        if env.storage().instance().get(&PAUSED).unwrap_or(false) {
            Err(Error::ContractPaused)
        } else {
            Ok(())
        }
    }

    fn generate_id(env: &Env) -> String {
        // Simple ID generation - in production use cryptographic random
        String::from_str(env, "id_")
    }

    fn update_compliance_score(env: &Env, positive: bool) -> Result<(), Error> {
        let mut score = env
            .storage()
            .instance()
            .get(&COMPLIANCE_SCORE)
            .unwrap_or(100u32);

        if positive && score < 100 {
            score = score.saturating_add(1);
        } else if !positive && score > 0 {
            score = score.saturating_sub(5); // Larger penalty for violations
        }

        env.storage().instance().set(&COMPLIANCE_SCORE, &score);
        Ok(())
    }

    fn count_audit_logs(env: &Env) -> u32 {
        let logs: Map<String, Vec<AuditLogEntry>> = env
            .storage()
            .persistent()
            .get(&AUDIT_LOGS)
            .unwrap_or(Map::new(env));

        let mut count = 0u32;
        for (_user, user_logs) in logs.iter() {
            count = count.saturating_add(user_logs.len());
        }
        count
    }

    fn count_consents(env: &Env) -> u32 {
        let consents: Map<String, ConsentRecord> = env
            .storage()
            .persistent()
            .get(&CONSENTS)
            .unwrap_or(Map::new(env));
        consents.len()
    }

    fn count_active_consents(env: &Env) -> u32 {
        let consents: Map<String, ConsentRecord> = env
            .storage()
            .persistent()
            .get(&CONSENTS)
            .unwrap_or(Map::new(env));

        let mut count = 0u32;
        let current_time = env.ledger().timestamp();

        for (_id, consent) in consents.iter() {
            if consent.status == ConsentStatus::Active
                && consent.granted_at <= current_time
                && (consent.expires_at == 0 || consent.expires_at > current_time)
            {
                count = count.saturating_add(1);
            }
        }
        count
    }

    fn count_breaches(env: &Env) -> u32 {
        let breaches: Map<String, BreachReport> = env
            .storage()
            .persistent()
            .get(&BREACH_REPORTS)
            .unwrap_or(Map::new(env));
        breaches.len()
    }

    fn count_resolved_breaches(env: &Env) -> u32 {
        let breaches: Map<String, BreachReport> = env
            .storage()
            .persistent()
            .get(&BREACH_REPORTS)
            .unwrap_or(Map::new(env));

        let mut count = 0u32;
        for (_id, breach) in breaches.iter() {
            if breach.resolution_status == String::from_str(env, "resolved")
                || breach.resolution_status == String::from_str(env, "closed")
            {
                count = count.saturating_add(1);
            }
        }
        count
    }

    fn count_pending_violations(env: &Env) -> u32 {
        let violations: Map<String, ViolationReport> = env
            .storage()
            .persistent()
            .get(&VIOLATION_REPORTS)
            .unwrap_or(Map::new(env));

        let mut count = 0u32;
        for (_id, violation) in violations.iter() {
            if !violation.resolved {
                count = count.saturating_add(1);
            }
        }
        count
    }
}
