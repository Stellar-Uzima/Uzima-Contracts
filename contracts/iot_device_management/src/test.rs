#![cfg(test)]

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{vec, Address, BytesN, Env, String};

fn setup(env: &Env) -> (IoTDeviceManagementClient, Address) {
    let contract_id = Address::generate(env);
    env.register_contract(&contract_id, IoTDeviceManagement);
    let client = IoTDeviceManagementClient::new(env, &contract_id);
    let admin = Address::generate(env);
    env.mock_all_auths();
    (client, admin)
}

fn make_bytes32(env: &Env, val: u8) -> BytesN<32> {
    let mut bytes = [0u8; 32];
    bytes[0] = val;
    BytesN::from_array(env, &bytes)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    // Calling initialize again should fail
    let result = client.try_initialize(&admin);
    assert_eq!(result, Err(Ok(IoTError::AlreadyInitialized)));
}

#[test]
fn test_pause_unpause() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    client.pause(&admin);
    // set_role should fail when paused
    let user = Address::generate(&env);
    let result = client.try_set_role(&admin, &user, &Role::Operator);
    assert_eq!(result, Err(Ok(IoTError::ContractPaused)));
    client.unpause(&admin);
    // Should work after unpause
    client.set_role(&admin, &user, &Role::Operator);
}

#[test]
fn test_pause_not_admin() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    let non_admin = Address::generate(&env);
    let result = client.try_pause(&non_admin);
    assert_eq!(result, Err(Ok(IoTError::NotAdmin)));
}

#[test]
fn test_set_role() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    let user = Address::generate(&env);
    client.set_role(&admin, &user, &Role::Operator);
    let role = client.get_role(&user);
    assert_eq!(role, Role::Operator);
}

fn register_manufacturer(
    env: &Env,
    client: &IoTDeviceManagementClient,
    admin: &Address,
    id_byte: u8,
) -> BytesN<32> {
    let mfr_id = make_bytes32(env, id_byte);
    let cert = make_bytes32(env, id_byte.wrapping_add(100));
    let name = String::from_str(env, "TestManufacturer");
    client.register_manufacturer(admin, &mfr_id, &name, &cert);
    mfr_id
}

#[test]
fn test_register_manufacturer() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    let mfr_id = register_manufacturer(&env, &client, &admin, 1);
    let mfr = client.get_manufacturer(&mfr_id);
    assert_eq!(mfr.is_active, true);
    assert_eq!(mfr.device_count, 0);
}

#[test]
fn test_register_manufacturer_duplicate() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    let mfr_id = register_manufacturer(&env, &client, &admin, 1);
    let cert = make_bytes32(&env, 200);
    let name = String::from_str(&env, "Dup");
    let result = client.try_register_manufacturer(&admin, &mfr_id, &name, &cert);
    assert_eq!(result, Err(Ok(IoTError::ManufacturerAlreadyRegistered)));
}

#[test]
fn test_deactivate_manufacturer() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    let mfr_id = register_manufacturer(&env, &client, &admin, 1);
    client.deactivate_manufacturer(&admin, &mfr_id);
    let mfr = client.get_manufacturer(&mfr_id);
    assert_eq!(mfr.is_active, false);
}

fn register_device(
    env: &Env,
    client: &IoTDeviceManagementClient,
    operator: &Address,
    mfr_id: &BytesN<32>,
    device_byte: u8,
) -> BytesN<32> {
    let device_id = make_bytes32(env, device_byte);
    let model = String::from_str(env, "Model-X100");
    let serial = String::from_str(env, "SN-00001");
    let location = String::from_str(env, "Ward A, Room 101");
    let enc_key = make_bytes32(env, device_byte.wrapping_add(50));
    let metadata = String::from_str(env, "ipfs://Qm...");
    client.register_device(
        operator,
        &device_id,
        mfr_id,
        &DeviceType::VitalSignsMonitor,
        &model,
        &serial,
        &location,
        &enc_key,
        &metadata,
    );
    device_id
}

#[test]
fn test_register_device() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    let operator = Address::generate(&env);
    client.set_role(&admin, &operator, &Role::Operator);
    let mfr_id = register_manufacturer(&env, &client, &admin, 1);

    let device_id = register_device(&env, &client, &operator, &mfr_id, 10);
    let device = client.get_device(&device_id);
    assert_eq!(device.status, DeviceStatus::Provisioning);
    assert_eq!(device.device_type, DeviceType::VitalSignsMonitor);
    assert_eq!(device.operator, operator);
}

