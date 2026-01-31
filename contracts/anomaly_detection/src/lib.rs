#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Map,
    Symbol,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    ContractPaused = 2,
    InvalidAnomaly = 3,
    InvalidThreshold = 4,
    DetectorNotWhitelisted = 5,
}

// Storage keys
const ADMIN: Symbol = symbol_short!("ADMIN");
const DETECTOR: Symbol = symbol_short!("DETECTOR");
const THRESHOLD: Symbol = symbol_short!("THRESH"); // bps
const SENSITIVITY: Symbol = symbol_short!("SENSE"); // 1-10
const PAUSED: Symbol = symbol_short!("PAUSED");
const ANOMALIES: Symbol = symbol_short!("ANOMALIES");
const WHITELIST: Symbol = symbol_short!("WHITE");

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnomalyRecord {
    pub id: u64,
    pub detector: Address,
    pub target_resource: BytesN<32>, // Hash of resource ID (e.g. claim ID)
    pub timestamp: u64,
    pub anomaly_score: u32,
    pub metadata_hash: BytesN<32>,
    pub status: AnomalyStatus,
}

#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AnomalyStatus {
    Pending,
    Confirmed,
    FalsePositive,
    Ignored,
}

#[contract]
pub struct AnomalyDetectionContract;

#[contractimpl]
impl AnomalyDetectionContract {
    /// Initialize the contract
    pub fn initialize(
        env: Env,
        admin: Address,
        primary_detector: Address,
        threshold: u32,
        sensitivity: u32,
    ) -> Result<bool, Error> {
        admin.require_auth();

        if env.storage().persistent().has(&ADMIN) {
            return Err(Error::NotAuthorized); // Already initialized
        }

        env.storage().persistent().set(&ADMIN, &admin);
        env.storage().persistent().set(&DETECTOR, &primary_detector);
        env.storage().persistent().set(&THRESHOLD, &threshold);
        env.storage().persistent().set(&SENSITIVITY, &sensitivity);
        env.storage().persistent().set(&PAUSED, &false);

        // Add primary detector to whitelist
        let mut whitelist: Map<Address, bool> = env
            .storage()
            .persistent()
            .get(&WHITELIST)
            .unwrap_or(Map::new(&env));
        whitelist.set(primary_detector, true);
        env.storage().persistent().set(&WHITELIST, &whitelist);

        Ok(true)
    }

    /// Whitelist a detector
    pub fn whitelist_detector(env: Env, admin: Address, detector: Address) -> Result<bool, Error> {
        admin.require_auth();
        let stored_admin: Address = env.storage().persistent().get(&ADMIN).unwrap();
        if admin != stored_admin {
            return Err(Error::NotAuthorized);
        }

        let mut whitelist: Map<Address, bool> = env
            .storage()
            .persistent()
            .get(&WHITELIST)
            .unwrap_or(Map::new(&env));
        whitelist.set(detector, true);
        env.storage().persistent().set(&WHITELIST, &whitelist);

        Ok(true)
    }

    /// Update configuration
    pub fn update_config(
        env: Env,
        admin: Address,
        new_detector: Option<Address>,
        new_threshold: Option<u32>,
        new_sensitivity: Option<u32>,
        enabled: Option<bool>,
    ) -> Result<bool, Error> {
        admin.require_auth();
        let stored_admin: Address = env.storage().persistent().get(&ADMIN).unwrap();
        if admin != stored_admin {
            return Err(Error::NotAuthorized);
        }

        if let Some(det) = new_detector {
            env.storage().persistent().set(&DETECTOR, &det);
            // Also whitelist
            let mut whitelist: Map<Address, bool> = env
                .storage()
                .persistent()
                .get(&WHITELIST)
                .unwrap_or(Map::new(&env));
            whitelist.set(det, true);
            env.storage().persistent().set(&WHITELIST, &whitelist);
        }

        if let Some(thresh) = new_threshold {
            env.storage().persistent().set(&THRESHOLD, &thresh);
        }

        if let Some(sens) = new_sensitivity {
            env.storage().persistent().set(&SENSITIVITY, &sens);
        }

        if let Some(is_active) = enabled {
            env.storage().persistent().set(&PAUSED, &(!is_active));
        }

        Ok(true)
    }

    /// Report an anomaly
    pub fn detect_anomaly(
        env: Env,
        detector: Address,
        target_resource: BytesN<32>,
        anomaly_score: u32,
        metadata_hash: BytesN<32>,
    ) -> Result<u64, Error> {
        detector.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Check whitelist
        let whitelist: Map<Address, bool> = env
            .storage()
            .persistent()
            .get(&WHITELIST)
            .unwrap_or(Map::new(&env));
        if !whitelist.get(detector.clone()).unwrap_or(false) {
            return Err(Error::DetectorNotWhitelisted);
        }

        let threshold: u32 = env.storage().persistent().get(&THRESHOLD).unwrap_or(0);
        if anomaly_score < threshold {
            return Err(Error::InvalidThreshold); // Score too low to be an anomaly
        }

        let id = env.ledger().timestamp(); // Simple ID

        let record = AnomalyRecord {
            id,
            detector,
            target_resource,
            timestamp: env.ledger().timestamp(),
            anomaly_score,
            metadata_hash,
            status: AnomalyStatus::Pending,
        };

        let mut anomalies: Map<u64, AnomalyRecord> = env
            .storage()
            .persistent()
            .get(&ANOMALIES)
            .unwrap_or(Map::new(&env));

        anomalies.set(id, record);
        env.storage().persistent().set(&ANOMALIES, &anomalies);

        Ok(id)
    }

