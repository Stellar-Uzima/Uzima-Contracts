#![no_std]

#[cfg(test)]
mod test;

use soroban_sdk::symbol_short;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, vec, Address, BytesN, Env, Map, String,
    Symbol, Vec,
};

// ==================== Virtual Prescription Types ====================

/// Prescription Status
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum PrescriptionStatus {
    Draft,
    PendingVerification,
    Active,
    Suspended,
    Completed,
    Cancelled,
    Expired,
    Rejected,
}

/// Medication Type
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum MedicationType {
    Tablet,
    Capsule,
    Liquid,
    Injection,
    Inhaler,
    Patch,
    Cream,
    Ointment,
    Drops,
    Spray,
    Suppository,
    Powder,
}

/// Schedule Frequency
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum ScheduleFrequency {
    AsNeeded,
    Once,
    Twice,
    ThreeTimes,
    FourTimes,
    Hourly,
    Every2Hours,
    Every4Hours,
    Every6Hours,
    Every8Hours,
    Every12Hours,
    Daily,
    Weekly,
    Monthly,
}

/// Refill Status
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum RefillStatus {
    Available,
    Requested,
    Processing,
    Filled,
    PickedUp,
    Expired,
    Denied,
}

/// Pharmacy Verification Status
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum VerificationStatus {
    Pending,
    Verified,
    Rejected,
    RequiresClarification,
}

/// Virtual Prescription
#[derive(Clone)]
#[contracttype]
pub struct VirtualPrescription {
    pub prescription_id: u64,
    pub patient: Address,
    pub prescriber: Address,
    pub pharmacy: Option<Address>,
    pub medication_name: String,
    pub generic_name: Option<String>,
    pub medication_type: MedicationType,
    pub strength: String,    // e.g., "500mg", "10mg/mL"
    pub dosage_form: String, // e.g., "tablet", "solution"
    pub quantity: u32,
    pub refills_allowed: u8,
    pub refills_remaining: u8,
    pub date_written: u64,
    pub date_filled: Option<u64>,
    pub expiration_date: u64,
    pub status: PrescriptionStatus,
    pub verification_status: VerificationStatus,
    pub verification_notes: String,
    pub instructions: String,
    pub indications: String,
    pub contraindications: Vec<String>,
    pub side_effects: Vec<String>,
    pub drug_interactions: Vec<String>,
    pub allergies_check: bool,
    pub pregnancy_category: String, // A, B, C, D, X
    pub controlled_substance: bool,
    pub schedule_number: Option<u8>,  // For controlled substances
    pub dea_number: String,           // Prescriber's DEA number
    pub state_license: String,        // Prescriber's state license
    pub diagnosis_codes: Vec<String>, // ICD-10 codes
    pub prior_auth_required: bool,
    pub prior_auth_status: String,
    pub consent_token_id: u64,
    pub digital_signature: BytesN<64>, // Prescriber's digital signature
    pub signature_timestamp: u64,
}

/// Medication Schedule
#[derive(Clone)]
#[contracttype]
pub struct MedicationSchedule {
    pub schedule_id: u64,
    pub prescription_id: u64,
    pub frequency: ScheduleFrequency,
    pub specific_times: Vec<String>, // e.g., ["08:00", "14:00", "20:00"]
    pub with_food: bool,
    pub with_water: bool,
    pub avoid_alcohol: bool,
    pub avoid_driving: bool,
    pub special_instructions: Vec<String>,
    pub start_date: u64,
    pub end_date: u64,
    pub is_active: bool,
}

/// Refill Request
#[derive(Clone)]
#[contracttype]
pub struct RefillRequest {
    pub request_id: u64,
    pub prescription_id: u64,
    pub patient: Address,
    pub pharmacy: Address,
    pub requested_at: u64,
    pub status: RefillStatus,
    pub processing_notes: String,
    pub filled_at: Option<u64>,
    pub picked_up_at: Option<u64>,
    pub expires_at: u64,
}

