#![no_std]

pub mod errors;
pub use errors::Error;
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Map, Symbol, Vec,
};
use upgradeability::UpgradeValidation;

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
    pub is_emergency: bool,
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
    pub emergency_approvals: u32,
}

// Minimal interface for target contracts
#[soroban_sdk::contractclient(name = "TargetContractClient")]
pub trait TargetContract {
    fn upgrade(env: Env, new_wasm_hash: BytesN<32>);
    fn validate_upgrade(env: Env, new_wasm_hash: BytesN<32>) -> UpgradeValidation;
}

#[contractimpl]
impl UpgradeManager {
    pub fn initialize(env: Env, admin: Address, validators: Vec<Address>) -> Result<(), Error> {
        if env.storage().instance().has(&CONFIG) {
            return Err(Error::AlreadyInitialized);
        }
        let config = Config {
            admin,
            min_delay: MIN_DELAY,
            required_approvals: REQUIRED_APPROVALS,
            validators: validators.clone(),
            emergency_approvals: validators.len(), // Default to all validators for emergency
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
        is_emergency: bool,
    ) -> Result<u64, Error> {
        proposer.require_auth();
        let config: Config = env
            .storage()
            .instance()
            .get(&CONFIG)
            .ok_or(Error::ConfigNotFound)?;

        let mut proposals: Map<u64, UpgradeProposal> = env
            .storage()
            .persistent()
            .get(&PROPOSALS)
            .unwrap_or(Map::new(&env));

        let id = proposals.len() as u64;
        let executable_at = if is_emergency {
            env.ledger().timestamp()
        } else {
            env.ledger()
                .timestamp()
                .checked_add(config.min_delay)
                .ok_or(Error::InvalidState)?
        };

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
            is_emergency,
        };

        proposals.set(id, proposal);
        env.storage().persistent().set(&PROPOSALS, &proposals);

        env.events()
            .publish((symbol_short!("proposed"), id), proposer);
        Ok(id)
    }

    pub fn approve(env: Env, validator: Address, proposal_id: u64) -> Result<(), Error> {
        validator.require_auth();
        let config: Config = env
            .storage()
            .instance()
            .get(&CONFIG)
            .ok_or(Error::ConfigNotFound)?;

        if !config.validators.contains(&validator) {
            return Err(Error::NotAValidator);
        }

        let mut proposals: Map<u64, UpgradeProposal> = env
            .storage()
            .persistent()
            .get(&PROPOSALS)
            .ok_or(Error::ProposalNotFound)?;

        let mut proposal = proposals.get(proposal_id).ok_or(Error::ProposalNotFound)?;

        if proposal.approvals.contains(&validator) {
            return Err(Error::AlreadyApproved);
        }

        proposal.approvals.push_back(validator);
        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&PROPOSALS, &proposals);
        Ok(())
    }

    pub fn execute(env: Env, proposal_id: u64) -> Result<(), Error> {
        let mut proposals: Map<u64, UpgradeProposal> = env
            .storage()
            .persistent()
            .get(&PROPOSALS)
            .ok_or(Error::ProposalNotFound)?;

        let mut proposal = proposals.get(proposal_id).ok_or(Error::ProposalNotFound)?;
        let config: Config = env
            .storage()
            .instance()
            .get(&CONFIG)
            .ok_or(Error::ConfigNotFound)?;

        if proposal.executed || proposal.canceled {
            return Err(Error::InvalidState);
        }

        if env.ledger().timestamp() < proposal.executable_at {
            return Err(Error::TimelockNotExpired);
        }

        if proposal.approvals.len() < config.required_approvals {
            return Err(Error::NotEnoughApprovals);
        }

        // Call target.upgrade(new_wasm_hash)
        let target_client = TargetContractClient::new(&env, &proposal.target);
        target_client.upgrade(&proposal.new_wasm_hash);

        proposal.executed = true;
        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&PROPOSALS, &proposals);

        env.events()
            .publish((symbol_short!("executed"), proposal_id), ());
        Ok(())
    }

    pub fn execute_emergency(env: Env, proposal_id: u64) -> Result<(), Error> {
        let mut proposals: Map<u64, UpgradeProposal> = env
            .storage()
            .persistent()
            .get(&PROPOSALS)
            .ok_or(Error::ProposalNotFound)?;

        let mut proposal = proposals.get(proposal_id).ok_or(Error::ProposalNotFound)?;
        let config: Config = env
            .storage()
            .instance()
            .get(&CONFIG)
            .ok_or(Error::ConfigNotFound)?;

        if !proposal.is_emergency {
            return Err(Error::InvalidState);
        }

        if proposal.executed || proposal.canceled {
            return Err(Error::InvalidState);
        }

        if proposal.approvals.len() < config.emergency_approvals {
            return Err(Error::NotEnoughApprovals);
        }

        let target_client = TargetContractClient::new(&env, &proposal.target);
        target_client.upgrade(&proposal.new_wasm_hash);

        proposal.executed = true;
        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&PROPOSALS, &proposals);

        env.events()
            .publish((symbol_short!("emergency"), proposal_id), ());
        Ok(())
    }

    pub fn validate_proposal(env: Env, proposal_id: u64) -> Result<UpgradeValidation, Error> {
        let proposals: Map<u64, UpgradeProposal> = env
            .storage()
            .persistent()
            .get(&PROPOSALS)
            .ok_or(Error::ProposalNotFound)?;

        let proposal = proposals.get(proposal_id).ok_or(Error::ProposalNotFound)?;

        let target_client = TargetContractClient::new(&env, &proposal.target);
        Ok(target_client.validate_upgrade(&proposal.new_wasm_hash))
    }
}
