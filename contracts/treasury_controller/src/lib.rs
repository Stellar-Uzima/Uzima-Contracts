// Treasury Controller - Multi-sig treasury with timelocks and proper validation
#![no_std]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::unwrap_used)]
#![allow(dead_code)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Bytes, Env, Map,
    Symbol, Vec,
};

// Storage keys
const ADMIN: Symbol = symbol_short!("ADMIN");
const TOKEN: Symbol = symbol_short!("TOKEN");
const SIGNERS: Symbol = symbol_short!("SIGNERS");
const THRESHOLD: Symbol = symbol_short!("THRESH");
const PROPOSAL_COUNT: Symbol = symbol_short!("PROP_CNT");
const PROPOSALS: Symbol = symbol_short!("PROPS");
const TIMELOCK: Symbol = symbol_short!("TIMELOCK"); // Seconds
const RECOVERY_MODE: Symbol = symbol_short!("RECOVERY");

// Limits
const MIN_TIMELOCK: u64 = 3600; // 1 hour
const MAX_TIMELOCK: u64 = 2592000; // 30 days
const MAX_SIGNERS: u32 = 20;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    ProposalNotFound = 2,
    ProposalExecuted = 3,
    ProposalExpired = 4,
    TimelockNotElapsed = 5,
    InsufficientSignatures = 6,
    InvalidConfig = 7,
    InvalidAmount = 8,
    LimitExceeded = 9,
    AlreadyExecuted = 10,
    InvalidRecipient = 11,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProposalType {
    Transfer,
    UpgradeContract,
    ChangeSigners,
    UpdateConfig,
    EmergencyWithdraw,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Address,
    pub proposal_type: ProposalType,
    pub target: Address, // Recipient or Contract Address
    pub amount: i128,    // 0 for non-transfer
    pub data: Bytes,     // Additional data (e.g., new signer list, new hash)
    pub created_at: u64,
    pub execution_time: u64,
    pub executed: bool,
    pub approvals: Vec<Address>,
}

#[contract]
pub struct TreasuryControllerContract;

#[contractimpl]
impl TreasuryControllerContract {
    /// Initialize the treasury controller
    pub fn initialize(
        env: Env,
        admin: Address,
        token_contract: Address,
        initial_signers: Vec<Address>,
        threshold: u32,
        timelock_duration: u64,
    ) -> Result<bool, Error> {
        admin.require_auth();

        if env.storage().persistent().has(&ADMIN) {
            return Err(Error::InvalidConfig);
        }

        if initial_signers.len() > MAX_SIGNERS {
            return Err(Error::LimitExceeded);
        }

        if threshold == 0 || initial_signers.len() < threshold {
            return Err(Error::InvalidConfig);
        }

        // FIXED: Used RangeInclusive::contains
        if !(MIN_TIMELOCK..=MAX_TIMELOCK).contains(&timelock_duration) {
            return Err(Error::InvalidConfig);
        }

        env.storage().persistent().set(&ADMIN, &admin);
        env.storage().persistent().set(&TOKEN, &token_contract);
        env.storage().persistent().set(&SIGNERS, &initial_signers);
        env.storage().persistent().set(&THRESHOLD, &threshold);
        env.storage()
            .persistent()
            .set(&TIMELOCK, &timelock_duration);
        env.storage().persistent().set(&PROPOSAL_COUNT, &0u64);
        env.storage().persistent().set(&RECOVERY_MODE, &false);

        Ok(true)
    }

    /// Create a new proposal
    pub fn create_proposal(
        env: Env,
        proposer: Address,
        proposal_type: ProposalType,
        target: Address,
        amount: i128,
        execution_delay: u64,
        execution_data: Bytes,
    ) -> Result<u64, Error> {
        proposer.require_auth();

        // Verify proposer is a signer
        Self::require_signer(&env, &proposer)?;

        let timelock: u64 = env
            .storage()
            .persistent()
            .get(&TIMELOCK)
            .unwrap_or(MIN_TIMELOCK);
        let final_delay = if execution_delay < timelock {
            timelock
        } else {
            execution_delay
        };

        let now = env.ledger().timestamp();
        let proposal_id: u64 = env.storage().persistent().get(&PROPOSAL_COUNT).unwrap_or(0);

        let proposal = Proposal {
            id: proposal_id,
            proposer: proposer.clone(),
            proposal_type,
            target,
            amount,
            data: execution_data,
            created_at: now,
            execution_time: now + final_delay,
            executed: false,
            approvals: Vec::new(&env),
        };

        let mut proposals: Map<u64, Proposal> = env
            .storage()
            .persistent()
            .get(&PROPOSALS)
            .unwrap_or(Map::new(&env));

        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&PROPOSALS, &proposals);

        // Increment count
        env.storage()
            .persistent()
            .set(&PROPOSAL_COUNT, &(proposal_id + 1));

