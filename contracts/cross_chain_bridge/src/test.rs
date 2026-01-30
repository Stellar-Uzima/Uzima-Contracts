#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
use crate::{
    AtomicTxStatus, ChainId, CrossChainBridgeContract, CrossChainBridgeContractClient, Error,
    MessageStatus, MessageType, SyncStatus,
};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

fn create_contract(
    env: &Env,
) -> (
    CrossChainBridgeContractClient<'_>,
    Address,
    Address,
    Address,
    Address,
) {
    let contract_id = env.register_contract(None, CrossChainBridgeContract);
    let client = CrossChainBridgeContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let medical_contract = Address::generate(&env);
    let identity_contract = Address::generate(&env);
    let access_contract = Address::generate(&env);
    (
        client,
        admin,
        medical_contract,
        identity_contract,
        access_contract,
    )
}

fn initialize_contract(
    env: &Env,
    client: &CrossChainBridgeContractClient,
    admin: &Address,
    medical: &Address,
    identity: &Address,
    access: &Address,
) {
    env.mock_all_auths();
    client.initialize(admin, medical, identity, access);
}

fn generate_message_id(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[1u8; 32])
}

fn generate_signature(env: &Env) -> BytesN<64> {
    BytesN::from_array(env, &[2u8; 64])
}

fn generate_public_key(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[3u8; 32])
}

// ==================== Initialization Tests ====================

#[test]
fn test_initialize() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);

    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    assert!(!client.is_paused());
    assert_eq!(client.get_message_count(), 0);
}

#[test]
fn test_initialize_twice_fails() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);

    env.mock_all_auths();
    client.initialize(&admin, &medical, &identity, &access);

    let result = client.try_initialize(&admin, &medical, &identity, &access);
    assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
}

// ==================== Validator Tests ====================

#[test]
fn test_add_validator() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let validator = Address::generate(&env);
    let public_key = generate_public_key(&env);

    env.mock_all_auths();
    let result = client.add_validator(&admin, &validator, &public_key, &1000);
    assert!(result);

    let validator_info = client.get_validator(&validator);
    assert!(validator_info.is_some());

    let v = validator_info.unwrap();
    assert!(v.is_active);
    assert_eq!(v.stake, 1000);
    assert_eq!(v.confirmed_messages, 0);
}

#[test]
fn test_deactivate_validator() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let validator = Address::generate(&env);
    let public_key = generate_public_key(&env);

    env.mock_all_auths();
    client.add_validator(&admin, &validator, &public_key, &1000);
    client.deactivate_validator(&admin, &validator);

    let validator_info = client.get_validator(&validator).unwrap();
    assert!(!validator_info.is_active);
}

#[test]
fn test_add_validator_not_admin() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let non_admin = Address::generate(&env);
    let validator = Address::generate(&env);
    let public_key = generate_public_key(&env);

    env.mock_all_auths();
    let result = client.try_add_validator(&non_admin, &validator, &public_key, &1000);
    assert_eq!(result, Err(Ok(Error::NotAuthorized)));
}

// ==================== Chain Support Tests ====================

#[test]
fn test_supported_chains() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let chains = client.get_supported_chains();
    assert!(chains.contains(&ChainId::Stellar));
    assert!(chains.contains(&ChainId::Ethereum));
    assert!(chains.contains(&ChainId::Polygon));
}

#[test]
fn test_add_supported_chain() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    env.mock_all_auths();
    client.add_supported_chain(&admin, &ChainId::Avalanche);

    let chains = client.get_supported_chains();
    assert!(chains.contains(&ChainId::Avalanche));
}

// ==================== Message Tests ====================

#[test]
fn test_submit_message() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let validator = Address::generate(&env);
    let public_key = generate_public_key(&env);
    let recipient = Address::generate(&env);

    env.mock_all_auths();
    client.add_validator(&admin, &validator, &public_key, &1000);

    let message_id = generate_message_id(&env);
    let sender = String::from_str(&env, "0x1234567890abcdef");
    let payload = String::from_str(&env, "{\"record_id\": 1}");
    let signature = generate_signature(&env);

    let result = client.submit_message(
        &validator,
        &message_id,
        &ChainId::Ethereum,
        &ChainId::Stellar,
        &sender,
        &recipient,
        &MessageType::RecordRequest,
        &payload,
        &1,
        &signature,
    );

    assert_eq!(result, message_id);
    assert_eq!(client.get_message_count(), 1);

    let msg = client.get_message(&message_id).unwrap();
    assert_eq!(msg.status, MessageStatus::Pending);
}

