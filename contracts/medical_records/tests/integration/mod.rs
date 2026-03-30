// tests/integration/mod.rs
use soroban_sdk::{Address, Env, String};

pub mod medical_records_tests {
    use super::*;
    use medical_records::{MedicalRecordsContract, MedicalRecordsContractClient, Role, Error, RateLimitConfig};
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
                &vec![String::from_str(&env, "herbal"), String::from_str(&env, "spiritual")],
                String::from_str(&env, "Traditional"),
                String::from_str(&env, "Herbal Therapy"),
            );

        // Verify record was added
        let record_opt = client.get_record(&patient, &record_id);
        assert!(record_opt.is_some());
        let record = record_opt.unwrap();
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
                &vec![String::from_str(&env, "herbal")],
                String::from_str(&env, "Traditional"),
                String::from_str(&env, "Herbal Therapy"),
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

    fn setup_default_workflow(env: &Env) -> (Address, MedicalRecordsContractClient<'_>, Address, Address, Address) {
        let contract_id = env.register_contract(None, MedicalRecordsContract);
        let client = MedicalRecordsContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let doctor = Address::generate(&env);
        let patient = Address::generate(&env);

        client.mock_all_auths().initialize(&admin);
        client.mock_all_auths().manage_user(&admin, &doctor, &Role::Doctor);
        client.mock_all_auths().manage_user(&admin, &patient, &Role::Patient);

        (contract_id, client, admin, doctor, patient)
    }

    fn add_sample_record(
        env: &Env,
        contract_id: &Address,
        client: &MedicalRecordsContractClient<'_>,
        doctor: &Address,
        patient: &Address,
    ) -> u64 {
        client
            .mock_auths(&[MockAuth {
                address: doctor,
                invoke: &MockAuthInvoke {
                    contract: contract_id,
                    fn_name: "add_record",
                    args: (),
                    sub_invokes: &[],
                },
            }])
            .add_record(
                doctor,
                patient,
                &String::from_str(env, "Test Diagnosis"),
                &String::from_str(env, "Test Treatment"),
                &false,
                &vec![String::from_str(env, "tag")],
                String::from_str(env, "Modern"),
                String::from_str(env, "General"),
            )
    }

    #[test]
    fn test_admin_only_manage_user_integration() {
        let env = Env::default();
        let (_contract_id, client, admin, doctor, patient) = setup_default_workflow(&env);

        let unauthorized_result = client
            .mock_all_auths()
            .try_manage_user(&doctor, &Address::generate(&env), &Role::Patient);
        assert_eq!(unauthorized_result, Err(Ok(Error::NotAuthorized)));

        // Admin must still be able to manage roles.
        let valid_result = client
            .mock_all_auths()
            .manage_user(&admin, &Address::generate(&env), &Role::Patient);
        assert!(valid_result.is_ok());
    }

    #[test]
    fn test_unauthorized_record_access_integration() {
        let env = Env::default();
        let (contract_id, client, _admin, doctor, patient) = setup_default_workflow(&env);

        let record_id = add_sample_record(&env, &contract_id, &client, &doctor, &patient);
        let stranger = Address::generate(&env);

        let unauthorized_access = client
            .mock_all_auths()
            .try_get_record(&stranger, &record_id);
        assert_eq!(unauthorized_access, Err(Ok(Error::NotAuthorized)));

        let patient_access = client.mock_all_auths().get_record(&patient, &record_id);
        assert_eq!(patient_access.patient_id, patient);
    }

    #[test]
    fn test_encryption_required_prevents_add_record() {
        let env = Env::default();
        let (_contract_id, client, admin, doctor, patient) = setup_default_workflow(&env);

        assert!(client
            .mock_all_auths()
            .set_encryption_required(&admin, &true)
            .unwrap());

        let result = client.mock_all_auths().try_add_record(
            &doctor,
            &patient,
            &String::from_str(&env, "Diag"),
            &String::from_str(&env, "Treat"),
            &false,
            &vec![String::from_str(&env, "tag")],
            String::from_str(&env, "Modern"),
            String::from_str(&env, "General"),
        );
        assert_eq!(result, Err(Ok(Error::EncryptionRequired)));
    }

    #[test]
    fn test_zk_enforcement_blocks_without_grant() {
        let env = Env::default();
        let (contract_id, client, admin, doctor, patient) = setup_default_workflow(&env);
        let record_id = add_sample_record(&env, &contract_id, &client, &doctor, &patient);

        assert!(client.mock_all_auths().set_zk_enforced(&admin, &true).unwrap());
        assert!(!client.has_valid_zk_access_grant(&patient, &record_id));

        let res = client.mock_all_auths().try_get_record(&patient, &record_id);
        assert_eq!(res, Err(Ok(Error::InvalidCredential)));
    }

    #[test]
    fn test_emergency_access_grant_revoke_cycle() {
        let env = Env::default();
        let (contract_id, client, _admin, doctor, patient) = setup_default_workflow(&env);
        let record_id = add_sample_record(&env, &contract_id, &client, &doctor, &patient);

        assert!(client
            .mock_all_auths()
            .grant_emergency_access(&patient, &doctor, &3600, &vec![&env, record_id])
            .unwrap());

        assert!(client.has_emergency_access(&doctor, &patient, &record_id));

        assert!(client
            .mock_all_auths()
            .revoke_emergency_access(&patient, &doctor)
            .unwrap());

        assert!(!client.has_emergency_access(&doctor, &patient, &record_id));
    }

    #[test]
    fn test_update_record_metadata_and_history_integration() {
        let env = Env::default();
        let (contract_id, client, _admin, doctor, patient) = setup_default_workflow(&env);
        let record_id = add_sample_record(&env, &contract_id, &client, &doctor, &patient);

        assert!(client
            .mock_all_auths()
            .update_record_metadata(
                &doctor,
                &record_id,
                &vec![String::from_str(&env, "updated")],
                &soroban_sdk::Map::new(&env),
            )
            .is_ok());

        let history = client
            .mock_all_auths()
            .get_history(&doctor, &patient, &0u32, &10u32)
            .unwrap();
        assert!(!history.is_empty());
        assert_eq!(history[0].0, record_id);
    }

    #[test]
    fn test_rate_limit_enforcement_integration() {
        let env = Env::default();
        let (contract_id, client, admin, doctor, patient) = setup_default_workflow(&env);

        let config = RateLimitConfig {
            doctor_max_calls: 1,
            patient_max_calls: 0,
            admin_max_calls: 0,
            window_secs: 3600,
        };

        assert!(client
            .mock_all_auths()
            .set_rate_limit_config(&admin, &1u32, &config)
            .is_ok());

        let _ = add_sample_record(&env, &contract_id, &client, &doctor, &patient);
        let second_call = client.mock_all_auths().try_add_record(
            &doctor,
            &patient,
            &String::from_str(&env, "Diag 2"),
            &String::from_str(&env, "Treat 2"),
            &false,
            &vec![String::from_str(&env, "tag")],
            String::from_str(&env, "Modern"),
            String::from_str(&env, "General"),
        );

        assert_eq!(second_call, Err(Ok(Error::RateLimitExceeded)));
    }

    #[test]
    fn test_record_count_monotonicity_integration() {
        let env = Env::default();
        let (contract_id, client, _admin, doctor, patient) = setup_default_workflow(&env);

        let id1 = add_sample_record(&env, &contract_id, &client, &doctor, &patient);
        let id2 = add_sample_record(&env, &contract_id, &client, &doctor, &patient);

        assert!(id2 > id1);
        assert_eq!(client.get_record_count(), 2u64);
    }
}

// tests/unit/mod.rs
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