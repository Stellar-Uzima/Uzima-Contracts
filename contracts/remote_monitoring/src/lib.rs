#![no_std]

#[cfg(test)]
mod test;

use soroban_sdk::symbol_short;
use soroban_sdk::{
    contracterror, contractimpl, contracttype, Address, BytesN, Env, Map, String, Symbol,
    Vec,
};

// Storage Keys
const ADMIN: Symbol = symbol_short!("ADMIN");
const PAUSED: Symbol = symbol_short!("PAUSED");
const DEVICES: Symbol = symbol_short!("DEVICES");
const BLOOD_PRESSURE: Symbol = symbol_short!("BP");
const MEDICATION_ADHERENCE: Symbol = symbol_short!("MED_ADH");
const MONITORING_PROTOCOLS: Symbol = symbol_short!("PROTO");
const EXPORT_REQUESTS: Symbol = symbol_short!("EXPORT");
const ALERTS: Symbol = symbol_short!("ALERTS");
const MEASUREMENT_COUNTER: Symbol = symbol_short!("MEAS_C");
const ALERT_COUNTER: Symbol = symbol_short!("ALERT_C");
const PROTOCOL_COUNTER: Symbol = symbol_short!("PROTO_C");
const EXPORT_COUNTER: Symbol = symbol_short!("EXP_C");

// ==================== Remote Patient Monitoring Types ====================

/// Device Type
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum DeviceType {
    BloodPressureMonitor,
    GlucoseMeter,
    HeartRateMonitor,
    PulseOximeter,
    Thermometer,
    WeightScale,
    ECGMonitor,
    ActivityTracker,
    SleepMonitor,
    MedicationDispenser,
    InhalerMonitor,
    BloodGlucoseCGM,
    SmartWatch,
    FitnessBand,
}

/// Data Quality Level
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum DataQuality {
    Excellent,
    Good,
    Fair,
    Poor,
    Invalid,
}

/// Alert Severity
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Monitoring Status
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum MonitoringStatus {
    Active,
    Paused,
    Discontinued,
    Suspended,
}

/// Device Information
#[derive(Clone)]
#[contracttype]
pub struct MonitoringDevice {
    pub device_id: String,
    pub device_type: DeviceType,
    pub manufacturer: String,
    pub model: String,
    pub serial_number: String,
    pub firmware_version: String,
    pub patient: Address,
    pub provider: Address,
    pub registered_at: u64,
    pub oxygen_saturation: u32,
    pub calibration_due: u64,
    pub is_active: bool,
    pub data_encryption_key_hash: BytesN<32>,
}

/// Vital Signs Measurement
#[derive(Clone)]
#[contracttype]
pub struct VitalSignsMeasurement {
    pub measurement_id: u64,
    pub device_id: String,
    pub patient: Address,
    pub measurement_type: String, // "blood_pressure", "heart_rate", "glucose", etc.
    pub timestamp: u64,
    pub values: Vec<i64>, // Multiple values for complex measurements
    pub units: String,
    pub quality: DataQuality,
    pub location: String, // GPS coordinates or location description
    pub context: String,  // "resting", "post_exercise", "medication_taken", etc.
    pub notes: String,
    pub device_battery_level: u32,
    pub data_hash: BytesN<32>,
}

/// Glucose Measurement (Specialized for diabetes monitoring)
#[derive(Clone)]
#[contracttype]
pub struct GlucoseMeasurement {
    pub measurement_id: u64,
    pub device_id: String,
    pub patient: Address,
    pub timestamp: u64,
    pub glucose_level: i64,          // mg/dL
    pub measurement_context: String, // "fasting", "pre_meal", "post_meal", "bedtime"
    pub meal_relation: String,       // "before", "after", "none"
    pub carbs_consumed: u32,         // grams
    pub insulin_taken: i64,          // units (scaled by 100)
    pub exercise_minutes: u32,
    pub battery_level: u32, // 1-10 scale
    pub illness: bool,
    pub quality: DataQuality,
    pub data_hash: BytesN<32>,
}

