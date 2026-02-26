#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
use super::*;
use crate::SwapStatus;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String, Vec};

fn create_contract(
    env: &Env,
) -> (
    CrossChainAccessContractClient<'_>,
    Address,
    Address,
    Address,
) {
    let contract_id = Address::generate(env);
    env.register_contract(&contract_id, CrossChainAccessContract);
    let client = CrossChainAccessContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let bridge_contract = Address::generate(&env);
    let identity_contract = Address::generate(&env);
    (client, admin, bridge_contract, identity_contract)
}

fn initialize_contract(
    env: &Env,
    client: &CrossChainAccessContractClient,
    admin: &Address,
    bridge: &Address,
    identity: &Address,
) {
    env.mock_all_auths();
    client.initialize(admin, bridge, identity);
}

fn generate_ip_hash(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[1u8; 32])
}

// ==================== Initialization Tests ====================

#[test]
fn test_initialize() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);

    initialize_contract(&env, &client, &admin, &bridge, &identity);

    assert!(!client.is_paused());
}

#[test]
fn test_initialize_twice_fails() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);

    env.mock_all_auths();
    client.initialize(&admin, &bridge, &identity);

    let result = client.try_initialize(&admin, &bridge, &identity);
    assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
}

// ==================== Access Grant Tests ====================

#[test]
fn test_grant_access() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let grantee_address = String::from_str(&env, "0x1234567890abcdef1234567890abcdef12345678");

    env.mock_all_auths();

    let grant_id = client.grant_access(
        &patient,
        &ChainId::Ethereum,
        &grantee_address,
        &PermissionLevel::Read,
        &AccessScope::AllRecords,
        &2592000, // 30 days
        &Vec::new(&env),
    );

    assert!(grant_id > 0);

    let grant = client.get_grant(&grant_id).unwrap();
    assert!(grant.grantor == patient);
    assert!(grant.grantee_chain == ChainId::Ethereum);
    assert!(grant.permission_level == PermissionLevel::Read);
    assert!(grant.is_active);
}

#[test]
fn test_grant_access_with_specific_records() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let grantee_address = String::from_str(&env, "0x1234567890abcdef");

    env.mock_all_auths();

    let record_ids = soroban_sdk::vec![&env, 1u64, 2u64, 3u64];
    let scope = AccessScope::SpecificRecords(record_ids);

    let grant_id = client.grant_access(
        &patient,
        &ChainId::Polygon,
        &grantee_address,
        &PermissionLevel::ReadConfidential,
        &scope,
        &86400, // 1 day
        &Vec::new(&env),
    );

    let grant = client.get_grant(&grant_id).unwrap();
    assert!(grant.permission_level == PermissionLevel::ReadConfidential);
}

#[test]
fn test_grant_access_with_conditions() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let grantee_address = String::from_str(&env, "0x1234567890abcdef");

    env.mock_all_auths();

    let conditions = soroban_sdk::vec![
        &env,
        AccessCondition::AuditRequired,
        AccessCondition::EmergencyOnly
    ];

    let grant_id = client.grant_access(
        &patient,
        &ChainId::Ethereum,
        &grantee_address,
        &PermissionLevel::Read,
        &AccessScope::AllRecords,
        &86400,
        &conditions,
    );

    let grant = client.get_grant(&grant_id).unwrap();
    assert_eq!(grant.conditions.len(), 2);
}

#[test]
fn test_revoke_access() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let grantee_address = String::from_str(&env, "0x1234567890abcdef");

    env.mock_all_auths();

    let grant_id = client.grant_access(
        &patient,
        &ChainId::Ethereum,
        &grantee_address,
        &PermissionLevel::Read,
        &AccessScope::AllRecords,
        &86400,
        &Vec::new(&env),
    );

    // Revoke access
    let result = client.revoke_access(&patient, &grant_id);
    assert!(result);

    let grant = client.get_grant(&grant_id).unwrap();
    assert!(!grant.is_active);
}

#[test]
fn test_revoke_access_not_authorized() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let other_user = Address::generate(&env);
    let grantee_address = String::from_str(&env, "0x1234567890abcdef");

    env.mock_all_auths();

    let grant_id = client.grant_access(
        &patient,
        &ChainId::Ethereum,
        &grantee_address,
        &PermissionLevel::Read,
        &AccessScope::AllRecords,
        &86400,
        &Vec::new(&env),
    );

    // Try to revoke by non-authorized user
    let result = client.try_revoke_access(&other_user, &grant_id);
    assert_eq!(result, Err(Ok(Error::NotAuthorized)));
}

