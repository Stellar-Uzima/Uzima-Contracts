#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec, map, Map};

// ============================================================================
// Data Types & Constants
// ============================================================================

const ROLE_ADMIN: u32 = 1;
const ROLE_OPERATOR: u32 = 2;
const ROLE_AUDITOR: u32 = 4;
const ALL_ROLES: u32 = 7;

const MAX_RETRIES: u32 = 3;
const SYNC_TIMEOUT_MS: u64 = 60000; // 60 seconds

#[derive(Clone)]
#[contracttype]
pub enum SyncStatus {
    Pending = 0,
    InProgress = 1,
    Completed = 2,
    Failed = 3,
    PartialSuccess = 4,
}

#[derive(Clone)]
#[contracttype]
pub enum ConsistencyLevel {
    Eventual = 0,
    Strong = 1,
    Causal = 2,
}

#[derive(Clone)]
#[contracttype]
pub struct SyncOperation {
    pub operation_id: u64,
    pub source_region_id: u32,
    pub target_region_ids: Vec<u32>,
    pub data_hash: u64,
    pub initiated_at: u64,
    pub completed_at: u64,
    pub status: SyncStatus,
    pub consistency_level: ConsistencyLevel,
    pub retry_count: u32,
    pub success_count: u32,
    pub failure_count: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct SyncWindow {
    pub window_id: u64,
    pub region_id: u32,
    pub start_ts: u64,
    pub end_ts: u64,
    pub data_version: u64,
    pub checksum: u64,
    pub is_applied: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct ReplicationLag {
    pub lag_id: u64,
    pub source_region_id: u32,
    pub target_region_id: u32,
    pub lag_ms: u64,
    pub measured_at: u64,
    pub acceptable: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct SyncPolicy {
    pub sync_interval_ms: u64,
    pub max_lag_ms: u64,
    pub consistency_mode: ConsistencyLevel,
    pub max_retries: u32,
    pub auto_sync_enabled: bool,
    pub conflict_resolution_strategy: u32, // 0=accept-all, 1=reject-all, 2=custom
}

#[derive(Clone)]
#[contracttype]
pub struct ConflictResolution {
    pub conflict_id: u64,
    pub operation_id: u64,
    pub source_region_id: u32,
    pub conflicting_regions: Vec<u32>,
    pub detected_at: u64,
    pub resolved: bool,
    pub resolution_strategy: u32,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    InvalidInput = 4,
    SyncOperationNotFound = 5,
    SyncFailed = 6,
    ConflictDetected = 7,
    MaxRetriesExceeded = 8,
    InconsistentState = 9,
    TargetUnavailable = 10,
}

// ============================================================================
// Storage Keys
// ============================================================================

const ADMIN: Symbol = symbol_short!("ADMIN");
const INITIALIZED: Symbol = symbol_short!("INIT");
const ROLES: Symbol = symbol_short!("ROLES");
const OPERATIONS: Symbol = symbol_short!("OPS");
const WINDOWS: Symbol = symbol_short!("WINS");
const LAGS: Symbol = symbol_short!("LAGS");
const CONFLICTS: Symbol = symbol_short!("CONF");
const SYNC_POLICY: Symbol = symbol_short!("SPOL");
const NEXT_OPERATION_ID: Symbol = symbol_short!("NOID");
const NEXT_WINDOW_ID: Symbol = symbol_short!("NWID");
const NEXT_LAG_ID: Symbol = symbol_short!("NLID");
const NEXT_CONFLICT_ID: Symbol = symbol_short!("NCID");

// ============================================================================
// Contract Implementation
// ============================================================================

#[contract]
pub struct SyncManager;

#[contractimpl]
impl SyncManager {
    // ========================================================================
    // Initialization & Admin
    // ========================================================================

    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&INITIALIZED) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&INITIALIZED, &true);
        env.storage().instance().set(&NEXT_OPERATION_ID, &1u64);
        env.storage().instance().set(&NEXT_WINDOW_ID, &1u64);
        env.storage().instance().set(&NEXT_LAG_ID, &1u64);
        env.storage().instance().set(&NEXT_CONFLICT_ID, &1u64);

        // Default policy
        let default_policy = SyncPolicy {
            sync_interval_ms: 60000,
            max_lag_ms: 5000,
            consistency_mode: ConsistencyLevel::Eventual,
            max_retries: MAX_RETRIES,
            auto_sync_enabled: true,
            conflict_resolution_strategy: 0, // accept-all
        };
        env.storage().instance().set(&SYNC_POLICY, &default_policy);

        env.events().publish((symbol_short!("SM_INIT"),), admin);
        Ok(())
    }