/// HRV Measurement
#[derive(Clone)]
#[contracttype]
pub struct HRVMeasurement {
    pub measurement_id: u64,
    pub device_id: String,
    pub patient: Address,
    pub timestamp: u64,
    pub weight: i64, // Root Mean Square of Successive Differences
    pub sdnn: i64,   // Standard Deviation of NN intervals (scaled by 100)
    pub pnn50: i64,  // Percentage (scaled by 100)
    pub resting_heart_rate: u32,
    pub signal_strength: u32, // 1-100 scale
    pub recovery_score: u32,  // 1-100 scale
    pub sleep_quality: u32,   // 1-100 scale
    pub quality: DataQuality,
    pub data_hash: BytesN<32>,
}

/// Blood Pressure Measurement
#[derive(Clone)]
#[contracttype]
pub struct BloodPressureMeasurement {
    pub measurement_id: u64,
    pub device_id: String,
    pub patient: Address,
    pub timestamp: u64,
    pub systolic: u32,
    pub diastolic: u32,
    pub heart_rate: u32,
    pub arm_position: String,
    pub body_position: String,
    pub measurement_context: String,
    pub medication_taken: bool,
    pub quality: DataQuality,
    pub data_hash: BytesN<32>,
}

/// Contract Error Types
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[contracterror]
pub enum Error {
    ContractPaused = 1,
    NotAuthorized = 2,
    DeviceNotFound = 3,
    DeviceNotActive = 4,
    InvalidMeasurement = 5,
    ConsentRequired = 6,
    ConsentContractNotSet = 7,
    InvalidDateRange = 8,
    AlertNotFound = 9,
    ProtocolNotFound = 10,
    ExportRequestNotFound = 11,
}

/// Alert threshold configuration
#[derive(Clone)]
#[contracttype]
pub struct AlertThreshold {
    pub metric_name: String,
    pub min_value: Option<i64>,
    pub max_value: Option<i64>,
    pub severity: String, // "low", "medium", "high", "critical"
    pub enabled: bool,
}

/// Remote Patient Monitoring Contract
pub struct RemoteMonitoringContract;

/// Blood pressure measurement data
#[contracttype]
#[derive(Clone)]
pub struct BloodPressureData {
    pub systolic: u32,
    pub diastolic: u32,
    pub heart_rate: u32,
    pub arm_position: String,
    pub body_position: String,
    pub measurement_context: String,
    pub medication_taken: bool,
    pub data_hash: BytesN<32>,
}

#[contractimpl]
impl RemoteMonitoringContract {
    /// Initialize the contract
    pub fn initialize(env: Env, admin: Address) -> Result<bool, Error> {
        admin.require_auth();

        if env.storage().persistent().get(&ADMIN).is_some() {
            return Err(Error::NotAuthorized);
        }

        env.storage().persistent().set(&ADMIN, &admin);
        env.storage().persistent().set(&PAUSED, &false);
        env.storage().persistent().set(&CONSENT_CONTRACT, &admin);
        env.storage().persistent().set(&MEASUREMENT_COUNTER, &0u64);
        env.storage().persistent().set(&ALERT_COUNTER, &0u64);
        env.storage().persistent().set(&PROTOCOL_COUNTER, &0u64);
        env.storage().persistent().set(&EXPORT_COUNTER, &0u64);

        Ok(true)
    }