#[test]
fn test_extend_grant() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let grantee_address = String::from_str(&env, "0x1234567890abcdef");

    env.mock_all_auths();

    let grant_id = client.grant_access(
        &patient,
        &ChainId::Ethereum,
        &grantee_address,
        &PermissionLevel::Read,
        &AccessScope::AllRecords,
        &86400, // 1 day
        &Vec::new(&env),
    );

    let original_grant = client.get_grant(&grant_id).unwrap();
    let original_expiry = original_grant.expires_at;

    // Extend by 1 more day
    client.extend_grant(&patient, &grant_id, &86400);

    let updated_grant = client.get_grant(&grant_id).unwrap();
    assert_eq!(updated_grant.expires_at, original_expiry + 86400);
}

#[test]
fn test_update_grant_conditions() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let grantee_address = String::from_str(&env, "0x1234567890abcdef");

    env.mock_all_auths();

    let grant_id = client.grant_access(
        &patient,
        &ChainId::Ethereum,
        &grantee_address,
        &PermissionLevel::Read,
        &AccessScope::AllRecords,
        &86400,
        &Vec::new(&env),
    );

    let new_conditions = soroban_sdk::vec![&env, AccessCondition::SingleUse];
    client.update_grant_conditions(&patient, &grant_id, &new_conditions);

    let grant = client.get_grant(&grant_id).unwrap();
    assert_eq!(grant.conditions.len(), 1);
}

// ==================== Access Request Tests ====================

#[test]
fn test_request_access() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let requester_address = String::from_str(&env, "0xabcdef1234567890");
    let requested_records = soroban_sdk::vec![&env, 1u64, 2u64];
    let purpose = String::from_str(&env, "Medical consultation");

    env.mock_all_auths();

    let request_id = client.request_access(
        &ChainId::Ethereum,
        &requester_address,
        &patient,
        &requested_records,
        &purpose,
        &false, // not emergency
    );

    assert!(request_id > 0);

    let request = client.get_request(&request_id).unwrap();
    assert_eq!(request.patient, patient);
    assert!(request.requester_chain == ChainId::Ethereum);
    assert!(request.status == RequestStatus::Pending);
    assert!(!request.is_emergency);
}

#[test]
fn test_request_access_emergency() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let requester_address = String::from_str(&env, "0xabcdef1234567890");
    let requested_records = soroban_sdk::vec![&env, 1u64];
    let purpose = String::from_str(&env, "Emergency treatment");

    env.mock_all_auths();

    let request_id = client.request_access(
        &ChainId::Polygon,
        &requester_address,
        &patient,
        &requested_records,
        &purpose,
        &true, // emergency
    );

    let request = client.get_request(&request_id).unwrap();
    assert!(request.is_emergency);
}

#[test]
fn test_process_request_approve() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let requester_address = String::from_str(&env, "0xabcdef1234567890");
    let requested_records = soroban_sdk::vec![&env, 1u64];
    let purpose = String::from_str(&env, "Medical consultation");

    env.mock_all_auths();

    let request_id = client.request_access(
        &ChainId::Ethereum,
        &requester_address,
        &patient,
        &requested_records,
        &purpose,
        &false,
    );

    // Patient approves
    let result = client.process_request(&patient, &request_id, &true);
    assert!(result);

    let request = client.get_request(&request_id).unwrap();
    assert!(request.status == RequestStatus::Approved);
    assert!(request.decision_by.is_some());
}

#[test]
fn test_process_request_reject() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let requester_address = String::from_str(&env, "0xabcdef1234567890");
    let requested_records = soroban_sdk::vec![&env, 1u64];
    let purpose = String::from_str(&env, "Unknown purpose");

    env.mock_all_auths();

    let request_id = client.request_access(
        &ChainId::Ethereum,
        &requester_address,
        &patient,
        &requested_records,
        &purpose,
        &false,
    );

    // Patient rejects
    client.process_request(&patient, &request_id, &false);

    let request = client.get_request(&request_id).unwrap();
    assert!(request.status == RequestStatus::Rejected);
}

