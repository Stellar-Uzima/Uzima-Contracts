#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppointmentBookingEscrow, AppointmentBookingEscrowClient, AppointmentStatus, Error,
    };
    use soroban_sdk::{Address, Env};

    fn setup() -> (Env, AppointmentBookingEscrowClient, Address, Address) {
        let env = Env::default();
        let admin = Address::random(&env);
        let token = Address::random(&env);
        let client = AppointmentBookingEscrowClient::new(
            &env,
            &env.register_contract(None, AppointmentBookingEscrow),
        );
        (env, client, admin, token)
    }

    #[test]
    fn test_initialize() {
        let (env, client, admin, token) = setup();
        let result = client.initialize(&admin, &token);
        assert!(result.is_ok());
    }

    #[test]
    fn test_initialize_twice_fails() {
        let (env, client, admin, token) = setup();
        client.initialize(&admin, &token).unwrap();
        let result = client.initialize(&admin, &token);
        assert_eq!(result, Err(Error::AlreadyInitialized));
    }

    #[test]
    fn test_book_appointment() {
        let (env, client, admin, token) = setup();
        client.initialize(&admin, &token).unwrap();

        let patient = Address::random(&env);
        let provider = Address::random(&env);
        let amount: i128 = 1000;

        let result = client.book_appointment(&patient, &provider, &amount, &token);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // First appointment ID is 1
    }

    #[test]
    fn test_book_appointment_invalid_amount() {
        let (env, client, admin, token) = setup();
        client.initialize(&admin, &token).unwrap();

        let patient = Address::random(&env);
        let provider = Address::random(&env);

        // Try with zero amount
        let result = client.book_appointment(&patient, &provider, &0, &token);
        assert_eq!(result, Err(Error::InvalidAmount));

        // Try with negative amount
        let result = client.book_appointment(&patient, &provider, &-100, &token);
        assert_eq!(result, Err(Error::InvalidAmount));
    }

    #[test]
    fn test_book_appointment_self_provider() {
        let (env, client, admin, token) = setup();
        client.initialize(&admin, &token).unwrap();

        let patient = Address::random(&env);
        let amount: i128 = 1000;

        // Try to book with patient as provider
        let result = client.book_appointment(&patient, &patient, &amount, &token);
        assert_eq!(result, Err(Error::InvalidProvider));
    }

    #[test]
    fn test_multiple_appointments_increment_id() {
        let (env, client, admin, token) = setup();
        client.initialize(&admin, &token).unwrap();

        let patient = Address::random(&env);
        let provider1 = Address::random(&env);
        let provider2 = Address::random(&env);
        let amount: i128 = 1000;

        let appt1 = client
            .book_appointment(&patient, &provider1, &amount, &token)
            .unwrap();
        let appt2 = client
            .book_appointment(&patient, &provider2, &amount, &token)
            .unwrap();

        assert_eq!(appt1, 1);
        assert_eq!(appt2, 2);
    }

    #[test]
    fn test_confirm_appointment() {
        let (env, client, admin, token) = setup();
        client.initialize(&admin, &token).unwrap();

        let patient = Address::random(&env);
        let provider = Address::random(&env);
        let amount: i128 = 1000;

        let appointment_id = client
            .book_appointment(&patient, &provider, &amount, &token)
            .unwrap();

        let result = client.confirm_appointment(&provider, &appointment_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_confirm_appointment_wrong_provider() {
        let (env, client, admin, token) = setup();
        client.initialize(&admin, &token).unwrap();

        let patient = Address::random(&env);
        let provider = Address::random(&env);
        let wrong_provider = Address::random(&env);
        let amount: i128 = 1000;

        let appointment_id = client
            .book_appointment(&patient, &provider, &amount, &token)
            .unwrap();

        // Try to confirm as wrong provider
        let result = client.confirm_appointment(&wrong_provider, &appointment_id);
        assert_eq!(result, Err(Error::OnlyProviderCanConfirm));
    }

    #[test]
    fn test_confirm_appointment_twice_fails() {
        let (env, client, admin, token) = setup();
        client.initialize(&admin, &token).unwrap();

        let patient = Address::random(&env);
        let provider = Address::random(&env);
        let amount: i128 = 1000;

        let appointment_id = client
            .book_appointment(&patient, &provider, &amount, &token)
            .unwrap();

        client
            .confirm_appointment(&provider, &appointment_id)
            .unwrap();

        // Try to confirm again
        let result = client.confirm_appointment(&provider, &appointment_id);
        assert_eq!(result, Err(Error::AppointmentAlreadyConfirmed));
    }

    #[test]
    fn test_refund_appointment() {
        let (env, client, admin, token) = setup();
        client.initialize(&admin, &token).unwrap();

        let patient = Address::random(&env);
        let provider = Address::random(&env);
        let amount: i128 = 1000;

        let appointment_id = client
            .book_appointment(&patient, &provider, &amount, &token)
            .unwrap();

        let result = client.refund_appointment(&patient, &appointment_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_refund_appointment_wrong_patient() {
        let (env, client, admin, token) = setup();
        client.initialize(&admin, &token).unwrap();

        let patient = Address::random(&env);
        let wrong_patient = Address::random(&env);
        let provider = Address::random(&env);
        let amount: i128 = 1000;

        let appointment_id = client
            .book_appointment(&patient, &provider, &amount, &token)
            .unwrap();

        // Try to refund as wrong patient
        let result = client.refund_appointment(&wrong_patient, &appointment_id);
        assert_eq!(result, Err(Error::OnlyPatientCanRefund));
    }

    #[test]
    fn test_refund_confirmed_appointment_fails() {
        let (env, client, admin, token) = setup();
        client.initialize(&admin, &token).unwrap();

        let patient = Address::random(&env);
        let provider = Address::random(&env);
        let amount: i128 = 1000;

        let appointment_id = client
            .book_appointment(&patient, &provider, &amount, &token)
            .unwrap();

        // Confirm appointment
        client
            .confirm_appointment(&provider, &appointment_id)
            .unwrap();

        // Try to refund confirmed appointment
        let result = client.refund_appointment(&patient, &appointment_id);
        assert_eq!(result, Err(Error::InvalidState));
    }

    #[test]
    fn test_double_refund_prevention() {
        let (env, client, admin, token) = setup();
        client.initialize(&admin, &token).unwrap();

        let patient = Address::random(&env);
        let provider = Address::random(&env);
        let amount: i128 = 1000;

        let appointment_id = client
            .book_appointment(&patient, &provider, &amount, &token)
            .unwrap();

        // Refund once
        client
            .refund_appointment(&patient, &appointment_id)
            .unwrap();

        // Try to refund again
        let result = client.refund_appointment(&patient, &appointment_id);
        assert_eq!(result, Err(Error::AppointmentAlreadyRefunded));
    }

    #[test]
    fn test_double_withdrawal_prevention_on_confirm() {
        let (env, client, admin, token) = setup();
        client.initialize(&admin, &token).unwrap();

        let patient = Address::random(&env);
        let provider = Address::random(&env);
        let amount: i128 = 1000;

        let appointment_id = client
            .book_appointment(&patient, &provider, &amount, &token)
            .unwrap();

        // Confirm appointment (funds_released becomes true)
        client
            .confirm_appointment(&provider, &appointment_id)
            .unwrap();

        // Try to refund after confirmation (should fail because funds_released is true)
        let result = client.refund_appointment(&patient, &appointment_id);
        assert_eq!(result, Err(Error::InvalidState)); // Also covers the double withdrawal case
    }

    #[test]
    fn test_get_appointment() {
        let (env, client, admin, token) = setup();
        client.initialize(&admin, &token).unwrap();

        let patient = Address::random(&env);
        let provider = Address::random(&env);
        let amount: i128 = 1000;

        let appointment_id = client
            .book_appointment(&patient, &provider, &amount, &token)
            .unwrap();

        let result = client.get_appointment(&appointment_id);
        assert!(result.is_some());
        let appointment = result.unwrap();
        assert_eq!(appointment.appointment_id, appointment_id);
        assert_eq!(appointment.patient, patient);
        assert_eq!(appointment.provider, provider);
        assert_eq!(appointment.amount, amount);
        assert_eq!(appointment.status, AppointmentStatus::Booked);
    }

    #[test]
    fn test_get_appointment_status() {
        let (env, client, admin, token) = setup();
        client.initialize(&admin, &token).unwrap();

        let patient = Address::random(&env);
        let provider = Address::random(&env);
        let amount: i128 = 1000;

        let appointment_id = client
            .book_appointment(&patient, &provider, &amount, &token)
            .unwrap();

        let status = client.get_appointment_status(&appointment_id).unwrap();
        assert_eq!(status, AppointmentStatus::Booked);

        // Confirm appointment
        client
            .confirm_appointment(&provider, &appointment_id)
            .unwrap();

        let status = client.get_appointment_status(&appointment_id).unwrap();
        assert_eq!(status, AppointmentStatus::Completed);
    }

    #[test]
    fn test_get_patient_appointments() {
        let (env, client, admin, token) = setup();
        client.initialize(&admin, &token).unwrap();

        let patient = Address::random(&env);
        let provider1 = Address::random(&env);
        let provider2 = Address::random(&env);
        let amount: i128 = 1000;

        let appt1 = client
            .book_appointment(&patient, &provider1, &amount, &token)
            .unwrap();
        let appt2 = client
            .book_appointment(&patient, &provider2, &amount, &token)
            .unwrap();

        let appointments = client.get_patient_appointments(&patient);
        assert_eq!(appointments.len(), 2);
        assert_eq!(appointments.get(0).unwrap(), appt1);
        assert_eq!(appointments.get(1).unwrap(), appt2);
    }

    #[test]
    fn test_get_provider_appointments() {
        let (env, client, admin, token) = setup();
        client.initialize(&admin, &token).unwrap();

        let patient1 = Address::random(&env);
        let patient2 = Address::random(&env);
        let provider = Address::random(&env);
        let amount: i128 = 1000;

        let appt1 = client
            .book_appointment(&patient1, &provider, &amount, &token)
            .unwrap();
        let appt2 = client
            .book_appointment(&patient2, &provider, &amount, &token)
            .unwrap();

        let appointments = client.get_provider_appointments(&provider);
        assert_eq!(appointments.len(), 2);
        assert_eq!(appointments.get(0).unwrap(), appt1);
        assert_eq!(appointments.get(1).unwrap(), appt2);
    }

    #[test]
    fn test_appointment_not_found() {
        let (env, client, admin, token) = setup();
        client.initialize(&admin, &token).unwrap();

        let result = client.get_appointment(&999);
        assert!(result.is_none());
    }

    #[test]
    fn test_appointment_state_transitions() {
        let (env, client, admin, token) = setup();
        client.initialize(&admin, &token).unwrap();

        let patient = Address::random(&env);
        let provider = Address::random(&env);
        let amount: i128 = 1000;

        let appointment_id = client
            .book_appointment(&patient, &provider, &amount, &token)
            .unwrap();

        // Check Booked state
        let appt = client.get_appointment(&appointment_id).unwrap();
        assert_eq!(appt.status, AppointmentStatus::Booked);
        assert!(!appt.funds_released);

        // Transition to Confirmed
        client
            .confirm_appointment(&provider, &appointment_id)
            .unwrap();
        let appt = client.get_appointment(&appointment_id).unwrap();
        assert_eq!(appt.status, AppointmentStatus::Completed);
        assert!(appt.funds_released);
        assert!(appt.confirmed_at > 0);
    }

    #[test]
    fn test_refund_state_transition() {
        let (env, client, admin, token) = setup();
        client.initialize(&admin, &token).unwrap();

        let patient = Address::random(&env);
        let provider = Address::random(&env);
        let amount: i128 = 1000;

        let appointment_id = client
            .book_appointment(&patient, &provider, &amount, &token)
            .unwrap();

        // Transition to Refunded
        client
            .refund_appointment(&patient, &appointment_id)
            .unwrap();
        let appt = client.get_appointment(&appointment_id).unwrap();
        assert_eq!(appt.status, AppointmentStatus::Refunded);
        assert!(appt.funds_released);
        assert!(appt.refunded_at > 0);
    }

    #[test]
    fn test_escrow_balance_calculation() {
        let (env, client, admin, token) = setup();
        client.initialize(&admin, &token).unwrap();

        let patient1 = Address::random(&env);
        let patient2 = Address::random(&env);
        let provider1 = Address::random(&env);
        let provider2 = Address::random(&env);
        let amount1: i128 = 1000;
        let amount2: i128 = 2000;

        // Book appointments
        let appt1 = client
            .book_appointment(&patient1, &provider1, &amount1, &token)
            .unwrap();
        let appt2 = client
            .book_appointment(&patient2, &provider2, &amount2, &token)
            .unwrap();

        // Check escrow balance (should include both)
        let balance = client.get_escrow_balance();
        assert_eq!(balance, amount1 + amount2);

        // Confirm one appointment
        client.confirm_appointment(&provider1, &appt1).unwrap();

        // Escrow balance should only include the booked one now
        let balance = client.get_escrow_balance();
        assert_eq!(balance, amount2);
    }

    #[test]
    fn test_multiple_appointments_same_provider() {
        let (env, client, admin, token) = setup();
        client.initialize(&admin, &token).unwrap();

        let patient1 = Address::random(&env);
        let patient2 = Address::random(&env);
        let provider = Address::random(&env);
        let amount: i128 = 1000;

        let appt1 = client
            .book_appointment(&patient1, &provider, &amount, &token)
            .unwrap();
        let appt2 = client
            .book_appointment(&patient2, &provider, &amount, &token)
            .unwrap();

        // Confirm both
        client.confirm_appointment(&provider, &appt1).unwrap();
        client.confirm_appointment(&provider, &appt2).unwrap();

        // Both should be completed
        let status1 = client.get_appointment_status(&appt1).unwrap();
        let status2 = client.get_appointment_status(&appt2).unwrap();
        assert_eq!(status1, AppointmentStatus::Completed);
        assert_eq!(status2, AppointmentStatus::Completed);
    }

    #[test]
    fn test_all_appointment_statuses() {
        let (env, client, admin, token) = setup();
        client.initialize(&admin, &token).unwrap();

        let patient1 = Address::random(&env);
        let patient2 = Address::random(&env);
        let provider1 = Address::random(&env);
        let provider2 = Address::random(&env);
        let amount: i128 = 1000;

        // Booked appointment
        let appt_booked = client
            .book_appointment(&patient1, &provider1, &amount, &token)
            .unwrap();
        assert_eq!(
            client.get_appointment_status(&appt_booked).unwrap(),
            AppointmentStatus::Booked
        );

        // Completed appointment
        let appt_completed = client
            .book_appointment(&patient1, &provider2, &amount, &token)
            .unwrap();
        client
            .confirm_appointment(&provider2, &appt_completed)
            .unwrap();
        assert_eq!(
            client.get_appointment_status(&appt_completed).unwrap(),
            AppointmentStatus::Completed
        );

        // Refunded appointment
        let appt_refunded = client
            .book_appointment(&patient2, &provider1, &amount, &token)
            .unwrap();
        client
            .refund_appointment(&patient2, &appt_refunded)
            .unwrap();
        assert_eq!(
            client.get_appointment_status(&appt_refunded).unwrap(),
            AppointmentStatus::Refunded
        );
    }

    #[test]
    fn test_error_codes_are_stable() {
        assert_eq!(Error::Unauthorized as u32, 100);
        assert_eq!(Error::OnlyPatientCanRefund as u32, 110);
        assert_eq!(Error::OnlyProviderCanConfirm as u32, 111);
        assert_eq!(Error::InvalidAmount as u32, 205);
        assert_eq!(Error::NotInitialized as u32, 300);
        assert_eq!(Error::AlreadyInitialized as u32, 301);
        assert_eq!(Error::InvalidState as u32, 304);
        assert_eq!(Error::AppointmentNotFound as u32, 410);
        assert_eq!(Error::InsufficientFunds as u32, 500);
        assert_eq!(Error::TokenTransferFailed as u32, 501);
        assert_eq!(Error::DoubleWithdrawal as u32, 505);
    }

    #[test]
    fn test_get_suggestion_returns_expected_hint() {
        use crate::errors::get_suggestion;
        use soroban_sdk::symbol_short;
        assert_eq!(get_suggestion(Error::Unauthorized), symbol_short!("CHK_AUTH"));
        assert_eq!(get_suggestion(Error::NotInitialized), symbol_short!("INIT_CTR"));
        assert_eq!(get_suggestion(Error::AlreadyInitialized), symbol_short!("ALREADY"));
        assert_eq!(get_suggestion(Error::AppointmentNotFound), symbol_short!("CHK_ID"));
        assert_eq!(get_suggestion(Error::InsufficientFunds), symbol_short!("ADD_FUND"));
    }
}
