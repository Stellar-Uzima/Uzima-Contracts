use soroban_sdk::{symbol_short, Address, BytesN, Env};

use crate::{DeviceStatus, DeviceType, FirmwareStatus, HealthStatus};

pub fn emit_initialized(env: &Env, admin: &Address) {
    env.events()
        .publish(("IoT", symbol_short!("init")), admin.clone());
}

pub fn emit_device_registered(
    env: &Env,
    device_id: &BytesN<32>,
    device_type: DeviceType,
    operator: &Address,
) {
    env.events().publish(
        ("IoT", symbol_short!("dev_reg")),
        (device_id.clone(), device_type as u32, operator.clone()),
    );
}

pub fn emit_device_status_changed(
    env: &Env,
    device_id: &BytesN<32>,
    old_status: DeviceStatus,
    new_status: DeviceStatus,
) {
    env.events().publish(
        ("IoT", symbol_short!("dev_sts")),
        (device_id.clone(), old_status as u32, new_status as u32),
    );
}

pub fn emit_firmware_published(
    env: &Env,
    manufacturer_id: &BytesN<32>,
    version: u32,
    device_type: DeviceType,
) {
    env.events().publish(
        ("IoT", symbol_short!("fw_pub")),
        (manufacturer_id.clone(), version, device_type as u32),
    );
}

pub fn emit_firmware_status_changed(
    env: &Env,
    manufacturer_id: &BytesN<32>,
    version: u32,
    status: FirmwareStatus,
) {
    env.events().publish(
        ("IoT", symbol_short!("fw_sts")),
        (manufacturer_id.clone(), version, status as u32),
    );
}

pub fn emit_firmware_updated(
    env: &Env,
    device_id: &BytesN<32>,
    from_version: u32,
    to_version: u32,
    success: bool,
) {
    env.events().publish(
        ("IoT", symbol_short!("fw_upd")),
        (device_id.clone(), from_version, to_version, success),
    );
}

pub fn emit_heartbeat(env: &Env, device_id: &BytesN<32>, health_status: HealthStatus) {
    env.events().publish(
        ("IoT", symbol_short!("hbeat")),
        (device_id.clone(), health_status as u32),
    );
}

pub fn emit_key_rotated(env: &Env, device_id: &BytesN<32>, rotation_count: u32) {
    env.events().publish(
        ("IoT", symbol_short!("keyrot")),
        (device_id.clone(), rotation_count),
    );
}

pub fn emit_manufacturer_registered(env: &Env, manufacturer_id: &BytesN<32>, name: &str) {
    env.events().publish(
        ("IoT", symbol_short!("mfr_reg")),
        manufacturer_id.clone(),
    );
}

pub fn emit_paused(env: &Env, admin: &Address) {
    env.events()
        .publish(("IoT", symbol_short!("paused")), admin.clone());
}

pub fn emit_unpaused(env: &Env, admin: &Address) {
    env.events()
        .publish(("IoT", symbol_short!("unpause")), admin.clone());
}
