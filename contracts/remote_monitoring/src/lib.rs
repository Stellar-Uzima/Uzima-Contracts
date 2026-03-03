#![no_std]

#[cfg(test)]
mod test;

use soroban_sdk::symbol_short;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, vec, Address, BytesN, Env, Map, String,
    Symbol, Vec,
};

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
    pub last_calibration: u64,
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
    pub values: Vec<f32>, // Multiple values for complex measurements
    pub units: String,
    pub quality: DataQuality,
    pub location: String, // GPS coordinates or location description
    pub context: String, // "resting", "post_exercise", "medication_taken", etc.
    pub notes: String,
    pub device_battery_level: u8,
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
    pub glucose_level: f32, // mg/dL
    pub measurement_context: String, // "fasting", "pre_meal", "post_meal", "bedtime"
    pub meal_relation: String, // "before", "after", "none"
    pub carbs_consumed: u32, // grams
    pub insulin_taken: f32, // units
    pub exercise_minutes: u32,
    pub stress_level: u8, // 1-10 scale
    pub illness: bool,
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
    pub systolic: u16,
    pub diastolic: u16,
    pub heart_rate: u16,
    pub arm_position: String, // "left_upper", "right_upper", etc.
    pub body_position: String, // "sitting", "standing", "lying"
    pub measurement_context: String, // "resting", "post_exercise", etc.
    pub medication_taken: bool,
    pub quality: DataQuality,
    pub data_hash: BytesN<32>,
}

/// Heart Rate Variability (HRV) Data
#[derive(Clone)]
#[contracttype]
pub struct HRVMeasurement {
    pub measurement_id: u64,
    pub device_id: String,
    pub patient: Address,
    pub timestamp: u64,
    pub rmssd: f32, // Root Mean Square of Successive Differences
    pub sdnn: f32,  // Standard Deviation of NN intervals
    pub pnn50: f32, // Percentage of successive NN intervals that differ by more than 50ms
    pub resting_heart_rate: u16,
    pub stress_score: u8, // 1-100 scale
    pub recovery_score: u8, // 1-100 scale
    pub sleep_quality: u8, // 1-100 scale
    pub quality: DataQuality,
    pub data_hash: BytesN<32>,
}

/// Activity and Sleep Data
#[derive(Clone)]
#[contracttype]
pub struct ActivitySleepData {
    pub measurement_id: u64,
    pub device_id: String,
    pub patient: Address,
    pub date: u64, // Unix timestamp for start of day
    pub steps_count: u32,
    pub active_minutes: u32,
    pub calories_burned: u32,
    pub distance_meters: u32,
    pub floors_climbed: u16,
    pub sleep_duration_minutes: u32,
    pub deep_sleep_minutes: u32,
    pub rem_sleep_minutes: u32,
    pub light_sleep_minutes: u32,
    pub sleep_efficiency: u8, // percentage
    pub awake_minutes: u32,
    pub quality: DataQuality,
    pub data_hash: BytesN<32>,
}

/// Medication Adherence Data
#[derive(Clone)]
#[contracttype]
pub struct MedicationAdherence {
    pub adherence_id: u64,
    pub device_id: String,
    pub patient: Address,
    pub medication_name: String,
    pub scheduled_time: u64,
    pub taken_time: u64,
    pub dose_taken: f32,
    pub prescribed_dose: f32,
    pub adherence_type: String, // "pill", "inhaler", "injection", "liquid"
    pub missed: bool,
    pub late_minutes: u32,
    pub notes: String,
    pub data_hash: BytesN<32>,
}

/// Monitoring Alert
#[derive(Clone)]
#[contracttype]
pub struct MonitoringAlert {
    pub alert_id: u64,
    pub patient: Address,
    pub provider: Address,
    pub device_id: String,
    pub alert_type: String, // "threshold_breach", "device_offline", "missed_medication", etc.
    pub severity: AlertSeverity,
    pub message: String,
    pub measurement_id: Option<u64>,
    pub threshold_value: Option<f32>,
    pub actual_value: Option<f32>,
    pub timestamp: u64,
    pub acknowledged: bool,
    pub acknowledged_by: Option<Address>,
    pub acknowledged_at: Option<u64>,
    pub resolved: bool,
    pub resolved_by: Option<Address>,
    pub resolved_at: Option<u64>,
    pub resolution_notes: String,
}