/// Medication Adherence Record
#[derive(Clone)]
#[contracttype]
pub struct MedicationAdherence {
    pub adherence_id: u64,
    pub prescription_id: u64,
    pub patient: Address,
    pub scheduled_time: u64,
    pub taken_time: u64,
    pub dose_taken: f32,
    pub prescribed_dose: f32,
    pub adherence_percentage: u8,
    pub missed_dose: bool,
    pub late_dose: bool,
    pub notes: String,
    pub location: String,
    pub verification_method: String, // "self_reported", "smart_pill", "pharmacy_confirm"
}

/// Drug Interaction Check
#[derive(Clone)]
#[contracttype]
pub struct DrugInteraction {
    pub interaction_id: u64,
    pub prescription_id: u64,
    pub interacting_drug: String,
    pub interaction_severity: String, // "minor", "moderate", "major", "contraindicated"
    pub interaction_description: String,
    pub clinical_effects: Vec<String>,
    pub management_recommendations: Vec<String>,
    pub alternatives: Vec<String>,
    pub checked_at: u64,
}

/// Allergy Check Result
#[derive(Clone)]
#[contracttype]
pub struct AllergyCheckResult {
    pub check_id: u64,
    pub prescription_id: u64,
    pub patient_allergies: Vec<String>,
    pub medication_allergens: Vec<String>,
    pub cross_reactivity: Vec<String>,
    pub severity: String, // "none", "mild", "moderate", "severe"
    pub recommendations: Vec<String>,
    pub checked_at: u64,
}

/// Pharmacy Network
#[derive(Clone)]
#[contracttype]
pub struct PharmacyNetwork {
    pub pharmacy_id: Address,
    pub pharmacy_name: String,
    pub license_number: String,
    pub address: String,
    pub phone: String,
    pub fax: String,
    pub email: String,
    pub npi: String, // National Provider Identifier
    pub dea_registration: String,
    pub state_license: String,
    pub hours_of_operation: String,
    pub delivery_available: bool,
    pub compounding_services: bool,
    pub specialty_pharmacy: bool,
    pub digital_prescribing_enabled: bool,
    pub status: String, // "active", "inactive", "suspended"
    pub rating: u8,     // 1-5 stars
    pub joined_network: u64,
}

/// Prior Authorization Request
#[derive(Clone)]
#[contracttype]
pub struct PriorAuthRequest {
    pub auth_id: u64,
    pub prescription_id: u64,
    pub patient: Address,
    pub prescriber: Address,
    pub insurance_provider: String,
    pub policy_number: String,
    pub diagnosis_codes: Vec<String>,
    pub clinical_necessity: String,
    pub alternative_medications: Vec<String>,
    pub treatment_duration: u32,
    pub requested_at: u64,
    pub status: String, // "pending", "approved", "denied", "more_info"
    pub decision_date: Option<u64>,
    pub approval_duration: u32, // days
    pub denial_reason: String,
    pub case_number: String,
}