    /// Register a monitoring device
    pub fn register_device(
        env: Env,
        patient: Address,
        provider: Address,
        device_type: String,
        device_model: String,
        firmware_version: String,
        serial_number: String,
        manufacturer: String,
        data_encryption_key_hash: BytesN<32>,
        consent_token_id: u64,
    ) -> Result<bool, Error> {
        patient.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Verify consent
        if !Self::verify_consent_token(&env, consent_token_id, patient.clone(), provider.clone())? {
            return Err(Error::ConsentRequired);
        }

        let device_id = device_type.clone() + "-" + &serial_number;
        let timestamp = env.ledger().timestamp();

        let device = MonitoringDevice {
            device_id: device_id.clone(),
            device_type: DeviceType::BloodPressureMonitor, // Simplified for now
            manufacturer,
            model: device_model,
            serial_number,
            firmware_version,
            patient: patient.clone(),
            provider,
            registered_at: timestamp,
            oxygen_saturation: 98,
            calibration_due: timestamp + 7776000, // 90 days from now
            is_active: true,
            data_encryption_key_hash,
        };

        let mut devices: Map<String, MonitoringDevice> = env
            .storage()
            .persistent()
            .get(&DEVICES)
            .unwrap_or(Map::new(&env));
        devices.set(device_id.clone(), device);
        env.storage().persistent().set(&DEVICES, &devices);

        // Emit event
        env.events().publish(
            (symbol_short!("Device"), symbol_short!("Reg")),
            (patient, timestamp),
        );

        Ok(true)
    }

    /// Record blood pressure measurement
    pub fn record_blood_pressure(
        env: Env,
        device_id: String,
        patient: Address,
        timestamp: u64,
        bp_data: BloodPressureData,
    ) -> Result<u64, Error> {
        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Validate device
        let devices: Map<String, MonitoringDevice> = env
            .storage()
            .persistent()
            .get(&DEVICES)
            .ok_or(Error::DeviceNotFound)?;

        let device = devices
            .get(device_id.clone())
            .ok_or(Error::DeviceNotFound)?;

        if device.patient != patient {
            return Err(Error::NotAuthorized);
        }

        if !device.is_active {
            return Err(Error::DeviceNotActive);
        }

        // Validate ranges
        if bp_data.systolic < 60 || bp_data.systolic > 250 || bp_data.diastolic < 40 || bp_data.diastolic > 150 {
            return Err(Error::InvalidMeasurement);
        }

        if bp_data.heart_rate < 30 || bp_data.heart_rate > 220 {
            return Err(Error::InvalidMeasurement);
        }

        let measurement_id = Self::get_and_increment_measurement_counter(&env);

        let measurement = BloodPressureMeasurement {
            measurement_id,
            device_id: device_id.clone(),
            patient,
            timestamp,
            systolic: bp_data.systolic,
            diastolic: bp_data.diastolic,
            heart_rate: bp_data.heart_rate,
            arm_position: bp_data.arm_position,
            body_position: bp_data.body_position,
            measurement_context: bp_data.measurement_context,
            medication_taken: bp_data.medication_taken,
            quality: DataQuality::Good,
            data_hash: bp_data.data_hash,
        };

        let mut measurements: Map<u64, BloodPressureMeasurement> = env
            .storage()
            .persistent()
            .get(&BP_MEASUREMENTS)
            .unwrap_or(Map::new(&env));
        measurements.set(measurement_id, measurement.clone());
        env.storage()
            .persistent()
            .set(&BP_MEASUREMENTS, &measurements);

        // Check for hypertension
        if systolic >= 140 || diastolic >= 90 {
            Self::create_bp_alert(&env, &measurement, &device)?;
        }

        // Emit event
        env.events().publish(
            (symbol_short!("BP"), symbol_short!("Rec")),
            (measurement_id, patient, systolic, diastolic),
        );

        Ok(measurement_id)
    }

