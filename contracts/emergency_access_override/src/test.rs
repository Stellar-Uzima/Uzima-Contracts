#[cfg(test)]
mod tests {
    use crate::{EmergencyAccessOverride, EmergencyAccessOverrideClient, Error};
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        Address, Env, Vec,
    };

    fn setup() -> (
        Env,
        EmergencyAccessOverrideClient<'static>,
        Address,
        Address,
        Address,
        Address,
        Vec<Address>,
    ) {
        let env = Env::default();
        env.mock_all_auths();

        // Set a realistic ledger timestamp so cooldown checks
        // behave correctly (default zero timestamp would cause
        // false-positive rate limits on second calls).
        env.ledger().with_mut(|li| {
            li.timestamp = 1_000_000;
        });

        let admin = Address::generate(&env);
        let approver1 = Address::generate(&env);
        let approver2 = Address::generate(&env);
        let approver3 = Address::generate(&env);
        let contract_id = env.register_contract(None, EmergencyAccessOverride);
        let client = EmergencyAccessOverrideClient::new(&env, &contract_id);

        let mut approvers = Vec::new(&env);
        approvers.push_back(approver1.clone());
        approvers.push_back(approver2.clone());
        approvers.push_back(approver3.clone());

        (
            env, client, admin, approver1, approver2, approver3, approvers,
        )
    }

    #[test]
    fn test_initialize() {
        let (_env, client, admin, _, _, _, approvers) = setup();
        client.initialize(&admin, &approvers, &2);
    }

    #[test]
    fn test_initialize_threshold_invalid() {
        let (_env, client, admin, _, _, _, approvers) = setup();
        let result = client.try_initialize(&admin, &approvers, &0);
        assert_eq!(result, Err(Ok(Error::InvalidThreshold)));
    }

    #[test]
    fn test_grant_emergency_access_minimum_approvals() {
        let (env, client, admin, approver1, approver2, _approver3, approvers) = setup();
        client.initialize(&admin, &approvers, &2);

        let patient = Address::generate(&env);
        let provider = Address::generate(&env);

        let first = client.grant_emergency_access(&approver1, &patient, &provider, &600);
        assert!(!first);

        let second = client.grant_emergency_access(&approver2, &patient, &provider, &600);
        assert!(second);

        assert!(client.check_emergency_access(&patient, &provider));
    }

    #[test]
    fn test_duplicate_approval_no_effect() {
        let (env, client, admin, approver1, _approver2, _approver3, approvers) = setup();
        client.initialize(&admin, &approvers, &2);

        let patient = Address::generate(&env);
        let provider = Address::generate(&env);

        let first = client.grant_emergency_access(&approver1, &patient, &provider, &600);
        assert!(!first);

        // Advance past the cooldown period so the same approver can call again
        env.ledger().with_mut(|li| {
            li.timestamp = li.timestamp.saturating_add(86_401);
        });

        let second = client.grant_emergency_access(&approver1, &patient, &provider, &600);
        assert!(!second);

        assert!(!client.check_emergency_access(&patient, &provider));
    }

    #[test]
    fn test_check_access_expiry() {
        let (env, client, admin, approver1, approver2, _approver3, approvers) = setup();
        client.initialize(&admin, &approvers, &2);

        let patient = Address::generate(&env);
        let provider = Address::generate(&env);

        client.grant_emergency_access(&approver1, &patient, &provider, &1);
        client.grant_emergency_access(&approver2, &patient, &provider, &1);

        assert!(client.check_emergency_access(&patient, &provider));

        let record = client
            .get_emergency_access_record(&patient, &provider)
            .unwrap();
        assert!(record.expiry_at > record.granted_at);
    }

    #[test]
    fn test_revocation() {
        let (env, client, admin, approver1, approver2, _approver3, approvers) = setup();
        client.initialize(&admin, &approvers, &2);

        let patient = Address::generate(&env);
        let provider = Address::generate(&env);

        client.grant_emergency_access(&approver1, &patient, &provider, &600);
        client.grant_emergency_access(&approver2, &patient, &provider, &600);

        assert!(client.check_emergency_access(&patient, &provider));

        client.revoke_emergency_access(&admin, &patient, &provider);

        assert!(!client.check_emergency_access(&patient, &provider));
    }

    #[test]
    fn test_only_trusted_can_approve() {
        let (env, client, admin, _approver1, _approver2, _approver3, approvers) = setup();
        client.initialize(&admin, &approvers, &2);

        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let outsider = Address::generate(&env);

        let result = client.try_grant_emergency_access(&outsider, &patient, &provider, &600);
        assert_eq!(result, Err(Ok(Error::Unauthorized)));
    }

    #[test]
    fn test_get_access_record() {
        let (env, client, admin, approver1, approver2, _approver3, approvers) = setup();
        client.initialize(&admin, &approvers, &2);

        let patient = Address::generate(&env);
        let provider = Address::generate(&env);

        client.grant_emergency_access(&approver1, &patient, &provider, &600);
        client.grant_emergency_access(&approver2, &patient, &provider, &600);

        let record = client
            .get_emergency_access_record(&patient, &provider)
            .unwrap();
        assert!(record.approved);
        assert_eq!(record.patient, patient);
        assert_eq!(record.provider, provider);
    }

    #[test]
    fn test_error_codes_are_stable() {
        assert_eq!(Error::Unauthorized as u32, 100);
        assert_eq!(Error::NotInitialized as u32, 300);
        assert_eq!(Error::AlreadyInitialized as u32, 301);
        assert_eq!(Error::InvalidThreshold as u32, 230);
        assert_eq!(Error::InvalidDuration as u32, 231);
        assert_eq!(Error::RecordNotFound as u32, 403);
        assert_eq!(Error::RateLimitExceeded as u32, 429);
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
            get_suggestion(Error::RecordNotFound),
            symbol_short!("CHK_ID")
        );
        assert_eq!(
            get_suggestion(Error::InvalidThreshold),
            symbol_short!("CHK_LEN")
        );
    }

    #[test]
    fn test_emergency_access_expiry_and_auto_revoke() {
        let (env, client, admin, approver1, approver2, _approver3, approvers) = setup();
        client.initialize(&admin, &approvers, &2);

        let patient = Address::generate(&env);
        let provider = Address::generate(&env);

        // Grant emergency access with a short 60-second TTL
        client.grant_emergency_access(&approver1, &patient, &provider, &60);
        client.grant_emergency_access(&approver2, &patient, &provider, &60);

        // Immediately after granting, access should be valid
        assert!(client.check_emergency_access(&patient, &provider));

        let record = client
            .get_emergency_access_record(&patient, &provider)
            .unwrap();
        assert!(record.approved);
        assert!(record.expiry_at > record.granted_at);

        // Advance the ledger timestamp past the expiry
        env.ledger().set_timestamp(record.expiry_at + 1);

        // After the TTL has elapsed, access should be auto-revoked (no longer valid)
        assert!(!client.check_emergency_access(&patient, &provider));

        // Verify the record still exists but check_emergency_access returns false
        let record_after = client
            .get_emergency_access_record(&patient, &provider)
            .unwrap();
        assert!(record_after.approved);
        // Expiry timestamp should still be in the past
        assert!(env.ledger().timestamp() > record_after.expiry_at);
    }

    #[test]
    fn test_emergency_access_expiry_event_emitted_on_check() {
        let (env, client, admin, approver1, approver2, _approver3, approvers) = setup();
        client.initialize(&admin, &approvers, &2);

        let patient = Address::generate(&env);
        let provider = Address::generate(&env);

        // Grant access and let it expire
        client.grant_emergency_access(&approver1, &patient, &provider, &10);
        client.grant_emergency_access(&approver2, &patient, &provider, &10);

        let record = client
            .get_emergency_access_record(&patient, &provider)
            .unwrap();

        // Move past expiry
        env.ledger().set_timestamp(record.expiry_at + 1);

        // Verify that checking expired access emits the appropriate events
        assert!(!client.check_emergency_access(&patient, &provider));

        // Verify event was published at check time
        let events = env.events().all();
        let check_events: Vec<_> = events
            .iter()
            .filter(|e| {
                let topics = e.0.clone();
                topics.len() >= 2
                    && topics.get(0).unwrap()
                        == soroban_sdk::Val::from(&soroban_sdk::symbol_short!("EMER"))
                    && topics.get(1).unwrap()
                        == soroban_sdk::Val::from(&soroban_sdk::symbol_short!("CHECK"))
            })
            .collect();

        // There should be at least one EMER/CHECK event emitted
        assert!(!check_events.is_empty(), "Expected EMER/CHECK events to be emitted");

        // The functional test above already verified access is expired:
        // assert!(!client.check_emergency_access(...)) confirms the auto-revoke behavior
        // Event emission is verified by the non-empty check above
    }

    #[test]
    fn test_revoke_emits_audit_event() {
        let (env, client, admin, approver1, approver2, _approver3, approvers) = setup();
        client.initialize(&admin, &approvers, &2);

        let patient = Address::generate(&env);
        let provider = Address::generate(&env);

        client.grant_emergency_access(&approver1, &patient, &provider, &600);
        client.grant_emergency_access(&approver2, &patient, &provider, &600);

        assert!(client.check_emergency_access(&patient, &provider));

        // Perform admin revocation
        client.revoke_emergency_access(&admin, &patient, &provider);

        // Verify revocation event was emitted
        let events = env.events().all();
        let revoke_events: Vec<_> = events
            .iter()
            .filter(|e| {
                let topics = e.0.clone();
                topics.len() >= 2
                    && topics.get(0).unwrap()
                        == soroban_sdk::Val::from(&soroban_sdk::symbol_short!("EMER"))
                    && topics.get(1).unwrap()
                        == soroban_sdk::Val::from(&soroban_sdk::symbol_short!("REVOKE"))
            })
            .collect();

        assert!(
            !revoke_events.is_empty(),
            "Expected EMER/REVOKE event to be emitted on revocation"
        );

        // Access should be revoked after admin action
        assert!(!client.check_emergency_access(&patient, &provider));
    }
}

