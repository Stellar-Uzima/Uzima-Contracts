#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

#[test]
fn test_initialize_contract() {
    let env = Env::default();
    let admin = Address::generate(&env);
    
    // Initialize contract
    let result = HealthcareComplianceContract::initialize(env.clone(), admin.clone());
    assert!(result.is_ok());
    
    // Try to initialize again - should fail
    let result2 = HealthcareComplianceContract::initialize(env.clone(), admin.clone());
    assert!(result2.is_err());
    assert_eq!(result2.unwrap_err(), Error::ConsentAlreadyExists);
    
    // Check config is set
    let config = HealthcareComplianceContract::get_config(env.clone()).unwrap();
    assert!(config.hipaa_enabled);
    assert!(config.gdpr_enabled);
    assert!(config.hl7_fhir_enabled);
    assert!(config.audit_logging_enabled);
    assert!(config.breach_notification_enabled);
}

#[test]
fn test_consent_management() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let patient = Address::generate(&env);
    
    HealthcareComplianceContract::initialize(env.clone(), admin.clone()).unwrap();
    
    // Create consent record
    let consent = ConsentRecord {
        consent_id: String::from_str(&env, "consent_001"),
        patient: patient.clone(),
        data_controller: admin.clone(),
        data_processor: Address::generate(&env),
        purpose: String::from_str(&env, "medical_treatment"),
        data_categories: Vec::from_array(&env, [String::from_str(&env, "medical_records")]),
        processing_categories: Vec::from_array(&env, [GDPRProcessingCategory::Consent]),
        status: ConsentStatus::Active,
        granted_at: env.ledger().timestamp(),
        expires_at: env.ledger().timestamp() + 31536000, // 1 year
        revoked_at: 0,
        revocation_reason: String::from_str(&env, ""),
        signature: [0u8; 64].into(),
    };
    
    // Grant consent
    let result = HealthcareComplianceContract::grant_consent(env.clone(), patient.clone(), consent.clone());
    assert!(result.is_ok());
    
    // Try to grant same consent again - should fail
    let result2 = HealthcareComplianceContract::grant_consent(env.clone(), patient.clone(), consent.clone());
    assert!(result2.is_err());
    assert_eq!(result2.unwrap_err(), Error::ConsentAlreadyExists);
    
    // Check if patient has valid consent
    let has_consent = HealthcareComplianceContract::has_valid_consent(
        env.clone(),
        patient.clone(),
        String::from_str(&env, "medical_treatment"),
        String::from_str(&env, "medical_records"),
    ).unwrap();
    assert!(has_consent);
    
    // Revoke consent
    let result3 = HealthcareComplianceContract::revoke_consent(
        env.clone(),
        patient.clone(),
        String::from_str(&env, "consent_001"),
        String::from_str(&env, "Patient requested revocation"),
    );
    assert!(result3.is_ok());
    
    // Check consent is no longer valid
    let has_consent_after = HealthcareComplianceContract::has_valid_consent(
        env.clone(),
        patient.clone(),
        String::from_str(&env, "medical_treatment"),
        String::from_str(&env, "medical_records"),
    ).unwrap();
    assert!(!has_consent_after);
}

#[test]
fn test_audit_logging() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);
    
    HealthcareComplianceContract::initialize(env.clone(), admin.clone()).unwrap();
    
    // Log audit event
    let result = HealthcareComplianceContract::log_audit_event(
        env.clone(),
        doctor.clone(),
        AuditEventType::Read,
        FHIRResourceType::Patient,
        String::from_str(&env, "patient_123"),
        String::from_str(&env, patient.to_string()),
        String::from_str(&env, "Doctor accessed patient record"),
        ComplianceFramework::HIPAA,
        Some(HIPAACategory::Treatment),
        None,
    );
    assert!(result.is_ok());
    
    // Get audit logs
    let logs = HealthcareComplianceContract::get_audit_logs(env.clone(), doctor.clone(), 10).unwrap();
    assert_eq!(logs.len(), 1);
    
    let log_entry = logs.get(0).unwrap();
    assert_eq!(log_entry.actor, doctor);
    assert_eq!(log_entry.action, AuditEventType::Read);
    assert_eq!(log_entry.resource_type, FHIRResourceType::Patient);
    assert_eq!(log_entry.compliance_framework, ComplianceFramework::HIPAA);
    assert_eq!(log_entry.hipaa_category, Some(HIPAACategory::Treatment));
}

