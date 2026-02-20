#[cfg(test)]
mod test {
    use crate::{AuditForensicsContract, AuditForensicsContractClient, AuditAction};
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Env, Address, Map, String, BytesN};

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
        metadata.set(String::from_str(&env, "client_ip"), String::from_str(&env, "192.168.1.1"));

        // Log an event
        client.mock_all_auths().log_event(&doctor, &AuditAction::RecordCreated, &Some(record_id), &details_hash, &metadata);

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
        
        client.log_event(&doctor, &AuditAction::RecordAccess, &Some(1), &BytesN::from_array(&env, &[0u8; 32]), &Map::new(&env));
        client.log_event(&doctor, &AuditAction::RecordAccess, &Some(2), &BytesN::from_array(&env, &[0u8; 32]), &Map::new(&env));
        client.log_event(&doctor, &AuditAction::RecordUpdate, &Some(1), &BytesN::from_array(&env, &[0u8; 32]), &Map::new(&env));

        let report = client.generate_compliance_report(&0, &env.ledger().timestamp());
        assert_eq!(report.get(AuditAction::RecordAccess).unwrap(), 2);
        assert_eq!(report.get(AuditAction::RecordUpdate).unwrap(), 1);
    }
}
