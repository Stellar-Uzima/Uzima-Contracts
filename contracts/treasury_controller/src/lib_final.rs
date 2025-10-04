#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracterror, contracttype, 
    Address, Bytes, BytesN, Env, Map, String, Vec, symbol_short
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    InvalidThreshold = 3,
    InvalidTimelock = 4,
    NotSigner = 5,
    ProposalNotFound = 6,
    NotPending = 7,
    AlreadyApproved = 8,
    TimelockNotExpired = 9,
    NotApproved = 10,
    Halted = 11,
    NotAuthorized = 12,
    SymbolTooLong = 13,
}

/// Treasury proposal types
#[derive(Clone)]
#[contracttype]
pub enum ProposalType {
    Withdrawal,
    ConfigChange,
    EmergencyHalt,
}

/// Proposal status
#[derive(Clone)]
#[contracttype]
pub enum ProposalStatus {
    Pending,
    Approved,
    Executed,
    Rejected,
    Expired,
}

/// Treasury proposal structure
#[derive(Clone)]
#[contracttype]
pub struct TreasuryProposal {
    pub proposal_id: u64,
    pub proposal_type: ProposalType,
    pub proposer: Address,
    pub target_address: Address,
    pub token_contract: Address,
    pub amount: i128,
    pub purpose: String,
    pub metadata: String,
    pub created_at: u64,
    pub timelock_end: u64,
    pub status: ProposalStatus,
    pub approvals: Vec<Address>,
    pub rejections: Vec<Address>,
    pub execution_data: Bytes,
}

/// Multisig configuration
#[derive(Clone)]
#[contracttype]
pub struct MultisigConfig {
    pub signers: Vec<Address>,
    pub threshold: u32,
    pub timelock_duration: u64,
    pub emergency_threshold: u32,
}

/// Treasury configuration
#[derive(Clone)]
#[contracttype]
pub struct TreasuryConfig {
    pub admin: Address,
    pub multisig_config: MultisigConfig,
    pub max_withdrawal_amount: i128,
    pub emergency_halted: bool,
    pub supported_tokens: Vec<Address>,
}

/// Withdrawal record for audit trail
#[derive(Clone)]
#[contracttype]
pub struct WithdrawalRecord {
    pub proposal_id: u64,
    pub token_contract: Address,
    pub amount: i128,
    pub recipient: Address,
    pub purpose: String,
    pub executed_at: u64,
    pub executed_by: Address,
    pub transaction_hash: BytesN<32>,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Config,
    Proposals,
    ProposalCount,
    Withdrawals,
}

// Constants
const MIN_TIMELOCK: u64 = 3600; // 1 hour minimum
const MAX_TIMELOCK: u64 = 604800; // 1 week maximum
const PROPOSAL_EXPIRY: u64 = 2592000; // 30 days

#[contract]
pub struct TreasuryController;

#[contractimpl]
impl TreasuryController {
    /// Initialize the treasury controller
    pub fn initialize(
        env: Env,
        admin: Address,
        signers: Vec<Address>,
        threshold: u32,
        timelock_duration: u64,
        emergency_threshold: u32,
        max_withdrawal_amount: i128,
    ) -> Result<(), Error> {
        admin.require_auth();

        // Check if already initialized
        if env.storage().instance().has(&DataKey::Config) {
            return Err(Error::AlreadyInitialized);
        }

        // Validate parameters
        if (signers.len() as u32) < threshold {
            return Err(Error::InvalidThreshold);
        }

        if timelock_duration < MIN_TIMELOCK || timelock_duration > MAX_TIMELOCK {
            return Err(Error::InvalidTimelock);
        }

        if emergency_threshold > threshold {
            return Err(Error::InvalidThreshold);
        }

        let multisig_config = MultisigConfig {
            signers,
            threshold,
            timelock_duration,
            emergency_threshold,
        };

        let config = TreasuryConfig {
            admin: admin.clone(),
            multisig_config,
            max_withdrawal_amount,
            emergency_halted: false,
            supported_tokens: Vec::new(&env),
        };

        env.storage().instance().set(&DataKey::Config, &config);
        env.storage().instance().set(&DataKey::ProposalCount, &0u64);

        // Emit initialization event
        env.events().publish(
            (symbol_short!("INIT"),),
            admin,
        );

        Ok(())
    }

    /// Add supported token for treasury operations
    pub fn add_supported_token(env: Env, token_address: Address) -> Result<(), Error> {
        let mut config: TreasuryConfig = env.storage().instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        config.admin.require_auth();

        if config.emergency_halted {
            return Err(Error::Halted);
        }

        if !config.supported_tokens.contains(&token_address) {
            config.supported_tokens.push_back(token_address.clone());
            env.storage().instance().set(&DataKey::Config, &config);

            env.events().publish(
                (symbol_short!("TOKEN_ADD"),),
                token_address,
            );
        }

        Ok(())
    }

