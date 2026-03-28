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

    // ============================================================
    // MANUFACTURERS
    // ============================================================

    pub fn register_manufacturer(
        env: Env,
        admin: Address,
        manufacturer_id: BytesN<32>,
        name: String,
        certification_hash: BytesN<32>,
    ) -> Result<(), IoTError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        Self::check_not_paused(&env)?;
        validation::validate_name(&name)?;

        if env.storage().persistent().has(&DataKey::Manufacturer(manufacturer_id.clone())) {
            return Err(IoTError::ManufacturerAlreadyRegistered);
        }

        let mfr = Manufacturer {
            manufacturer_id: manufacturer_id.clone(),
            address: admin.clone(),
            name,
            certification_hash,
            is_active: true,
            registered_at: env.ledger().timestamp(),
            device_count: 0,
        };

        env.storage().persistent().set(&DataKey::Manufacturer(manufacturer_id.clone()), &mfr);
        env.storage().persistent().set(&DataKey::ManufacturerByAddr(admin.clone()), &manufacturer_id);

        let count: u32 = env.storage().persistent().get(&DataKey::ManufacturerCount).unwrap_or(0);
        env.storage().persistent().set(&DataKey::ManufacturerCount, &count.checked_add(1).unwrap());

        Ok(())
    }

    pub fn get_manufacturer(env: Env, manufacturer_id: BytesN<32>) -> Result<Manufacturer, IoTError> {
        env.storage()
            .persistent()
            .get(&DataKey::Manufacturer(manufacturer_id))
            .ok_or(IoTError::ManufacturerNotRegistered)
    }

    pub fn deactivate_manufacturer(
        env: Env,
        admin: Address,
        manufacturer_id: BytesN<32>,
    ) -> Result<(), IoTError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        Self::check_not_paused(&env)?;

        let mut mfr: Manufacturer = env
            .storage()
            .persistent()
            .get(&DataKey::Manufacturer(manufacturer_id.clone()))
            .ok_or(IoTError::ManufacturerNotRegistered)?;

        mfr.is_active = false;
        env.storage().persistent().set(&DataKey::Manufacturer(manufacturer_id), &mfr);
        Ok(())
    }

    // ============================================================
    // DEVICE ENROLLMENT & PROVISIONING
    // ============================================================

    pub fn register_device(
        env: Env,
        operator: Address,
        device_id: BytesN<32>,
        manufacturer_id: BytesN<32>,
        device_type: DeviceType,
        model: String,
        serial_number: String,
        location: String,
        encryption_key_hash: BytesN<32>,
        metadata_ref: String,
    ) -> Result<(), IoTError> {
        operator.require_auth();
        Self::check_not_paused(&env)?;
        Self::require_role(&env, &operator, Role::Operator)?;

        validation::validate_model(&model)?;
        validation::validate_serial(&serial_number)?;
        validation::validate_location(&location)?;

        if env.storage().persistent().has(&DataKey::Device(device_id.clone())) {
            return Err(IoTError::DeviceAlreadyRegistered);
        }

        let mfr: Manufacturer = env
            .storage()
            .persistent()
            .get(&DataKey::Manufacturer(manufacturer_id.clone()))
            .ok_or(IoTError::ManufacturerNotRegistered)?;
        if !mfr.is_active {
            return Err(IoTError::ManufacturerNotRegistered);
        }

        let now = env.ledger().timestamp();
        let device = Device {
            device_id: device_id.clone(),
            manufacturer_id: manufacturer_id.clone(),
            device_type,
            model,
            serial_number,
            firmware_version: 0,
            status: DeviceStatus::Provisioning,
            operator: operator.clone(),
            location,
            registered_at: now,
            last_heartbeat: 0,
            health_status: HealthStatus::Offline,
            uptime_start: 0,
            total_uptime_secs: 0,
            total_downtime_secs: 0,
            encryption_key_hash,
            metadata_ref,
        };

        env.storage().persistent().set(&DataKey::Device(device_id.clone()), &device);

        // Index by operator
        let mut op_devices: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::DevicesByOperator(operator.clone()))
            .unwrap_or(Vec::new(&env));
        op_devices.push_back(device_id.clone());
        env.storage().persistent().set(&DataKey::DevicesByOperator(operator.clone()), &op_devices);

        // Index by manufacturer
        let mut mfr_devices: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::DevicesByManufacturer(manufacturer_id.clone()))
            .unwrap_or(Vec::new(&env));
        mfr_devices.push_back(device_id.clone());
        env.storage().persistent().set(&DataKey::DevicesByManufacturer(manufacturer_id.clone()), &mfr_devices);

        // Update manufacturer device count
        let mut updated_mfr = mfr;
        updated_mfr.device_count = updated_mfr.device_count.checked_add(1).unwrap();
        env.storage().persistent().set(&DataKey::Manufacturer(manufacturer_id), &updated_mfr);

        // Increment device count
        let count: u64 = env.storage().persistent().get(&DataKey::DeviceCount).unwrap_or(0);
        env.storage().persistent().set(&DataKey::DeviceCount, &count.checked_add(1).unwrap());

        events::emit_device_registered(&env, &device_id, device_type, &operator);
        Ok(())
    }

    pub fn get_device(env: Env, device_id: BytesN<32>) -> Result<Device, IoTError> {
        env.storage()
            .persistent()
            .get(&DataKey::Device(device_id))
            .ok_or(IoTError::DeviceNotFound)
    }

    pub fn get_device_count(env: Env) -> u64 {
        env.storage().persistent().get(&DataKey::DeviceCount).unwrap_or(0)
    }

    pub fn get_devices_by_operator(env: Env, operator: Address) -> Vec<BytesN<32>> {
        env.storage()
            .persistent()
            .get(&DataKey::DevicesByOperator(operator))
            .unwrap_or(Vec::new(&env))
    }

    pub fn activate_device(
        env: Env,
        caller: Address,
        device_id: BytesN<32>,
    ) -> Result<(), IoTError> {
        caller.require_auth();
        Self::check_not_paused(&env)?;
        Self::require_role(&env, &caller, Role::Operator)?;

        let mut device: Device = env
            .storage()
            .persistent()
            .get(&DataKey::Device(device_id.clone()))
            .ok_or(IoTError::DeviceNotFound)?;

        if device.status == DeviceStatus::Decommissioned {
            return Err(IoTError::DeviceDecommissioned);
        }

        let old_status = device.status;
        let now = env.ledger().timestamp();
        device.status = DeviceStatus::Active;
        device.health_status = HealthStatus::Healthy;
        device.uptime_start = now;

        env.storage().persistent().set(&DataKey::Device(device_id.clone()), &device);

        // Increment active count if transitioning from non-active
        if old_status != DeviceStatus::Active {
            let active: u64 = env.storage().persistent().get(&DataKey::ActiveDeviceCount).unwrap_or(0);
            env.storage().persistent().set(&DataKey::ActiveDeviceCount, &active.checked_add(1).unwrap());
        }

        events::emit_device_status_changed(&env, &device_id, old_status, DeviceStatus::Active);
        Ok(())
    }

    pub fn suspend_device(
        env: Env,
        caller: Address,
        device_id: BytesN<32>,
    ) -> Result<(), IoTError> {
        caller.require_auth();
        Self::check_not_paused(&env)?;
        Self::require_role(&env, &caller, Role::Operator)?;

        let mut device: Device = env
            .storage()
            .persistent()
            .get(&DataKey::Device(device_id.clone()))
            .ok_or(IoTError::DeviceNotFound)?;

        if device.status == DeviceStatus::Decommissioned {
            return Err(IoTError::DeviceDecommissioned);
        }

        let old_status = device.status;
        let now = env.ledger().timestamp();

        // Accumulate uptime if was active
        if old_status == DeviceStatus::Active && device.uptime_start > 0 {
            let uptime_delta = now.saturating_sub(device.uptime_start);
            device.total_uptime_secs = device.total_uptime_secs.saturating_add(uptime_delta);
        }

        device.status = DeviceStatus::Suspended;
        device.uptime_start = 0;
        env.storage().persistent().set(&DataKey::Device(device_id.clone()), &device);

        if old_status == DeviceStatus::Active {
            let active: u64 = env.storage().persistent().get(&DataKey::ActiveDeviceCount).unwrap_or(0);
            env.storage().persistent().set(&DataKey::ActiveDeviceCount, &active.saturating_sub(1));
        }

        events::emit_device_status_changed(&env, &device_id, old_status, DeviceStatus::Suspended);
        Ok(())
    }

    pub fn decommission_device(
        env: Env,
        admin: Address,
        device_id: BytesN<32>,
    ) -> Result<(), IoTError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        Self::check_not_paused(&env)?;

        let mut device: Device = env
            .storage()
            .persistent()
            .get(&DataKey::Device(device_id.clone()))
            .ok_or(IoTError::DeviceNotFound)?;

        let old_status = device.status;
        device.status = DeviceStatus::Decommissioned;
        device.health_status = HealthStatus::Offline;
        device.uptime_start = 0;
        env.storage().persistent().set(&DataKey::Device(device_id.clone()), &device);

        if old_status == DeviceStatus::Active {
            let active: u64 = env.storage().persistent().get(&DataKey::ActiveDeviceCount).unwrap_or(0);
            env.storage().persistent().set(&DataKey::ActiveDeviceCount, &active.saturating_sub(1));
        }

        events::emit_device_status_changed(&env, &device_id, old_status, DeviceStatus::Decommissioned);
        Ok(())
    }

    // ============================================================
    // FIRMWARE MANAGEMENT
    // ============================================================

    pub fn publish_firmware(
        env: Env,
        caller: Address,
        manufacturer_id: BytesN<32>,
        version: u32,
        device_type: DeviceType,
        binary_hash: BytesN<32>,
        release_notes_ref: String,
        min_version: u32,
        size_bytes: u64,
    ) -> Result<(), IoTError> {
        caller.require_auth();
        Self::check_not_paused(&env)?;
        Self::require_role(&env, &caller, Role::Manufacturer)?;

        // Verify manufacturer exists
        let _mfr: Manufacturer = env
            .storage()
            .persistent()
            .get(&DataKey::Manufacturer(manufacturer_id.clone()))
            .ok_or(IoTError::ManufacturerNotRegistered)?;

        if env.storage().persistent().has(&DataKey::Firmware(manufacturer_id.clone(), version)) {
            return Err(IoTError::FirmwareAlreadyExists);
        }

        let fw = FirmwareVersion {
            version,
            manufacturer_id: manufacturer_id.clone(),
            device_type,
            binary_hash,
            release_notes_ref,
            status: FirmwareStatus::Pending,
            min_version,
            published_at: env.ledger().timestamp(),
            approved_by: caller.clone(),
            size_bytes,
        };

        env.storage().persistent().set(&DataKey::Firmware(manufacturer_id.clone(), version), &fw);
        events::emit_firmware_published(&env, &manufacturer_id, version, device_type);
        Ok(())
    }

    pub fn approve_firmware(
        env: Env,
        admin: Address,
        manufacturer_id: BytesN<32>,
        version: u32,
    ) -> Result<(), IoTError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        Self::check_not_paused(&env)?;

        let mut fw: FirmwareVersion = env
            .storage()
            .persistent()
            .get(&DataKey::Firmware(manufacturer_id.clone(), version))
            .ok_or(IoTError::FirmwareVersionNotFound)?;

        fw.status = FirmwareStatus::Approved;
        fw.approved_by = admin;
        env.storage().persistent().set(&DataKey::Firmware(manufacturer_id.clone(), version), &fw);

        // Update latest firmware pointer
        env.storage().persistent().set(
            &DataKey::LatestFirmware(manufacturer_id.clone(), fw.device_type as u32),
            &version,
        );

        events::emit_firmware_status_changed(&env, &manufacturer_id, version, FirmwareStatus::Approved);
        Ok(())
    }

    pub fn reject_firmware(
        env: Env,
        admin: Address,
        manufacturer_id: BytesN<32>,
        version: u32,
    ) -> Result<(), IoTError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let mut fw: FirmwareVersion = env
            .storage()
            .persistent()
            .get(&DataKey::Firmware(manufacturer_id.clone(), version))
            .ok_or(IoTError::FirmwareVersionNotFound)?;

        fw.status = FirmwareStatus::Rejected;
        env.storage().persistent().set(&DataKey::Firmware(manufacturer_id.clone(), version), &fw);
        events::emit_firmware_status_changed(&env, &manufacturer_id, version, FirmwareStatus::Rejected);
        Ok(())
    }

    pub fn get_firmware(
        env: Env,
        manufacturer_id: BytesN<32>,
        version: u32,
    ) -> Result<FirmwareVersion, IoTError> {
        env.storage()
            .persistent()
            .get(&DataKey::Firmware(manufacturer_id, version))
            .ok_or(IoTError::FirmwareVersionNotFound)
    }

    pub fn get_latest_firmware_version(
        env: Env,
        manufacturer_id: BytesN<32>,
        device_type: DeviceType,
    ) -> Result<u32, IoTError> {
        env.storage()
            .persistent()
            .get(&DataKey::LatestFirmware(manufacturer_id, device_type as u32))
            .ok_or(IoTError::FirmwareVersionNotFound)
    }

    pub fn update_device_firmware(
        env: Env,
        caller: Address,
        device_id: BytesN<32>,
        target_version: u32,
    ) -> Result<u64, IoTError> {
        caller.require_auth();
        Self::check_not_paused(&env)?;
        Self::require_role(&env, &caller, Role::Operator)?;

        let mut device: Device = env
            .storage()
            .persistent()
            .get(&DataKey::Device(device_id.clone()))
            .ok_or(IoTError::DeviceNotFound)?;

        if device.status == DeviceStatus::Decommissioned {
            return Err(IoTError::DeviceDecommissioned);
        }

        // Cannot downgrade
        if target_version <= device.firmware_version {
            return Err(IoTError::DowngradeNotAllowed);
        }

        // Firmware must be approved
        let fw: FirmwareVersion = env
            .storage()
            .persistent()
            .get(&DataKey::Firmware(device.manufacturer_id.clone(), target_version))
            .ok_or(IoTError::FirmwareVersionNotFound)?;

        if fw.status != FirmwareStatus::Approved {
            return Err(IoTError::FirmwareNotApproved);
        }

        let from_version = device.firmware_version;
        let now = env.ledger().timestamp();

        // Record the update
        let update_count: u64 = env.storage().persistent().get(&DataKey::FirmwareUpdateCount).unwrap_or(0);
        let update_id = update_count;

        let record = FirmwareUpdateRecord {
            update_id,
            device_id: device_id.clone(),
            from_version,
            to_version: target_version,
            initiated_by: caller.clone(),
            initiated_at: now,
            completed_at: now,
            success: true,
            error_ref: String::from_str(&env, ""),
        };

        env.storage().persistent().set(&DataKey::FirmwareUpdateRecord(update_id), &record);
        env.storage().persistent().set(&DataKey::FirmwareUpdateCount, &update_count.checked_add(1).unwrap());

        // Track per-device updates
        let mut device_updates: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::DeviceFirmwareUpdates(device_id.clone()))
            .unwrap_or(Vec::new(&env));
        device_updates.push_back(update_id);
        env.storage().persistent().set(&DataKey::DeviceFirmwareUpdates(device_id.clone()), &device_updates);

        // Update device firmware version
        device.firmware_version = target_version;
        env.storage().persistent().set(&DataKey::Device(device_id.clone()), &device);

        events::emit_firmware_updated(&env, &device_id, from_version, target_version, true);
        Ok(update_id)
    }

    // ============================================================
    // DEVICE HEALTH MONITORING
    // ============================================================

    pub fn submit_heartbeat(
        env: Env,
        caller: Address,
        device_id: BytesN<32>,
        health_status: HealthStatus,
        battery_pct: u32,
        signal_strength: u32,
        error_count: u32,
        metrics_ref: String,
    ) -> Result<(), IoTError> {
        caller.require_auth();
        Self::check_not_paused(&env)?;
        Self::require_role(&env, &caller, Role::Operator)?;

        validation::validate_metric_value(battery_pct, 100)?;
        validation::validate_metric_value(signal_strength, 100)?;

        let mut device: Device = env
            .storage()
            .persistent()
            .get(&DataKey::Device(device_id.clone()))
            .ok_or(IoTError::DeviceNotFound)?;

        if device.status == DeviceStatus::Decommissioned {
            return Err(IoTError::DeviceDecommissioned);
        }
        if device.status != DeviceStatus::Active && device.status != DeviceStatus::Maintenance {
            return Err(IoTError::DeviceNotActive);
        }

        let now = env.ledger().timestamp();
        let min_interval: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::HeartbeatMinInterval)
            .unwrap_or(60);

        if device.last_heartbeat > 0 && now.saturating_sub(device.last_heartbeat) < min_interval {
            return Err(IoTError::HeartbeatTooFrequent);
        }

        let heartbeat = Heartbeat {
            device_id: device_id.clone(),
            timestamp: now,
            health_status,
            battery_pct,
            signal_strength,
            error_count,
            metrics_ref,
        };

        // Store last N heartbeats (rolling window of 10)
        let mut heartbeats: Vec<Heartbeat> = env
            .storage()
            .persistent()
            .get(&DataKey::DeviceHeartbeats(device_id.clone()))
            .unwrap_or(Vec::new(&env));

        if heartbeats.len() >= 10 {
            heartbeats.remove(0);
        }
        heartbeats.push_back(heartbeat);
        env.storage().persistent().set(&DataKey::DeviceHeartbeats(device_id.clone()), &heartbeats);

        // Update device health
        device.last_heartbeat = now;
        device.health_status = health_status;
        env.storage().persistent().set(&DataKey::Device(device_id.clone()), &device);

        events::emit_heartbeat(&env, &device_id, health_status);
        Ok(())
    }

    pub fn get_device_heartbeats(
        env: Env,
        device_id: BytesN<32>,
    ) -> Result<Vec<Heartbeat>, IoTError> {
        if !env.storage().persistent().has(&DataKey::Device(device_id.clone())) {
            return Err(IoTError::DeviceNotFound);
        }
        Ok(env
            .storage()
            .persistent()
            .get(&DataKey::DeviceHeartbeats(device_id))
            .unwrap_or(Vec::new(&env)))
    }

    pub fn get_device_uptime_bps(env: Env, device_id: BytesN<32>) -> Result<u32, IoTError> {
        let device: Device = env
            .storage()
            .persistent()
            .get(&DataKey::Device(device_id))
            .ok_or(IoTError::DeviceNotFound)?;

        let now = env.ledger().timestamp();

        // Calculate current uptime
        let mut total_up = device.total_uptime_secs;
        if device.status == DeviceStatus::Active && device.uptime_start > 0 {
            total_up = total_up.saturating_add(now.saturating_sub(device.uptime_start));
        }

        let total_time = total_up.saturating_add(device.total_downtime_secs);
        if total_time == 0 {
            // Device just registered, consider it at 100% if active
            if device.status == DeviceStatus::Active {
                return Ok(10000);
            }
            return Ok(0);
        }

        // bps = (uptime / total) * 10000
        let bps = total_up
            .checked_mul(10000)
            .unwrap_or(u64::MAX)
            .checked_div(total_time)
            .unwrap_or(0);

        Ok(bps as u32)
    }

    pub fn get_active_device_count(env: Env) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::ActiveDeviceCount)
            .unwrap_or(0)
    }

    pub fn set_heartbeat_interval(
        env: Env,
        admin: Address,
        interval_secs: u64,
    ) -> Result<(), IoTError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        env.storage().persistent().set(&DataKey::HeartbeatMinInterval, &interval_secs);
        Ok(())
    }

    // ============================================================
    // SECURE DEVICE COMMUNICATION
    // ============================================================

    pub fn create_comm_channel(
        env: Env,
        caller: Address,
        device_id: BytesN<32>,
        channel_id: BytesN<32>,
        encryption_key_hash: BytesN<32>,
        protocol: String,
    ) -> Result<(), IoTError> {
        caller.require_auth();
        Self::check_not_paused(&env)?;
        Self::require_role(&env, &caller, Role::Operator)?;

        // Device must exist
        let _device: Device = env
            .storage()
            .persistent()
            .get(&DataKey::Device(device_id.clone()))
            .ok_or(IoTError::DeviceNotFound)?;

        let now = env.ledger().timestamp();
        let channel = CommChannel {
            channel_id: channel_id.clone(),
            device_id: device_id.clone(),
            encryption_key_hash,
            protocol,
            created_at: now,
            last_rotated: now,
            rotation_count: 0,
        };

        env.storage().persistent().set(&DataKey::CommChannel(channel_id.clone()), &channel);
        env.storage().persistent().set(&DataKey::DeviceChannel(device_id), &channel_id);
        Ok(())
    }

    pub fn get_comm_channel(
        env: Env,
        channel_id: BytesN<32>,
    ) -> Result<CommChannel, IoTError> {
        env.storage()
            .persistent()
            .get(&DataKey::CommChannel(channel_id))
            .ok_or(IoTError::ChannelNotFound)
    }

    pub fn rotate_encryption_key(
        env: Env,
        caller: Address,
        channel_id: BytesN<32>,
        new_encryption_key_hash: BytesN<32>,
    ) -> Result<(), IoTError> {
        caller.require_auth();
        Self::check_not_paused(&env)?;
        Self::require_role(&env, &caller, Role::Operator)?;

        let mut channel: CommChannel = env
            .storage()
            .persistent()
            .get(&DataKey::CommChannel(channel_id.clone()))
            .ok_or(IoTError::ChannelNotFound)?;

        let now = env.ledger().timestamp();
        let min_interval: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::KeyRotationMinInterval)
            .unwrap_or(3600);

        if now.saturating_sub(channel.last_rotated) < min_interval {
            return Err(IoTError::KeyRotationTooFrequent);
        }

        channel.encryption_key_hash = new_encryption_key_hash;
        channel.last_rotated = now;
        channel.rotation_count = channel.rotation_count.checked_add(1).unwrap();

        env.storage().persistent().set(&DataKey::CommChannel(channel_id), &channel);
        events::emit_key_rotated(&env, &channel.device_id, channel.rotation_count);
        Ok(())
    }

    pub fn rotate_device_key(
        env: Env,
        caller: Address,
        device_id: BytesN<32>,
        new_encryption_key_hash: BytesN<32>,
    ) -> Result<(), IoTError> {
        caller.require_auth();
        Self::check_not_paused(&env)?;
        Self::require_role(&env, &caller, Role::Operator)?;

        let mut device: Device = env
            .storage()
            .persistent()
            .get(&DataKey::Device(device_id.clone()))
            .ok_or(IoTError::DeviceNotFound)?;

        if device.status == DeviceStatus::Decommissioned {
            return Err(IoTError::DeviceDecommissioned);
        }

        device.encryption_key_hash = new_encryption_key_hash;
        env.storage().persistent().set(&DataKey::Device(device_id.clone()), &device);
        events::emit_key_rotated(&env, &device_id, 0);
        Ok(())
    }

    pub fn set_key_rotation_interval(
        env: Env,
        admin: Address,
        interval_secs: u64,
    ) -> Result<(), IoTError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        env.storage().persistent().set(&DataKey::KeyRotationMinInterval, &interval_secs);
        Ok(())
    }
}