    pub fn assign_role(env: Env, caller: Address, user: Address, role_mask: u32) -> Result<(), Error> {
        Self::require_admin(&env, &caller)?;
        if role_mask > ALL_ROLES {
            return Err(Error::InvalidInput);
        }

        let mut roles: Map<Address, u32> = env
            .storage()
            .instance()
            .get(&ROLES)
            .unwrap_or_else(|| map![(&env, (user.clone(), 0))]);
        roles.set(user, role_mask);
        env.storage().instance().set(&ROLES, &roles);
        Ok(())
    }

    // ========================================================================
    // Synchronization Operations
    // ========================================================================

    pub fn initiate_sync(
        env: Env,
        caller: Address,
        source_region_id: u32,
        target_region_ids: Vec<u32>,
        data_hash: u64,
        consistency_level: ConsistencyLevel,
    ) -> Result<u64, Error> {
        Self::require_operator(&env, &caller)?;

        if target_region_ids.len() == 0 {
            return Err(Error::InvalidInput);
        }

        let operation_id: u64 = env.storage().instance().get(&NEXT_OPERATION_ID).unwrap();
        let current_time = env.ledger().timestamp();

        let operation = SyncOperation {
            operation_id,
            source_region_id,
            target_region_ids,
            data_hash,
            initiated_at: current_time,
            completed_at: 0,
            status: SyncStatus::Pending,
            consistency_level,
            retry_count: 0,
            success_count: 0,
            failure_count: 0,
        };

        let mut operations: Vec<SyncOperation> = env
            .storage()
            .persistent()
            .get(&OPERATIONS)
            .unwrap_or_else(|| Vec::new(&env));
        operations.push_back(operation);
        env.storage().persistent().set(&OPERATIONS, &operations);
        env.storage()
            .instance()
            .set(&NEXT_OPERATION_ID, &(operation_id + 1));

        env.events().publish((symbol_short!("SM_INIT_S"),), operation_id);
        Ok(operation_id)
    }

    pub fn execute_sync(
        env: Env,
        caller: Address,
        operation_id: u64,
    ) -> Result<bool, Error> {
        Self::require_operator(&env, &caller)?;

        let mut operations: Vec<SyncOperation> = env
            .storage()
            .persistent()
            .get(&OPERATIONS)
            .unwrap_or_else(|| Vec::new(&env));

        let mut found_index: Option<usize> = None;
        for i in 0..operations.len() {
            if operations.get_unchecked(i).operation_id == operation_id {
                found_index = Some(i);
                break;
            }
        }

        let idx = found_index.ok_or(Error::SyncOperationNotFound)?;
        let mut operation = operations.get_unchecked(idx).clone();

        operation.status = SyncStatus::InProgress;
        let targets = operation.target_region_ids.len();

        // Simulate sync execution
        let current_time = env.ledger().timestamp();
        operation.success_count = targets as u32;
        operation.failure_count = 0;
        operation.completed_at = current_time;
        operation.status = SyncStatus::Completed;

        operations.set(idx, operation.clone());
        env.storage().persistent().set(&OPERATIONS, &operations);

        env.events().publish((symbol_short!("SM_EXEC"),), operation_id);
        Ok(true)
    }

    pub fn retry_sync(
        env: Env,
        caller: Address,
        operation_id: u64,
    ) -> Result<bool, Error> {
        Self::require_operator(&env, &caller)?;

        let mut operations: Vec<SyncOperation> = env
            .storage()
            .persistent()
            .get(&OPERATIONS)
            .unwrap_or_else(|| Vec::new(&env));

        let mut found_index: Option<usize> = None;
        for i in 0..operations.len() {
            if operations.get_unchecked(i).operation_id == operation_id {
                found_index = Some(i);
                break;
            }
        }

        let idx = found_index.ok_or(Error::SyncOperationNotFound)?;
        let mut operation = operations.get_unchecked(idx).clone();

        if operation.retry_count >= MAX_RETRIES {
            return Err(Error::MaxRetriesExceeded);
        }

        operation.retry_count += 1;
        operation.status = SyncStatus::InProgress;

        let current_time = env.ledger().timestamp();
        operation.success_count = operation.target_region_ids.len() as u32;
        operation.completed_at = current_time;
        operation.status = SyncStatus::Completed;

        operations.set(idx, operation);
        env.storage().persistent().set(&OPERATIONS, &operations);

        env.events().publish((symbol_short!("SM_RETR"),), operation_id);
        Ok(true)
    }

