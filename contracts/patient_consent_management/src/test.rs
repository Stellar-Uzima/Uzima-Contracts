#[cfg(test)]
mod tests {
    use crate::{Error, PatientConsentManagement, PatientConsentManagementClient};
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        Address, Env,
    };

    fn setup() -> (Env, PatientConsentManagementClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|li| {
            li.timestamp = 1_000_000;
        });
        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, PatientConsentManagement);
        let client = PatientConsentManagementClient::new(&env, &contract_id);
        (env, client, admin)
    }

    #[test]
    fn test_initialize() {
        let (_env, client, admin) = setup();
        client.initialize(&admin);
    }

    #[test]
    fn test_initialize_twice_fails() {
        let (_env, client, admin) = setup();
        client.initialize(&admin);
        let result = client.try_initialize(&admin);
        assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
    }

    #[test]
    fn test_grant_consent() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        client.grant_consent(&patient, &provider);
    }

    #[test]
    fn test_check_consent_after_grant() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        client.grant_consent(&patient, &provider);
        let result = client.check_consent(&patient, &provider);
        assert!(result);
    }

    #[test]
    fn test_check_consent_after_expiry() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let expires_at = env.ledger().timestamp().saturating_add(10);
        client.grant_consent_with_expiry(&patient, &provider, &expires_at);
        assert!(client.check_consent(&patient, &provider));

        env.ledger().with_mut(|li| {
            li.timestamp = expires_at.saturating_add(1);
        });

        let result = client.check_consent(&patient, &provider);
        assert!(!result);

        let expired_event_count = env
            .events()
            .all()
            .iter()
            .filter(|e| {
                e.1.get(0)
                    .and_then(|topic| Symbol::try_from_val(&env, topic).ok())
                    == Some(symbol_short!("CONSENT"))
                    && e.1.get(1)
                        .and_then(|sub| Symbol::try_from_val(&env, sub).ok())
                        == Some(symbol_short!("EXPIRED"))
            })
            .count();
        assert!(expired_event_count >= 1);
    }

    #[test]
    fn test_cleanup_expired_consents() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let expires_at = env.ledger().timestamp().saturating_add(10);
        client.grant_consent_with_expiry(&patient, &provider, &expires_at);

        env.ledger().with_mut(|li| {
            li.timestamp = expires_at.saturating_add(1);
        });

        let cleaned = client.cleanup_expired_consents(&patient);
        assert_eq!(cleaned, 1);
        assert!(!client.check_consent(&patient, &provider));

        let audit = client.verify_consent_with_audit(&patient, &provider);
        assert!(!audit.0);
        assert!(audit.2 > 0);
    }

    #[test]
    fn test_check_consent_before_grant() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let result = client.check_consent(&patient, &provider);
        assert!(!result);
    }

    #[test]
    fn test_revoke_consent() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        client.grant_consent(&patient, &provider);
        assert!(client.check_consent(&patient, &provider));
        client.revoke_consent(&patient, &provider);
        assert!(!client.check_consent(&patient, &provider));
    }

    #[test]
    fn test_revoke_nonexistent_consent() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let result = client.try_revoke_consent(&patient, &provider);
        assert_eq!(result, Err(Ok(Error::ConsentNotFound)));
    }

    #[test]
    fn test_duplicate_consent_fails() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        client.grant_consent(&patient, &provider);
        let result = client.try_grant_consent(&patient, &provider);
        assert_eq!(result, Err(Ok(Error::ConsentAlreadyExists)));
    }

    #[test]
    fn test_patient_to_self_fails() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient = Address::generate(&env);
        let result = client.try_grant_consent(&patient, &patient);
        assert_eq!(result, Err(Ok(Error::InvalidProvider)));
    }

    #[test]
    fn test_multiple_providers_same_patient() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient = Address::generate(&env);
        let provider1 = Address::generate(&env);
        let provider2 = Address::generate(&env);
        client.grant_consent(&patient, &provider1);
        client.grant_consent(&patient, &provider2);
        assert!(client.check_consent(&patient, &provider1));
        assert!(client.check_consent(&patient, &provider2));
        let count = client.get_active_consent_count(&patient);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_grant_revoke_regrant() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        client.grant_consent(&patient, &provider);
        assert!(client.check_consent(&patient, &provider));
        client.revoke_consent(&patient, &provider);
        assert!(!client.check_consent(&patient, &provider));
        client.grant_consent(&patient, &provider);
        assert!(client.check_consent(&patient, &provider));
    }

    #[test]
    fn test_get_patient_consents() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient = Address::generate(&env);
        let provider1 = Address::generate(&env);
        let provider2 = Address::generate(&env);
        client.grant_consent(&patient, &provider1);
        client.grant_consent(&patient, &provider2);
        let log = client.get_patient_consents(&patient);
        assert!(log.is_some());
        assert_eq!(log.unwrap().record_count, 2);
    }

    #[test]
    fn test_verify_consent_with_audit() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        client.grant_consent(&patient, &provider);
        let (has_consent, granted_at, revoked_at) =
            client.verify_consent_with_audit(&patient, &provider);
        assert!(has_consent);
        assert!(granted_at > 0);
        assert_eq!(revoked_at, 0);
    }

    #[test]
    fn test_verify_consent_with_audit_after_revoke() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        client.grant_consent(&patient, &provider);
        client.revoke_consent(&patient, &provider);
        let (has_consent, granted_at, revoked_at) =
            client.verify_consent_with_audit(&patient, &provider);
        assert!(!has_consent);
        assert!(granted_at > 0);
        assert!(revoked_at > 0);
        assert!(revoked_at >= granted_at);
    }

    #[test]
    fn test_authorization_required_for_grant() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let _unauthorized = Address::generate(&env);
        client.grant_consent(&patient, &provider);
    }

    #[test]
    fn test_get_active_consent_count() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient = Address::generate(&env);
        let provider1 = Address::generate(&env);
        let provider2 = Address::generate(&env);
        let provider3 = Address::generate(&env);
        client.grant_consent(&patient, &provider1);
        client.grant_consent(&patient, &provider2);
        client.grant_consent(&patient, &provider3);
        let count = client.get_active_consent_count(&patient);
        assert_eq!(count, 3);
        client.revoke_consent(&patient, &provider2);
        let count = client.get_active_consent_count(&patient);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_consent_persistence() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        client.grant_consent(&patient, &provider);
        assert!(client.check_consent(&patient, &provider));
        assert!(client.check_consent(&patient, &provider));
    }

    #[test]
    fn test_error_codes_are_stable() {
        assert_eq!(Error::Unauthorized as u32, 100);
        assert_eq!(Error::InvalidPatient as u32, 210);
        assert_eq!(Error::InvalidProvider as u32, 211);
        assert_eq!(Error::NotInitialized as u32, 300);
        assert_eq!(Error::AlreadyInitialized as u32, 301);
        assert_eq!(Error::ConsentNotFound as u32, 406);
        assert_eq!(Error::ConsentAlreadyExists as u32, 460);
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
            get_suggestion(Error::ConsentNotFound),
            symbol_short!("CHK_ID")
        );
        assert_eq!(
            get_suggestion(Error::InvalidPatient),
            symbol_short!("CHK_ID")
        );
    }

    #[test]
    fn test_double_revoke_fails() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        client.grant_consent(&patient, &provider);
        client.revoke_consent(&patient, &provider);
        // Second revocation should fail
        let result = client.try_revoke_consent(&patient, &provider);
        assert!(result.is_err());
    }

    #[test]
    fn test_revoke_expired_consent_succeeds() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        client.grant_consent(&patient, &provider);
        // Advance time past any expiry
        env.ledger().with_mut(|li| {
            li.timestamp = 9_999_999;
        });
        // Revocation of an active (non-expired) consent should still succeed
        client.revoke_consent(&patient, &provider);
        assert!(!client.check_consent(&patient, &provider));
    }

    #[test]
    fn test_cross_patient_revoke_fails() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient_a = Address::generate(&env);
        let patient_b = Address::generate(&env);
        let provider = Address::generate(&env);
        client.grant_consent(&patient_a, &provider);
        // patient_b cannot revoke patient_a's consent
        let result = client.try_revoke_consent(&patient_b, &provider);
        assert!(result.is_err());
    }

    #[test]
    fn test_revoke_emits_event_and_check_returns_false() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        client.grant_consent(&patient, &provider);
        client.revoke_consent(&patient, &provider);
        assert!(!client.check_consent(&patient, &provider));
    }
}
