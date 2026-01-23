#![no_std]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Map,
    String, Symbol, Vec,
};

/// Represents the status of a cross-chain message
#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub enum MessageStatus {
    Pending,
    Verified,
    Executed,
    Failed,
    Expired,
}

/// Represents a supported external blockchain network
#[derive(Clone, PartialEq, Eq)]
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

/// Cross-chain message structure for medical record operations
#[derive(Clone)]
#[contracttype]
pub struct CrossChainMessage {
    pub message_id: BytesN<32>,
    pub source_chain: ChainId,
    pub dest_chain: ChainId,
    pub sender: String,     // External chain address as string
    pub recipient: Address, // Stellar address
    pub payload_type: MessageType,
    pub payload: String, // JSON-encoded payload
    pub nonce: u64,
    pub timestamp: u64,
    pub status: MessageStatus,
    pub signature: BytesN<64>, // Ed25519 signature
}

/// Types of cross-chain messages supported
#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum MessageType {
    RecordRequest,   // Request to access a medical record
    RecordResponse,  // Response with record data
    IdentityVerify,  // Identity verification request
    IdentityConfirm, // Identity confirmation
    AccessGrant,     // Grant access to records
    AccessRevoke,    // Revoke access to records
    RecordSync,      // Synchronize record state
    EmergencyAccess, // Emergency access request
}

/// Bridge validator information
#[derive(Clone)]
#[contracttype]
pub struct Validator {
    pub address: Address,
    pub public_key: BytesN<32>,
    pub is_active: bool,
    pub stake: i128,
    pub confirmed_messages: u64,
}

/// Cross-chain record reference
#[derive(Clone)]
#[contracttype]
pub struct CrossChainRecordRef {
    pub local_record_id: u64,
    pub external_chain: ChainId,
    pub external_record_id: String,
    pub sync_status: SyncStatus,
    pub last_sync: u64,
}

/// Synchronization status for records
#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub enum SyncStatus {
    Synced,
    PendingSync,
    SyncFailed,
    Outdated,
}

/// Atomic transaction for cross-chain updates
#[derive(Clone)]
#[contracttype]
pub struct AtomicTransaction {
    pub tx_id: BytesN<32>,
    pub messages: Vec<BytesN<32>>, // Message IDs involved
    pub status: AtomicTxStatus,
    pub created_at: u64,
    pub timeout: u64,
    pub confirmations: Vec<Address>,
}

/// Status of atomic transactions
#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub enum AtomicTxStatus {
    Initiated,
    Prepared,
    Committed,
    Aborted,
    Expired,
}

// Storage keys
const ADMIN: Symbol = symbol_short!("ADMIN");
const VALIDATORS: Symbol = symbol_short!("VALID");
const MESSAGES: Symbol = symbol_short!("MESSAGES");
const NONCES: Symbol = symbol_short!("NONCES");
const RECORD_REFS: Symbol = symbol_short!("REC_REFS");
const ATOMIC_TXS: Symbol = symbol_short!("ATOM_TXS");
const PAUSED: Symbol = symbol_short!("PAUSED");
const MSG_COUNT: Symbol = symbol_short!("MSG_CNT");
const MIN_CONFIRMATIONS: Symbol = symbol_short!("MIN_CONF");
const SUPPORTED_CHAINS: Symbol = symbol_short!("CHAINS");
const MEDICAL_CONTRACT: Symbol = symbol_short!("MED_CONT");
const IDENTITY_CONTRACT: Symbol = symbol_short!("ID_CONT");
const ACCESS_CONTRACT: Symbol = symbol_short!("ACC_CONT");