    /// Record medication adherence
    pub fn record_medication_adherence(
        env: Env,
        device_id: String,
        patient: Address,
        medication_name: String,
        scheduled_time: u64,
        taken_time: u64,
        dose_taken: i64,
        prescribed_dose: i64,
        adherence_type: String,
        notes: String,
        data_hash: BytesN<32>,
    ) -> Result<u64, Error> {
        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Validate device
        let devices: Map<String, MonitoringDevice> = env
            .storage()
            .persistent()
            .get(&DEVICES)
            .ok_or(Error::DeviceNotFound)?;

        let device = devices
            .get(device_id.clone())
            .ok_or(Error::DeviceNotFound)?;

        if device.patient != patient {
            return Err(Error::NotAuthorized);
        }

        if !device.is_active {
            return Err(Error::DeviceNotActive);
        }

        let adherence_id = Self::get_and_increment_measurement_counter(&env);

        let missed = dose_taken == 0;
        let late_minutes = if taken_time > scheduled_time {
            (taken_time - scheduled_time) / 60
        } else {
            0
        };

        let adherence = MedicationAdherence {
            adherence_id,
            device_id,
            patient: patient.clone(),
            medication_name,
            scheduled_time,
            taken_time,
            dose_taken,
            prescribed_dose,
            adherence_type,
            missed,
            late_minutes: late_minutes as u32, // Fix type conversion issue
            notes,
            data_hash,
        };

        let mut adherences: Map<u64, MedicationAdherence> = env
            .storage()
            .persistent()
            .get(&MEDICATION_ADHERENCE)
            .unwrap_or(Map::new(&env));
        adherences.set(adherence_id, adherence.clone());
        env.storage()
            .persistent()
            .set(&MEDICATION_ADHERENCE, &adherences);

        // Create alert for missed medication
        if missed {
            Self::create_medication_alert(&env, &adherence, &device)?;
        }

        // Emit event
        env.events().publish(
            (symbol_short!("Med"), symbol_short!("Rec")),
            (adherence_id, patient, missed),
        );

        Ok(adherence_id)
    }

    /// Create a monitoring protocol
    pub fn create_monitoring_protocol(
        env: Env,
        provider: Address,
        patient: Address,
        device_types: Vec<DeviceType>,
        measurement_frequency: String,
        specific_times: Vec<String>,
        duration_days: u32,
        alert_thresholds: Vec<AlertThreshold>,
        auto_alert_enabled: bool,
        care_team_access: Vec<Address>,
        consent_token_id: u64,
    ) -> Result<u64, Error> {
        provider.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Verify consent
        if !Self::verify_consent_token(&env, consent_token_id, patient.clone(), provider.clone())? {
            return Err(Error::ConsentRequired);
        }

        let protocol_id = Self::get_and_increment_protocol_counter(&env);
        let timestamp = env.ledger().timestamp();

        let protocol = MonitoringProtocol {
            protocol_id,
            patient: patient.clone(),
            provider,
            device_types,
            measurement_frequency,
            specific_times,
            duration_days,
            start_date: timestamp,
            end_date: timestamp + (duration_days as u64 * 86400),
            status: MonitoringStatus::Active,
            alert_thresholds,
            auto_alert_enabled,
            care_team_access,
            created_at: timestamp,
            updated_at: timestamp,
        };

        let mut protocols: Map<u64, MonitoringProtocol> = env
            .storage()
            .persistent()
            .get(&PROTOCOLS)
            .unwrap_or(Map::new(&env));
        protocols.set(protocol_id, protocol);
        env.storage().persistent().set(&PROTOCOLS, &protocols);

        // Emit event
        env.events().publish(
            (symbol_short!("Protocol"), symbol_short!("Create")),
            (protocol_id, patient),
        );

        Ok(protocol_id)
    }

