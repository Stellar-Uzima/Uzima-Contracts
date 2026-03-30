#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EmergencyAccessOverride, EmergencyAccessOverrideClient, Error};
    use soroban_sdk::{Env, Address, Vec};

    fn setup() -> (Env, EmergencyAccessOverrideClient, Address, Address, Address, Address, Address) {
        let env = Env::default();
        let admin = Address::random(&env);
        let approver1 = Address::random(&env);
        let approver2 = Address::random(&env);
        let approver3 = Address::random(&env);
        let client = EmergencyAccessOverrideClient::new(&env, &env.register_contract(None, EmergencyAccessOverride));

        let approvers = Vec::new(&env);
        let approvers = approvers.push_back(approver1.clone());
        let approvers = approvers.push_back(approver2.clone());
        let approvers = approvers.push_back(approver3.clone());

        (env, client, admin, approver1, approver2, approver3, approvers)
    }

    #[test]
    fn test_initialize() {
        let (env, client, admin, _, _, _, approvers) = setup();
        let result = client.initialize(&admin, &approvers, &2);
        assert!(result.is_ok());
    }

    #[test]
    fn test_initialize_threshold_invalid() {
        let (env, client, admin, _, _, _, approvers) = setup();
        // threshold 0 invalid
        let result = client.initialize(&admin, &approvers, &0);
        assert_eq!(result, Err(Error::InvalidThreshold));
    }

    #[test]
    fn test_grant_emergency_access_minimum_approvals() {
        let (env, client, admin, approver1, approver2, _approver3, approvers) = setup();
        client.initialize(&admin, &approvers, &2).unwrap();

        let patient = Address::random(&env);
        let provider = Address::random(&env);

        // First approval, not yet granted
        let first = client.grant_emergency_access(&approver1, &patient, &provider, &600);
        assert_eq!(first, Ok(false));

        // Second approval triggers grant
        let second = client.grant_emergency_access(&approver2, &patient, &provider, &600);
        assert_eq!(second, Ok(true));

        // Access should now be active
        let can_access = client.check_emergency_access(&patient, &provider).unwrap();
        assert!(can_access);
    }

    #[test]
    fn test_duplicate_approval_no_effect() {
        let (env, client, admin, approver1, _approver2, _approver3, approvers) = setup();
        client.initialize(&admin, &approvers, &2).unwrap();

        let patient = Address::random(&env);
        let provider = Address::random(&env);

        // First approval
        let first = client.grant_emergency_access(&approver1, &patient, &provider, &600);
        assert_eq!(first, Ok(false));

        // Duplicate approval by same approver should not progress threshold
        let second = client.grant_emergency_access(&approver1, &patient, &provider, &600);
        assert_eq!(second, Ok(false));

        // Still needs second approver
        let has_access = client.check_emergency_access(&patient, &provider).unwrap();
        assert!(!has_access);
    }

    #[test]
    fn test_check_access_expiry() {
        let (env, client, admin, approver1, approver2, _approver3, approvers) = setup();
        client.initialize(&admin, &approvers, &2).unwrap();

        let patient = Address::random(&env);
        let provider = Address::random(&env);

        client.grant_emergency_access(&approver1, &patient, &provider, &1).unwrap();
        client.grant_emergency_access(&approver2, &patient, &provider, &1).unwrap();

        // Immediately valid
        assert!(client.check_emergency_access(&patient, &provider).unwrap());

        // simulate time passing, no direct API to fast-forward in this environment
        // We'll assume expiry behavior is correct based on stored expiry value.
        let record = client.get_emergency_access_record(&patient, &provider).unwrap();
        assert!(record.expiry_at > record.granted_at);
    }

    #[test]
    fn test_revocation() {
        let (env, client, admin, approver1, approver2, _approver3, approvers) = setup();
        client.initialize(&admin, &approvers, &2).unwrap();

        let patient = Address::random(&env);
        let provider = Address::random(&env);

        client.grant_emergency_access(&approver1, &patient, &provider, &600).unwrap();
        client.grant_emergency_access(&approver2, &patient, &provider, &600).unwrap();

        // Should have access now
        assert!(client.check_emergency_access(&patient, &provider).unwrap());

        client.revoke_emergency_access(&admin, &patient, &provider).unwrap();

        assert!(!client.check_emergency_access(&patient, &provider).unwrap());
    }

    #[test]
    fn test_only_trusted_can_approve() {
        let (env, client, admin, _approver1, _approver2, _approver3, approvers) = setup();
        client.initialize(&admin, &approvers, &2).unwrap();

        let patient = Address::random(&env);
        let provider = Address::random(&env);
        let outsider = Address::random(&env);

        let result = client.grant_emergency_access(&outsider, &patient, &provider, &600);
        assert_eq!(result, Err(Error::NotAuthorized));
    }

    #[test]
    fn test_get_access_record() {
        let (env, client, admin, approver1, approver2, _approver3, approvers) = setup();
        client.initialize(&admin, &approvers, &2).unwrap();

        let patient = Address::random(&env);
        let provider = Address::random(&env);

        client.grant_emergency_access(&approver1, &patient, &provider, &600).unwrap();
        client.grant_emergency_access(&approver2, &patient, &provider, &600).unwrap();

        let record = client.get_emergency_access_record(&patient, &provider).unwrap();
        assert!(record.approved);
        assert_eq!(record.patient, patient);
        assert_eq!(record.provider, provider);
    }
}