#[test]
fn test_process_request_not_authorized() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let other_user = Address::generate(&env);
    let requester_address = String::from_str(&env, "0xabcdef1234567890");
    let requested_records = soroban_sdk::vec![&env, 1u64];
    let purpose = String::from_str(&env, "Medical consultation");

    env.mock_all_auths();

    let request_id = client.request_access(
        &ChainId::Ethereum,
        &requester_address,
        &patient,
        &requested_records,
        &purpose,
        &false,
    );

    // Non-authorized user tries to process
    let result = client.try_process_request(&other_user, &request_id, &true);
    assert_eq!(result, Err(Ok(Error::NotAuthorized)));
}

// ==================== Delegation Tests ====================

#[test]
fn test_create_delegation() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let delegate = Address::generate(&env);

    env.mock_all_auths();

    let result = client.create_delegation(
        &patient,
        &delegate,
        &ChainId::Stellar,
        &String::from_str(&env, ""),
        &true,    // can_grant
        &true,    // can_revoke
        &false,   // can_manage_emergency
        &2592000, // 30 days
    );

    assert!(result);

    let delegation = client.get_delegation(&patient, &delegate).unwrap();
    assert_eq!(delegation.delegator, patient);
    assert_eq!(delegation.delegate, delegate);
    assert!(delegation.can_grant);
    assert!(delegation.can_revoke);
    assert!(!delegation.can_manage_emergency);
    assert!(delegation.is_active);
}

#[test]
fn test_create_delegation_with_chain() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let delegate = Address::generate(&env);
    let delegate_external_address = String::from_str(&env, "0x1234567890abcdef");

    env.mock_all_auths();

    client.create_delegation(
        &patient,
        &delegate,
        &ChainId::Ethereum,
        &delegate_external_address,
        &true,
        &false,
        &true,
        &86400,
    );

    let delegation = client.get_delegation(&patient, &delegate).unwrap();
    assert_eq!(delegation.delegate_chain, ChainId::Ethereum);
    assert_eq!(delegation.delegate_address, delegate_external_address);
}

#[test]
fn test_revoke_delegation() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let delegate = Address::generate(&env);

    env.mock_all_auths();

    client.create_delegation(
        &patient,
        &delegate,
        &ChainId::Stellar,
        &String::from_str(&env, ""),
        &true,
        &true,
        &false,
        &86400,
    );

    let result = client.revoke_delegation(&patient, &delegate);
    assert!(result);

    let delegation = client.get_delegation(&patient, &delegate).unwrap();
    assert!(!delegation.is_active);
}

#[test]
fn test_revoke_delegation_not_found() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let delegate = Address::generate(&env);

    env.mock_all_auths();

    let result = client.try_revoke_delegation(&patient, &delegate);
    assert_eq!(result, Err(Ok(Error::DelegationNotFound)));
}

// ==================== Emergency Access Tests ====================

#[test]
fn test_configure_emergency() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let trusted_provider = String::from_str(&env, "0xhospital123");
    let trusted_providers = soroban_sdk::vec![&env, trusted_provider];

    env.mock_all_auths();

    let result = client.configure_emergency(
        &patient,
        &true, // enabled
        &3600, // 1 hour auto-approve duration
        &2,    // required attestations
        &trusted_providers,
    );

    assert!(result);

    let config = client.get_emergency_config(&patient).unwrap();
    assert!(config.is_enabled);
    assert_eq!(config.auto_approve_duration, 3600);
    assert_eq!(config.required_attestations, 2);
    assert_eq!(config.trusted_providers.len(), 1);
}

#[test]
fn test_configure_emergency_disabled() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient = Address::generate(&env);

    env.mock_all_auths();

    client.configure_emergency(
        &patient,
        &false, // disabled
        &0,
        &0,
        &Vec::new(&env),
    );

    let config = client.get_emergency_config(&patient).unwrap();
    assert!(!config.is_enabled);
}

// ==================== Audit Log Tests ====================

#[test]
fn test_log_access() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let accessor_address = String::from_str(&env, "0xdoctor123");
    let ip_hash = generate_ip_hash(&env);

    env.mock_all_auths();

    let entry_id = client.log_access(
        &ChainId::Ethereum,
        &accessor_address,
        &patient,
        &1, // record_id
        &AccessAction::View,
        &ip_hash,
        &true, // success
    );

    assert!(entry_id > 0);

    let entry = client.get_audit_entry(&entry_id).unwrap();
    assert_eq!(entry.patient, patient);
    assert_eq!(entry.record_id, 1);
    assert!(entry.action == AccessAction::View);
    assert!(entry.success);
}

