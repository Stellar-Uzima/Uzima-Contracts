/// Comprehensive integration tests for healthcare workflow scenarios
use soroban_sdk::{vec, String, testutils::Address as _};
use crate::utils::{IntegrationTestEnv};
use medical_records::{Role, MedicalRecordsContractClient};
use patient_consent_management::PatientConsentManagementClient;

#[test]
fn test_user_registration_workflow() {
    let test_env = IntegrationTestEnv::new();
    let (_, records_client) = test_env.register_medical_records();
    
    let admin = &test_env.team.admin.address;
    let doctor = &test_env.team.doctors[0].address;
    let patient = &test_env.team.patients[0].address;

    // Workflow: Admin registers doctor and patient
    records_client.initialize(admin);
    records_client.manage_user(admin, doctor, &Role::Doctor);
    records_client.manage_user(admin, patient, &Role::Patient);

    // Verify roles
    assert_eq!(records_client.get_user_role(doctor), Role::Doctor);
    assert_eq!(records_client.get_user_role(patient), Role::Patient);
}

#[test]
fn test_record_creation_retrieval_workflow() {
    let test_env = IntegrationTestEnv::new();
    let (records_id, records_client) = test_env.register_medical_records();
    
    let admin = &test_env.team.admin.address;
    let doctor = &test_env.team.doctors[0].address;
    let patient = &test_env.team.patients[0].address;

    records_client.initialize(admin);
    records_client.manage_user(admin, doctor, &Role::Doctor);
    records_client.manage_user(admin, patient, &Role::Patient);

    // 1. Doctor creates a medical record
    let diagnosis = String::from_str(&test_env.env, "Integration Test Diagnosis");
    let treatment = String::from_str(&test_env.env, "Automated Treatment");
    
    let record_id = records_client.add_record(
        doctor,
        patient,
        &diagnosis,
        &treatment,
        &false,
        &vec![&test_env.env, String::from_str(&test_env.env, "test")],
        &String::from_str(&test_env.env, "General"),
        &String::from_str(&test_env.env, "Testing"),
        &String::from_str(&test_env.env, "QmHash"),
    );

    // 2. Retrieve and verify
    let record = records_client.get_record(patient, &record_id);
    assert_eq!(record.diagnosis, diagnosis);
    assert_eq!(record.doctor_id, *doctor);

    // 3. Verify event
    test_env.assert_event_topics(&records_id, test_env.topics(&["EVENT", "REC_NEW"]));
}

#[test]
fn test_pause_emergency_workflow() {
    let test_env = IntegrationTestEnv::new();
    let (_, records_client) = test_env.register_medical_records();
    
    let admin = &test_env.team.admin.address;
    let doctor = &test_env.team.doctors[0].address;
    let patient = &test_env.team.patients[0].address;

    records_client.initialize(admin);
    records_client.manage_user(admin, doctor, &Role::Doctor);

    // Pause contract
    records_client.pause(admin);
    assert!(records_client.is_paused());

    // Try to add record while paused (should fail)
    let res = records_client.try_add_record(
        doctor,
        patient,
        &String::from_str(&test_env.env, "D"),
        &String::from_str(&test_env.env, "T"),
        &false,
        &vec![&test_env.env],
        &String::from_str(&test_env.env, "C"),
        &String::from_str(&test_env.env, "T"),
        &String::from_str(&test_env.env, "H"),
    );
    assert!(res.is_err());

    // Unpause
    records_client.unpause(admin);
    assert!(!records_client.is_paused());
}

// ============================================================================
// Integrated Patient Consent → Medical Records → RBAC Pipeline Tests
// ============================================================================

