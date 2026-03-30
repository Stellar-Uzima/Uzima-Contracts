#![no_std]
#![allow(dead_code)]

#[cfg(test)]
mod test;

mod errors;
mod events;

pub use errors::Error;

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, Address, Env, String, Vec,
};

// ==================== Data Types ====================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
#[repr(u32)]
pub enum AppointmentStatus {
    Booked = 0,
    Confirmed = 1,
    Refunded = 2,
    Completed = 3,
}

#[derive(Clone)]
#[contracttype]
pub struct AppointmentEscrow {
    pub appointment_id: u64,
    pub patient: Address,
    pub provider: Address,
    pub amount: i128,
    pub token: Address,
    pub booked_at: u64,
    pub confirmed_at: u64, // 0 if not confirmed
    pub refunded_at: u64,   // 0 if not refunded
    pub status: AppointmentStatus,
    pub funds_released: bool, // Prevents double withdrawal
}

#[contracttype]
pub enum DataKey {
    Initialized,
    Admin,
    AppointmentCounter,
    Appointment(u64), // appointment_id -> AppointmentEscrow
    PatientAppointments(Address), // patient -> Vec<u64>
    ProviderAppointments(Address), // provider -> Vec<u64>
}

// ==================== Contract ====================

#[contract]
pub struct AppointmentBookingEscrow;

#[contractimpl]
impl AppointmentBookingEscrow {
    /// Initialize the contract with an admin and token address
    pub fn initialize(env: Env, admin: Address, token: Address) -> Result<(), Error> {
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::AppointmentCounter, &0u64);