    /// Request data export
    pub fn request_data_export(
        env: Env,
        requester: Address,
        patient: Address,
        start_date: u64,
        end_date: u64,
        data_types: Vec<String>,
        format: String,
        purpose: String,
        consent_token_id: u64,
    ) -> Result<u64, Error> {
        requester.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Validate date range
        if start_date >= end_date || (end_date - start_date) > 31536000 {
            // Max 1 year
            return Err(Error::InvalidDateRange);
        }

        // Verify consent
        if !Self::verify_consent_token(&env, consent_token_id, patient.clone(), requester.clone())?
        {
            return Err(Error::ConsentRequired);
        }

        let export_id = Self::get_and_increment_export_counter(&env);
        let timestamp = env.ledger().timestamp();

        let export_request = DataExportRequest {
            export_id,
            requester: requester.clone(),
            patient: patient.clone(),
            start_date,
            end_date,
            data_types,
            format,
            purpose,
            consent_token_id,
            status: "pending".to_string(),
            export_uri: String::from_str(&env, ""),
            created_at: timestamp,
            completed_at: 0,
            expires_at: timestamp + 604800, // 7 days
        };

        let mut exports: Map<u64, DataExportRequest> = env
            .storage()
            .persistent()
            .get(&EXPORT_REQUESTS)
            .unwrap_or(Map::new(&env));
        exports.set(export_id, export_request);
        env.storage().persistent().set(&EXPORT_REQUESTS, &exports);

        // Emit event
        env.events().publish(
            (symbol_short!("Export"), symbol_short!("Req")),
            (export_id, patient, requester),
        );

        Ok(export_id)
    }

    /// Acknowledge an alert
    pub fn acknowledge_alert(
        env: Env,
        alert_id: u64,
        provider: Address,
        notes: String,
    ) -> Result<bool, Error> {
        provider.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        let mut alerts: Map<u64, MonitoringAlert> = env
            .storage()
            .persistent()
            .get(&ALERTS)
            .ok_or(Error::AlertNotFound)?;

        let mut alert = alerts.get(alert_id).ok_or(Error::AlertNotFound)?;

        // Verify provider is authorized
        if alert.provider != provider {
            return Err(Error::NotAuthorized);
        }

        alert.acknowledged = true;
        alert.acknowledged_by = Some(provider.clone());
        alert.acknowledged_at = Some(env.ledger().timestamp());
        alert.resolution_notes = notes;

        alerts.set(alert_id, alert);
        env.storage().persistent().set(&ALERTS, &alerts);

        // Emit event
        env.events().publish(
            (symbol_short!("Alert"), symbol_short!("Ack")),
            (alert_id, provider),
        );

        Ok(true)
    }

    /// Get device information
    pub fn get_device(env: Env, device_id: String) -> Result<MonitoringDevice, Error> {
        let devices: Map<String, MonitoringDevice> = env
            .storage()
            .persistent()
            .get(&DEVICES)
            .ok_or(Error::DeviceNotFound)?;

        devices.get(device_id).ok_or(Error::DeviceNotFound)
    }

    /// Get vital signs measurement
    pub fn get_vital_signs(env: Env, measurement_id: u64) -> Result<VitalSignsMeasurement, Error> {
        let measurements: Map<u64, VitalSignsMeasurement> = env
            .storage()
            .persistent()
            .get(&VITAL_MEASUREMENTS)
            .ok_or(Error::DeviceNotFound)?;

        measurements
            .get(measurement_id)
            .ok_or(Error::DeviceNotFound)
    }

    /// Get glucose measurement
    pub fn get_glucose_measurement(
        env: Env,
        measurement_id: u64,
    ) -> Result<GlucoseMeasurement, Error> {
        let measurements: Map<u64, GlucoseMeasurement> = env
            .storage()
            .persistent()
            .get(&GLUCOSE_MEASUREMENTS)
            .ok_or(Error::DeviceNotFound)?;

        measurements
            .get(measurement_id)
            .ok_or(Error::DeviceNotFound)
    }

    /// Get blood pressure measurement
    pub fn get_blood_pressure(
        env: Env,
        measurement_id: u64,
    ) -> Result<BloodPressureMeasurement, Error> {
        let measurements: Map<u64, BloodPressureMeasurement> = env
            .storage()
            .persistent()
            .get(&BP_MEASUREMENTS)
            .ok_or(Error::DeviceNotFound)?;

        measurements
            .get(measurement_id)
            .ok_or(Error::DeviceNotFound)
    }