// ─── Issue #1173: Rate-Limited Emergency Access & Audit Trail Tests ─────────

#[cfg(test)]
mod rate_limit_tests {
    use crate::{
        EmergencyAccessOverride, EmergencyAccessOverrideClient, Error, RateLimitConfig,
    };
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        symbol_short, Address, Env, Vec,
    };

    fn setup() -> (Env, EmergencyAccessOverrideClient<'static>, Address, Vec<Address>) {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|li| {
            li.timestamp = 1_000_000;
        });

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, EmergencyAccessOverride);
        let client = EmergencyAccessOverrideClient::new(&env, &contract_id);

        let a1 = Address::generate(&env);
        let a2 = Address::generate(&env);
        let a3 = Address::generate(&env);
        let mut approvers = Vec::new(&env);
        approvers.push_back(a1.clone());
        approvers.push_back(a2.clone());
        approvers.push_back(a3.clone());

        client.initialize(&admin, &approvers, &2);

        (env, client, admin, approvers)
    }

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.max_requests_per_window, 5);
        assert_eq!(config.window_seconds, 3_600);
        assert_eq!(config.cooldown_seconds, 86_400);
        assert_eq!(config.max_grants_per_window, 10);
    }

    #[test]
    fn test_submit_access_request() {
        let (env, client, _admin, _approvers) = setup();

        let requester = Address::generate(&env);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);

        let request_id = client.submit_emergency_access_request(
            &requester,
            &patient,
            &provider,
            &symbol_short!("CARDIAC"),
            &600,
        );

        assert_eq!(request_id, 1);

        let request = client.get_access_request(&request_id).unwrap();
        assert_eq!(request.request_id, 1);
        assert_eq!(request.patient, patient);
        assert_eq!(request.provider, provider);
        assert!(!request.granted);
    }

    #[test]
    fn test_rate_limit_enforced() {
        let (env, client, _admin, _approvers) = setup();

        let requester = Address::generate(&env);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);

        // Set aggressive rate limit: max 2 per window
        let config = RateLimitConfig {
            max_requests_per_window: 2,
            window_seconds: 3_600,
            cooldown_seconds: 0,
            max_grants_per_window: 10,
        };
        client.update_rate_limit_config(&_admin, &config);

        // Submit 2 requests - should succeed
        client.submit_emergency_access_request(
            &requester,
            &patient,
            &provider,
            &symbol_short!("TEST"),
            &600,
        );
        client.submit_emergency_access_request(
            &requester,
            &Address::generate(&env),
            &Address::generate(&env),
            &symbol_short!("TEST2"),
            &600,
        );

        // 3rd request should fail with rate limit
        let result = client.try_submit_emergency_access_request(
            &requester,
            &Address::generate(&env),
            &Address::generate(&env),
            &symbol_short!("TEST3"),
            &600,
        );
        assert_eq!(result, Err(Ok(Error::RateLimitExceeded)));
    }

    #[test]
    fn test_approve_request_with_audit_trail() {
        let (env, client, _admin, approvers) = setup();

        let requester = Address::generate(&env);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);

        let request_id = client.submit_emergency_access_request(
            &requester,
            &patient,
            &provider,
            &symbol_short!("STROKE"),
            &600,
        );

        let a1 = approvers.get(0).unwrap();
        let a2 = approvers.get(1).unwrap();

        // First approval
        let granted = client.approve_access_request(&a1, &request_id);
        assert!(!granted);

        // Second approval - should grant access
        let granted = client.approve_access_request(&a2, &request_id);
        assert!(granted);

        // Verify audit trail
        let trail = client.get_approval_audit_trail(&request_id);
        assert_eq!(trail.len(), 2);

        let entry1 = trail.get(0).unwrap();
        assert_eq!(entry1.request_id, request_id);
        assert_eq!(entry1.approver, a1);
        assert!(!entry1.final_approval);

        let entry2 = trail.get(1).unwrap();
        assert!(entry2.final_approval);

        // Verify global audit trail count
        assert_eq!(client.get_audit_trail_count(), 2);
    }

    #[test]
    fn test_audit_trail_entry_retrievable() {
        let (env, client, _admin, approvers) = setup();

        let requester = Address::generate(&env);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);

        let request_id = client.submit_emergency_access_request(
            &requester,
            &patient,
            &provider,
            &symbol_short!("BURN"),
            &300,
        );

        let a1 = approvers.get(0).unwrap();
        client.approve_access_request(&a1, &request_id);

        let entry = client.get_audit_trail_entry(&1).unwrap();
        assert_eq!(entry.request_id, request_id);
        assert_eq!(entry.approver, a1);
        assert!(entry.ledger_sequence > 0);
    }

    #[test]
    fn test_request_window_resets_after_window_seconds() {
        let (env, client, _admin, _approvers) = setup();

        let requester = Address::generate(&env);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);

        let config = RateLimitConfig {
            max_requests_per_window: 1,
            window_seconds: 60,
            cooldown_seconds: 0,
            max_grants_per_window: 10,
        };
        client.update_rate_limit_config(&_admin, &config);

        // First request succeeds
        client.submit_emergency_access_request(
            &requester,
            &patient,
            &provider,
            &symbol_short!("T1"),
            &600,
        );

        // Second request fails (rate limited)
        let result = client.try_submit_emergency_access_request(
            &requester,
            &Address::generate(&env),
            &Address::generate(&env),
            &symbol_short!("T2"),
            &600,
        );
        assert_eq!(result, Err(Ok(Error::RateLimitExceeded)));

        // Advance time past window
        env.ledger().with_mut(|li| {
            li.timestamp = li.timestamp.saturating_add(61);
        });

        // Should succeed now
        client.submit_emergency_access_request(
            &requester,
            &Address::generate(&env),
            &Address::generate(&env),
            &symbol_short!("T3"),
            &600,
        );
    }

    #[test]
    fn test_only_trusted_can_approve_rate_limited_request() {
        let (env, client, _admin, _approvers) = setup();

        let requester = Address::generate(&env);
        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let outsider = Address::generate(&env);

        let request_id = client.submit_emergency_access_request(
            &requester,
            &patient,
            &provider,
            &symbol_short!("TEST"),
            &600,
        );

        let result = client.try_approve_access_request(&outsider, &request_id);
        assert_eq!(result, Err(Ok(Error::Unauthorized)));
    }

    #[test]
    fn test_get_rate_limit_config() {
        let (env, client, admin, _approvers) = setup();

        let config = RateLimitConfig {
            max_requests_per_window: 10,
            window_seconds: 7_200,
            cooldown_seconds: 43_200,
            max_grants_per_window: 20,
        };
        client.update_rate_limit_config(&admin, &config);

        let stored = client.get_rate_limit_config();
        assert_eq!(stored.max_requests_per_window, 10);
        assert_eq!(stored.window_seconds, 7_200);
        assert_eq!(stored.cooldown_seconds, 43_200);
        assert_eq!(stored.max_grants_per_window, 20);
    }
}