    pub fn get_sync_operation(env: Env, operation_id: u64) -> Option<SyncOperation> {
        let operations: Vec<SyncOperation> = env
            .storage()
            .persistent()
            .get(&OPERATIONS)
            .unwrap_or_else(|| Vec::new(&env));

        for i in 0..operations.len() {
            if operations.get_unchecked(i).operation_id == operation_id {
                return Some(operations.get_unchecked(i).clone());
            }
        }
        None
    }

    pub fn list_sync_operations(env: Env) -> Vec<SyncOperation> {
        env.storage()
            .persistent()
            .get(&OPERATIONS)
            .unwrap_or_else(|| Vec::new(&env))
    }

    // ========================================================================
    // Replication Lag Monitoring
    // ========================================================================

    pub fn record_replication_lag(
        env: Env,
        caller: Address,
        source_region_id: u32,
        target_region_id: u32,
        lag_ms: u64,
    ) -> Result<u64, Error> {
        Self::require_operator(&env, &caller)?;

        let policy: SyncPolicy = env.storage().instance().get(&SYNC_POLICY).unwrap();
        let acceptable = lag_ms <= policy.max_lag_ms;

        let lag_id: u64 = env.storage().instance().get(&NEXT_LAG_ID).unwrap();

        let lag_record = ReplicationLag {
            lag_id,
            source_region_id,
            target_region_id,
            lag_ms,
            measured_at: env.ledger().timestamp(),
            acceptable,
        };

        let mut lags: Vec<ReplicationLag> = env
            .storage()
            .persistent()
            .get(&LAGS)
            .unwrap_or_else(|| Vec::new(&env));
        lags.push_back(lag_record);
        env.storage().persistent().set(&LAGS, &lags);
        env.storage().instance().set(&NEXT_LAG_ID, &(lag_id + 1));

        env.events().publish((symbol_short!("SM_LAG"),), lag_id);
        Ok(lag_id)
    }

    pub fn get_replication_lags(env: Env) -> Vec<ReplicationLag> {
        env.storage()
            .persistent()
            .get(&LAGS)
            .unwrap_or_else(|| Vec::new(&env))
    }

    pub fn get_region_lag(env: Env, source_region_id: u32, target_region_id: u32) -> Option<ReplicationLag> {
        let lags: Vec<ReplicationLag> = env
            .storage()
            .persistent()
            .get(&LAGS)
            .unwrap_or_else(|| Vec::new(&env));

        if lags.len() == 0 {
            return None;
        }

        for i in (0..lags.len()).rev() {
            let lag = lags.get_unchecked(i);
            if lag.source_region_id == source_region_id && lag.target_region_id == target_region_id {
                return Some(lag.clone());
            }
        }
        None
    }

    // ========================================================================
    // Conflict Detection & Resolution
    // ========================================================================

    pub fn detect_sync_conflict(
        env: Env,
        caller: Address,
        operation_id: u64,
        conflicting_regions: Vec<u32>,
    ) -> Result<u64, Error> {
        Self::require_operator(&env, &caller)?;

        if conflicting_regions.len() == 0 {
            return Err(Error::InvalidInput);
        }

        let operation = Self::get_sync_operation(env.clone(), operation_id)
            .ok_or(Error::SyncOperationNotFound)?;

        let conflict_id: u64 = env.storage().instance().get(&NEXT_CONFLICT_ID).unwrap();

        let conflict = ConflictResolution {
            conflict_id,
            operation_id,
            source_region_id: operation.source_region_id,
            conflicting_regions,
            detected_at: env.ledger().timestamp(),
            resolved: false,
            resolution_strategy: 0,
        };

        let mut conflicts: Vec<ConflictResolution> = env
            .storage()
            .persistent()
            .get(&CONFLICTS)
            .unwrap_or_else(|| Vec::new(&env));
        conflicts.push_back(conflict);
        env.storage().persistent().set(&CONFLICTS, &conflicts);
        env.storage()
            .instance()
            .set(&NEXT_CONFLICT_ID, &(conflict_id + 1));

        env.events().publish((symbol_short!("SM_CONF"),), conflict_id);
        Ok(conflict_id)
    }