/// Monitoring Protocol
#[derive(Clone)]
#[contracttype]
pub struct MonitoringProtocol {
    pub protocol_id: u64,
    pub patient: Address,
    pub provider: Address,
    pub device_types: Vec<DeviceType>,
    pub measurement_frequency: String, // "hourly", "daily", "weekly", "as_needed"
    pub specific_times: Vec<String>, // Times of day for measurements
    pub duration_days: u32,
    pub start_date: u64,
    pub end_date: u64,
    pub status: MonitoringStatus,
    pub alert_thresholds: Vec<AlertThreshold>,
    pub auto_alert_enabled: bool,
    pub care_team_access: Vec<Address>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Alert Threshold Configuration
#[derive(Clone)]
#[contracttype]
pub struct AlertThreshold {
    pub measurement_type: String,
    pub min_value: Option<f32>,
    pub max_value: Option<f32>,
    pub severity: AlertSeverity,
    pub enabled: bool,
    pub notification_delay_minutes: u32,
}

/// Data Export Request
#[derive(Clone)]
#[contracttype]
pub struct DataExportRequest {
    pub export_id: u64,
    pub requester: Address,
    pub patient: Address,
    pub start_date: u64,
    pub end_date: u64,
    pub data_types: Vec<String>,
    pub format: String, // "json", "csv", "fhir", "hl7"
    pub purpose: String,
    pub consent_token_id: u64,
    pub status: String, // "pending", "processing", "completed", "failed"
    pub export_uri: String,
    pub created_at: u64,
    pub completed_at: u64,
    pub expires_at: u64,
}

// Storage Keys
const ADMIN: Symbol = symbol_short!("ADMIN");
const DEVICES: Symbol = symbol_short!("DEVICES");
const VITAL_MEASUREMENTS: Symbol = symbol_short!("VITALS");
const GLUCOSE_MEASUREMENTS: Symbol = symbol_short!("GLUCOSE");
const BP_MEASUREMENTS: Symbol = symbol_short!("BLOOD_PRESSURE");
const HRV_MEASUREMENTS: Symbol = symbol_short!("HRV");
const ACTIVITY_SLEEP: Symbol = symbol_short!("ACTIVITY");
const MEDICATION_ADHERENCE: Symbol = symbol_short!("MED_ADH");
const ALERTS: Symbol = symbol_short!("ALERTS");
const PROTOCOLS: Symbol = symbol_short!("PROTOCOLS");
const EXPORT_REQUESTS: Symbol = symbol_short!("EXPORTS");
const MEASUREMENT_COUNTER: Symbol = symbol_short!("MEAS_CNT");
const ALERT_COUNTER: Symbol = symbol_short!("ALERT_CNT");
const PROTOCOL_COUNTER: Symbol = symbol_short!("PROT_CNT");
const EXPORT_COUNTER: Symbol = symbol_short!("EXP_CNT");
const PAUSED: Symbol = symbol_short!("PAUSED");
const CONSENT_CONTRACT: Symbol = symbol_short!("CONSENT");
const MEDICAL_RECORDS_CONTRACT: Symbol = symbol_short!("MEDICAL");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    ContractPaused = 2,
    DeviceNotFound = 3,
    DeviceAlreadyExists = 4,
    PatientNotFound = 5,
    InvalidMeasurement = 6,
    InvalidDeviceType = 7,
    DataQualityTooLow = 8,
    ThresholdBreach = 9,
    AlertNotFound = 10,
    ProtocolNotFound = 11,
    ProtocolAlreadyExists = 12,
    ExportRequestNotFound = 13,
    ConsentRequired = 14,
    ConsentRevoked = 15,
    InvalidDateRange = 16,
    InvalidFrequency = 17,
    DeviceNotActive = 18,
    MeasurementTooOld = 19,
    DuplicateMeasurement = 20,
    InvalidThreshold = 21,
    ExportFailed = 22,
    MedicalRecordsContractNotSet = 23,
    ConsentContractNotSet = 24,
}

#[contract]
pub struct RemoteMonitoringContract;

#[contractimpl]
impl RemoteMonitoringContract {
    /// Initialize the remote monitoring contract
    pub fn initialize(
        env: Env,
        admin: Address,
        consent_contract: Address,
        medical_records_contract: Address,
    ) -> Result<bool, Error> {
        admin.require_auth();

        if env.storage().persistent().has(&ADMIN) {
            return Err(Error::DeviceAlreadyExists);
        }

        env.storage().persistent().set(&ADMIN, &admin);
        env.storage().persistent().set(&CONSENT_CONTRACT, &consent_contract);
        env.storage()
            .persistent()
            .set(&MEDICAL_RECORDS_CONTRACT, &medical_records_contract);
        env.storage().persistent().set(&PAUSED, &false);
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
        device_id: String,
        device_type: DeviceType,
        manufacturer: String,
        model: String,
        serial_number: String,
        firmware_version: String,
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

        let mut devices: Map<String, MonitoringDevice> = env
            .storage()
            .persistent()
            .get(&DEVICES)
            .unwrap_or(Map::new(&env));

        if devices.contains_key(device_id.clone()) {
            return Err(Error::DeviceAlreadyExists);
        }

        let timestamp = env.ledger().timestamp();
        let device = MonitoringDevice {
            device_id: device_id.clone(),
            device_type,
            manufacturer,
            model,
            serial_number,
            firmware_version,
            patient: patient.clone(),
            provider,
            registered_at: timestamp,
            last_calibration: timestamp,
            calibration_due: timestamp + 7776000, // 90 days from now
            is_active: true,
            data_encryption_key_hash,
        };

        devices.set(device_id, device);
        env.storage().persistent().set(&DEVICES, &devices);

        // Emit event
        env.events().publish(
            (symbol_short!("Device"), symbol_short!("Registered")),
            (patient, timestamp),
        );

        Ok(true)
    }

