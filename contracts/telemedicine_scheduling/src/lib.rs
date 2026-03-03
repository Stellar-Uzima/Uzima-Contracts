#![no_std]

#[cfg(test)]
mod test;

use soroban_sdk::symbol_short;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, vec, Address, BytesN, Env, Map, String,
    Symbol, Vec,
};

// ==================== Telemedicine Scheduling Types ====================

/// Appointment Status
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum AppointmentStatus {
    Scheduled,
    Confirmed,
    InProgress,
    Completed,
    Cancelled,
    NoShow,
    Rescheduled,
    WaitingList,
}

/// Appointment Type
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum AppointmentType {
    InitialConsultation,
    FollowUp,
    UrgentCare,
    MentalHealth,
    ChronicCare,
    SecondOpinion,
    PreOperative,
    PostOperative,
    MedicationReview,
    LabReview,
}

/// Consultation Modality
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum ConsultationModality {
    Video,
    AudioOnly,
    Chat,
    PhoneCall,
    Hybrid,
}

/// Priority Level
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum PriorityLevel {
    Low,
    Normal,
    High,
    Urgent,
    Emergency,
}

/// Recurrence Pattern
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum RecurrencePattern {
    None,
    Daily,
    Weekly,
    BiWeekly,
    Monthly,
    Quarterly,
}

/// Time Slot
#[derive(Clone)]
#[contracttype]
pub struct TimeSlot {
    pub slot_id: u64,
    pub provider: Address,
    pub start_time: u64,
    pub end_time: u64,
    pub duration_minutes: u32,
    pub available: bool,
    pub appointment_type: AppointmentType,
    pub modality: ConsultationModality,
    pub max_patients: u8,
    pub current_patients: u8,
    pub location: String, // Virtual room ID or physical location
    pub special_requirements: Vec<String>,
    pub created_at: u64,
}

