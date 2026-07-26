#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
use crate::{
    AtomicTxStatus, ChainId, CrossChainBridgeContract, CrossChainBridgeContractClient,
    CrossChainEventType, Error, EventSyncStatus, MessageStatus, MessageType, OracleStatus,
    RollbackOpType, RollbackStatus, SubmitMessageRequest, SyncStatus,
};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env, String, Vec};

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

fn dummy_sig(env: &Env) -> BytesN<64> {
    BytesN::from_array(env, &[2u8; 64])
}

fn generate_keypair() -> (VerifyingKey, SigningKey) {
    let mut rng = rand::thread_rng();
    let signing_key = SigningKey::generate(&mut rng);
    let verifying_key = signing_key.verifying_key();
    (verifying_key, signing_key)
}

fn make_public_key(env: &Env, vk: &VerifyingKey) -> BytesN<32> {
    BytesN::from_array(env, &vk.to_bytes())
}

fn create_sig(env: &Env, signing_key: &SigningKey, data: &BytesN<32>, nonce: u64) -> BytesN<64> {
    let mut payload = Bytes::new(env);
    payload.extend_from_array(&data.to_array());
    payload.extend_from_array(&nonce.to_be_bytes());
    let hash = env.crypto().sha256(&payload);
    let sig = signing_key.sign(&hash.to_array());
    BytesN::from_array(env, &sig.to_bytes())
}

fn setup_validator(
    env: &Env,
    client: &CrossChainBridgeContractClient,
    admin: &Address,
) -> (Address, SigningKey) {
    let (vk, sk) = generate_keypair();
    let public_key = make_public_key(env, &vk);
    let validator = Address::generate(env);
    env.mock_all_auths();
    client.add_validator(admin, &validator, &public_key, &1000);
    (validator, sk)
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
    let public_key = BytesN::from_array(&env, &[3u8; 32]);

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
    let public_key = BytesN::from_array(&env, &[3u8; 32]);

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
    let public_key = BytesN::from_array(&env, &[3u8; 32]);

    env.mock_all_auths();
    let result = client.try_add_validator(&non_admin, &validator, &public_key, &1000);
    assert_eq!(result, Err(Ok(Error::Unauthorized)));
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

    let (validator, sk) = setup_validator(&env, &client, &admin);
    let recipient = Address::generate(&env);

    let message_id = generate_message_id(&env);
    let sender = String::from_str(&env, "0x1234567890abcdef");
    let payload = String::from_str(&env, "{\"record_id\": 1}");

    env.mock_all_auths();
    let v_sig = create_sig(&env, &sk, &message_id, 1);
    let result = client.submit_message(
        &validator,
        &SubmitMessageRequest {
            message_id: message_id.clone(),
            source_chain: ChainId::Ethereum,
            dest_chain: ChainId::Stellar,
            sender: sender.clone(),
            recipient: recipient.clone(),
            payload_type: MessageType::RecordRequest,
            payload: payload.clone(),
            nonce: 1,
            signature: dummy_sig(&env),
            v_signature: v_sig,
            v_nonce: 1,
        },
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

    let (validator, _sk) = setup_validator(&env, &client, &admin);
    let recipient = Address::generate(&env);

    env.mock_all_auths();
    let result = client.try_submit_message(
        &validator,
        &SubmitMessageRequest {
            message_id: generate_message_id(&env),
            source_chain: ChainId::BinanceSmartChain,
            dest_chain: ChainId::Stellar,
            sender: String::from_str(&env, "0x1234"),
            recipient: recipient.clone(),
            payload_type: MessageType::RecordRequest,
            payload: String::from_str(&env, "{}"),
            nonce: 1,
            signature: dummy_sig(&env),
            v_signature: dummy_sig(&env),
            v_nonce: 1,
        },
    );

    assert_eq!(result, Err(Ok(Error::ChainNotSupported)));
}

#[test]
fn test_confirm_message() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let (validator1, sk1) = setup_validator(&env, &client, &admin);
    let (validator2, sk2) = setup_validator(&env, &client, &admin);
    let recipient = Address::generate(&env);

    env.mock_all_auths();

    let message_id = generate_message_id(&env);
    let v_sig1 = create_sig(&env, &sk1, &message_id, 1);
    client.submit_message(
        &validator1,
        &SubmitMessageRequest {
            message_id: message_id.clone(),
            source_chain: ChainId::Ethereum,
            dest_chain: ChainId::Stellar,
            sender: String::from_str(&env, "0x1234567890abcdef"),
            recipient: recipient.clone(),
            payload_type: MessageType::RecordRequest,
            payload: String::from_str(&env, "{\"record_id\": 1}"),
            nonce: 1,
            signature: dummy_sig(&env),
            v_signature: v_sig1,
            v_nonce: 1,
        },
    );

    let confirm_sig1 = create_sig(&env, &sk1, &message_id, 2);
    client.confirm_message(&validator1, &message_id, &confirm_sig1, &2);
    let msg = client.get_message(&message_id).unwrap();
    assert_eq!(msg.status, MessageStatus::Pending);

    let confirm_sig2 = create_sig(&env, &sk2, &message_id, 1);
    client.confirm_message(&validator2, &message_id, &confirm_sig2, &1);
    let msg = client.get_message(&message_id).unwrap();
    assert_eq!(msg.status, MessageStatus::Verified);
}

// ==================== Storage Key Uniqueness Regression Tests ====================

/// Regression test: two different messages must have independent confirmation tracking
#[test]
fn test_confirmations_unique_per_message() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let (validator1, sk1) = setup_validator(&env, &client, &admin);
    let (validator2, sk2) = setup_validator(&env, &client, &admin);
    let recipient = Address::generate(&env);

    env.mock_all_auths();

    // Submit message A
    let msg_id_a = BytesN::from_array(&env, &[0xaau8; 32]);
    let v_sig_a = create_sig(&env, &sk1, &msg_id_a, 1);
    client.submit_message(
        &validator1,
        &SubmitMessageRequest {
            message_id: msg_id_a.clone(),
            source_chain: ChainId::Ethereum,
            dest_chain: ChainId::Stellar,
            sender: String::from_str(&env, "0xAAA"),
            recipient: recipient.clone(),
            payload_type: MessageType::RecordRequest,
            payload: String::from_str(&env, "{}"),
            nonce: 1,
            signature: dummy_sig(&env),
            v_signature: v_sig_a,
            v_nonce: 1,
        },
    );

    // Submit message B (different data, same validator, needs nonce 2)
    let msg_id_b = BytesN::from_array(&env, &[0xbbu8; 32]);
    let v_sig_b = create_sig(&env, &sk1, &msg_id_b, 2);
    client.submit_message(
        &validator1,
        &SubmitMessageRequest {
            message_id: msg_id_b.clone(),
            source_chain: ChainId::Ethereum,
            dest_chain: ChainId::Stellar,
            sender: String::from_str(&env, "0xBBB"),
            recipient: recipient.clone(),
            payload_type: MessageType::RecordRequest,
            payload: String::from_str(&env, "{}"),
            nonce: 1,
            signature: dummy_sig(&env),
            v_signature: v_sig_b,
            v_nonce: 2,
        },
    );

    // Confirm only message A with both validators (nonce resets per validator pubkey)
    let conf_sig_1a = create_sig(&env, &sk1, &msg_id_a, 3);
    let conf_sig_2a = create_sig(&env, &sk2, &msg_id_a, 1);
    client.confirm_message(&validator1, &msg_id_a, &conf_sig_1a, &3);
    client.confirm_message(&validator2, &msg_id_a, &conf_sig_2a, &1);

    // Message A should be Verified, message B should still be Pending
    assert_eq!(
        client.get_message(&msg_id_a).unwrap().status,
        MessageStatus::Verified
    );
    assert_eq!(
        client.get_message(&msg_id_b).unwrap().status,
        MessageStatus::Pending
    );
}