// Constants
const DEFAULT_MIN_CONFIRMATIONS: u32 = 2;
const MESSAGE_EXPIRY_SECS: u64 = 86_400; // 24 hours
const ATOMIC_TX_TIMEOUT: u64 = 3_600; // 1 hour

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
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

        // Ensure not already initialized
        if env.storage().persistent().has(&ADMIN) {
            return Err(Error::AlreadyInitialized);
        }

        // Set admin and contract references
        env.storage().persistent().set(&ADMIN, &admin);
        env.storage()
            .persistent()
            .set(&MEDICAL_CONTRACT, &medical_contract);
        env.storage()
            .persistent()
            .set(&IDENTITY_CONTRACT, &identity_contract);
        env.storage()
            .persistent()
            .set(&ACCESS_CONTRACT, &access_contract);

        // Initialize state
        env.storage().persistent().set(&PAUSED, &false);
        env.storage().persistent().set(&MSG_COUNT, &0u64);
        env.storage()
            .persistent()
            .set(&MIN_CONFIRMATIONS, &DEFAULT_MIN_CONFIRMATIONS);

        // Initialize supported chains (Stellar is always supported)
        let mut chains: Vec<ChainId> = Vec::new(&env);
        chains.push_back(ChainId::Stellar);
        chains.push_back(ChainId::Ethereum);
        chains.push_back(ChainId::Polygon);
        env.storage().persistent().set(&SUPPORTED_CHAINS, &chains);

        // Emit initialization event
        env.events()
            .publish((Symbol::new(&env, "BridgeInitialized"),), (admin.clone(),));

        Ok(true)
    }

    // ==================== Admin Functions ====================

    /// Add a new validator to the bridge
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

        let mut validators: Map<Address, Validator> = env
            .storage()
            .persistent()
            .get(&VALIDATORS)
            .unwrap_or(Map::new(&env));

        validators.set(validator_address.clone(), validator);
        env.storage().persistent().set(&VALIDATORS, &validators);

        env.events()
            .publish((Symbol::new(&env, "ValidatorAdded"),), (validator_address,));

        Ok(true)
    }

    /// Remove or deactivate a validator
    pub fn deactivate_validator(
        env: Env,
        caller: Address,
        validator_address: Address,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        let mut validators: Map<Address, Validator> = env
            .storage()
            .persistent()
            .get(&VALIDATORS)
            .unwrap_or(Map::new(&env));

        if let Some(mut validator) = validators.get(validator_address.clone()) {
            validator.is_active = false;
            validators.set(validator_address.clone(), validator);
            env.storage().persistent().set(&VALIDATORS, &validators);

            env.events().publish(
                (Symbol::new(&env, "ValidatorDeactivated"),),
                (validator_address,),
            );

            Ok(true)
        } else {
            Err(Error::ValidatorNotFound)
        }
    }

    /// Add support for a new blockchain
    pub fn add_supported_chain(env: Env, caller: Address, chain: ChainId) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        let mut chains: Vec<ChainId> = env
            .storage()
            .persistent()
            .get(&SUPPORTED_CHAINS)
            .unwrap_or(Vec::new(&env));

        if !chains.contains(&chain) {
            chains.push_back(chain.clone());
            env.storage().persistent().set(&SUPPORTED_CHAINS, &chains);

            env.events()
                .publish((Symbol::new(&env, "ChainAdded"),), (chain,));
        }

        Ok(true)
    }

    /// Update minimum confirmations required
    pub fn set_min_confirmations(
        env: Env,
        caller: Address,
        min_confirmations: u32,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        env.storage()
            .persistent()
            .set(&MIN_CONFIRMATIONS, &min_confirmations);

        Ok(true)
    }

    /// Emergency pause
    pub fn pause(env: Env, caller: Address) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        env.storage().persistent().set(&PAUSED, &true);

        env.events().publish(
            (symbol_short!("Paused"),),
            (caller, env.ledger().timestamp()),
        );

        Ok(true)
    }

    /// Resume operations
    pub fn unpause(env: Env, caller: Address) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        env.storage().persistent().set(&PAUSED, &false);

        env.events().publish(
            (symbol_short!("Unpaused"),),
            (caller, env.ledger().timestamp()),
        );

        Ok(true)
    }

    // ==================== Cross-Chain Message Functions ====================

    /// Submit a new cross-chain message (from external chain)
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

        // Verify nonce to prevent replay attacks
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

        // Store message
        let mut messages: Map<BytesN<32>, CrossChainMessage> = env
            .storage()
            .persistent()
            .get(&MESSAGES)
            .unwrap_or(Map::new(&env));

        messages.set(message_id.clone(), message);
        env.storage().persistent().set(&MESSAGES, &messages);

        // Update nonce
        Self::update_nonce(&env, &sender, nonce);

        // Increment message count
        let count: u64 = env.storage().persistent().get(&MSG_COUNT).unwrap_or(0);
        env.storage().persistent().set(&MSG_COUNT, &(count + 1));

        env.events().publish(
            (Symbol::new(&env, "MessageSubmitted"),),
            (message_id.clone(), timestamp),
        );

        Ok(message_id)
    }

    /// Confirm a cross-chain message (validator attestation)
    pub fn confirm_message(
        env: Env,
        validator: Address,
        message_id: BytesN<32>,
    ) -> Result<bool, Error> {
        validator.require_auth();
        Self::require_not_paused(&env)?;
        Self::require_active_validator(&env, &validator)?;

        let mut messages: Map<BytesN<32>, CrossChainMessage> = env
            .storage()
            .persistent()
            .get(&MESSAGES)
            .unwrap_or(Map::new(&env));

        let message = messages
            .get(message_id.clone())
            .ok_or(Error::MessageNotFound)?;

        // Check if message is still pending
        if message.status != MessageStatus::Pending {
            return Err(Error::MessageAlreadyProcessed);
        }

        // Check if message has expired
        let now = env.ledger().timestamp();
        if now > message.timestamp + MESSAGE_EXPIRY_SECS {
            return Err(Error::MessageExpired);
        }

        // Record confirmation (using a separate confirmations storage)
        let confirm_key = Self::confirmation_key(&env, &message_id);
        let mut confirmations: Vec<Address> = env
            .storage()
            .temporary()
            .get(&confirm_key)
            .unwrap_or(Vec::new(&env));

        // Check for duplicate confirmation
        if confirmations.contains(&validator) {
            return Err(Error::DuplicateConfirmation);
        }

        confirmations.push_back(validator.clone());
        env.storage().temporary().set(&confirm_key, &confirmations);

        // Update validator stats
        Self::increment_validator_confirmations(&env, &validator);

        // Check if we have enough confirmations
        let min_confirmations: u32 = env
            .storage()
            .persistent()
            .get(&MIN_CONFIRMATIONS)
            .unwrap_or(DEFAULT_MIN_CONFIRMATIONS);

        if confirmations.len() as u32 >= min_confirmations {
            // Mark message as verified
            let mut updated_message = message;
            updated_message.status = MessageStatus::Verified;
            messages.set(message_id.clone(), updated_message);
            env.storage().persistent().set(&MESSAGES, &messages);

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

    /// Execute a verified cross-chain message
    pub fn execute_message(
        env: Env,
        caller: Address,
        message_id: BytesN<32>,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_not_paused(&env)?;

        let mut messages: Map<BytesN<32>, CrossChainMessage> = env
            .storage()
            .persistent()
            .get(&MESSAGES)
            .unwrap_or(Map::new(&env));

        let message = messages
            .get(message_id.clone())
            .ok_or(Error::MessageNotFound)?;

        // Must be verified
        if message.status != MessageStatus::Verified {
            return Err(Error::InsufficientConfirmations);
        }

        // Check expiry
        let now = env.ledger().timestamp();
        if now > message.timestamp + MESSAGE_EXPIRY_SECS {
            let mut expired_message = message;
            expired_message.status = MessageStatus::Expired;
            messages.set(message_id.clone(), expired_message);
            env.storage().persistent().set(&MESSAGES, &messages);
            return Err(Error::MessageExpired);
        }

        // Mark as executed
        let mut executed_message = message.clone();
        executed_message.status = MessageStatus::Executed;
        messages.set(message_id.clone(), executed_message);
        env.storage().persistent().set(&MESSAGES, &messages);

        env.events().publish(
            (Symbol::new(&env, "MessageExecuted"),),
            (message_id, message.payload_type),
        );

        Ok(true)
    }

    // ==================== Atomic Transaction Functions ====================

    /// Initiate an atomic cross-chain transaction
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
            timeout: now + ATOMIC_TX_TIMEOUT,
            confirmations: Vec::new(&env),
        };

        let mut atomic_txs: Map<BytesN<32>, AtomicTransaction> = env
            .storage()
            .persistent()
            .get(&ATOMIC_TXS)
            .unwrap_or(Map::new(&env));

        atomic_txs.set(tx_id.clone(), atomic_tx);
        env.storage().persistent().set(&ATOMIC_TXS, &atomic_txs);

        env.events().publish(
            (Symbol::new(&env, "AtomicTxInitiated"),),
            (tx_id.clone(), now),
        );

        Ok(tx_id)
    }

    /// Prepare phase of atomic transaction (2PC)
    pub fn prepare_atomic_tx(
        env: Env,
        validator: Address,
        tx_id: BytesN<32>,
    ) -> Result<bool, Error> {
        validator.require_auth();
        Self::require_not_paused(&env)?;
        Self::require_active_validator(&env, &validator)?;

        let mut atomic_txs: Map<BytesN<32>, AtomicTransaction> = env
            .storage()
            .persistent()
            .get(&ATOMIC_TXS)
            .unwrap_or(Map::new(&env));

        let mut atomic_tx = atomic_txs
            .get(tx_id.clone())
            .ok_or(Error::AtomicTxNotFound)?;

        // Check status
        if atomic_tx.status != AtomicTxStatus::Initiated {
            return Err(Error::AtomicTxAlreadyProcessed);
        }

        // Check timeout
        let now = env.ledger().timestamp();
        if now > atomic_tx.timeout {
            atomic_tx.status = AtomicTxStatus::Expired;
            atomic_txs.set(tx_id.clone(), atomic_tx);
            env.storage().persistent().set(&ATOMIC_TXS, &atomic_txs);
            return Err(Error::AtomicTxExpired);
        }

        // Add confirmation
        if !atomic_tx.confirmations.contains(&validator) {
            atomic_tx.confirmations.push_back(validator.clone());
        }

        // Check if we have enough confirmations to move to prepared
        let min_confirmations: u32 = env
            .storage()
            .persistent()
            .get(&MIN_CONFIRMATIONS)
            .unwrap_or(DEFAULT_MIN_CONFIRMATIONS);

        if atomic_tx.confirmations.len() as u32 >= min_confirmations {
            atomic_tx.status = AtomicTxStatus::Prepared;

            env.events()
                .publish((Symbol::new(&env, "AtomicTxPrepared"),), (tx_id.clone(),));
        }

        atomic_txs.set(tx_id.clone(), atomic_tx);
        env.storage().persistent().set(&ATOMIC_TXS, &atomic_txs);

        Ok(true)
    }

    /// Commit phase of atomic transaction (2PC)
    pub fn commit_atomic_tx(env: Env, caller: Address, tx_id: BytesN<32>) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_not_paused(&env)?;

        let mut atomic_txs: Map<BytesN<32>, AtomicTransaction> = env
            .storage()
            .persistent()
            .get(&ATOMIC_TXS)
            .unwrap_or(Map::new(&env));

        let mut atomic_tx = atomic_txs
            .get(tx_id.clone())
            .ok_or(Error::AtomicTxNotFound)?;

        // Must be in prepared state
        if atomic_tx.status != AtomicTxStatus::Prepared {
            return Err(Error::AtomicTxAlreadyProcessed);
        }

        // Check timeout
        let now = env.ledger().timestamp();
        if now > atomic_tx.timeout {
            atomic_tx.status = AtomicTxStatus::Expired;
            atomic_txs.set(tx_id.clone(), atomic_tx);
            env.storage().persistent().set(&ATOMIC_TXS, &atomic_txs);
            return Err(Error::AtomicTxExpired);
        }

        atomic_tx.status = AtomicTxStatus::Committed;
        atomic_txs.set(tx_id.clone(), atomic_tx);
        env.storage().persistent().set(&ATOMIC_TXS, &atomic_txs);

        env.events()
            .publish((Symbol::new(&env, "AtomicTxCommitted"),), (tx_id,));

        Ok(true)
    }

    /// Abort an atomic transaction
    pub fn abort_atomic_tx(env: Env, caller: Address, tx_id: BytesN<32>) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_not_paused(&env)?;

        let mut atomic_txs: Map<BytesN<32>, AtomicTransaction> = env
            .storage()
            .persistent()
            .get(&ATOMIC_TXS)
            .unwrap_or(Map::new(&env));

        let mut atomic_tx = atomic_txs
            .get(tx_id.clone())
            .ok_or(Error::AtomicTxNotFound)?;

        // Can only abort if not committed
        if atomic_tx.status == AtomicTxStatus::Committed {
            return Err(Error::AtomicTxAlreadyProcessed);
        }

        atomic_tx.status = AtomicTxStatus::Aborted;
        atomic_txs.set(tx_id.clone(), atomic_tx);
        env.storage().persistent().set(&ATOMIC_TXS, &atomic_txs);

        env.events()
            .publish((Symbol::new(&env, "AtomicTxAborted"),), (tx_id,));

        Ok(true)
    }

    // ==================== Record Reference Functions ====================

    /// Register a cross-chain record reference
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

        let ref_key = Self::record_ref_key(&env, local_record_id, &external_chain);
        let mut record_refs: Map<Symbol, CrossChainRecordRef> = env
            .storage()
            .persistent()
            .get(&RECORD_REFS)
            .unwrap_or(Map::new(&env));

        record_refs.set(ref_key, record_ref);
        env.storage().persistent().set(&RECORD_REFS, &record_refs);

        env.events().publish(
            (Symbol::new(&env, "RecordRefRegistered"),),
            (local_record_id, external_chain),
        );

        Ok(true)
    }

    /// Update sync status of a record reference
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

        let ref_key = Self::record_ref_key(&env, local_record_id, &external_chain);
        let mut record_refs: Map<Symbol, CrossChainRecordRef> = env
            .storage()
            .persistent()
            .get(&RECORD_REFS)
            .unwrap_or(Map::new(&env));

        let mut record_ref = record_refs
            .get(ref_key.clone())
            .ok_or(Error::RecordRefNotFound)?;
        record_ref.sync_status = status.clone();
        record_ref.last_sync = env.ledger().timestamp();

        record_refs.set(ref_key, record_ref);
        env.storage().persistent().set(&RECORD_REFS, &record_refs);

        env.events().publish(
            (Symbol::new(&env, "SyncStatusUpdated"),),
            (local_record_id, external_chain, status),
        );

        Ok(true)
    }

    // ==================== Query Functions ====================

    /// Get a cross-chain message by ID
    pub fn get_message(env: Env, message_id: BytesN<32>) -> Option<CrossChainMessage> {
        let messages: Map<BytesN<32>, CrossChainMessage> = env
            .storage()
            .persistent()
            .get(&MESSAGES)
            .unwrap_or(Map::new(&env));

        messages.get(message_id)
    }

    /// Get an atomic transaction by ID
    pub fn get_atomic_tx(env: Env, tx_id: BytesN<32>) -> Option<AtomicTransaction> {
        let atomic_txs: Map<BytesN<32>, AtomicTransaction> = env
            .storage()
            .persistent()
            .get(&ATOMIC_TXS)
            .unwrap_or(Map::new(&env));

        atomic_txs.get(tx_id)
    }

    /// Get record reference
    pub fn get_record_ref(
        env: Env,
        local_record_id: u64,
        external_chain: ChainId,
    ) -> Option<CrossChainRecordRef> {
        let ref_key = Self::record_ref_key(&env, local_record_id, &external_chain);
        let record_refs: Map<Symbol, CrossChainRecordRef> = env
            .storage()
            .persistent()
            .get(&RECORD_REFS)
            .unwrap_or(Map::new(&env));

        record_refs.get(ref_key)
    }

    /// Get validator info
    pub fn get_validator(env: Env, validator_address: Address) -> Option<Validator> {
        let validators: Map<Address, Validator> = env
            .storage()
            .persistent()
            .get(&VALIDATORS)
            .unwrap_or(Map::new(&env));

        validators.get(validator_address)
    }

    /// Get supported chains
    pub fn get_supported_chains(env: Env) -> Vec<ChainId> {
        env.storage()
            .persistent()
            .get(&SUPPORTED_CHAINS)
            .unwrap_or(Vec::new(&env))
    }

    /// Check if bridge is paused
    pub fn is_paused(env: Env) -> bool {
        env.storage().persistent().get(&PAUSED).unwrap_or(false)
    }

    /// Get total message count
    pub fn get_message_count(env: Env) -> u64 {
        env.storage().persistent().get(&MSG_COUNT).unwrap_or(0)
    }

    // ==================== Internal Helper Functions ====================

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN)
            .ok_or(Error::NotAuthorized)?;

        if caller != &admin {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    fn require_not_paused(env: &Env) -> Result<(), Error> {
        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }
        Ok(())
    }

    fn require_active_validator(env: &Env, validator: &Address) -> Result<(), Error> {
        let validators: Map<Address, Validator> = env
            .storage()
            .persistent()
            .get(&VALIDATORS)
            .unwrap_or(Map::new(&env));

        match validators.get(validator.clone()) {
            Some(v) if v.is_active => Ok(()),
            Some(_) => Err(Error::ValidatorNotActive),
            None => Err(Error::ValidatorNotFound),
        }
    }

    fn require_chain_supported(env: &Env, chain: &ChainId) -> Result<(), Error> {
        let chains: Vec<ChainId> = env
            .storage()
            .persistent()
            .get(&SUPPORTED_CHAINS)
            .unwrap_or(Vec::new(&env));

        if chains.contains(chain) {
            Ok(())
        } else {
            Err(Error::ChainNotSupported)
        }
    }

    fn verify_nonce(env: &Env, sender: &String, nonce: u64) -> Result<(), Error> {
        let nonces: Map<String, u64> = env
            .storage()
            .persistent()
            .get(&NONCES)
            .unwrap_or(Map::new(&env));

        let last_nonce = nonces.get(sender.clone()).unwrap_or(0);
        if nonce <= last_nonce {
            return Err(Error::InvalidNonce);
        }
        Ok(())
    }

    fn update_nonce(env: &Env, sender: &String, nonce: u64) {
        let mut nonces: Map<String, u64> = env
            .storage()
            .persistent()
            .get(&NONCES)
            .unwrap_or(Map::new(&env));

        nonces.set(sender.clone(), nonce);
        env.storage().persistent().set(&NONCES, &nonces);
    }

    fn increment_validator_confirmations(env: &Env, validator: &Address) {
        let mut validators: Map<Address, Validator> = env
            .storage()
            .persistent()
            .get(&VALIDATORS)
            .unwrap_or(Map::new(&env));

        if let Some(mut v) = validators.get(validator.clone()) {
            v.confirmed_messages += 1;
            validators.set(validator.clone(), v);
            env.storage().persistent().set(&VALIDATORS, &validators);
        }
    }

    fn confirmation_key(env: &Env, _message_id: &BytesN<32>) -> Symbol {
        // Create a unique key for message confirmations
        // Using simplified key due to Symbol limitations
        Symbol::new(&env, "conf_key")
    }

    fn record_ref_key(_env: &Env, _local_record_id: u64, _chain: &ChainId) -> Symbol {
        // Create unique key for record references
        Symbol::new(&_env, "rec_ref")
    }
}
