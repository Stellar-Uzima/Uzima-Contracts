#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
use crate::{
    AtomicTxStatus, ChainId, CrossChainBridgeContract, CrossChainBridgeContractClient,
    CrossChainEventType, Error, EventSyncStatus, MessageStatus, MessageType, OracleStatus,
    RollbackOpType, RollbackStatus, SyncStatus,
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

fn setup_validator(
    env: &Env,
    client: &CrossChainBridgeContractClient,
    admin: &Address,
) -> Address {
    let validator = Address::generate(env);
    let public_key = generate_public_key(env);
    env.mock_all_auths();
    client.add_validator(admin, &validator, &public_key, &1000);
    validator
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

    let validator = setup_validator(&env, &client, &admin);
    let recipient = Address::generate(&env);

    let message_id = generate_message_id(&env);
    let sender = String::from_str(&env, "0x1234567890abcdef");
    let payload = String::from_str(&env, "{\"record_id\": 1}");
    let signature = generate_signature(&env);

    env.mock_all_auths();
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

    let validator = setup_validator(&env, &client, &admin);
    let recipient = Address::generate(&env);

    env.mock_all_auths();
    let result = client.try_submit_message(
        &validator,
        &generate_message_id(&env),
        &ChainId::BinanceSmartChain,
        &ChainId::Stellar,
        &String::from_str(&env, "0x1234"),
        &recipient,
        &MessageType::RecordRequest,
        &String::from_str(&env, "{}"),
        &1,
        &generate_signature(&env),
    );

    assert_eq!(result, Err(Ok(Error::ChainNotSupported)));
}

#[test]
fn test_confirm_message() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let validator1 = setup_validator(&env, &client, &admin);
    let validator2 = setup_validator(&env, &client, &admin);
    let recipient = Address::generate(&env);

    env.mock_all_auths();

    let message_id = generate_message_id(&env);
    client.submit_message(
        &validator1,
        &message_id,
        &ChainId::Ethereum,
        &ChainId::Stellar,
        &String::from_str(&env, "0x1234567890abcdef"),
        &recipient,
        &MessageType::RecordRequest,
        &String::from_str(&env, "{\"record_id\": 1}"),
        &1,
        &generate_signature(&env),
    );

    client.confirm_message(&validator1, &message_id);
    let msg = client.get_message(&message_id).unwrap();
    assert_eq!(msg.status, MessageStatus::Pending);

    client.confirm_message(&validator2, &message_id);
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

    let validator1 = setup_validator(&env, &client, &admin);
    let validator2 = setup_validator(&env, &client, &admin);
    let recipient = Address::generate(&env);

    env.mock_all_auths();

    // Submit message A
    let msg_id_a = BytesN::from_array(&env, &[0xaau8; 32]);
    client.submit_message(
        &validator1,
        &msg_id_a,
        &ChainId::Ethereum,
        &ChainId::Stellar,
        &String::from_str(&env, "0xAAA"),
        &recipient,
        &MessageType::RecordRequest,
        &String::from_str(&env, "{}"),
        &1,
        &generate_signature(&env),
    );

    // Submit message B
    let msg_id_b = BytesN::from_array(&env, &[0xbbu8; 32]);
    client.submit_message(
        &validator1,
        &msg_id_b,
        &ChainId::Ethereum,
        &ChainId::Stellar,
        &String::from_str(&env, "0xBBB"),
        &recipient,
        &MessageType::RecordRequest,
        &String::from_str(&env, "{}"),
        &2,
        &generate_signature(&env),
    );

    // Confirm only message A with both validators
    client.confirm_message(&validator1, &msg_id_a);
    client.confirm_message(&validator2, &msg_id_a);

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
    let validator = setup_validator(&env, &client, &admin);

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
    client.update_sync_status(&validator, &1, &ChainId::Ethereum, &SyncStatus::Synced);

    let eth_ref = client.get_record_ref(&1, &ChainId::Ethereum).unwrap();
    let poly_ref = client.get_record_ref(&1, &ChainId::Polygon).unwrap();
    let eth_ref2 = client.get_record_ref(&2, &ChainId::Ethereum).unwrap();

    assert_eq!(eth_ref.sync_status, SyncStatus::Synced);
    assert_eq!(poly_ref.sync_status, SyncStatus::PendingSync); // unaffected
    assert_eq!(eth_ref2.sync_status, SyncStatus::PendingSync); // unaffected
    assert_eq!(eth_ref.external_record_id, String::from_str(&env, "eth_record_001"));
    assert_eq!(poly_ref.external_record_id, String::from_str(&env, "poly_record_001"));
}