#[test]
fn test_breach_reporting() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let reporter = Address::generate(&env);
    
    HealthcareComplianceContract::initialize(env.clone(), admin.clone()).unwrap();
    
    // Create breach report
    let breach = BreachReport {
        report_id: String::from_str(&env, "breach_001"),
        timestamp: env.ledger().timestamp(),
        reporter: reporter.clone(),
        severity: BreachSeverity::High,
        affected_records: 100,
        affected_patients: Vec::from_array(&env, [Address::generate(&env)]),
        breach_type: String::from_str(&env, "unauthorized_access"),
        description: String::from_str(&env, "Unauthorized access to patient records detected"),
        mitigation_steps: Vec::from_array(&env, [
            String::from_str(&env, "Immediate access revocation"),
            String::from_str(&env, "Security audit initiated"),
        ]),
        notified_authorities: false,
        notified_patients: false,
        resolution_status: String::from_str(&env, "investigating"),
    };
    
    // Report breach
    let result = HealthcareComplianceContract::report_breach(env.clone(), reporter.clone(), breach.clone());
    assert!(result.is_ok());
    
    // Try to report same breach again - should fail
    let result2 = HealthcareComplianceContract::report_breach(env.clone(), reporter.clone(), breach.clone());
    assert!(result2.is_err());
    assert_eq!(result2.unwrap_err(), Error::DataBreachAlreadyReported);
}

#[test]
fn test_compliance_metrics() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);
    
    HealthcareComplianceContract::initialize(env.clone(), admin.clone()).unwrap();
    
    // Create some activity
    let consent = ConsentRecord {
        consent_id: String::from_str(&env, "consent_001"),
        patient: patient.clone(),
        data_controller: admin.clone(),
        data_processor: doctor.clone(),
        purpose: String::from_str(&env, "medical_treatment"),
        data_categories: Vec::from_array(&env, [String::from_str(&env, "medical_records")]),
        processing_categories: Vec::from_array(&env, [GDPRProcessingCategory::Consent]),
        status: ConsentStatus::Active,
        granted_at: env.ledger().timestamp(),
        expires_at: env.ledger().timestamp() + 31536000,
        revoked_at: 0,
        revocation_reason: String::from_str(&env, ""),
        signature: [0u8; 64].into(),
    };
    
    HealthcareComplianceContract::grant_consent(env.clone(), patient.clone(), consent).unwrap();
    HealthcareComplianceContract::log_audit_event(
        env.clone(),
        doctor.clone(),
        AuditEventType::Read,
        FHIRResourceType::Patient,
        String::from_str(&env, "patient_123"),
        String::from_str(&env, patient.to_string()),
        String::from_str(&env, "Doctor accessed patient record"),
        ComplianceFramework::HIPAA,
        Some(HIPAACategory::Treatment),
        None,
    ).unwrap();
    
    // Get compliance metrics
    let metrics = HealthcareComplianceContract::get_compliance_metrics(env.clone()).unwrap();
    assert!(metrics.total_audits > 0);
    assert!(metrics.total_consents > 0);
    assert!(metrics.active_consents > 0);
    assert_eq!(metrics.total_breaches, 0);
    assert!(metrics.compliance_score > 0);
}

#[test]
fn test_admin_functions() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    
    HealthcareComplianceContract::initialize(env.clone(), admin.clone()).unwrap();
    
    // Test pause functionality
    let result = HealthcareComplianceContract::pause(env.clone(), admin.clone());
    assert!(result.is_ok());
    
    // Try to perform actions when paused
    let consent = ConsentRecord {
        consent_id: String::from_str(&env, "consent_002"),
        patient: non_admin.clone(),
        data_controller: admin.clone(),
        data_processor: Address::generate(&env),
        purpose: String::from_str(&env, "medical_treatment"),
        data_categories: Vec::from_array(&env, [String::from_str(&env, "medical_records")]),
        processing_categories: Vec::from_array(&env, [GDPRProcessingCategory::Consent]),
        status: ConsentStatus::Active,
        granted_at: env.ledger().timestamp(),
        expires_at: env.ledger().timestamp() + 31536000,
        revoked_at: 0,
        revocation_reason: String::from_str(&env, ""),
        signature: [0u8; 64].into(),
    };
    
    let result2 = HealthcareComplianceContract::grant_consent(env.clone(), non_admin.clone(), consent);
    assert!(result2.is_err());
    assert_eq!(result2.unwrap_err(), Error::ContractPaused);
    
    // Resume contract
    let result3 = HealthcareComplianceContract::resume(env.clone(), admin.clone());
    assert!(result3.is_ok());
    
    // Non-admin should not be able to pause
    let result4 = HealthcareComplianceContract::pause(env.clone(), non_admin.clone());
    assert!(result4.is_err());
    assert_eq!(result4.unwrap_err(), Error::NotAuthorized);
}