#[test]
fn test_log_access_failed() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let accessor_address = String::from_str(&env, "0xunauthorized");
    let ip_hash = generate_ip_hash(&env);

    env.mock_all_auths();

    let entry_id = client.log_access(
        &ChainId::Polygon,
        &accessor_address,
        &patient,
        &5,
        &AccessAction::Download,
        &ip_hash,
        &false, // failed access
    );

    let entry = client.get_audit_entry(&entry_id).unwrap();
    assert!(!entry.success);
    assert!(entry.action == AccessAction::Download);
}

#[test]
fn test_log_emergency_access() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let accessor_address = String::from_str(&env, "0xemergency_responder");
    let ip_hash = generate_ip_hash(&env);

    env.mock_all_auths();

    let entry_id = client.log_access(
        &ChainId::Stellar,
        &accessor_address,
        &patient,
        &10,
        &AccessAction::EmergencyAccess,
        &ip_hash,
        &true,
    );

    let entry = client.get_audit_entry(&entry_id).unwrap();
    assert!(entry.action == AccessAction::EmergencyAccess);
}

// ==================== Access Verification Tests ====================

#[test]
fn test_verify_access_granted() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let grantee_address = String::from_str(&env, "0xdoctor123");

    env.mock_all_auths();

    // Grant access
    client.grant_access(
        &patient,
        &ChainId::Ethereum,
        &grantee_address,
        &PermissionLevel::Read,
        &AccessScope::AllRecords,
        &86400,
        &Vec::new(&env),
    );

    // Verify access
    let has_access = client.verify_access(
        &ChainId::Ethereum,
        &grantee_address,
        &patient,
        &1, // record_id
        &PermissionLevel::Read,
    );

    assert!(has_access);
}

#[test]
fn test_verify_access_denied_no_grant() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let grantee_address = String::from_str(&env, "0xunauthorized");

    env.mock_all_auths();

    // No grant exists
    let has_access = client.verify_access(
        &ChainId::Ethereum,
        &grantee_address,
        &patient,
        &1,
        &PermissionLevel::Read,
    );

    assert!(!has_access);
}

#[test]
fn test_verify_access_insufficient_permission() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let grantee_address = String::from_str(&env, "0xdoctor123");

    env.mock_all_auths();

    // Grant Read access
    client.grant_access(
        &patient,
        &ChainId::Ethereum,
        &grantee_address,
        &PermissionLevel::Read,
        &AccessScope::AllRecords,
        &86400,
        &Vec::new(&env),
    );

    // Try to verify Write permission (should fail)
    let has_access = client.verify_access(
        &ChainId::Ethereum,
        &grantee_address,
        &patient,
        &1,
        &PermissionLevel::Write, // requesting higher permission
    );

    assert!(!has_access);
}

#[test]
fn test_verify_access_specific_records() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let grantee_address = String::from_str(&env, "0xdoctor123");

    env.mock_all_auths();

    // Grant access to specific records (1, 2, 3)
    let record_ids = soroban_sdk::vec![&env, 1u64, 2u64, 3u64];
    client.grant_access(
        &patient,
        &ChainId::Ethereum,
        &grantee_address,
        &PermissionLevel::Read,
        &AccessScope::SpecificRecords(record_ids),
        &86400,
        &Vec::new(&env),
    );

    // Should have access to record 2
    let has_access = client.verify_access(
        &ChainId::Ethereum,
        &grantee_address,
        &patient,
        &2,
        &PermissionLevel::Read,
    );
    assert!(has_access);

    // Should NOT have access to record 5
    let has_access = client.verify_access(
        &ChainId::Ethereum,
        &grantee_address,
        &patient,
        &5,
        &PermissionLevel::Read,
    );
    assert!(!has_access);
}

// ==================== Pause Tests ====================

#[test]
fn test_pause_unpause() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

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
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let grantee_address = String::from_str(&env, "0xdoctor123");

    env.mock_all_auths();
    client.pause(&admin);

    let result = client.try_grant_access(
        &patient,
        &ChainId::Ethereum,
        &grantee_address,
        &PermissionLevel::Read,
        &AccessScope::AllRecords,
        &86400,
        &Vec::new(&env),
    );

    assert_eq!(result, Err(Ok(Error::ContractPaused)));
}