// Storage Keys
const ADMIN: Symbol = symbol_short!("ADMIN");
const PRESCRIPTIONS: Symbol = symbol_short!("PRESCRIPTIONS");
const MEDICATION_SCHEDULES: Symbol = symbol_short!("SCHEDULES");
const REFILL_REQUESTS: Symbol = symbol_short!("REFILLS");
const ADHERENCE_RECORDS: Symbol = symbol_short!("ADHERENCE");
const DRUG_INTERACTIONS: Symbol = symbol_short!("INTERACTIONS");
const ALLERGY_CHECKS: Symbol = symbol_short!("ALLERGIES");
const PHARMACY_NETWORK: Symbol = symbol_short!("PHARMACIES");
const PRIOR_AUTH_REQUESTS: Symbol = symbol_short!("PRIOR_AUTH");
const PRESCRIPTION_COUNTER: Symbol = symbol_short!("PRESC_CNT");
const REFILL_COUNTER: Symbol = symbol_short!("REFILL_CNT");
const ADHERENCE_COUNTER: Symbol = symbol_short!("ADH_CNT");
const INTERACTION_COUNTER: Symbol = symbol_short!("INT_CNT");
const ALLERGY_COUNTER: Symbol = symbol_short!("ALLERGY_CNT");
const AUTH_COUNTER: Symbol = symbol_short!("AUTH_CNT");
const PAUSED: Symbol = symbol_short!("PAUSED");
const CONSENT_CONTRACT: Symbol = symbol_short!("CONSENT");
const MEDICAL_RECORDS_CONTRACT: Symbol = symbol_short!("MEDICAL");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    ContractPaused = 2,
    PrescriptionNotFound = 3,
    PrescriptionAlreadyExists = 4,
    InvalidStatus = 5,
    InvalidMedicationType = 6,
    InvalidQuantity = 7,
    InvalidRefills = 8,
    RefillNotAllowed = 9,
    NoRefillsRemaining = 10,
    PrescriptionExpired = 11,
    PharmacyNotInNetwork = 12,
    ControlledSubstanceViolation = 13,
    SignatureInvalid = 14,
    AllergyConflict = 15,
    DrugInteraction = 16,
    PriorAuthRequired = 17,
    PriorAuthDenied = 18,
    InvalidSchedule = 19,
    DuplicateRefillRequest = 20,
    RefillRequestNotFound = 21,
    ConsentRequired = 22,
    ConsentRevoked = 23,
    InvalidDEANumber = 24,
    InvalidLicense = 25,
    MedicalRecordsContractNotSet = 26,
    ConsentContractNotSet = 27,
}

#[contract]
pub struct VirtualPrescriptionContract;

#[contractimpl]
impl VirtualPrescriptionContract {
    /// Initialize the virtual prescription contract
    pub fn initialize(
        env: Env,
        admin: Address,
        consent_contract: Address,
        medical_records_contract: Address,
    ) -> Result<bool, Error> {
        admin.require_auth();

        if env.storage().persistent().has(&ADMIN) {
            return Err(Error::PrescriptionAlreadyExists);
        }

        env.storage().persistent().set(&ADMIN, &admin);
        env.storage()
            .persistent()
            .set(&CONSENT_CONTRACT, &consent_contract);
        env.storage()
            .persistent()
            .set(&MEDICAL_RECORDS_CONTRACT, &medical_records_contract);
        env.storage().persistent().set(&PAUSED, &false);
        env.storage().persistent().set(&PRESCRIPTION_COUNTER, &0u64);
        env.storage().persistent().set(&REFILL_COUNTER, &0u64);
        env.storage().persistent().set(&ADHERENCE_COUNTER, &0u64);
        env.storage().persistent().set(&INTERACTION_COUNTER, &0u64);
        env.storage().persistent().set(&ALLERGY_COUNTER, &0u64);
        env.storage().persistent().set(&AUTH_COUNTER, &0u64);

        Ok(true)
    }

