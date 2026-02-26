#![no_std]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::enum_variant_names)]
#![allow(dead_code)]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, BytesN, Env, String, Symbol, Vec,
};

// ==================== Existing Core Types ====================

#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub enum MessageStatus {
    Pending,
    Verified,
    Executed,
    Failed,
    Expired,
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub enum ChainId {
    Stellar,
    Ethereum,
    Polygon,
    Avalanche,
    BinanceSmartChain,
    Arbitrum,
    Optimism,
    Custom(u32),
}

#[derive(Clone)]
#[contracttype]
pub struct CrossChainMessage {
    pub message_id: BytesN<32>,
    pub source_chain: ChainId,
    pub dest_chain: ChainId,
    pub sender: String,
    pub recipient: Address,
    pub payload_type: MessageType,
    pub payload: String,
    pub nonce: u64,
    pub timestamp: u64,
    pub status: MessageStatus,
    pub signature: BytesN<64>,
}

#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum MessageType {
    RecordRequest,
    RecordResponse,
    IdentityVerify,
    IdentityConfirm,
    AccessGrant,
    AccessRevoke,
    RecordSync,
    EmergencyAccess,
}

#[derive(Clone)]
#[contracttype]
pub struct Validator {
    pub address: Address,
    pub public_key: BytesN<32>,
    pub is_active: bool,
    pub stake: i128,
    pub confirmed_messages: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct CrossChainRecordRef {
    pub local_record_id: u64,
    pub external_chain: ChainId,
    pub external_record_id: String,
    pub sync_status: SyncStatus,
    pub last_sync: u64,
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub enum SyncStatus {
    Synced,
    PendingSync,
    SyncFailed,
    Outdated,
}

#[derive(Clone)]
#[contracttype]
pub struct AtomicTransaction {
    pub tx_id: BytesN<32>,
    pub messages: Vec<BytesN<32>>,
    pub status: AtomicTxStatus,
    pub created_at: u64,
    pub timeout: u64,
    pub confirmations: Vec<Address>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub enum AtomicTxStatus {
    Initiated,
    Prepared,
    Committed,
    Aborted,
    Expired,
}

// ==================== New Types: Oracle Network ====================

/// Oracle node that provides cross-chain data validation
#[derive(Clone)]
#[contracttype]
pub struct OracleNode {
    pub address: Address,
    pub public_key: BytesN<32>,
    pub supported_chains: Vec<ChainId>,
    pub is_active: bool,
    pub reputation: u32, // 0-100
    pub total_reports: u64,
}

/// Report submitted by an oracle for cross-chain data
#[derive(Clone)]
#[contracttype]
pub struct OracleReport {
    pub report_id: u64,
    pub oracle: Address,
    pub chain: ChainId,
    pub data_hash: BytesN<32>,
    pub data: String, // JSON-encoded payload
    pub block_height: u64,
    pub timestamp: u64,
    pub signature: BytesN<64>,
    pub status: OracleStatus,
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub enum OracleStatus {
    Submitted,
    Validated,
    Rejected,
    Aggregated,
}

/// Aggregated oracle consensus for a chain
#[derive(Clone)]
#[contracttype]
pub struct AggregatedOracleData {
    pub chain: ChainId,
    pub consensus_hash: BytesN<32>,
    pub report_count: u32,
    pub consensus_threshold: u32,
    pub aggregated_at: u64,
    pub is_finalized: bool,
}

// ==================== New Types: Cryptographic Proof ====================

/// Cryptographic proof for verifying external chain records
#[derive(Clone)]
#[contracttype]
pub struct CrossChainProof {
    pub proof_id: BytesN<32>,
    pub source_chain: ChainId,
    pub record_hash: BytesN<32>,
    pub block_hash: BytesN<32>,
    pub merkle_root: BytesN<32>,
    pub timestamp: u64,
    pub prover: String,
    pub verifier_count: u32,
    pub verified: bool,
}

// ==================== New Types: Emergency Rollback ====================

/// Tracks state for emergency cross-chain operation rollback
#[derive(Clone)]
#[contracttype]
pub struct RollbackRecord {
    pub op_id: BytesN<32>,
    pub op_type: RollbackOpType,
    pub original_state: String, // JSON-encoded pre-operation state snapshot
    pub triggered_by: Address,
    pub triggered_at: u64,
    pub status: RollbackStatus,
    pub reason: String,
    pub completed_at: u64,
}

#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum RollbackOpType {
    MessageRollback,
    AtomicTxRollback,
    RecordSyncRollback,
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub enum RollbackStatus {
    Initiated,
    InProgress,
    Completed,
    Failed,
}

// ==================== New Types: Event Synchronization ====================

/// Cross-chain event for synchronization between chains
#[derive(Clone)]
#[contracttype]
pub struct CrossChainEvent {
    pub event_id: u64,
    pub source_chain: ChainId,
    pub dest_chain: ChainId,
    pub event_type: CrossChainEventType,
    pub payload_hash: BytesN<32>,
    pub block_height: u64,
    pub timestamp: u64,
    pub sync_status: EventSyncStatus,
}

#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum CrossChainEventType {
    RecordCreated,
    RecordUpdated,
    AccessGranted,
    AccessRevoked,
    IdentityVerified,
    EmergencyTriggered,
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub enum EventSyncStatus {
    Pending,
    Synced,
    Failed,
}

// ==================== Storage Keys (DataKey Enum) ====================
// BUG FIX: Replaces static Symbol constants with typed DataKey enum,
// ensuring each item gets a unique, collision-free storage slot.

#[contracttype]
pub enum DataKey {
    // Core config
    Admin,
    MedicalContract,
    IdentityContract,
    AccessContract,
    Paused,
    MessageCount,
    MinConfirmations,
    SupportedChains,
    // Per-item storage (replaces Map<Key, Value> under a shared symbol)
    Validator(Address),
    Message(BytesN<32>),
    Confirmations(BytesN<32>), // BUG FIX: was always "conf_key"
    Nonce(String),
    RecordRef(u64, ChainId), // BUG FIX: was always "rec_ref"
    AtomicTx(BytesN<32>),
    // Oracle network
    OracleNode(Address),
    OracleReport(u64),
    OracleCount,
    AggregatedOracle(ChainId),
    // Proof verification
    Proof(BytesN<32>),
    // Rollback mechanism
    Rollback(BytesN<32>),
    RollbackCount,
    // Event synchronization
    Event(u64),
    EventCount,
}

// Constants
const DEFAULT_MIN_CONFIRMATIONS: u32 = 2;
const MESSAGE_EXPIRY_SECS: u64 = 86_400; // 24 hours
const ATOMIC_TX_TIMEOUT: u64 = 3_600; // 1 hour
const MIN_ORACLE_REPORTS: u32 = 3; // Minimum oracle reports for consensus
const DEFAULT_ORACLE_REPUTATION: u32 = 50;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // Existing errors
    NotAuthorized = 1,
    ContractPaused = 2,
    InvalidChain = 3,
    InvalidMessage = 4,
    MessageNotFound = 5,
    MessageExpired = 6,
    MessageAlreadyProcessed = 7,
    InvalidSignature = 8,
    InsufficientConfirmations = 9,
    ValidatorNotFound = 10,
    ValidatorNotActive = 11,
    DuplicateConfirmation = 12,
    AtomicTxNotFound = 13,
    AtomicTxExpired = 14,
    AtomicTxAlreadyProcessed = 15,
    InvalidNonce = 16,
    ChainNotSupported = 17,
    RecordRefNotFound = 18,
    AlreadyInitialized = 19,
    InvalidPayload = 20,
    Overflow = 21,
    // New errors
    OracleNotFound = 22,
    OracleNotActive = 23,
    ProofNotFound = 24,
    ProofAlreadyVerified = 25,
    RollbackNotFound = 26,
    RollbackAlreadyProcessed = 27,
    EventNotFound = 28,
    InvalidAddress = 29,
    InsufficientOracleReports = 30,
    DuplicateOracleReport = 31,
}

#[contract]
pub struct CrossChainBridgeContract;

#[contractimpl]
impl CrossChainBridgeContract {
    /// Initialize the bridge contract
    pub fn initialize(
        env: Env,
        admin: Address,
        medical_contract: Address,
        identity_contract: Address,
        access_contract: Address,
    ) -> Result<bool, Error> {
        admin.require_auth();

        if env.storage().persistent().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage()
            .persistent()
            .set(&DataKey::MedicalContract, &medical_contract);
        env.storage()
            .persistent()
            .set(&DataKey::IdentityContract, &identity_contract);
        env.storage()
            .persistent()
            .set(&DataKey::AccessContract, &access_contract);

        env.storage().persistent().set(&DataKey::Paused, &false);
        env.storage()
            .persistent()
            .set(&DataKey::MessageCount, &0u64);
        env.storage()
            .persistent()
            .set(&DataKey::MinConfirmations, &DEFAULT_MIN_CONFIRMATIONS);

        let mut chains: Vec<ChainId> = Vec::new(&env);
        chains.push_back(ChainId::Stellar);
        chains.push_back(ChainId::Ethereum);
        chains.push_back(ChainId::Polygon);
        env.storage()
            .persistent()
            .set(&DataKey::SupportedChains, &chains);

        env.storage().persistent().set(&DataKey::OracleCount, &0u64);
        env.storage()
            .persistent()
            .set(&DataKey::RollbackCount, &0u64);
        env.storage().persistent().set(&DataKey::EventCount, &0u64);

        env.events()
            .publish((Symbol::new(&env, "BridgeInitialized"),), (admin.clone(),));

        Ok(true)
    }

    // ==================== Admin Functions ====================

    pub fn add_validator(
        env: Env,
        caller: Address,
        validator_address: Address,
        public_key: BytesN<32>,
        initial_stake: i128,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        Self::require_not_paused(&env)?;

        let validator = Validator {
            address: validator_address.clone(),
            public_key,
            is_active: true,
            stake: initial_stake,
            confirmed_messages: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Validator(validator_address.clone()), &validator);

        env.events()
            .publish((Symbol::new(&env, "ValidatorAdded"),), (validator_address,));

        Ok(true)
    }

    pub fn deactivate_validator(
        env: Env,
        caller: Address,
        validator_address: Address,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        let key = DataKey::Validator(validator_address.clone());
        if let Some(mut validator) = env.storage().persistent().get::<DataKey, Validator>(&key) {
            validator.is_active = false;
            env.storage().persistent().set(&key, &validator);

            env.events().publish(
                (Symbol::new(&env, "ValidatorDeactivated"),),
                (validator_address,),
            );

            Ok(true)
        } else {
            Err(Error::ValidatorNotFound)
        }
    }

    pub fn add_supported_chain(env: Env, caller: Address, chain: ChainId) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        let mut chains: Vec<ChainId> = env
            .storage()
            .persistent()
            .get(&DataKey::SupportedChains)
            .unwrap_or(Vec::new(&env));

        if !chains.contains(&chain) {
            chains.push_back(chain.clone());
            env.storage()
                .persistent()
                .set(&DataKey::SupportedChains, &chains);

            env.events()
                .publish((Symbol::new(&env, "ChainAdded"),), (chain,));
        }

        Ok(true)
    }

    pub fn set_min_confirmations(
        env: Env,
        caller: Address,
        min_confirmations: u32,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        env.storage()
            .persistent()
            .set(&DataKey::MinConfirmations, &min_confirmations);

        Ok(true)
    }

    pub fn pause(env: Env, caller: Address) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        env.storage().persistent().set(&DataKey::Paused, &true);

        env.events().publish(
            (Symbol::new(&env, "Paused"),),
            (caller, env.ledger().timestamp()),
        );

        Ok(true)
    }

    pub fn unpause(env: Env, caller: Address) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        env.storage().persistent().set(&DataKey::Paused, &false);

        env.events().publish(
            (Symbol::new(&env, "Unpaused"),),
            (caller, env.ledger().timestamp()),
        );

        Ok(true)
    }