    /// Create a new treasury proposal
    pub fn create_proposal(
        env: Env,
        proposer: Address,
        proposal_type: ProposalType,
        target_address: Address,
        token_contract: Address,
        amount: i128,
        purpose: String,
        metadata: String,
        execution_data: Bytes,
    ) -> Result<u64, Error> {
        proposer.require_auth();

        let config: TreasuryConfig = env.storage().instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        if config.emergency_halted {
            return Err(Error::Halted);
        }

        // Validate proposer is a signer
        if !config.multisig_config.signers.contains(&proposer) {
            return Err(Error::NotSigner);
        }

        // Validate withdrawal amount and token
        if matches!(proposal_type, ProposalType::Withdrawal) {
            if amount > config.max_withdrawal_amount {
                return Err(Error::NotAuthorized);
            }

            if !config.supported_tokens.contains(&token_contract) {
                return Err(Error::NotAuthorized);
            }
        }

        let proposal_id = env.storage().instance()
            .get(&DataKey::ProposalCount)
            .unwrap_or(0u64) + 1;

        let current_time = env.ledger().timestamp();
        let timelock_end = current_time + config.multisig_config.timelock_duration;

        let proposal = TreasuryProposal {
            proposal_id,
            proposal_type: proposal_type.clone(),
            proposer: proposer.clone(),
            target_address,
            token_contract,
            amount,
            purpose: purpose.clone(),
            metadata,
            created_at: current_time,
            timelock_end,
            status: ProposalStatus::Pending,
            approvals: Vec::new(&env),
            rejections: Vec::new(&env),
            execution_data,
        };

        let mut proposals: Map<u64, TreasuryProposal> = env.storage()
            .persistent()
            .get(&DataKey::Proposals)
            .unwrap_or(Map::new(&env));

        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&DataKey::Proposals, &proposals);
        env.storage().instance().set(&DataKey::ProposalCount, &proposal_id);

        // Emit proposal created event
        env.events().publish(
            (symbol_short!("PROPOSAL"),),
            (proposal_id, proposer, amount),
        );