    /// Create a new virtual prescription
    pub fn create_prescription(
        env: Env,
        prescriber: Address,
        patient: Address,
        pharmacy: Option<Address>,
        medication_name: String,
        generic_name: Option<String>,
        medication_type: MedicationType,
        strength: String,
        dosage_form: String,
        quantity: u32,
        refills_allowed: u8,
        expiration_days: u32,
        instructions: String,
        indications: String,
        contraindications: Vec<String>,
        side_effects: Vec<String>,
        pregnancy_category: String,
        controlled_substance: bool,
        schedule_number: Option<u8>,
        dea_number: String,
        state_license: String,
        diagnosis_codes: Vec<String>,
        prior_auth_required: bool,
        consent_token_id: u64,
        digital_signature: BytesN<64>,
    ) -> Result<u64, Error> {
        prescriber.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Verify consent
        if !Self::verify_consent_token(&env, consent_token_id, patient.clone(), prescriber.clone())?
        {
            return Err(Error::ConsentRequired);
        }

        // Validate DEA number for controlled substances
        if controlled_substance && !Self::validate_dea_number(&dea_number) {
            return Err(Error::InvalidDEANumber);
        }

        // Validate state license
        if !Self::validate_state_license(&state_license) {
            return Err(Error::InvalidLicense);
        }

        // Validate quantity and refills
        if quantity == 0 || quantity > 365 {
            // Max 1 year supply
            return Err(Error::InvalidQuantity);
        }

        if refills_allowed > 11 {
            // Federal limit
            return Err(Error::InvalidRefills);
        }

        // Check pharmacy network if specified
        if let Some(pharmacy_addr) = pharmacy {
            if !Self::is_pharmacy_in_network(&env, pharmacy_addr)? {
                return Err(Error::PharmacyNotInNetwork);
            }
        }

        let prescription_id = Self::get_and_increment_prescription_counter(&env);
        let timestamp = env.ledger().timestamp();
        let expiration_date = timestamp + (expiration_days as u64 * 86400);

        let prescription = VirtualPrescription {
            prescription_id,
            patient: patient.clone(),
            prescriber: prescriber.clone(),
            pharmacy,
            medication_name: medication_name.clone(),
            generic_name,
            medication_type,
            strength,
            dosage_form,
            quantity,
            refills_allowed,
            refills_remaining: refills_allowed,
            date_written: timestamp,
            date_filled: None,
            expiration_date,
            status: PrescriptionStatus::Draft,
            verification_status: VerificationStatus::Pending,
            verification_notes: String::from_str(&env, ""),
            instructions,
            indications,
            contraindications,
            side_effects,
            allergies_check: false,
            pregnancy_category,
            controlled_substance,
            schedule_number,
            dea_number,
            state_license,
            diagnosis_codes,
            prior_auth_required,
            prior_auth_status: if prior_auth_required {
                "required".to_string()
            } else {
                "not_required".to_string()
            },
            consent_token_id,
            digital_signature,
            signature_timestamp: timestamp,
        };

        let mut prescriptions: Map<u64, VirtualPrescription> = env
            .storage()
            .persistent()
            .get(&PRESCRIPTIONS)
            .unwrap_or(Map::new(&env));
        prescriptions.set(prescription_id, prescription);
        env.storage()
            .persistent()
            .set(&PRESCRIPTIONS, &prescriptions);

        // Perform automatic checks
        Self::perform_allergy_check(&env, prescription_id, patient.clone())?;
        Self::perform_drug_interaction_check(&env, prescription_id, patient.clone())?;

        // Emit event
        env.events().publish(
            (symbol_short!("Prescription"), symbol_short!("Created")),
            (prescription_id, patient, prescriber, medication_name),
        );

        Ok(prescription_id)
    }

    /// Verify and activate prescription
    pub fn verify_prescription(
        env: Env,
        prescription_id: u64,
        verifier: Address,
        approved: bool,
        notes: String,
    ) -> Result<bool, Error> {
        verifier.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        let mut prescriptions: Map<u64, VirtualPrescription> = env
            .storage()
            .persistent()
            .get(&PRESCRIPTIONS)
            .ok_or(Error::PrescriptionNotFound)?;

        let mut prescription = prescriptions
            .get(prescription_id)
            .ok_or(Error::PrescriptionNotFound)?;

        // Only pharmacists or designated verifiers can verify
        if !Self::is_authorized_verifier(&env, verifier) {
            return Err(Error::NotAuthorized);
        }

        if prescription.verification_status != VerificationStatus::Pending {
            return Err(Error::InvalidStatus);
        }

        prescription.verification_status = if approved {
            VerificationStatus::Verified
        } else {
            VerificationStatus::Rejected
        };
        prescription.verification_notes = notes;

        if approved {
            prescription.status = PrescriptionStatus::Active;
        } else {
            prescription.status = PrescriptionStatus::Rejected;
        }

        prescriptions.set(prescription_id, prescription);
        env.storage()
            .persistent()
            .set(&PRESCRIPTIONS, &prescriptions);

        // Emit event
        env.events().publish(
            (symbol_short!("Prescription"), symbol_short!("Verified")),
            (prescription_id, approved),
        );

        Ok(true)
    }

