#[cfg(test)]
mod tests {
    use crate::{
        Error, ProposalStatus, ProposalType, TreasuryConfig, TreasuryController,
        TreasuryControllerClient,
    };
    use soroban_sdk::testutils::{Address as _, Ledger as _};
    use soroban_sdk::{Address, Bytes, Env, String, Vec};

    fn setup() -> (
        Env,
        TreasuryControllerClient<'static>,
        Address,
        Vec<Address>,
    ) {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_000_000);

        let admin = Address::generate(&env);
        let signer1 = Address::generate(&env);
        let signer2 = Address::generate(&env);
        let signer3 = Address::generate(&env);
        let signers = Vec::from_array(&env, [signer1.clone(), signer2.clone(), signer3.clone()]);

        let contract_id = env.register_contract(None, TreasuryController);
        let client = TreasuryControllerClient::new(&env, &contract_id);

        client.initialize(
            &admin,
            &signers,
            &2u32,          // threshold
            &3600u64,       // 1 hour timelock
            &2u32,          // emergency threshold (must be <= threshold)
            &1_000_000i128, // max withdrawal
        );

        (env, client, admin, signers)
    }

    // ============================================================================
    // INITIALIZATION TESTS
    // ============================================================================

    #[test]
    fn test_initialize() {
        let (_env, _client, _admin, signers) = setup();
        assert_eq!(signers.len(), 3);
    }

    #[test]
    fn test_double_initialize() {
        let (env, client, admin, _signers) = setup();

        let result = client.try_initialize(
            &admin,
            &Vec::from_array(&env, [Address::generate(&env)]),
            &1u32,
            &3600u64,
            &1u32,
            &1_000_000i128,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_initialize_invalid_threshold() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);

        let contract_id = env.register_contract(None, TreasuryController);
        let client = TreasuryControllerClient::new(&env, &contract_id);

        // Threshold > number of signers should fail
        let result = client.try_initialize(
            &admin,
            &Vec::from_array(&env, [Address::generate(&env)]),
            &5u32, // threshold > 1 signer
            &3600u64,
            &1u32,
            &1_000_000i128,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_initialize_invalid_timelock() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);

        let contract_id = env.register_contract(None, TreasuryController);
        let client = TreasuryControllerClient::new(&env, &contract_id);

        // Timelock below minimum
        let result = client.try_initialize(
            &admin,
            &Vec::from_array(&env, [Address::generate(&env)]),
            &1u32,
            &100u64, // < 3600 minimum
            &1u32,
            &1_000_000i128,
        );
        assert!(result.is_err());
    }

    // ============================================================================
    // SUPPORTED TOKEN TESTS
    // ============================================================================

    #[test]
    fn test_add_supported_token() {
        let (_env, client, _admin, _signers) = setup();
        let token = Address::generate(&client.env);

        assert!(client.try_add_supported_token(&token).is_ok());
    }

    // ============================================================================
    // PROPOSAL TESTS
    // ============================================================================

    #[test]
    fn test_create_proposal() {
        let (env, client, _admin, signers) = setup();
        let signer = signers.get(0).unwrap();
        let token = Address::generate(&env);

        client.add_supported_token(&token);

        let proposal_id = client.create_proposal(
            &signer,
            &ProposalType::Withdrawal,
            &Address::generate(&env),
            &token,
            &1000,
            &String::from_str(&env, "Test withdrawal"),
            &String::from_str(&env, ""),
            &Bytes::from_array(&env, &[0u8; 32]),
        );
        assert!(proposal_id > 0);
    }

    #[test]
    fn test_create_proposal_not_signer() {
        let (env, client, _admin, _signers) = setup();
        let non_signer = Address::generate(&env);
        let token = Address::generate(&env);

        client.add_supported_token(&token);

        let result = client.try_create_proposal(
            &non_signer,
            &ProposalType::Withdrawal,
            &Address::generate(&env),
            &token,
            &1000,
            &String::from_str(&env, "Unauthorized"),
            &String::from_str(&env, ""),
            &Bytes::from_array(&env, &[0u8; 32]),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_create_proposal_exceeds_max_withdrawal() {
        let (env, client, _admin, signers) = setup();
        let signer = signers.get(0).unwrap();
        let token = Address::generate(&env);

        client.add_supported_token(&token);

        let result = client.try_create_proposal(
            &signer,
            &ProposalType::Withdrawal,
            &Address::generate(&env),
            &token,
            &2_000_000i128, // > max 1_000_000
            &String::from_str(&env, "Too much"),
            &String::from_str(&env, ""),
            &Bytes::from_array(&env, &[0u8; 32]),
        );
        assert!(result.is_err());
    }

    // ============================================================================
    // APPROVAL TESTS
    // ============================================================================

    #[test]
    fn test_approve_proposal() {
        let (env, client, _admin, signers) = setup();
        let signer1 = signers.get(0).unwrap();
        let signer2 = signers.get(1).unwrap();
        let token = Address::generate(&env);

        client.add_supported_token(&token);

        let proposal_id = client.create_proposal(
            &signer1,
            &ProposalType::Withdrawal,
            &Address::generate(&env),
            &token,
            &1000,
            &String::from_str(&env, "Test"),
            &String::from_str(&env, ""),
            &Bytes::from_array(&env, &[0u8; 32]),
        );

        assert!(client.try_approve_proposal(&signer2, &proposal_id).is_ok());
    }

    #[test]
    fn test_approve_proposal_not_signer() {
        let (env, client, _admin, signers) = setup();
        let signer1 = signers.get(0).unwrap();
        let non_signer = Address::generate(&env);
        let token = Address::generate(&env);

        client.add_supported_token(&token);

        let proposal_id = client.create_proposal(
            &signer1,
            &ProposalType::Withdrawal,
            &Address::generate(&env),
            &token,
            &1000,
            &String::from_str(&env, "Test"),
            &String::from_str(&env, ""),
            &Bytes::from_array(&env, &[0u8; 32]),
        );

        let result = client.try_approve_proposal(&non_signer, &proposal_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_approve_nonexistent_proposal() {
        let (_env, client, _admin, signers) = setup();
        let signer = signers.get(0).unwrap();

        let result = client.try_approve_proposal(&signer, &999);
        assert!(result.is_err());
    }

    #[test]
    fn test_double_approve() {
        let (env, client, _admin, signers) = setup();
        let signer1 = signers.get(0).unwrap();
        let token = Address::generate(&env);

        client.add_supported_token(&token);

        let proposal_id = client.create_proposal(
            &signer1,
            &ProposalType::Withdrawal,
            &Address::generate(&env),
            &token,
            &1000,
            &String::from_str(&env, "Test"),
            &String::from_str(&env, ""),
            &Bytes::from_array(&env, &[0u8; 32]),
        );

        // First approval by signer1 should succeed
        assert!(client.try_approve_proposal(&signer1, &proposal_id).is_ok());
        // Second approval by the same signer should fail (AlreadyApproved)
        let result = client.try_approve_proposal(&signer1, &proposal_id);
        assert!(result.is_err());
    }

    // ============================================================================
    // EXECUTION TESTS
    // ============================================================================

    #[test]
    fn test_execute_proposal_timelock_not_expired() {
        let (env, client, _admin, signers) = setup();
        let signer1 = signers.get(0).unwrap();
        let signer2 = signers.get(1).unwrap();
        let token = Address::generate(&env);

        client.add_supported_token(&token);

        let proposal_id = client.create_proposal(
            &signer1,
            &ProposalType::ConfigChange, // Non-withdrawal to avoid token transfer
            &Address::generate(&env),
            &token,
            &0,
            &String::from_str(&env, "Config change"),
            &String::from_str(&env, ""),
            &Bytes::from_array(&env, &[0u8; 32]),
        );

        let _ = client
            .try_approve_proposal(&signer2, &proposal_id)
            .unwrap_or(Ok(()));

        // Timelock hasn't expired yet
        let result = client.try_execute_proposal(&signer1, &proposal_id);
        assert!(result.is_err());
    }

    // ============================================================================
    // EMERGENCY TESTS
    // ============================================================================

    #[test]
    fn test_emergency_halt() {
        let (_env, client, _admin, signers) = setup();
        let signer = signers.get(0).unwrap();

        assert!(client.try_emergency_halt(&signer).is_ok());
    }

    #[test]
    fn test_emergency_halt_unauthorized() {
        let (_env, client, _admin, _signers) = setup();
        let non_signer = Address::generate(&client.env);

        let result = client.try_emergency_halt(&non_signer);
        assert!(result.is_err());
    }

    #[test]
    fn test_resume_operations() {
        let (_env, client, admin, _signers) = setup();

        // Admin halts
        let _ = client.try_emergency_halt(&admin).unwrap_or(Ok(()));
        // Admin resumes
        assert!(client.try_resume_operations(&admin).is_ok());
    }

    #[test]
    fn test_resume_operations_unauthorized() {
        let (_env, client, admin, signers) = setup();
        let signer = signers.get(0).unwrap();

        let _ = client.try_emergency_halt(&admin).unwrap_or(Ok(()));
        let result = client.try_resume_operations(&signer); // Only admin can resume
        assert!(result.is_err());
    }

    // ============================================================================
    // VIEW FUNCTION TESTS
    // ============================================================================

    #[test]
    fn test_get_config() {
        let (_env, client, _admin, _signers) = setup();
        let config: TreasuryConfig = client.get_config();
        assert_eq!(config.multisig_config.threshold, 2);
        assert_eq!(config.max_withdrawal_amount, 1_000_000);
    }

    #[test]
    fn test_get_proposal_count() {
        let (env, client, _admin, signers) = setup();
        let signer = signers.get(0).unwrap();
        let token = Address::generate(&env);

        client.add_supported_token(&token);

        let count = client.get_proposal_count();
        assert_eq!(count, 0);

        client.create_proposal(
            &signer,
            &ProposalType::ConfigChange,
            &Address::generate(&env),
            &token,
            &0,
            &String::from_str(&env, "Prop 1"),
            &String::from_str(&env, ""),
            &Bytes::from_array(&env, &[0u8; 32]),
        );

        assert_eq!(client.get_proposal_count(), 1);
    }

    #[test]
    fn test_gnosis_get_threshold() {
        let (_env, client, _admin, _signers) = setup();
        assert_eq!(client.gnosis_get_threshold(), 2);
    }

    #[test]
    fn test_gnosis_get_owners() {
        let (_env, client, _admin, _signers) = setup();
        let owners = client.gnosis_get_owners();
        assert_eq!(owners.len(), 3);
    }

    #[test]
    fn test_is_proposal_executable_no_config() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, TreasuryController);
        let client = TreasuryControllerClient::new(&env, &contract_id);

        // Without initialization, should return false
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
}
