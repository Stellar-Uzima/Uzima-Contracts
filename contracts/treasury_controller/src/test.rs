#[cfg(test)]
mod tests {
    use crate::{ProposalStatus, ProposalType, TreasuryController, TreasuryControllerClient};
    use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, String, Vec};

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
    fn test_basic_initialization() {
        let (env, admin, signers) = create_test_env();
        let contract_id = setup_treasury_controller(&env, &admin, &signers);

        assert!(!contract_id.to_string().is_empty());
    }

    #[test]
    fn test_proposal_creation_and_approval_workflow() {
        let (env, admin, signers) = create_test_env();
        let treasury_contract = setup_treasury_controller(&env, &admin, &signers);
        let client = TreasuryControllerClient::new(&env, &treasury_contract);

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

        // Verify proposal was created correctly
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
    }

    #[test]
    fn test_non_withdrawal_proposals_work() {
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

        // Advance time past timelock
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