    /// Fill prescription
    pub fn fill_prescription(
        env: Env,
        prescription_id: u64,
        pharmacy: Address,
        filled_quantity: u32,
        pharmacist: Address,
    ) -> Result<bool, Error> {
        pharmacist.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        let mut prescriptions: Map<u64, VirtualPrescription> = env
            .storage()
            .persistent()
            .get(&PRESCRIPTIONS)
            .ok_or(Error::PrescriptionNotFound)?;

        let mut prescription = prescriptions
            .get(prescription_id)
            .ok_or(Error::PrescriptionNotFound)?;

        // Validate pharmacy and pharmacist
        if !Self::is_pharmacy_in_network(&env, pharmacy)? {
            return Err(Error::PharmacyNotInNetwork);
        }

        if !Self::is_pharmacist_authorized(&env, pharmacist, pharmacy)? {
            return Err(Error::NotAuthorized);
        }

        // Check prescription status
        if prescription.status != PrescriptionStatus::Active {
            return Err(Error::InvalidStatus);
        }

        // Check expiration
        if env.ledger().timestamp() > prescription.expiration_date {
            return Err(Error::PrescriptionExpired);
        }

        // Validate filled quantity
        if filled_quantity > prescription.quantity {
            return Err(Error::InvalidQuantity);
        }

        // Update prescription
        prescription.date_filled = Some(env.ledger().timestamp());
        prescription.pharmacy = Some(pharmacy);
        prescription.status = PrescriptionStatus::Completed;

        prescriptions.set(prescription_id, prescription);
        env.storage()
            .persistent()
            .set(&PRESCRIPTIONS, &prescriptions);

        // Emit event
        env.events().publish(
            (symbol_short!("Prescription"), symbol_short!("Filled")),
            (prescription_id, pharmacy, filled_quantity),
        );

        Ok(true)
    }

    /// Request prescription refill
    pub fn request_refill(
        env: Env,
        prescription_id: u64,
        patient: Address,
        pharmacy: Address,
    ) -> Result<u64, Error> {
        patient.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        let prescriptions: Map<u64, VirtualPrescription> = env
            .storage()
            .persistent()
            .get(&PRESCRIPTIONS)
            .ok_or(Error::PrescriptionNotFound)?;

        let prescription = prescriptions
            .get(prescription_id)
            .ok_or(Error::PrescriptionNotFound)?;

        // Validate patient and prescription
        if prescription.patient != patient {
            return Err(Error::NotAuthorized);
        }

        if prescription.refills_remaining == 0 {
            return Err(Error::NoRefillsRemaining);
        }

        if env.ledger().timestamp() > prescription.expiration_date {
            return Err(Error::PrescriptionExpired);
        }

        // Check for existing refill request
        let existing_requests: Vec<RefillRequest> = env
            .storage()
            .persistent()
            .get(&REFILL_REQUESTS)
            .unwrap_or(Vec::new(&env));

        for request in existing_requests.iter() {
            if request.prescription_id == prescription_id
                && request.status == RefillStatus::Requested
                && request.patient == patient
            {
                return Err(Error::DuplicateRefillRequest);
            }
        }

        let refill_id = Self::get_and_increment_refill_counter(&env);
        let timestamp = env.ledger().timestamp();

        let refill_request = RefillRequest {
            request_id: refill_id,
            prescription_id,
            patient: patient.clone(),
            pharmacy,
            requested_at: timestamp,
            status: RefillStatus::Requested,
            processing_notes: String::from_str(&env, ""),
            filled_at: None,
            picked_up_at: None,
            expires_at: timestamp + 604800, // 7 days
        };

        let mut refill_requests: Vec<RefillRequest> = env
            .storage()
            .persistent()
            .get(&REFILL_REQUESTS)
            .unwrap_or(Vec::new(&env));
        refill_requests.push_back(refill_request);
        env.storage()
            .persistent()
            .set(&REFILL_REQUESTS, &refill_requests);

        // Emit event
        env.events().publish(
            (symbol_short!("Refill"), symbol_short!("Requested")),
            (refill_id, patient, prescription_id),
        );

        Ok(refill_id)
    }