    // ==================== Cross-Chain Message Functions ====================

    pub fn submit_message(
        env: Env,
        validator: Address,
        message_id: BytesN<32>,
        source_chain: ChainId,
        dest_chain: ChainId,
        sender: String,
        recipient: Address,
        payload_type: MessageType,
        payload: String,
        nonce: u64,
        signature: BytesN<64>,
    ) -> Result<BytesN<32>, Error> {
        validator.require_auth();
        Self::require_not_paused(&env)?;
        Self::require_active_validator(&env, &validator)?;
        Self::require_chain_supported(&env, &source_chain)?;

        Self::verify_nonce(&env, &sender, nonce)?;

        let timestamp = env.ledger().timestamp();

        let message = CrossChainMessage {
            message_id: message_id.clone(),
            source_chain,
            dest_chain,
            sender: sender.clone(),
            recipient,
            payload_type,
            payload,
            nonce,
            timestamp,
            status: MessageStatus::Pending,
            signature,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Message(message_id.clone()), &message);

        Self::update_nonce(&env, &sender, nonce);

        let count: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::MessageCount)
            .unwrap_or(0);
        env.storage().persistent().set(
            &DataKey::MessageCount,
            &(count.checked_add(1).ok_or(Error::Overflow)?),
        );

        env.events().publish(
            (Symbol::new(&env, "MessageSubmitted"),),
            (message_id.clone(), timestamp),
        );