    pub fn resolve_conflict(
        env: Env,
        caller: Address,
        conflict_id: u64,
        strategy: u32,
    ) -> Result<(), Error> {
        Self::require_operator(&env, &caller)?;

        let mut conflicts: Vec<ConflictResolution> = env
            .storage()
            .persistent()
            .get(&CONFLICTS)
            .unwrap_or_else(|| Vec::new(&env));

        let mut found = false;
        for i in 0..conflicts.len() {
            let mut conflict = conflicts.get_unchecked(i).clone();
            if conflict.conflict_id == conflict_id {
                conflict.resolved = true;
                conflict.resolution_strategy = strategy;
                conflicts.set(i, conflict);
                found = true;
                break;
            }
        }

        if !found {
            return Err(Error::InconsistentState);
        }

        env.storage().persistent().set(&CONFLICTS, &conflicts);
        env.events().publish((symbol_short!("SM_RESO"),), conflict_id);
        Ok(())
    }

    pub fn get_conflicts(env: Env) -> Vec<ConflictResolution> {
        env.storage()
            .persistent()
            .get(&CONFLICTS)
            .unwrap_or_else(|| Vec::new(&env))
    }

    // ========================================================================
    // Policy Management
    // ========================================================================

    pub fn set_sync_policy(env: Env, caller: Address, policy: SyncPolicy) -> Result<(), Error> {
        Self::require_admin(&env, &caller)?;

        if policy.max_lag_ms == 0 || policy.sync_interval_ms == 0 {
            return Err(Error::InvalidInput);
        }

        env.storage().instance().set(&SYNC_POLICY, &policy);
        env.events().publish((symbol_short!("SM_SETP"),), caller);
        Ok(())
    }

    pub fn get_sync_policy(env: Env) -> SyncPolicy {
        env.storage()
            .instance()
            .get(&SYNC_POLICY)
            .unwrap_or_else(|| SyncPolicy {
                sync_interval_ms: 60000,
                max_lag_ms: 5000,
                consistency_mode: ConsistencyLevel::Eventual,
                max_retries: MAX_RETRIES,
                auto_sync_enabled: true,
                conflict_resolution_strategy: 0,
            })
    }

    // ========================================================================
    // Internal Utilities
    // ========================================================================

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env.storage().instance().get(&ADMIN).ok_or(Error::NotInitialized)?;
        if admin != *caller {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    fn require_operator(env: &Env, caller: &Address) -> Result<(), Error> {
        let roles: Map<Address, u32> = env
            .storage()
            .instance()
            .get(&ROLES)
            .ok_or(Error::NotAuthorized)?;

        let role_mask = roles.get(caller.clone()).ok_or(Error::NotAuthorized)?;
        if (role_mask & (ROLE_ADMIN | ROLE_OPERATOR)) == 0 {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Env};

    #[test]
    fn test_initialize() {
        let env = Env::default();
        let admin = Address::random(&env);

        let result = SyncManager::initialize(env.clone(), admin.clone());
        assert!(result.is_ok());

        let result = SyncManager::initialize(env, admin);
        assert!(matches!(result, Err(Error::AlreadyInitialized)));
    }

    #[test]
    fn test_initiate_sync() {
        let env = Env::default();
        let admin = Address::random(&env);
        let operator = Address::random(&env);

        SyncManager::initialize(env.clone(), admin.clone()).unwrap();
        SyncManager::assign_role(env.clone(), admin, operator.clone(), ROLE_OPERATOR).unwrap();

        let mut targets = Vec::new(&env);
        targets.push_back(2u32);
        targets.push_back(3u32);

        let result = SyncManager::initiate_sync(
            env,
            operator,
            1,
            targets,
            12345u64,
            ConsistencyLevel::Eventual,
        );

        assert!(result.is_ok());
    }
}