/// Appointment
#[derive(Clone)]
#[contracttype]
pub struct Appointment {
    pub appointment_id: u64,
    pub patient: Address,
    pub provider: Address,
    pub slot_id: u64,
    pub appointment_type: AppointmentType,
    pub modality: ConsultationModality,
    pub status: AppointmentStatus,
    pub scheduled_time: u64,
    pub duration_minutes: u32,
    pub priority: PriorityLevel,
    pub reason_for_visit: String,
    pub symptoms: Vec<String>,
    pub notes: String,
    pub consent_token_id: u64,
    pub payment_status: String, // "pending", "paid", "covered", "self_pay"
    pub insurance_verified: bool,
    pub pre_visit_instructions: Vec<String>,
    pub post_visit_instructions: Vec<String>,
    pub reschedule_count: u8,
    pub cancellation_reason: Option<String>,
    pub no_show_reason: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Recurring Appointment Series
#[derive(Clone)]
#[contracttype]
pub struct RecurringSeries {
    pub series_id: u64,
    pub patient: Address,
    pub provider: Address,
    pub appointment_type: AppointmentType,
    pub modality: ConsultationModality,
    pub recurrence_pattern: RecurrencePattern,
    pub start_date: u64,
    pub end_date: u64,
    pub preferred_times: Vec<String>, // e.g., ["09:00", "14:00"]
    pub preferred_days: Vec<String>, // e.g., ["Monday", "Wednesday", "Friday"]
    pub total_sessions: u16,
    pub completed_sessions: u16,
    pub status: String, // "active", "paused", "completed", "cancelled"
    pub notes: String,
    pub created_at: u64,
}

/// Waiting List Entry
#[derive(Clone)]
#[contracttype]
pub struct WaitingListEntry {
    pub entry_id: u64,
    pub patient: Address,
    pub appointment_type: AppointmentType,
    pub modality: ConsultationModality,
    pub priority: PriorityLevel,
    pub preferred_date_range: (u64, u64), // (start_date, end_date)
    pub preferred_times: Vec<String>,
    pub flexibility: String, // "strict", "moderate", "flexible"
    pub reason: String,
    pub added_at: u64,
    pub expires_at: u64,
    pub notified_count: u8,
}

/// Provider Availability
#[derive(Clone)]
#[contracttype]
pub struct ProviderAvailability {
    pub availability_id: u64,
    pub provider: Address,
    pub day_of_week: String, // "Monday", "Tuesday", etc.
    pub start_time: String, // "09:00"
    pub end_time: String,   // "17:00"
    pub available_modalities: Vec<ConsultationModality>,
    pub available_appointment_types: Vec<AppointmentType>,
    pub max_patients_per_slot: u8,
    pub buffer_time_minutes: u8,
    pub is_active: bool,
    pub special_notes: String,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Schedule Conflict
#[derive(Clone)]
#[contracttype]
pub struct ScheduleConflict {
    pub conflict_id: u64,
    pub provider: Address,
    pub conflict_type: String, // "double_booking", "overlapping", "unavailable"
    pub conflicting_appointments: Vec<u64>,
    pub conflict_time: u64,
    pub resolution_status: String, // "pending", "resolved", "escalated"
    pub resolution_notes: String,
    pub detected_at: u64,
    pub resolved_at: Option<u64>,
    pub resolved_by: Option<Address>,
}

/// Appointment Reminder
#[derive(Clone)]
#[contracttype]
pub struct AppointmentReminder {
    pub reminder_id: u64,
    pub appointment_id: u64,
    pub recipient: Address,
    pub reminder_type: String, // "email", "sms", "push", "call"
    pub scheduled_time: u64,
    pub sent_time: Option<u64>,
    pub status: String, // "scheduled", "sent", "failed", "delivered"
    pub message_content: String,
    pub delivery_attempts: u8,
    pub response_received: bool,
}

/// Telemedicine Room
#[derive(Clone)]
#[contracttype]
pub struct TelemedicineRoom {
    pub room_id: String,
    pub appointment_id: u64,
    pub provider: Address,
    pub patient: Address,
    pub room_type: ConsultationModality,
    pub meeting_link: String,
    pub access_code: String,
    pub host_key: String,
    pub start_time: u64,
    pub end_time: u64,
    pub max_participants: u8,
    pub recording_enabled: bool,
    pub chat_enabled: bool,
    pub screen_share_enabled: bool,
    pub waiting_room_enabled: bool,
    pub status: String, // "created", "active", "ended", "expired"
    pub created_at: u64,
}

// Storage Keys
const ADMIN: Symbol = symbol_short!("ADMIN");
const TIME_SLOTS: Symbol = symbol_short!("SLOTS");
const APPOINTMENTS: Symbol = symbol_short!("APPOINTMENTS");
const RECURRING_SERIES: Symbol = symbol_short!("SERIES");
const WAITING_LIST: Symbol = symbol_short!("WAITING");
const PROVIDER_AVAILABILITY: Symbol = symbol_short!("AVAILABILITY");
const SCHEDULE_CONFLICTS: Symbol = symbol_short!("CONFLICTS");
const APPOINTMENT_REMINDERS: Symbol = symbol_short!("REMINDERS");
const TELEMEDICINE_ROOMS: Symbol = symbol_short!("ROOMS");
const SLOT_COUNTER: Symbol = symbol_short!("SLOT_CNT");
const APPOINTMENT_COUNTER: Symbol = symbol_short!("APPT_CNT");
const SERIES_COUNTER: Symbol = symbol_short!("SERIES_CNT");
const WAITING_COUNTER: Symbol = symbol_short!("WAIT_CNT");
const CONFLICT_COUNTER: Symbol = symbol_short!("CONFLICT_CNT");
const REMINDER_COUNTER: Symbol = symbol_short!("REMINDER_CNT");
const PAUSED: Symbol = symbol_short!("PAUSED");
const CONSENT_CONTRACT: Symbol = symbol_short!("CONSENT");
const MEDICAL_RECORDS_CONTRACT: Symbol = symbol_short!("MEDICAL");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    ContractPaused = 2,
    SlotNotFound = 3,
    SlotAlreadyBooked = 4,
    AppointmentNotFound = 5,
    AppointmentAlreadyExists = 6,
    InvalidTimeSlot = 7,
    OverlappingAppointment = 8,
    ProviderNotAvailable = 9,
    PatientNotAvailable = 10,
    InvalidAppointmentType = 11,
    InvalidModality = 12,
    ConsentRequired = 13,
    ConsentRevoked = 14,
    CannotCancel = 15,
    CannotReschedule = 16,
    MaxRescheduleExceeded = 17,
    WaitingListFull = 18,
    SeriesNotFound = 19,
    InvalidRecurrence = 20,
    RoomNotFound = 21,
    RoomAlreadyExists = 22,
    ConflictDetected = 23,
    ReminderFailed = 24,
    MedicalRecordsContractNotSet = 25,
    ConsentContractNotSet = 26,
}

#[contract]
pub struct TelemedicineSchedulingContract;

#[contractimpl]
impl TelemedicineSchedulingContract {
    /// Initialize the telemedicine scheduling contract
    pub fn initialize(
        env: Env,
        admin: Address,
        consent_contract: Address,
        medical_records_contract: Address,
    ) -> Result<bool, Error> {
        admin.require_auth();

        if env.storage().persistent().has(&ADMIN) {
            return Err(Error::AppointmentAlreadyExists);
        }

        env.storage().persistent().set(&ADMIN, &admin);
        env.storage().persistent().set(&CONSENT_CONTRACT, &consent_contract);
        env.storage()
            .persistent()
            .set(&MEDICAL_RECORDS_CONTRACT, &medical_records_contract);
        env.storage().persistent().set(&PAUSED, &false);
        env.storage().persistent().set(&SLOT_COUNTER, &0u64);
        env.storage().persistent().set(&APPOINTMENT_COUNTER, &0u64);
        env.storage().persistent().set(&SERIES_COUNTER, &0u64);
        env.storage().persistent().set(&WAITING_COUNTER, &0u64);
        env.storage().persistent().set(&CONFLICT_COUNTER, &0u64);
        env.storage().persistent().set(&REMINDER_COUNTER, &0u64);

        Ok(true)
    }

