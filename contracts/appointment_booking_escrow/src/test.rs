#[cfg(test)]
mod tests {
    use crate::{
        AppointmentBookingEscrow, AppointmentBookingEscrowClient, AppointmentStatus, Error,
    };
    use soroban_sdk::{testutils::Address as _, testutils::Ledger as _, token, Address, Env};

    fn setup() -> (
        Env,
        AppointmentBookingEscrowClient<'static>,
        Address,
        Address,
    ) {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(12345);
        let admin = Address::generate(&env);
        let token_admin = Address::generate(&env);
        let token_contract = env.register_stellar_asset_contract_v2(token_admin);
        let token_id = token_contract.address();
        let contract_id = env.register_contract(None, AppointmentBookingEscrow);
        let client = AppointmentBookingEscrowClient::new(&env, &contract_id);
        (env, client, admin, token_id)
    }

    fn mint(env: &Env, token_id: &Address, to: &Address, amount: i128) {
        token::StellarAssetClient::new(env, token_id).mint(to, &amount);
    }

    #[test]
    fn test_initialize() {
        let (_env, client, admin, token_id) = setup();
        client.initialize(&admin, &token_id);
    }

    #[test]
    fn test_initialize_twice_fails() {
        let (_env, client, admin, token_id) = setup();
        client.initialize(&admin, &token_id);
        let result = client.try_initialize(&admin, &token_id);
        assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
    }

    #[test]
    fn test_book_appointment() {
        let (env, client, admin, token_id) = setup();
        client.initialize(&admin, &token_id);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let amount: i128 = 1000;
        mint(&env, &token_id, &patient, 10000);
        let appt_id = client.book_appointment(&patient, &provider, &amount, &token_id);
        assert_eq!(appt_id, 1);
    }

    #[test]
    fn test_book_appointment_invalid_amount() {
        let (env, client, admin, token_id) = setup();
        client.initialize(&admin, &token_id);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let result = client.try_book_appointment(&patient, &provider, &0, &token_id);
        assert_eq!(result, Err(Ok(Error::InvalidAmount)));
        let result = client.try_book_appointment(&patient, &provider, &-100, &token_id);
        assert_eq!(result, Err(Ok(Error::InvalidAmount)));
    }

    #[test]
    fn test_book_appointment_self_provider() {
        let (env, client, admin, token_id) = setup();
        client.initialize(&admin, &token_id);
        let patient = Address::generate(&env);
        let amount: i128 = 1000;
        let result = client.try_book_appointment(&patient, &patient, &amount, &token_id);
        assert_eq!(result, Err(Ok(Error::InvalidProvider)));
    }

    #[test]
    fn test_multiple_appointments_increment_id() {
        let (env, client, admin, token_id) = setup();
        client.initialize(&admin, &token_id);
        let patient = Address::generate(&env);
        let provider1 = Address::generate(&env);
        let provider2 = Address::generate(&env);
        let amount: i128 = 1000;
        mint(&env, &token_id, &patient, 10000);
        let appt1 = client.book_appointment(&patient, &provider1, &amount, &token_id);
        let appt2 = client.book_appointment(&patient, &provider2, &amount, &token_id);
        assert_eq!(appt1, 1);
        assert_eq!(appt2, 2);
    }

    #[test]
    fn test_confirm_appointment() {
        let (env, client, admin, token_id) = setup();
        client.initialize(&admin, &token_id);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let amount: i128 = 1000;
        mint(&env, &token_id, &patient, 10000);
        let appointment_id = client.book_appointment(&patient, &provider, &amount, &token_id);
        client.confirm_appointment(&provider, &appointment_id);
    }

    #[test]
    fn test_confirm_appointment_wrong_provider() {
        let (env, client, admin, token_id) = setup();
        client.initialize(&admin, &token_id);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let wrong_provider = Address::generate(&env);
        let amount: i128 = 1000;
        mint(&env, &token_id, &patient, 10000);
        let appointment_id = client.book_appointment(&patient, &provider, &amount, &token_id);
        let result = client.try_confirm_appointment(&wrong_provider, &appointment_id);
        assert_eq!(result, Err(Ok(Error::OnlyProviderCanConfirm)));
    }

    #[test]
    fn test_confirm_appointment_twice_fails() {
        let (env, client, admin, token_id) = setup();
        client.initialize(&admin, &token_id);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let amount: i128 = 1000;
        mint(&env, &token_id, &patient, 10000);
        let appointment_id = client.book_appointment(&patient, &provider, &amount, &token_id);
        client.confirm_appointment(&provider, &appointment_id);
        let result = client.try_confirm_appointment(&provider, &appointment_id);
        assert_eq!(result, Err(Ok(Error::AppointmentAlreadyConfirmed)));
    }

    #[test]
    fn test_refund_appointment() {
        let (env, client, admin, token_id) = setup();
        client.initialize(&admin, &token_id);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let amount: i128 = 1000;
        mint(&env, &token_id, &patient, 10000);
        let appointment_id = client.book_appointment(&patient, &provider, &amount, &token_id);
        client.refund_appointment(&patient, &appointment_id);
    }