#[test]
fn test_submit_message_invalid_chain() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let validator = Address::generate(&env);
    let public_key = generate_public_key(&env);
    let recipient = Address::generate(&env);

    env.mock_all_auths();
    client.add_validator(&admin, &validator, &public_key, &1000);

    let message_id = generate_message_id(&env);
    let sender = String::from_str(&env, "0x1234567890abcdef");
    let payload = String::from_str(&env, "{\"record_id\": 1}");
    let signature = generate_signature(&env);

    // BinanceSmartChain is not added as supported
    let result = client.try_submit_message(
        &validator,
        &message_id,
        &ChainId::BinanceSmartChain,
        &ChainId::Stellar,
        &sender,
        &recipient,
        &MessageType::RecordRequest,
        &payload,
        &1,
        &signature,
    );

    assert_eq!(result, Err(Ok(Error::ChainNotSupported)));
}

#[test]
fn test_confirm_message() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let validator1 = Address::generate(&env);
    let validator2 = Address::generate(&env);
    let public_key = generate_public_key(&env);
    let recipient = Address::generate(&env);

    env.mock_all_auths();
    client.add_validator(&admin, &validator1, &public_key, &1000);
    client.add_validator(&admin, &validator2, &public_key, &1000);

    // Submit message
    let message_id = generate_message_id(&env);
    let sender = String::from_str(&env, "0x1234567890abcdef");
    let payload = String::from_str(&env, "{\"record_id\": 1}");
    let signature = generate_signature(&env);

    client.submit_message(
        &validator1,
        &message_id,
        &ChainId::Ethereum,
        &ChainId::Stellar,
        &sender,
        &recipient,
        &MessageType::RecordRequest,
        &payload,
        &1,
        &signature,
    );

    // First confirmation
    client.confirm_message(&validator1, &message_id);
    let msg = client.get_message(&message_id).unwrap();
    assert_eq!(msg.status, MessageStatus::Pending);

    // Second confirmation should verify the message
    client.confirm_message(&validator2, &message_id);
    let msg = client.get_message(&message_id).unwrap();
    assert_eq!(msg.status, MessageStatus::Verified);
}

#[test]
fn test_execute_message() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let validator1 = Address::generate(&env);
    let validator2 = Address::generate(&env);
    let public_key = generate_public_key(&env);
    let recipient = Address::generate(&env);

    env.mock_all_auths();
    client.add_validator(&admin, &validator1, &public_key, &1000);
    client.add_validator(&admin, &validator2, &public_key, &1000);

    // Submit and verify message
    let message_id = generate_message_id(&env);
    let sender = String::from_str(&env, "0x1234567890abcdef");
    let payload = String::from_str(&env, "{\"record_id\": 1}");
    let signature = generate_signature(&env);

    client.submit_message(
        &validator1,
        &message_id,
        &ChainId::Ethereum,
        &ChainId::Stellar,
        &sender,
        &recipient,
        &MessageType::RecordRequest,
        &payload,
        &1,
        &signature,
    );

    client.confirm_message(&validator1, &message_id);
    client.confirm_message(&validator2, &message_id);

    // Execute
    let result = client.execute_message(&recipient, &message_id);
    assert!(result);

    let msg = client.get_message(&message_id).unwrap();
    assert_eq!(msg.status, MessageStatus::Executed);
}

// ==================== Atomic Transaction Tests ====================

#[test]
fn test_atomic_transaction_flow() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let validator1 = Address::generate(&env);
    let validator2 = Address::generate(&env);
    let public_key = generate_public_key(&env);
    let caller = Address::generate(&env);

    env.mock_all_auths();
    client.add_validator(&admin, &validator1, &public_key, &1000);
    client.add_validator(&admin, &validator2, &public_key, &1000);

    // Initiate atomic transaction
    let tx_id = BytesN::from_array(&env, &[5u8; 32]);
    let message_ids = soroban_sdk::vec![&env, generate_message_id(&env)];

    let result = client.initiate_atomic_tx(&caller, &tx_id, &message_ids);
    assert_eq!(result, tx_id);

    let atomic_tx = client.get_atomic_tx(&tx_id).unwrap();
    assert_eq!(atomic_tx.status, AtomicTxStatus::Initiated);

    // Prepare phase
    client.prepare_atomic_tx(&validator1, &tx_id);
    let atomic_tx = client.get_atomic_tx(&tx_id).unwrap();
    assert_eq!(atomic_tx.status, AtomicTxStatus::Initiated); // Still initiated, need more confirmations

    client.prepare_atomic_tx(&validator2, &tx_id);
    let atomic_tx = client.get_atomic_tx(&tx_id).unwrap();
    assert_eq!(atomic_tx.status, AtomicTxStatus::Prepared);

    // Commit phase
    client.commit_atomic_tx(&caller, &tx_id);
    let atomic_tx = client.get_atomic_tx(&tx_id).unwrap();
    assert_eq!(atomic_tx.status, AtomicTxStatus::Committed);
}

