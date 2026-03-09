#[allow(clippy::too_many_arguments)]
use soroban_sdk::{
    contract, contractimpl, contracttype, log, Address, Bytes, Env, Map, String, Vec,
};

#[cfg(test)]
mod test;

/// Pharmaceutical Supply Chain Tracking System
/// Tracks medications from manufacturer to patient with anti-counterfeiting,
/// condition monitoring, recall management, and regulatory compliance

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Manufacturers,
    Medications,
    Batches,
    Shipments,
    Dispensations,
    Recalls,
    Prescriptions,
    ConditionLogs,
    AdverseEvents,
    AuditTrail,
    Analytics,
}

#[derive(Clone, PartialEq, Debug)]
#[contracttype]
pub enum MedicationType {
    ControlledSubstance,
    Prescription,
    OverTheCounter,
    Biologic,
    Vaccine,
    ChemicalTherapy,
}

#[derive(Clone, PartialEq, Debug)]
#[contracttype]
pub enum ControlledSubstanceSchedule {
    Schedule1, // No accepted medical use
    Schedule2, // High potential for abuse
    Schedule3, // Moderate potential for abuse
    Schedule4, // Low potential for abuse
    Schedule5, // Lowest potential for abuse
    NotControlled,
}

#[derive(Clone, PartialEq, Debug)]
#[contracttype]
pub enum SupplyChainStage {
    Manufacturing,
    QualityControl,
    Packaging,
    Warehousing,
    Distribution,
    Wholesale,
    Pharmacy,
    Hospital,
    ClinicDispensary,
    Patient,
}

#[derive(Clone, PartialEq, Debug)]
#[contracttype]
pub enum ShipmentStatus {
    Pending,
    InTransit,
    Delivered,
    Delayed,
    ConditionViolation,
    Recalled,
    Rejected,
}