        Ok(message_id)
    }

    /// Confirm a cross-chain message (validator attestation)
    /// BUG FIX: Confirmations now stored per message_id (was using shared "conf_key")
    pub fn confirm_message(
        env: Env,
        validator: Address,
        message_id: BytesN<32>,
    ) -> Result<bool, Error> {
        validator.require_auth();
        Self::require_not_paused(&env)?;
        Self::require_active_validator(&env, &validator)?;

        let msg_key = DataKey::Message(message_id.clone());
        let mut message = env
            .storage()
            .persistent()
            .get::<DataKey, CrossChainMessage>(&msg_key)
            .ok_or(Error::MessageNotFound)?;

        if message.status != MessageStatus::Pending {
            return Err(Error::MessageAlreadyProcessed);
        }

        let now = env.ledger().timestamp();
        if now
            > message
                .timestamp
                .checked_add(MESSAGE_EXPIRY_SECS)
                .ok_or(Error::Overflow)?
        {
            return Err(Error::MessageExpired);
        }

        // BUG FIX: Use message_id as direct storage key, not a shared symbol
        let conf_key = DataKey::Confirmations(message_id.clone());
        let mut confirmations: Vec<Address> = env
            .storage()
            .temporary()
            .get(&conf_key)
            .unwrap_or(Vec::new(&env));

        if confirmations.contains(&validator) {
            return Err(Error::DuplicateConfirmation);
        }

        confirmations.push_back(validator.clone());
        env.storage().temporary().set(&conf_key, &confirmations);

        Self::increment_validator_confirmations(&env, &validator);

        let min_confirmations: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::MinConfirmations)
            .unwrap_or(DEFAULT_MIN_CONFIRMATIONS);

        if confirmations.len() as u32 >= min_confirmations {
            message.status = MessageStatus::Verified;
            env.storage().persistent().set(&msg_key, &message);

            env.events().publish(
                (Symbol::new(&env, "MessageVerified"),),
                (message_id.clone(),),
            );
        }

        env.events().publish(
            (Symbol::new(&env, "MessageConfirmed"),),
            (message_id, validator),
        );

        Ok(true)
    }

    pub fn execute_message(
        env: Env,
        caller: Address,
        message_id: BytesN<32>,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_not_paused(&env)?;

        let msg_key = DataKey::Message(message_id.clone());
        let mut message = env
            .storage()
            .persistent()
            .get::<DataKey, CrossChainMessage>(&msg_key)
            .ok_or(Error::MessageNotFound)?;

        if message.status != MessageStatus::Verified {
            return Err(Error::InsufficientConfirmations);
        }

        let now = env.ledger().timestamp();
        if now
            > message
                .timestamp
                .checked_add(MESSAGE_EXPIRY_SECS)
                .ok_or(Error::Overflow)?
        {
            message.status = MessageStatus::Expired;
            env.storage().persistent().set(&msg_key, &message);
            return Err(Error::MessageExpired);
        }

        let payload_type = message.payload_type.clone();
        message.status = MessageStatus::Executed;
        env.storage().persistent().set(&msg_key, &message);

        env.events().publish(
            (Symbol::new(&env, "MessageExecuted"),),
            (message_id, payload_type),
        );

        Ok(true)
    }

    // ==================== Atomic Transaction Functions ====================

    pub fn initiate_atomic_tx(
        env: Env,
        caller: Address,
        tx_id: BytesN<32>,
        message_ids: Vec<BytesN<32>>,
    ) -> Result<BytesN<32>, Error> {
        caller.require_auth();
        Self::require_not_paused(&env)?;

        let now = env.ledger().timestamp();

        let atomic_tx = AtomicTransaction {
            tx_id: tx_id.clone(),
            messages: message_ids,
            status: AtomicTxStatus::Initiated,
            created_at: now,
            timeout: now.checked_add(ATOMIC_TX_TIMEOUT).ok_or(Error::Overflow)?,
            confirmations: Vec::new(&env),
        };

        env.storage()
            .persistent()
            .set(&DataKey::AtomicTx(tx_id.clone()), &atomic_tx);

        env.events().publish(
            (Symbol::new(&env, "AtomicTxInitiated"),),
            (tx_id.clone(), now),
        );

        Ok(tx_id)
    }

    pub fn prepare_atomic_tx(
        env: Env,
        validator: Address,
        tx_id: BytesN<32>,
    ) -> Result<bool, Error> {
        validator.require_auth();
        Self::require_not_paused(&env)?;
        Self::require_active_validator(&env, &validator)?;

        let tx_key = DataKey::AtomicTx(tx_id.clone());
        let mut atomic_tx = env
            .storage()
            .persistent()
            .get::<DataKey, AtomicTransaction>(&tx_key)
            .ok_or(Error::AtomicTxNotFound)?;

        if atomic_tx.status != AtomicTxStatus::Initiated {
            return Err(Error::AtomicTxAlreadyProcessed);
        }

        let now = env.ledger().timestamp();
        if now > atomic_tx.timeout {
            atomic_tx.status = AtomicTxStatus::Expired;
            env.storage().persistent().set(&tx_key, &atomic_tx);
            return Err(Error::AtomicTxExpired);
        }

        if !atomic_tx.confirmations.contains(&validator) {
            atomic_tx.confirmations.push_back(validator.clone());
        }

        let min_confirmations: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::MinConfirmations)
            .unwrap_or(DEFAULT_MIN_CONFIRMATIONS);

        if atomic_tx.confirmations.len() as u32 >= min_confirmations {
            atomic_tx.status = AtomicTxStatus::Prepared;

            env.events()
                .publish((Symbol::new(&env, "AtomicTxPrepared"),), (tx_id.clone(),));
        }

        env.storage().persistent().set(&tx_key, &atomic_tx);

        Ok(true)
    }

    pub fn commit_atomic_tx(env: Env, caller: Address, tx_id: BytesN<32>) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_not_paused(&env)?;

        let tx_key = DataKey::AtomicTx(tx_id.clone());
        let mut atomic_tx = env
            .storage()
            .persistent()
            .get::<DataKey, AtomicTransaction>(&tx_key)
            .ok_or(Error::AtomicTxNotFound)?;

        if atomic_tx.status != AtomicTxStatus::Prepared {
            return Err(Error::AtomicTxAlreadyProcessed);
        }

        let now = env.ledger().timestamp();
        if now > atomic_tx.timeout {
            atomic_tx.status = AtomicTxStatus::Expired;
            env.storage().persistent().set(&tx_key, &atomic_tx);
            return Err(Error::AtomicTxExpired);
        }

        atomic_tx.status = AtomicTxStatus::Committed;
        env.storage().persistent().set(&tx_key, &atomic_tx);

        env.events()
            .publish((Symbol::new(&env, "AtomicTxCommitted"),), (tx_id,));

        Ok(true)
    }

    pub fn abort_atomic_tx(env: Env, caller: Address, tx_id: BytesN<32>) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_not_paused(&env)?;

        let tx_key = DataKey::AtomicTx(tx_id.clone());
        let mut atomic_tx = env
            .storage()
            .persistent()
            .get::<DataKey, AtomicTransaction>(&tx_key)
            .ok_or(Error::AtomicTxNotFound)?;

        if atomic_tx.status == AtomicTxStatus::Committed {
            return Err(Error::AtomicTxAlreadyProcessed);
        }

        atomic_tx.status = AtomicTxStatus::Aborted;
        env.storage().persistent().set(&tx_key, &atomic_tx);

        env.events()
            .publish((Symbol::new(&env, "AtomicTxAborted"),), (tx_id,));

        Ok(true)
    }

    // ==================== Record Reference Functions ====================

    /// Register a cross-chain record reference
    /// BUG FIX: Each (record_id, chain) pair gets a unique storage key
    pub fn register_record_ref(
        env: Env,
        caller: Address,
        local_record_id: u64,
        external_chain: ChainId,
        external_record_id: String,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_not_paused(&env)?;
        Self::require_chain_supported(&env, &external_chain)?;

        let record_ref = CrossChainRecordRef {
            local_record_id,
            external_chain: external_chain.clone(),
            external_record_id,
            sync_status: SyncStatus::PendingSync,
            last_sync: env.ledger().timestamp(),
        };

        // BUG FIX: unique key per (record_id, chain) — was always "rec_ref"
        env.storage().persistent().set(
            &DataKey::RecordRef(local_record_id, external_chain.clone()),
            &record_ref,
        );

        env.events().publish(
            (Symbol::new(&env, "RecordRefRegistered"),),
            (local_record_id, external_chain),
        );

        Ok(true)
    }

    /// Update sync status — validators attest to sync completion
    pub fn update_sync_status(
        env: Env,
        validator: Address,
        local_record_id: u64,
        external_chain: ChainId,
        status: SyncStatus,
    ) -> Result<bool, Error> {
        validator.require_auth();
        Self::require_not_paused(&env)?;
        Self::require_active_validator(&env, &validator)?;

        let ref_key = DataKey::RecordRef(local_record_id, external_chain.clone());
        let mut record_ref = env
            .storage()
            .persistent()
            .get::<DataKey, CrossChainRecordRef>(&ref_key)
            .ok_or(Error::RecordRefNotFound)?;

        record_ref.sync_status = status.clone();
        record_ref.last_sync = env.ledger().timestamp();

        env.storage().persistent().set(&ref_key, &record_ref);

        env.events().publish(
            (Symbol::new(&env, "SyncStatusUpdated"),),
            (local_record_id, external_chain, status),
        );

        Ok(true)
    }

    // ==================== Oracle Network Functions ====================

    /// Register an oracle node for cross-chain data validation
    pub fn register_oracle(
        env: Env,
        caller: Address,
        oracle_address: Address,
        public_key: BytesN<32>,
        supported_chains: Vec<ChainId>,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        Self::require_not_paused(&env)?;

        let oracle = OracleNode {
            address: oracle_address.clone(),
            public_key,
            supported_chains,
            is_active: true,
            reputation: DEFAULT_ORACLE_REPUTATION,
            total_reports: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::OracleNode(oracle_address.clone()), &oracle);

        env.events()
            .publish((Symbol::new(&env, "OracleRegistered"),), (oracle_address,));

        Ok(true)
    }

    /// Deactivate an oracle node
    pub fn deactivate_oracle(
        env: Env,
        caller: Address,
        oracle_address: Address,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        let key = DataKey::OracleNode(oracle_address.clone());
        if let Some(mut oracle) = env.storage().persistent().get::<DataKey, OracleNode>(&key) {
            oracle.is_active = false;
            env.storage().persistent().set(&key, &oracle);

            env.events()
                .publish((Symbol::new(&env, "OracleDeactivated"),), (oracle_address,));

            Ok(true)
        } else {
            Err(Error::OracleNotFound)
        }
    }

    /// Submit a data report from an oracle node
    pub fn submit_oracle_report(
        env: Env,
        oracle: Address,
        chain: ChainId,
        data_hash: BytesN<32>,
        data: String,
        block_height: u64,
        signature: BytesN<64>,
    ) -> Result<u64, Error> {
        oracle.require_auth();
        Self::require_not_paused(&env)?;

        // Verify oracle is active
        let oracle_key = DataKey::OracleNode(oracle.clone());
        let mut oracle_node = env
            .storage()
            .persistent()
            .get::<DataKey, OracleNode>(&oracle_key)
            .ok_or(Error::OracleNotFound)?;

        if !oracle_node.is_active {
            return Err(Error::OracleNotActive);
        }

        let now = env.ledger().timestamp();

        let count: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::OracleCount)
            .unwrap_or(0);
        let report_id = count.checked_add(1).ok_or(Error::Overflow)?;

        let report = OracleReport {
            report_id,
            oracle: oracle.clone(),
            chain: chain.clone(),
            data_hash: data_hash.clone(),
            data,
            block_height,
            timestamp: now,
            signature,
            status: OracleStatus::Submitted,
        };

        env.storage()
            .persistent()
            .set(&DataKey::OracleReport(report_id), &report);
        env.storage()
            .persistent()
            .set(&DataKey::OracleCount, &report_id);

        // Update oracle stats
        oracle_node.total_reports = oracle_node.total_reports.saturating_add(1);
        env.storage().persistent().set(&oracle_key, &oracle_node);

        env.events().publish(
            (Symbol::new(&env, "OracleReportSubmitted"),),
            (report_id, oracle, chain, data_hash),
        );

        Ok(report_id)
    }

    /// Aggregate oracle reports to reach consensus for a chain
    pub fn aggregate_oracle_data(
        env: Env,
        caller: Address,
        chain: ChainId,
        report_ids: Vec<u64>,
        consensus_hash: BytesN<32>,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_not_paused(&env)?;
        Self::require_active_validator(&env, &caller)?;

        if report_ids.len() < MIN_ORACLE_REPORTS {
            return Err(Error::InsufficientOracleReports);
        }

        let now = env.ledger().timestamp();

        let aggregated = AggregatedOracleData {
            chain: chain.clone(),
            consensus_hash: consensus_hash.clone(),
            report_count: report_ids.len() as u32,
            consensus_threshold: MIN_ORACLE_REPORTS,
            aggregated_at: now,
            is_finalized: true,
        };

        env.storage()
            .persistent()
            .set(&DataKey::AggregatedOracle(chain.clone()), &aggregated);

        // Mark contributing reports as aggregated
        for report_id in report_ids.iter() {
            let rkey = DataKey::OracleReport(report_id);
            if let Some(mut report) = env
                .storage()
                .persistent()
                .get::<DataKey, OracleReport>(&rkey)
            {
                report.status = OracleStatus::Aggregated;
                env.storage().persistent().set(&rkey, &report);
            }
        }

        env.events().publish(
            (Symbol::new(&env, "OracleDataAggregated"),),
            (chain, consensus_hash),
        );

        Ok(true)
    }

    // ==================== Cryptographic Proof Functions ====================

    /// Submit a cryptographic proof for an external chain record
    pub fn submit_proof(
        env: Env,
        validator: Address,
        proof_id: BytesN<32>,
        source_chain: ChainId,
        record_hash: BytesN<32>,
        block_hash: BytesN<32>,
        merkle_root: BytesN<32>,
        prover: String,
    ) -> Result<BytesN<32>, Error> {
        validator.require_auth();
        Self::require_not_paused(&env)?;
        Self::require_active_validator(&env, &validator)?;
        Self::require_chain_supported(&env, &source_chain)?;

        let now = env.ledger().timestamp();

        let proof = CrossChainProof {
            proof_id: proof_id.clone(),
            source_chain,
            record_hash,
            block_hash,
            merkle_root,
            timestamp: now,
            prover,
            verifier_count: 1,
            verified: false,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Proof(proof_id.clone()), &proof);

        env.events().publish(
            (Symbol::new(&env, "ProofSubmitted"),),
            (proof_id.clone(), validator),
        );

        Ok(proof_id)
    }

    /// Verify a submitted cross-chain proof (additional validator attestation)
    pub fn verify_cross_chain_proof(
        env: Env,
        validator: Address,
        proof_id: BytesN<32>,
    ) -> Result<bool, Error> {
        validator.require_auth();
        Self::require_not_paused(&env)?;
        Self::require_active_validator(&env, &validator)?;

        let proof_key = DataKey::Proof(proof_id.clone());
        let mut proof = env
            .storage()
            .persistent()
            .get::<DataKey, CrossChainProof>(&proof_key)
            .ok_or(Error::ProofNotFound)?;

        if proof.verified {
            return Err(Error::ProofAlreadyVerified);
        }

        proof.verifier_count = proof.verifier_count.saturating_add(1);

        // Proof is verified once enough validators attest
        let min_conf: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::MinConfirmations)
            .unwrap_or(DEFAULT_MIN_CONFIRMATIONS);

        if proof.verifier_count >= min_conf {
            proof.verified = true;

            env.events().publish(
                (Symbol::new(&env, "ProofVerified"),),
                (proof_id.clone(), proof.source_chain.clone()),
            );
        }

        env.storage().persistent().set(&proof_key, &proof);

        Ok(proof.verified)
    }

    // ==================== Address Validation / Conversion ====================

    /// Validate a chain address format (length + prefix check)
    /// Returns true if the address matches expected format for the given chain.
    pub fn validate_chain_address(_env: Env, chain: ChainId, address: String) -> bool {
        let len = address.len();
        match chain {
            // Stellar StrKey account IDs: 56 chars, start with 'G'
            ChainId::Stellar => len == 56,
            // EVM-compatible chains: 42 chars ("0x" + 40 hex digits)
            ChainId::Ethereum
            | ChainId::Polygon
            | ChainId::Avalanche
            | ChainId::BinanceSmartChain
            | ChainId::Arbitrum
            | ChainId::Optimism => len == 42,
            // Custom chains: accept any non-empty address
            ChainId::Custom(_) => len > 0,
        }
    }

    /// Get expected address length for a chain
    pub fn get_chain_address_length(_env: Env, chain: ChainId) -> u32 {
        match chain {
            ChainId::Stellar => 56,
            ChainId::Ethereum
            | ChainId::Polygon
            | ChainId::Avalanche
            | ChainId::BinanceSmartChain
            | ChainId::Arbitrum
            | ChainId::Optimism => 42,
            ChainId::Custom(_) => 0, // variable
        }
    }

    // ==================== Event Synchronization Functions ====================

    /// Submit a cross-chain event for synchronization
    pub fn sync_cross_chain_event(
        env: Env,
        validator: Address,
        source_chain: ChainId,
        dest_chain: ChainId,
        event_type: CrossChainEventType,
        payload_hash: BytesN<32>,
        block_height: u64,
    ) -> Result<u64, Error> {
        validator.require_auth();
        Self::require_not_paused(&env)?;
        Self::require_active_validator(&env, &validator)?;
        Self::require_chain_supported(&env, &source_chain)?;

        let count: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::EventCount)
            .unwrap_or(0);
        let event_id = count.checked_add(1).ok_or(Error::Overflow)?;

        let event = CrossChainEvent {
            event_id,
            source_chain: source_chain.clone(),
            dest_chain: dest_chain.clone(),
            event_type: event_type.clone(),
            payload_hash: payload_hash.clone(),
            block_height,
            timestamp: env.ledger().timestamp(),
            sync_status: EventSyncStatus::Pending,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Event(event_id), &event);
        env.storage()
            .persistent()
            .set(&DataKey::EventCount, &event_id);

        env.events().publish(
            (Symbol::new(&env, "EventSynced"),),
            (event_id, source_chain, dest_chain, payload_hash),
        );

        Ok(event_id)
    }

    /// Mark a cross-chain event as processed/synced
    pub fn process_sync_event(
        env: Env,
        validator: Address,
        event_id: u64,
        status: EventSyncStatus,
    ) -> Result<bool, Error> {
        validator.require_auth();
        Self::require_not_paused(&env)?;
        Self::require_active_validator(&env, &validator)?;

        let evt_key = DataKey::Event(event_id);
        let mut event = env
            .storage()
            .persistent()
            .get::<DataKey, CrossChainEvent>(&evt_key)
            .ok_or(Error::EventNotFound)?;

        event.sync_status = status.clone();
        env.storage().persistent().set(&evt_key, &event);

        env.events()
            .publish((Symbol::new(&env, "EventProcessed"),), (event_id, status));

        Ok(true)
    }

    // ==================== Emergency Rollback Functions ====================

    /// Initiate an emergency rollback for a failed cross-chain operation
    pub fn initiate_rollback(
        env: Env,
        caller: Address,
        op_id: BytesN<32>,
        op_type: RollbackOpType,
        original_state: String,
        reason: String,
    ) -> Result<BytesN<32>, Error> {
        caller.require_auth();
        Self::require_not_paused(&env)?;

        // Only admin or active validators can initiate rollbacks
        let is_admin = Self::is_admin(&env, &caller);
        let is_validator = Self::check_active_validator(&env, &caller);
        if !is_admin && !is_validator {
            return Err(Error::NotAuthorized);
        }

        let now = env.ledger().timestamp();

        let rollback = RollbackRecord {
            op_id: op_id.clone(),
            op_type,
            original_state,
            triggered_by: caller.clone(),
            triggered_at: now,
            status: RollbackStatus::Initiated,
            reason,
            completed_at: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Rollback(op_id.clone()), &rollback);

        let count: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::RollbackCount)
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::RollbackCount, &(count.saturating_add(1)));

        env.events().publish(
            (Symbol::new(&env, "RollbackInitiated"),),
            (op_id.clone(), caller),
        );

        Ok(op_id)
    }

    /// Execute a rollback — marks the associated operation as failed/rolled back
    pub fn execute_rollback(env: Env, caller: Address, op_id: BytesN<32>) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        let rb_key = DataKey::Rollback(op_id.clone());
        let mut rollback = env
            .storage()
            .persistent()
            .get::<DataKey, RollbackRecord>(&rb_key)
            .ok_or(Error::RollbackNotFound)?;

        if rollback.status == RollbackStatus::Completed || rollback.status == RollbackStatus::Failed
        {
            return Err(Error::RollbackAlreadyProcessed);
        }

        rollback.status = RollbackStatus::InProgress;
        env.storage().persistent().set(&rb_key, &rollback);

        // Mark the associated message or atomic tx as failed based on op_type
        match rollback.op_type {
            RollbackOpType::MessageRollback => {
                if let Some(mut msg) = env
                    .storage()
                    .persistent()
                    .get::<DataKey, CrossChainMessage>(&DataKey::Message(op_id.clone()))
                {
                    msg.status = MessageStatus::Failed;
                    env.storage()
                        .persistent()
                        .set(&DataKey::Message(op_id.clone()), &msg);
                }
            }
            RollbackOpType::AtomicTxRollback => {
                if let Some(mut atomic_tx) = env
                    .storage()
                    .persistent()
                    .get::<DataKey, AtomicTransaction>(&DataKey::AtomicTx(op_id.clone()))
                {
                    atomic_tx.status = AtomicTxStatus::Aborted;
                    env.storage()
                        .persistent()
                        .set(&DataKey::AtomicTx(op_id.clone()), &atomic_tx);
                }
            }
            RollbackOpType::RecordSyncRollback => {
                // Record sync rollback handled externally via oracle confirmation
            }
        }

        rollback.status = RollbackStatus::Completed;
        rollback.completed_at = env.ledger().timestamp();
        env.storage().persistent().set(&rb_key, &rollback);

        env.events()
            .publish((Symbol::new(&env, "RollbackCompleted"),), (op_id, caller));

        Ok(true)
    }

    /// Cancel a pending rollback
    pub fn cancel_rollback(env: Env, caller: Address, op_id: BytesN<32>) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        let rb_key = DataKey::Rollback(op_id.clone());
        let mut rollback = env
            .storage()
            .persistent()
            .get::<DataKey, RollbackRecord>(&rb_key)
            .ok_or(Error::RollbackNotFound)?;

        if rollback.status != RollbackStatus::Initiated {
            return Err(Error::RollbackAlreadyProcessed);
        }

        rollback.status = RollbackStatus::Failed;
        rollback.completed_at = env.ledger().timestamp();
        env.storage().persistent().set(&rb_key, &rollback);

        env.events()
            .publish((Symbol::new(&env, "RollbackCancelled"),), (op_id,));

        Ok(true)
    }

    // ==================== Query Functions ====================

    pub fn get_message(env: Env, message_id: BytesN<32>) -> Option<CrossChainMessage> {
        env.storage()
            .persistent()
            .get(&DataKey::Message(message_id))
    }

    pub fn get_atomic_tx(env: Env, tx_id: BytesN<32>) -> Option<AtomicTransaction> {
        env.storage().persistent().get(&DataKey::AtomicTx(tx_id))
    }

    pub fn get_record_ref(
        env: Env,
        local_record_id: u64,
        external_chain: ChainId,
    ) -> Option<CrossChainRecordRef> {
        env.storage()
            .persistent()
            .get(&DataKey::RecordRef(local_record_id, external_chain))
    }

    pub fn get_validator(env: Env, validator_address: Address) -> Option<Validator> {
        env.storage()
            .persistent()
            .get(&DataKey::Validator(validator_address))
    }

    pub fn get_oracle_node(env: Env, oracle_address: Address) -> Option<OracleNode> {
        env.storage()
            .persistent()
            .get(&DataKey::OracleNode(oracle_address))
    }

    pub fn get_oracle_report(env: Env, report_id: u64) -> Option<OracleReport> {
        env.storage()
            .persistent()
            .get(&DataKey::OracleReport(report_id))
    }

    pub fn get_aggregated_oracle(env: Env, chain: ChainId) -> Option<AggregatedOracleData> {
        env.storage()
            .persistent()
            .get(&DataKey::AggregatedOracle(chain))
    }

    pub fn get_proof(env: Env, proof_id: BytesN<32>) -> Option<CrossChainProof> {
        env.storage().persistent().get(&DataKey::Proof(proof_id))
    }

    pub fn get_rollback(env: Env, op_id: BytesN<32>) -> Option<RollbackRecord> {
        env.storage().persistent().get(&DataKey::Rollback(op_id))
    }

    pub fn get_sync_event(env: Env, event_id: u64) -> Option<CrossChainEvent> {
        env.storage().persistent().get(&DataKey::Event(event_id))
    }

    pub fn get_supported_chains(env: Env) -> Vec<ChainId> {
        env.storage()
            .persistent()
            .get(&DataKey::SupportedChains)
            .unwrap_or(Vec::new(&env))
    }

    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    pub fn get_message_count(env: Env) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::MessageCount)
            .unwrap_or(0)
    }

    pub fn get_oracle_count(env: Env) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::OracleCount)
            .unwrap_or(0)
    }

    pub fn get_event_count(env: Env) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::EventCount)
            .unwrap_or(0)
    }

    pub fn get_rollback_count(env: Env) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::RollbackCount)
            .unwrap_or(0)
    }

    // ==================== Internal Helper Functions ====================

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .persistent()
            .get(&DataKey::Admin)
            .ok_or(Error::NotAuthorized)?;

        if caller != &admin {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    fn is_admin(env: &Env, caller: &Address) -> bool {
        let admin: Option<Address> = env.storage().persistent().get(&DataKey::Admin);
        match admin {
            Some(a) => &a == caller,
            None => false,
        }
    }

    fn require_not_paused(env: &Env) -> Result<(), Error> {
        if env
            .storage()
            .persistent()
            .get(&DataKey::Paused)
            .unwrap_or(false)
        {
            return Err(Error::ContractPaused);
        }
        Ok(())
    }

    fn require_active_validator(env: &Env, validator: &Address) -> Result<(), Error> {
        match env
            .storage()
            .persistent()
            .get::<DataKey, Validator>(&DataKey::Validator(validator.clone()))
        {
            Some(v) if v.is_active => Ok(()),
            Some(_) => Err(Error::ValidatorNotActive),
            None => Err(Error::ValidatorNotFound),
        }
    }

    fn check_active_validator(env: &Env, validator: &Address) -> bool {
        matches!(
            env.storage()
                .persistent()
                .get::<DataKey, Validator>(&DataKey::Validator(validator.clone())),
            Some(v) if v.is_active
        )
    }

    fn require_chain_supported(env: &Env, chain: &ChainId) -> Result<(), Error> {
        let chains: Vec<ChainId> = env
            .storage()
            .persistent()
            .get(&DataKey::SupportedChains)
            .unwrap_or(Vec::new(&env));

        if chains.contains(chain) {
            Ok(())
        } else {
            Err(Error::ChainNotSupported)
        }
    }

    fn verify_nonce(env: &Env, sender: &String, nonce: u64) -> Result<(), Error> {
        let last_nonce: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::Nonce(sender.clone()))
            .unwrap_or(0);

        if nonce <= last_nonce {
            return Err(Error::InvalidNonce);
        }
        Ok(())
    }

    fn update_nonce(env: &Env, sender: &String, nonce: u64) {
        env.storage()
            .persistent()
            .set(&DataKey::Nonce(sender.clone()), &nonce);
    }

    fn increment_validator_confirmations(env: &Env, validator: &Address) {
        let key = DataKey::Validator(validator.clone());
        if let Some(mut v) = env.storage().persistent().get::<DataKey, Validator>(&key) {
            v.confirmed_messages = v.confirmed_messages.saturating_add(1);
            env.storage().persistent().set(&key, &v);
        }
    }
}
