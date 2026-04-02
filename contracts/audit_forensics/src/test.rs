#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use crate::{AuditAction, AuditForensicsContract, AuditForensicsContractClient};
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{vec, Address, BytesN, Env, Map, String};

    #[test]
    fn test_audit_flow() {
        let env = Env::default();
        let contract_id = env.register_contract(None, AuditForensicsContract);
        let client = AuditForensicsContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin);

        let doctor = Address::generate(&env);
        let record_id = 101u64;
        let details_hash = BytesN::from_array(&env, &[1u8; 32]);
        let mut metadata = Map::new(&env);
        metadata.set(
            String::from_str(&env, "client_ip"),
            String::from_str(&env, "192.168.1.1"),
        );

        // Log an event
        client.mock_all_auths().log_event(
            &doctor,
            &AuditAction::RecordCreated,
            &Some(record_id),
            &details_hash,
            &metadata,
        );

        // Analyze timeline
        let timeline = client.analyze_timeline(&record_id);
        assert_eq!(timeline.len(), 1);
        let entry = timeline.get(0).unwrap();
        assert_eq!(entry.actor, doctor);
        assert_eq!(entry.action, AuditAction::RecordCreated);

        // Investigate user
        let user_history = client.investigate_user(&doctor);
        assert_eq!(user_history.len(), 1);
    }

    #[test]
    fn test_compliance_reporting() {
        let env = Env::default();
        let contract_id = env.register_contract(None, AuditForensicsContract);
        let client = AuditForensicsContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin);

        let doctor = Address::generate(&env);
        env.mock_all_auths();

        client.log_event(
            &doctor,
            &AuditAction::RecordAccess,
            &Some(1),
            &BytesN::from_array(&env, &[0u8; 32]),
            &Map::new(&env),
        );
        client.log_event(
            &doctor,
            &AuditAction::RecordAccess,
            &Some(2),
            &BytesN::from_array(&env, &[0u8; 32]),
            &Map::new(&env),
        );
        client.log_event(
            &doctor,
            &AuditAction::RecordUpdate,
            &Some(1),
            &BytesN::from_array(&env, &[0u8; 32]),
            &Map::new(&env),
        );

        let report = client.generate_compliance_report(&0, &env.ledger().timestamp());
        assert_eq!(report.get(AuditAction::RecordAccess).unwrap(), 2);
        assert_eq!(report.get(AuditAction::RecordUpdate).unwrap(), 1);
    }

    #[test]
    fn test_automated_audit_and_formal_verification() {
        let env = Env::default();
        let contract_id = env.register_contract(None, AuditForensicsContract);
        let client = AuditForensicsContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        env.mock_all_auths();
        client.initialize(&admin);

        let reentrancy_rule = client.configure_audit_rule(
            &admin,
            &String::from_str(&env, "Reentrancy Guard Missing"),
            &String::from_str(&env, "soroban-rust"),
            &8500u32,
            &String::from_str(&env, "static/reentrancy"),
            &String::from_str(&env, "Add state-transition guards before external calls"),
        );
        let auth_rule = client.configure_audit_rule(
            &admin,
            &String::from_str(&env, "Auth Bypass"),
            &String::from_str(&env, "soroban-rust"),
            &9500u32,
            &String::from_str(&env, "fuzz/auth-bypass"),
            &String::from_str(&env, "Require explicit auth checks on privileged paths"),
        );

        let execution_id = client.run_automated_audit(
            &admin,
            &BytesN::from_array(&env, &[9u8; 32]),
            &String::from_str(&env, "soroban-rust"),
            &String::from_str(&env, "ml-assisted-static"),
            &vec![&env, reentrancy_rule, auth_rule],
            &9100u32,
        );

        let execution = client.get_execution(&execution_id).unwrap();
        assert_eq!(execution.finding_ids.len(), 2);
        assert!(!execution.passed);
        assert!(execution.duration_minutes < 60);

        let findings = client.get_findings_by_execution(&execution_id);
        assert_eq!(findings.len(), 2);
        assert_eq!(
            findings.get(0).unwrap().analysis_mode,
            String::from_str(&env, "ml-assisted-static")
        );

        let remediation = client.generate_remediation_plan(&execution_id);
        assert_eq!(remediation.len(), 2);

        assert!(client.record_formal_verification(
            &admin,
            &execution_id,
            &String::from_str(&env, "No unauthorized mint"),
            &true,
            &String::from_str(&env, "ipfs://proofs/no-unauthorized-mint"),
        ));
        let formal = client.get_formal_verification(&execution_id).unwrap();
        assert!(formal.proved);
    }
}
