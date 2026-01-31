#[cfg(test)]
mod tests {
    use medical_records::{MedicalRecordsContract, MedicalRecordsContractClient, Role};
    use soroban_sdk::testutils::{Address as _, AuthorizedFunction, MockAuth, MockAuthInvoke};
    use soroban_sdk::{vec, Address, Env, String};

pub mod property_based_tests;
pub mod performance_tests;

    #[test]
    fn test_initialize_and_roles() {
        let env = Env::default();
        let contract_id = env.register_contract(None, MedicalRecordsContract);
        let client = MedicalRecordsContractClient::new(&env, &contract_id);

        // Create test addresses
        let admin = Address::generate(&env);
        let doctor = Address::generate(&env);
        let patient = Address::generate(&env);

        // Initialize contract with admin
        assert!(client
            .mock_all_auths()
            .initialize(&admin));

        // Admin adds a doctor
        assert!(client
            .mock_all_auths()
            .manage_user(&admin, &doctor, &Role::Doctor));

        // Admin adds a patient
        assert!(client
            .mock_all_auths()
            .manage_user(&admin, &patient, &Role::Patient));

        // Doctor tries to add another doctor (should fail)
        let result = client
            .mock_all_auths()
            .try_manage_user(&doctor, &Address::generate(&env), &Role::Doctor);
        assert!(result.is_err());
    }

    #[test]
    fn test_medical_record_access() {
        let env = Env::default();
        let contract_id = env.register_contract(None, MedicalRecordsContract);
        let client = MedicalRecordsContractClient::new(&env, &contract_id);

        // Setup users
        let admin = Address::generate(&env);
        let doctor1 = Address::generate(&env);
        let doctor2 = Address::generate(&env);
        let patient = Address::generate(&env);

        // Initialize and set up roles
        client.mock_all_auths().initialize(&admin);
        client.mock_all_auths().manage_user(&admin, &doctor1, &Role::Doctor);
        client.mock_all_auths().manage_user(&admin, &doctor2, &Role::Doctor);
        client.mock_all_auths().manage_user(&admin, &patient, &Role::Patient);

        // Doctor1 creates a confidential record
        let record_id = client
            .mock_all_auths()
            .add_record(
                &doctor1,
                &patient,
                &String::from_str(&env, "Diagnosis"),
                &String::from_str(&env, "Treatment"),
                &true,
                &vec![&env, String::from_str(&env, "herbal")],
                &String::from_str(&env, "Traditional"),
                &String::from_str(&env, "Herbal Therapy"),
                &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx"),
            );

        // Patient can access their record
        let patient_view = client
            .mock_all_auths()
            .get_record(&patient, &record_id);
        assert_eq!(patient_view.patient_id, patient);

        // Doctor1 (creator) can access the record
        let doctor1_view = client
            .mock_all_auths()
            .get_record(&doctor1, &record_id);
        assert_eq!(doctor1_view.doctor_id, doctor1);

        // Doctor2 cannot access confidential record
        let result = client
            .mock_all_auths()
            .try_get_record(&doctor2, &record_id);
        assert!(result.is_err());

        // Admin can access any record
        let admin_view = client
            .mock_all_auths()
            .get_record(&admin, &record_id);
        assert_eq!(admin_view.patient_id, patient);
    }

    #[test]
    fn test_user_deactivation() {
        let env = Env::default();
        let contract_id = env.register_contract(None, MedicalRecordsContract);
        let client = MedicalRecordsContractClient::new(&env, &contract_id);

        // Setup users
        let admin = Address::generate(&env);
        let doctor = Address::generate(&env);
        let patient = Address::generate(&env);

        // Initialize and set up roles
        client.mock_all_auths().initialize(&admin);
        client.mock_all_auths().manage_user(&admin, &doctor, &Role::Doctor);
        client.mock_all_auths().manage_user(&admin, &patient, &Role::Patient);

        // Doctor creates a record
        let _record_id = client
            .mock_all_auths()
            .add_record(
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

        // Deactivate doctor
        assert!(client
            .mock_all_auths()
            .deactivate_user(&admin, &doctor));

        // Deactivated doctor cannot create new records
        let result = client
            .mock_all_auths()
            .try_add_record(
                &doctor,
                &patient,
                &String::from_str(&env, "New Diagnosis"),
                &String::from_str(&env, "New Treatment"),
                &false,
                &vec![&env, String::from_str(&env, "herbal")],
                &String::from_str(&env, "Traditional"),
                &String::from_str(&env, "Herbal Therapy"),
                &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx"),
            );
        assert!(result.is_err());
    }
}