    #[test]
    fn test_refund_appointment_wrong_patient() {
        let (env, client, admin, token_id) = setup();
        client.initialize(&admin, &token_id);
        let patient = Address::generate(&env);
        let wrong_patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let amount: i128 = 1000;
        mint(&env, &token_id, &patient, 10000);
        let appointment_id = client.book_appointment(&patient, &provider, &amount, &token_id);
        let result = client.try_refund_appointment(&wrong_patient, &appointment_id);
        assert_eq!(result, Err(Ok(Error::OnlyPatientCanRefund)));
    }

    #[test]
    fn test_refund_confirmed_appointment_fails() {
        let (env, client, admin, token_id) = setup();
        client.initialize(&admin, &token_id);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let amount: i128 = 1000;
        mint(&env, &token_id, &patient, 10000);
        let appointment_id = client.book_appointment(&patient, &provider, &amount, &token_id);
        client.confirm_appointment(&provider, &appointment_id);
        let result = client.try_refund_appointment(&patient, &appointment_id);
        assert_eq!(result, Err(Ok(Error::InvalidState)));
    }

    #[test]
    fn test_double_refund_prevention() {
        let (env, client, admin, token_id) = setup();
        client.initialize(&admin, &token_id);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let amount: i128 = 1000;
        mint(&env, &token_id, &patient, 10000);
        let appointment_id = client.book_appointment(&patient, &provider, &amount, &token_id);
        client.refund_appointment(&patient, &appointment_id);
        let result = client.try_refund_appointment(&patient, &appointment_id);
        assert_eq!(result, Err(Ok(Error::AppointmentAlreadyRefunded)));
    }

    #[test]
    fn test_double_withdrawal_prevention_on_confirm() {
        let (env, client, admin, token_id) = setup();
        client.initialize(&admin, &token_id);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let amount: i128 = 1000;
        mint(&env, &token_id, &patient, 10000);
        let appointment_id = client.book_appointment(&patient, &provider, &amount, &token_id);
        client.confirm_appointment(&provider, &appointment_id);
        let result = client.try_refund_appointment(&patient, &appointment_id);
        assert_eq!(result, Err(Ok(Error::InvalidState)));
    }

    #[test]
    fn test_get_appointment() {
        let (env, client, admin, token_id) = setup();
        client.initialize(&admin, &token_id);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let amount: i128 = 1000;
        mint(&env, &token_id, &patient, 10000);
        let appointment_id = client.book_appointment(&patient, &provider, &amount, &token_id);
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
        let (env, client, admin, token_id) = setup();
        client.initialize(&admin, &token_id);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let amount: i128 = 1000;
        mint(&env, &token_id, &patient, 10000);
        let appointment_id = client.book_appointment(&patient, &provider, &amount, &token_id);
        let status = client.get_appointment_status(&appointment_id);
        assert_eq!(status, AppointmentStatus::Booked);
        client.confirm_appointment(&provider, &appointment_id);
        let status = client.get_appointment_status(&appointment_id);
        assert_eq!(status, AppointmentStatus::Completed);
    }

    #[test]
    fn test_get_patient_appointments() {
        let (env, client, admin, token_id) = setup();
        client.initialize(&admin, &token_id);
        let patient = Address::generate(&env);
        let provider1 = Address::generate(&env);
        let provider2 = Address::generate(&env);
        let amount: i128 = 1000;
        mint(&env, &token_id, &patient, 10000);
        let appt1 = client.book_appointment(&patient, &provider1, &amount, &token_id);
        let appt2 = client.book_appointment(&patient, &provider2, &amount, &token_id);
        let appointments = client.get_patient_appointments(&patient);
        assert_eq!(appointments.len(), 2);
        assert_eq!(appointments.get(0).unwrap(), appt1);
        assert_eq!(appointments.get(1).unwrap(), appt2);
    }

    #[test]
    fn test_get_provider_appointments() {
        let (env, client, admin, token_id) = setup();
        client.initialize(&admin, &token_id);
        let patient1 = Address::generate(&env);
        let patient2 = Address::generate(&env);
        let provider = Address::generate(&env);
        let amount: i128 = 1000;
        mint(&env, &token_id, &patient1, 10000);
        mint(&env, &token_id, &patient2, 10000);
        let appt1 = client.book_appointment(&patient1, &provider, &amount, &token_id);
        let appt2 = client.book_appointment(&patient2, &provider, &amount, &token_id);
        let appointments = client.get_provider_appointments(&provider);
        assert_eq!(appointments.len(), 2);
        assert_eq!(appointments.get(0).unwrap(), appt1);
        assert_eq!(appointments.get(1).unwrap(), appt2);
    }

    #[test]
    fn test_appointment_not_found() {
        let (_env, client, admin, token_id) = setup();
        client.initialize(&admin, &token_id);
        let result = client.get_appointment(&999);
        assert!(result.is_none());
    }

