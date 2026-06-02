#[cfg(test)]
mod tests {
    use crate::{
        Error, ProposalStatus, ProposalType, TreasuryController, TreasuryControllerClient,
    };
    use soroban_sdk::testutils::{Address as _, Ledger as _};
    use soroban_sdk::{Address, Bytes, Env, String, Vec};

    fn setup() -> (Env, TreasuryControllerClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_000_000);

        let admin = Address::generate(&env);
        let signer1 = Address::generate(&env);
        let signer2 = Address::generate(&env);
        let signers = Vec::from_array(&env, [signer1.clone(), signer2.clone()]);

        let contract_id = env.register_contract(None, TreasuryController);
        let client = TreasuryControllerClient::new(&env, &contract_id);

        client.initialize(&admin, &signers, &2u32, &3600u64, &2u32, &1_000_000i128);

        (env, client, admin)
    }

    // ============================================================================
    // INITIALIZATION TESTS
    // ============================================================================

    #[test]
    fn test_initialize() {
        let (_env, _client, _admin) = setup();
    }

    #[test]
    fn test_double_initialize() {
        let (env, client, admin) = setup();
        let result = client.try_initialize(
            &admin,
            &Vec::from_array(&env, [Address::generate(&env)]),
            &1u32, &3600u64, &1u32, &1_000_000i128,
        );
        assert!(result.is_err());
    }

    // ============================================================================
    // VIEW FUNCTION TESTS (updated for Result return types)
    // ============================================================================

    #[test]
    fn test_get_config_returns_result() {
        let (_env, client, _admin) = setup();
        let result = client.try_get_config();
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_config_not_initialized() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, TreasuryController);
        let client = TreasuryControllerClient::new(&env, &contract_id);
        let result = client.try_get_config();
        assert!(result.is_err());
    }

    #[test]
    fn test_get_proposal_count() {
        let (_env, client, _admin) = setup();
        let count = client.get_proposal_count();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_gnosis_get_threshold_returns_result() {
        let (_env, client, _admin) = setup();
        let result = client.try_gnosis_get_threshold();
        assert!(result.is_ok());
    }

    #[test]
    fn test_gnosis_get_owners_returns_result() {
        let (_env, client, _admin) = setup();
        let result = client.try_gnosis_get_owners();
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_proposal_executable_no_config() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, TreasuryController);
        let client = TreasuryControllerClient::new(&env, &contract_id);
        assert!(!client.is_proposal_executable(&1));
    }

    // ============================================================================
    // ERROR CODES TESTS
    // ============================================================================

    #[test]
    fn test_error_types_exist() {
        let _error = Error::NotInitialized;
        let _error = Error::AlreadyInitialized;
        let _error = Error::InvalidThreshold;
        let _error = Error::NotSigner;
        let _error = Error::ProposalNotFound;
        let _error = Error::NotPending;
        let _error = Error::TransferFailed;
        let _error = Error::ConfigNotFound;
    }

    #[test]
    fn test_proposal_types_exist() {
        let _withdrawal = ProposalType::Withdrawal;
        let _config_change = ProposalType::ConfigChange;
        let _emergency_halt = ProposalType::EmergencyHalt;
    }

    #[test]
    fn test_proposal_status_types() {
        let _pending = ProposalStatus::Pending;
        let _approved = ProposalStatus::Approved;
        let _executed = ProposalStatus::Executed;
        let _rejected = ProposalStatus::Rejected;
        let _expired = ProposalStatus::Expired;
    }

    #[test]
    fn test_error_code_values_stable() {
        assert_eq!(Error::NotInitialized as u32, 1);
        assert_eq!(Error::ConfigNotFound as u32, 15);
    }
}