#[test]
fn test_pause_not_admin() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let non_admin = Address::generate(&env);

    env.mock_all_auths();

    let result = client.try_pause(&non_admin);
    assert_eq!(result, Err(Ok(Error::NotAuthorized)));
}

// ==================== Permission Level Tests ====================

#[test]
fn test_admin_permission_covers_all() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let grantee_address = String::from_str(&env, "0xadmin_user");

    env.mock_all_auths();

    // Grant Admin permission
    client.grant_access(
        &patient,
        &ChainId::Ethereum,
        &grantee_address,
        &PermissionLevel::Admin,
        &AccessScope::AllRecords,
        &86400,
        &Vec::new(&env),
    );

    // Admin should have access to all permission levels
    assert!(client.verify_access(
        &ChainId::Ethereum,
        &grantee_address,
        &patient,
        &1,
        &PermissionLevel::Read,
    ));
    assert!(client.verify_access(
        &ChainId::Ethereum,
        &grantee_address,
        &patient,
        &1,
        &PermissionLevel::ReadConfidential,
    ));
    assert!(client.verify_access(
        &ChainId::Ethereum,
        &grantee_address,
        &patient,
        &1,
        &PermissionLevel::Write,
    ));
    assert!(client.verify_access(
        &ChainId::Ethereum,
        &grantee_address,
        &patient,
        &1,
        &PermissionLevel::Admin,
    ));
}

// ==================== Chain ID Tests ====================

#[test]
fn test_different_chains() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let grantee_address = String::from_str(&env, "0xdoctor123");

    env.mock_all_auths();

    // Grant access on Ethereum
    client.grant_access(
        &patient,
        &ChainId::Ethereum,
        &grantee_address,
        &PermissionLevel::Read,
        &AccessScope::AllRecords,
        &86400,
        &Vec::new(&env),
    );

    // Should have access on Ethereum
    assert!(client.verify_access(
        &ChainId::Ethereum,
        &grantee_address,
        &patient,
        &1,
        &PermissionLevel::Read,
    ));

    // Should NOT have access on Polygon (different chain)
    assert!(!client.verify_access(
        &ChainId::Polygon,
        &grantee_address,
        &patient,
        &1,
        &PermissionLevel::Read,
    ));
}

#[test]
fn test_custom_chain_id() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient = Address::generate(&env);
    let grantee_address = String::from_str(&env, "0xdoctor123");

    env.mock_all_auths();

    // Grant access on custom chain
    client.grant_access(
        &patient,
        &ChainId::Custom(999),
        &grantee_address,
        &PermissionLevel::Read,
        &AccessScope::AllRecords,
        &86400,
        &Vec::new(&env),
    );

    let has_access = client.verify_access(
        &ChainId::Custom(999),
        &grantee_address,
        &patient,
        &1,
        &PermissionLevel::Read,
    );

    assert!(has_access);
}

// ==================== Storage Key Uniqueness Regression Tests ====================

/// Regression test: multiple delegations must be independent (was "deleg_key" collision)
#[test]
fn test_multiple_delegations_independent() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient1 = Address::generate(&env);
    let patient2 = Address::generate(&env);
    let delegate1 = Address::generate(&env);
    let delegate2 = Address::generate(&env);

    env.mock_all_auths();

    // patient1 delegates to delegate1 (can_grant=true, can_revoke=false)
    client.create_delegation(
        &patient1,
        &delegate1,
        &ChainId::Stellar,
        &String::from_str(&env, ""),
        &true,
        &false,
        &false,
        &86400,
    );

    // patient2 delegates to delegate2 (can_grant=false, can_revoke=true)
    client.create_delegation(
        &patient2,
        &delegate2,
        &ChainId::Stellar,
        &String::from_str(&env, ""),
        &false,
        &true,
        &false,
        &86400,
    );

    let d1 = client.get_delegation(&patient1, &delegate1).unwrap();
    let d2 = client.get_delegation(&patient2, &delegate2).unwrap();

    // Verify each delegation has its own settings
    assert!(d1.can_grant);
    assert!(!d1.can_revoke);
    assert!(!d2.can_grant);
    assert!(d2.can_revoke);

    // Cross-lookup should return None
    assert!(client.get_delegation(&patient1, &delegate2).is_none());
    assert!(client.get_delegation(&patient2, &delegate1).is_none());
}