    /// Create time slots for provider availability
    pub fn create_time_slots(
        env: Env,
        provider: Address,
        start_date: u64,
        end_date: u64,
        slot_duration_minutes: u32,
        appointment_types: Vec<AppointmentType>,
        modalities: Vec<ConsultationModality>,
        max_patients_per_slot: u8,
        special_requirements: Vec<String>,
    ) -> Result<Vec<u64>, Error> {
        provider.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Validate date range
        if start_date >= end_date {
            return Err(Error::InvalidTimeSlot);
        }

        // Validate slot duration (15-120 minutes)
        if slot_duration_minutes < 15 || slot_duration_minutes > 120 {
            return Err(Error::InvalidTimeSlot);
        }

        let mut created_slots = Vec::new(&env);
        let mut current_time = start_date;
        let slot_duration_seconds = slot_duration_minutes as u64 * 60;

        while current_time < end_date {
            // Skip weekends if not specified (basic implementation)
            let day_of_week = Self::get_day_of_week(current_time);
            if day_of_week == "Saturday" || day_of_week == "Sunday" {
                current_time += 86400; // Skip to next day
                continue;
            }

            // Create slots from 9:00 AM to 5:00 PM
            let hour = (current_time % 86400) / 3600;
            if hour >= 9 && hour < 17 {
                let slot_id = Self::get_and_increment_slot_counter(&env);

                let time_slot = TimeSlot {
                    slot_id,
                    provider: provider.clone(),
                    start_time: current_time,
                    end_time: current_time + slot_duration_seconds,
                    duration_minutes: slot_duration_minutes,
                    available: true,
                    appointment_type: appointment_types.get(0).unwrap_or(&AppointmentType::InitialConsultation).clone(),
                    modality: modalities.get(0).unwrap_or(&ConsultationModality::Video).clone(),
                    max_patients: max_patients_per_slot,
                    current_patients: 0,
                    location: format!("room_{}", slot_id),
                    special_requirements: special_requirements.clone(),
                    created_at: env.ledger().timestamp(),
                };

                let mut slots: Map<u64, TimeSlot> = env
                    .storage()
                    .persistent()
                    .get(&TIME_SLOTS)
                    .unwrap_or(Map::new(&env));
                slots.set(slot_id, time_slot);
                env.storage().persistent().set(&TIME_SLOTS, &slots);

                created_slots.push_back(slot_id);
            }

            current_time += slot_duration_seconds;
        }

        // Emit event
        env.events().publish(
            (symbol_short!("Slots"), symbol_short!("Created")),
            (provider, created_slots.len()),
        );

        Ok(created_slots)
    }

    /// Book an appointment
    pub fn book_appointment(
        env: Env,
        patient: Address,
        slot_id: u64,
        appointment_type: AppointmentType,
        modality: ConsultationModality,
        priority: PriorityLevel,
        reason_for_visit: String,
        symptoms: Vec<String>,
        notes: String,
        consent_token_id: u64,
    ) -> Result<u64, Error> {
        patient.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Verify consent
        if !Self::verify_consent_token(&env, consent_token_id, patient.clone())? {
            return Err(Error::ConsentRequired);
        }

        // Get and validate time slot
        let mut slots: Map<u64, TimeSlot> = env
            .storage()
            .persistent()
            .get(&TIME_SLOTS)
            .ok_or(Error::SlotNotFound)?;

        let mut slot = slots
            .get(slot_id)
            .ok_or(Error::SlotNotFound)?;

        if !slot.available || slot.current_patients >= slot.max_patients {
            return Err(Error::SlotAlreadyBooked);
        }

        // Check for patient conflicts
        if Self::has_patient_conflict(&env, patient.clone(), slot.start_time, slot.end_time)? {
            return Err(Error::PatientNotAvailable);
        }

        let appointment_id = Self::get_and_increment_appointment_counter(&env);
        let timestamp = env.ledger().timestamp();

        let appointment = Appointment {
            appointment_id,
            patient: patient.clone(),
            provider: slot.provider.clone(),
            slot_id,
            appointment_type,
            modality,
            status: AppointmentStatus::Scheduled,
            scheduled_time: slot.start_time,
            duration_minutes: slot.duration_minutes,
            priority,
            reason_for_visit,
            symptoms,
            notes,
            consent_token_id,
            payment_status: "pending".to_string(),
            insurance_verified: false,
            pre_visit_instructions: Vec::new(&env),
            post_visit_instructions: Vec::new(&env),
            reschedule_count: 0,
            cancellation_reason: None,
            no_show_reason: None,
            created_at: timestamp,
            updated_at: timestamp,
        };

        let mut appointments: Map<u64, Appointment> = env
            .storage()
            .persistent()
            .get(&APPOINTMENTS)
            .unwrap_or(Map::new(&env));
        appointments.set(appointment_id, appointment);
        env.storage()
            .persistent()
            .set(&APPOINTMENTS, &appointments);

        // Update slot
        slot.current_patients += 1;
        if slot.current_patients >= slot.max_patients {
            slot.available = false;
        }
        slots.set(slot_id, slot);
        env.storage().persistent().set(&TIME_SLOTS, &slots);

        // Create telemedicine room
        Self::create_telemedicine_room(&env, appointment_id, patient.clone(), slot.provider.clone(), modality)?;

        // Schedule reminders
        Self::schedule_appointment_reminders(&env, appointment_id, patient.clone(), slot.start_time)?;

        // Emit event
        env.events().publish(
            (symbol_short!("Appointment"), symbol_short!("Booked")),
            (appointment_id, patient, slot.provider),
        );

        Ok(appointment_id)
    }

