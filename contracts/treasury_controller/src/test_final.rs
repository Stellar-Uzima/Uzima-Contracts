#[cfg(test)]
pub mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        Address, Bytes, Env, String, Vec,
    };

    fn create_test_env() -> (Env, Address, Vec<Address>) {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let signer1 = Address::generate(&env);
        let signer2 = Address::generate(&env);
        let signer3 = Address::generate(&env);
        
        let signers = Vec::from_array(&env, [signer1, signer2, signer3]);
        
        (env, admin, signers)
    }

    fn setup_treasury_controller(env: &Env, admin: &Address, signers: &Vec<Address>) -> Address {
        let contract_id = env.register_contract(None, TreasuryController);
        let client = TreasuryControllerClient::new(env, &contract_id);

        // Initialize with 2-of-3 multisig, 1 hour timelock
        let _ = client.try_initialize(
            admin,
            signers,
            &2u32,
            &3600u64, // 1 hour
            &2u32, // Emergency threshold
            &1_000_000i128, // Max withdrawal
        );

        contract_id
    }

    #[test]
    fn test_initialization() {
        let (env, admin, signers) = create_test_env();
        let contract_id = env.register_contract(None, TreasuryController);
        let client = TreasuryControllerClient::new(&env, &contract_id);

        // Test successful initialization
        let result = client.try_initialize(
            &admin,
            &signers,
            &2u32,
            &3600u64,
            &2u32,
            &1_000_000i128,
        );
        assert!(result.is_ok());

        // Verify configuration
        let config = client.get_config();
        assert_eq!(config.admin, admin);
        assert_eq!(config.multisig_config.threshold, 2);
        assert_eq!(config.multisig_config.timelock_duration, 3600);
        assert_eq!(config.max_withdrawal_amount, 1_000_000);
        assert!(!config.emergency_halted);
    }

    #[test]
    fn test_add_supported_token() {
        let (env, admin, signers) = create_test_env();
        let contract_id = setup_treasury_controller(&env, &admin, &signers);
        let client = TreasuryControllerClient::new(&env, &contract_id);

        let token_address = Address::generate(&env);

        // Add supported token
        let result = client.try_add_supported_token(&token_address);
        assert!(result.is_ok());

        // Verify token was added
        let config = client.get_config();
        assert!(config.supported_tokens.contains(&token_address));
    }

    #[test]
    fn test_create_withdrawal_proposal() {
        let (env, admin, signers) = create_test_env();
        let contract_id = setup_treasury_controller(&env, &admin, &signers);
        let client = TreasuryControllerClient::new(&env, &contract_id);

        let token_address = Address::generate(&env);
        let target_address = Address::generate(&env);

        // Add supported token first
        client.try_add_supported_token(&token_address).unwrap();

        // Create withdrawal proposal
        let proposal_id = client.try_create_proposal(
            &signers.get(0).unwrap(),
            &ProposalType::Withdrawal,
            &target_address,
            &token_address,
            &500_000i128,
            &String::from_str(&env, "Development funding"),
            &String::from_str(&env, "Q1 budget allocation"),
            &Bytes::new(&env),
        ).unwrap();

        assert_eq!(proposal_id, 1);

        // Verify proposal
        let proposal = client.get_proposal(&proposal_id);
        assert_eq!(proposal.amount, 500_000);
        assert_eq!(proposal.target_address, target_address);
        assert!(matches!(proposal.status, ProposalStatus::Pending));
    }

    #[test]
    fn test_proposal_approval_flow() {
        let (env, admin, signers) = create_test_env();
        let contract_id = setup_treasury_controller(&env, &admin, &signers);
        let client = TreasuryControllerClient::new(&env, &contract_id);

        let token_address = Address::generate(&env);
        let target_address = Address::generate(&env);

        // Setup
        client.try_add_supported_token(&token_address).unwrap();
        
        let proposal_id = client.try_create_proposal(
            &signers.get(0).unwrap(),
            &ProposalType::Withdrawal,
            &target_address,
            &token_address,
            &500_000i128,
            &String::from_str(&env, "Development funding"),
            &String::from_str(&env, "Q1 budget allocation"),
            &Bytes::new(&env),
        ).unwrap();

        // First approval
        client.try_approve_proposal(&signers.get(0).unwrap(), &proposal_id).unwrap();
        let proposal = client.get_proposal(&proposal_id);
        assert!(matches!(proposal.status, ProposalStatus::Pending));
        assert_eq!(proposal.approvals.len(), 1);

        // Second approval should make it approved
        client.try_approve_proposal(&signers.get(1).unwrap(), &proposal_id).unwrap();
        let proposal = client.get_proposal(&proposal_id);
        assert!(matches!(proposal.status, ProposalStatus::Approved));
        assert_eq!(proposal.approvals.len(), 2);
    }

    #[test]
    fn test_proposal_execution_with_timelock() {
        let (env, admin, signers) = create_test_env();
        let contract_id = setup_treasury_controller(&env, &admin, &signers);
        let client = TreasuryControllerClient::new(&env, &contract_id);

        let token_address = Address::generate(&env);
        let target_address = Address::generate(&env);

        // Setup
        client.try_add_supported_token(&token_address).unwrap();
        
        let proposal_id = client.try_create_proposal(
            &signers.get(0).unwrap(),
            &ProposalType::Withdrawal,
            &target_address,
            &token_address,
            &500_000i128,
            &String::from_str(&env, "Development funding"),
            &String::from_str(&env, "Q1 budget allocation"),
            &Bytes::new(&env),
        ).unwrap();

        // Approve proposal
        client.try_approve_proposal(&signers.get(0).unwrap(), &proposal_id).unwrap();
        client.try_approve_proposal(&signers.get(1).unwrap(), &proposal_id).unwrap();

        // Try to execute before timelock expires (should fail)
        let result = client.try_execute_proposal(&signers.get(0).unwrap(), &proposal_id);
        assert!(result.is_err());

        // Advance time past timelock
        env.ledger().with_mut(|li| {
            li.timestamp = li.timestamp + 3700; // 1 hour + 100 seconds
        });

        // Now execution should succeed
        let result = client.try_execute_proposal(&signers.get(0).unwrap(), &proposal_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_emergency_halt() {
        let (env, admin, signers) = create_test_env();
        let contract_id = setup_treasury_controller(&env, &admin, &signers);
        let client = TreasuryControllerClient::new(&env, &contract_id);

        // Emergency halt by admin
        client.try_emergency_halt(&admin).unwrap();

        // Verify system is halted
        let config = client.get_config();
        assert!(config.emergency_halted);

        // Try to create proposal while halted (should fail)
        let token_address = Address::generate(&env);
        client.try_add_supported_token(&token_address).unwrap();

        let result = client.try_create_proposal(
            &signers.get(0).unwrap(),
            &ProposalType::Withdrawal,
            &Address::generate(&env),
            &token_address,
            &100_000i128,
            &String::from_str(&env, "Test"),
            &String::from_str(&env, "Test"),
            &Bytes::new(&env),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_gnosis_safe_compatibility() {
        let (env, admin, signers) = create_test_env();
        let contract_id = setup_treasury_controller(&env, &admin, &signers);
        let client = TreasuryControllerClient::new(&env, &contract_id);

        // Test Gnosis Safe compatibility functions
        let threshold = client.gnosis_get_threshold();
        assert_eq!(threshold, 2);

        let owners = client.gnosis_get_owners();
        assert_eq!(owners.len(), 3);
        assert_eq!(owners, signers);
    }

    #[test]
    fn test_proposal_count() {
        let (env, admin, signers) = create_test_env();
        let contract_id = setup_treasury_controller(&env, &admin, &signers);
        let client = TreasuryControllerClient::new(&env, &contract_id);

        // Initially should be 0
        assert_eq!(client.get_proposal_count(), 0);

        let token_address = Address::generate(&env);
        client.try_add_supported_token(&token_address).unwrap();

        // Create a proposal
        let _proposal_id = client.try_create_proposal(
            &signers.get(0).unwrap(),
            &ProposalType::Withdrawal,
            &Address::generate(&env),
            &token_address,
            &500_000i128,
            &String::from_str(&env, "Test"),
            &String::from_str(&env, "Test"),
            &Bytes::new(&env),
        ).unwrap();

        // Should now be 1
        assert_eq!(client.get_proposal_count(), 1);
    }

    #[test]
    fn test_is_proposal_executable() {
        let (env, admin, signers) = create_test_env();
        let contract_id = setup_treasury_controller(&env, &admin, &signers);
        let client = TreasuryControllerClient::new(&env, &contract_id);

        let token_address = Address::generate(&env);
        client.try_add_supported_token(&token_address).unwrap();

        let proposal_id = client.try_create_proposal(
            &signers.get(0).unwrap(),
            &ProposalType::Withdrawal,
            &Address::generate(&env),
            &token_address,
            &500_000i128,
            &String::from_str(&env, "Test"),
            &String::from_str(&env, "Test"),
            &Bytes::new(&env),
        ).unwrap();

        // Should not be executable initially (not approved)
        assert!(!client.is_proposal_executable(&proposal_id));

        // Approve proposal
        client.try_approve_proposal(&signers.get(0).unwrap(), &proposal_id).unwrap();
        client.try_approve_proposal(&signers.get(1).unwrap(), &proposal_id).unwrap();

        // Still not executable (timelock not expired)
        assert!(!client.is_proposal_executable(&proposal_id));

        // Advance time past timelock
        env.ledger().with_mut(|li| {
            li.timestamp = li.timestamp + 3700; // 1 hour + 100 seconds
        });

        // Now should be executable
        assert!(client.is_proposal_executable(&proposal_id));
    }
}