#[derive(Clone, PartialEq, Debug)]
#[contracttype]
pub enum RecallLevel {
    Class1, // Life-threatening
    Class2, // Serious adverse effects
    Class3, // Minor adverse effects
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct Manufacturer {
    pub id: String,
    pub address: Address,
    pub name: String,
    pub license_number: String,
    pub certifications: Vec<String>,
    pub country: String,
    pub is_active: bool,
    pub registered_at: u64,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct Medication {
    pub id: String,
    pub name: String,
    pub generic_name: String,
    pub ndc_code: String, // National Drug Code
    pub medication_type: MedicationType,
    pub schedule: ControlledSubstanceSchedule,
    pub manufacturer_id: String,
    pub requires_cold_chain: bool,
    pub min_temp_celsius: i32,
    pub max_temp_celsius: i32,
    pub max_humidity_percent: u32,
    pub shelf_life_days: u32,
    pub dosage_form: String,
    pub strength: String,
    pub active_ingredients: Vec<String>,
    pub created_at: u64,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct MedicationConfig {
    pub requires_cold_chain: bool,
    pub min_temp_celsius: i32,
    pub max_temp_celsius: i32,
    pub max_humidity_percent: u32,
    pub shelf_life_days: u32,
    pub dosage_form: String,
    pub strength: String,
    pub active_ingredients: Vec<String>,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct MedicationBatch {
    pub batch_id: String,
    pub medication_id: String,
    pub manufacturer_id: String,
    pub quantity: u64,
    pub manufacturing_date: u64,
    pub expiry_date: u64,
    pub lot_number: String,
    pub production_facility: String,
    pub quality_certificate: String,
    pub authentication_hash: Bytes, // Cryptographic hash for verification
    pub blockchain_anchor: Bytes,   // Anchor to external blockchain
    pub current_stage: SupplyChainStage,
    pub current_holder: Address,
    pub is_recalled: bool,
    pub recall_reason: Option<String>,
    pub created_at: u64,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct Shipment {
    pub shipment_id: String,
    pub batch_id: String,
    pub quantity: u64,
    pub from_address: Address,
    pub to_address: Address,
    pub from_stage: SupplyChainStage,
    pub to_stage: SupplyChainStage,
    pub status: ShipmentStatus,
    pub iot_device_id: Option<String>,
    pub started_at: u64,
    pub estimated_arrival: u64,
    pub delivered_at: Option<u64>,
    pub transport_conditions_verified: bool,
    pub digital_signature: Bytes,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct ConditionLog {
    pub log_id: String,
    pub shipment_id: String,
    pub batch_id: String,
    pub timestamp: u64,
    pub temperature_celsius: i32,
    pub humidity_percent: u32,
    pub location_lat: Option<i64>,
    pub location_lon: Option<i64>,
    pub iot_device_id: String,
    pub is_violation: bool,
    pub violation_type: Option<String>,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct Prescription {
    pub prescription_id: String,
    pub patient_address: Address,
    pub prescriber_address: Address,
    pub medication_id: String,
    pub quantity: u32,
    pub dosage_instructions: String,
    pub refills_allowed: u32,
    pub refills_used: u32,
    pub issued_date: u64,
    pub expiry_date: u64,
    pub medical_record_ref: Option<String>,
    pub is_active: bool,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct Dispensation {
    pub dispensation_id: String,
    pub prescription_id: String,
    pub batch_id: String,
    pub patient_address: Address,
    pub pharmacy_address: Address,
    pub quantity: u32,
    pub dispensed_at: u64,
    pub pharmacist_signature: Bytes,
    pub verification_code: String,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct Recall {
    pub recall_id: String,
    pub batch_ids: Vec<String>,
    pub medication_id: String,
    pub level: RecallLevel,
    pub reason: String,
    pub initiated_by: Address,
    pub initiated_at: u64,
    pub affected_patients: Vec<Address>,
    pub notifications_sent: u64,
    pub units_recovered: u64,
    pub is_active: bool,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct AdverseEvent {
    pub event_id: String,
    pub patient_address: Address,
    pub medication_id: String,
    pub batch_id: String,
    pub dispensation_id: String,
    pub severity: u32, // 1-5 scale
    pub description: String,
    pub reported_at: u64,
    pub reporter_address: Address,
    pub verified: bool,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct AuditEntry {
    pub entry_id: String,
    pub action: String,
    pub actor: Address,
    pub target_id: String,
    pub timestamp: u64,
    pub data_hash: Bytes,
    pub regulatory_flag: bool,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct SupplyChainAnalytics {
    pub total_batches: u64,
    pub active_shipments: u64,
    pub total_recalls: u64,
    pub total_adverse_events: u64,
    pub avg_delivery_time: u64,
    pub condition_violations: u64,
    pub counterfeit_attempts: u64,
    pub cs_dispensations: u64,
}

#[contract]
pub struct PharmaSupplyChain;

#[contractimpl]
impl PharmaSupplyChain {
    /// Initialize the contract
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(
            &DataKey::Manufacturers,
            &Map::<String, Manufacturer>::new(&env),
        );
        env.storage()
            .instance()
            .set(&DataKey::Medications, &Map::<String, Medication>::new(&env));
        env.storage().instance().set(
            &DataKey::Batches,
            &Map::<String, MedicationBatch>::new(&env),
        );
        env.storage()
            .instance()
            .set(&DataKey::Shipments, &Map::<String, Shipment>::new(&env));
        env.storage().instance().set(
            &DataKey::Dispensations,
            &Map::<String, Dispensation>::new(&env),
        );
        env.storage()
            .instance()
            .set(&DataKey::Recalls, &Map::<String, Recall>::new(&env));
        env.storage().instance().set(
            &DataKey::Prescriptions,
            &Map::<String, Prescription>::new(&env),
        );
        env.storage()
            .instance()
            .set(&DataKey::ConditionLogs, &Vec::<ConditionLog>::new(&env));
        env.storage()
            .instance()
            .set(&DataKey::AdverseEvents, &Vec::<AdverseEvent>::new(&env));
        env.storage()
            .instance()
            .set(&DataKey::AuditTrail, &Vec::<AuditEntry>::new(&env));

        let analytics = SupplyChainAnalytics {
            total_batches: 0,
            active_shipments: 0,
            total_recalls: 0,
            total_adverse_events: 0,
            avg_delivery_time: 0,
            condition_violations: 0,
            counterfeit_attempts: 0,
            cs_dispensations: 0,
        };
        env.storage()
            .instance()
            .set(&DataKey::Analytics, &analytics);

        log!(&env, "PharmaSupplyChain: Initialized with admin: {}", admin);
    }

    // ==================== MANUFACTURER MANAGEMENT ====================

    /// Register a new pharmaceutical manufacturer
    pub fn register_manufacturer(
        env: Env,
        id: String,
        manufacturer_address: Address,
        name: String,
        license_number: String,
        certifications: Vec<String>,
        country: String,
    ) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let mut manufacturers: Map<String, Manufacturer> = env
            .storage()
            .instance()
            .get(&DataKey::Manufacturers)
            .unwrap();

        let manufacturer = Manufacturer {
            id: id.clone(),
            address: manufacturer_address.clone(),
            name: name.clone(),
            license_number,
            certifications,
            country,
            is_active: true,
            registered_at: env.ledger().timestamp(),
        };

        manufacturers.set(id.clone(), manufacturer);
        env.storage()
            .instance()
            .set(&DataKey::Manufacturers, &manufacturers);

        Self::add_audit_entry(
            env.clone(),
            String::from_str(&env, "MANUFACTURER_REGISTERED"),
            admin,
            id,
            true,
        );

        log!(&env, "PharmaSupplyChain: Manufacturer registered: {}", name);
    }

    /// Deactivate a manufacturer
    pub fn deactivate_manufacturer(env: Env, manufacturer_id: String) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let mut manufacturers: Map<String, Manufacturer> = env
            .storage()
            .instance()
            .get(&DataKey::Manufacturers)
            .unwrap();

        if let Some(mut manufacturer) = manufacturers.get(manufacturer_id.clone()) {
            manufacturer.is_active = false;
            manufacturers.set(manufacturer_id.clone(), manufacturer);
            env.storage()
                .instance()
                .set(&DataKey::Manufacturers, &manufacturers);

            Self::add_audit_entry(
                env.clone(),
                String::from_str(&env, "MANUFACTURER_DEACTIVATED"),
                admin,
                manufacturer_id,
                true,
            );
        }
    }

    // ==================== MEDICATION MANAGEMENT ====================

    /// Register a new medication
    pub fn register_medication(
        env: Env,
        id: String,
        name: String,
        generic_name: String,
        ndc_code: String,
        medication_type: MedicationType,
        schedule: ControlledSubstanceSchedule,
        manufacturer_id: String,
        config: MedicationConfig,
    ) {
        let manufacturers: Map<String, Manufacturer> = env
            .storage()
            .instance()
            .get(&DataKey::Manufacturers)
            .unwrap();

        let manufacturer = manufacturers
            .get(manufacturer_id.clone())
            .expect("Manufacturer not found");
        manufacturer.address.require_auth();

        let mut medications: Map<String, Medication> =
            env.storage().instance().get(&DataKey::Medications).unwrap();

        let medication = Medication {
            id: id.clone(),
            name: name.clone(),
            generic_name,
            ndc_code,
            medication_type,
            schedule,
            manufacturer_id: manufacturer_id.clone(),
            requires_cold_chain: config.requires_cold_chain,
            min_temp_celsius: config.min_temp_celsius,
            max_temp_celsius: config.max_temp_celsius,
            max_humidity_percent: config.max_humidity_percent,
            shelf_life_days: config.shelf_life_days,
            dosage_form: config.dosage_form,
            strength: config.strength,
            active_ingredients: config.active_ingredients,
            created_at: env.ledger().timestamp(),
        };

        medications.set(id.clone(), medication);
        env.storage()
            .instance()
            .set(&DataKey::Medications, &medications);

        Self::add_audit_entry(
            env.clone(),
            String::from_str(&env, "MEDICATION_REGISTERED"),
            manufacturer.address,
            id,
            true,
        );

        log!(&env, "PharmaSupplyChain: Medication registered: {}", name);
    }

    // ==================== BATCH MANAGEMENT ====================

    /// Create a new medication batch with anti-counterfeiting measures
    pub fn create_batch(
        env: Env,
        batch_id: String,
        medication_id: String,
        quantity: u64,
        manufacturing_date: u64,
        lot_number: String,
        production_facility: String,
        quality_certificate: String,
    ) -> Bytes {
        let medications: Map<String, Medication> =
            env.storage().instance().get(&DataKey::Medications).unwrap();
        let medication = medications
            .get(medication_id.clone())
            .expect("Medication not found");

        let manufacturers: Map<String, Manufacturer> = env
            .storage()
            .instance()
            .get(&DataKey::Manufacturers)
            .unwrap();
        let manufacturer = manufacturers
            .get(medication.manufacturer_id.clone())
            .expect("Manufacturer not found");
        manufacturer.address.require_auth();

        // Generate cryptographic authentication hash
        let auth_data = Bytes::from_slice(
            &env,
            &[
                batch_id.to_string().as_bytes(),
                medication_id.to_string().as_bytes(),
                lot_number.to_string().as_bytes(),
            ]
            .concat(),
        );
        let authentication_hash = env.crypto().sha256(&auth_data);

        // Generate blockchain anchor (simplified - would integrate with external chain)
        let auth_hash_bytes: Bytes = authentication_hash.clone().into();
        let blockchain_anchor = env.crypto().sha256(&auth_hash_bytes);

        let expiry_date = manufacturing_date + (medication.shelf_life_days as u64 * 86400);

        let mut batches: Map<String, MedicationBatch> =
            env.storage().instance().get(&DataKey::Batches).unwrap();

        let batch = MedicationBatch {
            batch_id: batch_id.clone(),
            medication_id: medication_id.clone(),
            manufacturer_id: medication.manufacturer_id.clone(),
            quantity,
            manufacturing_date,
            expiry_date,
            lot_number,
            production_facility,
            quality_certificate,
            authentication_hash: authentication_hash.clone().into(),
            blockchain_anchor: blockchain_anchor.clone().into(),
            current_stage: SupplyChainStage::Manufacturing,
            current_holder: manufacturer.address.clone(),
            is_recalled: false,
            recall_reason: None,
            created_at: env.ledger().timestamp(),
        };

        batches.set(batch_id.clone(), batch);
        env.storage().instance().set(&DataKey::Batches, &batches);

        // Update analytics
        let mut analytics: SupplyChainAnalytics =
            env.storage().instance().get(&DataKey::Analytics).unwrap();
        analytics.total_batches += 1;
        env.storage()
            .instance()
            .set(&DataKey::Analytics, &analytics);

        Self::add_audit_entry(
            env.clone(),
            String::from_str(&env, "BATCH_CREATED"),
            manufacturer.address,
            batch_id,
            true,
        );

        log!(
            &env,
            "PharmaSupplyChain: Batch created with authentication hash"
        );
        authentication_hash.into()
    }

    /// Verify batch authenticity using cryptographic hash
    pub fn verify_batch_authenticity(env: Env, batch_id: String, provided_hash: Bytes) -> bool {
        let batches: Map<String, MedicationBatch> =
            env.storage().instance().get(&DataKey::Batches).unwrap();

        if let Some(batch) = batches.get(batch_id.clone()) {
            let is_valid = batch.authentication_hash == provided_hash;

            if !is_valid {
                // Log potential counterfeit attempt
                let mut analytics: SupplyChainAnalytics =
                    env.storage().instance().get(&DataKey::Analytics).unwrap();
                analytics.counterfeit_attempts += 1;
                env.storage()
                    .instance()
                    .set(&DataKey::Analytics, &analytics);

                log!(
                    &env,
                    "PharmaSupplyChain: Counterfeit attempt detected for batch: {}",
                    batch_id
                );
            }

            is_valid
        } else {
            false
        }
    }

    // ==================== SHIPMENT & TRACKING ====================

    /// Create a new shipment with IoT device tracking
    pub fn create_shipment(
        env: Env,
        shipment_id: String,
        batch_id: String,
        quantity: u64,
        to_address: Address,
        to_stage: SupplyChainStage,
        estimated_arrival: u64,
        iot_device_id: Option<String>,
    ) {
        let batches: Map<String, MedicationBatch> =
            env.storage().instance().get(&DataKey::Batches).unwrap();
        let batch = batches.get(batch_id.clone()).expect("Batch not found");

        batch.current_holder.require_auth();

        // Verify batch is not recalled
        if batch.is_recalled {
            panic!("Cannot ship recalled batch");
        }

        // Generate digital signature for shipment
        let signature_data = Bytes::from_slice(
            &env,
            &[
                shipment_id.to_string().as_bytes(),
                batch_id.to_string().as_bytes(),
            ]
            .concat(),
        );
        let digital_signature = env.crypto().sha256(&signature_data);

        let mut shipments: Map<String, Shipment> =
            env.storage().instance().get(&DataKey::Shipments).unwrap();

        let shipment = Shipment {
            shipment_id: shipment_id.clone(),
            batch_id: batch_id.clone(),
            quantity,
            from_address: batch.current_holder.clone(),
            to_address: to_address.clone(),
            from_stage: batch.current_stage.clone(),
            to_stage: to_stage.clone(),
            status: ShipmentStatus::InTransit,
            iot_device_id,
            started_at: env.ledger().timestamp(),
            estimated_arrival,
            delivered_at: None,
            transport_conditions_verified: false,
            digital_signature: digital_signature.into(),
        };

        shipments.set(shipment_id.clone(), shipment);
        env.storage()
            .instance()
            .set(&DataKey::Shipments, &shipments);

        // Update analytics
        let mut analytics: SupplyChainAnalytics =
            env.storage().instance().get(&DataKey::Analytics).unwrap();
        analytics.active_shipments += 1;
        env.storage()
            .instance()
            .set(&DataKey::Analytics, &analytics);

        Self::add_audit_entry(
            env.clone(),
            String::from_str(&env, "SHIPMENT_CREATED"),
            batch.current_holder,
            shipment_id,
            false,
        );

        log!(
            &env,
            "PharmaSupplyChain: Shipment created for batch: {}",
            batch_id
        );
    }

    /// Log condition data from IoT device
    pub fn log_condition_data(
        env: Env,
        log_id: String,
        shipment_id: String,
        temperature: i32,
        humidity: u32,
        location_lat: Option<i64>,
        location_lon: Option<i64>,
        iot_device_id: String,
    ) {
        let shipments: Map<String, Shipment> =
            env.storage().instance().get(&DataKey::Shipments).unwrap();
        let shipment = shipments
            .get(shipment_id.clone())
            .expect("Shipment not found");

        // Verify IoT device matches shipment
        if let Some(ref device_id) = shipment.iot_device_id {
            if device_id != &iot_device_id {
                panic!("IoT device mismatch");
            }
        }

        let batches: Map<String, MedicationBatch> =
            env.storage().instance().get(&DataKey::Batches).unwrap();
        let batch = batches
            .get(shipment.batch_id.clone())
            .expect("Batch not found");

        let medications: Map<String, Medication> =
            env.storage().instance().get(&DataKey::Medications).unwrap();
        let medication = medications
            .get(batch.medication_id.clone())
            .expect("Medication not found");

        // Check for violations
        let mut is_violation = false;
        let mut violation_type: Option<String> = None;

        if temperature < medication.min_temp_celsius || temperature > medication.max_temp_celsius {
            is_violation = true;
            violation_type = Some(String::from_str(&env, "TEMPERATURE_VIOLATION"));
        }

        if humidity > medication.max_humidity_percent {
            is_violation = true;
            violation_type = Some(String::from_str(&env, "HUMIDITY_VIOLATION"));
        }

        let condition_log = ConditionLog {
            log_id: log_id.clone(),
            shipment_id: shipment_id.clone(),
            batch_id: shipment.batch_id.clone(),
            timestamp: env.ledger().timestamp(),
            temperature_celsius: temperature,
            humidity_percent: humidity,
            location_lat,
            location_lon,
            iot_device_id: iot_device_id.clone(),
            is_violation,
            violation_type,
        };

        let mut condition_logs: Vec<ConditionLog> = env
            .storage()
            .instance()
            .get(&DataKey::ConditionLogs)
            .unwrap();
        condition_logs.push_back(condition_log);
        env.storage()
            .instance()
            .set(&DataKey::ConditionLogs, &condition_logs);

        // If violation detected, update shipment status and analytics
        if is_violation {
            let mut shipments_mut: Map<String, Shipment> =
                env.storage().instance().get(&DataKey::Shipments).unwrap();
            let mut shipment_mut = shipments_mut.get(shipment_id.clone()).unwrap();
            shipment_mut.status = ShipmentStatus::ConditionViolation;
            shipments_mut.set(shipment_id.clone(), shipment_mut);
            env.storage()
                .instance()
                .set(&DataKey::Shipments, &shipments_mut);

            let mut analytics: SupplyChainAnalytics =
                env.storage().instance().get(&DataKey::Analytics).unwrap();
            analytics.condition_violations += 1;
            env.storage()
                .instance()
                .set(&DataKey::Analytics, &analytics);

            log!(
                &env,
                "PharmaSupplyChain: Condition violation detected for shipment: {}",
                shipment_id
            );
        }
    }

    /// Complete a shipment delivery
    pub fn complete_shipment(env: Env, shipment_id: String, conditions_verified: bool) {
        let mut shipments: Map<String, Shipment> =
            env.storage().instance().get(&DataKey::Shipments).unwrap();
        let mut shipment = shipments
            .get(shipment_id.clone())
            .expect("Shipment not found");

        shipment.to_address.require_auth();

        // Check for condition violations
        if shipment.status == ShipmentStatus::ConditionViolation && !conditions_verified {
            panic!("Cannot complete shipment with unresolved condition violations");
        }

        shipment.status = ShipmentStatus::Delivered;
        shipment.delivered_at = Some(env.ledger().timestamp());
        shipment.transport_conditions_verified = conditions_verified;

        shipments.set(shipment_id.clone(), shipment.clone());
        env.storage()
            .instance()
            .set(&DataKey::Shipments, &shipments);

        // Update batch location and stage
        let mut batches: Map<String, MedicationBatch> =
            env.storage().instance().get(&DataKey::Batches).unwrap();
        let mut batch = batches.get(shipment.batch_id.clone()).unwrap();
        batch.current_holder = shipment.to_address.clone();
        batch.current_stage = shipment.to_stage.clone();
        batches.set(shipment.batch_id.clone(), batch);
        env.storage().instance().set(&DataKey::Batches, &batches);

        // Update analytics
        let mut analytics: SupplyChainAnalytics =
            env.storage().instance().get(&DataKey::Analytics).unwrap();
        analytics.active_shipments = analytics.active_shipments.saturating_sub(1);

        // Update average delivery time
        let delivery_time = shipment.delivered_at.unwrap() - shipment.started_at;
        if analytics.total_batches > 0 {
            analytics.avg_delivery_time =
                (analytics.avg_delivery_time * (analytics.total_batches - 1) + delivery_time)
                    / analytics.total_batches;
        }
        env.storage()
            .instance()
            .set(&DataKey::Analytics, &analytics);

        Self::add_audit_entry(
            env.clone(),
            String::from_str(&env, "SHIPMENT_DELIVERED"),
            shipment.to_address,
            shipment_id,
            false,
        );

        log!(
            &env,
            "PharmaSupplyChain: Shipment completed: {}",
            shipment.shipment_id
        );
    }

    // ==================== PRESCRIPTION & DISPENSATION ====================

    /// Create a prescription
    pub fn create_prescription(
        env: Env,
        prescription_id: String,
        prescriber_address: Address,
        patient_address: Address,
        medication_id: String,
        quantity: u32,
        dosage_instructions: String,
        refills_allowed: u32,
        expiry_date: u64,
        medical_record_ref: Option<String>,
    ) {
        prescriber_address.require_auth();

        let medications: Map<String, Medication> =
            env.storage().instance().get(&DataKey::Medications).unwrap();
        medications
            .get(medication_id.clone())
            .expect("Medication not found");

        let mut prescriptions: Map<String, Prescription> = env
            .storage()
            .instance()
            .get(&DataKey::Prescriptions)
            .unwrap();

        let prescription = Prescription {
            prescription_id: prescription_id.clone(),
            patient_address: patient_address.clone(),
            prescriber_address: prescriber_address.clone(),
            medication_id: medication_id.clone(),
            quantity,
            dosage_instructions,
            refills_allowed,
            refills_used: 0,
            issued_date: env.ledger().timestamp(),
            expiry_date,
            medical_record_ref,
            is_active: true,
        };

        prescriptions.set(prescription_id.clone(), prescription);
        env.storage()
            .instance()
            .set(&DataKey::Prescriptions, &prescriptions);

        Self::add_audit_entry(
            env.clone(),
            String::from_str(&env, "PRESCRIPTION_CREATED"),
            prescriber_address,
            prescription_id,
            true,
        );

        log!(&env, "PharmaSupplyChain: Prescription created for patient");
    }

    /// Dispense medication with prescription verification
    pub fn dispense_medication(
        env: Env,
        dispensation_id: String,
        pharmacist_address: Address,
        prescription_id: String,
        batch_id: String,
        quantity: u32,
        verification_code: String,
    ) {
        pharmacist_address.require_auth();

        let mut prescriptions: Map<String, Prescription> = env
            .storage()
            .instance()
            .get(&DataKey::Prescriptions)
            .unwrap();
        let mut prescription = prescriptions
            .get(prescription_id.clone())
            .expect("Prescription not found");

        // Verify prescription is active and valid
        if !prescription.is_active {
            panic!("Prescription is not active");
        }

        if env.ledger().timestamp() > prescription.expiry_date {
            panic!("Prescription has expired");
        }

        if prescription.refills_used > prescription.refills_allowed {
            panic!("No refills remaining");
        }

        // Verify batch exists and matches medication
        let batches: Map<String, MedicationBatch> =
            env.storage().instance().get(&DataKey::Batches).unwrap();
        let batch = batches.get(batch_id.clone()).expect("Batch not found");

        if batch.medication_id != prescription.medication_id {
            panic!("Batch medication does not match prescription");
        }

        if batch.is_recalled {
            panic!("Cannot dispense recalled medication");
        }

        // Check expiry
        if env.ledger().timestamp() > batch.expiry_date {
            panic!("Medication batch has expired");
        }

        // Generate pharmacist signature
        let signature_data = Bytes::from_slice(
            &env,
            &[
                dispensation_id.to_string().as_bytes(),
                prescription_id.to_string().as_bytes(),
            ]
            .concat(),
        );
        let pharmacist_signature = env.crypto().sha256(&signature_data);

        let mut dispensations: Map<String, Dispensation> = env
            .storage()
            .instance()
            .get(&DataKey::Dispensations)
            .unwrap();

        let dispensation = Dispensation {
            dispensation_id: dispensation_id.clone(),
            prescription_id: prescription_id.clone(),
            batch_id: batch_id.clone(),
            patient_address: prescription.patient_address.clone(),
            pharmacy_address: pharmacist_address.clone(),
            quantity,
            dispensed_at: env.ledger().timestamp(),
            pharmacist_signature: pharmacist_signature.into(),
            verification_code,
        };

        dispensations.set(dispensation_id.clone(), dispensation);
        env.storage()
            .instance()
            .set(&DataKey::Dispensations, &dispensations);

        // Update prescription refills
        prescription.refills_used += 1;
        prescriptions.set(prescription_id.clone(), prescription.clone());
        env.storage()
            .instance()
            .set(&DataKey::Prescriptions, &prescriptions);

        // Track controlled substances
        let medications: Map<String, Medication> =
            env.storage().instance().get(&DataKey::Medications).unwrap();
        let medication = medications.get(prescription.medication_id.clone()).unwrap();

        if medication.schedule != ControlledSubstanceSchedule::NotControlled {
            let mut analytics: SupplyChainAnalytics =
                env.storage().instance().get(&DataKey::Analytics).unwrap();
            analytics.cs_dispensations += 1;
            env.storage()
                .instance()
                .set(&DataKey::Analytics, &analytics);
        }

        Self::add_audit_entry(
            env.clone(),
            String::from_str(&env, "MEDICATION_DISPENSED"),
            pharmacist_address,
            dispensation_id,
            true,
        );

        log!(&env, "PharmaSupplyChain: Medication dispensed");
    }

    // ==================== RECALL MANAGEMENT ====================

    /// Initiate a medication recall
    pub fn initiate_recall(
        env: Env,
        recall_id: String,
        batch_ids: Vec<String>,
        medication_id: String,
        level: RecallLevel,
        reason: String,
    ) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let mut batches: Map<String, MedicationBatch> =
            env.storage().instance().get(&DataKey::Batches).unwrap();

        // Mark all batches as recalled
        for i in 0..batch_ids.len() {
            let batch_id = batch_ids.get(i).unwrap();
            if let Some(mut batch) = batches.get(batch_id.clone()) {
                batch.is_recalled = true;
                batch.recall_reason = Some(reason.clone());
                batches.set(batch_id, batch);
            }
        }
        env.storage().instance().set(&DataKey::Batches, &batches);

        // Find affected patients
        let _dispensations: Map<String, Dispensation> = env
            .storage()
            .instance()
            .get(&DataKey::Dispensations)
            .unwrap();
        let affected_patients = Vec::<Address>::new(&env);

        // Simplified: would iterate through all dispensations
        // For production, would use indexed queries

        let mut recalls: Map<String, Recall> =
            env.storage().instance().get(&DataKey::Recalls).unwrap();

        let recall = Recall {
            recall_id: recall_id.clone(),
            batch_ids: batch_ids.clone(),
            medication_id,
            level,
            reason,
            initiated_by: admin.clone(),
            initiated_at: env.ledger().timestamp(),
            affected_patients,
            notifications_sent: 0, // Would be updated as notifications are sent
            units_recovered: 0,
            is_active: true,
        };

        recalls.set(recall_id.clone(), recall);
        env.storage().instance().set(&DataKey::Recalls, &recalls);

        // Update analytics
        let mut analytics: SupplyChainAnalytics =
            env.storage().instance().get(&DataKey::Analytics).unwrap();
        analytics.total_recalls += 1;
        env.storage()
            .instance()
            .set(&DataKey::Analytics, &analytics);

        Self::add_audit_entry(
            env.clone(),
            String::from_str(&env, "RECALL_INITIATED"),
            admin,
            recall_id,
            true,
        );

        log!(
            &env,
            "PharmaSupplyChain: Recall initiated for {} batches",
            batch_ids.len()
        );
    }

    /// Update recall recovery status
    pub fn update_recall_recovery(env: Env, recall_id: String, units_recovered: u64) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let mut recalls: Map<String, Recall> =
            env.storage().instance().get(&DataKey::Recalls).unwrap();

        if let Some(mut recall) = recalls.get(recall_id.clone()) {
            recall.units_recovered += units_recovered;
            recalls.set(recall_id, recall);
            env.storage().instance().set(&DataKey::Recalls, &recalls);
        }
    }

    // ==================== ADVERSE EVENTS ====================

    /// Report an adverse event
    pub fn report_adverse_event(
        env: Env,
        event_id: String,
        reporter_address: Address,
        patient_address: Address,
        medication_id: String,
        batch_id: String,
        dispensation_id: String,
        severity: u32,
        description: String,
    ) {
        reporter_address.require_auth();

        let adverse_event = AdverseEvent {
            event_id: event_id.clone(),
            patient_address,
            medication_id,
            batch_id,
            dispensation_id,
            severity,
            description,
            reported_at: env.ledger().timestamp(),
            reporter_address: reporter_address.clone(),
            verified: false,
        };

        let mut adverse_events: Vec<AdverseEvent> = env
            .storage()
            .instance()
            .get(&DataKey::AdverseEvents)
            .unwrap();
        adverse_events.push_back(adverse_event);
        env.storage()
            .instance()
            .set(&DataKey::AdverseEvents, &adverse_events);

        // Update analytics
        let mut analytics: SupplyChainAnalytics =
            env.storage().instance().get(&DataKey::Analytics).unwrap();
        analytics.total_adverse_events += 1;
        env.storage()
            .instance()
            .set(&DataKey::Analytics, &analytics);

        Self::add_audit_entry(
            env.clone(),
            String::from_str(&env, "ADVERSE_EVENT_REPORTED"),
            reporter_address,
            event_id,
            true,
        );

        log!(&env, "PharmaSupplyChain: Adverse event reported");
    }

    // ==================== ANALYTICS & REPORTING ====================

    /// Get supply chain analytics
    pub fn get_analytics(env: Env) -> SupplyChainAnalytics {
        env.storage().instance().get(&DataKey::Analytics).unwrap()
    }

    /// Get batch information
    pub fn get_batch(env: Env, batch_id: String) -> Option<MedicationBatch> {
        let batches: Map<String, MedicationBatch> =
            env.storage().instance().get(&DataKey::Batches).unwrap();
        batches.get(batch_id)
    }

    /// Get medication information
    pub fn get_medication(env: Env, medication_id: String) -> Option<Medication> {
        let medications: Map<String, Medication> =
            env.storage().instance().get(&DataKey::Medications).unwrap();
        medications.get(medication_id)
    }

    /// Get shipment status
    pub fn get_shipment(env: Env, shipment_id: String) -> Option<Shipment> {
        let shipments: Map<String, Shipment> =
            env.storage().instance().get(&DataKey::Shipments).unwrap();
        shipments.get(shipment_id)
    }

    /// Check if batch is expired
    pub fn is_batch_expired(env: Env, batch_id: String) -> bool {
        let batches: Map<String, MedicationBatch> =
            env.storage().instance().get(&DataKey::Batches).unwrap();

        if let Some(batch) = batches.get(batch_id) {
            env.ledger().timestamp() > batch.expiry_date
        } else {
            false
        }
    }

    /// Get recall information
    pub fn get_recall(env: Env, recall_id: String) -> Option<Recall> {
        let recalls: Map<String, Recall> = env.storage().instance().get(&DataKey::Recalls).unwrap();
        recalls.get(recall_id)
    }

    // ==================== INTERNAL HELPERS ====================

    fn add_audit_entry(
        env: Env,
        action: String,
        actor: Address,
        target_id: String,
        regulatory_flag: bool,
    ) {
        let entry_id = String::from_str(&env, "audit_");

        let data = Bytes::from_slice(
            &env,
            &[
                action.to_string().as_bytes(),
                target_id.to_string().as_bytes(),
            ]
            .concat(),
        );
        let data_hash = env.crypto().sha256(&data);

        let audit_entry = AuditEntry {
            entry_id,
            action,
            actor,
            target_id,
            timestamp: env.ledger().timestamp(),
            data_hash: data_hash.into(),
            regulatory_flag,
        };

        let mut audit_trail: Vec<AuditEntry> =
            env.storage().instance().get(&DataKey::AuditTrail).unwrap();
        audit_trail.push_back(audit_entry);
        env.storage()
            .instance()
            .set(&DataKey::AuditTrail, &audit_trail);
    }
}
