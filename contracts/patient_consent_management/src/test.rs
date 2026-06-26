#[cfg(test)]
mod tests {
    use crate::{Error, PatientConsentManagement, PatientConsentManagementClient};
    use soroban_sdk::{
        symbol_short,
        testutils::{Address as _, Events, Ledger},
        Address, Env, Symbol, TryFromVal,
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
                    .and_then(|topic| Symbol::try_from_val(&env, &topic).ok())
                    == Some(symbol_short!("CONSENT"))
                    && e.1
                        .get(1)
                        .and_then(|sub| Symbol::try_from_val(&env, &sub).ok())
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

    // ── Timestamp / Time-dependent edge case tests ──────────────────

    /// Test: consent expiry boundary - exactly at expiry
    #[test]
    fn test_consent_expires_exactly_at_boundary() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let now = env.ledger().timestamp();
        let expires_at = now + 100;
        client.grant_consent_with_expiry(&patient, &provider, &expires_at);

        // Exactly at expiry - should be expired
        env.ledger().with_mut(|li| {
            li.timestamp = expires_at;
        });
        let result = client.check_consent(&patient, &provider);
        assert!(!result, "Consent should be expired at expiry boundary");
    }

    /// Test: consent expiry - 1 second before expiry
    #[test]
    fn test_consent_one_second_before_expiry() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let now = env.ledger().timestamp();
        let expires_at = now + 100;
        client.grant_consent_with_expiry(&patient, &provider, &expires_at);

