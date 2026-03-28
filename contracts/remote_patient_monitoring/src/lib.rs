#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Symbol, Vec};

#[contract]
pub struct RemotePatientMonitoringContract;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Device {
    pub id: u64,
    pub device_type: String,
    pub patient: Address,
    pub caregivers: Vec<Address>,
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
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Alert {
    pub patient: Address,
    pub alert_type: String,
    pub message: String,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Threshold {
    pub vital_type: String,
    pub min_value: i64,
    pub max_value: i64,
}

#[contractimpl]
impl RemotePatientMonitoringContract {
    // Initialize the contract
    pub fn initialize(env: Env, admin: Address) {
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "admin"), &admin);
    }

    // Register a device
    pub fn register_device(
        env: Env,
        caller: Address,
        device_id: u64,
        device_type: String,
        patient: Address,
    ) {
        caller.require_auth();
        let admin: Address = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "admin"))
            .unwrap();
        assert!(caller == admin || caller == patient, "Unauthorized");

        let device = Device {
            id: device_id,
            device_type,
            patient: patient.clone(),
            caregivers: Vec::new(&env),
        };

        let key = (Symbol::new(&env, "device"), device_id);
        env.storage().persistent().set(&key, &device);
    }

    // Add caregiver to device
    pub fn add_caregiver(env: Env, caller: Address, device_id: u64, caregiver: Address) {
        caller.require_auth();
        let key = (Symbol::new(&env, "device"), device_id);
        let mut device: Device = env.storage().persistent().get(&key).unwrap();
        assert!(caller == device.patient, "Only patient can add caregivers");

        device.caregivers.push_back(caregiver);
        env.storage().persistent().set(&key, &device);
    }

    // Submit vital sign
    pub fn submit_vital_sign(
        env: Env,
        caller: Address,
        patient: Address,
        device_id: u64,
        vital_type: String,
        value: i64,
        unit: String,
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
                    alert_type: String::from_str(&env, "threshold_exceeded"),
                    message: String::from_str(&env, "out of range"), // Simplified, no format
                    timestamp: env.ledger().timestamp(),
                };
                let alert_key = (
                    Symbol::new(&env, "alert"),
                    patient.clone(),
                    env.ledger().sequence(),
                );
                env.storage().persistent().set(&alert_key, &alert);

                // Emit event
                env.events()
                    .publish((Symbol::new(&env, "alert"), patient), alert);
            }
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
    ) {
        caller.require_auth();
        let device_key = (Symbol::new(&env, "device"), 0u64); // Assuming device 0 for simplicity
        let device: Device = env.storage().persistent().get(&device_key).unwrap();
        assert!(
            caller == device.patient || device.caregivers.contains(&caller),
            "Unauthorized"
        );

        let threshold = Threshold {
            vital_type: vital_type.clone(),
            min_value,
            max_value,
        };
        let key = (Symbol::new(&env, "threshold"), patient, vital_type);
        env.storage().persistent().set(&key, &threshold);
    }

    // Get vitals for patient
    pub fn get_vitals(env: Env, _patient: Address) -> Vec<VitalSign> {
        // Simplified: return last few vitals
        // In practice, iterate over storage keys
        Vec::new(&env)
    }

    // Get alerts for patient
    pub fn get_alerts(env: Env, _patient: Address) -> Vec<Alert> {
        // Simplified
        Vec::new(&env)
    }
}