        events::publish_initialization(&env, &admin);
        Ok(())
    }

    /// Book an appointment with payment locked in escrow
    /// Transfers `amount` from patient to contract and creates an appointment escrow
    pub fn book_appointment(
        env: Env,
        patient: Address,
        provider: Address,
        amount: i128,
        token: Address,
    ) -> Result<u64, Error> {
        patient.require_auth();
        Self::require_initialized(&env)?;

        // Validate inputs
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        if patient == provider {
            return Err(Error::InvalidProvider);
        }

        // Get next appointment ID
        let counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::AppointmentCounter)
            .unwrap_or(0);
        let appointment_id = counter.checked_add(1).ok_or(Error::InvalidState)?;

        // Update counter
        env.storage()
            .instance()
            .set(&DataKey::AppointmentCounter, &appointment_id);

        let timestamp = env.ledger().timestamp();

        // Transfer funds from patient to contract
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&patient, &env.current_contract_address(), &amount);

        // Create appointment escrow record
        let appointment = AppointmentEscrow {
            appointment_id,
            patient: patient.clone(),
            provider: provider.clone(),
            amount,
            token: token.clone(),
            booked_at: timestamp,
            confirmed_at: 0,
            refunded_at: 0,
            status: AppointmentStatus::Booked,
            funds_released: false,
        };

        // Store appointment
        env.storage()
            .persistent()
            .set(&DataKey::Appointment(appointment_id), &appointment);

        // Add to patient's appointments list
        let mut patient_appts: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::PatientAppointments(patient.clone()))
            .unwrap_or_else(|| Vec::new(&env));
        patient_appts.push_back(appointment_id);
        env.storage()
            .persistent()
            .set(&DataKey::PatientAppointments(patient), &patient_appts);

        // Add to provider's appointments list
        let mut provider_appts: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::ProviderAppointments(provider.clone()))
            .unwrap_or_else(|| Vec::new(&env));
        provider_appts.push_back(appointment_id);
        env.storage()
            .persistent()
            .set(&DataKey::ProviderAppointments(provider), &provider_appts);

        events::publish_appointment_booked(&env, appointment_id, &patient, &provider, amount, timestamp);

        Ok(appointment_id)
    }

    /// Confirm appointment completion and release funds to provider
    /// Only the provider can confirm the appointment
    pub fn confirm_appointment(env: Env, provider: Address, appointment_id: u64) -> Result<(), Error> {
        provider.require_auth();
        Self::require_initialized(&env)?;

        // Get appointment
        let appointment_key = DataKey::Appointment(appointment_id);
        let mut appointment: AppointmentEscrow = env
            .storage()
            .persistent()
            .get(&appointment_key)
            .ok_or(Error::AppointmentNotFound)?;

        // Verify provider matches
        if appointment.provider != provider {
            return Err(Error::OnlyProviderCanConfirm);
        }

        // Check if already confirmed or refunded
        if appointment.status == AppointmentStatus::Confirmed {
            return Err(Error::AppointmentAlreadyConfirmed);
        }
        if appointment.status == AppointmentStatus::Refunded {
            return Err(Error::AppointmentAlreadyRefunded);
        }

        // Prevent double withdrawal
        if appointment.funds_released {
            return Err(Error::DoubleWithdrawal);
        }

        let timestamp = env.ledger().timestamp();

        // Transfer funds from contract to provider
        let token_client = token::Client::new(&env, &appointment.token);
        token_client.transfer(&env.current_contract_address(), &provider, &appointment.amount);

        // Update appointment status
        appointment.confirmed_at = timestamp;
        appointment.status = AppointmentStatus::Completed;
        appointment.funds_released = true;

        // Store updated appointment
        env.storage()
            .persistent()
            .set(&appointment_key, &appointment);

        events::publish_appointment_confirmed(&env, appointment_id, &provider, timestamp);
        events::publish_funds_released(&env, appointment_id, &provider, appointment.amount, timestamp);

        Ok(())
    }

    /// Refund appointment if canceled
    /// Only the patient can request a refund
    /// Can only be done if appointment is still in Booked state (not Confirmed/Refunded)
    pub fn refund_appointment(env: Env, patient: Address, appointment_id: u64) -> Result<(), Error> {
        patient.require_auth();
        Self::require_initialized(&env)?;

        // Get appointment
        let appointment_key = DataKey::Appointment(appointment_id);
        let mut appointment: AppointmentEscrow = env
            .storage()
            .persistent()
            .get(&appointment_key)
            .ok_or(Error::AppointmentNotFound)?;

        // Verify patient matches
        if appointment.patient != patient {
            return Err(Error::OnlyPatientCanRefund);
        }

        // Check if already refunded
        if appointment.status == AppointmentStatus::Refunded {
            return Err(Error::AppointmentAlreadyRefunded);
        }

        // Check if already confirmed (can't refund confirmed appointment)
        if appointment.status == AppointmentStatus::Confirmed || appointment.status == AppointmentStatus::Completed {
            return Err(Error::InvalidState);
        }

        // Prevent double withdrawal
        if appointment.funds_released {
            return Err(Error::DoubleWithdrawal);
        }

        let timestamp = env.ledger().timestamp();

        // Transfer funds from contract back to patient
        let token_client = token::Client::new(&env, &appointment.token);
        token_client.transfer(&env.current_contract_address(), &patient, &appointment.amount);

        // Update appointment status
        appointment.refunded_at = timestamp;
        appointment.status = AppointmentStatus::Refunded;
        appointment.funds_released = true;

        // Store updated appointment
        env.storage()
            .persistent()
            .set(&appointment_key, &appointment);

        events::publish_appointment_refunded(&env, appointment_id, &patient, appointment.amount, timestamp);

        Ok(())
    }

    /// Get appointment details
    pub fn get_appointment(env: Env, appointment_id: u64) -> Option<AppointmentEscrow> {
        env.storage()
            .persistent()
            .get(&DataKey::Appointment(appointment_id))
    }

    /// Get all appointments for a patient
    pub fn get_patient_appointments(env: Env, patient: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::PatientAppointments(patient))
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Get all appointments for a provider
    pub fn get_provider_appointments(env: Env, provider: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::ProviderAppointments(provider))
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Get appointment status
    pub fn get_appointment_status(env: Env, appointment_id: u64) -> Result<AppointmentStatus, Error> {
        env.storage()
            .persistent()
            .get::<_, AppointmentEscrow>(&DataKey::Appointment(appointment_id))
            .map(|appt| appt.status)
            .ok_or(Error::AppointmentNotFound)
    }

    /// Get escrow balance (should be equal to sum of all booked but not confirmed/refunded appointments)
    pub fn get_escrow_balance(env: Env) -> i128 {
        let counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::AppointmentCounter)
            .unwrap_or(0);

        let mut balance: i128 = 0;
        for i in 1..=counter {
            if let Some(appointment) = env
                .storage()
                .persistent()
                .get::<_, AppointmentEscrow>(&DataKey::Appointment(i))
            {
                if appointment.status == AppointmentStatus::Booked && !appointment.funds_released {
                    balance = balance.checked_add(appointment.amount).unwrap_or(balance);
                }
            }
        }
        balance
    }

    /// Get the current admin
    pub fn get_admin(env: Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)
    }

    // ==================== Internal Helpers ====================

    fn require_initialized(env: &Env) -> Result<(), Error> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::NotInitialized);
        }
        Ok(())
    }
}
