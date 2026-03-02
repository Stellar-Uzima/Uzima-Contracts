/// Test data generators for various contract scenarios
use soroban_sdk::{Address, Env, String as SorobanString, Vec};
use std::time::{SystemTime, UNIX_EPOCH};

#[allow(clippy::unwrap_used)]

/// Medical record data generator
pub struct MedicalRecordGenerator {
    counter: usize,
}

impl MedicalRecordGenerator {
    pub fn new() -> Self {
        Self { counter: 0 }
    }

    /// Generate synthetic patient ID
    pub fn generate_patient_id(&mut self) -> u64 {
        self.counter += 1;
        (self.counter as u64) * 1000 + 1
    }

    /// Generate synthetic record ID
    pub fn generate_record_id(&mut self) -> u64 {
        self.counter += 1;
        (self.counter as u64) * 100 + 2
    }

    /// Generate medical record metadata
    pub fn generate_record_metadata(env: &Env, record_id: u64) -> RecordMetadata {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        RecordMetadata {
            record_id,
            created_at: timestamp,
            updated_at: timestamp,
            version: 1,
            status: SorobanString::from_str(env, "active"),
        }
    }

    /// Generate medical data entries
    pub fn generate_medical_entries(env: &Env, count: usize) -> Vec<MedicalEntry> {
        let mut entries = Vec::new();
        let diagnoses = vec!["Diabetes", "Hypertension", "Asthma", "Migraine", "GERD"];
        let medications = vec!["Metformin", "Lisinopril", "Albuterol", "Sumatriptan", "Omeprazole"];

        for i in 0..count {
            entries.push(MedicalEntry {
                entry_type: SorobanString::from_str(
                    env,
                    if i % 2 == 0 { "diagnosis" } else { "medication" },
                ),
                description: SorobanString::from_str(
                    env,
                    if i % 2 == 0 {
                        diagnoses[i % diagnoses.len()]
                    } else {
                        medications[i % medications.len()]
                    },
                ),
                date: (SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    - (i as u64 * 86400)) as u32,
            });
        }
        entries
    }
}

/// Medical record metadata structure
#[derive(Clone)]
pub struct RecordMetadata {
    pub record_id: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub version: u32,
    pub status: SorobanString,
}

/// Medical entry structure
#[derive(Clone)]
pub struct MedicalEntry {
    pub entry_type: SorobanString,
    pub description: SorobanString,
    pub date: u32,
}

/// Consent data generator
pub struct ConsentDataGenerator {
    counter: usize,
}

impl ConsentDataGenerator {
    pub fn new() -> Self {
        Self { counter: 0 }
    }

    /// Generate consent grant
    pub fn generate_consent_grant(env: &Env, from: &Address, to: &Address) -> ConsentGrant {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        ConsentGrant {
            grantor: from.clone(),
            grantee: to.clone(),
            granted_at: timestamp,
            expires_at: timestamp + (30 * 24 * 60 * 60), // 30 days
            permissions: SorobanString::from_str(env, "read,share"),
        }
    }

    /// Generate multiple consent grants
    pub fn generate_consent_grants(
        env: &Env,
        grantor: &Address,
        grantees: &[Address],
    ) -> Vec<ConsentGrant> {
        grantees
            .iter()
            .map(|grantee| Self::generate_consent_grant(env, grantor, grantee))
            .collect()
    }
}

/// Consent grant structure
#[derive(Clone)]
pub struct ConsentGrant {
    pub grantor: Address,
    pub grantee: Address,
    pub granted_at: u64,
    pub expires_at: u64,
    pub permissions: SorobanString,
}

/// Transaction data generator
pub struct TransactionDataGenerator;

impl TransactionDataGenerator {
    /// Generate transaction ID
    pub fn generate_tx_id(env: &Env) -> SorobanString {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        SorobanString::from_str(env, &format!("TX_{}", timestamp))
    }

    /// Generate transaction record
    pub fn generate_transaction(env: &Env, from: &Address, to: &Address, amount: u128) -> Transaction {
        Transaction {
            tx_id: Self::generate_tx_id(env),
            from: from.clone(),
            to: to.clone(),
            amount,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            status: SorobanString::from_str(env, "completed"),
        }
    }
}

