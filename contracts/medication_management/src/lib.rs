#![no_std]
#![allow(clippy::too_many_arguments)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String, Vec,
};

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    MedicationNotFound = 4,
    ScheduleNotFound = 5,
    InvalidData = 6,
    RefillNotFound = 7,
    InteractionAlreadyExists = 8,
    DuplicateMedication = 9,
    DoseAlreadyRecorded = 10,
    AutoRefillDisabled = 11,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum MedicationSource {
    Fda,
    ManualClinicalEntry,
    FhirMedicationStatement,
    TelemedicinePrescription,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum Severity {
    Low,
    Moderate,
    High,
    Contraindicated,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ScheduleStatus {
    Active,
    Paused,
    Completed,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum AdherenceEventStatus {
    Taken,
    Missed,
    Skipped,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum RefillStatus {
    Monitoring,
    ReminderDue,
    Requested,
    Processing,
    Fulfilled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum DosingSchedule {
    OnceDaily,
    TwiceDaily,
    ThreeTimesDaily,
    EveryNHours(u32),
    EveryNDays(u32),
    Weekly,
    SpecificTimes(Vec<u32>),
}

#[derive(Clone)]
#[contracttype]
pub struct Config {
    pub admin: Address,
    pub pharmacist: Address,
    pub fda_oracle: Address,
    pub medical_records_contract: Address,
    pub healthcare_payment_contract: Address,
}

#[derive(Clone)]
#[contracttype]
pub struct MedicationDefinition {
    pub code: String,
    pub ndc_code: String,
    pub name: String,
    pub generic_name: String,
    pub manufacturer: String,
    pub dosage_form: String,
    pub strength: String,
    pub controlled_substance: bool,
    pub source: MedicationSource,
    pub last_fda_sync: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct MedicationSchedule {
    pub id: u64,
    pub patient: Address,
    pub provider: Address,
    pub medication_code: String,
    pub dosage_amount: String,
    pub schedule: DosingSchedule,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub instructions: String,
    pub status: ScheduleStatus,
    pub linked_record_id: Option<u64>,
    pub linked_claim_id: Option<u64>,
    pub prescription_ref: Option<String>,
    pub refill: RefillPolicy,
    pub adherence_baseline_bps: u32,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct ScheduleLinks {
    pub linked_record_id: Option<u64>,
    pub linked_claim_id: Option<u64>,
    pub prescription_ref: Option<String>,
}

#[derive(Clone)]
#[contracttype]
pub struct ScheduleRequest {
    pub medication_code: String,
    pub dosage_amount: String,
    pub schedule: DosingSchedule,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub instructions: String,
    pub links: ScheduleLinks,
    pub refill: RefillPolicy,
    pub adherence_baseline_bps: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct RefillPolicy {
    pub enabled: bool,
    pub auto_refill: bool,
    pub total_authorized_refills: u32,
    pub refills_used: u32,
    pub reminder_window_days: u32,
    pub doses_remaining: u32,
    pub low_supply_threshold: u32,
    pub last_refill_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct RefillReminder {
    pub schedule_id: u64,
    pub patient: Address,
    pub status: RefillStatus,
    pub reminder_due_at: u64,
    pub last_notified_at: Option<u64>,
    pub next_refill_eta: Option<u64>,
    pub auto_refill_triggered_at: Option<u64>,
}

#[derive(Clone)]
#[contracttype]
pub struct DrugInteraction {
    pub medication_a: String,
    pub medication_b: String,
    pub severity: Severity,
    pub advisory: String,
    pub clinical_guidance: String,
    pub source_ref: String,
    pub updated_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct InteractionAlert {
    pub schedule_id: u64,
    pub interacting_schedule_id: u64,
    pub patient: Address,
    pub medication_a: String,
    pub medication_b: String,
    pub severity: Severity,
    pub advisory: String,
    pub created_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct DoseEvent {
    pub schedule_id: u64,
    pub patient: Address,
    pub scheduled_for: u64,
    pub recorded_at: u64,
    pub status: AdherenceEventStatus,
    pub notes: String,
}

#[derive(Clone)]
#[contracttype]
pub struct AdherenceReport {
    pub schedule_id: u64,
    pub patient: Address,
    pub expected_doses: u32,
    pub recorded_doses: u32,
    pub taken_doses: u32,
    pub missed_doses: u32,
    pub skipped_doses: u32,
    pub adherence_bps: u32,
    pub baseline_bps: u32,
    pub improvement_bps: i32,
    pub target_improvement_met: bool,
    pub generated_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Config,
    MedicationCount,
    ScheduleCount,
    Medication(String),
    Schedule(u64),
    PatientSchedules(Address),
    Interaction(String, String),
    ScheduleAlerts(u64),
    RefillReminder(u64),
    DoseEvents(u64),
}

#[contract]
pub struct MedicationManagement;

#[contractimpl]
impl MedicationManagement {
    pub fn initialize(
        env: Env,
        admin: Address,
        pharmacist: Address,
        fda_oracle: Address,
        medical_records_contract: Address,
        healthcare_payment_contract: Address,
    ) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Config) {
            return Err(Error::AlreadyInitialized);
        }

        let config = Config {
            admin,
            pharmacist,
            fda_oracle,
            medical_records_contract,
            healthcare_payment_contract,
        };

        env.storage().instance().set(&DataKey::Config, &config);
        env.storage()
            .instance()
            .set(&DataKey::MedicationCount, &0u64);
        env.storage().instance().set(&DataKey::ScheduleCount, &0u64);
        Ok(())
    }

    pub fn upsert_fda_medication(
        env: Env,
        operator: Address,
        medication: MedicationDefinition,
    ) -> Result<(), Error> {
        operator.require_auth();
        Self::authorize_catalog_operator(&env, &operator)?;
        Self::store_medication_definition(&env, medication, true)?;
        Ok(())
    }

    pub fn sync_fda_catalog(
        env: Env,
        operator: Address,
        medications: Vec<MedicationDefinition>,
    ) -> Result<u32, Error> {
        operator.require_auth();
        Self::authorize_catalog_operator(&env, &operator)?;
        if medications.is_empty() {
            return Err(Error::InvalidData);
        }

        let mut synced = 0u32;
        for medication in medications.iter() {
            Self::store_medication_definition(&env, medication, false)?;
            synced = synced.saturating_add(1);
        }

        env.events().publish((symbol_short!("CAT_SYNC"),), synced);
        Ok(synced)
    }

    pub fn create_schedule(
        env: Env,
        patient: Address,
        provider: Address,
        request: ScheduleRequest,
    ) -> Result<u64, Error> {
        provider.require_auth();
        Self::get_config(&env)?;
        Self::validate_schedule_inputs(
            &env,
            request.medication_code.clone(),
            request.dosage_amount.clone(),
            &request.schedule,
            request.start_time,
            request.end_time,
            &request.refill,
            request.adherence_baseline_bps,
        )?;

        let schedule_id = Self::schedule_count(&env).saturating_add(1);
        let now = env.ledger().timestamp();
        let schedule_record = MedicationSchedule {
            id: schedule_id,
            patient: patient.clone(),
            provider: provider.clone(),
            medication_code: request.medication_code.clone(),
            dosage_amount: request.dosage_amount,
            schedule: request.schedule,
            start_time: request.start_time,
            end_time: request.end_time,
            instructions: request.instructions,
            status: ScheduleStatus::Active,
            linked_record_id: request.links.linked_record_id,
            linked_claim_id: request.links.linked_claim_id,
            prescription_ref: request.links.prescription_ref,
            refill: request.refill.clone(),
            adherence_baseline_bps: request.adherence_baseline_bps,
            created_at: now,
            updated_at: now,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Schedule(schedule_id), &schedule_record);
        env.storage()
            .instance()
            .set(&DataKey::ScheduleCount, &schedule_id);

        let mut patient_schedules = Self::load_patient_schedules(&env, patient.clone());
        patient_schedules.push_back(schedule_id);
        env.storage().persistent().set(
            &DataKey::PatientSchedules(patient.clone()),
            &patient_schedules,
        );

        let reminder =
            Self::build_refill_reminder(&env, schedule_id, patient.clone(), &request.refill);
        env.storage()
            .persistent()
            .set(&DataKey::RefillReminder(schedule_id), &reminder);

        let alerts = Self::scan_interactions_for_schedule(
            &env,
            patient,
            schedule_id,
            request.medication_code,
            now,
        )?;
        env.storage()
            .persistent()
            .set(&DataKey::ScheduleAlerts(schedule_id), &alerts);

        env.events()
            .publish((symbol_short!("MED_SCHED"),), (schedule_id, provider));

        Ok(schedule_id)
    }

    pub fn update_schedule_status(
        env: Env,
        schedule_id: u64,
        actor: Address,
        status: ScheduleStatus,
    ) -> Result<(), Error> {
        actor.require_auth();
        let mut schedule = Self::get_schedule_internal(&env, schedule_id)?;
        if actor != schedule.provider
            && actor != schedule.patient
            && actor != Self::get_config(&env)?.admin
        {
            return Err(Error::Unauthorized);
        }

        schedule.status = status;
        schedule.updated_at = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::Schedule(schedule_id), &schedule);
        Ok(())
    }

    pub fn register_interaction(
        env: Env,
        operator: Address,
        interaction: DrugInteraction,
    ) -> Result<(), Error> {
        operator.require_auth();
        let config = Self::get_config(&env)?;
        if operator != config.admin
            && operator != config.fda_oracle
            && operator != config.pharmacist
        {
            return Err(Error::Unauthorized);
        }

        if interaction.medication_a == interaction.medication_b
            || interaction.medication_a.is_empty()
            || interaction.medication_b.is_empty()
            || interaction.advisory.is_empty()
        {
            return Err(Error::InvalidData);
        }

        let (left, right) =
            Self::normalized_pair(&interaction.medication_a, &interaction.medication_b);
        let key = DataKey::Interaction(left, right);
        if env.storage().persistent().has(&key) {
            return Err(Error::InteractionAlreadyExists);
        }

        env.storage().persistent().set(&key, &interaction);
        Ok(())
    }

    pub fn record_dose(
        env: Env,
        patient: Address,
        schedule_id: u64,
        scheduled_for: u64,
        status: AdherenceEventStatus,
        notes: String,
    ) -> Result<(), Error> {
        patient.require_auth();
        let schedule = Self::get_schedule_internal(&env, schedule_id)?;
        if patient != schedule.patient {
            return Err(Error::Unauthorized);
        }
        if scheduled_for < schedule.start_time {
            return Err(Error::InvalidData);
        }

        let mut events: Vec<DoseEvent> = env
            .storage()
            .persistent()
            .get(&DataKey::DoseEvents(schedule_id))
            .unwrap_or(Vec::new(&env));

        for existing in events.iter() {
            if existing.scheduled_for == scheduled_for {
                return Err(Error::DoseAlreadyRecorded);
            }
        }

        let event = DoseEvent {
            schedule_id,
            patient: patient.clone(),
            scheduled_for,
            recorded_at: env.ledger().timestamp(),
            status,
            notes,
        };
        events.push_back(event);
        env.storage()
            .persistent()
            .set(&DataKey::DoseEvents(schedule_id), &events);

        env.storage()
            .persistent()
            .get::<_, RefillReminder>(&DataKey::RefillReminder(schedule_id))
            .ok_or(Error::RefillNotFound)?;

        let mut schedule_record = schedule.clone();
        if schedule_record.refill.enabled && schedule_record.refill.doses_remaining > 0 {
            let patient_address = schedule_record.patient.clone();
            schedule_record.refill.doses_remaining -= 1;
            schedule_record.updated_at = env.ledger().timestamp();
            env.storage()
                .persistent()
                .set(&DataKey::Schedule(schedule_id), &schedule_record.clone());
            let reminder = Self::refresh_refill_reminder(
                &env,
                schedule_id,
                patient_address,
                &schedule_record.refill,
            );
            env.storage()
                .persistent()
                .set(&DataKey::RefillReminder(schedule_id), &reminder);
        }

        Ok(())
    }

    pub fn process_refill(
        env: Env,
        actor: Address,
        schedule_id: u64,
    ) -> Result<RefillReminder, Error> {
        actor.require_auth();
        let mut schedule = Self::get_schedule_internal(&env, schedule_id)?;
        let config = Self::get_config(&env)?;
        if actor != schedule.patient
            && actor != schedule.provider
            && actor != config.pharmacist
            && actor != config.admin
        {
            return Err(Error::Unauthorized);
        }
        if !schedule.refill.enabled {
            return Err(Error::InvalidData);
        }
        if schedule.refill.refills_used >= schedule.refill.total_authorized_refills {
            return Err(Error::InvalidData);
        }

        let now = env.ledger().timestamp();
        let mut reminder: RefillReminder = env
            .storage()
            .persistent()
            .get(&DataKey::RefillReminder(schedule_id))
            .ok_or(Error::RefillNotFound)?;

        schedule.refill.refills_used = schedule.refill.refills_used.saturating_add(1);
        schedule.refill.last_refill_at = now;
        let replenish_amount = schedule
            .refill
            .low_supply_threshold
            .saturating_mul(4)
            .max(schedule.refill.low_supply_threshold.saturating_add(1));
        schedule.refill.doses_remaining = schedule
            .refill
            .doses_remaining
            .saturating_add(replenish_amount);
        schedule.updated_at = now;

        reminder.status = RefillStatus::Fulfilled;
        reminder.last_notified_at = Some(now);
        reminder.next_refill_eta = Some(Self::estimate_refill_due_at(
            &schedule.refill,
            schedule.start_time,
            Self::doses_per_day(&schedule.schedule),
        ));

        env.storage()
            .persistent()
            .set(&DataKey::Schedule(schedule_id), &schedule);
        env.storage()
            .persistent()
            .set(&DataKey::RefillReminder(schedule_id), &reminder.clone());

        Ok(reminder)
    }

    pub fn trigger_auto_refill(
        env: Env,
        actor: Address,
        schedule_id: u64,
    ) -> Result<RefillReminder, Error> {
        let schedule = Self::get_schedule_internal(&env, schedule_id)?;
        let config = Self::get_config(&env)?;
        if actor != config.pharmacist && actor != config.admin && actor != schedule.patient {
            return Err(Error::Unauthorized);
        }
        if !schedule.refill.auto_refill {
            return Err(Error::AutoRefillDisabled);
        }

        let reminder = Self::process_refill(env.clone(), actor, schedule_id)?;
        let mut refreshed = reminder.clone();
        refreshed.auto_refill_triggered_at = Some(env.ledger().timestamp());
        refreshed.status = RefillStatus::Processing;
        env.storage()
            .persistent()
            .set(&DataKey::RefillReminder(schedule_id), &refreshed.clone());
        Ok(refreshed)
    }

    pub fn get_schedule(env: Env, schedule_id: u64) -> Result<MedicationSchedule, Error> {
        Self::get_schedule_internal(&env, schedule_id)
    }

    pub fn get_medication(
        env: Env,
        medication_code: String,
    ) -> Result<MedicationDefinition, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Medication(medication_code))
            .ok_or(Error::MedicationNotFound)
    }

    pub fn get_refill_status(env: Env, schedule_id: u64) -> Result<RefillReminder, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::RefillReminder(schedule_id))
            .ok_or(Error::RefillNotFound)
    }

    pub fn get_interaction_alerts(
        env: Env,
        schedule_id: u64,
    ) -> Result<Vec<InteractionAlert>, Error> {
        Ok(env
            .storage()
            .persistent()
            .get(&DataKey::ScheduleAlerts(schedule_id))
            .unwrap_or(Vec::new(&env)))
    }

    pub fn get_patient_schedules(env: Env, patient: Address) -> Vec<u64> {
        Self::load_patient_schedules(&env, patient)
    }

    pub fn generate_adherence_report(env: Env, schedule_id: u64) -> Result<AdherenceReport, Error> {
        let schedule = Self::get_schedule_internal(&env, schedule_id)?;
        let now = env.ledger().timestamp();
        let events: Vec<DoseEvent> = env
            .storage()
            .persistent()
            .get(&DataKey::DoseEvents(schedule_id))
            .unwrap_or(Vec::new(&env));

        let expected_doses = Self::expected_doses_until(&schedule, now);
        let mut taken_doses = 0u32;
        let mut missed_doses = 0u32;
        let mut skipped_doses = 0u32;

        for event in events.iter() {
            match event.status {
                AdherenceEventStatus::Taken => taken_doses = taken_doses.saturating_add(1),
                AdherenceEventStatus::Missed => missed_doses = missed_doses.saturating_add(1),
                AdherenceEventStatus::Skipped => skipped_doses = skipped_doses.saturating_add(1),
            }
        }

        let recorded_doses = taken_doses
            .saturating_add(missed_doses)
            .saturating_add(skipped_doses);
        let adherence_bps = if expected_doses == 0 {
            10_000
        } else {
            taken_doses.saturating_mul(10_000) / expected_doses
        };
        let improvement_bps = adherence_bps as i32 - schedule.adherence_baseline_bps as i32;

        Ok(AdherenceReport {
            schedule_id,
            patient: schedule.patient,
            expected_doses,
            recorded_doses,
            taken_doses,
            missed_doses,
            skipped_doses,
            adherence_bps,
            baseline_bps: schedule.adherence_baseline_bps,
            improvement_bps,
            target_improvement_met: improvement_bps >= 2_500,
            generated_at: now,
        })
    }

    pub fn get_catalog_size(env: Env) -> u64 {
        Self::medication_count(&env)
    }

    fn get_config(env: &Env) -> Result<Config, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)
    }

    fn get_schedule_internal(env: &Env, schedule_id: u64) -> Result<MedicationSchedule, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Schedule(schedule_id))
            .ok_or(Error::ScheduleNotFound)
    }

    fn medication_count(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::MedicationCount)
            .unwrap_or(0u64)
    }

    fn schedule_count(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::ScheduleCount)
            .unwrap_or(0u64)
    }

    fn load_patient_schedules(env: &Env, patient: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::PatientSchedules(patient))
            .unwrap_or(Vec::new(env))
    }

    fn validate_medication_definition(medication: &MedicationDefinition) -> Result<(), Error> {
        if medication.code.is_empty()
            || medication.ndc_code.is_empty()
            || medication.name.is_empty()
            || medication.generic_name.is_empty()
            || medication.strength.is_empty()
        {
            return Err(Error::InvalidData);
        }

        Ok(())
    }

    fn authorize_catalog_operator(env: &Env, operator: &Address) -> Result<(), Error> {
        let config = Self::get_config(env)?;
        if operator != &config.admin && operator != &config.fda_oracle {
            return Err(Error::Unauthorized);
        }
        Ok(())
    }

    fn store_medication_definition(
        env: &Env,
        medication: MedicationDefinition,
        emit_event: bool,
    ) -> Result<(), Error> {
        Self::validate_medication_definition(&medication)?;

        let key = DataKey::Medication(medication.code.clone());
        let existed = env.storage().persistent().has(&key);
        env.storage().persistent().set(&key, &medication.clone());

        if !existed {
            let count = Self::medication_count(env).saturating_add(1);
            env.storage()
                .instance()
                .set(&DataKey::MedicationCount, &count);
        }

        if emit_event {
            env.events().publish(
                (symbol_short!("MED_SYNC"), medication.code),
                medication.last_fda_sync,
            );
        }

        Ok(())
    }

    fn validate_schedule_inputs(
        env: &Env,
        medication_code: String,
        dosage_amount: String,
        schedule: &DosingSchedule,
        start_time: u64,
        end_time: Option<u64>,
        refill: &RefillPolicy,
        adherence_baseline_bps: u32,
    ) -> Result<(), Error> {
        if medication_code.is_empty() || dosage_amount.is_empty() || start_time == 0 {
            return Err(Error::InvalidData);
        }
        if adherence_baseline_bps > 10_000 {
            return Err(Error::InvalidData);
        }
        if let Some(end) = end_time {
            if end <= start_time {
                return Err(Error::InvalidData);
            }
        }
        if Self::doses_per_day(schedule) == 0 {
            return Err(Error::InvalidData);
        }
        if refill.enabled && refill.low_supply_threshold == 0 {
            return Err(Error::InvalidData);
        }
        if !env
            .storage()
            .persistent()
            .has(&DataKey::Medication(medication_code))
        {
            return Err(Error::MedicationNotFound);
        }

        Ok(())
    }

    fn build_refill_reminder(
        env: &Env,
        schedule_id: u64,
        patient: Address,
        refill: &RefillPolicy,
    ) -> RefillReminder {
        let due_status = if refill.enabled && refill.doses_remaining <= refill.low_supply_threshold
        {
            RefillStatus::ReminderDue
        } else {
            RefillStatus::Monitoring
        };
        RefillReminder {
            schedule_id,
            patient,
            status: due_status,
            reminder_due_at: env.ledger().timestamp(),
            last_notified_at: None,
            next_refill_eta: None,
            auto_refill_triggered_at: None,
        }
    }

    fn refresh_refill_reminder(
        env: &Env,
        schedule_id: u64,
        patient: Address,
        refill: &RefillPolicy,
    ) -> RefillReminder {
        let now = env.ledger().timestamp();
        let mut reminder = Self::build_refill_reminder(env, schedule_id, patient, refill);
        if refill.enabled {
            reminder.reminder_due_at =
                now.saturating_add(u64::from(refill.reminder_window_days).saturating_mul(86_400));
        }
        reminder
    }

    fn scan_interactions_for_schedule(
        env: &Env,
        patient: Address,
        schedule_id: u64,
        medication_code: String,
        now: u64,
    ) -> Result<Vec<InteractionAlert>, Error> {
        let patient_schedules = Self::load_patient_schedules(env, patient.clone());
        let mut alerts = Vec::new(env);

        for existing_id in patient_schedules.iter() {
            if existing_id == schedule_id {
                continue;
            }
            let existing = Self::get_schedule_internal(env, existing_id)?;
            if existing.status != ScheduleStatus::Active {
                continue;
            }
            let (left, right) = Self::normalized_pair(&medication_code, &existing.medication_code);
            let interaction: Option<DrugInteraction> = env
                .storage()
                .persistent()
                .get(&DataKey::Interaction(left, right));
            if let Some(matchup) = interaction {
                alerts.push_back(InteractionAlert {
                    schedule_id,
                    interacting_schedule_id: existing_id,
                    patient: patient.clone(),
                    medication_a: matchup.medication_a,
                    medication_b: matchup.medication_b,
                    severity: matchup.severity,
                    advisory: matchup.advisory,
                    created_at: now,
                });
            }
        }

        Ok(alerts)
    }

    fn normalized_pair(left: &String, right: &String) -> (String, String) {
        if left < right {
            (left.clone(), right.clone())
        } else {
            (right.clone(), left.clone())
        }
    }

    fn doses_per_day(schedule: &DosingSchedule) -> u32 {
        match schedule {
            DosingSchedule::OnceDaily => 1,
            DosingSchedule::TwiceDaily => 2,
            DosingSchedule::ThreeTimesDaily => 3,
            DosingSchedule::EveryNHours(hours) => {
                if *hours == 0 || *hours > 24 {
                    0
                } else {
                    24 / *hours
                }
            }
            DosingSchedule::EveryNDays(days) => {
                if *days == 0 {
                    0
                } else {
                    1
                }
            }
            DosingSchedule::Weekly => 1,
            DosingSchedule::SpecificTimes(times) => times.len(),
        }
    }

    fn expected_doses_until(schedule: &MedicationSchedule, until: u64) -> u32 {
        if until <= schedule.start_time {
            return 0;
        }

        let capped_until = match schedule.end_time {
            Some(end) if end < until => end,
            _ => until,
        };

        if capped_until <= schedule.start_time {
            return 0;
        }

        let elapsed_days = ((capped_until - schedule.start_time) / 86_400).saturating_add(1);
        match &schedule.schedule {
            DosingSchedule::EveryNDays(days) => {
                if *days == 0 {
                    0
                } else {
                    let days_u64 = u64::from(*days);
                    elapsed_days.div_ceil(days_u64) as u32
                }
            }
            DosingSchedule::Weekly => elapsed_days.div_ceil(7) as u32,
            _ => (elapsed_days as u32).saturating_mul(Self::doses_per_day(&schedule.schedule)),
        }
    }

    fn estimate_refill_due_at(refill: &RefillPolicy, start_time: u64, doses_per_day: u32) -> u64 {
        if doses_per_day == 0 {
            return start_time;
        }

        let days_left = u64::from(refill.doses_remaining).div_ceil(u64::from(doses_per_day));
        start_time.saturating_add(days_left.saturating_mul(86_400))
    }
}

