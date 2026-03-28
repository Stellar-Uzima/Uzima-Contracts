#![no_std]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::used_underscore_binding)]

mod errors;
mod events;
mod validation;
pub use errors::IoTError;

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contractimpl, contracttype, Address, BytesN, Env, String, Vec,
};

// ============================================================
// ENUMS
// ============================================================

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum DeviceStatus {
    Provisioning = 0,
    Active = 1,
    Suspended = 2,
    Maintenance = 3,
    Decommissioned = 4,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum DeviceType {
    VitalSignsMonitor = 0,
    BloodPressureMonitor = 1,
    GlucoseMonitor = 2,
    PulseOximeter = 3,
    ECGMonitor = 4,
    TemperatureSensor = 5,
    InfusionPump = 6,
    Ventilator = 7,
    WearableSensor = 8,
    ImagingDevice = 9,
    LabAnalyzer = 10,
    Other = 99,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum FirmwareStatus {
    Pending = 0,
    Approved = 1,
    Rejected = 2,
    Deprecated = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum HealthStatus {
    Healthy = 0,
    Degraded = 1,
    Critical = 2,
    Offline = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum Role {
    Admin = 0,
    Manufacturer = 1,
    Operator = 2,
    Viewer = 3,
}

// ============================================================
// DATA STRUCTURES
// ============================================================

#[derive(Clone, Debug)]
#[contracttype]
pub struct Manufacturer {
    pub manufacturer_id: BytesN<32>,
    pub address: Address,
    pub name: String,
    pub certification_hash: BytesN<32>,
    pub is_active: bool,
    pub registered_at: u64,
    pub device_count: u32,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct Device {
    pub device_id: BytesN<32>,
    pub manufacturer_id: BytesN<32>,
    pub device_type: DeviceType,
    pub model: String,
    pub serial_number: String,
    pub firmware_version: u32,
    pub status: DeviceStatus,
    pub operator: Address,
    pub location: String,
    pub registered_at: u64,
    pub last_heartbeat: u64,
    pub health_status: HealthStatus,
    pub uptime_start: u64,
    pub total_uptime_secs: u64,
    pub total_downtime_secs: u64,
    pub encryption_key_hash: BytesN<32>,
    pub metadata_ref: String,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct FirmwareVersion {
    pub version: u32,
    pub manufacturer_id: BytesN<32>,
    pub device_type: DeviceType,
    pub binary_hash: BytesN<32>,
    pub release_notes_ref: String,
    pub status: FirmwareStatus,
    pub min_version: u32,
    pub published_at: u64,
    pub approved_by: Address,
    pub size_bytes: u64,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct FirmwareUpdateRecord {
    pub update_id: u64,
    pub device_id: BytesN<32>,
    pub from_version: u32,
    pub to_version: u32,
    pub initiated_by: Address,
    pub initiated_at: u64,
    pub completed_at: u64,
    pub success: bool,
    pub error_ref: String,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct Heartbeat {
    pub device_id: BytesN<32>,
    pub timestamp: u64,
    pub health_status: HealthStatus,
    pub battery_pct: u32,
    pub signal_strength: u32,
    pub error_count: u32,
    pub metrics_ref: String,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct CommChannel {
    pub channel_id: BytesN<32>,
    pub device_id: BytesN<32>,
    pub encryption_key_hash: BytesN<32>,
    pub protocol: String,
    pub created_at: u64,
    pub last_rotated: u64,
    pub rotation_count: u32,
}

// ============================================================
// STORAGE KEYS
// ============================================================

#[contracttype]
pub enum DataKey {
    // System
    Initialized,
    Admin,
    Paused,

    // RBAC
    UserRole(Address),

    // Manufacturers
    Manufacturer(BytesN<32>),
    ManufacturerByAddr(Address),
    ManufacturerCount,

    // Devices
    Device(BytesN<32>),
    DevicesByOperator(Address),
    DevicesByManufacturer(BytesN<32>),
    DevicesByType(u32),
    DeviceCount,
    ActiveDeviceCount,

    // Firmware
    Firmware(BytesN<32>, u32),         // (manufacturer_id, version)
    LatestFirmware(BytesN<32>, u32),   // (manufacturer_id, device_type) -> version
    FirmwareUpdateRecord(u64),
    FirmwareUpdateCount,
    DeviceFirmwareUpdates(BytesN<32>), // device_id -> Vec<u64>

    // Health
    DeviceHeartbeats(BytesN<32>),      // device_id -> Vec<Heartbeat> (last N)
    HeartbeatMinInterval,              // u64 seconds

    // Communication
    CommChannel(BytesN<32>),           // channel_id -> CommChannel
    DeviceChannel(BytesN<32>),         // device_id -> channel_id
    KeyRotationMinInterval,            // u64 seconds
}

// ============================================================
// CONTRACT
// ============================================================

#[contract]
pub struct IoTDeviceManagement;

#[contractimpl]
impl IoTDeviceManagement {
    // ============================================================
    // SYSTEM
    // ============================================================

    pub fn initialize(env: Env, admin: Address) -> Result<(), IoTError> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(IoTError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage().persistent().set(&DataKey::DeviceCount, &0u64);
        env.storage().persistent().set(&DataKey::ActiveDeviceCount, &0u64);
        env.storage().persistent().set(&DataKey::ManufacturerCount, &0u32);
        env.storage().persistent().set(&DataKey::FirmwareUpdateCount, &0u64);
        env.storage().persistent().set(&DataKey::HeartbeatMinInterval, &60u64);
        env.storage().persistent().set(&DataKey::KeyRotationMinInterval, &3600u64);
        events::emit_initialized(&env, &admin);
        Ok(())
    }

    pub fn pause(env: Env, admin: Address) -> Result<(), IoTError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        env.storage().instance().set(&DataKey::Paused, &true);
        events::emit_paused(&env, &admin);
        Ok(())
    }

    pub fn unpause(env: Env, admin: Address) -> Result<(), IoTError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        let paused: bool = env.storage().instance().get(&DataKey::Paused).unwrap_or(false);
        if !paused {
            return Err(IoTError::NotPaused);
        }
        env.storage().instance().set(&DataKey::Paused, &false);
        events::emit_unpaused(&env, &admin);
        Ok(())
    }

    // ============================================================
    // RBAC
    // ============================================================

    pub fn set_role(env: Env, admin: Address, user: Address, role: Role) -> Result<(), IoTError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        Self::check_not_paused(&env)?;
        env.storage().persistent().set(&DataKey::UserRole(user), &role);
        Ok(())
    }

    pub fn get_role(env: Env, user: Address) -> Role {
        env.storage()
            .persistent()
            .get(&DataKey::UserRole(user))
            .unwrap_or(Role::Viewer)
    }

    // ============================================================
    // INTERNAL HELPERS
    // ============================================================

    fn require_admin(env: &Env, caller: &Address) -> Result<(), IoTError> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(IoTError::NotInitialized);
        }
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if caller != &admin {
            return Err(IoTError::NotAdmin);
        }
        Ok(())
    }

    fn check_not_paused(env: &Env) -> Result<(), IoTError> {
        let paused: bool = env.storage().instance().get(&DataKey::Paused).unwrap_or(false);
        if paused {
            return Err(IoTError::ContractPaused);
        }
        Ok(())
    }

    fn require_role(env: &Env, caller: &Address, required: Role) -> Result<(), IoTError> {
        let role: Role = env
            .storage()
            .persistent()
            .get(&DataKey::UserRole(caller.clone()))
            .unwrap_or(Role::Viewer);
        // Admin can do anything; otherwise must match
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if caller == &admin {
            return Ok(());
        }
        if role as u32 > required as u32 {
            return Err(IoTError::NotAuthorized);
        }
        Ok(())
    }
}