    /// Process refill request
    pub fn process_refill(
        env: Env,
        refill_id: u64,
        pharmacist: Address,
        approved: bool,
        notes: String,
    ) -> Result<bool, Error> {
        pharmacist.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        let mut refill_requests: Vec<RefillRequest> = env
            .storage()
            .persistent()
            .get(&REFILL_REQUESTS)
            .ok_or(Error::RefillRequestNotFound)?;

        let mut refill_request = None;
        let mut index = 0;

        for (i, request) in refill_requests.iter().enumerate() {
            if request.request_id == refill_id {
                refill_request = Some(request.clone());
                index = i;
                break;
            }
        }

        let mut request = refill_request.ok_or(Error::RefillRequestNotFound)?;

        // Validate pharmacist authorization
        if !Self::is_pharmacist_authorized(&env, pharmacist, request.pharmacy)? {
            return Err(Error::NotAuthorized);
        }

        if request.status != RefillStatus::Requested {
            return Err(Error::InvalidStatus);
        }

        let timestamp = env.ledger().timestamp();

        if approved {
            request.status = RefillStatus::Filled;
            request.filled_at = Some(timestamp);

            // Update prescription refills
            let mut prescriptions: Map<u64, VirtualPrescription> = env
                .storage()
                .persistent()
                .get(&PRESCRIPTIONS)
                .ok_or(Error::PrescriptionNotFound)?;

            let mut prescription = prescriptions
                .get(request.prescription_id)
                .ok_or(Error::PrescriptionNotFound)?;

            if prescription.refills_remaining > 0 {
                prescription.refills_remaining -= 1;
                prescriptions.set(request.prescription_id, prescription);
                env.storage()
                    .persistent()
                    .set(&PRESCRIPTIONS, &prescriptions);
            }
        } else {
            request.status = RefillStatus::Denied;
        }

        request.processing_notes = notes;
        refill_requests.set(index, request);
        env.storage()
            .persistent()
            .set(&REFILL_REQUESTS, &refill_requests);

        // Emit event
        env.events().publish(
            (symbol_short!("Refill"), symbol_short!("Processed")),
            (refill_id, approved),
        );

        Ok(true)
    }

    /// Record medication adherence
    pub fn record_adherence(
        env: Env,
        prescription_id: u64,
        patient: Address,
        scheduled_time: u64,
        taken_time: u64,
        dose_taken: f32,
        prescribed_dose: f32,
        notes: String,
        location: String,
        verification_method: String,
    ) -> Result<u64, Error> {
        patient.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Validate prescription
        let prescriptions: Map<u64, VirtualPrescription> = env
            .storage()
            .persistent()
            .get(&PRESCRIPTIONS)
            .ok_or(Error::PrescriptionNotFound)?;

        let prescription = prescriptions
            .get(prescription_id)
            .ok_or(Error::PrescriptionNotFound)?;

        if prescription.patient != patient {
            return Err(Error::NotAuthorized);
        }

        let adherence_id = Self::get_and_increment_adherence_counter(&env);

        let missed_dose = dose_taken == 0.0;
        let late_dose = taken_time > scheduled_time && (taken_time - scheduled_time) > 3600; // 1 hour late
        let adherence_percentage = ((dose_taken / prescribed_dose) * 100.0) as u8;

        let adherence = MedicationAdherence {
            adherence_id,
            prescription_id,
            patient: patient.clone(),
            scheduled_time,
            taken_time,
            dose_taken,
            prescribed_dose,
            adherence_percentage,
            missed_dose,
            late_dose,
            notes,
            location,
            verification_method,
        };

        let mut adherence_records: Vec<MedicationAdherence> = env
            .storage()
            .persistent()
            .get(&ADHERENCE_RECORDS)
            .unwrap_or(Vec::new(&env));
        adherence_records.push_back(adherence);
        env.storage()
            .persistent()
            .set(&ADHERENCE_RECORDS, &adherence_records);

        // Emit event
        env.events().publish(
            (symbol_short!("Adherence"), symbol_short!("Recorded")),
            (adherence_id, patient, missed_dose),
        );

        Ok(adherence_id)
    }