#[test]
fn test_abort_atomic_tx() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let caller = Address::generate(&env);

    env.mock_all_auths();

    let tx_id = BytesN::from_array(&env, &[5u8; 32]);
    let message_ids = soroban_sdk::vec![&env, generate_message_id(&env)];

    client.initiate_atomic_tx(&caller, &tx_id, &message_ids);
    client.abort_atomic_tx(&caller, &tx_id);

    let atomic_tx = client.get_atomic_tx(&tx_id).unwrap();
    assert_eq!(atomic_tx.status, AtomicTxStatus::Aborted);
}

// ==================== Record Reference Tests ====================

#[test]
fn test_register_record_ref() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let caller = Address::generate(&env);

    env.mock_all_auths();

    let external_record_id = String::from_str(&env, "eth_record_123");
    let result = client.register_record_ref(&caller, &1, &ChainId::Ethereum, &external_record_id);
    assert!(result);

    let record_ref = client.get_record_ref(&1, &ChainId::Ethereum).unwrap();
    assert_eq!(record_ref.local_record_id, 1);
    assert_eq!(record_ref.sync_status, SyncStatus::PendingSync);
}

#[test]
fn test_update_sync_status() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let validator = Address::generate(&env);
    let public_key = generate_public_key(&env);
    let caller = Address::generate(&env);

    env.mock_all_auths();
    client.add_validator(&admin, &validator, &public_key, &1000);

    let external_record_id = String::from_str(&env, "eth_record_123");
    client.register_record_ref(&caller, &1, &ChainId::Ethereum, &external_record_id);

    client.update_sync_status(&validator, &1, &ChainId::Ethereum, &SyncStatus::Synced);

    let record_ref = client.get_record_ref(&1, &ChainId::Ethereum).unwrap();
    assert_eq!(record_ref.sync_status, SyncStatus::Synced);
}

// ==================== Pause Tests ====================

#[test]
fn test_pause_unpause() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    env.mock_all_auths();

    assert!(!client.is_paused());

    client.pause(&admin);
    assert!(client.is_paused());

    client.unpause(&admin);
    assert!(!client.is_paused());
}

#[test]
fn test_operations_blocked_when_paused() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let validator = Address::generate(&env);
    let public_key = generate_public_key(&env);

    env.mock_all_auths();
    client.add_validator(&admin, &validator, &public_key, &1000);
    client.pause(&admin);

    let message_id = generate_message_id(&env);
    let sender = String::from_str(&env, "0x1234567890abcdef");
    let recipient = Address::generate(&env);
    let payload = String::from_str(&env, "{\"record_id\": 1}");
    let signature = generate_signature(&env);

    let result = client.try_submit_message(
        &validator,
        &message_id,
        &ChainId::Ethereum,
        &ChainId::Stellar,
        &sender,
        &recipient,
        &MessageType::RecordRequest,
        &payload,
        &1,
        &signature,
    );

    assert_eq!(result, Err(Ok(Error::ContractPaused)));
}

// ==================== Nonce Tests ====================

#[test]
fn test_nonce_replay_protection() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let validator = Address::generate(&env);
    let public_key = generate_public_key(&env);
    let recipient = Address::generate(&env);

    env.mock_all_auths();
    client.add_validator(&admin, &validator, &public_key, &1000);

    let message_id1 = generate_message_id(&env);
    let message_id2 = BytesN::from_array(&env, &[4u8; 32]);
    let sender = String::from_str(&env, "0x1234567890abcdef");
    let payload = String::from_str(&env, "{\"record_id\": 1}");
    let signature = generate_signature(&env);

    // First message with nonce 1
    client.submit_message(
        &validator,
        &message_id1,
        &ChainId::Ethereum,
        &ChainId::Stellar,
        &sender,
        &recipient,
        &MessageType::RecordRequest,
        &payload,
        &1,
        &signature,
    );

    // Second message with same nonce should fail
    let result = client.try_submit_message(
        &validator,
        &message_id2,
        &ChainId::Ethereum,
        &ChainId::Stellar,
        &sender,
        &recipient,
        &MessageType::RecordRequest,
        &payload,
        &1,
        &signature,
    );

    assert_eq!(result, Err(Ok(Error::InvalidNonce)));
}