#[test]
fn test_execute_message() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let validator1 = setup_validator(&env, &client, &admin);
    let validator2 = setup_validator(&env, &client, &admin);
    let recipient = Address::generate(&env);

    env.mock_all_auths();

    let message_id = generate_message_id(&env);
    client.submit_message(
        &validator1,
        &message_id,
        &ChainId::Ethereum,
        &ChainId::Stellar,
        &String::from_str(&env, "0x1234567890abcdef"),
        &recipient,
        &MessageType::RecordRequest,
        &String::from_str(&env, "{\"record_id\": 1}"),
        &1,
        &generate_signature(&env),
    );

    client.confirm_message(&validator1, &message_id);
    client.confirm_message(&validator2, &message_id);

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

    let validator1 = setup_validator(&env, &client, &admin);
    let validator2 = setup_validator(&env, &client, &admin);
    let caller = Address::generate(&env);

    env.mock_all_auths();

    let tx_id = BytesN::from_array(&env, &[5u8; 32]);
    let message_ids = soroban_sdk::vec![&env, generate_message_id(&env)];

    let result = client.initiate_atomic_tx(&caller, &tx_id, &message_ids);
    assert_eq!(result, tx_id);

    let atomic_tx = client.get_atomic_tx(&tx_id).unwrap();
    assert_eq!(atomic_tx.status, AtomicTxStatus::Initiated);

    client.prepare_atomic_tx(&validator1, &tx_id);
    let atomic_tx = client.get_atomic_tx(&tx_id).unwrap();
    assert_eq!(atomic_tx.status, AtomicTxStatus::Initiated);

    client.prepare_atomic_tx(&validator2, &tx_id);
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

    let validator = setup_validator(&env, &client, &admin);
    let caller = Address::generate(&env);

    env.mock_all_auths();
    let external_record_id = String::from_str(&env, "eth_record_123");
    client.register_record_ref(&caller, &1, &ChainId::Ethereum, &external_record_id);

    client.update_sync_status(&validator, &1, &ChainId::Ethereum, &SyncStatus::Synced);

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
    let public_key = generate_public_key(&env);
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
    let public_key = generate_public_key(&env);
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
    let public_key = generate_public_key(&env);
    let chains = soroban_sdk::vec![&env, ChainId::Ethereum];

    env.mock_all_auths();
    client.register_oracle(&admin, &oracle, &public_key, &chains);

    let data_hash = BytesN::from_array(&env, &[0xabu8; 32]);
    let data = String::from_str(&env, "{\"block\": 12345678}");
    let signature = generate_signature(&env);

    let report_id =
        client.submit_oracle_report(&oracle, &ChainId::Ethereum, &data_hash, &data, &100000, &signature);

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

    let validator = setup_validator(&env, &client, &admin);

    // Register 3 oracles and submit reports
    let oracle1 = Address::generate(&env);
    let oracle2 = Address::generate(&env);
    let oracle3 = Address::generate(&env);
    let chains = soroban_sdk::vec![&env, ChainId::Ethereum];
    let pk = generate_public_key(&env);
    let data_hash = BytesN::from_array(&env, &[0xabu8; 32]);
    let sig = generate_signature(&env);

    env.mock_all_auths();

    let mut report_ids = soroban_sdk::vec![&env];
    for oracle in [&oracle1, &oracle2, &oracle3] {
        client.register_oracle(&admin, oracle, &pk, &chains);
        let rid = client.submit_oracle_report(
            oracle,
            &ChainId::Ethereum,
            &data_hash,
            &String::from_str(&env, "{}"),
            &100,
            &sig,
        );
        report_ids.push_back(rid);
    }

    let consensus_hash = BytesN::from_array(&env, &[0xccu8; 32]);
    let result = client.aggregate_oracle_data(
        &validator,
        &ChainId::Ethereum,
        &report_ids,
        &consensus_hash,
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

    let validator = setup_validator(&env, &client, &admin);

    env.mock_all_auths();

    // Only 2 reports (below MIN_ORACLE_REPORTS = 3)
    let report_ids = soroban_sdk::vec![&env, 1u64, 2u64];
    let consensus_hash = BytesN::from_array(&env, &[0xaau8; 32]);

    let result = client.try_aggregate_oracle_data(
        &validator,
        &ChainId::Ethereum,
        &report_ids,
        &consensus_hash,
    );
    assert_eq!(result, Err(Ok(Error::InsufficientOracleReports)));
}

// ==================== Cryptographic Proof Tests ====================

