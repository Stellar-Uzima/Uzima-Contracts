#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Map, Symbol, Vec,
};
use upgradeability::{storage as up_storage, UpgradeError, UpgradeHistory};

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

#[contractimpl]
impl UpgradeManager {
    pub fn initialize(env: Env, admin: Address, validators: Vec<Address>) {
        if env.storage().instance().has(&CONFIG) {
            panic!("Already initialized");
        }
        let config = Config {
            admin,
            min_delay: MIN_DELAY,
            required_approvals: REQUIRED_APPROVALS,
            validators,
        };
        env.storage().instance().set(&CONFIG, &config);
    }

    pub fn propose_upgrade(
        env: Env,
        proposer: Address,
        target: Address,
        new_wasm_hash: BytesN<32>,
        new_version: u32,
        description: Symbol,
    ) -> u64 {
        proposer.require_auth();
        let config: Config = env.storage().instance().get(&CONFIG).unwrap();
        
        let mut proposals: Map<u64, UpgradeProposal> = env
            .storage()
            .persistent()
            .get(&PROPOSALS)
            .unwrap_or(Map::new(&env));
        
        let id = proposals.len() as u64;
        let proposal = UpgradeProposal {
            target,
            new_wasm_hash,
            new_version,
            description,
            proposer: proposer.clone(),
            created_at: env.ledger().timestamp(),
            executable_at: env.ledger().timestamp() + config.min_delay,
            executed: false,
            canceled: false,
            approvals: Vec::new(&env),
        };
        
        proposals.set(id, proposal);
        env.storage().persistent().set(&PROPOSALS, &proposals);
        
        env.events().publish((symbol_short!("proposed"), id), proposer);
        id
    }

    pub fn approve(env: Env, validator: Address, proposal_id: u64) {
        validator.require_auth();
        let config: Config = env.storage().instance().get(&CONFIG).unwrap();
        
        if !config.validators.contains(&validator) {
            panic!("Not a validator");
        }

        let mut proposals: Map<u64, UpgradeProposal> = env
            .storage()
            .persistent()
            .get(&PROPOSALS)
            .unwrap();
        
        let mut proposal = proposals.get(proposal_id).expect("Proposal not found");
        
        if proposal.approvals.contains(&validator) {
            panic!("Already approved");
        }
        
        proposal.approvals.push_back(validator);
        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&PROPOSALS, &proposals);
    }

    pub fn execute(env: Env, proposal_id: u64) {
        let mut proposals: Map<u64, UpgradeProposal> = env
            .storage()
            .persistent()
            .get(&PROPOSALS)
            .unwrap();
        
        let mut proposal = proposals.get(proposal_id).expect("Proposal not found");
        let config: Config = env.storage().instance().get(&CONFIG).unwrap();

        if proposal.executed || proposal.canceled {
            panic!("Invalid state");
        }

        if env.ledger().timestamp() < proposal.executable_at {
            panic!("Timelock not expired");
        }

        if (proposal.approvals.len() as u32) < config.required_approvals {
            panic!("Not enough approvals");
        }

        // Call target.upgrade(new_wasm_hash)
        // Note: The UpgradeManager must be the admin of the target contract
        let target_client = TargetContractClient::new(&env, &proposal.target);
        target_client.upgrade(&proposal.new_wasm_hash);

        proposal.executed = true;
        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&PROPOSALS, &proposals);
        
        env.events().publish((symbol_short!("executed"), proposal_id), ());
    }
}

// Minimal interface for target contracts
#[soroban_sdk::contractclient(name = "TargetContractClient")]
pub trait TargetContract {
    fn upgrade(env: Env, new_wasm_hash: BytesN<32>);
}