#[cfg(test)]
mod test {
    extern crate std;

    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, EnvTestConfig, Ledger},
        Address,
    };

    fn setup() -> (
        Env,
        MedicationManagementClient<'static>,
        Address,
        Address,
        Address,
        Address,
        Address,
    ) {
        let mut env = Env::default();
        env.set_config(EnvTestConfig {
            capture_snapshot_at_drop: false,
        });
        env.mock_all_auths();
        env.budget().reset_unlimited();
        env.ledger().with_mut(|ledger| {
            ledger.timestamp = 1_700_000_000;
        });

        let contract_id = env.register_contract(None, MedicationManagement);
        let client = MedicationManagementClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let pharmacist = Address::generate(&env);
        let fda_oracle = Address::generate(&env);
        let medical_records_contract = Address::generate(&env);
        let healthcare_payment_contract = Address::generate(&env);

        client.initialize(
            &admin,
            &pharmacist,
            &fda_oracle,
            &medical_records_contract,
            &healthcare_payment_contract,
        );

        (
            env,
            client,
            admin,
            pharmacist,
            fda_oracle,
            medical_records_contract,
            healthcare_payment_contract,
        )
    }

    fn med(env: &Env, code: &str, ndc: &str, name: &str, synced_at: u64) -> MedicationDefinition {
        MedicationDefinition {
            code: String::from_str(env, code),
            ndc_code: String::from_str(env, ndc),
            name: String::from_str(env, name),
            generic_name: String::from_str(env, name),
            manufacturer: String::from_str(env, "Acme Pharma"),
            dosage_form: String::from_str(env, "tablet"),
            strength: String::from_str(env, "10mg"),
            controlled_substance: false,
            source: MedicationSource::Fda,
            last_fda_sync: synced_at,
        }
    }

    fn refill_policy() -> RefillPolicy {
        RefillPolicy {
            enabled: true,
            auto_refill: true,
            total_authorized_refills: 3,
            refills_used: 0,
            reminder_window_days: 3,
            doses_remaining: 5,
            low_supply_threshold: 2,
            last_refill_at: 0,
        }
    }

    #[test]
    fn manages_schedule_and_links_to_external_healthcare_refs() {
        let (env, client, admin, pharmacist, _fda_oracle, _records_contract, _payments_contract) =
            setup();
        let provider = Address::generate(&env);
        let patient = Address::generate(&env);

        client.upsert_fda_medication(
            &admin,
            &med(&env, "RX-001", "0001-0001", "Lisinopril", 1_700_000_000),
        );

        let schedule_id = client.create_schedule(
            &patient,
            &provider,
            &ScheduleRequest {
                medication_code: String::from_str(&env, "RX-001"),
                dosage_amount: String::from_str(&env, "10mg"),
                schedule: DosingSchedule::TwiceDaily,
                start_time: 1_700_000_000,
                end_time: Some(1_700_864_000),
                instructions: String::from_str(&env, "Take after meals"),
                links: ScheduleLinks {
                    linked_record_id: Some(42),
                    linked_claim_id: Some(7),
                    prescription_ref: Some(String::from_str(&env, "telemed-rx-99")),
                },
                refill: refill_policy(),
                adherence_baseline_bps: 6_000,
            },
        );

        let schedule = client.get_schedule(&schedule_id);
        assert_eq!(schedule.patient, patient);
        assert_eq!(schedule.linked_record_id, Some(42));
        assert_eq!(schedule.linked_claim_id, Some(7));
        assert_eq!(
            schedule.prescription_ref,
            Some(String::from_str(&env, "telemed-rx-99"))
        );

        let refill = client.get_refill_status(&schedule_id);
        assert_eq!(refill.status, RefillStatus::Monitoring);

        let patient_schedules = client.get_patient_schedules(&patient);
        assert_eq!(patient_schedules.len(), 1);

        let reminder = client.trigger_auto_refill(&pharmacist, &schedule_id);
        assert_eq!(reminder.status, RefillStatus::Processing);
    }

    #[test]
    fn creates_interaction_alerts_for_active_medications() {
        let (env, client, admin, pharmacist, _fda_oracle, _records_contract, _payments_contract) =
            setup();
        let provider = Address::generate(&env);
        let patient = Address::generate(&env);

        client.upsert_fda_medication(&admin, &med(&env, "RX-A", "0001", "Warfarin", 1));
        client.upsert_fda_medication(&admin, &med(&env, "RX-B", "0002", "Ibuprofen", 1));

        client.register_interaction(
            &pharmacist,
            &DrugInteraction {
                medication_a: String::from_str(&env, "RX-A"),
                medication_b: String::from_str(&env, "RX-B"),
                severity: Severity::High,
                advisory: String::from_str(&env, "Bleeding risk increases"),
                clinical_guidance: String::from_str(&env, "Use alternative analgesic"),
                source_ref: String::from_str(&env, "FDA-Label-123"),
                updated_at: 1,
            },
        );

        client.create_schedule(
            &patient,
            &provider,
            &ScheduleRequest {
                medication_code: String::from_str(&env, "RX-A"),
                dosage_amount: String::from_str(&env, "5mg"),
                schedule: DosingSchedule::OnceDaily,
                start_time: 1_700_000_000,
                end_time: None,
                instructions: String::from_str(&env, "Nightly"),
                links: ScheduleLinks {
                    linked_record_id: None,
                    linked_claim_id: None,
                    prescription_ref: None,
                },
                refill: refill_policy(),
                adherence_baseline_bps: 5_500,
            },
        );

        let second_schedule = client.create_schedule(
            &patient,
            &provider,
            &ScheduleRequest {
                medication_code: String::from_str(&env, "RX-B"),
                dosage_amount: String::from_str(&env, "400mg"),
                schedule: DosingSchedule::EveryNHours(12),
                start_time: 1_700_000_000,
                end_time: None,
                instructions: String::from_str(&env, "As needed"),
                links: ScheduleLinks {
                    linked_record_id: None,
                    linked_claim_id: None,
                    prescription_ref: None,
                },
                refill: refill_policy(),
                adherence_baseline_bps: 5_500,
            },
        );

        let alerts = client.get_interaction_alerts(&second_schedule);
        assert_eq!(alerts.len(), 1);
        let alert = alerts.get(0).unwrap();
        assert_eq!(alert.severity, Severity::High);
        assert_eq!(alert.interacting_schedule_id, 1);
    }

    #[test]
    fn tracks_adherence_and_reports_improvement_target() {
        let (env, client, admin, _pharmacist, _fda_oracle, _records_contract, _payments_contract) =
            setup();
        let provider = Address::generate(&env);
        let patient = Address::generate(&env);

        client.upsert_fda_medication(&admin, &med(&env, "RX-ADH", "0003", "Metformin", 1));

        let schedule_id = client.create_schedule(
            &patient,
            &provider,
            &ScheduleRequest {
                medication_code: String::from_str(&env, "RX-ADH"),
                dosage_amount: String::from_str(&env, "500mg"),
                schedule: DosingSchedule::TwiceDaily,
                start_time: 1_700_000_000,
                end_time: None,
                instructions: String::from_str(&env, "Morning and evening"),
                links: ScheduleLinks {
                    linked_record_id: None,
                    linked_claim_id: None,
                    prescription_ref: None,
                },
                refill: refill_policy(),
                adherence_baseline_bps: 5_000,
            },
        );

        client.record_dose(
            &patient,
            &schedule_id,
            &1_700_000_000,
            &AdherenceEventStatus::Taken,
            &String::from_str(&env, "completed"),
        );
        client.record_dose(
            &patient,
            &schedule_id,
            &1_700_043_200,
            &AdherenceEventStatus::Taken,
            &String::from_str(&env, "completed"),
        );
        client.record_dose(
            &patient,
            &schedule_id,
            &1_700_086_400,
            &AdherenceEventStatus::Taken,
            &String::from_str(&env, "completed"),
        );

        env.ledger().with_mut(|ledger| {
            ledger.timestamp = 1_700_086_400;
        });

        let report = client.generate_adherence_report(&schedule_id);
        assert_eq!(report.expected_doses, 4);
        assert_eq!(report.taken_doses, 3);
        assert_eq!(report.adherence_bps, 7_500);
        assert_eq!(report.improvement_bps, 2_500);
        assert!(report.target_improvement_met);

        let refill = client.get_refill_status(&schedule_id);
        assert_eq!(refill.status, RefillStatus::ReminderDue);
    }

    #[test]
    fn supports_large_medication_catalogs() {
        const STRESS_IMPORT_SIZE: u32 = 1_024;

        let mut env = Env::default();
        env.set_config(EnvTestConfig {
            capture_snapshot_at_drop: false,
        });
        env.mock_all_auths();
        env.budget().reset_unlimited();
        let contract_id = env.register_contract(None, MedicationManagement);
        let admin = Address::generate(&env);
        let pharmacist = Address::generate(&env);
        let fda_oracle = Address::generate(&env);
        let medical_records_contract = Address::generate(&env);
        let healthcare_payment_contract = Address::generate(&env);
        let mut medications = Vec::new(&env);

        for index in 0..STRESS_IMPORT_SIZE {
            let code = format_code(&env, "FDA", index);
            let ndc = format_code(&env, "NDC", index);
            let name = format_code(&env, "Drug", index);
            medications.push_back(MedicationDefinition {
                code,
                ndc_code: ndc,
                name: name.clone(),
                generic_name: name,
                manufacturer: String::from_str(&env, "Catalog Labs"),
                dosage_form: String::from_str(&env, "capsule"),
                strength: String::from_str(&env, "25mg"),
                controlled_substance: false,
                source: MedicationSource::Fda,
                last_fda_sync: u64::from(index),
            });
        }

        env.as_contract(&contract_id, || {
            MedicationManagement::initialize(
                env.clone(),
                admin.clone(),
                pharmacist,
                fda_oracle,
                medical_records_contract,
                healthcare_payment_contract,
            )
            .unwrap();

            assert_eq!(
                MedicationManagement::sync_fda_catalog(
                    env.clone(),
                    admin.clone(),
                    medications.clone(),
                )
                .unwrap(),
                STRESS_IMPORT_SIZE
            );

            assert_eq!(
                MedicationManagement::get_catalog_size(env.clone()),
                u64::from(STRESS_IMPORT_SIZE)
            );
            let record = MedicationManagement::get_medication(
                env.clone(),
                String::from_str(&env, "FDA-1023"),
            )
            .unwrap();
            assert_eq!(record.ndc_code, String::from_str(&env, "NDC-1023"));
        });
    }

    fn format_code(env: &Env, prefix: &str, index: u32) -> String {
        String::from_str(env, &std::format!("{prefix}-{index}"))
    }
}
