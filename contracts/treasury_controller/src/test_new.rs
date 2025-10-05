#[cfg(test)]
pub mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Ledger, MockAuth, MockAuthInvoke},
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
        let config = client.try_get_config().unwrap();
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
        let config = client.try_get_config().unwrap();
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
        let proposal = client.try_get_proposal(&proposal_id).unwrap();
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
        let proposal = client.try_get_proposal(&proposal_id).unwrap();
        assert!(matches!(proposal.status, ProposalStatus::Pending));
        assert_eq!(proposal.approvals.len(), 1);

        // Second approval should make it approved
        client.try_approve_proposal(&signers.get(1).unwrap(), &proposal_id).unwrap();
        let proposal = client.try_get_proposal(&proposal_id).unwrap();
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
        let config = client.try_get_config().unwrap();
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
        let threshold = client.try_gnosis_get_threshold().unwrap();
        assert_eq!(threshold, 2);

        let owners = client.try_gnosis_get_owners().unwrap();
        assert_eq!(owners.len(), 3);
        assert_eq!(owners, signers);
    }
}

use soroban_sdk::contractclient;

#[contractclient(name = "TreasuryControllerClient")]
pub trait TreasuryControllerTrait {
    fn initialize(
        env: Env,
        admin: Address,
        signers: Vec<Address>,
        threshold: u32,
        timelock_duration: u64,
        emergency_threshold: u32,
        max_withdrawal_amount: i128,
    ) -> Result<(), Error>;

    fn add_supported_token(env: Env, token_address: Address) -> Result<(), Error>;

    fn create_proposal(
        env: Env,
        proposer: Address,
        proposal_type: ProposalType,
        target_address: Address,
        token_contract: Address,
        amount: i128,
        purpose: String,
        metadata: String,
        execution_data: Bytes,
    ) -> Result<u64, Error>;

    fn approve_proposal(env: Env, signer: Address, proposal_id: u64) -> Result<(), Error>;

    fn execute_proposal(env: Env, executor: Address, proposal_id: u64) -> Result<(), Error>;

    fn emergency_halt(env: Env, caller: Address) -> Result<(), Error>;

    fn resume_operations(env: Env, caller: Address) -> Result<(), Error>;

    fn get_config(env: Env) -> Result<TreasuryConfig, Error>;

    fn get_proposal(env: Env, proposal_id: u64) -> Result<TreasuryProposal, Error>;

    fn get_proposal_count(env: Env) -> u64;

    fn is_proposal_executable(env: Env, proposal_id: u64) -> Result<bool, Error>;

    fn gnosis_get_threshold(env: Env) -> Result<u32, Error>;

    fn gnosis_get_owners(env: Env) -> Result<Vec<Address>, Error>;
}