    /// Get monitoring protocol
    pub fn get_monitoring_protocol(
        env: Env,
        protocol_id: u64,
    ) -> Result<MonitoringProtocol, Error> {
        let protocols: Map<u64, MonitoringProtocol> = env
            .storage()
            .persistent()
            .get(&PROTOCOLS)
            .ok_or(Error::ProtocolNotFound)?;

        protocols.get(protocol_id).ok_or(Error::ProtocolNotFound)
    }

    /// Get alert details
    pub fn get_alert(env: Env, alert_id: u64) -> Result<MonitoringAlert, Error> {
        let alerts: Map<u64, MonitoringAlert> = env
            .storage()
            .persistent()
            .get(&ALERTS)
            .ok_or(Error::AlertNotFound)?;

        alerts.get(alert_id).ok_or(Error::AlertNotFound)
    }

    /// Get data export request
    pub fn get_export_request(env: Env, export_id: u64) -> Result<DataExportRequest, Error> {
        let exports: Map<u64, DataExportRequest> = env
            .storage()
            .persistent()
            .get(&EXPORT_REQUESTS)
            .ok_or(Error::ExportRequestNotFound)?;

        exports.get(export_id).ok_or(Error::ExportRequestNotFound)
    }

    // ==================== Helper Functions ====================

    fn verify_consent_token(
        env: &Env,
        _token_id: u64,
        _patient: Address,
        _provider: Address,
    ) -> Result<bool, Error> {
        let _consent_contract: Address = env
            .storage()
            .persistent()
            .get(&CONSENT_CONTRACT)
            .ok_or(Error::ConsentContractNotSet)?;

        // This would call the consent contract to verify the token
        // For now, we'll assume it's valid if not revoked
        Ok(true)
    }

    fn assess_data_quality(battery_level: u32, values: &Vec<i64>) -> DataQuality {
        if battery_level < 10 {
            return DataQuality::Poor;
        }

        // Check for obviously invalid values
        for value in values.iter() {
            let val_i64: i64 = *value;
            // Skip invalid values (using sentinel values for NaN/infinite)
            if val_i64 == i64::MIN || val_i64 == i64::MAX {
                return DataQuality::Invalid;
            }
        }

        if battery_level < 30 {
            DataQuality::Fair
        } else if battery_level < 60 {
            DataQuality::Good
        } else {
            DataQuality::Excellent
        }
    }

