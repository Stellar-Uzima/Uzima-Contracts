#[cfg(test)]
mod tests {
    use soroban_sdk::testutils::{Address as AddressTestUtils, Signature};
    use soroban_sdk::{Address, Env, Map, String};

    use crate::types::LoggingConfig;
    use crate::HealthDataAccessLogging;

    fn create_test_env() -> Env {
        Env::default()
    }

    fn setup_contract(env: &Env) -> (Address, Address) {
        let admin = Address::random(&env);
        let config = LoggingConfig {
            max_logs_per_patient: 1000,
            allow_public_queries: false,
            retention_period: 0,
        };

        HealthDataAccessLogging::initialize(env.clone(), admin.clone(), config);

        (admin, admin)
    }

    #[test]
    fn test_initialize() {
        let env = create_test_env();
        let admin = Address::random(&env);
        let config = LoggingConfig {
            max_logs_per_patient: 1000,
            allow_public_queries: false,
            retention_period: 0,
        };

        HealthDataAccessLogging::initialize(env.clone(), admin.clone(), config.clone());

        // Verify configuration was set
        let stored_config = HealthDataAccessLogging::get_config(env.clone());
        assert_eq!(stored_config.max_logs_per_patient, 1000);
    }

    #[test]
    #[should_panic(expected = "Contract already initialized")]
    fn test_initialize_twice_fails() {
        let env = create_test_env();
        let admin = Address::random(&env);
        let config = LoggingConfig {
            max_logs_per_patient: 1000,
            allow_public_queries: false,
            retention_period: 0,
        };

        HealthDataAccessLogging::initialize(env.clone(), admin.clone(), config.clone());
        HealthDataAccessLogging::initialize(env.clone(), admin.clone(), config.clone());
    }

    #[test]
    fn test_log_access() {
        let env = create_test_env();
        let (admin, _) = setup_contract(&env);

        let patient = Address::random(&env);
        let accessor = Address::random(&env);

        // Create metadata for the access
        let metadata = Map::from_array(
            &env,
            &[(
                String::from_slice(&env, "reason"),
                String::from_slice(&env, "consultation"),
            )],
        );

        env.mock_auths(&[Signature::Invoker]);

        // Manually authorize accessor to call log_access
        let log_id = HealthDataAccessLogging::log_access(
            env.clone(),
            patient.clone(),
            accessor.clone(),
            String::from_slice(&env, "read"),
            metadata,
        );

        // Verify log was created with ID 0
        assert_eq!(log_id, 0);
    }

    #[test]
    fn test_get_access_logs() {
        let env = create_test_env();
        let (_admin, _) = setup_contract(&env);

        let patient = Address::random(&env);
        let accessor1 = Address::random(&env);
        let accessor2 = Address::random(&env);

        env.mock_auths(&[Signature::Invoker]);

        // Log multiple accesses
        let _log_id_1 = HealthDataAccessLogging::log_access(
            env.clone(),
            patient.clone(),
            accessor1.clone(),
            String::from_slice(&env, "read"),
            Map::new(&env),
        );

        let _log_id_2 = HealthDataAccessLogging::log_access(
            env.clone(),
            patient.clone(),
            accessor2.clone(),
            String::from_slice(&env, "write"),
            Map::new(&env),
        );

        env.mock_auths(&[Signature::Invoker]);

        // Retrieve logs for the patient
        let logs = HealthDataAccessLogging::get_access_logs(env.clone(), patient.clone());

        // Verify both logs are returned
        assert_eq!(logs.len(), 2);
        assert_eq!(logs.get_unchecked(0).accessor_address, accessor1);
        assert_eq!(logs.get_unchecked(1).accessor_address, accessor2);
    }

    #[test]
    fn test_logs_immutability() {
        let env = create_test_env();
        let (_admin, _) = setup_contract(&env);

        let patient = Address::random(&env);
        let accessor = Address::random(&env);

        env.mock_auths(&[Signature::Invoker]);

        // Log an access
        let log_id_1 = HealthDataAccessLogging::log_access(
            env.clone(),
            patient.clone(),
            accessor.clone(),
            String::from_slice(&env, "read"),
            Map::new(&env),
        );

        // Get the first access log
        let logs_1 = HealthDataAccessLogging::get_access_logs(env.clone(), patient.clone());
        let first_log_hash = logs_1.get_unchecked(0).entry_hash.clone();

        env.mock_auths(&[Signature::Invoker]);

        // Log another access to the same patient
        let _log_id_2 = HealthDataAccessLogging::log_access(
            env.clone(),
            patient.clone(),
            accessor.clone(),
            String::from_slice(&env, "write"),
            Map::new(&env),
        );

        // Retrieve logs again
        let logs_2 = HealthDataAccessLogging::get_access_logs(env.clone(), patient.clone());

        // The first log should be unchanged
        let first_log_hash_after = logs_2.get_unchecked(0).entry_hash.clone();
        assert_eq!(first_log_hash, first_log_hash_after);
    }