/// Regression test: multiple patients can each have their own emergency config
#[test]
fn test_emergency_configs_independent_per_patient() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let patient1 = Address::generate(&env);
    let patient2 = Address::generate(&env);

    env.mock_all_auths();

    // patient1: emergency enabled
    client.configure_emergency(&patient1, &true, &3600, &2, &Vec::new(&env));

    // patient2: emergency disabled
    client.configure_emergency(&patient2, &false, &0, &0, &Vec::new(&env));

    let config1 = client.get_emergency_config(&patient1).unwrap();
    let config2 = client.get_emergency_config(&patient2).unwrap();

    assert!(config1.is_enabled);
    assert_eq!(config1.auto_approve_duration, 3600);

    assert!(!config2.is_enabled);
    assert_eq!(config2.auto_approve_duration, 0);
}

// ==================== Atomic Access Swap Tests ====================

#[test]
fn test_initiate_access_swap() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let initiator = Address::generate(&env);
    let grantee_address = String::from_str(&env, "0xdoctor123");

    env.mock_all_auths();

    // Create a grant for the initiator to offer
    let grant_id = client.grant_access(
        &initiator,
        &ChainId::Ethereum,
        &grantee_address,
        &PermissionLevel::Read,
        &AccessScope::AllRecords,
        &86400,
        &Vec::new(&env),
    );

    let hash_lock = BytesN::from_array(&env, &[0x42u8; 32]);
    let counterpart = String::from_str(&env, "0xhospital456");

    let swap_id = client.initiate_access_swap(
        &initiator,
        &ChainId::Polygon,
        &counterpart,
        &grant_id,
        &PermissionLevel::Read,
        &AccessScope::AllRecords,
        &hash_lock,
        &7200, // 2 hour timelock
    );

    assert_eq!(swap_id, 1);

    let swap = client.get_swap(&swap_id).unwrap();
    assert_eq!(swap.initiator, initiator);
    assert_eq!(swap.status, SwapStatus::Proposed);
    assert_eq!(swap.offered_grant_id, grant_id);
    assert_eq!(swap.counterpart_chain, ChainId::Polygon);
}

#[test]
fn test_accept_access_swap() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let initiator = Address::generate(&env);
    let acceptor = Address::generate(&env);
    let grantee_addr = String::from_str(&env, "0xgrantee");

    env.mock_all_auths();

    // Create offered grant
    let offered_grant_id = client.grant_access(
        &initiator,
        &ChainId::Ethereum,
        &grantee_addr,
        &PermissionLevel::Read,
        &AccessScope::AllRecords,
        &86400,
        &Vec::new(&env),
    );

    // Create counterpart grant (acceptor offering)
    let acceptor_grant_id = client.grant_access(
        &acceptor,
        &ChainId::Polygon,
        &grantee_addr,
        &PermissionLevel::Read,
        &AccessScope::AllRecords,
        &86400,
        &Vec::new(&env),
    );

    let hash_lock = BytesN::from_array(&env, &[0x55u8; 32]);

    let swap_id = client.initiate_access_swap(
        &initiator,
        &ChainId::Polygon,
        &grantee_addr,
        &offered_grant_id,
        &PermissionLevel::Read,
        &AccessScope::AllRecords,
        &hash_lock,
        &7200,
    );

    let result = client.accept_access_swap(&acceptor, &swap_id, &acceptor_grant_id);
    assert!(result);

    let swap = client.get_swap(&swap_id).unwrap();
    assert_eq!(swap.status, SwapStatus::Accepted);
    assert_eq!(swap.accepted_grant_id, acceptor_grant_id);
}

#[test]
fn test_finalize_access_swap() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let initiator = Address::generate(&env);
    let acceptor = Address::generate(&env);
    let grantee_addr = String::from_str(&env, "0xgrantee");

    env.mock_all_auths();

    // The secret and its hash
    let secret = BytesN::from_array(&env, &[0x99u8; 32]);
    let secret_hash: BytesN<32> = env.crypto().sha256(&secret.clone().into()).into();

    let offered_grant_id = client.grant_access(
        &initiator,
        &ChainId::Ethereum,
        &grantee_addr,
        &PermissionLevel::Read,
        &AccessScope::AllRecords,
        &86400,
        &Vec::new(&env),
    );

    let acceptor_grant_id = client.grant_access(
        &acceptor,
        &ChainId::Polygon,
        &grantee_addr,
        &PermissionLevel::Read,
        &AccessScope::AllRecords,
        &86400,
        &Vec::new(&env),
    );

    let swap_id = client.initiate_access_swap(
        &initiator,
        &ChainId::Polygon,
        &grantee_addr,
        &offered_grant_id,
        &PermissionLevel::Read,
        &AccessScope::AllRecords,
        &secret_hash,
        &7200,
    );

    client.accept_access_swap(&acceptor, &swap_id, &acceptor_grant_id);

    let result = client.finalize_access_swap(&initiator, &swap_id, &secret);
    assert!(result);

    let swap = client.get_swap(&swap_id).unwrap();
    assert_eq!(swap.status, SwapStatus::Completed);
}