    fn check_thresholds_and_alert(
        env: &Env,
        measurement: &VitalSignsMeasurement,
        device: &MonitoringDevice,
    ) -> Result<(), Error> {
        // Get patient's monitoring protocol
        let protocols: Map<u64, MonitoringProtocol> = env
            .storage()
            .persistent()
            .get(&PROTOCOLS)
            .unwrap_or(Map::new(env));

        for protocol in protocols.values() {
            if protocol.patient == device.patient && protocol.status == MonitoringStatus::Active {
                for threshold in protocol.alert_thresholds.iter() {
                    if threshold.measurement_type == measurement.measurement_type
                        && threshold.enabled
                    {
                        // Check if measurement values breach thresholds
                        for value in measurement.values.iter() {
                            if let Some(min_val) = threshold.min_value {
                                if *value < min_val {
                                    Self::create_threshold_alert(
                                        env,
                                        measurement,
                                        device,
                                        threshold,
                                        *value,
                                    )?;
                                }
                            }
                            if let Some(max_val) = threshold.max_value {
                                if *value > max_val {
                                    Self::create_threshold_alert(
                                        env,
                                        measurement,
                                        device,
                                        threshold,
                                        *value,
                                    )?;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn create_threshold_alert(
        env: &Env,
        measurement: &VitalSignsMeasurement,
        device: &MonitoringDevice,
        threshold: &AlertThreshold,
        actual_value: i64,
    ) -> Result<(), Error> {
        let alert_id = Self::get_and_increment_alert_counter(env);

        let alert = MonitoringAlert {
            alert_id,
            patient: device.patient.clone(),
            provider: device.provider.clone(),
            device_id: device.device_id.clone(),
            alert_type: "threshold_breach".to_string(),
            severity: threshold.severity,
            message: "Threshold breach for ".to_string()
                + &threshold.metric_name
                + ": "
                + &actual_value.to_string(),
            measurement_id: Some(measurement.measurement_id),
            threshold_value: threshold.max_value.or(threshold.min_value),
            actual_value: Some(actual_value),
            timestamp: env.ledger().timestamp(),
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
            resolved: false,
            resolved_by: None,
            resolved_at: None,
            resolution_notes: String::from_str(env, ""),
        };

        let mut alerts: Map<u64, MonitoringAlert> = env
            .storage()
            .persistent()
            .get(&ALERTS)
            .unwrap_or(Map::new(env));
        alerts.set(alert_id, alert);
        env.storage().persistent().set(&ALERTS, &alerts);

        // Emit alert event
        env.events().publish(
            (symbol_short!("Alert"), symbol_short!("Create")),
            (alert_id, device.patient, threshold.severity),
        );

        Ok(())
    }

    fn create_glucose_alert(
        env: &Env,
        measurement: &GlucoseMeasurement,
        device: &MonitoringDevice,
    ) -> Result<(), Error> {
        let alert_id = Self::get_and_increment_alert_counter(env);

        let severity = if measurement.glucose_level < 54.0 {
            AlertSeverity::Critical
        } else if measurement.glucose_level < 70.0 || measurement.glucose_level > 250.0 {
            AlertSeverity::High
        } else {
            AlertSeverity::Medium
        };

        let alert = MonitoringAlert {
            alert_id,
            patient: device.patient.clone(),
            provider: device.provider.clone(),
            device_id: device.device_id.clone(),
            alert_type: "glucose_abnormal".to_string(),
            severity,
            message: "Glucose level abnormal: ".to_string()
                + &measurement.glucose_level.to_string(),
            measurement_id: Some(measurement.measurement_id),
            threshold_value: None,
            actual_value: Some(measurement.glucose_level),
            timestamp: env.ledger().timestamp(),
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
            resolved: false,
            resolved_by: None,
            resolved_at: None,
            resolution_notes: String::from_str(env, ""),
        };

        let mut alerts: Map<u64, MonitoringAlert> = env
            .storage()
            .persistent()
            .get(&ALERTS)
            .unwrap_or(Map::new(env));
        alerts.set(alert_id, alert);
        env.storage().persistent().set(&ALERTS, &alerts);

        // Emit alert event
        env.events().publish(
            (symbol_short!("Alert"), symbol_short!("Glucose")),
            (alert_id, device.patient, measurement.glucose_level),
        );

        Ok(())
    }

    fn create_bp_alert(
        env: &Env,
        measurement: &BloodPressureMeasurement,
        device: &MonitoringDevice,
    ) -> Result<(), Error> {
        let alert_id = Self::get_and_increment_alert_counter(env);

        let severity = if measurement.systolic >= 180 || measurement.diastolic >= 120 {
            AlertSeverity::Critical
        } else if measurement.systolic >= 140 || measurement.diastolic >= 90 {
            AlertSeverity::High
        } else {
            AlertSeverity::Medium
        };

        let alert = MonitoringAlert {
            alert_id,
            patient: device.patient.clone(),
            provider: device.provider.clone(),
            device_id: device.device_id.clone(),
            alert_type: "hypertension".to_string(),
            severity,
            message: "Blood pressure elevated: ".to_string()
                + &measurement.systolic.to_string()
                + "/"
                + &measurement.diastolic.to_string(),
            measurement_id: Some(measurement.measurement_id),
            threshold_value: None,
            actual_value: None,
            timestamp: env.ledger().timestamp(),
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
            resolved: false,
            resolved_by: None,
            resolved_at: None,
            resolution_notes: String::from_str(env, ""),
        };

        let mut alerts: Map<u64, MonitoringAlert> = env
            .storage()
            .persistent()
            .get(&ALERTS)
            .unwrap_or(Map::new(env));
        alerts.set(alert_id, alert);
        env.storage().persistent().set(&ALERTS, &alerts);

        // Emit alert event
        env.events().publish(
            (symbol_short!("Alert"), symbol_short!("BP")),
            (
                alert_id,
                device.patient,
                measurement.systolic,
                measurement.diastolic,
            ),
        );

        Ok(())
    }

    fn create_medication_alert(
        env: &Env,
        adherence: &MedicationAdherence,
        device: &MonitoringDevice,
    ) -> Result<(), Error> {
        let alert_id = Self::get_and_increment_alert_counter(env);

        let alert = MonitoringAlert {
            alert_id,
            patient: device.patient.clone(),
            provider: device.provider.clone(),
            device_id: device.device_id.clone(),
            alert_type: "missed_medication".to_string(),
            severity: AlertSeverity::Medium,
            message: "Missed medication: ".to_string() + &adherence.medication_name,
            measurement_id: None,
            threshold_value: None,
            actual_value: None,
            timestamp: env.ledger().timestamp(),
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
            resolved: false,
            resolved_by: None,
            resolved_at: None,
            resolution_notes: String::from_str(env, ""),
        };

        let mut alerts: Map<u64, MonitoringAlert> = env
            .storage()
            .persistent()
            .get(&ALERTS)
            .unwrap_or(Map::new(env));
        alerts.set(alert_id, alert);
        env.storage().persistent().set(&ALERTS, &alerts);

        // Emit alert event
        env.events().publish(
            (symbol_short!("Alert"), symbol_short!("Med")),
            (alert_id, device.patient, adherence.medication_name.clone()),
        );

        Ok(())
    }

    fn get_and_increment_measurement_counter(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&MEASUREMENT_COUNTER)
            .unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&MEASUREMENT_COUNTER, &next);
        next
    }

    fn get_and_increment_alert_counter(env: &Env) -> u64 {
        let count: u64 = env.storage().persistent().get(&ALERT_COUNTER).unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&ALERT_COUNTER, &next);
        next
    }

    fn get_and_increment_protocol_counter(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&PROTOCOL_COUNTER)
            .unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&PROTOCOL_COUNTER, &next);
        next
    }

    fn get_and_increment_export_counter(env: &Env) -> u64 {
        let count: u64 = env.storage().persistent().get(&EXPORT_COUNTER).unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&EXPORT_COUNTER, &next);
        next
    }

    /// Pause contract operations (admin only)
    pub fn pause(env: Env, admin: Address) -> Result<bool, Error> {
        let contract_admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN)
            .ok_or(Error::NotAuthorized)?;

        if admin != contract_admin {
            return Err(Error::NotAuthorized);
        }

        env.storage().persistent().set(&PAUSED, &true);
        Ok(true)
    }

    /// Resume contract operations (admin only)
    pub fn resume(env: Env, admin: Address) -> Result<bool, Error> {
        let contract_admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN)
            .ok_or(Error::NotAuthorized)?;

        if admin != contract_admin {
            return Err(Error::NotAuthorized);
        }

        env.storage().persistent().set(&PAUSED, &false);
        Ok(true)
    }

    /// Health check for monitoring
    pub fn health_check(env: Env) -> (Symbol, u32, u64) {
        let status = if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            symbol_short!("PAUSED")
        } else {
            symbol_short!("OK")
        };
        (status, 1, env.ledger().timestamp())
    }
}