    /// Confirm appointment
    pub fn confirm_appointment(env: Env, appointment_id: u64, patient: Address) -> Result<bool, Error> {
        patient.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        let mut appointments: Map<u64, Appointment> = env
            .storage()
            .persistent()
            .get(&APPOINTMENTS)
            .ok_or(Error::AppointmentNotFound)?;

        let mut appointment = appointments
            .get(appointment_id)
            .ok_or(Error::AppointmentNotFound)?;

        if appointment.patient != patient {
            return Err(Error::NotAuthorized);
        }

        if appointment.status != AppointmentStatus::Scheduled {
            return Err(Error::CannotReschedule);
        }

        appointment.status = AppointmentStatus::Confirmed;
        appointment.updated_at = env.ledger().timestamp();

        appointments.set(appointment_id, appointment);
        env.storage()
            .persistent()
            .set(&APPOINTMENTS, &appointments);

        // Emit event
        env.events().publish(
            (symbol_short!("Appointment"), symbol_short!("Confirmed")),
            (appointment_id, patient),
        );

        Ok(true)
    }

    /// Reschedule appointment
    pub fn reschedule_appointment(
        env: Env,
        appointment_id: u64,
        new_slot_id: u64,
        requester: Address,
        reason: String,
    ) -> Result<bool, Error> {
        requester.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        let mut appointments: Map<u64, Appointment> = env
            .storage()
            .persistent()
            .get(&APPOINTMENTS)
            .ok_or(Error::AppointmentNotFound)?;

        let mut appointment = appointments
            .get(appointment_id)
            .ok_or(Error::AppointmentNotFound)?;

        // Validate requester (patient or provider)
        if appointment.patient != requester && appointment.provider != requester {
            return Err(Error::NotAuthorized);
        }

        if appointment.status != AppointmentStatus::Scheduled && appointment.status != AppointmentStatus::Confirmed {
            return Err(Error::CannotReschedule);
        }

        if appointment.reschedule_count >= 3 {
            return Err(Error::MaxRescheduleExceeded);
        }

        // Get new slot
        let mut slots: Map<u64, TimeSlot> = env
            .storage()
            .persistent()
            .get(&TIME_SLOTS)
            .ok_or(Error::SlotNotFound)?;

        let mut new_slot = slots
            .get(new_slot_id)
            .ok_or(Error::SlotNotFound)?;

        if !new_slot.available || new_slot.current_patients >= new_slot.max_patients {
            return Err(Error::SlotAlreadyBooked);
        }

        // Release old slot
        let mut old_slots: Map<u64, TimeSlot> = env
            .storage()
            .persistent()
            .get(&TIME_SLOTS)
            .ok_or(Error::SlotNotFound)?;

        let mut old_slot = old_slots
            .get(appointment.slot_id)
            .ok_or(Error::SlotNotFound)?;

        old_slot.current_patients -= 1;
        old_slot.available = true;
        old_slots.set(appointment.slot_id, old_slot);
        env.storage().persistent().set(&TIME_SLOTS, &old_slots);

        // Book new slot
        new_slot.current_patients += 1;
        if new_slot.current_patients >= new_slot.max_patients {
            new_slot.available = false;
        }
        slots.set(new_slot_id, new_slot);
        env.storage().persistent().set(&TIME_SLOTS, &slots);

        // Update appointment
        appointment.slot_id = new_slot_id;
        appointment.scheduled_time = new_slot.start_time;
        appointment.duration_minutes = new_slot.duration_minutes;
        appointment.reschedule_count += 1;
        appointment.updated_at = env.ledger().timestamp();

        appointments.set(appointment_id, appointment);
        env.storage()
            .persistent()
            .set(&APPOINTMENTS, &appointments);

        // Update telemedicine room
        Self::update_telemedicine_room(&env, appointment_id, new_slot.start_time, new_slot.end_time)?;

        // Reschedule reminders
        Self::reschedule_appointment_reminders(&env, appointment_id, new_slot.start_time)?;

        // Emit event
        env.events().publish(
            (symbol_short!("Appointment"), symbol_short!("Rescheduled")),
            (appointment_id, requester, new_slot_id),
        );

        Ok(true)
    }