    #[test]
    fn test_get_latest_access_logs() {
        let env = create_test_env();
        let (_admin, _) = setup_contract(&env);

        let patient = Address::random(&env);
        let accessor = Address::random(&env);

        env.mock_auths(&[Signature::Invoker]);

        // Create 5 access logs
        for i in 0..5 {
            let _reason = format!("access_{}", i);
            let _log_id = HealthDataAccessLogging::log_access(
                env.clone(),
                patient.clone(),
                accessor.clone(),
                String::from_slice(&env, "read"),
                Map::new(&env),
            );
        }

        env.mock_auths(&[Signature::Invoker]);

        // Get latest 3 logs
        let latest_logs =
            HealthDataAccessLogging::get_latest_access_logs(env.clone(), patient.clone(), 3);

        // Should return exactly 3 logs
        assert_eq!(latest_logs.len(), 3);
    }

    #[test]
    fn test_get_logs_by_accessor() {
        let env = create_test_env();
        let (_admin, _) = setup_contract(&env);

        let patient = Address::random(&env);
        let accessor1 = Address::random(&env);
        let accessor2 = Address::random(&env);

        env.mock_auths(&[Signature::Invoker]);

        // Log from accessor1
        let _log_id_1 = HealthDataAccessLogging::log_access(
            env.clone(),
            patient.clone(),
            accessor1.clone(),
            String::from_slice(&env, "read"),
            Map::new(&env),
        );

        // Log from accessor2
        let _log_id_2 = HealthDataAccessLogging::log_access(
            env.clone(),
            patient.clone(),
            accessor2.clone(),
            String::from_slice(&env, "write"),
            Map::new(&env),
        );

        // Log from accessor1 again
        let _log_id_3 = HealthDataAccessLogging::log_access(
            env.clone(),
            patient.clone(),
            accessor1.clone(),
            String::from_slice(&env, "export"),
            Map::new(&env),
        );

        env.mock_auths(&[Signature::Invoker]);

        // Get logs from accessor1
        let accessor1_logs = HealthDataAccessLogging::get_logs_by_accessor(
            env.clone(),
            patient.clone(),
            accessor1.clone(),
        );

        // Should return 2 logs from accessor1
        assert_eq!(accessor1_logs.len(), 2);

        // Get logs from accessor2
        let accessor2_logs = HealthDataAccessLogging::get_logs_by_accessor(
            env.clone(),
            patient.clone(),
            accessor2.clone(),
        );

        // Should return 1 log from accessor2
        assert_eq!(accessor2_logs.len(), 1);
    }

    #[test]
    fn test_get_unique_accessors_count() {
        let env = create_test_env();
        let (_admin, _) = setup_contract(&env);

        let patient = Address::random(&env);
        let accessor1 = Address::random(&env);
        let accessor2 = Address::random(&env);
        let accessor3 = Address::random(&env);

        env.mock_auths(&[Signature::Invoker]);

        // Log from three different accessors
        let _log_id_1 = HealthDataAccessLogging::log_access(
            env.clone(),
            patient.clone(),
            accessor1.clone(),
            String::from_slice(&env, "read"),
            Map::new(&env),
        );

        let _log_id_2 = HealthDataAccessLogging::log_access(
            env.clone(),
            patient.clone(),
            accessor2.clone(),
            String::from_slice(&env, "write"),
            Map::new(&env),
        );

        let _log_id_3 = HealthDataAccessLogging::log_access(
            env.clone(),
            patient.clone(),
            accessor3.clone(),
            String::from_slice(&env, "read"),
            Map::new(&env),
        );

        env.mock_auths(&[Signature::Invoker]);

        // Get unique accessor count
        let count =
            HealthDataAccessLogging::get_unique_accessors_count(env.clone(), patient.clone());

        // Should be 3 unique accessors
        assert_eq!(count, 3);
    }

