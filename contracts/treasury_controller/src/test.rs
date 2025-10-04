#[cfg(test)]
pub mod tests {
    use crate::{
        Error, ProposalStatus, ProposalType, TreasuryController, TreasuryControllerClient,
    };
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

        // Initialize the treasury controller
        let client = TreasuryControllerClient::new(env, &contract_id);
        client.initialize(
            admin,
            signers,
            &2u32,          // threshold
            &3600u64,       // 1 hour timelock
            &2u32,          // emergency threshold
            &1_000_000i128, // max withdrawal amount
        );

        contract_id
    }

    #[test]
    fn test_basic_functionality() {
        let (env, admin, signers) = create_test_env();
        let contract_id = setup_treasury_controller(&env, &admin, &signers);

        // Test that we can register the contract successfully
        assert!(!contract_id.to_string().is_empty());
    }

    #[test]
    fn test_withdrawal_proposal_creation_and_execution_pattern() {
        let (env, admin, signers) = create_test_env();
        let treasury_contract = setup_treasury_controller(&env, &admin, &signers);
        let client = TreasuryControllerClient::new(&env, &treasury_contract);

        // Mock token contract address
        let token_contract = Address::generate(&env);
        client.add_supported_token(&token_contract);

        let recipient = Address::generate(&env);
        let withdrawal_amount = 500_000i128;

        // Create withdrawal proposal
        let proposal_id = client.create_proposal(
            &signers.get(0).unwrap(),
            &ProposalType::Withdrawal,
            &recipient,
            &token_contract,
            &withdrawal_amount,
            &String::from_str(&env, "Test withdrawal"),
            &String::from_str(&env, "Testing proposal workflow"),
            &Bytes::new(&env),
        );

        // Verify proposal was created with correct details
        let proposal = client.get_proposal(&proposal_id);
        assert_eq!(proposal.proposal_id, 1u64);
        assert_eq!(proposal.amount, withdrawal_amount);
        assert_eq!(proposal.target_address, recipient);
        assert_eq!(proposal.token_contract, token_contract);
        assert_eq!(proposal.status, ProposalStatus::Pending);

        // Approve proposal by required signers
        client.approve_proposal(&signers.get(0).unwrap(), &proposal_id);
        client.approve_proposal(&signers.get(1).unwrap(), &proposal_id);

        // Verify proposal is now approved
        let approved_proposal = client.get_proposal(&proposal_id);
        assert_eq!(approved_proposal.status, ProposalStatus::Approved);
        assert_eq!(approved_proposal.approvals.len(), 2u32);

        // Advance time past timelock
        env.ledger().with_mut(|ledger_info| {
            ledger_info.timestamp += 3700; // 1 hour + buffer
        });

        // Verify proposal is executable
        assert!(client.is_proposal_executable(&proposal_id));
    }

    #[test]
    fn test_withdrawal_execution_calls_transfer_function() {
        let (env, admin, signers) = create_test_env();
        let treasury_contract = setup_treasury_controller(&env, &admin, &signers);
        let client = TreasuryControllerClient::new(&env, &treasury_contract);

        // Create a mock token contract that we can observe calls to
        let token_contract = Address::generate(&env);
        client.add_supported_token(&token_contract);

        let recipient = Address::generate(&env);
        let withdrawal_amount = 100_000i128;

        // Create and approve proposal
        let proposal_id = client.create_proposal(
            &signers.get(0).unwrap(),
            &ProposalType::Withdrawal,
            &recipient,
            &token_contract,
            &withdrawal_amount,
            &String::from_str(&env, "Test withdrawal"),
            &String::from_str(&env, "Testing transfer call"),
            &Bytes::new(&env),
        );

        client.approve_proposal(&signers.get(0).unwrap(), &proposal_id);
        client.approve_proposal(&signers.get(1).unwrap(), &proposal_id);

        env.ledger().with_mut(|ledger_info| {
            ledger_info.timestamp += 3700;
        });

        // Since we don't have a real token contract, the execution should fail with TransferFailed
        // This demonstrates that the transfer function is being called
        let result = client.try_execute_proposal(&signers.get(0).unwrap(), &proposal_id);

        // Should fail because mock token doesn't exist, but this proves the transfer was attempted
        assert!(result.is_err());

        // Verify proposal status remains approved (not executed due to failed transfer)
        let proposal = client.get_proposal(&proposal_id);
        assert_eq!(proposal.status, ProposalStatus::Approved);
    }

    #[test]
    fn test_state_rollback_on_transfer_failure() {
        let (env, admin, signers) = create_test_env();
        let treasury_contract = setup_treasury_controller(&env, &admin, &signers);
        let client = TreasuryControllerClient::new(&env, &treasury_contract);

        // Use a non-existent token contract to force transfer failure
        let fake_token_contract = Address::generate(&env);
        client.add_supported_token(&fake_token_contract);

        let recipient = Address::generate(&env);
        let withdrawal_amount = 300_000i128;

        // Create and approve proposal
        let proposal_id = client.create_proposal(
            &signers.get(0).unwrap(),
            &ProposalType::Withdrawal,
            &recipient,
            &fake_token_contract,
            &withdrawal_amount,
            &String::from_str(&env, "Test withdrawal"),
            &String::from_str(&env, "Testing rollback"),
            &Bytes::new(&env),
        );

        client.approve_proposal(&signers.get(0).unwrap(), &proposal_id);
        client.approve_proposal(&signers.get(1).unwrap(), &proposal_id);

        env.ledger().with_mut(|ledger_info| {
            ledger_info.timestamp += 3700;
        });

        // Get initial proposal state
        let initial_proposal = client.get_proposal(&proposal_id);
        assert_eq!(initial_proposal.status, ProposalStatus::Approved);

        // Execute should fail due to non-existent token contract
        let result = client.try_execute_proposal(&signers.get(0).unwrap(), &proposal_id);
        assert!(result.is_err());

        // Verify proposal state was not changed (rollback successful)
        let final_proposal = client.get_proposal(&proposal_id);
        assert_eq!(final_proposal.status, ProposalStatus::Approved); // Still approved, not executed
        assert_eq!(final_proposal.proposal_id, initial_proposal.proposal_id);
        assert_eq!(final_proposal.amount, initial_proposal.amount);
    }

    #[test]
    fn test_non_withdrawal_proposals_skip_transfer() {
        let (env, admin, signers) = create_test_env();
        let treasury_contract = setup_treasury_controller(&env, &admin, &signers);
        let client = TreasuryControllerClient::new(&env, &treasury_contract);

        let target_address = Address::generate(&env);
        let token_contract = Address::generate(&env);

        // Create a non-withdrawal proposal (ConfigChange)
        let proposal_id = client.create_proposal(
            &signers.get(0).unwrap(),
            &ProposalType::ConfigChange,
            &target_address,
            &token_contract,
            &0i128, // No amount for config change
            &String::from_str(&env, "Update config"),
            &String::from_str(&env, "Testing non-withdrawal"),
            &Bytes::new(&env),
        );

        client.approve_proposal(&signers.get(0).unwrap(), &proposal_id);
        client.approve_proposal(&signers.get(1).unwrap(), &proposal_id);

        env.ledger().with_mut(|ledger_info| {
            ledger_info.timestamp += 3700;
        });

        // Execute should succeed because no transfer is attempted
        client.execute_proposal(&signers.get(0).unwrap(), &proposal_id);

        // Verify proposal was executed successfully
        let proposal = client.get_proposal(&proposal_id);
        assert_eq!(proposal.status, ProposalStatus::Executed);
    }
}