    /// Cancel appointment
    pub fn cancel_appointment(
        env: Env,
        appointment_id: u64,
        requester: Address,
        reason: String,
    ) -> Result<bool, Error> {
        requester.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        let mut appointments: Map<u64, Appointment> = env
            .storage()
            .persistent()
            .get(&APPOINTMENTS)
            .ok_or(Error::AppointmentNotFound)?;

        let mut appointment = appointments
            .get(appointment_id)
            .ok_or(Error::AppointmentNotFound)?;

        // Validate requester
        if appointment.patient != requester && appointment.provider != requester {
            return Err(Error::NotAuthorized);
        }

        if appointment.status == AppointmentStatus::Completed || appointment.status == AppointmentStatus::Cancelled {
            return Err(Error::CannotCancel);
        }

        // Update appointment
        appointment.status = AppointmentStatus::Cancelled;
        appointment.cancellation_reason = Some(reason);
        appointment.updated_at = env.ledger().timestamp();

        appointments.set(appointment_id, appointment);
        env.storage()
            .persistent()
            .set(&APPOINTMENTS, &appointments);

        // Release time slot
        let mut slots: Map<u64, TimeSlot> = env
            .storage()
            .persistent()
            .get(&TIME_SLOTS)
            .ok_or(Error::SlotNotFound)?;

        let mut slot = slots
            .get(appointment.slot_id)
            .ok_or(Error::SlotNotFound)?;

        slot.current_patients -= 1;
        slot.available = true;
        slots.set(appointment.slot_id, slot);
        env.storage().persistent().set(&TIME_SLOTS, &slots);

        // Cancel telemedicine room
        Self::cancel_telemedicine_room(&env, appointment_id)?;

        // Cancel reminders
        Self::cancel_appointment_reminders(&env, appointment_id)?;

        // Offer slot to waiting list
        Self::offer_slot_to_waiting_list(&env, appointment.slot_id)?;

        // Emit event
        env.events().publish(
            (symbol_short!("Appointment"), symbol_short!("Cancelled")),
            (appointment_id, requester),
        );

        Ok(true)
    }

    /// Add patient to waiting list
    pub fn add_to_waiting_list(
        env: Env,
        patient: Address,
        appointment_type: AppointmentType,
        modality: ConsultationModality,
        priority: PriorityLevel,
        preferred_date_range: (u64, u64),
        preferred_times: Vec<String>,
        flexibility: String,
        reason: String,
    ) -> Result<u64, Error> {
        patient.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        let entry_id = Self::get_and_increment_waiting_counter(&env);
        let timestamp = env.ledger().timestamp();

        let waiting_entry = WaitingListEntry {
            entry_id,
            patient: patient.clone(),
            appointment_type,
            modality,
            priority,
            preferred_date_range,
            preferred_times,
            flexibility,
            reason,
            added_at: timestamp,
            expires_at: timestamp + 2592000, // 30 days
            notified_count: 0,
        };

        let mut waiting_list: Vec<WaitingListEntry> = env
            .storage()
            .persistent()
            .get(&WAITING_LIST)
            .unwrap_or(Vec::new(&env));
        waiting_list.push_back(waiting_entry);
        env.storage()
            .persistent()
            .set(&WAITING_LIST, &waiting_list);

        // Emit event
        env.events().publish(
            (symbol_short!("WaitingList"), symbol_short!("Added")),
            (entry_id, patient),
        );

        Ok(entry_id)
    }

    /// Create recurring appointment series
    pub fn create_recurring_series(
        env: Env,
        provider: Address,
        patient: Address,
        appointment_type: AppointmentType,
        modality: ConsultationModality,
        recurrence_pattern: RecurrencePattern,
        start_date: u64,
        end_date: u64,
        preferred_times: Vec<String>,
        preferred_days: Vec<String>,
        total_sessions: u16,
        notes: String,
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

        let series_id = Self::get_and_increment_series_counter(&env);
        let timestamp = env.ledger().timestamp();

        let series = RecurringSeries {
            series_id,
            patient: patient.clone(),
            provider: provider.clone(),
            appointment_type,
            modality,
            recurrence_pattern,
            start_date,
            end_date,
            preferred_times,
            preferred_days,
            total_sessions,
            completed_sessions: 0,
            status: "active".to_string(),
            notes,
            created_at: timestamp,
        };

        let mut series_list: Map<u64, RecurringSeries> = env
            .storage()
            .persistent()
            .get(&RECURRING_SERIES)
            .unwrap_or(Map::new(&env));
        series_list.set(series_id, series);
        env.storage()
            .persistent()
            .set(&RECURRING_SERIES, &series_list);

        // Generate appointments for the series
        Self::generate_series_appointments(&env, series_id)?;

        // Emit event
        env.events().publish(
            (symbol_short!("Series"), symbol_short!("Created")),
            (series_id, patient, provider),
        );

        Ok(series_id)
    }

