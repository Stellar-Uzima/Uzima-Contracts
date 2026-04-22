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
    pub refunded_at: u64,  // 0 if not refunded
    pub status: AppointmentStatus,
    pub funds_released: bool, // Prevents double withdrawal
}

#[contracttype]
pub enum DataKey {
    Initialized,
    Admin,
    AppointmentCounter,
    Appointment(u64),              // appointment_id -> AppointmentEscrow
    PatientAppointments(Address),  // patient -> Vec<u64>
    ProviderAppointments(Address), // provider -> Vec<u64>
    Paused,
    LastActivity,
    TotalOperations,
    FailedOperations,
    Version,
}

/// Contract health status
#[derive(Clone, Debug)]
#[contracttype]
pub struct ContractHealth {
    pub version: String,
    pub is_paused: bool,
    pub storage_usage: u64,
    pub last_activity: u64,
    pub total_operations: u64,
    pub failed_operations: u64,
    pub success_rate: u32,
    pub total_appointments: u64,
    pub active_escrow_balance: i128,
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
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage().instance().set(&DataKey::LastActivity, &env.ledger().timestamp());
        env.storage().instance().set(&DataKey::TotalOperations, &0u64);
        env.storage().instance().set(&DataKey::FailedOperations, &0u64);
        env.storage().instance().set(&DataKey::Version, &String::from_str(&env, "1.0.0"));

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
            Self::record_operation(&env, false);
            return Err(Error::InvalidAmount);
        }

        if patient == provider {
            Self::record_operation(&env, false);
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

        events::publish_appointment_booked(
            &env,
            appointment_id,
            &patient,
            &provider,
            amount,
            timestamp,
        );

        Self::record_operation(&env, true);
        Ok(appointment_id)
    }

    /// Confirm appointment completion and release funds to provider
    /// Only the provider can confirm the appointment
    pub fn confirm_appointment(
        env: Env,
        provider: Address,
        appointment_id: u64,
    ) -> Result<(), Error> {
        provider.require_auth();
        Self::require_initialized(&env)?;

        // Get appointment
        let appointment_key = DataKey::Appointment(appointment_id);
        let mut appointment: AppointmentEscrow = env
            .storage()
            .persistent()
            .get(&appointment_key)
            .ok_or_else(|| {
                Self::record_operation(&env, false);
                Error::AppointmentNotFound
            })?;

        // Verify provider matches
        if appointment.provider != provider {
            Self::record_operation(&env, false);
            return Err(Error::OnlyProviderCanConfirm);
        }

        // Check if already confirmed or refunded
        if appointment.status == AppointmentStatus::Confirmed {
            Self::record_operation(&env, false);
            return Err(Error::AppointmentAlreadyConfirmed);
        }
        if appointment.status == AppointmentStatus::Refunded {
            Self::record_operation(&env, false);
            return Err(Error::AppointmentAlreadyRefunded);
        }

        // Prevent double withdrawal
        if appointment.funds_released {
            Self::record_operation(&env, false);
            return Err(Error::DoubleWithdrawal);
        }

        let timestamp = env.ledger().timestamp();

        // Transfer funds from contract to provider
        let token_client = token::Client::new(&env, &appointment.token);
        token_client.transfer(
            &env.current_contract_address(),
            &provider,
            &appointment.amount,
        );

        // Update appointment status
        appointment.confirmed_at = timestamp;
        appointment.status = AppointmentStatus::Completed;
        appointment.funds_released = true;

        // Store updated appointment
        env.storage()
            .persistent()
            .set(&appointment_key, &appointment);

        events::publish_appointment_confirmed(&env, appointment_id, &provider, timestamp);
        events::publish_funds_released(
            &env,
            appointment_id,
            &provider,
            appointment.amount,
            timestamp,
        );

        Self::record_operation(&env, true);
        Ok(())
    }

    /// Refund appointment if canceled
    /// Only the patient can request a refund
    /// Can only be done if appointment is still in Booked state (not Confirmed/Refunded)
    pub fn refund_appointment(
        env: Env,
        patient: Address,
        appointment_id: u64,
    ) -> Result<(), Error> {
        patient.require_auth();
        Self::require_initialized(&env)?;

        // Get appointment
        let appointment_key = DataKey::Appointment(appointment_id);
        let mut appointment: AppointmentEscrow = env
            .storage()
            .persistent()
            .get(&appointment_key)
            .ok_or_else(|| {
                Self::record_operation(&env, false);
                Error::AppointmentNotFound
            })?;

        // Verify patient matches
        if appointment.patient != patient {
            Self::record_operation(&env, false);
            return Err(Error::OnlyPatientCanRefund);
        }

        // Check if already refunded
        if appointment.status == AppointmentStatus::Refunded {
            Self::record_operation(&env, false);
            return Err(Error::AppointmentAlreadyRefunded);
        }

        // Check if already confirmed (can't refund confirmed appointment)
        if appointment.status == AppointmentStatus::Confirmed
            || appointment.status == AppointmentStatus::Completed
        {
            Self::record_operation(&env, false);
            return Err(Error::InvalidState);
        }

        // Prevent double withdrawal
        if appointment.funds_released {
            Self::record_operation(&env, false);
            return Err(Error::DoubleWithdrawal);
        }

        let timestamp = env.ledger().timestamp();

        // Transfer funds from contract back to patient
        let token_client = token::Client::new(&env, &appointment.token);
        token_client.transfer(
            &env.current_contract_address(),
            &patient,
            &appointment.amount,
        );

        // Update appointment status
        appointment.refunded_at = timestamp;
        appointment.status = AppointmentStatus::Refunded;
        appointment.funds_released = true;

        // Store updated appointment
        env.storage()
            .persistent()
            .set(&appointment_key, &appointment);

        events::publish_appointment_refunded(
            &env,
            appointment_id,
            &patient,
            appointment.amount,
            timestamp,
        );

        Self::record_operation(&env, true);
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
    pub fn get_appointment_status(
        env: Env,
        appointment_id: u64,
    ) -> Result<AppointmentStatus, Error> {
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

    /// Get comprehensive health check
    pub fn health_check(env: Env) -> ContractHealth {
        let version = env.storage()
            .instance()
            .get(&DataKey::Version)
            .unwrap_or_else(|| String::from_str(&env, "1.0.0"));
        
        let is_paused = env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false);
        
        let last_activity = env.storage()
            .instance()
            .get(&DataKey::LastActivity)
            .unwrap_or(0);
        
        let total_operations = env.storage()
            .instance()
            .get(&DataKey::TotalOperations)
            .unwrap_or(0);
        
        let failed_operations = env.storage()
            .instance()
            .get(&DataKey::FailedOperations)
            .unwrap_or(0);
        
        let success_rate = if total_operations > 0 {
            let successful = total_operations.saturating_sub(failed_operations);
            ((successful * 10000) / total_operations) as u32
        } else {
            10000u32
        };

        let total_appointments = env.storage()
            .instance()
            .get(&DataKey::AppointmentCounter)
            .unwrap_or(0);

        let active_escrow_balance = Self::get_escrow_balance(env.clone());

        let storage_usage = 1024u64 + (total_appointments * 256);

        ContractHealth {
            version,
            is_paused,
            storage_usage,
            last_activity,
            total_operations,
            failed_operations,
            success_rate,
            total_appointments,
            active_escrow_balance,
        }
    }

    /// Set pause status (admin only)
    pub fn set_paused(env: Env, admin: Address, paused: bool) -> Result<(), Error> {
        admin.require_auth();
        Self::require_initialized(&env)?;
        
        let stored_admin: Address = env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        
        if admin != stored_admin {
            return Err(Error::NotInitialized); // Reusing error for unauthorized
        }

        env.storage().instance().set(&DataKey::Paused, &paused);
        Ok(())
    }

    /// Check if contract is paused
    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    // ==================== Internal Helpers ====================

    fn require_initialized(env: &Env) -> Result<(), Error> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::NotInitialized);
        }
        Ok(())
    }

    fn record_operation(env: &Env, success: bool) {
        env.storage().instance().set(&DataKey::LastActivity, &env.ledger().timestamp());
        
        let total: u64 = env.storage()
            .instance()
            .get(&DataKey::TotalOperations)
            .unwrap_or(0);
        env.storage().instance().set(&DataKey::TotalOperations, &(total + 1));
        
        if !success {
            let failed: u64 = env.storage()
                .instance()
                .get(&DataKey::FailedOperations)
                .unwrap_or(0);
            env.storage().instance().set(&DataKey::FailedOperations, &(failed + 1));
        }
    }
}