/// Regression test: record refs for different chains must be independent
#[test]
fn test_record_refs_unique_per_chain() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let caller = Address::generate(&env);
    let (validator, sk) = setup_validator(&env, &client, &admin);

    env.mock_all_auths();

    // Register record 1 on Ethereum
    client.register_record_ref(
        &caller,
        &1,
        &ChainId::Ethereum,
        &String::from_str(&env, "eth_record_001"),
    );

    // Register same local record 1 on Polygon
    client.register_record_ref(
        &caller,
        &1,
        &ChainId::Polygon,
        &String::from_str(&env, "poly_record_001"),
    );

    // Register a different local record 2 on Ethereum
    client.register_record_ref(
        &caller,
        &2,
        &ChainId::Ethereum,
        &String::from_str(&env, "eth_record_002"),
    );

    // Each should have its own sync status — update Ethereum record 1 only
    let mut target_id_bytes = [0u8; 32];
    target_id_bytes[24..32].copy_from_slice(&1u64.to_be_bytes());
    let target_id = BytesN::from_array(&env, &target_id_bytes);
    let update_sig = create_sig(&env, &sk, &target_id, 1);
    client.update_sync_status(
        &validator,
        &1,
        &ChainId::Ethereum,
        &SyncStatus::Synced,
        &update_sig,
        &1,
    );

    let eth_ref = client.get_record_ref(&1, &ChainId::Ethereum).unwrap();
    let poly_ref = client.get_record_ref(&1, &ChainId::Polygon).unwrap();
    let eth_ref2 = client.get_record_ref(&2, &ChainId::Ethereum).unwrap();

    assert_eq!(eth_ref.sync_status, SyncStatus::Synced);
    assert_eq!(poly_ref.sync_status, SyncStatus::PendingSync); // unaffected
    assert_eq!(eth_ref2.sync_status, SyncStatus::PendingSync); // unaffected
    assert_eq!(
        eth_ref.external_record_id,
        String::from_str(&env, "eth_record_001")
    );
    assert_eq!(
        poly_ref.external_record_id,
        String::from_str(&env, "poly_record_001")
    );
}

#[test]
fn test_execute_message() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let (validator1, sk1) = setup_validator(&env, &client, &admin);
    let (validator2, sk2) = setup_validator(&env, &client, &admin);
    let recipient = Address::generate(&env);

    env.mock_all_auths();

    let message_id = generate_message_id(&env);
    let v_sig = create_sig(&env, &sk1, &message_id, 1);
    client.submit_message(
        &validator1,
        &SubmitMessageRequest {
            message_id: message_id.clone(),
            source_chain: ChainId::Ethereum,
            dest_chain: ChainId::Stellar,
            sender: String::from_str(&env, "0x1234567890abcdef"),
            recipient: recipient.clone(),
            payload_type: MessageType::RecordRequest,
            payload: String::from_str(&env, "{\"record_id\": 1}"),
            nonce: 1,
            signature: dummy_sig(&env),
            v_signature: v_sig,
            v_nonce: 1,
        },
    );

    let conf_sig1 = create_sig(&env, &sk1, &message_id, 2);
    let conf_sig2 = create_sig(&env, &sk2, &message_id, 1);
    client.confirm_message(&validator1, &message_id, &conf_sig1, &2);
    client.confirm_message(&validator2, &message_id, &conf_sig2, &1);

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

    let (validator1, sk1) = setup_validator(&env, &client, &admin);
    let (validator2, sk2) = setup_validator(&env, &client, &admin);
    let caller = Address::generate(&env);

    env.mock_all_auths();

    let tx_id = BytesN::from_array(&env, &[5u8; 32]);
    let message_ids = soroban_sdk::vec![&env, generate_message_id(&env)];

    let result = client.initiate_atomic_tx(&caller, &tx_id, &message_ids);
    assert_eq!(result, tx_id);

    let atomic_tx = client.get_atomic_tx(&tx_id).unwrap();
    assert_eq!(atomic_tx.status, AtomicTxStatus::Initiated);

    let prep_sig1 = create_sig(&env, &sk1, &tx_id, 1);
    client.prepare_atomic_tx(&validator1, &tx_id, &prep_sig1, &1);
    let atomic_tx = client.get_atomic_tx(&tx_id).unwrap();
    assert_eq!(atomic_tx.status, AtomicTxStatus::Initiated);

    let prep_sig2 = create_sig(&env, &sk2, &tx_id, 1);
    client.prepare_atomic_tx(&validator2, &tx_id, &prep_sig2, &1);
    let atomic_tx = client.get_atomic_tx(&tx_id).unwrap();
    assert_eq!(atomic_tx.status, AtomicTxStatus::Prepared);

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

    let (validator, sk) = setup_validator(&env, &client, &admin);
    let caller = Address::generate(&env);

    env.mock_all_auths();
    let external_record_id = String::from_str(&env, "eth_record_123");
    client.register_record_ref(&caller, &1, &ChainId::Ethereum, &external_record_id);

    let mut target_id_bytes = [0u8; 32];
    target_id_bytes[24..32].copy_from_slice(&1u64.to_be_bytes());
    let target_id = BytesN::from_array(&env, &target_id_bytes);
    let update_sig = create_sig(&env, &sk, &target_id, 1);
    client.update_sync_status(
        &validator,
        &1,
        &ChainId::Ethereum,
        &SyncStatus::Synced,
        &update_sig,
        &1,
    );

    let record_ref = client.get_record_ref(&1, &ChainId::Ethereum).unwrap();
    assert_eq!(record_ref.sync_status, SyncStatus::Synced);
}

// ==================== Oracle Network Tests ====================

#[test]
fn test_register_oracle() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let oracle = Address::generate(&env);
    let public_key = BytesN::from_array(&env, &[3u8; 32]);
    let supported_chains = soroban_sdk::vec![&env, ChainId::Ethereum, ChainId::Polygon];

    env.mock_all_auths();
    let result = client.register_oracle(&admin, &oracle, &public_key, &supported_chains);
    assert!(result);

    let oracle_node = client.get_oracle_node(&oracle).unwrap();
    assert!(oracle_node.is_active);
    assert_eq!(oracle_node.reputation, 50);
    assert_eq!(oracle_node.total_reports, 0);
}

#[test]
fn test_deactivate_oracle() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let oracle = Address::generate(&env);
    let public_key = BytesN::from_array(&env, &[3u8; 32]);
    let chains = soroban_sdk::vec![&env, ChainId::Ethereum];

    env.mock_all_auths();
    client.register_oracle(&admin, &oracle, &public_key, &chains);
    client.deactivate_oracle(&admin, &oracle);

    let oracle_node = client.get_oracle_node(&oracle).unwrap();
    assert!(!oracle_node.is_active);
}

