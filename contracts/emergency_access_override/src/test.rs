#[cfg(test)]
mod tests {
    use crate::{EmergencyAccessOverride, EmergencyAccessOverrideClient, Error};
    use soroban_sdk::{testutils::Address as _, Address, Env, Vec};

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
}
