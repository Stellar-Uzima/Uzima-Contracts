#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MedicalRecordHashRegistry, MedicalRecordHashRegistryClient, Error};
    use soroban_sdk::{Env, Address, BytesN};

    fn setup() -> (Env, MedicalRecordHashRegistryClient, Address) {
        let env = Env::default();
        let admin = Address::random(&env);
        let client = MedicalRecordHashRegistryClient::new(&env, &env.register_contract(None, MedicalRecordHashRegistry));
        (env, client, admin)
    }

    #[test]
    fn test_initialize() {
        let (env, client, admin) = setup();
        let result = client.initialize(&admin);
        assert!(result.is_ok());
    }

    #[test]
    fn test_initialize_twice_fails() {
        let (env, client, admin) = setup();
        client.initialize(&admin).unwrap();
        let result = client.initialize(&admin);
        assert_eq!(result, Err(Error::AlreadyInitialized));
    }

    #[test]
    fn test_store_record() {
        let (env, client, admin) = setup();
        client.initialize(&admin).unwrap();

        let patient_id = Address::random(&env);
        let record_hash: BytesN<32> = BytesN::from_array(&env, &[1u8; 32]);

        let result = client.store_record(&admin, &patient_id, &record_hash);
        assert!(result.is_ok());
    }

    #[test]
    fn test_duplicate_record_rejected() {
        let (env, client, admin) = setup();
        client.initialize(&admin).unwrap();

        let patient_id = Address::random(&env);
        let record_hash: BytesN<32> = BytesN::from_array(&env, &[1u8; 32]);

        // Store first record
        client.store_record(&admin, &patient_id, &record_hash).unwrap();

        // Try to store duplicate
        let result = client.store_record(&admin, &patient_id, &record_hash);
        assert_eq!(result, Err(Error::DuplicateRecord));
    }

    #[test]
    fn test_verify_record() {
        let (env, client, admin) = setup();
        client.initialize(&admin).unwrap();

        let patient_id = Address::random(&env);
        let record_hash: BytesN<32> = BytesN::from_array(&env, &[1u8; 32]);

        // Store record
        client.store_record(&admin, &patient_id, &record_hash).unwrap();

        // Verify record
        let result = client.verify_record(&patient_id, &record_hash);
        assert_eq!(result, Ok(true));
    }

    #[test]
    fn test_verify_nonexistent_record() {
        let (env, client, admin) = setup();
        client.initialize(&admin).unwrap();

        let patient_id = Address::random(&env);
        let record_hash: BytesN<32> = BytesN::from_array(&env, &[1u8; 32]);

        // Try to verify non-existent record
        let result = client.verify_record(&patient_id, &record_hash);
        assert_eq!(result, Ok(false));
    }

    #[test]
    fn test_multiple_records_same_patient() {
        let (env, client, admin) = setup();
        client.initialize(&admin).unwrap();

        let patient_id = Address::random(&env);
        let hash1: BytesN<32> = BytesN::from_array(&env, &[1u8; 32]);
        let hash2: BytesN<32> = BytesN::from_array(&env, &[2u8; 32]);

        // Store two different records for same patient
        client.store_record(&admin, &patient_id, &hash1).unwrap();
        client.store_record(&admin, &patient_id, &hash2).unwrap();

        // Verify both
        assert_eq!(client.verify_record(&patient_id, &hash1), Ok(true));
        assert_eq!(client.verify_record(&patient_id, &hash2), Ok(true));

        // Check record count
        let count = client.get_record_count(&patient_id);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_get_patient_by_hash() {
        let (env, client, admin) = setup();
        client.initialize(&admin).unwrap();

        let patient_id = Address::random(&env);
        let record_hash: BytesN<32> = BytesN::from_array(&env, &[1u8; 32]);

        // Store record
        client.store_record(&admin, &patient_id, &record_hash).unwrap();

        // Get patient by hash
        let result = client.get_patient_by_hash(&record_hash);
        assert_eq!(result, Some(patient_id));
    }

    #[test]
    fn test_get_patient_records() {
        let (env, client, admin) = setup();
        client.initialize(&admin).unwrap();

        let patient_id = Address::random(&env);
        let hash1: BytesN<32> = BytesN::from_array(&env, &[1u8; 32]);
        let hash2: BytesN<32> = BytesN::from_array(&env, &[2u8; 32]);

        client.store_record(&admin, &patient_id, &hash1).unwrap();
        client.store_record(&admin, &patient_id, &hash2).unwrap();

        let records = client.get_patient_records(&patient_id);
        assert!(records.is_some());
        assert_eq!(records.unwrap().record_count, 2);
    }

    #[test]
    fn test_immutability() {
        let (env, client, admin) = setup();
        client.initialize(&admin).unwrap();

        let patient_id = Address::random(&env);
        let record_hash: BytesN<32> = BytesN::from_array(&env, &[1u8; 32]);

        // Store record
        client.store_record(&admin, &patient_id, &record_hash).unwrap();

        // Verify it's immutable (can't store duplicate)
        let result = client.store_record(&admin, &patient_id, &record_hash);
        assert_eq!(result, Err(Error::DuplicateRecord));
    }
}