    /// Record vital signs measurement
    pub fn record_vital_signs(
        env: Env,
        device_id: String,
        patient: Address,
        measurement_type: String,
        timestamp: u64,
        values: Vec<f32>,
        units: String,
        location: String,
        context: String,
        notes: String,
        device_battery_level: u8,
        data_hash: BytesN<32>,
    ) -> Result<u64, Error> {
        // This would typically be called by an authorized device or data ingestion service
        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Validate device exists and is active
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

        // Validate timestamp is not too old (max 24 hours)
        let current_time = env.ledger().timestamp();
        if current_time > timestamp && (current_time - timestamp) > 86400 {
            return Err(Error::MeasurementTooOld);
        }

        // Validate data quality based on battery level and other factors
        let quality = Self::assess_data_quality(device_battery_level, &values);

        let measurement_id = Self::get_and_increment_measurement_counter(&env);

        let measurement = VitalSignsMeasurement {
            measurement_id,
            device_id,
            patient: patient.clone(),
            measurement_type: measurement_type.clone(),
            timestamp,
            values,
            units,
            quality,
            location,
            context,
            notes,
            device_battery_level,
            data_hash,
        };

        let mut measurements: Map<u64, VitalSignsMeasurement> = env
            .storage()
            .persistent()
            .get(&VITAL_MEASUREMENTS)
            .unwrap_or(Map::new(&env));
        measurements.set(measurement_id, measurement.clone());
        env.storage()
            .persistent()
            .set(&VITAL_MEASUREMENTS, &measurements);

        // Check for threshold breaches and create alerts
        Self::check_thresholds_and_alert(&env, &measurement, &device)?;

        // Emit event
        env.events().publish(
            (symbol_short!("Vitals"), symbol_short!("Recorded")),
            (measurement_id, patient, measurement_type),
        );

        Ok(measurement_id)
    }

    /// Record glucose measurement (specialized for diabetes)
    pub fn record_glucose_measurement(
        env: Env,
        device_id: String,
        patient: Address,
        timestamp: u64,
        glucose_level: f32,
        measurement_context: String,
        meal_relation: String,
        carbs_consumed: u32,
        insulin_taken: f32,
        exercise_minutes: u32,
        stress_level: u8,
        illness: bool,
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

        // Validate glucose range (20-600 mg/dL)
        if glucose_level < 20.0 || glucose_level > 600.0 {
            return Err(Error::InvalidMeasurement);
        }

        let measurement_id = Self::get_and_increment_measurement_counter(&env);

        let measurement = GlucoseMeasurement {
            measurement_id,
            device_id,
            patient: patient.clone(),
            timestamp,
            glucose_level,
            measurement_context,
            meal_relation,
            carbs_consumed,
            insulin_taken,
            exercise_minutes,
            stress_level,
            illness,
            quality: DataQuality::Good, // Would assess based on device and conditions
            data_hash,
        };

        let mut measurements: Map<u64, GlucoseMeasurement> = env
            .storage()
            .persistent()
            .get(&GLUCOSE_MEASUREMENTS)
            .unwrap_or(Map::new(&env));
        measurements.set(measurement_id, measurement.clone());
        env.storage()
            .persistent()
            .set(&GLUCOSE_MEASUREMENTS, &measurements);

        // Check for critical glucose levels
        if glucose_level < 70.0 || glucose_level > 250.0 {
            Self::create_glucose_alert(&env, &measurement, &device)?;
        }

        // Emit event
        env.events().publish(
            (symbol_short!("Glucose"), symbol_short!("Recorded")),
            (measurement_id, patient, glucose_level),
        );

        Ok(measurement_id)
    }

