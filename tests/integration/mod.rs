pub mod ihe_fhir_integration_tests;
pub mod framework_tests;
pub mod multi_region_dr_integration;

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