    #[test]
    fn test_get_access_log_summary() {
        let env = create_test_env();
        let (_admin, _) = setup_contract(&env);

        let patient = Address::random(&env);
        let accessor1 = Address::random(&env);
        let accessor2 = Address::random(&env);

        env.mock_auths(&[Signature::Invoker]);

        // Log some accesses
        let _log_id_1 = HealthDataAccessLogging::log_access(
            env.clone(),
            patient.clone(),
            accessor1.clone(),
            String::from_slice(&env, "read"),
            Map::new(&env),
        );

        let _log_id_2 = HealthDataAccessLogging::log_access(
            env.clone(),
            patient.clone(),
            accessor2.clone(),
            String::from_slice(&env, "write"),
            Map::new(&env),
        );

        env.mock_auths(&[Signature::Invoker]);

        // Get summary
        let summary = HealthDataAccessLogging::get_access_log_summary(env.clone(), patient.clone());

        // Verify summary contains correct data
        assert_eq!(summary.total_accesses, 2);
        assert_eq!(summary.unique_accessors_count, 2);
        assert!(summary.first_access_timestamp > 0);
        assert!(summary.last_access_timestamp > 0);
    }

    #[test]
    fn test_verify_logs_integrity() {
        let env = create_test_env();
        let (_admin, _) = setup_contract(&env);

        let patient = Address::random(&env);
        let accessor = Address::random(&env);

        env.mock_auths(&[Signature::Invoker]);

        // Get initial hash
        let initial_hash = HealthDataAccessLogging::verify_logs_integrity(env.clone());

        // Log an access
        let _log_id = HealthDataAccessLogging::log_access(
            env.clone(),
            patient.clone(),
            accessor.clone(),
            String::from_slice(&env, "read"),
            Map::new(&env),
        );

        // Get hash after logging
        let hash_after_log = HealthDataAccessLogging::verify_logs_integrity(env.clone());

        // Hashes should be different (rolling hash updated)
        assert_ne!(initial_hash, hash_after_log);
    }

    #[test]
    fn test_update_config() {
        let env = create_test_env();
        let (admin, _) = setup_contract(&env);

        let new_config = LoggingConfig {
            max_logs_per_patient: 500,
            allow_public_queries: true,
            retention_period: 86400,
        };

        env.mock_auths(&[Signature::Invoker]);

        // Update config
        HealthDataAccessLogging::update_config(env.clone(), new_config.clone());

        // Verify config was updated
        let stored_config = HealthDataAccessLogging::get_config(env.clone());
        assert_eq!(stored_config.max_logs_per_patient, 500);
        assert_eq!(stored_config.allow_public_queries, true);
        assert_eq!(stored_config.retention_period, 86400);
    }

    #[test]
    fn test_get_unique_accessors() {
        let env = create_test_env();
        let (_admin, _) = setup_contract(&env);

        let patient = Address::random(&env);
        let accessor1 = Address::random(&env);
        let accessor2 = Address::random(&env);

        env.mock_auths(&[Signature::Invoker]);

        // Log from two different accessors
        let _log_id_1 = HealthDataAccessLogging::log_access(
            env.clone(),
            patient.clone(),
            accessor1.clone(),
            String::from_slice(&env, "read"),
            Map::new(&env),
        );

        let _log_id_2 = HealthDataAccessLogging::log_access(
            env.clone(),
            patient.clone(),
            accessor2.clone(),
            String::from_slice(&env, "write"),
            Map::new(&env),
        );

        env.mock_auths(&[Signature::Invoker]);

        // Get unique accessors
        let accessors = HealthDataAccessLogging::get_unique_accessors(env.clone(), patient.clone());

        // Should return both accessors
        assert_eq!(accessors.len(), 2);
    }

    #[test]
    fn test_get_access_logs_in_range() {
        let env = create_test_env();
        let (_admin, _) = setup_contract(&env);

        let patient = Address::random(&env);
        let accessor = Address::random(&env);

        env.mock_auths(&[Signature::Invoker]);

        // Get current timestamp
        let start_time = env.ledger().timestamp();

        // Log an access
        let _log_id = HealthDataAccessLogging::log_access(
            env.clone(),
            patient.clone(),
            accessor.clone(),
            String::from_slice(&env, "read"),
            Map::new(&env),
        );

        let end_time = env.ledger().timestamp() + 1000;

        env.mock_auths(&[Signature::Invoker]);

        // Get logs in range
        let logs_in_range = HealthDataAccessLogging::get_access_logs_in_range(
            env.clone(),
            patient.clone(),
            start_time,
            end_time,
        );

        // Should return 1 log
        assert_eq!(logs_in_range.len(), 1);
    }
}