    /// Get anomaly record
    pub fn get_anomaly(env: Env, id: u64) -> Result<AnomalyRecord, Error> {
        let anomalies: Map<u64, AnomalyRecord> = env
            .storage()
            .persistent()
            .get(&ANOMALIES)
            .ok_or(Error::InvalidAnomaly)?;

        anomalies.get(id).ok_or(Error::InvalidAnomaly)
    }

    /// Update anomaly status (e.g. after manual review)
    pub fn update_status(
        env: Env,
        admin: Address,
        id: u64,
        status: AnomalyStatus,
    ) -> Result<bool, Error> {
        admin.require_auth();
        let stored_admin: Address = env.storage().persistent().get(&ADMIN).unwrap();
        if admin != stored_admin {
            return Err(Error::NotAuthorized);
        }

        let mut anomalies: Map<u64, AnomalyRecord> = env
            .storage()
            .persistent()
            .get(&ANOMALIES)
            .ok_or(Error::InvalidAnomaly)?;

        let mut record = anomalies.get(id).ok_or(Error::InvalidAnomaly)?;
        record.status = status;
        anomalies.set(id, record);
        env.storage().persistent().set(&ANOMALIES, &anomalies);

        Ok(true)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{BytesN, Env};

    #[test]
    fn test_anomaly_flow() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, AnomalyDetectionContract);
        let client = AnomalyDetectionContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let detector = Address::generate(&env);

        client.initialize(&admin, &detector, &7000u32, &5u32);

        let resource = BytesN::from_array(&env, &[1u8; 32]);
        let metadata = BytesN::from_array(&env, &[2u8; 32]);

        // FIXED: Removed unwrap()
        let anomaly_id = client.detect_anomaly(
            &detector, &resource, &8500u32, // > 7000
            &metadata,
        );

        let record = client.get_anomaly(&anomaly_id);
        assert_eq!(record.anomaly_score, 8500u32);
        assert_eq!(record.status, AnomalyStatus::Pending);

        // Update status
        client.update_status(&admin, &anomaly_id, &AnomalyStatus::Confirmed);
        let updated = client.get_anomaly(&anomaly_id);
        assert_eq!(updated.status, AnomalyStatus::Confirmed);
    }

    #[test]
    fn test_threshold_check() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, AnomalyDetectionContract);
        let client = AnomalyDetectionContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let detector = Address::generate(&env);

        client.initialize(&admin, &detector, &9000u32, &5u32);

        let resource = BytesN::from_array(&env, &[1u8; 32]);
        let metadata = BytesN::from_array(&env, &[2u8; 32]);

        // Score 8000 < 9000, should fail
        let result = client.try_detect_anomaly(&detector, &resource, &8000u32, &metadata);
        assert_eq!(result, Err(Ok(Error::InvalidThreshold)));
    }

    #[test]
    fn test_config_update() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, AnomalyDetectionContract);
        let client = AnomalyDetectionContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let detector = Address::generate(&env);

        client.initialize(&admin, &detector, &5000u32, &5u32);

        // FIXED: Passed references to Options and removed .is_ok()
        let success = client.update_config(
            &admin,
            &Some(Address::generate(&env)), // new detector
            &Some(8000u32),                 // new threshold
            &Some(7u32),                    // new sensitivity
            &Some(false),                   // disable
        );
        assert!(success);

        // Verify paused
        let resource = BytesN::from_array(&env, &[1u8; 32]);
        let metadata = BytesN::from_array(&env, &[2u8; 32]);
        let result = client.try_detect_anomaly(&detector, &resource, &9000u32, &metadata);
        assert_eq!(result, Err(Ok(Error::ContractPaused)));
    }

    #[test]
    fn test_whitelist() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, AnomalyDetectionContract);
        let client = AnomalyDetectionContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let primary = Address::generate(&env);
        let detector = Address::generate(&env);

        client.initialize(&admin, &primary, &5000u32, &5u32);

        // Detector not whitelisted yet
        let resource = BytesN::from_array(&env, &[1u8; 32]);
        let metadata = BytesN::from_array(&env, &[2u8; 32]);
        let res = client.try_detect_anomaly(&detector, &resource, &9000u32, &metadata);
        assert_eq!(res, Err(Ok(Error::DetectorNotWhitelisted)));

        // Whitelist
        // FIXED: Removed .is_ok()
        assert!(client.whitelist_detector(&admin, &detector));

        // Now should work
        let res2 = client.try_detect_anomaly(&detector, &resource, &9000u32, &metadata);
        assert!(res2.is_ok());
    }
}