#[test]
fn test_submit_oracle_report() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let oracle = Address::generate(&env);
    let public_key = BytesN::from_array(&env, &[3u8; 32]);
    let chains = soroban_sdk::vec![&env, ChainId::Ethereum];

    env.mock_all_auths();
    client.register_oracle(&admin, &oracle, &public_key, &chains);

    let data_hash = BytesN::from_array(&env, &[0xabu8; 32]);
    let data = String::from_str(&env, "{\"block\": 12345678}");

    let report_id = client.submit_oracle_report(
        &oracle,
        &ChainId::Ethereum,
        &data_hash,
        &data,
        &100000,
        &dummy_sig(&env),
    );

    assert_eq!(report_id, 1);
    assert_eq!(client.get_oracle_count(), 1);

    let report = client.get_oracle_report(&report_id).unwrap();
    assert_eq!(report.oracle, oracle);
    assert_eq!(report.block_height, 100000);
    assert_eq!(report.status, OracleStatus::Submitted);

    // Verify oracle stats updated
    let node = client.get_oracle_node(&oracle).unwrap();
    assert_eq!(node.total_reports, 1);
}

#[test]
fn test_aggregate_oracle_data() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let (validator, sk) = setup_validator(&env, &client, &admin);

    // Register 3 oracles and submit reports
    let oracle1 = Address::generate(&env);
    let oracle2 = Address::generate(&env);
    let oracle3 = Address::generate(&env);
    let chains = soroban_sdk::vec![&env, ChainId::Ethereum];
    let data_hash = BytesN::from_array(&env, &[0xabu8; 32]);

    env.mock_all_auths();

    let mut report_ids = soroban_sdk::vec![&env];
    for oracle in [&oracle1, &oracle2, &oracle3] {
        client.register_oracle(&admin, oracle, &data_hash, &chains);
        let rid = client.submit_oracle_report(
            oracle,
            &ChainId::Ethereum,
            &data_hash,
            &String::from_str(&env, "{}"),
            &100,
            &dummy_sig(&env),
        );
        report_ids.push_back(rid);
    }

    let consensus_hash = BytesN::from_array(&env, &[0xccu8; 32]);
    let agg_sig = create_sig(&env, &sk, &consensus_hash, 1);
    let result = client.aggregate_oracle_data(
        &validator,
        &ChainId::Ethereum,
        &report_ids,
        &consensus_hash,
        &agg_sig,
        &1,
    );
    assert!(result);

    let aggregated = client.get_aggregated_oracle(&ChainId::Ethereum).unwrap();
    assert!(aggregated.is_finalized);
    assert_eq!(aggregated.report_count, 3);
    assert_eq!(aggregated.consensus_hash, consensus_hash);
}

#[test]
fn test_aggregate_oracle_insufficient_reports() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let (validator, sk) = setup_validator(&env, &client, &admin);

    env.mock_all_auths();

    // Only 2 reports (below MIN_ORACLE_REPORTS = 3)
    let report_ids = soroban_sdk::vec![&env, 1u64, 2u64];
    let consensus_hash = BytesN::from_array(&env, &[0xaau8; 32]);
    // Signature is verified before the report count check
    let agg_sig = create_sig(&env, &sk, &consensus_hash, 1);
    let result = client.try_aggregate_oracle_data(
        &validator,
        &ChainId::Ethereum,
        &report_ids,
        &consensus_hash,
        &agg_sig,
        &1,
    );
    assert_eq!(result, Err(Ok(Error::InsufficientOracleReports)));
}

// ==================== Cryptographic Proof Tests ====================

#[test]
fn test_submit_proof() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let (validator, sk) = setup_validator(&env, &client, &admin);

    env.mock_all_auths();

    let proof_id = BytesN::from_array(&env, &[0xddu8; 32]);
    let record_hash = BytesN::from_array(&env, &[0x11u8; 32]);
    let block_hash = BytesN::from_array(&env, &[0x22u8; 32]);
    let merkle_root = BytesN::from_array(&env, &[0x33u8; 32]);
    let prover = String::from_str(&env, "0x1234567890abcdef1234567890abcdef12345678");

    let proof_sig = create_sig(&env, &sk, &proof_id, 1);
    let result = client.submit_proof(
        &validator,
        &proof_id,
        &ChainId::Ethereum,
        &record_hash,
        &block_hash,
        &merkle_root,
        &prover,
        &proof_sig,
        &1,
    );
    assert_eq!(result, proof_id);

    let proof = client.get_proof(&proof_id).unwrap();
    assert!(!proof.verified);
    assert_eq!(proof.verifier_count, 1);
}

#[test]
fn test_verify_cross_chain_proof() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let (validator1, sk1) = setup_validator(&env, &client, &admin);
    let (validator2, sk2) = setup_validator(&env, &client, &admin);

    env.mock_all_auths();

    let proof_id = BytesN::from_array(&env, &[0xeeu8; 32]);

    let proof_sig1 = create_sig(&env, &sk1, &proof_id, 1);
    client.submit_proof(
        &validator1,
        &proof_id,
        &ChainId::Ethereum,
        &BytesN::from_array(&env, &[1u8; 32]),
        &BytesN::from_array(&env, &[2u8; 32]),
        &BytesN::from_array(&env, &[3u8; 32]),
        &String::from_str(&env, "0x1234567890abcdef1234567890abcdef12345678"),
        &proof_sig1,
        &1,
    );

    // First verification — not yet verified (needs min_confirmations = 2)
    let verify_sig = create_sig(&env, &sk2, &proof_id, 1);
    let verified = client.verify_cross_chain_proof(&validator2, &verify_sig, &1, &proof_id);
    assert!(verified); // 1 (submit) + 1 = 2 => matches min_confirmations

    let proof = client.get_proof(&proof_id).unwrap();
    assert!(proof.verified);
    assert_eq!(proof.verifier_count, 2);
}

#[test]
fn test_proof_not_found() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let (validator, sk) = setup_validator(&env, &client, &admin);

    env.mock_all_auths();

    let bad_id = BytesN::from_array(&env, &[0xffu8; 32]);
    // Proof check happens before signature verification, so dummy sig is fine
    let result = client.try_verify_cross_chain_proof(
        &validator,
        &create_sig(&env, &sk, &bad_id, 1),
        &1,
        &bad_id,
    );
    assert_eq!(result, Err(Ok(Error::ProofNotFound)));
}

// ==================== Address Validation Tests ====================

#[test]
fn test_validate_ethereum_address() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    // Valid EVM address: 42 chars
    let valid = String::from_str(&env, "0x1234567890abcdef1234567890abcdef12345678");
    assert!(client.validate_chain_address(&ChainId::Ethereum, &valid));

    // Invalid: too short
    let invalid = String::from_str(&env, "0x1234");
    assert!(!client.validate_chain_address(&ChainId::Ethereum, &invalid));
}

#[test]
fn test_validate_stellar_address() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    // Valid Stellar StrKey: 56 chars (G + 55 base32 chars)
    let valid = String::from_str(
        &env,
        "GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWNA",
    );
    assert!(client.validate_chain_address(&ChainId::Stellar, &valid));
}

#[test]
fn test_validate_polygon_address() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let valid = String::from_str(&env, "0xabcdef1234567890abcdef1234567890abcdef12");
    assert!(client.validate_chain_address(&ChainId::Polygon, &valid));
    assert!(client.validate_chain_address(&ChainId::Arbitrum, &valid));
    assert!(client.validate_chain_address(&ChainId::Optimism, &valid));
}