#[test]
fn test_register_device_duplicate() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    let operator = Address::generate(&env);
    client.set_role(&admin, &operator, &Role::Operator);
    let mfr_id = register_manufacturer(&env, &client, &admin, 1);
    let device_id = register_device(&env, &client, &operator, &mfr_id, 10);

    let model = String::from_str(&env, "M");
    let serial = String::from_str(&env, "S");
    let location = String::from_str(&env, "L");
    let enc = make_bytes32(&env, 99);
    let meta = String::from_str(&env, "x");
    let result = client.try_register_device(
        &operator, &device_id, &mfr_id,
        &DeviceType::VitalSignsMonitor, &model, &serial, &location, &enc, &meta,
    );
    assert_eq!(result, Err(Ok(IoTError::DeviceAlreadyRegistered)));
}

#[test]
fn test_activate_device() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    let operator = Address::generate(&env);
    client.set_role(&admin, &operator, &Role::Operator);
    let mfr_id = register_manufacturer(&env, &client, &admin, 1);
    let device_id = register_device(&env, &client, &operator, &mfr_id, 10);

    client.activate_device(&operator, &device_id);
    let device = client.get_device(&device_id);
    assert_eq!(device.status, DeviceStatus::Active);
}

#[test]
fn test_suspend_and_reactivate_device() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    let operator = Address::generate(&env);
    client.set_role(&admin, &operator, &Role::Operator);
    let mfr_id = register_manufacturer(&env, &client, &admin, 1);
    let device_id = register_device(&env, &client, &operator, &mfr_id, 10);
    client.activate_device(&operator, &device_id);

    client.suspend_device(&operator, &device_id);
    let device = client.get_device(&device_id);
    assert_eq!(device.status, DeviceStatus::Suspended);

    client.activate_device(&operator, &device_id);
    let device = client.get_device(&device_id);
    assert_eq!(device.status, DeviceStatus::Active);
}

#[test]
fn test_decommission_device() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    let operator = Address::generate(&env);
    client.set_role(&admin, &operator, &Role::Operator);
    let mfr_id = register_manufacturer(&env, &client, &admin, 1);
    let device_id = register_device(&env, &client, &operator, &mfr_id, 10);

    client.decommission_device(&admin, &device_id);
    let device = client.get_device(&device_id);
    assert_eq!(device.status, DeviceStatus::Decommissioned);
}

#[test]
fn test_get_device_count() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    let operator = Address::generate(&env);
    client.set_role(&admin, &operator, &Role::Operator);
    let mfr_id = register_manufacturer(&env, &client, &admin, 1);
    register_device(&env, &client, &operator, &mfr_id, 10);
    register_device(&env, &client, &operator, &mfr_id, 11);

    assert_eq!(client.get_device_count(), 2);
}

#[test]
fn test_publish_firmware() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    let mfr_id = register_manufacturer(&env, &client, &admin, 1);

    let binary_hash = make_bytes32(&env, 200);
    let notes = String::from_str(&env, "ipfs://release-notes");
    client.publish_firmware(
        &admin, &mfr_id, &1u32, &DeviceType::VitalSignsMonitor,
        &binary_hash, &notes, &0u32, &1024u64,
    );

    let fw = client.get_firmware(&mfr_id, &1u32);
    assert_eq!(fw.status, FirmwareStatus::Pending);
    assert_eq!(fw.size_bytes, 1024);
}

#[test]
fn test_approve_firmware() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    let mfr_id = register_manufacturer(&env, &client, &admin, 1);

    let binary_hash = make_bytes32(&env, 200);
    let notes = String::from_str(&env, "notes");
    client.publish_firmware(
        &admin, &mfr_id, &1u32, &DeviceType::VitalSignsMonitor,
        &binary_hash, &notes, &0u32, &1024u64,
    );
    client.approve_firmware(&admin, &mfr_id, &1u32);
    let fw = client.get_firmware(&mfr_id, &1u32);
    assert_eq!(fw.status, FirmwareStatus::Approved);
}

#[test]
fn test_update_device_firmware() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    let operator = Address::generate(&env);
    client.set_role(&admin, &operator, &Role::Operator);
    let mfr_id = register_manufacturer(&env, &client, &admin, 1);
    let device_id = register_device(&env, &client, &operator, &mfr_id, 10);
    client.activate_device(&operator, &device_id);

    // Publish and approve firmware v1
    let binary_hash = make_bytes32(&env, 200);
    let notes = String::from_str(&env, "v1");
    client.publish_firmware(
        &admin, &mfr_id, &1u32, &DeviceType::VitalSignsMonitor,
        &binary_hash, &notes, &0u32, &1024u64,
    );
    client.approve_firmware(&admin, &mfr_id, &1u32);

    // Update device
    client.update_device_firmware(&operator, &device_id, &1u32);
    let device = client.get_device(&device_id);
    assert_eq!(device.firmware_version, 1);
}