#[test]
fn test_consent_expiration() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let patient = Address::generate(&env);
    
    HealthcareComplianceContract::initialize(env.clone(), admin.clone()).unwrap();
    
    // Create expired consent
    let expired_consent = ConsentRecord {
        consent_id: String::from_str(&env, "consent_expired"),
        patient: patient.clone(),
        data_controller: admin.clone(),
        data_processor: Address::generate(&env),
        purpose: String::from_str(&env, "medical_treatment"),
        data_categories: Vec::from_array(&env, [String::from_str(&env, "medical_records")]),
        processing_categories: Vec::from_array(&env, [GDPRProcessingCategory::Consent]),
        status: ConsentStatus::Active,
        granted_at: env.ledger().timestamp() - 31536000, // 1 year ago
        expires_at: env.ledger().timestamp() - 1000, // Already expired
        revoked_at: 0,
        revocation_reason: String::from_str(&env, ""),
        signature: [0u8; 64].into(),
    };
    
    // Grant expired consent - should fail
    let result = contract.grant_consent(env.clone(), patient.clone(), expired_consent);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Error::ConsentExpired);
}

#[test]
fn test_invalid_consent_operations() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let patient = Address::generate(&env);
    let other_patient = Address::generate(&env);
    
    HealthcareComplianceContract::initialize(env.clone(), admin.clone()).unwrap();
    
    // Create consent for one patient
    let consent = ConsentRecord {
        consent_id: String::from_str(&env, "consent_001"),
        patient: patient.clone(),
        data_controller: admin.clone(),
        data_processor: Address::generate(&env),
        purpose: String::from_str(&env, "medical_treatment"),
        data_categories: Vec::from_array(&env, [String::from_str(&env, "medical_records")]),
        processing_categories: Vec::from_array(&env, [GDPRProcessingCategory::Consent]),
        status: ConsentStatus::Active,
        granted_at: env.ledger().timestamp(),
        expires_at: env.ledger().timestamp() + 31536000,
        revoked_at: 0,
        revocation_reason: String::from_str(&env, ""),
        signature: [0u8; 64].into(),
    };
    
    HealthcareComplianceContract::grant_consent(env.clone(), patient.clone(), consent).unwrap();
    
    // Other patient tries to revoke consent - should fail
    let result = contract.revoke_consent(
        env.clone(),
        other_patient.clone(),
        String::from_str(&env, "consent_001"),
        String::from_str(&env, "Unauthorized revocation attempt"),
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Error::NotAuthorized);
}

#[test]
fn test_compliance_score_updates() {
    let env = Env::default();
    let admin = Address::generate(&env);
    
    HealthcareComplianceContract::initialize(env.clone(), admin.clone()).unwrap();
    
    // Get initial compliance score
    let initial_score = contract.get_compliance_metrics(env.clone()).unwrap().compliance_score;
    assert_eq!(initial_score, 100);
    
    // Report a breach to reduce score
    let breach = BreachReport {
        report_id: String::from_str(&env, "breach_001"),
        timestamp: env.ledger().timestamp(),
        reporter: admin.clone(),
        severity: BreachSeverity::High,
        affected_records: 1,
        affected_patients: Vec::from_array(&env, [Address::generate(&env)]),
        breach_type: String::from_str(&env, "test_breach"),
        description: String::from_str(&env, "Test breach for compliance score"),
        mitigation_steps: Vec::from_array(&env, [String::from_str(&env, "Test mitigation")]),
        notified_authorities: false,
        notified_patients: false,
        resolution_status: String::from_str(&env, "investigating"),
    };
    
    HealthcareComplianceContract::report_breach(env.clone(), admin.clone(), breach).unwrap();
    
    // Check score decreased
    let new_score = contract.get_compliance_metrics(env.clone()).unwrap().compliance_score;
    assert!(new_score < initial_score);
    assert_eq!(new_score, 95); // Should be 5 points less
}