#[test]
fn test_get_chain_address_length() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    assert_eq!(client.get_chain_address_length(&ChainId::Stellar), 56);
    assert_eq!(client.get_chain_address_length(&ChainId::Ethereum), 42);
    assert_eq!(client.get_chain_address_length(&ChainId::Polygon), 42);
}

// ==================== Event Synchronization Tests ====================

#[test]
fn test_sync_cross_chain_event() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let (validator, sk) = setup_validator(&env, &client, &admin);

    env.mock_all_auths();

    let payload_hash = BytesN::from_array(&env, &[0x55u8; 32]);
    let sync_sig = create_sig(&env, &sk, &payload_hash, 1);
    let event_id = client.sync_cross_chain_event(
        &validator,
        &ChainId::Ethereum,
        &ChainId::Stellar,
        &CrossChainEventType::RecordCreated,
        &payload_hash,
        &99999,
        &sync_sig,
        &1,
    );

    assert_eq!(event_id, 1);
    assert_eq!(client.get_event_count(), 1);

    let event = client.get_sync_event(&event_id).unwrap();
    assert_eq!(event.sync_status, EventSyncStatus::Pending);
    assert_eq!(event.block_height, 99999);
    assert_eq!(event.source_chain, ChainId::Ethereum);
}

#[test]
fn test_process_sync_event() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let (validator, sk) = setup_validator(&env, &client, &admin);

    env.mock_all_auths();

    let payload_hash = BytesN::from_array(&env, &[0x66u8; 32]);
    let sync_sig = create_sig(&env, &sk, &payload_hash, 1);
    let event_id = client.sync_cross_chain_event(
        &validator,
        &ChainId::Ethereum,
        &ChainId::Stellar,
        &CrossChainEventType::AccessGranted,
        &payload_hash,
        &200,
        &sync_sig,
        &1,
    );

    // process_sync_event signs target_id constructed from event_id
    let mut target_bytes = [0u8; 32];
    target_bytes[24..32].copy_from_slice(&event_id.to_be_bytes());
    let target_id = BytesN::from_array(&env, &target_bytes);
    let process_sig = create_sig(&env, &sk, &target_id, 2);
    client.process_sync_event(
        &validator,
        &event_id,
        &EventSyncStatus::Synced,
        &process_sig,
        &2,
    );

    let event = client.get_sync_event(&event_id).unwrap();
    assert_eq!(event.sync_status, EventSyncStatus::Synced);
}

#[test]
fn test_multiple_events_unique_ids() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let (validator, sk) = setup_validator(&env, &client, &admin);

    env.mock_all_auths();

    let hash1 = BytesN::from_array(&env, &[0x11u8; 32]);
    let hash2 = BytesN::from_array(&env, &[0x22u8; 32]);

    let sig1 = create_sig(&env, &sk, &hash1, 1);
    let id1 = client.sync_cross_chain_event(
        &validator,
        &ChainId::Ethereum,
        &ChainId::Stellar,
        &CrossChainEventType::RecordCreated,
        &hash1,
        &100,
        &sig1,
        &1,
    );

    let sig2 = create_sig(&env, &sk, &hash2, 2);
    let id2 = client.sync_cross_chain_event(
        &validator,
        &ChainId::Polygon,
        &ChainId::Stellar,
        &CrossChainEventType::AccessRevoked,
        &hash2,
        &200,
        &sig2,
        &2,
    );

    assert_ne!(id1, id2);
    assert_eq!(client.get_event_count(), 2);

    let event1 = client.get_sync_event(&id1).unwrap();
    let event2 = client.get_sync_event(&id2).unwrap();
    assert_eq!(event1.source_chain, ChainId::Ethereum);
    assert_eq!(event2.source_chain, ChainId::Polygon);
}

// ==================== Emergency Rollback Tests ====================

#[test]
fn test_initiate_rollback() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let op_id = BytesN::from_array(&env, &[0x77u8; 32]);
    let state_snapshot = String::from_str(&env, "{\"status\":\"pending\"}");
    let reason = String::from_str(&env, "Oracle consensus failure");

    env.mock_all_auths();
    let result = client.initiate_rollback(
        &admin,
        &op_id,
        &RollbackOpType::MessageRollback,
        &state_snapshot,
        &reason,
    );
    assert_eq!(result, op_id);
    assert_eq!(client.get_rollback_count(), 1);

    let rollback = client.get_rollback(&op_id).unwrap();
    assert_eq!(rollback.status, RollbackStatus::Initiated);
    assert_eq!(rollback.triggered_by, admin);
}

#[test]
fn test_execute_rollback_for_message() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let (validator, sk) = setup_validator(&env, &client, &admin);
    let recipient = Address::generate(&env);

    env.mock_all_auths();

    // Submit a message
    let message_id = generate_message_id(&env);
    let v_sig = create_sig(&env, &sk, &message_id, 1);
    client.submit_message(
        &validator,
        &SubmitMessageRequest {
            message_id: message_id.clone(),
            source_chain: ChainId::Ethereum,
            dest_chain: ChainId::Stellar,
            sender: String::from_str(&env, "0x1234"),
            recipient: recipient.clone(),
            payload_type: MessageType::RecordRequest,
            payload: String::from_str(&env, "{}"),
            nonce: 1,
            signature: dummy_sig(&env),
            v_signature: v_sig,
            v_nonce: 1,
        },
    );

    // Initiate and execute rollback on that message
    client.initiate_rollback(
        &admin,
        &message_id,
        &RollbackOpType::MessageRollback,
        &String::from_str(&env, "{\"status\":\"pending\"}"),
        &String::from_str(&env, "Test rollback"),
    );

    let result = client.execute_rollback(&admin, &message_id);
    assert!(result);

    let rollback = client.get_rollback(&message_id).unwrap();
    assert_eq!(rollback.status, RollbackStatus::Completed);

    // Message should now be marked as Failed
    let msg = client.get_message(&message_id).unwrap();
    assert_eq!(msg.status, MessageStatus::Failed);
}

#[test]
fn test_execute_rollback_for_atomic_tx() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let caller = Address::generate(&env);

    env.mock_all_auths();

    let tx_id = BytesN::from_array(&env, &[0x88u8; 32]);
    client.initiate_atomic_tx(
        &caller,
        &tx_id,
        &soroban_sdk::vec![&env, generate_message_id(&env)],
    );

    client.initiate_rollback(
        &admin,
        &tx_id,
        &RollbackOpType::AtomicTxRollback,
        &String::from_str(&env, "{}"),
        &String::from_str(&env, "Rollback atomic tx"),
    );

    client.execute_rollback(&admin, &tx_id);

    let atomic_tx = client.get_atomic_tx(&tx_id).unwrap();
    assert_eq!(atomic_tx.status, AtomicTxStatus::Aborted);
}

#[test]
fn test_cancel_rollback() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let op_id = BytesN::from_array(&env, &[0x99u8; 32]);

    env.mock_all_auths();
    client.initiate_rollback(
        &admin,
        &op_id,
        &RollbackOpType::RecordSyncRollback,
        &String::from_str(&env, "{}"),
        &String::from_str(&env, "Test"),
    );

    let result = client.cancel_rollback(&admin, &op_id);
    assert!(result);

    let rollback = client.get_rollback(&op_id).unwrap();
    assert_eq!(rollback.status, RollbackStatus::Failed);
}

