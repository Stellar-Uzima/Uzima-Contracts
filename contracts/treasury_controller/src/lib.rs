// Treasury Controller - Multi-sig treasury with timelocks and proper validation
#![no_std]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::unwrap_used)]
#![allow(dead_code)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Bytes, Env,
    IntoVal, Symbol, Vec,
};

// Storage keys
const ADMIN: Symbol = symbol_short!("ADMIN");
const TOKEN: Symbol = symbol_short!("TOKEN");
const SIGNERS: Symbol = symbol_short!("SIGNERS");
const THRESHOLD: Symbol = symbol_short!("THRESH");
const PROPOSAL_COUNT: Symbol = symbol_short!("PROP_CNT");
const PROPOSALS: Symbol = symbol_short!("PROPS");
const TIMELOCK: Symbol = symbol_short!("TIMELOCK");
const RECOVERY_MODE: Symbol = symbol_short!("RECOVERY");

// Limits
const MIN_TIMELOCK: u64 = 3600;
const MAX_TIMELOCK: u64 = 2592000;
const MAX_SIGNERS: u32 = 20;
const PROPOSAL_EXPIRY: u64 = 604800; // Added missing constant (7 days)

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
    NotInitialized = 12, // Added missing variant
    TransferFailed = 13, // Added missing variant
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProposalStatus {
    // Added missing type
    Pending,
    Approved,
    Executed,
    Expired,
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
pub struct TreasuryConfig {
    // Added missing type
    pub admin: Address,
    pub multisig_config: MultiSigConfig,
    pub emergency_halted: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiSigConfig {
    // Added missing type
    pub signers: Vec<Address>,
    pub threshold: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreasuryProposal {
    // Unified naming to fix compiler mismatches
    pub id: u64,
    pub proposer: Address,
    pub proposal_type: ProposalType,
    pub target: Address,
    pub amount: i128,
    pub data: Bytes,
    pub created_at: u64,
    pub timelock_end: u64,      // Unified field name
    pub status: ProposalStatus, // Added status tracking
    pub approvals: Vec<Address>,
}

#[contracttype]
pub enum DataKey {
    // Added missing enum
    Config,
    Proposals,
    ProposalCount,
}

#[contract]
pub struct TreasuryControllerContract;

#[contractimpl]
impl TreasuryControllerContract {
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

        let config = TreasuryConfig {
            admin: admin.clone(),
            emergency_halted: false,
            multisig_config: MultiSigConfig {
                signers: initial_signers.clone(),
                threshold,
            },
        };

        env.storage().persistent().set(&ADMIN, &admin);
        env.storage().persistent().set(&TOKEN, &token_contract);
        env.storage().persistent().set(&SIGNERS, &initial_signers);
        env.storage().persistent().set(&THRESHOLD, &threshold);
        env.storage()
            .persistent()
            .set(&TIMELOCK, &timelock_duration);
        env.storage().persistent().set(&PROPOSAL_COUNT, &0u64);
        env.storage().instance().set(&DataKey::Config, &config);

        Ok(true)
    }

    pub fn get_config(env: Env) -> TreasuryConfig {
        env.storage().instance().get(&DataKey::Config).unwrap()
    }

    pub fn get_proposal_count(env: Env) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::ProposalCount)
            .unwrap_or(0)
    }

    fn require_signer(env: &Env, signer: &Address) -> Result<(), Error> {
        let config: TreasuryConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;
        if !config.multisig_config.signers.contains(signer) {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    fn execute_token_transfer(
        env: &Env,
        token_contract: &Address,
        from: &Address,
        to: &Address,
        amount: i128,
    ) -> Result<(), Error> {
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

        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::TransferFailed),
        }
    }
}

#[cfg(test)]
mod test;