        Ok(proposal_id)
    }

    /// Approve a treasury proposal
    pub fn approve_proposal(env: Env, signer: Address, proposal_id: u64) -> Result<(), Error> {
        signer.require_auth();

        let config: TreasuryConfig = env.storage().instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        if config.emergency_halted {
            return Err(Error::Halted);
        }

        // Validate signer
        if !config.multisig_config.signers.contains(&signer) {
            return Err(Error::NotSigner);
        }

        let mut proposals: Map<u64, TreasuryProposal> = env.storage()
            .persistent()
            .get(&DataKey::Proposals)
            .ok_or(Error::ProposalNotFound)?;

        let mut proposal = proposals.get(proposal_id)
            .ok_or(Error::ProposalNotFound)?;

        // Check if proposal is still pending
        if !matches!(proposal.status, ProposalStatus::Pending) {
            return Err(Error::NotPending);
        }

        // Check if already approved by this signer
        if proposal.approvals.contains(&signer) {
            return Err(Error::AlreadyApproved);
        }

        // Remove from rejections if previously rejected
        if let Some(index) = proposal.rejections.iter().position(|x| x == signer) {
            proposal.rejections.remove(index as u32);
        }

        proposal.approvals.push_back(signer.clone());

        // Check if threshold reached
        if proposal.approvals.len() >= config.multisig_config.threshold {
            proposal.status = ProposalStatus::Approved;
        }

        let approvals_len = proposal.approvals.len();
        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&DataKey::Proposals, &proposals);

        // Emit approval event
        env.events().publish(
            (symbol_short!("APPROVED"),),
            (proposal_id, signer, approvals_len),
        );

        Ok(())
    }

    /// Execute an approved proposal after timelock
    pub fn execute_proposal(env: Env, executor: Address, proposal_id: u64) -> Result<(), Error> {
        executor.require_auth();

        let config: TreasuryConfig = env.storage().instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        if config.emergency_halted {
            return Err(Error::Halted);
        }

        // Validate executor is a signer
        if !config.multisig_config.signers.contains(&executor) {
            return Err(Error::NotSigner);
        }

        let mut proposals: Map<u64, TreasuryProposal> = env.storage()
            .persistent()
            .get(&DataKey::Proposals)
            .ok_or(Error::ProposalNotFound)?;

        let mut proposal = proposals.get(proposal_id)
            .ok_or(Error::ProposalNotFound)?;

        // Check if proposal is approved
        if !matches!(proposal.status, ProposalStatus::Approved) {
            return Err(Error::NotApproved);
        }

        // Check timelock
        let current_time = env.ledger().timestamp();
        if current_time < proposal.timelock_end {
            return Err(Error::TimelockNotExpired);
        }

        // Mark as executed
        proposal.status = ProposalStatus::Executed;
        proposals.set(proposal_id, proposal.clone());
        env.storage().persistent().set(&DataKey::Proposals, &proposals);

        // Record withdrawal for audit trail
        if matches!(proposal.proposal_type, ProposalType::Withdrawal) {
            let withdrawal_record = WithdrawalRecord {
                proposal_id,
                token_contract: proposal.token_contract.clone(),
                amount: proposal.amount,
                recipient: proposal.target_address.clone(),
                purpose: proposal.purpose.clone(),
                executed_at: current_time,
                executed_by: executor.clone(),
                transaction_hash: BytesN::from_array(&env, &[0u8; 32]),
            };

            let mut withdrawals: Map<u64, WithdrawalRecord> = env.storage()
                .persistent()
                .get(&DataKey::Withdrawals)
                .unwrap_or(Map::new(&env));

            withdrawals.set(proposal_id, withdrawal_record);
            env.storage().persistent().set(&DataKey::Withdrawals, &withdrawals);
        }

        // Emit execution event
        env.events().publish(
            (symbol_short!("EXECUTED"),),
            (proposal_id, executor, proposal.amount),
        );

        Ok(())
    }

    /// Emergency halt all treasury operations
    pub fn emergency_halt(env: Env, caller: Address) -> Result<(), Error> {
        caller.require_auth();

        let mut config: TreasuryConfig = env.storage().instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        // Check if caller is authorized (admin or signer)
        let is_admin = caller == config.admin;
        let is_signer = config.multisig_config.signers.contains(&caller);

        if !is_admin && !is_signer {
            return Err(Error::NotAuthorized);
        }

        config.emergency_halted = true;
        env.storage().instance().set(&DataKey::Config, &config);

        // Emit emergency halt event
        env.events().publish(
            (symbol_short!("EMERGENCY"),),
            caller,
        );

        Ok(())
    }

    /// Resume operations after emergency halt
    pub fn resume_operations(env: Env, caller: Address) -> Result<(), Error> {
        caller.require_auth();

        let mut config: TreasuryConfig = env.storage().instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        // Only admin can resume operations
        if caller != config.admin {
            return Err(Error::NotAuthorized);
        }

        config.emergency_halted = false;
        env.storage().instance().set(&DataKey::Config, &config);

        // Emit resume event
        env.events().publish(
            (symbol_short!("RESUMED"),),
            caller,
        );

        Ok(())
    }

    // === View Functions ===

    /// Get treasury configuration
    pub fn get_config(env: Env) -> TreasuryConfig {
        env.storage().instance()
            .get(&DataKey::Config)
            .unwrap()
    }

    /// Get proposal details
    pub fn get_proposal(env: Env, proposal_id: u64) -> TreasuryProposal {
        let proposals: Map<u64, TreasuryProposal> = env.storage()
            .persistent()
            .get(&DataKey::Proposals)
            .unwrap();

        proposals.get(proposal_id)
            .unwrap()
    }

    /// Get total number of proposals
    pub fn get_proposal_count(env: Env) -> u64 {
        env.storage().instance()
            .get(&DataKey::ProposalCount)
            .unwrap_or(0)
    }

    /// Check if proposal is ready for execution
    pub fn is_proposal_executable(env: Env, proposal_id: u64) -> bool {
        let proposals: Map<u64, TreasuryProposal> = match env.storage()
            .persistent()
            .get(&DataKey::Proposals) {
                Some(p) => p,
                None => return false,
            };

        let proposal = match proposals.get(proposal_id) {
            Some(p) => p,
            None => return false,
        };

        let config: TreasuryConfig = match env.storage().instance()
            .get(&DataKey::Config) {
                Some(c) => c,
                None => return false,
            };

        if !matches!(proposal.status, ProposalStatus::Approved) {
            return false;
        }

        let current_time = env.ledger().timestamp();
        
        // Check timelock
        if current_time < proposal.timelock_end {
            return false;
        }

        // Check expiry
        if current_time > proposal.created_at + PROPOSAL_EXPIRY {
            return false;
        }

        // Check if emergency halted
        if config.emergency_halted {
            return false;
        }

        true
    }

    // === Gnosis Safe Compatibility Interface ===
    
    /// Get threshold for Gnosis Safe compatibility
    pub fn gnosis_get_threshold(env: Env) -> u32 {
        let config = Self::get_config(env);
        config.multisig_config.threshold
    }

    /// Get owners for Gnosis Safe compatibility
    pub fn gnosis_get_owners(env: Env) -> Vec<Address> {
        let config = Self::get_config(env);
        config.multisig_config.signers
    }
}

pub mod test;
