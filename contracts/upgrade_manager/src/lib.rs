#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Map, Symbol, Vec,
};

#[cfg(test)]
mod test;

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct UpgradeProposal {
    pub target: Address,
    pub new_wasm_hash: BytesN<32>,
    pub new_version: u32,
    pub description: Symbol,
    pub proposer: Address,
    pub created_at: u64,
    pub executable_at: u64,
    pub executed: bool,
    pub canceled: bool,
    pub approvals: Vec<Address>,
}

#[soroban_sdk::contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum UpgradeManagerError {
    AlreadyInitialized = 1,
    NotAValidator = 2,
    ProposalNotFound = 3,
    AlreadyApproved = 4,
    InvalidState = 5,
    TimelockNotExpired = 6,
    NotEnoughApprovals = 7,
    ConfigNotFound = 8,
}

#[contract]
pub struct UpgradeManager;

const PROPOSALS: Symbol = symbol_short!("PROPS");
const CONFIG: Symbol = symbol_short!("CONFIG");
const MIN_DELAY: u64 = 86400; // 24 hours
const REQUIRED_APPROVALS: u32 = 3;

#[contracttype]
pub struct Config {
    pub admin: Address,
    pub min_delay: u64,
    pub required_approvals: u32,
    pub validators: Vec<Address>,
}

// Minimal interface for target contracts
#[soroban_sdk::contractclient(name = "TargetContractClient")]
pub trait TargetContract {
    fn upgrade(env: Env, new_wasm_hash: BytesN<32>);
}

#[contractimpl]
impl UpgradeManager {
    pub fn initialize(
        env: Env,
        admin: Address,
        validators: Vec<Address>,
    ) -> Result<(), UpgradeManagerError> {
        if env.storage().instance().has(&CONFIG) {
            return Err(UpgradeManagerError::AlreadyInitialized);
        }
        let config = Config {
            admin,
            min_delay: MIN_DELAY,
            required_approvals: REQUIRED_APPROVALS,
            validators,
        };
        env.storage().instance().set(&CONFIG, &config);
        Ok(())
    }

    pub fn propose_upgrade(
        env: Env,
        proposer: Address,
        target: Address,
        new_wasm_hash: BytesN<32>,
        new_version: u32,
        description: Symbol,
    ) -> Result<u64, UpgradeManagerError> {
        proposer.require_auth();
        let config: Config = env
            .storage()
            .instance()
            .get(&CONFIG)
            .ok_or(UpgradeManagerError::ConfigNotFound)?;

        let mut proposals: Map<u64, UpgradeProposal> = env
            .storage()
            .persistent()
            .get(&PROPOSALS)
            .unwrap_or(Map::new(&env));

        let id = proposals.len() as u64;
        let executable_at = env
            .ledger()
            .timestamp()
            .checked_add(config.min_delay)
            .ok_or(UpgradeManagerError::InvalidState)?; // Should not happen

        let proposal = UpgradeProposal {
            target,
            new_wasm_hash,
            new_version,
            description,
            proposer: proposer.clone(),
            created_at: env.ledger().timestamp(),
            executable_at,
            executed: false,
            canceled: false,
            approvals: Vec::new(&env),
        };

        proposals.set(id, proposal);
        env.storage().persistent().set(&PROPOSALS, &proposals);

        env.events()
            .publish((symbol_short!("proposed"), id), proposer);
        Ok(id)
    }

    pub fn approve(
        env: Env,
        validator: Address,
        proposal_id: u64,
    ) -> Result<(), UpgradeManagerError> {
        validator.require_auth();
        let config: Config = env
            .storage()
            .instance()
            .get(&CONFIG)
            .ok_or(UpgradeManagerError::ConfigNotFound)?;

        if !config.validators.contains(&validator) {
            return Err(UpgradeManagerError::NotAValidator);
        }

        let mut proposals: Map<u64, UpgradeProposal> = env
            .storage()
            .persistent()
            .get(&PROPOSALS)
            .ok_or(UpgradeManagerError::ProposalNotFound)?;

        let mut proposal = proposals
            .get(proposal_id)
            .ok_or(UpgradeManagerError::ProposalNotFound)?;

        if proposal.approvals.contains(&validator) {
            return Err(UpgradeManagerError::AlreadyApproved);
        }

        proposal.approvals.push_back(validator);
        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&PROPOSALS, &proposals);
        Ok(())
    }

    pub fn execute(env: Env, proposal_id: u64) -> Result<(), UpgradeManagerError> {
        let mut proposals: Map<u64, UpgradeProposal> = env
            .storage()
            .persistent()
            .get(&PROPOSALS)
            .ok_or(UpgradeManagerError::ProposalNotFound)?;

        let mut proposal = proposals
            .get(proposal_id)
            .ok_or(UpgradeManagerError::ProposalNotFound)?;
        let config: Config = env
            .storage()
            .instance()
            .get(&CONFIG)
            .ok_or(UpgradeManagerError::ConfigNotFound)?;

        if proposal.executed || proposal.canceled {
            return Err(UpgradeManagerError::InvalidState);
        }

        if env.ledger().timestamp() < proposal.executable_at {
            return Err(UpgradeManagerError::TimelockNotExpired);
        }

        if proposal.approvals.len() < config.required_approvals {
            return Err(UpgradeManagerError::NotEnoughApprovals);
        }

        // Call target.upgrade(new_wasm_hash)
        // Note: The UpgradeManager must be the admin of the target contract
        let target_client = TargetContractClient::new(&env, &proposal.target);
        target_client.upgrade(&proposal.new_wasm_hash);

        proposal.executed = true;
        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&PROPOSALS, &proposals);

        env.events()
            .publish((symbol_short!("executed"), proposal_id), ());
        Ok(())
    }
}