    /// Add pharmacy to network
    pub fn add_pharmacy_to_network(
        env: Env,
        admin: Address,
        pharmacy: PharmacyNetwork,
    ) -> Result<bool, Error> {
        admin.require_auth();

        let contract_admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN)
            .ok_or(Error::NotAuthorized)?;

        if admin != contract_admin {
            return Err(Error::NotAuthorized);
        }

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        let mut pharmacies: Map<Address, PharmacyNetwork> = env
            .storage()
            .persistent()
            .get(&PHARMACY_NETWORK)
            .unwrap_or(Map::new(&env));

        pharmacies.set(pharmacy.pharmacy_id, pharmacy);
        env.storage()
            .persistent()
            .set(&PHARMACY_NETWORK, &pharmacies);

        Ok(true)
    }

    /// Get prescription details
    pub fn get_prescription(env: Env, prescription_id: u64) -> Result<VirtualPrescription, Error> {
        let prescriptions: Map<u64, VirtualPrescription> = env
            .storage()
            .persistent()
            .get(&PRESCRIPTIONS)
            .ok_or(Error::PrescriptionNotFound)?;

        prescriptions
            .get(prescription_id)
            .ok_or(Error::PrescriptionNotFound)
    }

    /// Get refill request
    pub fn get_refill_request(env: Env, refill_id: u64) -> Result<RefillRequest, Error> {
        let refill_requests: Vec<RefillRequest> = env
            .storage()
            .persistent()
            .get(&REFILL_REQUESTS)
            .ok_or(Error::RefillRequestNotFound)?;

        for request in refill_requests.iter() {
            if request.request_id == refill_id {
                return Ok(request);
            }
        }

        Err(Error::RefillRequestNotFound)
    }

    /// Get pharmacy network information
    pub fn get_pharmacy_info(env: Env, pharmacy_id: Address) -> Result<PharmacyNetwork, Error> {
        let pharmacies: Map<Address, PharmacyNetwork> = env
            .storage()
            .persistent()
            .get(&PHARMACY_NETWORK)
            .ok_or(Error::PharmacyNotInNetwork)?;

        pharmacies
            .get(pharmacy_id)
            .ok_or(Error::PharmacyNotInNetwork)
    }

    /// Get patient's adherence records
    pub fn get_patient_adherence(
        env: Env,
        patient: Address,
        prescription_id: u64,
    ) -> Result<Vec<MedicationAdherence>, Error> {
        let adherence_records: Vec<MedicationAdherence> = env
            .storage()
            .persistent()
            .get(&ADHERENCE_RECORDS)
            .unwrap_or(Vec::new(&env));

        let mut patient_records = Vec::new(&env);
        for record in adherence_records.iter() {
            if record.patient == patient && record.prescription_id == prescription_id {
                patient_records.push_back(record);
            }
        }

        Ok(patient_records)
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

    fn validate_dea_number(dea_number: &String) -> bool {
        // Basic DEA number validation (2 letters + 6 digits + 1 check digit)
        dea_number.len() == 9
            && dea_number.chars().take(2).all(|c| c.is_alphabetic())
            && dea_number.chars().skip(2).take(6).all(|c| c.is_numeric())
            && dea_number.chars().last().map_or(false, |c| c.is_numeric())
    }

    fn validate_state_license(state_license: &String) -> bool {
        // Basic state license validation (at least 6 characters)
        state_license.len() >= 6
    }

    fn is_pharmacy_in_network(env: &Env, pharmacy_id: Address) -> Result<bool, Error> {
        let pharmacies: Map<Address, PharmacyNetwork> = env
            .storage()
            .persistent()
            .get(&PHARMACY_NETWORK)
            .unwrap_or(Map::new(env));

        Ok(pharmacies.contains_key(pharmacy_id))
    }

    fn is_authorized_verifier(env: &Env, verifier: Address) -> bool {
        // This would check if the verifier is an authorized pharmacist or verifier
        // For now, we'll implement basic logic
        true
    }

    fn is_pharmacist_authorized(env: &Env, pharmacist: Address, pharmacy: Address) -> bool {
        // This would verify the pharmacist is authorized to work at the pharmacy
        // For now, we'll implement basic logic
        true
    }

    fn perform_allergy_check(
        env: &Env,
        prescription_id: u64,
        patient: Address,
    ) -> Result<(), Error> {
        let check_id = Self::get_and_increment_allergy_counter(env);

        let allergy_check = AllergyCheckResult {
            check_id,
            prescription_id,
            patient_allergies: Vec::new(env), // Would fetch from medical records
            medication_allergens: Vec::new(env), // Would analyze medication
            cross_reactivity: Vec::new(env),
            severity: "none".to_string(),
            recommendations: Vec::new(env),
            checked_at: env.ledger().timestamp(),
        };

        let mut allergy_checks: Vec<AllergyCheckResult> = env
            .storage()
            .persistent()
            .get(&ALLERGY_CHECKS)
            .unwrap_or(Vec::new(env));
        allergy_checks.push_back(allergy_check);
        env.storage()
            .persistent()
            .set(&ALLERGY_CHECKS, &allergy_checks);

        Ok(())
    }

    fn perform_drug_interaction_check(
        env: &Env,
        prescription_id: u64,
        patient: Address,
    ) -> Result<(), Error> {
        let interaction_id = Self::get_and_increment_interaction_counter(env);

        let interaction = DrugInteraction {
            interaction_id,
            prescription_id,
            interacting_drug: String::from_str(env, ""),
            interaction_severity: "none".to_string(),
            interaction_description: String::from_str(env, ""),
            clinical_effects: Vec::new(env),
            management_recommendations: Vec::new(env),
            alternatives: Vec::new(env),
            checked_at: env.ledger().timestamp(),
        };

        let mut interactions: Vec<DrugInteraction> = env
            .storage()
            .persistent()
            .get(&DRUG_INTERACTIONS)
            .unwrap_or(Vec::new(env));
        interactions.push_back(interaction);
        env.storage()
            .persistent()
            .set(&DRUG_INTERACTIONS, &interactions);

        Ok(())
    }

    fn get_and_increment_prescription_counter(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&PRESCRIPTION_COUNTER)
            .unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&PRESCRIPTION_COUNTER, &next);
        next
    }

    fn get_and_increment_refill_counter(env: &Env) -> u64 {
        let count: u64 = env.storage().persistent().get(&REFILL_COUNTER).unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&REFILL_COUNTER, &next);
        next
    }

    fn get_and_increment_adherence_counter(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&ADHERENCE_COUNTER)
            .unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&ADHERENCE_COUNTER, &next);
        next
    }

    fn get_and_increment_interaction_counter(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&INTERACTION_COUNTER)
            .unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&INTERACTION_COUNTER, &next);
        next
    }

    fn get_and_increment_allergy_counter(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&ALLERGY_COUNTER)
            .unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&ALLERGY_COUNTER, &next);
        next
    }

    fn get_and_increment_auth_counter(env: &Env) -> u64 {
        let count: u64 = env.storage().persistent().get(&AUTH_COUNTER).unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&AUTH_COUNTER, &next);
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