/// Test the full healthcare workflow: patient grants consent → doctor accesses record →
/// RBAC verifies permission → audit events emitted
#[test]
fn test_patient_consent_to_record_access_happy_path() {
    let test_env = IntegrationTestEnv::new();
    let env = &test_env.env;

    // Register all three contracts
    let (consent_id, consent_client) = test_env.register_patient_consent();
    let (records_id, records_client) = test_env.register_medical_records();

    let admin = &test_env.team.admin.address;
    let doctor = &test_env.team.doctors[0].address;
    let patient = &test_env.team.patients[0].address;

    // 1. Initialize contracts
    consent_client.initialize(admin);
    records_client.initialize(admin);

    // 2. Set up users via MedicalRecords RBAC proxy
    records_client.manage_user(admin, doctor, &Role::Doctor);
    records_client.manage_user(admin, patient, &Role::Patient);

    // 3. Patient grants consent to doctor
    consent_client.grant_consent(patient, doctor);

    // 4. Verify consent is active
    let has_consent = consent_client.check_consent(patient, doctor);
    assert!(has_consent);

    // 5. Doctor creates a medical record for the patient
    let diagnosis = String::from_str(env, "Hypertension");
    let treatment = String::from_str(env, "ACE Inhibitor");
    let record_id = records_client.add_record(
        doctor,
        patient,
        &diagnosis,
        &treatment,
        &false,
        &vec![env, String::from_str(env, "cardiology")],
        &String::from_str(env, "Modern"),
        &String::from_str(env, "Medication"),
        &String::from_str(env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );

    // 6. Patient can access the record
    let record = records_client.get_record(patient, &record_id);
    assert_eq!(record.patient_id, *patient);
    assert_eq!(record.diagnosis, diagnosis);

    // 7. Doctor can access the record (creator)
    let record = records_client.get_record(doctor, &record_id);
    assert_eq!(record.patient_id, *patient);

    // 8. Verify events were emitted from both contracts
    test_env.assert_event_topics(&consent_id, test_env.topics(&["CONSENT", "GRANT"]));
    test_env.assert_event_topics(&records_id, test_env.topics(&["REC_NEW"]));
    test_env.assert_event_topics(&records_id, test_env.topics(&["REC_ACC"]));
}

/// Test that an unauthorized doctor cannot access records without consent
#[test]
fn test_unauthorized_access_denied() {
    let test_env = IntegrationTestEnv::new();
    let env = &test_env.env;

    let (consent_id, consent_client) = test_env.register_patient_consent();
    let (_, records_client) = test_env.register_medical_records();

    let admin = &test_env.team.admin.address;
    let authorized_doctor = &test_env.team.doctors[0].address;
    let unauthorized_doctor = &test_env.team.doctors[1].address;
    let patient = &test_env.team.patients[0].address;

    // Initialize contracts
    consent_client.initialize(admin);
    records_client.initialize(admin);

    // Set up users
    records_client.manage_user(admin, authorized_doctor, &Role::Doctor);
    records_client.manage_user(admin, unauthorized_doctor, &Role::Doctor);
    records_client.manage_user(admin, patient, &Role::Patient);

    // Patient grants consent ONLY to authorized doctor
    consent_client.grant_consent(patient, authorized_doctor);

    // Authorized doctor creates a record
    let record_id = records_client.add_record(
        authorized_doctor,
        patient,
        &String::from_str(env, "Diagnosis"),
        &String::from_str(env, "Treatment"),
        &false,
        &vec![env, String::from_str(env, "test")],
        &String::from_str(env, "General"),
        &String::from_str(env, "Testing"),
        &String::from_str(env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );

    // Unauthorized doctor tries to read the record (should fail)
    // Note: MedicalRecords allows the creator doctor to read, but not unauthorized doctors
    // unless they have been granted consent or permissions
    let result = records_client.try_get_record(unauthorized_doctor, &record_id);
    // The authorization check is performed via check_permission in medical_records;
    // an unauthorized doctor who is not the creator and has no explicit permission is denied.
    assert!(result.is_err());

    // Verify consent is checked: unauthorized doctor has no consent
    let has_consent = consent_client.check_consent(patient, unauthorized_doctor);
    assert!(!has_consent);

    // Verify the authorized doctor has consent
    let has_consent = consent_client.check_consent(patient, authorized_doctor);
    assert!(has_consent);

    // Verify events
    test_env.assert_event_topics(&consent_id, test_env.topics(&["CONSENT", "CHECK"]));
}

/// Test that revoked consent is reflected correctly
#[test]
fn test_revoked_consent_state() {
    let test_env = IntegrationTestEnv::new();
    let env = &test_env.env;

    let (consent_id, consent_client) = test_env.register_patient_consent();
    let (_, records_client) = test_env.register_medical_records();

    let admin = &test_env.team.admin.address;
    let doctor = &test_env.team.doctors[0].address;
    let patient = &test_env.team.patients[0].address;

    consent_client.initialize(admin);
    records_client.initialize(admin);
    records_client.manage_user(admin, doctor, &Role::Doctor);
    records_client.manage_user(admin, patient, &Role::Patient);

    // Patient grants consent
    consent_client.grant_consent(patient, doctor);
    assert!(consent_client.check_consent(patient, doctor));

    // Doctor creates a record
    let record_id = records_client.add_record(
        doctor,
        patient,
        &String::from_str(env, "Diagnosis"),
        &String::from_str(env, "Treatment"),
        &false,
        &vec![env, String::from_str(env, "test")],
        &String::from_str(env, "General"),
        &String::from_str(env, "Testing"),
        &String::from_str(env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );

    // Patient revokes consent
    consent_client.revoke_consent(patient, doctor);
    assert!(!consent_client.check_consent(patient, doctor));

    // Verify consent check events
    test_env.assert_event_topics(&consent_id, test_env.topics(&["CONSENT", "REVOKE"]));

    // Verify the doctor (who created the record) can still read it
    // (record creators have inherent access regardless of consent in medical_records)
    let record = records_client.get_record(doctor, &record_id);
    assert_eq!(record.patient_id, *patient);
}

/// Test multiple providers with consent management
#[test]
fn test_multiple_providers_consent_workflow() {
    let test_env = IntegrationTestEnv::new();
    let env = &test_env.env;

    let (_, consent_client) = test_env.register_patient_consent();
    let (_, records_client) = test_env.register_medical_records();

    let admin = &test_env.team.admin.address;
    let doctor1 = &test_env.team.doctors[0].address;
    let doctor2 = &test_env.team.doctors[1].address;
    let patient = &test_env.team.patients[0].address;

    consent_client.initialize(admin);
    records_client.initialize(admin);
    records_client.manage_user(admin, doctor1, &Role::Doctor);
    records_client.manage_user(admin, doctor2, &Role::Doctor);
    records_client.manage_user(admin, patient, &Role::Patient);

    // Patient grants consent to both doctors
    consent_client.grant_consent(patient, doctor1);
    consent_client.grant_consent(patient, doctor2);

    // Both doctors can create records
    let record_id1 = records_client.add_record(
        doctor1,
        patient,
        &String::from_str(env, "Diagnosis 1"),
        &String::from_str(env, "Treatment 1"),
        &false,
        &vec![env, String::from_str(env, "tag1")],
        &String::from_str(env, "General"),
        &String::from_str(env, "Type1"),
        &String::from_str(env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );

    let record_id2 = records_client.add_record(
        doctor2,
        patient,
        &String::from_str(env, "Diagnosis 2"),
        &String::from_str(env, "Treatment 2"),
        &false,
        &vec![env, String::from_str(env, "tag2")],
        &String::from_str(env, "General"),
        &String::from_str(env, "Type2"),
        &String::from_str(env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );

    // Verify both records exist
    assert_eq!(records_client.get_patient_record_count(patient), 2);

    // Patient can view both records
    let history = records_client.get_history(patient, patient, &0u32, &10u32);
    assert_eq!(history.len(), 2);

    // Verify active consent count
    let active_count = consent_client.get_active_consent_count(patient);
    assert_eq!(active_count, 2);

    // Revoke consent for doctor1
    consent_client.revoke_consent(patient, doctor1);

    // Verify count decreased
    let active_count = consent_client.get_active_consent_count(patient);
    assert_eq!(active_count, 1);
}

/// Test audit trail across consent and records
#[test]
fn test_audit_events_across_pipeline() {
    let test_env = IntegrationTestEnv::new();
    let env = &test_env.env;

    let (consent_id, consent_client) = test_env.register_patient_consent();
    let (records_id, records_client) = test_env.register_medical_records();

    let admin = &test_env.team.admin.address;
    let doctor = &test_env.team.doctors[0].address;
    let patient = &test_env.team.patients[0].address;

    // Initialize and set up
    consent_client.initialize(admin);
    records_client.initialize(admin);
    records_client.manage_user(admin, doctor, &Role::Doctor);
    records_client.manage_user(admin, patient, &Role::Patient);

    // Patient grants consent
    consent_client.grant_consent(patient, doctor);

    // Doctor creates a record
    let record_id = records_client.add_record(
        doctor,
        patient,
        &String::from_str(env, "Diagnosis"),
        &String::from_str(env, "Treatment"),
        &false,
        &vec![env, String::from_str(env, "test")],
        &String::from_str(env, "General"),
        &String::from_str(env, "Testing"),
        &String::from_str(env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );

    // Patient accesses the record
    let _record = records_client.get_record(patient, &record_id);

    // Get all events
    let events = test_env.get_events();

    // Count consent events
    let consent_events = events
        .iter()
        .filter(|(id, _, _)| *id == consent_id)
        .collect::<Vec<_>>();
    assert!(consent_events.len() >= 1, "Should have at least one consent event");

    // Count medical record events
    let record_events = events
        .iter()
        .filter(|(id, _, _)| *id == records_id)
        .collect::<Vec<_>>();
    assert!(record_events.len() >= 2, "Should have record and access events");

    // Verify specific event topics exist
    test_env.assert_event_topics(&consent_id, test_env.topics(&["CONSENT", "GRANT"]));
    test_env.assert_event_topics(&records_id, test_env.topics(&["REC_NEW"]));
    test_env.assert_event_topics(&records_id, test_env.topics(&["REC_ACC"]));
}

/// Test emergency access scenario (consent grant then record access)
#[test]
fn test_emergency_consent_and_access() {
    let test_env = IntegrationTestEnv::new();
    let env = &test_env.env;

    let (consent_id, consent_client) = test_env.register_patient_consent();
    let (_, records_client) = test_env.register_medical_records();

    let admin = &test_env.team.admin.address;
    let doctor = &test_env.team.doctors[0].address;
    let patient = &test_env.team.patients[0].address;

    // Initialize contracts
    consent_client.initialize(admin);
    records_client.initialize(admin);
    records_client.manage_user(admin, doctor, &Role::Doctor);
    records_client.manage_user(admin, patient, &Role::Patient);

    // Emergency: Patient grants consent under urgent circumstances
    consent_client.grant_consent(patient, doctor);
    assert!(consent_client.check_consent(patient, doctor));

    // Doctor creates a record in response to emergency
    let record_id = records_client.add_record(
        doctor,
        patient,
        &String::from_str(env, "Emergency Diagnosis"),
        &String::from_str(env, "Emergency Treatment"),
        &false,
        &vec![env, String::from_str(env, "emergency")],
        &String::from_str(env, "Emergency"),
        &String::from_str(env, "Emergency Care"),
        &String::from_str(env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );

    // Verify record was created and accessible
    let record = records_client.get_record(doctor, &record_id);
    assert_eq!(record.patient_id, *patient);
    assert_eq!(record.category, String::from_str(env, "Emergency"));

    // Verify events across both contracts
    test_env.assert_event_topics(&consent_id, test_env.topics(&["CONSENT", "GRANT"]));
    test_env.assert_event_topics(&consent_id, test_env.topics(&["CONSENT", "CHECK"]));
    test_env.assert_event_topics(&records_id, test_env.topics(&["REC_NEW"]));

    // After emergency, patient can view their consent history
    let consent_log = consent_client.get_patient_consents(patient);
    assert!(consent_log.is_some());
    let log = consent_log.unwrap();
    assert_eq!(log.record_count, 1);
    assert!(log.records.get(0).unwrap().active);
}