/// Transaction structure
#[derive(Clone)]
pub struct Transaction {
    pub tx_id: SorobanString,
    pub from: Address,
    pub to: Address,
    pub amount: u128,
    pub timestamp: u64,
    pub status: SorobanString,
}

/// Access log generator
pub struct AccessLogGenerator {
    counter: usize,
}

impl AccessLogGenerator {
    pub fn new() -> Self {
        Self { counter: 0 }
    }

    /// Generate access log entry
    pub fn generate_access_log(env: &Env, accessor: &Address, resource_id: u64) -> AccessLog {
        self.counter += 1;
        AccessLog {
            log_id: self.counter as u64,
            accessor: accessor.clone(),
            resource_id,
            action: SorobanString::from_str(env, "read"),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Generate multiple access logs
    pub fn generate_access_logs(
        &mut self,
        env: &Env,
        accessor: &Address,
        resource_ids: &[u64],
    ) -> Vec<AccessLog> {
        resource_ids
            .iter()
            .map(|&id| self.generate_access_log(env, accessor, id))
            .collect()
    }
}

/// Access log structure
#[derive(Clone)]
pub struct AccessLog {
    pub log_id: u64,
    pub accessor: Address,
    pub resource_id: u64,
    pub action: SorobanString,
    pub timestamp: u64,
}

/// Property-based test data generator
pub struct PropertyTestDataGenerator;

impl PropertyTestDataGenerator {
    /// Generate edge case amounts
    pub fn generate_edge_case_amounts() -> Vec<u128> {
        vec![
            0,                                   // Zero
            1,                                   // Minimum
            u128::MAX,                           // Maximum
            u128::MAX / 2,                       // Half max
            1_000_000_000_000_000_000,          // Large amount
        ]
    }

    /// Generate various timestamp values
    pub fn generate_timestamps() -> Vec<u64> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        vec![
            0,              // Unix epoch
            now,            // Current time
            now + 86400,    // Tomorrow
            now - 86400,    // Yesterday
            u64::MAX / 2,   // Far future
        ]
    }

    /// Generate boundary test values
    pub fn generate_boundary_values(min: u32, max: u32) -> Vec<u32> {
        vec![
            min,
            max,
            min + 1,
            max - 1,
            (min + max) / 2,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_medical_record_generator() {
        let mut gen = MedicalRecordGenerator::new();
        let id1 = gen.generate_record_id();
        let id2 = gen.generate_record_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_generate_medical_entries() {
        let env = Env::default();
        let entries = MedicalRecordGenerator::generate_medical_entries(&env, 5);
        assert_eq!(entries.len(), 5);
    }

    #[test]
    fn test_consent_data_generator() {
        let env = Env::default();
        let addr1 = Address::generate(&env);
        let addr2 = Address::generate(&env);
        let grant = ConsentDataGenerator::generate_consent_grant(&env, &addr1, &addr2);
        assert_eq!(grant.grantor, addr1);
        assert_eq!(grant.grantee, addr2);
    }

    #[test]
    fn test_transaction_generator() {
        let env = Env::default();
        let addr1 = Address::generate(&env);
        let addr2 = Address::generate(&env);
        let tx = TransactionDataGenerator::generate_transaction(&env, &addr1, &addr2, 1000);
        assert_eq!(tx.amount, 1000);
    }

    #[test]
    fn test_access_log_generator() {
        let env = Env::default();
        let mut gen = AccessLogGenerator::new();
        let addr = Address::generate(&env);
        let log = gen.generate_access_log(&env, &addr, 123);
        assert_eq!(log.resource_id, 123);
    }

    #[test]
    fn test_property_test_edge_cases() {
        let amounts = PropertyTestDataGenerator::generate_edge_case_amounts();
        assert!(amounts.len() >= 3);
        assert!(amounts.contains(&0));
        assert!(amounts.contains(&1));
    }

    #[test]
    fn test_property_test_timestamps() {
        let timestamps = PropertyTestDataGenerator::generate_timestamps();
        assert!(timestamps.len() >= 3);
    }
}