        // 1 second before expiry - should still be active
        env.ledger().with_mut(|li| {
            li.timestamp = expires_at - 1;
        });
        let result = client.check_consent(&patient, &provider);
        assert!(result, "Consent should be active 1 second before expiry");
    }

    /// Test: consent expiry - 1 second after expiry
    #[test]
    fn test_consent_one_second_after_expiry() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let now = env.ledger().timestamp();
        let expires_at = now + 100;
        client.grant_consent_with_expiry(&patient, &provider, &expires_at);

        // 1 second after expiry - should be expired
        env.ledger().with_mut(|li| {
            li.timestamp = expires_at + 1;
        });
        let result = client.check_consent(&patient, &provider);
        assert!(!result, "Consent should be expired 1 second after expiry");
    }

    /// Test: time manipulation - far future timestamp
    #[test]
    fn test_far_future_timestamp_consent_expired() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let now = env.ledger().timestamp();
        let expires_at = now + 100;
        client.grant_consent_with_expiry(&patient, &provider, &expires_at);

        // Far future - consent should be expired
        env.ledger().with_mut(|li| {
            li.timestamp = 9_999_999_999;
        });
        let result = client.check_consent(&patient, &provider);
        assert!(!result, "Consent should be expired in far future");
    }

    /// Test: consent with no expiry (expires_at = 0) never expires
    #[test]
    fn test_consent_with_no_expiry_never_expires() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        client.grant_consent(&patient, &provider);

        // Far future - consent should still be active (no expiry)
        env.ledger().with_mut(|li| {
            li.timestamp = 9_999_999_999;
        });
        let result = client.check_consent(&patient, &provider);
        assert!(result, "Consent without expiry should never expire");
    }

    /// Test: large time jump (epoch overflow edge case)
    #[test]
    fn test_large_time_jump_no_expiry() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        client.grant_consent(&patient, &provider);

        // Very large timestamp - should still work (no expiry)
        env.ledger().with_mut(|li| {
            li.timestamp = u64::MAX - 1;
        });
        let result = client.check_consent(&patient, &provider);
        assert!(result, "Consent without expiry should work at u64::MAX");
    }

    /// Test: grant_consent_with_expiry with expires_at == now fails
    #[test]
    fn test_expiry_at_current_time_fails() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let now = env.ledger().timestamp();
        let result = client.try_grant_consent_with_expiry(&patient, &provider, &now);
        assert_eq!(result, Err(Ok(Error::InvalidExpiry)));
    }

    /// Test: grant_consent_with_expiry with expires_at in past fails
    #[test]
    fn test_expiry_in_past_fails() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let now = env.ledger().timestamp();
        let result = client.try_grant_consent_with_expiry(&patient, &provider, &(now - 1));
        assert_eq!(result, Err(Ok(Error::InvalidExpiry)));
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

    // ── Property-Based Tests (Issue #832) ─────────────────────────

    // Property 1: Idempotent revoke - revoking already revoked consent fails
    #[test]
    fn proptest_revoke_idempotency() {
        use proptest::proptest;
        proptest!(|(seed in 1u64..=1000)| {
            let env = Env::default();
            env.mock_all_auths();
            env.ledger().with_mut(|li| {
                li.timestamp = 1_000_000;
            });
            let admin = Address::generate(&env);
            let contract_id = env.register_contract(None, PatientConsentManagement);
            let client = PatientConsentManagementClient::new(&env, &contract_id);
            client.initialize(&admin);

            let patient = Address::generate(&env);
            let provider = Address::generate(&env);
            
            client.grant_consent(&patient, &provider);
            client.revoke_consent(&patient, &provider);
            
            // Second revoke should fail (idempotent)
            let result = client.try_revoke_consent(&patient, &provider);
            prop_assert!(result.is_err(), "Second revoke should fail at seed {}", seed);
        });
    }

    // Property 2: Consent counter monotonicity - granting increases active count
    #[test]
    fn proptest_consent_counter_monotonicity() {
        use proptest::proptest;
        proptest!(|(provider_count in 1usize..=20)| {
            let env = Env::default();
            env.mock_all_auths();
            env.ledger().with_mut(|li| {
                li.timestamp = 1_000_000;
            });
            let admin = Address::generate(&env);
            let contract_id = env.register_contract(None, PatientConsentManagement);
            let client = PatientConsentManagementClient::new(&env, &contract_id);
            client.initialize(&admin);

            let patient = Address::generate(&env);
            let mut prev_count = 0u32;
            
            for _ in 0..provider_count {
                let provider = Address::generate(&env);
                client.grant_consent(&patient, &provider);
                let count = client.get_active_consent_count(&patient);
                prop_assert_eq!(count, prev_count + 1, 
                    "Consent count must increase by 1 after grant");
                prev_count = count;
            }
        });
    }

    // Property 3: Sum of granted consents <= total consent attempts
    #[test]
    fn proptest_consent_sum_invariant() {
        use proptest::proptest;
        proptest!(|(operations in proptest::collection::vec(0u32..2, 1..=50))| {
            let env = Env::default();
            env.mock_all_auths();
            env.ledger().with_mut(|li| {
                li.timestamp = 1_000_000;
            });
            let admin = Address::generate(&env);
            let contract_id = env.register_contract(None, PatientConsentManagement);
            let client = PatientConsentManagementClient::new(&env, &contract_id);
            client.initialize(&admin);

            let patient = Address::generate(&env);
            let mut grant_count = 0u32;
            let mut total_attempts = 0u32;
            
            for op in operations {
                if op == 0 {
                    let provider = Address::generate(&env);
                    let res = client.try_grant_consent(&patient, &provider);
                    if res.is_ok() {
                        grant_count += 1;
                    }
                    total_attempts += 1;
                }
            }
            
            let actual_count = client.get_active_consent_count(&patient);
            prop_assert!(actual_count <= total_attempts as u32,
                "Sum of active consents ({}) must be <= total attempts ({})", 
                actual_count, total_attempts);
        });
    }

    // Property 4: After revoke, consent status is always false
    #[test]
    fn proptest_post_revoke_status() {
        use proptest::proptest;
        proptest!(|(check_count in 1usize..=20)| {
            let env = Env::default();
            env.mock_all_auths();
            env.ledger().with_mut(|li| {
                li.timestamp = 1_000_000;
            });
            let admin = Address::generate(&env);
            let contract_id = env.register_contract(None, PatientConsentManagement);
            let client = PatientConsentManagementClient::new(&env, &contract_id);
            client.initialize(&admin);

            let patient = Address::generate(&env);
            let provider = Address::generate(&env);
            
            client.grant_consent(&patient, &provider);
            client.revoke_consent(&patient, &provider);
            
            // Multiple checks after revoke should all return false
            for _ in 0..check_count {
                let status = client.check_consent(&patient, &provider);
                prop_assert!(!status, 
                    "Consent status must be false after revoke, regardless of checks");
            }
        });
    }

    // Property 5: Expiry handling - expired consents are not active
    #[test]
    fn proptest_expiry_invariant() {
        use proptest::proptest;
        proptest!(|(time_offset in 1u64..=10000)| {
            let env = Env::default();
            env.mock_all_auths();
            let base_time = 1_000_000u64;
            env.ledger().with_mut(|li| {
                li.timestamp = base_time;
            });
            let admin = Address::generate(&env);
            let contract_id = env.register_contract(None, PatientConsentManagement);
            let client = PatientConsentManagementClient::new(&env, &contract_id);
            client.initialize(&admin);

            let patient = Address::generate(&env);
            let provider = Address::generate(&env);
            let expires_at = base_time + time_offset;
            
            client.grant_consent_with_expiry(&patient, &provider, &expires_at);
            
            // Before expiry - should be active
            env.ledger().with_mut(|li| {
                li.timestamp = expires_at.saturating_sub(1);
            });
            let before_expiry = client.check_consent(&patient, &provider);
            prop_assert!(before_expiry, "Consent must be active before expiry");
            
            // After expiry - should be inactive
            env.ledger().with_mut(|li| {
                li.timestamp = expires_at + 1;
            });
            let after_expiry = client.check_consent(&patient, &provider);
            prop_assert!(!after_expiry, "Consent must be inactive after expiry");
        });
    }

    // Property 6: Grant-revoke-grant cycle is always valid
    #[test]
    fn proptest_grant_revoke_grant_cycle() {
        use proptest::proptest;
        proptest!(|(cycle_count in 1usize..=10)| {
            let env = Env::default();
            env.mock_all_auths();
            env.ledger().with_mut(|li| {
                li.timestamp = 1_000_000;
            });
            let admin = Address::generate(&env);
            let contract_id = env.register_contract(None, PatientConsentManagement);
            let client = PatientConsentManagementClient::new(&env, &contract_id);
            client.initialize(&admin);

            let patient = Address::generate(&env);
            let provider = Address::generate(&env);
            
            for cycle in 0..cycle_count {
                // Grant
                client.grant_consent(&patient, &provider);
                let after_grant = client.check_consent(&patient, &provider);
                prop_assert!(after_grant, "Consent must be active after grant (cycle {})", cycle);
                
                // Revoke
                client.revoke_consent(&patient, &provider);
                let after_revoke = client.check_consent(&patient, &provider);
                prop_assert!(!after_revoke, "Consent must be inactive after revoke (cycle {})", cycle);
            }
        });
    }
}