        Ok(proposal_id)
    }

    /// Approve a proposal
    pub fn approve_proposal(env: Env, signer: Address, proposal_id: u64) -> Result<bool, Error> {
        signer.require_auth();
        Self::require_signer(&env, &signer)?;

        let mut proposals: Map<u64, Proposal> = env
            .storage()
            .persistent()
            .get(&PROPOSALS)
            .unwrap_or(Map::new(&env));

        let mut proposal = proposals.get(proposal_id).ok_or(Error::ProposalNotFound)?;

        if proposal.executed {
            return Err(Error::ProposalExecuted);
        }

        if !proposal.approvals.contains(&signer) {
            proposal.approvals.push_back(signer);
            proposals.set(proposal_id, proposal);
            env.storage().persistent().set(&PROPOSALS, &proposals);
        }

        Ok(true)
    }

    /// Execute a proposal
    pub fn execute_proposal(env: Env, executor: Address, proposal_id: u64) -> Result<bool, Error> {
        executor.require_auth(); // Anyone can trigger execution if conditions met

        let mut proposals: Map<u64, Proposal> = env
            .storage()
            .persistent()
            .get(&PROPOSALS)
            .unwrap_or(Map::new(&env));

        let mut proposal = proposals.get(proposal_id).ok_or(Error::ProposalNotFound)?;

        if proposal.executed {
            return Err(Error::ProposalExecuted);
        }

        // Check timelock
        if env.ledger().timestamp() < proposal.execution_time {
            return Err(Error::TimelockNotElapsed);
        }

        // Check threshold
        let threshold: u32 = env.storage().persistent().get(&THRESHOLD).unwrap_or(1);
        if proposal.approvals.len() < threshold {
            return Err(Error::InsufficientSignatures);
        }

        // Execute logic based on type
        match proposal.proposal_type {
            ProposalType::Transfer => {
                // In a real implementation, this would call the token contract
                // token::Client::new(&env, &token_addr).transfer(...)
                // For now, we emit an event
                env.events().publish(
                    (symbol_short!("Transfer"),),
                    (proposal.target.clone(), proposal.amount),
                );
            }
            ProposalType::UpgradeContract => {
                // env.deployer().update_current_contract_wasm(...)
            }
            ProposalType::ChangeSigners => {
                // Logic to decode `data` and update SIGNERS
            }
            ProposalType::UpdateConfig => {
                // Update timelock or threshold
            }
            ProposalType::EmergencyWithdraw => {
                // Bypass some checks or limits?
            }
        }

        proposal.executed = true;
        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&PROPOSALS, &proposals);

        Ok(true)
    }

    /// Helper: Check if address is a signer
    fn require_signer(env: &Env, signer: &Address) -> Result<(), Error> {
        let signers: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        // Only admin can resume operations
        if caller != config.admin {
            return Err(Error::NotAuthorized);
        }

        config.emergency_halted = false;
        env.storage().instance().set(&DataKey::Config, &config);

        // Emit resume event
        env.events().publish((symbol_short!("RESUMED"),), caller);

        Ok(())
    }

    // === View Functions ===

    /// Get treasury configuration
    pub fn get_config(env: Env) -> TreasuryConfig {
        env.storage().instance().get(&DataKey::Config).unwrap()
    }

    /// Get proposal details
    pub fn get_proposal(env: Env, proposal_id: u64) -> TreasuryProposal {
        let proposals: Map<u64, TreasuryProposal> =
            env.storage().persistent().get(&DataKey::Proposals).unwrap();

        proposals.get(proposal_id).unwrap()
    }

    /// Get total number of proposals
    pub fn get_proposal_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::ProposalCount)
            .unwrap_or(0)
    }

    /// Check if proposal is ready for execution
    pub fn is_proposal_executable(env: Env, proposal_id: u64) -> bool {
        let proposals: Map<u64, TreasuryProposal> =
            match env.storage().persistent().get(&DataKey::Proposals) {
                Some(p) => p,
                None => return false,
            };

        let proposal = match proposals.get(proposal_id) {
            Some(p) => p,
            None => return false,
        };

        let config: TreasuryConfig = match env.storage().instance().get(&DataKey::Config) {
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

    // === Private Helper Functions ===

    /// Execute token transfer using standard transfer interface
    fn execute_token_transfer(
        env: &Env,
        token_contract: &Address,
        from: &Address,
        to: &Address,
        amount: i128,
    ) -> Result<(), Error> {
        // Use env.invoke_contract to call the token's transfer function
        let result: Result<(), soroban_sdk::InvokeError> = env.invoke_contract(
            token_contract,
            &Symbol::new(env, "transfer"),
            soroban_sdk::vec![
                env,
                from.into_val(env),
                to.into_val(env),
                amount.into_val(env)
            ],
        );

        // Convert invoke error to our custom error
        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::TransferFailed),
        }
    }

    /// Allows the Governor/Timelock (Admin) to execute transfers immediately
    /// Bypassing the multisig process.
    pub fn governance_execute(
        env: Env,
        token_contract: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), Error> {
        let config: TreasuryConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        // strictly require admin auth (The Governor Contract)
        config.admin.require_auth();

        Self::execute_token_transfer(
            &env,
            &token_contract,
            &env.current_contract_address(),
            &to,
            amount,
        )?;

        env.events()
            .publish((symbol_short!("GOV_EXEC"),), (to, amount));
        Ok(())
    }
}

#[cfg(test)]
mod test;