#[test]
fn test_rollback_already_processed_fails() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let op_id = BytesN::from_array(&env, &[0xaau8; 32]);

    env.mock_all_auths();
    client.initiate_rollback(
        &admin,
        &op_id,
        &RollbackOpType::RecordSyncRollback,
        &String::from_str(&env, "{}"),
        &String::from_str(&env, "Test"),
    );
    client.execute_rollback(&admin, &op_id);

    // Second execute should fail
    let result = client.try_execute_rollback(&admin, &op_id);
    assert_eq!(result, Err(Ok(Error::RollbackAlreadyProcessed)));
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

    let (validator, _sk) = setup_validator(&env, &client, &admin);

    env.mock_all_auths();
    client.pause(&admin);

    let result = client.try_submit_message(
        &validator,
        &SubmitMessageRequest {
            message_id: generate_message_id(&env),
            source_chain: ChainId::Ethereum,
            dest_chain: ChainId::Stellar,
            sender: String::from_str(&env, "0x1234567890abcdef"),
            recipient: Address::generate(&env),
            payload_type: MessageType::RecordRequest,
            payload: String::from_str(&env, "{\"record_id\": 1}"),
            nonce: 1,
            signature: dummy_sig(&env),
            v_signature: dummy_sig(&env),
            v_nonce: 1,
        },
    );

    assert_eq!(result, Err(Ok(Error::ContractPaused)));
}

// ==================== Nonce Tests ====================

#[test]
fn test_nonce_replay_protection() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let (validator, sk) = setup_validator(&env, &client, &admin);
    let recipient = Address::generate(&env);

    env.mock_all_auths();

    let sender = String::from_str(&env, "0x1234567890abcdef");

    let msg_id_1 = BytesN::from_array(&env, &[1u8; 32]);
    let v_sig_1 = create_sig(&env, &sk, &msg_id_1, 1);
    client.submit_message(
        &validator,
        &SubmitMessageRequest {
            message_id: msg_id_1,
            source_chain: ChainId::Ethereum,
            dest_chain: ChainId::Stellar,
            sender: sender.clone(),
            recipient: recipient.clone(),
            payload_type: MessageType::RecordRequest,
            payload: String::from_str(&env, "{}"),
            nonce: 1,
            signature: dummy_sig(&env),
            v_signature: v_sig_1,
            v_nonce: 1,
        },
    );

    // Same nonce should fail (nonce 1 already used for this validator's pubkey)
    let msg_id_4 = BytesN::from_array(&env, &[4u8; 32]);
    let v_sig_2 = create_sig(&env, &sk, &msg_id_4, 2); // new nonce for second call
    let result = client.try_submit_message(
        &validator,
        &SubmitMessageRequest {
            message_id: msg_id_4,
            source_chain: ChainId::Ethereum,
            dest_chain: ChainId::Stellar,
            sender: sender.clone(),
            recipient: recipient.clone(),
            payload_type: MessageType::RecordRequest,
            payload: String::from_str(&env, "{}"),
            nonce: 1, // <-- sender nonce, not validator nonce
            signature: dummy_sig(&env),
            v_signature: v_sig_2, // validator nonce is 2, which IS valid
            v_nonce: 2,
        },
    );

    // This test checks sender nonce replay, not validator nonce
    assert_eq!(result, Err(Ok(Error::InvalidNonce)));
}

// ==================== Chaos / Resilience Tests ====================

/// Chaos test: message sent but acknowledgment lost — verify idempotent retry works
#[test]
fn test_chaos_lost_acknowledgment_retry() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let (validator, sk) = setup_validator(&env, &client, &admin);
    let recipient = Address::generate(&env);

    env.mock_all_auths();

    let message_id = BytesN::from_array(&env, &[0xca_u8; 32]);
    let v_sig = create_sig(&env, &sk, &message_id, 1);
    client.submit_message(
        &validator,
        &SubmitMessageRequest {
            message_id: message_id.clone(),
            source_chain: ChainId::Ethereum,
            dest_chain: ChainId::Stellar,
            sender: String::from_str(&env, "0xSENDER"),
            recipient: recipient.clone(),
            payload_type: MessageType::RecordRequest,
            payload: String::from_str(&env, "{\"record_id\":42}"),
            nonce: 1,
            signature: dummy_sig(&env),
            v_signature: v_sig,
            v_nonce: 1,
        },
    );

    let msg = client.get_message(&message_id).unwrap();
    assert_eq!(msg.status, MessageStatus::Pending);

    // Acknowledge (confirm) once — needs new nonce
    let conf_sig = create_sig(&env, &sk, &message_id, 2);
    client.confirm_message(&validator, &message_id, &conf_sig, &2);

    // Second confirm by the same validator must be rejected (idempotency guard)
    // Need a new nonce to call confirm_message (nonce check passes), but duplicate validator
    let dup_sig = create_sig(&env, &sk, &message_id, 3);
    let dup = client.try_confirm_message(&validator, &message_id, &dup_sig, &3);
    assert_eq!(dup, Err(Ok(Error::DuplicateConfirmation)));

    // Message should still be manageable
    let msg = client.get_message(&message_id).unwrap();
    assert!(msg.status == MessageStatus::Pending || msg.status == MessageStatus::Verified);
}

/// Chaos test: partial message delivery — verify atomic all-or-nothing
#[test]
fn test_chaos_partial_delivery_atomicity() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let (validator, sk) = setup_validator(&env, &client, &admin);
    let recipient = Address::generate(&env);

    env.mock_all_auths();

    // Create an atomic transaction with two message IDs
    let tx_id = BytesN::from_array(&env, &[0xbb_u8; 32]);
    let msg_id_a = BytesN::from_array(&env, &[0xbc_u8; 32]);
    let msg_id_b = BytesN::from_array(&env, &[0xbd_u8; 32]);

    let v_sig_a = create_sig(&env, &sk, &msg_id_a, 1);
    client.submit_message(
        &validator,
        &SubmitMessageRequest {
            message_id: msg_id_a.clone(),
            source_chain: ChainId::Ethereum,
            dest_chain: ChainId::Stellar,
            sender: String::from_str(&env, "0xA"),
            recipient: recipient.clone(),
            payload_type: MessageType::RecordRequest,
            payload: String::from_str(&env, "{\"data\":\"a\"}"),
            nonce: 1,
            signature: dummy_sig(&env),
            v_signature: v_sig_a,
            v_nonce: 1,
        },
    );
    let v_sig_b = create_sig(&env, &sk, &msg_id_b, 2);
    client.submit_message(
        &validator,
        &SubmitMessageRequest {
            message_id: msg_id_b.clone(),
            source_chain: ChainId::Ethereum,
            dest_chain: ChainId::Stellar,
            sender: String::from_str(&env, "0xB"),
            recipient: recipient.clone(),
            payload_type: MessageType::RecordRequest,
            payload: String::from_str(&env, "{\"data\":\"b\"}"),
            nonce: 2,
            signature: dummy_sig(&env),
            v_signature: v_sig_b,
            v_nonce: 2,
        },
    );

    let message_ids = soroban_sdk::vec![&env, msg_id_a.clone(), msg_id_b.clone()];
    client.initiate_atomic_tx(&recipient, &tx_id, &message_ids);

    // Only confirm message A, not B
    let conf_sig = create_sig(&env, &sk, &msg_id_a, 3);
    client.confirm_message(&validator, &msg_id_a, &conf_sig, &3);

    // Atomic tx should NOT be committable — partial state
    let atomic_tx = client.get_atomic_tx(&tx_id).unwrap();
    assert_eq!(atomic_tx.status, AtomicTxStatus::Initiated);

    // Abort the atomic tx — clean rollback
    client.abort_atomic_tx(&recipient, &tx_id);
    let atomic_tx = client.get_atomic_tx(&tx_id).unwrap();
    assert_eq!(atomic_tx.status, AtomicTxStatus::Aborted);

    // Individual messages should still be in their own states
    let msg_a = client.get_message(&msg_id_a).unwrap();
    let msg_b = client.get_message(&msg_id_b).unwrap();
    assert_eq!(msg_a.status, MessageStatus::Pending);
    assert_eq!(msg_b.status, MessageStatus::Pending);
}

