#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Symbol, Vec};

#[contract]
pub struct RemotePatientMonitoringContract;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Device {
    pub id: u64,
    pub device_type: u32, // 0: BloodPressureMonitor, 1: HeartRateMonitor, 2: GlucoseMeter, etc.
    pub patient: Address,
    pub caregivers: Vec<Address>,
    pub connectivity: Vec<String>,  // WiFi, Cellular, Bluetooth
    pub battery_level: Option<u32>, // 0-100
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VitalSign {
    pub patient: Address,
    pub device_id: u64,
    pub timestamp: u64,
    pub vital_type: String, // e.g., "heart_rate", "blood_pressure"
    pub value: i64,         // scaled value
    pub unit: String,
    pub quality: u32, // 0-100, data quality
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Alert {
    pub patient: Address,
    pub alert_type: u32, // 0: ThresholdExceeded, 1: DeviceOffline, 2: BatteryLow, 3: AbnormalReading
    pub message: String,
    pub timestamp: u64,
    pub severity: u32, // 1-5
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Threshold {
    pub vital_type: String,
    pub min_value: i64,
    pub max_value: i64,
    pub alert_severity: u32,
}

#[contractimpl]
impl RemotePatientMonitoringContract {
    // Initialize the contract
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "admin"), &admin);
    }

    // Register a device
    pub fn register_device(
        env: Env,
        caller: Address,
        device_id: u64,
        device_type: u32,
        patient: Address,
        connectivity: Vec<String>,
    ) {
        caller.require_auth();
        let admin_opt: Option<Address> = env.storage().instance().get(&Symbol::new(&env, "admin"));
        if let Some(admin) = admin_opt {
            if caller == admin || caller == patient {
                // Validate device type (0-255)
                if device_type > 255 {
                    return;
                }
                let device = Device {
                    id: device_id,
                    device_type,
                    patient: patient.clone(),
                    caregivers: Vec::new(&env),
                    connectivity,
                    battery_level: None,
                };

                let key = (Symbol::new(&env, "device"), device_id);
                env.storage().persistent().set(&key, &device);
            }
        }
    }

    // Add caregiver to device
    pub fn add_caregiver(env: Env, caller: Address, device_id: u64, caregiver: Address) {
        caller.require_auth();
        let key = (Symbol::new(&env, "device"), device_id);
        if let Some(mut device) = env
            .storage()
            .persistent()
            .get::<(Symbol, u64), Device>(&key)
        {
            if caller != device.patient {
                return;
            }

            device.caregivers.push_back(caregiver);
            env.storage().persistent().set(&key, &device);
        }
    }

    // Submit vital sign
    #[allow(clippy::too_many_arguments)]
    pub fn submit_vital_sign(
        env: Env,
        caller: Address,
        patient: Address,
        device_id: u64,
        vital_type: String,
        value: i64,
        unit: String,
        quality: u32,
    ) {
        caller.require_auth();
        // Assume caller is authorized device or oracle

        let vital = VitalSign {
            patient: patient.clone(),
            device_id,
            timestamp: env.ledger().timestamp(),
            vital_type: vital_type.clone(),
            value,
            unit,
            quality,
        };

        // Store vital sign
        let key = (
            Symbol::new(&env, "vital"),
            patient.clone(),
            env.ledger().sequence(),
        );
        env.storage().persistent().set(&key, &vital);

        // Check thresholds and create alert if needed
        let threshold_key = (
            Symbol::new(&env, "threshold"),
            patient.clone(),
            vital_type.clone(),
        );
        if let Some(threshold) = env
            .storage()
            .persistent()
            .get::<(Symbol, Address, String), Threshold>(&threshold_key)
        {
            if value < threshold.min_value || value > threshold.max_value {
                let alert = Alert {
                    patient: patient.clone(),
                    alert_type: 0, // ThresholdExceeded
                    message: String::from_str(&env, "Vital sign out of threshold range"),
                    timestamp: env.ledger().timestamp(),
                    severity: threshold.alert_severity,
                };
                let alert_key = (
                    Symbol::new(&env, "alert"),
                    patient.clone(),
                    env.ledger().sequence(),
                );
                env.storage().persistent().set(&alert_key, &alert);

                // Emit event for notifications
                env.events()
                    .publish((Symbol::new(&env, "alert"), patient.clone()), alert.clone());

                // Notify caregivers
                let device_key = (Symbol::new(&env, "device"), device_id);
                if let Some(device) = env
                    .storage()
                    .persistent()
                    .get::<(Symbol, u64), Device>(&device_key)
                {
                    for caregiver in device.caregivers.iter() {
                        env.events().publish(
                            (Symbol::new(&env, "caregiver_alert"), caregiver.clone()),
                            alert.clone(),
                        );
                    }
                }
            }
        }

        // Update device last seen
        let device_key = (Symbol::new(&env, "device"), device_id);
        if let Some(_device) = env
            .storage()
            .persistent()
            .get::<(Symbol, u64), Device>(&device_key)
        {
            let last_seen_key = (Symbol::new(&env, "last_seen"), device_id);
            env.storage()
                .persistent()
                .set(&last_seen_key, &env.ledger().timestamp());
        }
    }

