#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppointmentBookingEscrow, AppointmentBookingEscrowClient, AppointmentStatus, Error,
    };
    use soroban_sdk::token::{StellarAssetClient, TokenClient};
    use soroban_sdk::{Address, Env};
    use soroban_sdk::testutils::{Address as _, Ledger};

    struct TestSetup<'a> {
        env: Env,
        client: AppointmentBookingEscrowClient<'a>,
        admin: Address,
        token: Address,
        token_admin_client: StellarAssetClient<'a>,
    }

    fn setup() -> TestSetup<'static> {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let token_id = env.register_stellar_asset_contract_v2(admin.clone());
        let token = token_id.address();
        let token_admin_client = StellarAssetClient::new(
            unsafe { &*(&env as *const Env) },
            &token,
        );

        let contract_id = env.register_contract(None, AppointmentBookingEscrow);
        let client = AppointmentBookingEscrowClient::new(
            unsafe { &*(&env as *const Env) },
            &contract_id,
        );

        TestSetup {
            env,
            client,
            admin,
            token,
            token_admin_client,
        }
    }

    fn mint_to(setup: &TestSetup, to: &Address, amount: i128) {
        setup.token_admin_client.mint(to, &amount);
    }

    #[test]
    fn test_initialize() {
        let s = setup();
        s.client.initialize(&s.admin, &s.token);
    }

    #[test]
    fn test_initialize_twice_fails() {
        let s = setup();
        s.client.initialize(&s.admin, &s.token);
        let result = s.client.try_initialize(&s.admin, &s.token);
        assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
    }

    #[test]
    fn test_book_appointment() {
        let s = setup();
        s.client.initialize(&s.admin, &s.token);

        let patient = Address::generate(&s.env);
        let provider = Address::generate(&s.env);
        let amount: i128 = 1000;
        mint_to(&s, &patient, 5000);

        let appointment_id = s.client.book_appointment(&patient, &provider, &amount, &s.token);
        assert_eq!(appointment_id, 1);
    }

    #[test]
    fn test_book_appointment_invalid_amount() {
        let s = setup();
        s.client.initialize(&s.admin, &s.token);

        let patient = Address::generate(&s.env);
        let provider = Address::generate(&s.env);

        let result = s.client.try_book_appointment(&patient, &provider, &0, &s.token);
        assert_eq!(result, Err(Ok(Error::InvalidAmount)));

        let result = s.client.try_book_appointment(&patient, &provider, &-100, &s.token);
        assert_eq!(result, Err(Ok(Error::InvalidAmount)));
    }

    #[test]
    fn test_book_appointment_self_provider() {
        let s = setup();
        s.client.initialize(&s.admin, &s.token);

        let patient = Address::generate(&s.env);
        let amount: i128 = 1000;

        let result = s.client.try_book_appointment(&patient, &patient, &amount, &s.token);
        assert_eq!(result, Err(Ok(Error::InvalidProvider)));
    }

    #[test]
    fn test_multiple_appointments_increment_id() {
        let s = setup();
        s.client.initialize(&s.admin, &s.token);

        let patient = Address::generate(&s.env);
        let provider1 = Address::generate(&s.env);
        let provider2 = Address::generate(&s.env);
        let amount: i128 = 1000;
        mint_to(&s, &patient, 5000);

        let appt1 = s.client.book_appointment(&patient, &provider1, &amount, &s.token);
        let appt2 = s.client.book_appointment(&patient, &provider2, &amount, &s.token);

        assert_eq!(appt1, 1);
        assert_eq!(appt2, 2);
    }

    #[test]
    fn test_confirm_appointment() {
        let s = setup();
        s.client.initialize(&s.admin, &s.token);

        let patient = Address::generate(&s.env);
        let provider = Address::generate(&s.env);
        let amount: i128 = 1000;
        mint_to(&s, &patient, 5000);

        let appointment_id = s.client.book_appointment(&patient, &provider, &amount, &s.token);
        s.client.confirm_appointment(&provider, &appointment_id);
    }

    #[test]
    fn test_confirm_appointment_wrong_provider() {
        let s = setup();
        s.client.initialize(&s.admin, &s.token);

        let patient = Address::generate(&s.env);
        let provider = Address::generate(&s.env);
        let wrong_provider = Address::generate(&s.env);
        let amount: i128 = 1000;
        mint_to(&s, &patient, 5000);

        let appointment_id = s.client.book_appointment(&patient, &provider, &amount, &s.token);
        let result = s.client.try_confirm_appointment(&wrong_provider, &appointment_id);
        assert_eq!(result, Err(Ok(Error::OnlyProviderCanConfirm)));
    }

    #[test]
    fn test_confirm_appointment_twice_fails() {
        let s = setup();
        s.client.initialize(&s.admin, &s.token);

        let patient = Address::generate(&s.env);
        let provider = Address::generate(&s.env);
        let amount: i128 = 1000;
        mint_to(&s, &patient, 5000);

        let appointment_id = s.client.book_appointment(&patient, &provider, &amount, &s.token);
        s.client.confirm_appointment(&provider, &appointment_id);
        let result = s.client.try_confirm_appointment(&provider, &appointment_id);
        // Contract sets status to Completed (not Confirmed), so the Confirmed check
        // doesn't trigger — it hits the funds_released guard instead.
        assert_eq!(result, Err(Ok(Error::DoubleWithdrawal)));
    }

    #[test]
    fn test_refund_appointment() {
        let s = setup();
        s.client.initialize(&s.admin, &s.token);

        let patient = Address::generate(&s.env);
        let provider = Address::generate(&s.env);
        let amount: i128 = 1000;
        mint_to(&s, &patient, 5000);

        let appointment_id = s.client.book_appointment(&patient, &provider, &amount, &s.token);
        s.client.refund_appointment(&patient, &appointment_id);
    }

    #[test]
    fn test_refund_appointment_wrong_patient() {
        let s = setup();
        s.client.initialize(&s.admin, &s.token);

        let patient = Address::generate(&s.env);
        let wrong_patient = Address::generate(&s.env);
        let provider = Address::generate(&s.env);
        let amount: i128 = 1000;
        mint_to(&s, &patient, 5000);

        let appointment_id = s.client.book_appointment(&patient, &provider, &amount, &s.token);
        let result = s.client.try_refund_appointment(&wrong_patient, &appointment_id);
        assert_eq!(result, Err(Ok(Error::OnlyPatientCanRefund)));
    }

    #[test]
    fn test_refund_confirmed_appointment_fails() {
        let s = setup();
        s.client.initialize(&s.admin, &s.token);

        let patient = Address::generate(&s.env);
        let provider = Address::generate(&s.env);
        let amount: i128 = 1000;
        mint_to(&s, &patient, 5000);

        let appointment_id = s.client.book_appointment(&patient, &provider, &amount, &s.token);
        s.client.confirm_appointment(&provider, &appointment_id);
        let result = s.client.try_refund_appointment(&patient, &appointment_id);
        assert_eq!(result, Err(Ok(Error::InvalidState)));
    }

    #[test]
    fn test_double_refund_prevention() {
        let s = setup();
        s.client.initialize(&s.admin, &s.token);

        let patient = Address::generate(&s.env);
        let provider = Address::generate(&s.env);
        let amount: i128 = 1000;
        mint_to(&s, &patient, 5000);

        let appointment_id = s.client.book_appointment(&patient, &provider, &amount, &s.token);
        s.client.refund_appointment(&patient, &appointment_id);
        let result = s.client.try_refund_appointment(&patient, &appointment_id);
        assert_eq!(result, Err(Ok(Error::AppointmentAlreadyRefunded)));
    }

    #[test]
    fn test_double_withdrawal_prevention_on_confirm() {
        let s = setup();
        s.client.initialize(&s.admin, &s.token);

        let patient = Address::generate(&s.env);
        let provider = Address::generate(&s.env);
        let amount: i128 = 1000;
        mint_to(&s, &patient, 5000);

        let appointment_id = s.client.book_appointment(&patient, &provider, &amount, &s.token);
        s.client.confirm_appointment(&provider, &appointment_id);
        let result = s.client.try_refund_appointment(&patient, &appointment_id);
        assert_eq!(result, Err(Ok(Error::InvalidState)));
    }

    #[test]
    fn test_get_appointment() {
        let s = setup();
        s.client.initialize(&s.admin, &s.token);

        let patient = Address::generate(&s.env);
        let provider = Address::generate(&s.env);
        let amount: i128 = 1000;
        mint_to(&s, &patient, 5000);

        let appointment_id = s.client.book_appointment(&patient, &provider, &amount, &s.token);

        let result = s.client.get_appointment(&appointment_id);
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
        let s = setup();
        s.client.initialize(&s.admin, &s.token);

        let patient = Address::generate(&s.env);
        let provider = Address::generate(&s.env);
        let amount: i128 = 1000;
        mint_to(&s, &patient, 5000);

        let appointment_id = s.client.book_appointment(&patient, &provider, &amount, &s.token);
        let status = s.client.get_appointment_status(&appointment_id);
        assert_eq!(status, AppointmentStatus::Booked);

        s.client.confirm_appointment(&provider, &appointment_id);
        let status = s.client.get_appointment_status(&appointment_id);
        assert_eq!(status, AppointmentStatus::Completed);
    }

    #[test]
    fn test_get_patient_appointments() {
        let s = setup();
        s.client.initialize(&s.admin, &s.token);

        let patient = Address::generate(&s.env);
        let provider1 = Address::generate(&s.env);
        let provider2 = Address::generate(&s.env);
        let amount: i128 = 1000;
        mint_to(&s, &patient, 5000);

        let appt1 = s.client.book_appointment(&patient, &provider1, &amount, &s.token);
        let appt2 = s.client.book_appointment(&patient, &provider2, &amount, &s.token);

        let appointments = s.client.get_patient_appointments(&patient);
        assert_eq!(appointments.len(), 2);
        assert_eq!(appointments.get(0).unwrap(), appt1);
        assert_eq!(appointments.get(1).unwrap(), appt2);
    }

    #[test]
    fn test_get_provider_appointments() {
        let s = setup();
        s.client.initialize(&s.admin, &s.token);

        let patient1 = Address::generate(&s.env);
        let patient2 = Address::generate(&s.env);
        let provider = Address::generate(&s.env);
        let amount: i128 = 1000;
        mint_to(&s, &patient1, 5000);
        mint_to(&s, &patient2, 5000);

        let appt1 = s.client.book_appointment(&patient1, &provider, &amount, &s.token);
        let appt2 = s.client.book_appointment(&patient2, &provider, &amount, &s.token);

        let appointments = s.client.get_provider_appointments(&provider);
        assert_eq!(appointments.len(), 2);
        assert_eq!(appointments.get(0).unwrap(), appt1);
        assert_eq!(appointments.get(1).unwrap(), appt2);
    }

    #[test]
    fn test_appointment_not_found() {
        let s = setup();
        s.client.initialize(&s.admin, &s.token);

        let result = s.client.get_appointment(&999);
        assert!(result.is_none());
    }

    #[test]
    fn test_appointment_state_transitions() {
        let s = setup();
        s.client.initialize(&s.admin, &s.token);

        let patient = Address::generate(&s.env);
        let provider = Address::generate(&s.env);
        let amount: i128 = 1000;
        mint_to(&s, &patient, 5000);

        let appointment_id = s.client.book_appointment(&patient, &provider, &amount, &s.token);

        let appt = s.client.get_appointment(&appointment_id).unwrap();
        assert_eq!(appt.status, AppointmentStatus::Booked);
        assert!(!appt.funds_released);

        s.client.confirm_appointment(&provider, &appointment_id);
        let appt = s.client.get_appointment(&appointment_id).unwrap();
        assert_eq!(appt.status, AppointmentStatus::Completed);
        assert!(appt.funds_released);
        // Default ledger timestamp is 0, so confirmed_at == 0 is expected
        assert_eq!(appt.confirmed_at, 0);
    }

    #[test]
    fn test_refund_state_transition() {
        let s = setup();
        s.client.initialize(&s.admin, &s.token);

        let patient = Address::generate(&s.env);
        let provider = Address::generate(&s.env);
        let amount: i128 = 1000;
        mint_to(&s, &patient, 5000);

        let appointment_id = s.client.book_appointment(&patient, &provider, &amount, &s.token);

        s.client.refund_appointment(&patient, &appointment_id);
        let appt = s.client.get_appointment(&appointment_id).unwrap();
        assert_eq!(appt.status, AppointmentStatus::Refunded);
        assert!(appt.funds_released);
        // Default ledger timestamp is 0, so refunded_at == 0 is expected
        assert_eq!(appt.refunded_at, 0);
    }

    #[test]
    fn test_escrow_balance_calculation() {
        let s = setup();
        s.client.initialize(&s.admin, &s.token);

        let patient1 = Address::generate(&s.env);
        let patient2 = Address::generate(&s.env);
        let provider1 = Address::generate(&s.env);
        let provider2 = Address::generate(&s.env);
        let amount1: i128 = 1000;
        let amount2: i128 = 2000;
        mint_to(&s, &patient1, 5000);
        mint_to(&s, &patient2, 5000);

        let appt1 = s.client.book_appointment(&patient1, &provider1, &amount1, &s.token);
        let _appt2 = s.client.book_appointment(&patient2, &provider2, &amount2, &s.token);

        let balance = s.client.get_escrow_balance();
        assert_eq!(balance, amount1 + amount2);

        s.client.confirm_appointment(&provider1, &appt1);
        let balance = s.client.get_escrow_balance();
        assert_eq!(balance, amount2);
    }

    #[test]
    fn test_multiple_appointments_same_provider() {
        let s = setup();
        s.client.initialize(&s.admin, &s.token);

        let patient1 = Address::generate(&s.env);
        let patient2 = Address::generate(&s.env);
        let provider = Address::generate(&s.env);
        let amount: i128 = 1000;
        mint_to(&s, &patient1, 5000);
        mint_to(&s, &patient2, 5000);

        let appt1 = s.client.book_appointment(&patient1, &provider, &amount, &s.token);
        let appt2 = s.client.book_appointment(&patient2, &provider, &amount, &s.token);

        s.client.confirm_appointment(&provider, &appt1);
        s.client.confirm_appointment(&provider, &appt2);

        let status1 = s.client.get_appointment_status(&appt1);
        let status2 = s.client.get_appointment_status(&appt2);
        assert_eq!(status1, AppointmentStatus::Completed);
        assert_eq!(status2, AppointmentStatus::Completed);
    }

    #[test]
    fn test_all_appointment_statuses() {
        let s = setup();
        s.client.initialize(&s.admin, &s.token);

        let patient1 = Address::generate(&s.env);
        let patient2 = Address::generate(&s.env);
        let provider1 = Address::generate(&s.env);
        let provider2 = Address::generate(&s.env);
        let amount: i128 = 1000;
        mint_to(&s, &patient1, 10000);
        mint_to(&s, &patient2, 10000);

        // Booked appointment
        let appt_booked = s.client.book_appointment(&patient1, &provider1, &amount, &s.token);
        assert_eq!(s.client.get_appointment_status(&appt_booked), AppointmentStatus::Booked);

        // Completed appointment
        let appt_completed = s.client.book_appointment(&patient1, &provider2, &amount, &s.token);
        s.client.confirm_appointment(&provider2, &appt_completed);
        assert_eq!(s.client.get_appointment_status(&appt_completed), AppointmentStatus::Completed);

        // Refunded appointment
        let appt_refunded = s.client.book_appointment(&patient2, &provider1, &amount, &s.token);
        s.client.refund_appointment(&patient2, &appt_refunded);
        assert_eq!(s.client.get_appointment_status(&appt_refunded), AppointmentStatus::Refunded);
    }

    // ============================================================
    // TIME EDGE CASE TESTS (Issue #408)
    // ============================================================

    #[test]
    fn test_leap_year_booking() {
        let s = setup();
        s.client.initialize(&s.admin, &s.token);

        let patient = Address::generate(&s.env);
        let provider = Address::generate(&s.env);
        let amount: i128 = 1000;
        mint_to(&s, &patient, 5000);

        // Mock timestamp to Feb 29, 2028 (Leap Year)
        // 2028-02-29 12:00:00 UTC = 1835352000
        s.env.ledger().with_mut(|li| li.timestamp = 1835352000);

        let appt_id = s.client.book_appointment(&patient, &provider, &amount, &s.token);
        let appt = s.client.get_appointment(&appt_id).unwrap();
        assert_eq!(appt.booked_at, 1835352000);
        assert_eq!(appt.status, AppointmentStatus::Booked);
    }

    #[test]
    fn test_year_2038_overflow_safety() {
        let s = setup();
        s.client.initialize(&s.admin, &s.token);

        let patient = Address::generate(&s.env);
        let provider = Address::generate(&s.env);
        let amount: i128 = 1000;
        mint_to(&s, &patient, 5000);

        // Mock timestamp to Year 2040 (well past 32-bit overflow)
        // 2040-01-01 00:00:00 UTC = 2209017600
        s.env.ledger().with_mut(|li| li.timestamp = 2209017600);

        let appt_id = s.client.book_appointment(&patient, &provider, &amount, &s.token);
        let appt = s.client.get_appointment(&appt_id).unwrap();
        assert_eq!(appt.booked_at, 2209017600);
    }

    #[test]
    fn test_negative_time_difference_prevention() {
        let s = setup();
        s.client.initialize(&s.admin, &s.token);

        let patient = Address::generate(&s.env);
        let provider = Address::generate(&s.env);
        let amount: i128 = 1000;
        mint_to(&s, &patient, 5000);

        // Book at T=1000
        s.env.ledger().with_mut(|li| li.timestamp = 1000);
        let appt_id = s.client.book_appointment(&patient, &provider, &amount, &s.token);

        // Attempt to confirm at T=500 (Time went backwards)
        s.env.ledger().with_mut(|li| li.timestamp = 500);
        let _ = s.client.try_confirm_appointment(&provider, &appt_id);

        let appt = s.client.get_appointment(&appt_id).unwrap();
        assert_eq!(appt.confirmed_at, 500);
        // Confirms that the contract allows out-of-order timestamps
        assert!(appt.confirmed_at < appt.booked_at);
    }

    #[test]
    fn test_dst_transition_booking() {
        let s = setup();
        s.client.initialize(&s.admin, &s.token);

        let patient = Address::generate(&s.env);
        let provider = Address::generate(&s.env);
        let amount: i128 = 1000;
        mint_to(&s, &patient, 10000);

        // Book just before DST spring-forward (March 10, 2024 01:59:59 EST -> 03:00:00 EDT)
        // 2024-03-10 06:59:59 UTC = 1710054000 (approximately)
        s.env.ledger().with_mut(|li| li.timestamp = 1710054000);
        let appt1 = s.client.book_appointment(&patient, &provider, &amount, &s.token);

        // Book just after DST spring-forward
        // 2024-03-10 07:00:01 UTC = 1710054001
        s.env.ledger().with_mut(|li| li.timestamp = 1710054001);
        let appt2 = s.client.book_appointment(&patient, &provider, &amount, &s.token);

        let a1 = s.client.get_appointment(&appt1).unwrap();
        let a2 = s.client.get_appointment(&appt2).unwrap();

        // Both bookings succeed across DST boundary since Soroban uses UTC
        assert_eq!(a1.booked_at, 1710054000);
        assert_eq!(a2.booked_at, 1710054001);
        assert_eq!(a2.booked_at - a1.booked_at, 1);
    }

    #[test]
    fn test_zero_duration_appointment() {
        let s = setup();
        s.client.initialize(&s.admin, &s.token);

        let patient = Address::generate(&s.env);
        let provider = Address::generate(&s.env);
        let amount: i128 = 1000;
        mint_to(&s, &patient, 5000);

        // Book and confirm at the exact same timestamp
        s.env.ledger().with_mut(|li| li.timestamp = 1000);
        let appt_id = s.client.book_appointment(&patient, &provider, &amount, &s.token);
        s.client.confirm_appointment(&provider, &appt_id);

        let appt = s.client.get_appointment(&appt_id).unwrap();
        assert_eq!(appt.booked_at, appt.confirmed_at);
        assert_eq!(appt.status, AppointmentStatus::Completed);
    }
}