/// Chaos test: relayer goes offline mid-batch — verify pending messages are recoverable
#[test]
fn test_chaos_relayer_offline_recovery() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let (validator, sk) = setup_validator(&env, &client, &admin);
    let recipient = Address::generate(&env);

    env.mock_all_auths();

    let mut ids = soroban_sdk::vec![&env];
    for i in 0..5u64 {
        let mut arr = [0u8; 32];
        arr[31] = i as u8;
        let msg_id = BytesN::from_array(&env, &arr);
        let nonce = i + 1;
        let v_sig = create_sig(&env, &sk, &msg_id, nonce);
        client.submit_message(
            &validator,
            &SubmitMessageRequest {
                message_id: msg_id.clone(),
                source_chain: ChainId::Ethereum,
                dest_chain: ChainId::Stellar,
                sender: String::from_str(&env, "0xRELAY"),
                recipient: recipient.clone(),
                payload_type: MessageType::RecordSync,
                payload: String::from_str(&env, "{\"seq\":"),
                nonce,
                signature: dummy_sig(&env),
                v_signature: v_sig,
                v_nonce: nonce,
            },
        );
        ids.push_back(msg_id);
    }

    // Confirm only first 2 (simulating relayer going offline)
    let conf_sig_0 = create_sig(&env, &sk, &ids.get(0).unwrap(), 6);
    let conf_sig_1 = create_sig(&env, &sk, &ids.get(1).unwrap(), 7);
    client.confirm_message(&validator, &ids.get(0).unwrap(), &conf_sig_0, &6);
    client.confirm_message(&validator, &ids.get(1).unwrap(), &conf_sig_1, &7);

    // Remaining messages should still be Pending (recoverable)
    for i in 2..5 {
        let msg = client.get_message(&ids.get(i).unwrap()).unwrap();
        assert_eq!(
            msg.status,
            MessageStatus::Pending,
            "Message {} should be recoverable",
            i
        );
    }

    // After relayer comes back online, confirm remaining
    for i in 2..5 {
        let conf_sig = create_sig(&env, &sk, &ids.get(i).unwrap(), (8 + i - 2) as u64);
        client.confirm_message(
            &validator,
            &ids.get(i).unwrap(),
            &conf_sig,
            &((8 + i - 2) as u64),
        );
    }

    // All messages remain Pending with single validator (min_confirmations=2)
    for i in 0..5 {
        let msg = client.get_message(&ids.get(i).unwrap()).unwrap();
        assert_eq!(
            msg.status,
            MessageStatus::Pending,
            "Message {} is still recoverable",
            i
        );
    }
}

#[test]
fn test_error_codes_are_stable() {
    assert_eq!(Error::Unauthorized as u32, 100);
    assert_eq!(Error::InsufficientConfirmations as u32, 120);
    assert_eq!(Error::InvalidSignature as u32, 207);
    assert_eq!(Error::AlreadyInitialized as u32, 301);
    assert_eq!(Error::ContractPaused as u32, 302);
    assert_eq!(Error::MessageNotFound as u32, 480);
    assert_eq!(Error::InvalidChain as u32, 703);
}

#[test]
fn test_get_suggestion_returns_expected_hint() {
    use soroban_sdk::symbol_short;
    assert_eq!(
        crate::errors::get_suggestion(Error::Unauthorized),
        symbol_short!("CHK_AUTH")
    );
    assert_eq!(
        crate::errors::get_suggestion(Error::AlreadyInitialized),
        symbol_short!("ALREADY")
    );
    assert_eq!(
        crate::errors::get_suggestion(Error::ContractPaused),
        symbol_short!("RE_TRY_L")
    );
    assert_eq!(
        crate::errors::get_suggestion(Error::MessageNotFound),
        symbol_short!("CHK_ID")
    );
}

// ==================== Chaos Testing: Network Partition Scenarios ====================

/// Simulates message sent but acknowledgment lost — verifies idempotent retry works
#[test]
fn test_chaos_acknowledgment_lost_idempotent_retry() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let (validator1, sk1) = setup_validator(&env, &client, &admin);
    let (validator2, sk2) = setup_validator(&env, &client, &admin);
    let recipient = Address::generate(&env);

    env.mock_all_auths();

    // Submit message (simulates successful submission)
    let msg_id = generate_message_id(&env);
    let v_sig = create_sig(&env, &sk1, &msg_id, 1);
    client.submit_message(
        &validator1,
        &SubmitMessageRequest {
            message_id: msg_id.clone(),
            source_chain: ChainId::Ethereum,
            dest_chain: ChainId::Stellar,
            sender: String::from_str(&env, "0x1234"),
            recipient: recipient.clone(),
            payload_type: MessageType::RecordRequest,
            payload: String::from_str(&env, "{\"record_id\": 1}"),
            nonce: 1,
            signature: dummy_sig(&env),
            v_signature: v_sig,
            v_nonce: 1,
        },
    );

    // First confirmation (acknowledgment is "lost" - not processed further)
    let conf_sig1 = create_sig(&env, &sk1, &msg_id, 2);
    client.confirm_message(&validator1, &msg_id, &conf_sig1, &2);

    // Second confirmation (retry - idempotent)
    let conf_sig2 = create_sig(&env, &sk2, &msg_id, 1);
    client.confirm_message(&validator2, &msg_id, &conf_sig2, &1);

    // Message should be Verified despite simulated ack loss
    let msg = client.get_message(&msg_id).unwrap();
    assert_eq!(msg.status, MessageStatus::Verified);

    // Duplicate confirmation should error (message is already Verified)
    let dup_sig = create_sig(&env, &sk2, &msg_id, 2);
    let result = client.try_confirm_message(&validator2, &msg_id, &dup_sig, &2);
    assert_eq!(result, Err(Ok(Error::MessageAlreadyProcessed)));
}

