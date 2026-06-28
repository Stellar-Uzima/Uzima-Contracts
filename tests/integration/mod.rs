pub mod cross_chain_tests;
pub mod healthcare_workflows;
pub mod ihe_fhir_integration_tests;
pub mod medical_records_tests;
pub mod multi_region_dr_integration;

/// Basic environment integration tests
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
        // In Soroban test env, timestamp starts at 0; sequence is 0
        assert_eq!(env.ledger().timestamp(), 0);
        // Ledger sequence is also 0 in the default test env
        assert_eq!(env.ledger().sequence(), 0);
    }
}