#[test]
fn test_firmware_downgrade_not_allowed() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    let operator = Address::generate(&env);
    client.set_role(&admin, &operator, &Role::Operator);
    let mfr_id = register_manufacturer(&env, &client, &admin, 1);
    let device_id = register_device(&env, &client, &operator, &mfr_id, 10);
    client.activate_device(&operator, &device_id);

    // Publish and approve v1 and v2
    let hash1 = make_bytes32(&env, 200);
    let hash2 = make_bytes32(&env, 201);
    let notes = String::from_str(&env, "notes");
    client.publish_firmware(&admin, &mfr_id, &1u32, &DeviceType::VitalSignsMonitor, &hash1, &notes, &0u32, &512u64);
    client.approve_firmware(&admin, &mfr_id, &1u32);
    client.publish_firmware(&admin, &mfr_id, &2u32, &DeviceType::VitalSignsMonitor, &hash2, &notes, &1u32, &1024u64);
    client.approve_firmware(&admin, &mfr_id, &2u32);

    // Update to v2
    client.update_device_firmware(&operator, &device_id, &2u32);

    // Try to downgrade to v1
    let result = client.try_update_device_firmware(&operator, &device_id, &1u32);
    assert_eq!(result, Err(Ok(IoTError::DowngradeNotAllowed)));
}

#[test]
fn test_submit_heartbeat() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    let operator = Address::generate(&env);
    client.set_role(&admin, &operator, &Role::Operator);
    let mfr_id = register_manufacturer(&env, &client, &admin, 1);
    let device_id = register_device(&env, &client, &operator, &mfr_id, 10);
    client.activate_device(&operator, &device_id);

    // Advance ledger time
    env.ledger().with_mut(|li| li.timestamp = 1000);

    let metrics_ref = String::from_str(&env, "ipfs://metrics-001");
    client.submit_heartbeat(
        &operator, &device_id, &HealthStatus::Healthy,
        &95u32, &80u32, &0u32, &metrics_ref,
    );

    let device = client.get_device(&device_id);
    assert_eq!(device.last_heartbeat, 1000);
    assert_eq!(device.health_status, HealthStatus::Healthy);
}

#[test]
fn test_heartbeat_too_frequent() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    let operator = Address::generate(&env);
    client.set_role(&admin, &operator, &Role::Operator);
    let mfr_id = register_manufacturer(&env, &client, &admin, 1);
    let device_id = register_device(&env, &client, &operator, &mfr_id, 10);
    client.activate_device(&operator, &device_id);

    env.ledger().with_mut(|li| li.timestamp = 1000);
    let metrics_ref = String::from_str(&env, "m");
    client.submit_heartbeat(&operator, &device_id, &HealthStatus::Healthy, &95u32, &80u32, &0u32, &metrics_ref);

    // Try again too soon (within 60s default interval)
    env.ledger().with_mut(|li| li.timestamp = 1030);
    let result = client.try_submit_heartbeat(&operator, &device_id, &HealthStatus::Healthy, &95u32, &80u32, &0u32, &metrics_ref);
    assert_eq!(result, Err(Ok(IoTError::HeartbeatTooFrequent)));
}

#[test]
fn test_get_device_uptime() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    let operator = Address::generate(&env);
    client.set_role(&admin, &operator, &Role::Operator);
    let mfr_id = register_manufacturer(&env, &client, &admin, 1);
    let device_id = register_device(&env, &client, &operator, &mfr_id, 10);

    env.ledger().with_mut(|li| li.timestamp = 1000);
    client.activate_device(&operator, &device_id);

    // Check uptime at t=2000 (1000s uptime)
    env.ledger().with_mut(|li| li.timestamp = 2000);
    let uptime_bps = client.get_device_uptime_bps(&device_id);
    // 1000s uptime, 0s downtime => 10000 bps (100%)
    assert_eq!(uptime_bps, 10000);
}

#[test]
fn test_get_active_device_count() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    let operator = Address::generate(&env);
    client.set_role(&admin, &operator, &Role::Operator);
    let mfr_id = register_manufacturer(&env, &client, &admin, 1);

    let d1 = register_device(&env, &client, &operator, &mfr_id, 10);
    let d2 = register_device(&env, &client, &operator, &mfr_id, 11);
    client.activate_device(&operator, &d1);
    client.activate_device(&operator, &d2);

    assert_eq!(client.get_active_device_count(), 2);

    client.suspend_device(&operator, &d1);
    assert_eq!(client.get_active_device_count(), 1);
}