/// Simulates partial message delivery — verifies atomicity (all or nothing)
#[test]
fn test_chaos_partial_message_delivery_atomicity() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let (validator, sk) = setup_validator(&env, &client, &admin);
    let recipient = Address::generate(&env);

    env.mock_all_auths();

    // Submit message A (simulates partial delivery - never confirmed)
    let msg_id_a = BytesN::from_array(&env, &[0xaa; 32]);
    let v_sig_a = create_sig(&env, &sk, &msg_id_a, 1);
    client.submit_message(
        &validator,
        &SubmitMessageRequest {
            message_id: msg_id_a.clone(),
            source_chain: ChainId::Ethereum,
            dest_chain: ChainId::Stellar,
            sender: String::from_str(&env, "0xAAA"),
            recipient: recipient.clone(),
            payload_type: MessageType::RecordRequest,
            payload: String::from_str(&env, "{\"record_id\": 1}"),
            nonce: 1,
            signature: dummy_sig(&env),
            v_signature: v_sig_a,
            v_nonce: 1,
        },
    );

    // Message B sent at nonce 2 (nonce prevents replay on A)
    let msg_id_b = BytesN::from_array(&env, &[0xbb; 32]);
    let v_sig_b = create_sig(&env, &sk, &msg_id_b, 2);
    client.submit_message(
        &validator,
        &SubmitMessageRequest {
            message_id: msg_id_b.clone(),
            source_chain: ChainId::Ethereum,
            dest_chain: ChainId::Stellar,
            sender: String::from_str(&env, "0xAAA"),
            recipient: recipient.clone(),
            payload_type: MessageType::RecordRequest,
            payload: String::from_str(&env, "{\"record_id\": 2}"),
            nonce: 2,
            signature: dummy_sig(&env),
            v_signature: v_sig_b,
            v_nonce: 2,
        },
    );

    // Only message B is confirmed by both validators
    let (validator2, sk2) = setup_validator(&env, &client, &admin);
    let conf_sig_b1 = create_sig(&env, &sk, &msg_id_b, 3);
    let conf_sig_b2 = create_sig(&env, &sk2, &msg_id_b, 1);
    client.confirm_message(&validator, &msg_id_b, &conf_sig_b1, &3);
    client.confirm_message(&validator2, &msg_id_b, &conf_sig_b2, &1);

    // Message A should still be Pending (not lost, not executed)
    let msg_a = client.get_message(&msg_id_a).unwrap();
    assert_eq!(msg_a.status, MessageStatus::Pending);

    // Message B should be Verified
    let msg_b = client.get_message(&msg_id_b).unwrap();
    assert_eq!(msg_b.status, MessageStatus::Verified);

    // Execute only message B
    assert!(client.execute_message(&recipient, &msg_id_b));
    assert_eq!(
        client.get_message(&msg_id_b).unwrap().status,
        MessageStatus::Executed
    );
}

/// Simulates relayer going offline mid-batch — verifies pending messages are recoverable
#[test]
fn test_chaos_relayer_offline_mid_batch_recovery() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let (validator1, sk1) = setup_validator(&env, &client, &admin);
    let (validator2, sk2) = setup_validator(&env, &client, &admin);
    let recipient = Address::generate(&env);

    env.mock_all_auths();

    // Submit messages for a batch
    let mut msg_ids: Vec<BytesN<32>> = Vec::new(&env);
    for i in 0..3u64 {
        let mut bytes = [0u8; 32];
        bytes[0] = (i + 1) as u8;
        msg_ids.push_back(BytesN::from_array(&env, &bytes));
    }

    // Submit all messages (relayer is online for submission)
    for (i, msg_id) in msg_ids.iter().enumerate() {
        let nonce = i as u64 + 1;
        let v_sig = create_sig(&env, &sk1, &msg_id, nonce);
        client.submit_message(
            &validator1,
            &SubmitMessageRequest {
                message_id: msg_id.clone(),
                source_chain: ChainId::Ethereum,
                dest_chain: ChainId::Stellar,
                sender: String::from_str(&env, "0xBatch"),
                recipient: recipient.clone(),
                payload_type: MessageType::RecordSync,
                payload: String::from_str(&env, "{\"batch\": true}"),
                nonce,
                signature: dummy_sig(&env),
                v_signature: v_sig,
                v_nonce: nonce,
            },
        );
    }

    // Relayer goes offline — only first two messages get confirmed
    let conf_sig_0 = create_sig(&env, &sk1, &msg_ids.get(0).unwrap(), 4);
    let conf_sig_1 = create_sig(&env, &sk1, &msg_ids.get(1).unwrap(), 5);
    client.confirm_message(&validator1, &msg_ids.get(0).unwrap(), &conf_sig_0, &4);
    client.confirm_message(&validator1, &msg_ids.get(1).unwrap(), &conf_sig_1, &5);

    // Third message remains unconfirmed (relayer offline)
    let msg_2 = client.get_message(&msg_ids.get(2).unwrap()).unwrap();
    assert_eq!(msg_2.status, MessageStatus::Pending);

    // Relayer comes back online — can still confirm remaining messages
    let conf_sig_2 = create_sig(&env, &sk1, &msg_ids.get(2).unwrap(), 6);
    let conf_sig_v2 = create_sig(&env, &sk2, &msg_ids.get(2).unwrap(), 1);
    client.confirm_message(&validator1, &msg_ids.get(2).unwrap(), &conf_sig_2, &6);
    client.confirm_message(&validator2, &msg_ids.get(2).unwrap(), &conf_sig_v2, &1);

    let msg_2 = client.get_message(&msg_ids.get(2).unwrap()).unwrap();
    assert_eq!(msg_2.status, MessageStatus::Verified);
}

/// Chaos test: oracle goes offline mid-consensus — verifies incomplete data is not aggregated
#[test]
fn test_chaos_oracle_offline_mid_consensus() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let (validator, sk) = setup_validator(&env, &client, &admin);
    let oracle1 = Address::generate(&env);
    let oracle2 = Address::generate(&env);
    let chains = soroban_sdk::vec![&env, ChainId::Ethereum];
    let dummy_pk = BytesN::from_array(&env, &[3u8; 32]);

    env.mock_all_auths();

    // Register 3 oracles
    for oracle in [&oracle1, &oracle2] {
        client.register_oracle(&admin, oracle, &dummy_pk, &chains);
    }

    // Only 2 out of 3 reports (oracle #3 is offline)
    let data_hash = BytesN::from_array(&env, &[0xcc; 32]);
    client.submit_oracle_report(
        &oracle1,
        &ChainId::Ethereum,
        &data_hash,
        &String::from_str(&env, "{}"),
        &100,
        &dummy_sig(&env),
    );
    client.submit_oracle_report(
        &oracle2,
        &ChainId::Ethereum,
        &data_hash,
        &String::from_str(&env, "{}"),
        &100,
        &dummy_sig(&env),
    );

    // Attempt to aggregate with only 2 reports (MIN_ORACLE_REPORTS = 3)
    let report_ids = soroban_sdk::vec![&env, 1u64, 2u64];
    let agg_sig = create_sig(&env, &sk, &data_hash, 1);
    let result = client.try_aggregate_oracle_data(
        &validator,
        &ChainId::Ethereum,
        &report_ids,
        &data_hash,
        &agg_sig,
        &1,
    );
    assert_eq!(result, Err(Ok(Error::InsufficientOracleReports)));
}

// ==================== Batch Submit Message Tests ====================

#[test]
fn test_submit_message_batch_empty_fails() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);
    let (validator, _sk) = setup_validator(&env, &client, &admin);

    env.mock_all_auths();
    let result = client.try_submit_message_batch(&validator, &Vec::new(&env));
    assert_eq!(result, Err(Ok(Error::BatchTooLarge)));
}

