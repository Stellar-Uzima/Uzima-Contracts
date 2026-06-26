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
    fn test_first_call_succeeds_within_cooldown() {
        let (env, client, admin, approver1, _approver2, _approver3, approvers) = setup();
        client.initialize(&admin, &approvers, &2);

        let patient = Address::generate(&env);
        let provider = Address::generate(&env);

        // First call should succeed (no prior cooldown)
        let result = client.try_grant_emergency_access(&approver1, &patient, &provider, &600);
        assert!(result.is_ok());
    }

    #[test]
    fn test_second_call_within_cooldown_fails() {
        let (env, client, admin, approver1, _approver2, _approver3, approvers) = setup();
        client.initialize(&admin, &approvers, &2);

        let patient = Address::generate(&env);
        let provider = Address::generate(&env);

        // First call succeeds
        client.grant_emergency_access(&approver1, &patient, &provider, &600);

        // Second call immediately after should fail with RateLimitExceeded
        let result = client.try_grant_emergency_access(&approver1, &patient, &provider, &600);
        assert_eq!(result, Err(Ok(Error::RateLimitExceeded)));
    }

    #[test]
    fn test_call_after_cooldown_window_succeeds() {
        let (env, client, admin, approver1, _approver2, _approver3, approvers) = setup();
        client.initialize(&admin, &approvers, &2);

        // Set a short cooldown of 100 seconds
        client.update_cooldown_period(&admin, &100u64);

        let patient = Address::generate(&env);
        let provider = Address::generate(&env);

        // First call at t=0
        client.grant_emergency_access(&approver1, &patient, &provider, &600);

        // Advance ledger time past the cooldown window
        env.ledger().with_mut(|li| {
            li.timestamp = li.timestamp.saturating_add(101);
        });

        // Call after cooldown should succeed
        let result = client.try_grant_emergency_access(&approver1, &patient, &provider, &600);
        assert!(result.is_ok());
    }

    #[test]
    fn test_admin_can_update_cooldown_period() {
        let (_env, client, admin, _, _, _, approvers) = setup();
        client.initialize(&admin, &approvers, &2);

        assert_eq!(client.get_cooldown_period(), 86_400u64);

        client.update_cooldown_period(&admin, &3600u64);
        assert_eq!(client.get_cooldown_period(), 3600u64);
    }

    #[test]
    fn test_non_admin_cannot_update_cooldown_period() {
        let (env, client, admin, approver1, _, _, approvers) = setup();
        client.initialize(&admin, &approvers, &2);

        let outsider = Address::generate(&env);
        let result = client.try_update_cooldown_period(&outsider, &3600u64);
        assert_eq!(result, Err(Ok(Error::Unauthorized)));

        // Approver also cannot update
        let result2 = client.try_update_cooldown_period(&approver1, &3600u64);
        assert_eq!(result2, Err(Ok(Error::Unauthorized)));
    }

    // ===== Phase 4 governance_commons migration tests (issue #830) =====
    //
    // These tests document that the contract now delegates multi-sig logic to
    // governance_commons::multi_sig helpers (validate_approval_set,
    // validate_approver, is_already_approved, add_approval, check_approval_status).

    /// `initialize` should reject `threshold > approvers.len()` via
    /// `governance_commons::multi_sig::validate_approval_set`.
    #[test]
    fn test_initialize_threshold_exceeds_member_count_via_validate_approval_set() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|li| {
            li.timestamp = 1_000_000;
        });

        let admin = Address::generate(&env);
        let a1 = Address::generate(&env);
        let a2 = Address::generate(&env);
        let contract_id = env.register_contract(None, EmergencyAccessOverride);
        let client = EmergencyAccessOverrideClient::new(&env, &contract_id);
        let mut approvers = Vec::new(&env);
        approvers.push_back(a1);
        approvers.push_back(a2);

        // Threshold of 3 with only 2 approvers must be rejected,
        // proving the shared validate_approval_set helper is invoked.
        let result = client.try_initialize(&admin, &approvers, &3);
        assert_eq!(result, Err(Ok(Error::InvalidThreshold)));
    }

    /// `grant_emergency_access` should reject callers not in the trusted
    /// approver set via `governance_commons::multi_sig::validate_approver`.
    /// A non-trusted caller attempting to act as approver must be rejected with
    /// `Error::Unauthorized` (mapped from `GovernanceError::NotApprover`).
    #[test]
    fn test_grant_with_non_member_rejected_by_validate_approver() {
        let (env, client, admin, _approver1, _approver2, _approver3, approvers) = setup();
        client.initialize(&admin, &approvers, &2);

        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let non_member = Address::generate(&env);

        let result = client.try_grant_emergency_access(&non_member, &patient, &provider, &600);
        assert_eq!(result, Err(Ok(Error::Unauthorized)));
    }

    /// After Phase 4 migration the approver set lives in instance storage
    /// under `DataKey::TrustedApprovers`. Verifies all three configured
    /// approvers can be accepted (none raise `NotApprover`) and that each
    /// distinct approver is registered against the request by inspecting the
    /// stored record's `approvers` Vec length.
    #[test]
    fn test_all_trusted_approvers_accepted_after_migration() {
        let (env, client, admin, approver1, approver2, approver3, approvers) = setup();
        client.initialize(&admin, &approvers, &2);

        let patient = Address::generate(&env);
        let provider = Address::generate(&env);

        // Each approver should be accepted with no Unauthorized/NotApprover error.
        assert!(client
            .try_grant_emergency_access(&approver1, &patient, &provider, &600)
            .is_ok());
        env.ledger().with_mut(|li| {
            li.timestamp = li.timestamp.saturating_add(86_401);
        });
        assert!(client
            .try_grant_emergency_access(&approver2, &patient, &provider, &600)
            .is_ok());
        env.ledger().with_mut(|li| {
            li.timestamp = li.timestamp.saturating_add(86_401);
        });
        assert!(client
            .try_grant_emergency_access(&approver3, &patient, &provider, &600)
            .is_ok());

        let record = client
            .get_emergency_access_record(&patient, &provider)
            .unwrap();
        assert!(record.approved);
        assert_eq!(record.approvers.len(), 3);
    }
}