#[test]
fn test_submit_proof() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let validator = setup_validator(&env, &client, &admin);

    env.mock_all_auths();

    let proof_id = BytesN::from_array(&env, &[0xddu8; 32]);
    let record_hash = BytesN::from_array(&env, &[0x11u8; 32]);
    let block_hash = BytesN::from_array(&env, &[0x22u8; 32]);
    let merkle_root = BytesN::from_array(&env, &[0x33u8; 32]);
    let prover = String::from_str(&env, "0x1234567890abcdef1234567890abcdef12345678");

    let result = client.submit_proof(
        &validator,
        &proof_id,
        &ChainId::Ethereum,
        &record_hash,
        &block_hash,
        &merkle_root,
        &prover,
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

    let validator1 = setup_validator(&env, &client, &admin);
    let validator2 = setup_validator(&env, &client, &admin);

    env.mock_all_auths();

    let proof_id = BytesN::from_array(&env, &[0xeeu8; 32]);

    client.submit_proof(
        &validator1,
        &proof_id,
        &ChainId::Ethereum,
        &BytesN::from_array(&env, &[1u8; 32]),
        &BytesN::from_array(&env, &[2u8; 32]),
        &BytesN::from_array(&env, &[3u8; 32]),
        &String::from_str(&env, "0x1234567890abcdef1234567890abcdef12345678"),
    );

    // First verification — not yet verified (needs min_confirmations = 2)
    let verified = client.verify_cross_chain_proof(&validator2, &proof_id);
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

    let validator = setup_validator(&env, &client, &admin);

    env.mock_all_auths();

    let bad_id = BytesN::from_array(&env, &[0xffu8; 32]);
    let result = client.try_verify_cross_chain_proof(&validator, &bad_id);
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
    let valid =
        String::from_str(&env, "GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWNA");
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

    let validator = setup_validator(&env, &client, &admin);

    env.mock_all_auths();

    let payload_hash = BytesN::from_array(&env, &[0x55u8; 32]);
    let event_id = client.sync_cross_chain_event(
        &validator,
        &ChainId::Ethereum,
        &ChainId::Stellar,
        &CrossChainEventType::RecordCreated,
        &payload_hash,
        &99999,
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

    let validator = setup_validator(&env, &client, &admin);

    env.mock_all_auths();

    let payload_hash = BytesN::from_array(&env, &[0x66u8; 32]);
    let event_id = client.sync_cross_chain_event(
        &validator,
        &ChainId::Ethereum,
        &ChainId::Stellar,
        &CrossChainEventType::AccessGranted,
        &payload_hash,
        &200,
    );

    client.process_sync_event(&validator, &event_id, &EventSyncStatus::Synced);

    let event = client.get_sync_event(&event_id).unwrap();
    assert_eq!(event.sync_status, EventSyncStatus::Synced);
}

#[test]
fn test_multiple_events_unique_ids() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let validator = setup_validator(&env, &client, &admin);

    env.mock_all_auths();

    let hash1 = BytesN::from_array(&env, &[0x11u8; 32]);
    let hash2 = BytesN::from_array(&env, &[0x22u8; 32]);

    let id1 = client.sync_cross_chain_event(
        &validator,
        &ChainId::Ethereum,
        &ChainId::Stellar,
        &CrossChainEventType::RecordCreated,
        &hash1,
        &100,
    );

    let id2 = client.sync_cross_chain_event(
        &validator,
        &ChainId::Polygon,
        &ChainId::Stellar,
        &CrossChainEventType::AccessRevoked,
        &hash2,
        &200,
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

    let validator = setup_validator(&env, &client, &admin);
    let recipient = Address::generate(&env);

    env.mock_all_auths();

    // Submit a message
    let message_id = generate_message_id(&env);
    client.submit_message(
        &validator,
        &message_id,
        &ChainId::Ethereum,
        &ChainId::Stellar,
        &String::from_str(&env, "0x1234"),
        &recipient,
        &MessageType::RecordRequest,
        &String::from_str(&env, "{}"),
        &1,
        &generate_signature(&env),
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

    let validator = setup_validator(&env, &client, &admin);

    env.mock_all_auths();
    client.pause(&admin);

    let result = client.try_submit_message(
        &validator,
        &generate_message_id(&env),
        &ChainId::Ethereum,
        &ChainId::Stellar,
        &String::from_str(&env, "0x1234567890abcdef"),
        &Address::generate(&env),
        &MessageType::RecordRequest,
        &String::from_str(&env, "{\"record_id\": 1}"),
        &1,
        &generate_signature(&env),
    );

    assert_eq!(result, Err(Ok(Error::ContractPaused)));
}

// ==================== Nonce Tests ====================

#[test]
fn test_nonce_replay_protection() {
    let env = Env::default();
    let (client, admin, medical, identity, access) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &medical, &identity, &access);

    let validator = setup_validator(&env, &client, &admin);
    let recipient = Address::generate(&env);

    env.mock_all_auths();

    let sender = String::from_str(&env, "0x1234567890abcdef");

    client.submit_message(
        &validator,
        &BytesN::from_array(&env, &[1u8; 32]),
        &ChainId::Ethereum,
        &ChainId::Stellar,
        &sender,
        &recipient,
        &MessageType::RecordRequest,
        &String::from_str(&env, "{}"),
        &1,
        &generate_signature(&env),
    );

    // Same nonce should fail
    let result = client.try_submit_message(
        &validator,
        &BytesN::from_array(&env, &[4u8; 32]),
        &ChainId::Ethereum,
        &ChainId::Stellar,
        &sender,
        &recipient,
        &MessageType::RecordRequest,
        &String::from_str(&env, "{}"),
        &1,
        &generate_signature(&env),
    );

    assert_eq!(result, Err(Ok(Error::InvalidNonce)));
}