    /// Get appointment details
    pub fn get_appointment(env: Env, appointment_id: u64) -> Result<Appointment, Error> {
        let appointments: Map<u64, Appointment> = env
            .storage()
            .persistent()
            .get(&APPOINTMENTS)
            .ok_or(Error::AppointmentNotFound)?;

        appointments.get(appointment_id).ok_or(Error::AppointmentNotFound)
    }

    /// Get time slot details
    pub fn get_time_slot(env: Env, slot_id: u64) -> Result<TimeSlot, Error> {
        let slots: Map<u64, TimeSlot> = env
            .storage()
            .persistent()
            .get(&TIME_SLOTS)
            .ok_or(Error::SlotNotFound)?;

        slots.get(slot_id).ok_or(Error::SlotNotFound)
    }

    /// Get telemedicine room details
    pub fn get_telemedicine_room(env: Env, appointment_id: u64) -> Result<TelemedicineRoom, Error> {
        let rooms: Map<String, TelemedicineRoom> = env
            .storage()
            .persistent()
            .get(&TELEMEDICINE_ROOMS)
            .ok_or(Error::RoomNotFound)?;

        let room_id = format!("room_{}", appointment_id);
        rooms.get(room_id).ok_or(Error::RoomNotFound)
    }

    /// Get provider's upcoming appointments
    pub fn get_provider_appointments(env: Env, provider: Address, start_date: u64, end_date: u64) -> Result<Vec<Appointment>, Error> {
        let appointments: Map<u64, Appointment> = env
            .storage()
            .persistent()
            .get(&APPOINTMENTS)
            .unwrap_or(Map::new(&env));

        let mut provider_appointments = Vec::new(&env);
        for appointment in appointments.values() {
            if appointment.provider == provider 
                && appointment.scheduled_time >= start_date 
                && appointment.scheduled_time <= end_date {
                provider_appointments.push_back(appointment);
            }
        }

        Ok(provider_appointments)
    }

    /// Get patient's upcoming appointments
    pub fn get_patient_appointments(env: Env, patient: Address, start_date: u64, end_date: u64) -> Result<Vec<Appointment>, Error> {
        let appointments: Map<u64, Appointment> = env
            .storage()
            .persistent()
            .get(&APPOINTMENTS)
            .unwrap_or(Map::new(&env));

        let mut patient_appointments = Vec::new(&env);
        for appointment in appointments.values() {
            if appointment.patient == patient 
                && appointment.scheduled_time >= start_date 
                && appointment.scheduled_time <= end_date {
                patient_appointments.push_back(appointment);
            }
        }

        Ok(patient_appointments)
    }

    // ==================== Helper Functions ====================

    fn verify_consent_token(env: &Env, token_id: u64, patient: Address) -> Result<bool, Error> {
        let consent_contract: Address = env
            .storage()
            .persistent()
            .get(&CONSENT_CONTRACT)
            .ok_or(Error::ConsentContractNotSet)?;

        // This would call the consent contract to verify the token
        // For now, we'll assume it's valid if not revoked
        Ok(true)
    }