#[test]
fn test_finalize_swap_wrong_secret_fails() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let initiator = Address::generate(&env);
    let acceptor = Address::generate(&env);
    let grantee_addr = String::from_str(&env, "0xgrantee");

    env.mock_all_auths();

    let secret = BytesN::from_array(&env, &[0x99u8; 32]);
    let wrong_secret = BytesN::from_array(&env, &[0x00u8; 32]);
    let secret_hash: BytesN<32> = env.crypto().sha256(&secret.clone().into()).into();

    let offered_grant_id = client.grant_access(
        &initiator,
        &ChainId::Ethereum,
        &grantee_addr,
        &PermissionLevel::Read,
        &AccessScope::AllRecords,
        &86400,
        &Vec::new(&env),
    );

    let acceptor_grant_id = client.grant_access(
        &acceptor,
        &ChainId::Polygon,
        &grantee_addr,
        &PermissionLevel::Read,
        &AccessScope::AllRecords,
        &86400,
        &Vec::new(&env),
    );

    let swap_id = client.initiate_access_swap(
        &initiator,
        &ChainId::Polygon,
        &grantee_addr,
        &offered_grant_id,
        &PermissionLevel::Read,
        &AccessScope::AllRecords,
        &secret_hash,
        &7200,
    );

    client.accept_access_swap(&acceptor, &swap_id, &acceptor_grant_id);

    // Wrong secret should fail
    let result = client.try_finalize_access_swap(&initiator, &swap_id, &wrong_secret);
    assert_eq!(result, Err(Ok(Error::NotAuthorized)));
}

#[test]
fn test_cancel_swap() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let initiator = Address::generate(&env);
    let grantee_addr = String::from_str(&env, "0xgrantee");

    env.mock_all_auths();

    let offered_grant_id = client.grant_access(
        &initiator,
        &ChainId::Ethereum,
        &grantee_addr,
        &PermissionLevel::Read,
        &AccessScope::AllRecords,
        &86400,
        &Vec::new(&env),
    );

    let hash_lock = BytesN::from_array(&env, &[0x11u8; 32]);

    let swap_id = client.initiate_access_swap(
        &initiator,
        &ChainId::Polygon,
        &grantee_addr,
        &offered_grant_id,
        &PermissionLevel::Read,
        &AccessScope::AllRecords,
        &hash_lock,
        &7200,
    );

    let result = client.cancel_access_swap(&initiator, &swap_id);
    assert!(result);

    let swap = client.get_swap(&swap_id).unwrap();
    assert_eq!(swap.status, SwapStatus::Cancelled);
}

#[test]
fn test_swap_not_initiator_cannot_cancel() {
    let env = Env::default();
    let (client, admin, bridge, identity) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge, &identity);

    let initiator = Address::generate(&env);
    let other = Address::generate(&env);
    let grantee_addr = String::from_str(&env, "0xgrantee");

    env.mock_all_auths();

    let offered_grant_id = client.grant_access(
        &initiator,
        &ChainId::Ethereum,
        &grantee_addr,
        &PermissionLevel::Read,
        &AccessScope::AllRecords,
        &86400,
        &Vec::new(&env),
    );

    let swap_id = client.initiate_access_swap(
        &initiator,
        &ChainId::Polygon,
        &grantee_addr,
        &offered_grant_id,
        &PermissionLevel::Read,
        &AccessScope::AllRecords,
        &BytesN::from_array(&env, &[0x11u8; 32]),
        &7200,
    );

    // Non-initiator cannot cancel before timelock
    let result = client.try_cancel_access_swap(&other, &swap_id);
    assert_eq!(result, Err(Ok(Error::NotAuthorized)));
}
