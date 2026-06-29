//! End-to-end smoke tests for testnet deployment verification.
//!
//! Tests deploy contracts in a local Soroban environment and verify basic
//! CRUD operations through their public entrypoints.

#[cfg(test)]
mod e2e_smoke {
    use soroban_sdk::{
        testutils::Address as _, vec, Address, Env, String, Vec,
    };
    use medical_records::{
        MedicalRecordsContract, MedicalRecordsContractClient, MockRbac, MockRbacClient,
        RbacRole, Role,
    };

    struct SmokeEnv {
        _env: Env,
        admin: Address,
        doctor: Address,
        patient: Address,
        client: MedicalRecordsContractClient<'static>,
    }

    fn setup() -> SmokeEnv {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let rbac_id = env.register_contract(None, MockRbac);
        let rbac_client = MockRbacClient::new(&env, &rbac_id);
        rbac_client.assign_role(&admin, &RbacRole::Admin);

        let contract_id = Address::generate(&env);
        env.register_contract(&contract_id, MedicalRecordsContract);
        let client = MedicalRecordsContractClient::new(&env, &contract_id);
        client.initialize(&admin, &rbac_id);

        let doctor = Address::generate(&env);
        let patient = Address::generate(&env);
        client.manage_user(&admin, &doctor, &Role::Doctor);
        client.manage_user(&admin, &patient, &Role::Patient);

        SmokeEnv { _env: env, admin, doctor, patient, client }
    }

    /// Deploy contracts and verify initialization succeeds.
    #[test]
    fn test_contracts_deploy_and_initialize() {
        let smoke = setup();
        assert_ne!(smoke.admin, smoke.doctor);
        assert_ne!(smoke.doctor, smoke.patient);
    }

    /// Perform basic CRUD: add a record then retrieve it.
    #[test]
    fn test_medical_record_write_and_read() {
        let smoke = setup();
        let record_id = smoke.client.add_record(
            &smoke.doctor,
            &smoke.patient,
            &String::from_str(&smoke._env, "Smoke Diagnosis"),
            &String::from_str(&smoke._env, "Smoke Treatment"),
            &false,
            &vec![&smoke._env, String::from_str(&smoke._env, "smoke")],
            &String::from_str(&smoke._env, "General"),
            &String::from_str(&smoke._env, "Medication"),
            &String::from_str(&smoke._env, "ipfs://smoke-record"),
        );
        let record = smoke.client.get_record(&smoke.patient, &record_id);
        assert_eq!(
            record.diagnosis,
            String::from_str(&smoke._env, "Smoke Diagnosis")
        );
    }

    /// Verify that an unauthorized caller cannot write records.
    #[test]
    fn test_unauthorized_write_rejected() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let rbac_id = env.register_contract(None, MockRbac);
        let contract_id = Address::generate(&env);
        env.register_contract(&contract_id, MedicalRecordsContract);
        let client = MedicalRecordsContractClient::new(&env, &contract_id);
        client.initialize(&admin, &rbac_id);

        let stranger = Address::generate(&env);
        let result = client.try_add_record(
            &stranger,
            &Address::generate(&env),
            &String::from_str(&env, "Unauthorized"),
            &String::from_str(&env, "Test"),
            &false,
            &Vec::new(&env),
            &String::from_str(&env, "General"),
            &String::from_str(&env, "Medication"),
            &String::from_str(&env, "ipfs://unauth"),
        );
        assert!(result.is_err(), "Unauthorized write must be rejected");
    }

    /// Verify record count tracking.
    #[test]
    fn test_record_count_tracking() {
        let smoke = setup();
        let count_before = smoke.client.get_patient_record_count(&smoke.patient);
        assert_eq!(count_before, 0, "New patient should have zero records");

        smoke.client.add_record(
            &smoke.doctor,
            &smoke.patient,
            &String::from_str(&smoke._env, "Count Test"),
            &String::from_str(&smoke._env, "Treatment"),
            &false,
            &Vec::new(&smoke._env),
            &String::from_str(&smoke._env, "General"),
            &String::from_str(&smoke._env, "Medication"),
            &String::from_str(&smoke._env, "ipfs://count"),
        );

        let count_after = smoke.client.get_patient_record_count(&smoke.patient);
        assert_eq!(count_after, 1, "Record count should increment");
    }

    /// Verify the contract health check returns OK.
    #[test]
    fn test_health_check() {
        let smoke = setup();
        let (status, _version, _gas) = smoke.client.health_check();
        assert_eq!(status, symbol_short!("OK"));
    }
}