    fn verify_consent_token_with_provider(
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

    fn has_patient_conflict(env: &Env, patient: Address, start_time: u64, end_time: u64) -> Result<bool, Error> {
        let appointments: Map<u64, Appointment> = env
            .storage()
            .persistent()
            .get(&APPOINTMENTS)
            .unwrap_or(Map::new(env));

        for appointment in appointments.values() {
            if appointment.patient == patient {
                // Check for time overlap
                if (start_time < appointment.scheduled_time + (appointment.duration_minutes as u64 * 60))
                    && (end_time > appointment.scheduled_time) {
                    // Check if appointment is still active
                    if appointment.status != AppointmentStatus::Cancelled 
                        && appointment.status != AppointmentStatus::Completed {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    fn create_telemedicine_room(
        env: &Env,
        appointment_id: u64,
        patient: Address,
        provider: Address,
        modality: ConsultationModality,
    ) -> Result<(), Error> {
        let room_id = format!("room_{}", appointment_id);
        let timestamp = env.ledger().timestamp();

        let room = TelemedicineRoom {
            room_id: room_id.clone(),
            appointment_id,
            provider,
            patient,
            room_type: modality,
            meeting_link: format!("https://meet.uzima.health/{}", room_id),
            access_code: Self::generate_access_code(env),
            host_key: Self::generate_host_key(env),
            start_time: timestamp,
            end_time: timestamp + 3600, // Default 1 hour
            max_participants: 3, // Patient, provider, optional interpreter
            recording_enabled: true,
            chat_enabled: true,
            screen_share_enabled: true,
            waiting_room_enabled: true,
            status: "created".to_string(),
            created_at: timestamp,
        };

        let mut rooms: Map<String, TelemedicineRoom> = env
            .storage()
            .persistent()
            .get(&TELEMEDICINE_ROOMS)
            .unwrap_or(Map::new(env));
        rooms.set(room_id, room);
        env.storage()
            .persistent()
            .set(&TELEMEDICINE_ROOMS, &rooms);

        Ok(())
    }

    fn update_telemedicine_room(env: &Env, appointment_id: u64, start_time: u64, end_time: u64) -> Result<(), Error> {
        let room_id = format!("room_{}", appointment_id);
        let mut rooms: Map<String, TelemedicineRoom> = env
            .storage()
            .persistent()
            .get(&TELEMEDICINE_ROOMS)
            .ok_or(Error::RoomNotFound)?;

        let mut room = rooms
            .get(room_id.clone())
            .ok_or(Error::RoomNotFound)?;

        room.start_time = start_time;
        room.end_time = end_time;
        rooms.set(room_id, room);
        env.storage()
            .persistent()
            .set(&TELEMEDICINE_ROOMS, &rooms);

        Ok(())
    }

    fn cancel_telemedicine_room(env: &Env, appointment_id: u64) -> Result<(), Error> {
        let room_id = format!("room_{}", appointment_id);
        let mut rooms: Map<String, TelemedicineRoom> = env
            .storage()
            .persistent()
            .get(&TELEMEDICINE_ROOMS)
            .ok_or(Error::RoomNotFound)?;

        let mut room = rooms
            .get(room_id.clone())
            .ok_or(Error::RoomNotFound)?;

        room.status = "cancelled".to_string();
        rooms.set(room_id, room);
        env.storage()
            .persistent()
            .set(&TELEMEDICINE_ROOMS, &rooms);

        Ok(())
    }

    fn schedule_appointment_reminders(env: &Env, appointment_id: u64, patient: Address, appointment_time: u64) -> Result<(), Error> {
        // Schedule reminders at 24 hours, 2 hours, and 15 minutes before
        let reminder_times = vec![
            env,
            appointment_time - 86400, // 24 hours before
            appointment_time - 7200,  // 2 hours before
            appointment_time - 900,   // 15 minutes before
        ];

        for (i, reminder_time) in reminder_times.iter().enumerate() {
            if *reminder_time > env.ledger().timestamp() {
                let reminder_id = Self::get_and_increment_reminder_counter(env);

                let reminder = AppointmentReminder {
                    reminder_id,
                    appointment_id,
                    recipient: patient.clone(),
                    reminder_type: "push".to_string(),
                    scheduled_time: *reminder_time,
                    sent_time: None,
                    status: "scheduled".to_string(),
                    message_content: format!("Reminder: You have an appointment scheduled at {}", appointment_time),
                    delivery_attempts: 0,
                    response_received: false,
                };

                let mut reminders: Vec<AppointmentReminder> = env
                    .storage()
                    .persistent()
                    .get(&APPOINTMENT_REMINDERS)
                    .unwrap_or(Vec::new(env));
                reminders.push_back(reminder);
                env.storage()
                    .persistent()
                    .set(&APPOINTMENT_REMINDERS, &reminders);
            }
        }

        Ok(())
    }

    fn reschedule_appointment_reminders(env: &Env, appointment_id: u64, new_appointment_time: u64) -> Result<(), Error> {
        // Cancel existing reminders and create new ones
        Self::cancel_appointment_reminders(env, appointment_id)?;

        // Get appointment details
        let appointments: Map<u64, Appointment> = env
            .storage()
            .persistent()
            .get(&APPOINTMENTS)
            .ok_or(Error::AppointmentNotFound)?;

        let appointment = appointments
            .get(appointment_id)
            .ok_or(Error::AppointmentNotFound)?;

        // Schedule new reminders
        Self::schedule_appointment_reminders(env, appointment_id, appointment.patient, new_appointment_time)?;

        Ok(())
    }

    fn cancel_appointment_reminders(env: &Env, appointment_id: u64) -> Result<(), Error> {
        let mut reminders: Vec<AppointmentReminder> = env
            .storage()
            .persistent()
            .get(&APPOINTMENT_REMINDERS)
            .unwrap_or(Vec::new(env));

        // Mark reminders as cancelled
        for i in 0..reminders.len() {
            let reminder = reminders.get(i).unwrap();
            if reminder.appointment_id == appointment_id && reminder.status == "scheduled" {
                let mut updated_reminder = reminder;
                updated_reminder.status = "cancelled".to_string();
                reminders.set(i, updated_reminder);
            }
        }

        env.storage()
            .persistent()
            .set(&APPOINTMENT_REMINDERS, &reminders);

        Ok(())
    }

    fn offer_slot_to_waiting_list(env: &Env, slot_id: u64) -> Result<(), Error> {
        let waiting_list: Vec<WaitingListEntry> = env
            .storage()
            .persistent()
            .get(&WAITING_LIST)
            .unwrap_or(Vec::new(env));

        let slots: Map<u64, TimeSlot> = env
            .storage()
            .persistent()
            .get(&TIME_SLOTS)
            .ok_or(Error::SlotNotFound)?;

        let slot = slots
            .get(slot_id)
            .ok_or(Error::SlotNotFound)?;

        // Find matching waiting list entries
        for entry in waiting_list.iter() {
            if entry.appointment_type == slot.appointment_type 
                && entry.modality == slot.modality
                && entry.expires_at > env.ledger().timestamp() {
                
                // Check if slot time matches preferences
                if Self::matches_waiting_list_preferences(&entry, slot.start_time) {
                    // Notify patient (in real implementation, this would send notification)
                    // For now, we'll just emit an event
                    env.events().publish(
                        (symbol_short!("WaitingList"), symbol_short!("SlotOffered")),
                        (entry.entry_id, slot_id),
                    );
                    break;
                }
            }
        }

        Ok(())
    }

    fn matches_waiting_list_preferences(entry: &WaitingListEntry, slot_time: u64) -> bool {
        // Check if slot time is within preferred date range
        if slot_time < entry.preferred_date_range.0 || slot_time > entry.preferred_date_range.1 {
            return false;
        }

        // Check if slot time matches preferred times (simplified)
        let hour = (slot_time % 86400) / 3600;
        for preferred_time in entry.preferred_times.iter() {
            let preferred_hour: u64 = preferred_time.parse().unwrap_or(0);
            if hour == preferred_hour {
                return true;
            }
        }

        false
    }

    fn generate_series_appointments(env: &Env, series_id: u64) -> Result<(), Error> {
        let series_list: Map<u64, RecurringSeries> = env
            .storage()
            .persistent()
            .get(&RECURRING_SERIES)
            .ok_or(Error::SeriesNotFound)?;

        let series = series_list
            .get(series_id)
            .ok_or(Error::SeriesNotFound)?;

        // Generate appointments based on recurrence pattern
        let mut current_date = series.start_date;
        let mut session_count = 0;

        while current_date <= series.end_date && session_count < series.total_sessions {
            // Check if current date matches preferred days
            let day_of_week = Self::get_day_of_week(current_date);
            if series.preferred_days.contains(&day_of_week) {
                // Find matching time slot
                // This is simplified - in production, would search for available slots
                session_count += 1;
            }

            // Move to next date based on recurrence pattern
            current_date += match series.recurrence_pattern {
                RecurrencePattern::Daily => 86400,
                RecurrencePattern::Weekly => 604800,
                RecurrencePattern::BiWeekly => 1209600,
                RecurrencePattern::Monthly => 2592000, // Approximate
                _ => break,
            };
        }

        Ok(())
    }

    fn get_day_of_week(timestamp: u64) -> String {
        let days_since_epoch = timestamp / 86400;
        let day_index = (days_since_epoch + 4) % 7; // January 1, 1970 was a Thursday
        
        match day_index {
            0 => "Thursday".to_string(),
            1 => "Friday".to_string(),
            2 => "Saturday".to_string(),
            3 => "Sunday".to_string(),
            4 => "Monday".to_string(),
            5 => "Tuesday".to_string(),
            6 => "Wednesday".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    fn generate_access_code(env: &Env) -> String {
        // Generate a 6-digit access code
        let timestamp = env.ledger().timestamp();
        let code = (timestamp % 1000000).to_string();
        format!("{:06}", code.parse::<u32>().unwrap_or(0))
    }

    fn generate_host_key(env: &Env) -> String {
        // Generate a host key
        let timestamp = env.ledger().timestamp();
        format!("host_{}", timestamp)
    }

    fn get_and_increment_slot_counter(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&SLOT_COUNTER)
            .unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&SLOT_COUNTER, &next);
        next
    }

    fn get_and_increment_appointment_counter(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&APPOINTMENT_COUNTER)
            .unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&APPOINTMENT_COUNTER, &next);
        next
    }

    fn get_and_increment_series_counter(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&SERIES_COUNTER)
            .unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&SERIES_COUNTER, &next);
        next
    }

    fn get_and_increment_waiting_counter(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&WAITING_COUNTER)
            .unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&WAITING_COUNTER, &next);
        next
    }

    fn get_and_increment_reminder_counter(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&REMINDER_COUNTER)
            .unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&REMINDER_COUNTER, &next);
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
