pub mod cross_chain_tests;
pub mod framework_tests;
pub mod healthcare_workflows;
pub mod ihe_fhir_integration_tests;
pub mod medical_records_integration;
pub mod multi_region_dr_integration;

// ============================================================================
// Inline integration tests for Medical Records
// ============================================================================
#[cfg(test)]
pub mod medical_records_tests {
    use soroban_sdk::{vec, Address, Env, String};
    use medical_records::{MedicalRecordsContract, MedicalRecordsContractClient, Role};
    use soroban_sdk::{
        testutils::{Address as _, MockAuth, MockAuthInvoke},
    };

    #[test]
    fn test_full_medical_record_workflow() {
        let env = Env::default();
        let contract_id = env.register_contract(None, MedicalRecordsContract);
        let client = MedicalRecordsContractClient::new(&env, &contract_id);

        // Setup test data
        let admin = Address::generate(&env);
        let doctor = Address::generate(&env);
        let patient = Address::generate(&env);
        let diagnosis = String::from_str(&env, "Hypertension");
        let treatment = String::from_str(&env, "ACE inhibitor medication");

        // Initialize contract and roles
        client.mock_all_auths().initialize(&admin);
        client.mock_all_auths().manage_user(&admin, &doctor, &Role::Doctor);
        client.mock_all_auths().manage_user(&admin, &patient, &Role::Patient);

        // Add a medical record
        let record_id = client
            .mock_auths(&[MockAuth {
                address: &doctor,
                invoke: &MockAuthInvoke {
                    contract: &contract_id,
                    fn_name: "add_record",
                    args: (),
                    sub_invokes: &[],
                },
            }])
            .add_record(
                &doctor,
                &patient,
                &diagnosis,
                &treatment,
                &false,
                &vec![
                    &env,
                    String::from_str(&env, "herbal"),
                    String::from_str(&env, "spiritual"),
                ],
                &String::from_str(&env, "Traditional"),
                &String::from_str(&env, "Herbal Therapy"),
                &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx"),
            );

        // Verify record was added
        let record = client.get_record(&patient, &record_id);
        assert_eq!(record.patient_id, patient);
        assert_eq!(record.diagnosis, diagnosis);
        assert_eq!(record.category, String::from_str(&env, "Traditional"));
        assert_eq!(record.treatment_type, String::from_str(&env, "Herbal Therapy"));
        assert_eq!(record.tags.len(), 2);
    }

    #[test]
    fn test_pause_blocks_add_record_integration() {
        let env = Env::default();
        let contract_id = env.register_contract(None, MedicalRecordsContract);
        let client = MedicalRecordsContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let doctor = Address::generate(&env);
        let patient = Address::generate(&env);

        client.mock_all_auths().initialize(&admin);
        client.mock_all_auths().manage_user(&admin, &doctor, &Role::Doctor);
        client.mock_all_auths().manage_user(&admin, &patient, &Role::Patient);

        assert!(client.mock_all_auths().pause(&admin));

        let res = client
            .mock_auths(&[MockAuth { address: &doctor, invoke: &MockAuthInvoke { contract: &contract_id, fn_name: "add_record", args: (), sub_invokes: &[] } }])
            .try_add_record(
                &doctor,
                &patient,
                &String::from_str(&env, "Diagnosis"),
                &String::from_str(&env, "Treatment"),
                &false,
                &vec![&env, String::from_str(&env, "herbal")],
                &String::from_str(&env, "Traditional"),
                &String::from_str(&env, "Herbal Therapy"),
                &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx"),
            );
        assert!(res.is_err());
    }

    #[test]
    fn test_recovery_flow_integration() {
        let env = Env::default();
        let contract_id = env.register_contract(None, MedicalRecordsContract);
        let client = MedicalRecordsContractClient::new(&env, &contract_id);

        let admin1 = Address::generate(&env);
        let admin2 = Address::generate(&env);
        let token = Address::generate(&env);
        let recipient = Address::generate(&env);

        client.mock_all_auths().initialize(&admin1);
        client.mock_all_auths().manage_user(&admin1, &admin2, &Role::Admin);

        let proposal_id = client.mock_all_auths().propose_recovery(&admin1, &token, &recipient, &100i128);
        assert!(proposal_id > 0);

        assert!(client.mock_all_auths().approve_recovery(&admin2, &proposal_id));

        // Fail before timelock
        let res = client.mock_all_auths().try_execute_recovery(&admin1, &proposal_id);
        assert!(res.is_err());

        let now = env.ledger().timestamp();
        env.ledger().with_mut(|l| l.timestamp = now + 86_401);

        assert!(client.mock_all_auths().execute_recovery(&admin1, &proposal_id));
    }
}

// ============================================================================
// Inline integration tests for Patient Consent Management
// ============================================================================
#[cfg(test)]
mod patient_consent_tests {
    use super::*;
    use patient_consent_management::{PatientConsentManagement, PatientConsentManagementClient};
    use soroban_sdk::{
        testutils::{Address as _, MockAuth, MockAuthInvoke},
    };

    #[test]
    fn test_consent_grant_and_check_integration() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PatientConsentManagement);
        let client = PatientConsentManagementClient::new(&env, &contract_id);

        env.mock_all_auths();

        let admin = Address::generate(&env);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);

        client.initialize(&admin);
        client.grant_consent(&patient, &provider);

        assert!(client.check_consent(&patient, &provider));
    }

    #[test]
    fn test_consent_revoke_and_recheck() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PatientConsentManagement);
        let client = PatientConsentManagementClient::new(&env, &contract_id);

        env.mock_all_auths();

        let admin = Address::generate(&env);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);

        client.initialize(&admin);
        client.grant_consent(&patient, &provider);
        assert!(client.check_consent(&patient, &provider));

        client.revoke_consent(&patient, &provider);
        assert!(!client.check_consent(&patient, &provider));
    }

    #[test]
    fn test_consent_with_audit_trail() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PatientConsentManagement);
        let client = PatientConsentManagementClient::new(&env, &contract_id);

        env.mock_all_auths();

        let admin = Address::generate(&env);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);

        client.initialize(&admin);
        client.grant_consent(&patient, &provider);

        let (has_consent, granted_at, revoked_at) = client.verify_consent_with_audit(&patient, &provider);
        assert!(has_consent);
        assert!(granted_at > 0);
        assert_eq!(revoked_at, 0);

        client.revoke_consent(&patient, &provider);

        let (has_consent, _granted_at, revoked_at) = client.verify_consent_with_audit(&patient, &provider);
        assert!(!has_consent);
        assert!(revoked_at > 0);
    }
}

// ============================================================================
// Basic unit tests
// ============================================================================
#[cfg(test)]
mod unit_tests {
    use soroban_sdk::{Env, String};

    #[test]
    fn test_string_operations() {
        let env = Env::default();
        let test_string = String::from_str(&env, "test_patient_id");
        assert_eq!(test_string.len(), 15);
    }

    #[test]
    fn test_environment_setup() {
        let env = Env::default();
        assert!(env.ledger().timestamp() > 0);
        assert!(env.ledger().sequence() > 0);
    }
}
