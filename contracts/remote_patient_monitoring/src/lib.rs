#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Symbol, Vec};

#[cfg(test)]
mod test;

const MAX_DEVICE_TYPES: u32 = 64;
const MAX_BATTERY_LEVEL: u32 = 100;
const LOW_BATTERY_THRESHOLD: u32 = 20;
const MAX_ALERT_SEVERITY: u32 = 5;

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
                if device_type >= MAX_DEVICE_TYPES || connectivity.is_empty() {
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
                Self::append_device_to_patient_index(&env, &patient, device_id);
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

            if !device.caregivers.contains(&caregiver) {
                device.caregivers.push_back(caregiver);
            }
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
        let device_key = (Symbol::new(&env, "device"), device_id);
        let Some(device) = env
            .storage()
            .persistent()
            .get::<(Symbol, u64), Device>(&device_key)
        else {
            return;
        };
        if device.patient != patient {
            return;
        }

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
        Self::append_vital_index(&env, &patient, env.ledger().sequence());

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
                Self::append_alert_index(&env, &patient, env.ledger().sequence());

                // Emit event for notifications
                env.events()
                    .publish((Symbol::new(&env, "alert"), patient.clone()), alert.clone());

                // Notify caregivers
                for caregiver in device.caregivers.iter() {
                    env.events().publish(
                        (Symbol::new(&env, "caregiver_alert"), caregiver.clone()),
                        alert.clone(),
                    );
                    Self::append_caregiver_alert_index(
                        &env,
                        &caregiver,
                        patient.clone(),
                        env.ledger().sequence(),
                    );
                }
            }
        }

        // Update device last seen
        let last_seen_key = (Symbol::new(&env, "last_seen"), device_id);
        env.storage()
            .persistent()
            .set(&last_seen_key, &env.ledger().timestamp());
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
        if min_value > max_value
            || alert_severity == 0
            || alert_severity > MAX_ALERT_SEVERITY
            || !Self::caller_can_manage_thresholds(&env, &caller, &patient)
        {
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

    // Update battery level
    pub fn update_battery_level(env: Env, caller: Address, device_id: u64, battery_level: u32) {
        caller.require_auth();
        if battery_level > MAX_BATTERY_LEVEL {
            return;
        }
        let device_key = (Symbol::new(&env, "device"), device_id);
        if let Some(mut device) = env
            .storage()
            .persistent()
            .get::<(Symbol, u64), Device>(&device_key)
        {
            device.battery_level = Some(battery_level);
            env.storage().persistent().set(&device_key, &device);

            // Check for low battery alert
            if battery_level < LOW_BATTERY_THRESHOLD {
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
                Self::append_alert_index(&env, &device.patient, env.ledger().sequence());
                env.events().publish(
                    (Symbol::new(&env, "alert"), device.patient.clone()),
                    alert.clone(),
                );
                for caregiver in device.caregivers.iter() {
                    env.events().publish(
                        (Symbol::new(&env, "caregiver_alert"), caregiver.clone()),
                        alert.clone(),
                    );
                    Self::append_caregiver_alert_index(
                        &env,
                        &caregiver,
                        device.patient.clone(),
                        env.ledger().sequence(),
                    );
                }
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
        Self::collect_vitals(&_env, &_patient, _limit)
    }

    // Get alerts for patient
    pub fn get_alerts(_env: Env, _patient: Address, _limit: u32) -> Vec<Alert> {
        Self::collect_alerts(&_env, &_patient, _limit)
    }

    // Get caregiver alerts
    pub fn get_caregiver_alerts(_env: Env, _caregiver: Address) -> Vec<Alert> {
        let keys: Vec<(Address, u32)> = _env
            .storage()
            .persistent()
            .get(&(Symbol::new(&_env, "cg_alerts"), _caregiver))
            .unwrap_or(Vec::new(&_env));
        let mut alerts = Vec::new(&_env);
        for entry in keys.iter() {
            let key = (Symbol::new(&_env, "alert"), entry.0, entry.1);
            if let Some(alert) = _env
                .storage()
                .persistent()
                .get::<(Symbol, Address, u32), Alert>(&key)
            {
                alerts.push_back(alert);
            }
        }
        alerts
    }

    fn append_device_to_patient_index(env: &Env, patient: &Address, device_id: u64) {
        let key = (Symbol::new(env, "pt_devices"), patient.clone());
        let mut device_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));
        if !device_ids.contains(device_id) {
            device_ids.push_back(device_id);
            env.storage().persistent().set(&key, &device_ids);
        }
    }

    fn append_vital_index(env: &Env, patient: &Address, sequence: u32) {
        let key = (Symbol::new(env, "vital_idx"), patient.clone());
        let mut sequences: Vec<u32> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));
        sequences.push_back(sequence);
        env.storage().persistent().set(&key, &sequences);
    }

    fn append_alert_index(env: &Env, patient: &Address, sequence: u32) {
        let key = (Symbol::new(env, "alert_idx"), patient.clone());
        let mut sequences: Vec<u32> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));
        sequences.push_back(sequence);
        env.storage().persistent().set(&key, &sequences);
    }

    fn append_caregiver_alert_index(
        env: &Env,
        caregiver: &Address,
        patient: Address,
        sequence: u32,
    ) {
        let key = (Symbol::new(env, "cg_alerts"), caregiver.clone());
        let mut alert_keys: Vec<(Address, u32)> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));
        alert_keys.push_back((patient, sequence));
        env.storage().persistent().set(&key, &alert_keys);
    }

    fn caller_can_manage_thresholds(env: &Env, caller: &Address, patient: &Address) -> bool {
        if caller == patient {
            return true;
        }

        let key = (Symbol::new(env, "pt_devices"), patient.clone());
        let device_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));
        for device_id in device_ids.iter() {
            let device_key = (Symbol::new(env, "device"), device_id);
            if let Some(device) = env
                .storage()
                .persistent()
                .get::<(Symbol, u64), Device>(&device_key)
            {
                if device.caregivers.contains(caller) {
                    return true;
                }
            }
        }
        false
    }

    fn collect_vitals(env: &Env, patient: &Address, limit: u32) -> Vec<VitalSign> {
        let sequences: Vec<u32> = env
            .storage()
            .persistent()
            .get(&(Symbol::new(env, "vital_idx"), patient.clone()))
            .unwrap_or(Vec::new(env));
        let mut vitals = Vec::new(env);
        let start = if limit == 0 || limit >= sequences.len() {
            0
        } else {
            sequences.len().saturating_sub(limit)
        };
        for idx in start..sequences.len() {
            let seq = sequences.get(idx).unwrap_or(0);
            let key = (Symbol::new(env, "vital"), patient.clone(), seq);
            if let Some(vital) = env
                .storage()
                .persistent()
                .get::<(Symbol, Address, u32), VitalSign>(&key)
            {
                vitals.push_back(vital);
            }
        }
        vitals
    }

    fn collect_alerts(env: &Env, patient: &Address, limit: u32) -> Vec<Alert> {
        let sequences: Vec<u32> = env
            .storage()
            .persistent()
            .get(&(Symbol::new(env, "alert_idx"), patient.clone()))
            .unwrap_or(Vec::new(env));
        let mut alerts = Vec::new(env);
        let start = if limit == 0 || limit >= sequences.len() {
            0
        } else {
            sequences.len().saturating_sub(limit)
        };
        for idx in start..sequences.len() {
            let seq = sequences.get(idx).unwrap_or(0);
            let key = (Symbol::new(env, "alert"), patient.clone(), seq);
            if let Some(alert) = env
                .storage()
                .persistent()
                .get::<(Symbol, Address, u32), Alert>(&key)
            {
                alerts.push_back(alert);
            }
        }
        alerts
    }
}