#[test]
fn test_submit_message_batch_single() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);
    let (validator, sk) = setup_validator(&env, &client, &admin);
    let recipient = Address::generate(&env);

    let message_id = generate_message_id(&env);
    let sender = String::from_str(&env, "0xsender");
    let payload = String::from_str(&env, "{}");

    env.mock_all_auths();
    let v_sig = create_sig(&env, &sk, &message_id, 1);
    let request = SubmitMessageRequest {
        message_id: message_id.clone(),
        source_chain: ChainId::Ethereum,
        dest_chain: ChainId::Stellar,
        sender: sender.clone(),
        recipient: recipient.clone(),
        payload_type: MessageType::RecordRequest,
        payload: payload.clone(),
        nonce: 1,
        signature: dummy_sig(&env),
        v_signature: v_sig,
        v_nonce: 1,
    };

    let ids = client.submit_message_batch(&validator, &soroban_sdk::vec![&env, request]);
    assert_eq!(ids.len(), 1);
    assert_eq!(ids.get(0).unwrap(), message_id);
    assert_eq!(client.get_message_count(), 1);

    let msg = client.get_message(&message_id).unwrap();
    assert_eq!(msg.status, MessageStatus::Pending);
}

#[test]
fn test_submit_message_batch_multiple() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);
    let (validator, sk) = setup_validator(&env, &client, &admin);
    let recipient = Address::generate(&env);
    let sender = String::from_str(&env, "0xsender");

    env.mock_all_auths();
    let id1 = BytesN::from_array(&env, &[0x01; 32]);
    let id2 = BytesN::from_array(&env, &[0x02; 32]);

    let v_sig1 = create_sig(&env, &sk, &id1, 1);
    let v_sig2 = create_sig(&env, &sk, &id2, 2);

    let req1 = SubmitMessageRequest {
        message_id: id1.clone(),
        source_chain: ChainId::Ethereum,
        dest_chain: ChainId::Stellar,
        sender: sender.clone(),
        recipient: recipient.clone(),
        payload_type: MessageType::RecordRequest,
        payload: String::from_str(&env, "{}"),
        nonce: 1,
        signature: dummy_sig(&env),
        v_signature: v_sig1,
        v_nonce: 1,
    };
    let req2 = SubmitMessageRequest {
        message_id: id2.clone(),
        source_chain: ChainId::Polygon,
        dest_chain: ChainId::Stellar,
        sender: sender.clone(),
        recipient: recipient.clone(),
        payload_type: MessageType::RecordResponse,
        payload: String::from_str(&env, r#"{"data":"test"}"#),
        nonce: 2,
        signature: dummy_sig(&env),
        v_signature: v_sig2,
        v_nonce: 2,
    };

    let ids = client.submit_message_batch(&validator, &soroban_sdk::vec![&env, req1, req2]);
    assert_eq!(ids.len(), 2);
    assert_eq!(client.get_message_count(), 2);

    assert!(client.get_message(&id1).is_some());
    assert!(client.get_message(&id2).is_some());
}

#[test]
fn test_submit_message_batch_too_large() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);
    let (validator, sk) = setup_validator(&env, &client, &admin);
    let recipient = Address::generate(&env);
    let sender = String::from_str(&env, "0xsender");

    env.mock_all_auths();
    let mut requests = Vec::new(&env);
    for i in 0..51 {
        let mid = BytesN::from_array(&env, &[i as u8; 32]);
        let v_sig = create_sig(&env, &sk, &mid, i as u64 + 1);
        requests.push_back(SubmitMessageRequest {
            message_id: mid,
            source_chain: ChainId::Ethereum,
            dest_chain: ChainId::Stellar,
            sender: sender.clone(),
            recipient: recipient.clone(),
            payload_type: MessageType::RecordRequest,
            payload: String::from_str(&env, "{}"),
            nonce: i as u64 + 1,
            signature: dummy_sig(&env),
            v_signature: v_sig,
            v_nonce: i as u64 + 1,
        });
    }

    let result = client.try_submit_message_batch(&validator, &requests);
    assert_eq!(result, Err(Ok(Error::BatchTooLarge)));
}

#[test]
fn test_submit_message_batch_rejects_invalid_chain() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);
    let (validator, sk) = setup_validator(&env, &client, &admin);
    let recipient = Address::generate(&env);
    let sender = String::from_str(&env, "0xsender");

    let mid = generate_message_id(&env);
    let v_sig = create_sig(&env, &sk, &mid, 1);

    env.mock_all_auths();
    let request = SubmitMessageRequest {
        message_id: mid,
        source_chain: ChainId::BinanceSmartChain,
        dest_chain: ChainId::Stellar,
        sender,
        recipient,
        payload_type: MessageType::RecordRequest,
        payload: String::from_str(&env, "{}"),
        nonce: 1,
        signature: dummy_sig(&env),
        v_signature: v_sig,
        v_nonce: 1,
    };

    let result = client.try_submit_message_batch(&validator, &soroban_sdk::vec![&env, request]);
    assert_eq!(result, Err(Ok(Error::ChainNotSupported)));
}

// ==================== Offline Reconciliation Tests ====================

#[test]
fn test_reconcile_offline_creates_request() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let caller = Address::generate(&env);
    let record_id = BytesN::from_array(&env, &[2u8; 32]);
    let state_hash = BytesN::from_array(&env, &[3u8; 32]);
    let mut record_ids = soroban_sdk::Vec::new(&env);
    record_ids.push_back(record_id);

    env.mock_all_auths();
    let request_id = client.reconcile_offline(
        &caller,
        &ChainId::Stellar,
        &ChainId::Ethereum,
        &record_ids,
        &state_hash,
        &86400u64,
    );

    assert!(!request_id.to_array().is_empty());
}

#[test]
fn test_reconcile_offline_rejects_empty_records() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let caller = Address::generate(&env);
    let state_hash = BytesN::from_array(&env, &[3u8; 32]);
    let empty_ids: soroban_sdk::Vec<BytesN<32>> = soroban_sdk::Vec::new(&env);

    env.mock_all_auths();
    let result = client.try_reconcile_offline(
        &caller,
        &ChainId::Stellar,
        &ChainId::Ethereum,
        &empty_ids,
        &state_hash,
        &86400u64,
    );
    assert_eq!(result, Err(Ok(Error::InvalidPayload)));
}

#[test]
fn test_queue_async_reconciliation() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let caller = Address::generate(&env);
    let record_id = BytesN::from_array(&env, &[4u8; 32]);
    let state_hash = BytesN::from_array(&env, &[5u8; 32]);
    let mut record_ids = soroban_sdk::Vec::new(&env);
    record_ids.push_back(record_id);

    env.mock_all_auths();
    let job_id = client.queue_async_reconciliation(
        &caller,
        &ChainId::Stellar,
        &ChainId::Ethereum,
        &record_ids,
        &state_hash,
        &3u32,
    );

    assert_eq!(job_id, 1);
}

#[test]
fn test_complete_reconciliation() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let caller = Address::generate(&env);
    let record_id = BytesN::from_array(&env, &[6u8; 32]);
    let state_hash = BytesN::from_array(&env, &[7u8; 32]);
    let mut record_ids = soroban_sdk::Vec::new(&env);
    record_ids.push_back(record_id);

    env.mock_all_auths();
    let request_id = client.reconcile_offline(
        &caller,
        &ChainId::Stellar,
        &ChainId::Ethereum,
        &record_ids,
        &state_hash,
        &86400u64,
    );

    env.mock_all_auths();
    let result = client.complete_reconciliation(
        &caller,
        &request_id,
        &1u32,
        &0u32,
        &0u32,
        &true,
        &String::from_str(&env, "All records reconciled"),
    );
    assert!(result);

    let status = client.get_reconciliation_status(&request_id);
    assert!(status.is_some());
    let res = status.unwrap();
    assert_eq!(res.reconciled_records, 1);
    assert_eq!(res.failed_records, 0);
    assert!(res.state_hash_matched);
}