    #[test]
    fn test_appointment_state_transitions() {
        let (env, client, admin, token_id) = setup();
        client.initialize(&admin, &token_id);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let amount: i128 = 1000;
        mint(&env, &token_id, &patient, 10000);
        let appointment_id = client.book_appointment(&patient, &provider, &amount, &token_id);
        let appt = client.get_appointment(&appointment_id).unwrap();
        assert_eq!(appt.status, AppointmentStatus::Booked);
        assert!(!appt.funds_released);
        client.confirm_appointment(&provider, &appointment_id);
        let appt = client.get_appointment(&appointment_id).unwrap();
        assert_eq!(appt.status, AppointmentStatus::Completed);
        assert!(appt.funds_released);
        assert!(appt.confirmed_at > 0);
    }

    #[test]
    fn test_refund_state_transition() {
        let (env, client, admin, token_id) = setup();
        client.initialize(&admin, &token_id);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let amount: i128 = 1000;
        mint(&env, &token_id, &patient, 10000);
        let appointment_id = client.book_appointment(&patient, &provider, &amount, &token_id);
        client.refund_appointment(&patient, &appointment_id);
        let appt = client.get_appointment(&appointment_id).unwrap();
        assert_eq!(appt.status, AppointmentStatus::Refunded);
        assert!(appt.funds_released);
        assert!(appt.refunded_at > 0);
    }

    #[test]
    fn test_escrow_balance_calculation() {
        let (env, client, admin, token_id) = setup();
        client.initialize(&admin, &token_id);
        let patient1 = Address::generate(&env);
        let patient2 = Address::generate(&env);
        let provider1 = Address::generate(&env);
        let provider2 = Address::generate(&env);
        let amount1: i128 = 1000;
        let amount2: i128 = 2000;
        mint(&env, &token_id, &patient1, 10000);
        mint(&env, &token_id, &patient2, 10000);
        let appt1 = client.book_appointment(&patient1, &provider1, &amount1, &token_id);
        let _appt2 = client.book_appointment(&patient2, &provider2, &amount2, &token_id);
        let balance = client.get_escrow_balance();
        assert_eq!(balance, amount1 + amount2);
        client.confirm_appointment(&provider1, &appt1);
        let balance = client.get_escrow_balance();
        assert_eq!(balance, amount2);
    }

    #[test]
    fn test_multiple_appointments_same_provider() {
        let (env, client, admin, token_id) = setup();
        client.initialize(&admin, &token_id);
        let patient1 = Address::generate(&env);
        let patient2 = Address::generate(&env);
        let provider = Address::generate(&env);
        let amount: i128 = 1000;
        mint(&env, &token_id, &patient1, 10000);
        mint(&env, &token_id, &patient2, 10000);
        let appt1 = client.book_appointment(&patient1, &provider, &amount, &token_id);
        let appt2 = client.book_appointment(&patient2, &provider, &amount, &token_id);
        client.confirm_appointment(&provider, &appt1);
        client.confirm_appointment(&provider, &appt2);
        let status1 = client.get_appointment_status(&appt1);
        let status2 = client.get_appointment_status(&appt2);
        assert_eq!(status1, AppointmentStatus::Completed);
        assert_eq!(status2, AppointmentStatus::Completed);
    }

    #[test]
    fn test_all_appointment_statuses() {
        let (env, client, admin, token_id) = setup();
        client.initialize(&admin, &token_id);
        let patient1 = Address::generate(&env);
        let patient2 = Address::generate(&env);
        let provider1 = Address::generate(&env);
        let provider2 = Address::generate(&env);
        let amount: i128 = 1000;
        mint(&env, &token_id, &patient1, 10000);
        mint(&env, &token_id, &patient2, 10000);
        let appt_booked = client.book_appointment(&patient1, &provider1, &amount, &token_id);
        assert_eq!(
            client.get_appointment_status(&appt_booked),
            AppointmentStatus::Booked
        );
        let appt_completed = client.book_appointment(&patient1, &provider2, &amount, &token_id);
        client.confirm_appointment(&provider2, &appt_completed);
        assert_eq!(
            client.get_appointment_status(&appt_completed),
            AppointmentStatus::Completed
        );
        let appt_refunded = client.book_appointment(&patient2, &provider1, &amount, &token_id);
        client.refund_appointment(&patient2, &appt_refunded);
        assert_eq!(
            client.get_appointment_status(&appt_refunded),
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
        assert_eq!(
            get_suggestion(Error::Unauthorized),
            symbol_short!("CHK_AUTH")
        );
        assert_eq!(
            get_suggestion(Error::NotInitialized),
            symbol_short!("INIT_CTR")
        );
        assert_eq!(
            get_suggestion(Error::AlreadyInitialized),
            symbol_short!("ALREADY")
        );
        assert_eq!(
            get_suggestion(Error::AppointmentNotFound),
            symbol_short!("CHK_ID")
        );
        assert_eq!(
            get_suggestion(Error::InsufficientFunds),
            symbol_short!("ADD_FUND")
        );
    }
}