    // Set threshold
    pub fn set_threshold(
        env: Env,
        caller: Address,
        patient: Address,
        vital_type: String,
        min_value: i64,
        max_value: i64,
        alert_severity: u32,
    ) {
        caller.require_auth();
        // Allow patient or caregivers to set thresholds
        let device_key = (Symbol::new(&env, "device"), 0u64); // Simplified, assume device 0
        if let Some(device) = env
            .storage()
            .persistent()
            .get::<(Symbol, u64), Device>(&device_key)
        {
            if caller != device.patient && !device.caregivers.contains(&caller) {
                return;
            }

            let threshold = Threshold {
                vital_type: vital_type.clone(),
                min_value,
                max_value,
                alert_severity,
            };
            let key = (Symbol::new(&env, "threshold"), patient, vital_type);
            env.storage().persistent().set(&key, &threshold);
        }
    }

    // Update battery level
    pub fn update_battery_level(env: Env, caller: Address, device_id: u64, battery_level: u32) {
        caller.require_auth();
        let device_key = (Symbol::new(&env, "device"), device_id);
        if let Some(mut device) = env
            .storage()
            .persistent()
            .get::<(Symbol, u64), Device>(&device_key)
        {
            device.battery_level = Some(battery_level);
            env.storage().persistent().set(&device_key, &device);

            // Check for low battery alert
            if battery_level < 20 {
                let alert = Alert {
                    patient: device.patient.clone(),
                    alert_type: 2, // BatteryLow
                    message: String::from_str(&env, "Device battery low"),
                    timestamp: env.ledger().timestamp(),
                    severity: 2,
                };
                let alert_key = (
                    Symbol::new(&env, "alert"),
                    device.patient.clone(),
                    env.ledger().sequence(),
                );
                env.storage().persistent().set(&alert_key, &alert);
                env.events()
                    .publish((Symbol::new(&env, "alert"), device.patient), alert);
            }
        }
    }

    // Get device info
    pub fn get_device(env: Env, device_id: u64) -> Option<Device> {
        let key = (Symbol::new(&env, "device"), device_id);
        env.storage().persistent().get(&key)
    }

    // Get vitals for patient (last N)
    pub fn get_vitals(_env: Env, _patient: Address, _limit: u32) -> Vec<VitalSign> {
        // Simplified: in practice, maintain an index or use events
        // For now, return empty
        Vec::new(&_env)
    }

    // Get alerts for patient
    pub fn get_alerts(_env: Env, _patient: Address, _limit: u32) -> Vec<Alert> {
        // Simplified
        Vec::new(&_env)
    }

    // Get caregiver alerts
    pub fn get_caregiver_alerts(_env: Env, _caregiver: Address) -> Vec<Alert> {
        // Simplified
        Vec::new(&_env)
    }
}