    /// Record blood pressure measurement
    pub fn record_blood_pressure(
        env: Env,
        device_id: String,
        patient: Address,
        timestamp: u64,
        systolic: u16,
        diastolic: u16,
        heart_rate: u16,
        arm_position: String,
        body_position: String,
        measurement_context: String,
        medication_taken: bool,
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

        // Validate ranges
        if systolic < 60 || systolic > 250 || diastolic < 40 || diastolic > 150 {
            return Err(Error::InvalidMeasurement);
        }

        if heart_rate < 30 || heart_rate > 220 {
            return Err(Error::InvalidMeasurement);
        }

        let measurement_id = Self::get_and_increment_measurement_counter(&env);

        let measurement = BloodPressureMeasurement {
            measurement_id,
            device_id,
            patient: patient.clone(),
            timestamp,
            systolic,
            diastolic,
            heart_rate,
            arm_position,
            body_position,
            measurement_context,
            medication_taken,
            quality: DataQuality::Good,
            data_hash,
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
            (symbol_short!("BloodPressure"), symbol_short!("Recorded")),
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
        dose_taken: f32,
        prescribed_dose: f32,
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

        let missed = dose_taken == 0.0;
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
            late_minutes,
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
            (symbol_short!("Medication"), symbol_short!("Recorded")),
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
            (symbol_short!("Protocol"), symbol_short!("Created")),
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
        if start_date >= end_date || (end_date - start_date) > 31536000 { // Max 1 year
            return Err(Error::InvalidDateRange);
        }

        // Verify consent
        if !Self::verify_consent_token(&env, consent_token_id, patient.clone(), requester.clone())? {
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
            (symbol_short!("Export"), symbol_short!("Requested")),
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

        let mut alert = alerts
            .get(alert_id)
            .ok_or(Error::AlertNotFound)?;

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
            (symbol_short!("Alert"), symbol_short!("Acknowledged")),
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

        measurements.get(measurement_id).ok_or(Error::DeviceNotFound)
    }

    /// Get glucose measurement
    pub fn get_glucose_measurement(env: Env, measurement_id: u64) -> Result<GlucoseMeasurement, Error> {
        let measurements: Map<u64, GlucoseMeasurement> = env
            .storage()
            .persistent()
            .get(&GLUCOSE_MEASUREMENTS)
            .ok_or(Error::DeviceNotFound)?;

        measurements.get(measurement_id).ok_or(Error::DeviceNotFound)
    }

    /// Get blood pressure measurement
    pub fn get_blood_pressure(env: Env, measurement_id: u64) -> Result<BloodPressureMeasurement, Error> {
        let measurements: Map<u64, BloodPressureMeasurement> = env
            .storage()
            .persistent()
            .get(&BP_MEASUREMENTS)
            .ok_or(Error::DeviceNotFound)?;

        measurements.get(measurement_id).ok_or(Error::DeviceNotFound)
    }

    /// Get monitoring protocol
    pub fn get_monitoring_protocol(env: Env, protocol_id: u64) -> Result<MonitoringProtocol, Error> {
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
        token_id: u64,
        patient: Address,
        provider: Address,
    ) -> Result<bool, Error> {
        let consent_contract: Address = env
            .storage()
            .persistent()
            .get(&CONSENT_CONTRACT)
            .ok_or(Error::ConsentContractNotSet)?;

        // This would call the consent contract to verify the token
        // For now, we'll assume it's valid if not revoked
        Ok(true)
    }

    fn assess_data_quality(battery_level: u8, values: &Vec<f32>) -> DataQuality {
        if battery_level < 10 {
            return DataQuality::Poor;
        }

        // Check for obviously invalid values
        for value in values.iter() {
            if value.is_nan() || value.is_infinite() || *value < 0.0 {
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
                    if threshold.measurement_type == measurement.measurement_type && threshold.enabled {
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
        actual_value: f32,
    ) -> Result<(), Error> {
        let alert_id = Self::get_and_increment_alert_counter(env);

        let alert = MonitoringAlert {
            alert_id,
            patient: device.patient.clone(),
            provider: device.provider.clone(),
            device_id: device.device_id.clone(),
            alert_type: "threshold_breach".to_string(),
            severity: threshold.severity,
            message: format!(
                "Threshold breach for {}: {} (threshold: {:?})",
                measurement.measurement_type, actual_value, threshold
            ),
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
            (symbol_short!("Alert"), symbol_short!("Created")),
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
            message: format!(
                "Abnormal glucose level: {} mg/dL",
                measurement.glucose_level
            ),
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
            message: format!(
                "Elevated blood pressure: {}/{} mmHg",
                measurement.systolic, measurement.diastolic
            ),
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
            (symbol_short!("Alert"), symbol_short!("BloodPressure")),
            (alert_id, device.patient, measurement.systolic, measurement.diastolic),
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
            message: format!(
                "Missed medication: {}",
                adherence.medication_name
            ),
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
            (symbol_short!("Alert"), symbol_short!("Medication")),
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
        let count: u64 = env
            .storage()
            .persistent()
            .get(&ALERT_COUNTER)
            .unwrap_or(0);
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
        let count: u64 = env
            .storage()
            .persistent()
            .get(&EXPORT_COUNTER)
            .unwrap_or(0);
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
