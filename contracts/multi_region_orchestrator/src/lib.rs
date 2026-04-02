#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol, Vec, map, Map};

// ============================================================================
// Data Types & Constants
// ============================================================================

const ROLE_ADMIN: u32 = 1;
const ROLE_OPERATOR: u32 = 2;
const ROLE_AUDITOR: u32 = 4;
const ALL_ROLES: u32 = 7;

const MAX_REGIONS: u32 = 10;
const RTO_THRESHOLD_MS: u64 = 15 * 60 * 1000; // 15 minutes
const UPTIME_TARGET: u32 = 9999; // 99.99% (in basis points)

#[derive(Clone)]
#[contracttype]
pub enum GeoRegion {
    UsEast = 0,
    UsWest = 1,
    EuCentral = 2,
    EuWest = 3,
    ApSouth = 4,
    ApNorth = 5,
    SaEast = 6,
    AfSouth = 7,
    Custom = 8,
}

#[derive(Clone)]
#[contracttype]
pub enum RegionStatus {
    Active = 0,
    Degraded = 1,
    Unavailable = 2,
    RecoveryInProgress = 3,
}

#[derive(Clone)]
#[contracttype]
pub struct RegionNode {
    pub region: GeoRegion,
    pub node_id: u32,
    pub status: RegionStatus,
    pub endpoint_hash: u64,
    pub last_heartbeat: u64,
    pub replica_count: u32,
    pub is_primary: bool,
    pub failure_count: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct FailoverEvent {
    pub event_id: u64,
    pub triggered_at: u64,
    pub source_region: GeoRegion,
    pub target_region: GeoRegion,
    pub reason: Symbol,
    pub rto_ms: u64,
    pub success: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct UptimeMetric {
    pub start_time: u64,
    pub end_time: u64,
    pub uptime_basis_points: u32, // 10000 = 100%
    pub outages: u32,
    pub total_outage_ms: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct DRPolicy {
    pub min_replicas_per_region: u32,
    pub max_regions: u32,
    pub failover_timeout_ms: u64,
    pub sync_interval_ms: u64,
    pub health_check_interval_ms: u64,
    pub auto_failover_enabled: bool,
    pub rto_target_ms: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct SyncOperation {
    pub sync_id: u64,
    pub source_region: GeoRegion,
    pub target_regions: Vec<GeoRegion>,
    pub data_hash: u64,
    pub started_at: u64,
    pub completed_at: u64,
    pub success: bool,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    InvalidInput = 4,
    MaxRegionsExceeded = 5,
    AllRegionsUnavailable = 6,
    FailoverFailed = 7,
    SyncFailed = 8,
    RtoExceeded = 9,
    InsufficientReplicas = 10,
}

// ============================================================================
// Storage Keys
// ============================================================================

const ADMIN: Symbol = symbol_short!("ADMIN");
const INITIALIZED: Symbol = symbol_short!("INIT");
const PAUSED: Symbol = symbol_short!("PAUSE");
const ROLES: Symbol = symbol_short!("ROLES");
const DR_POLICY: Symbol = symbol_short!("DRPOL");
const REGIONS: Symbol = symbol_short!("REGS");
const NEXT_REGION_ID: Symbol = symbol_short!("NRID");
const NEXT_FAILOVER_ID: Symbol = symbol_short!("NFID");
const NEXT_SYNC_ID: Symbol = symbol_short!("NSID");
const FAILOVER_EVENTS: Symbol = symbol_short!("FEVS");
const UPTIME_METRICS: Symbol = symbol_short!("UPTIME");
const SYNC_OPS: Symbol = symbol_short!("SYNCS");
const LAST_HEALTH_CHECK: Symbol = symbol_short!("HLTH");

// ============================================================================
// Contract Implementation
// ============================================================================

#[contract]
pub struct MultiRegionOrchestrator;

#[contractimpl]
impl MultiRegionOrchestrator {
    // ========================================================================
    // Initialization & Admin
    // ========================================================================

    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&INITIALIZED) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&INITIALIZED, &true);
        env.storage().instance().set(&PAUSED, &false);
        env.storage().instance().set(&NEXT_REGION_ID, &1u32);
        env.storage().instance().set(&NEXT_FAILOVER_ID, &1u64);
        env.storage().instance().set(&NEXT_SYNC_ID, &1u64);

        // Initialize default policy
        let default_policy = DRPolicy {
            min_replicas_per_region: 3,
            max_regions: 5,
            failover_timeout_ms: 300000,   // 5 minutes
            sync_interval_ms: 60000,       // 1 minute
            health_check_interval_ms: 30000, // 30 seconds
            auto_failover_enabled: true,
            rto_target_ms: RTO_THRESHOLD_MS,
        };
        env.storage().instance().set(&DR_POLICY, &default_policy);

        env.events().publish((symbol_short!("DRO_INIT"),), admin);
        Ok(())
    }

    pub fn set_paused(env: Env, caller: Address, paused: bool) -> Result<(), Error> {
        Self::require_admin(&env, &caller)?;
        env.storage().instance().set(&PAUSED, &paused);
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
    // Region Management
    // ========================================================================

    pub fn register_region(
        env: Env,
        caller: Address,
        region: GeoRegion,
        node_id: u32,
        endpoint_hash: u64,
        is_primary: bool,
    ) -> Result<u32, Error> {
        Self::check_paused(&env)?;
        Self::require_operator(&env, &caller)?;

        let policy: DRPolicy = env.storage().instance().get(&DR_POLICY).unwrap();
        let mut regions: Vec<RegionNode> = env
            .storage()
            .persistent()
            .get(&REGIONS)
            .unwrap_or_else(|| Vec::new(&env));

        // Check region count
        if regions.len() >= policy.max_regions {
            return Err(Error::MaxRegionsExceeded);
        }

        let next_id: u32 = env.storage().instance().get(&NEXT_REGION_ID).unwrap();
        let region_node = RegionNode {
            region: region.clone(),
            node_id,
            status: RegionStatus::Active,
            endpoint_hash,
            last_heartbeat: env.ledger().timestamp(),
            replica_count: 0,
            is_primary,
            failure_count: 0,
        };

        regions.push_back(region_node);
        env.storage().persistent().set(&REGIONS, &regions);
        env.storage().instance().set(&NEXT_REGION_ID, &(next_id + 1));

        env.events().publish((symbol_short!("DRO_REGI"),), next_id);
        Ok(next_id)
    }

    pub fn list_regions(env: Env) -> Vec<RegionNode> {
        env.storage()
            .persistent()
            .get(&REGIONS)
            .unwrap_or_else(|| Vec::new(&env))
    }

    pub fn get_region_status(env: Env, region_id: u32) -> Option<RegionStatus> {
        let regions: Vec<RegionNode> = env
            .storage()
            .persistent()
            .get(&REGIONS)
            .unwrap_or_else(|| Vec::new(&env));

        for i in 0..regions.len() {
            if regions.get_unchecked(i).node_id == region_id {
                return Some(regions.get_unchecked(i).status.clone());
            }
        }
        None
    }

    pub fn update_region_status(
        env: Env,
        caller: Address,
        region_id: u32,
        status: RegionStatus,
    ) -> Result<(), Error> {
        Self::check_paused(&env)?;
        Self::require_operator(&env, &caller)?;

        let mut regions: Vec<RegionNode> = env
            .storage()
            .persistent()
            .get(&REGIONS)
            .unwrap_or_else(|| Vec::new(&env));

        let mut found = false;
        for i in 0..regions.len() {
            let mut node = regions.get_unchecked(i).clone();
            if node.node_id == region_id {
                node.status = status.clone();
                node.last_heartbeat = env.ledger().timestamp();
                regions.set(i, node);
                found = true;
                break;
            }
        }

        if !found {
            return Err(Error::InvalidInput);
        }

        env.storage().persistent().set(&REGIONS, &regions);
        env.events().publish((symbol_short!("DRO_STAT"),), region_id);
        Ok(())
    }

    // ========================================================================
    // Failover Management
    // ========================================================================

    pub fn trigger_failover(
        env: Env,
        caller: Address,
        source_region_id: u32,
        target_region_id: u32,
        reason: Symbol,
    ) -> Result<u64, Error> {
        Self::check_paused(&env)?;
        Self::require_operator(&env, &caller)?;

        let regions: Vec<RegionNode> = env
            .storage()
            .persistent()
            .get(&REGIONS)
            .unwrap_or_else(|| Vec::new(&env));

        // Verify regions exist and target is active
        let mut source_exists = false;
        let mut target_active = false;

        for i in 0..regions.len() {
            let node = regions.get_unchecked(i);
            if node.node_id == source_region_id {
                source_exists = true;
            }
            if node.node_id == target_region_id {
                match &node.status {
                    RegionStatus::Active => target_active = true,
                    _ => {},
                }
            }
        }

        if !source_exists {
            return Err(Error::InvalidInput);
        }
        if !target_active {
            return Err(Error::FailoverFailed);
        }

        let failover_id: u64 = env.storage().instance().get(&NEXT_FAILOVER_ID).unwrap();
        let start_time = env.ledger().timestamp();
        
        // Simulate failover execution
        let completed_at = env.ledger().timestamp();
        let rto_ms = (completed_at - start_time) * 1000;

        let policy: DRPolicy = env.storage().instance().get(&DR_POLICY).unwrap();
        let success = rto_ms <= policy.rto_target_ms;

        let failover_event = FailoverEvent {
            event_id: failover_id,
            triggered_at: start_time,
            source_region: RegionStatus::Active.clone(), // Simplified, should use actual region
            target_region: RegionStatus::Active.clone(),
            reason,
            rto_ms,
            success,
        };

        let mut failovers: Vec<FailoverEvent> = env
            .storage()
            .persistent()
            .get(&FAILOVER_EVENTS)
            .unwrap_or_else(|| Vec::new(&env));
        failovers.push_back(failover_event);
        env.storage().persistent().set(&FAILOVER_EVENTS, &failovers);
        env.storage()
            .instance()
            .set(&NEXT_FAILOVER_ID, &(failover_id + 1));

        if !success {
            return Err(Error::RtoExceeded);
        }

        env.events().publish((symbol_short!("DRO_FAIL"),), failover_id);
        Ok(failover_id)
    }

    pub fn get_failover_events(env: Env) -> Vec<FailoverEvent> {
        env.storage()
            .persistent()
            .get(&FAILOVER_EVENTS)
            .unwrap_or_else(|| Vec::new(&env))
    }

    // ========================================================================
    // Data Synchronization
    // ========================================================================

    pub fn sync_data(
        env: Env,
        caller: Address,
        source_region_id: u32,
        target_region_ids: Vec<u32>,
        data_hash: u64,
    ) -> Result<u64, Error> {
        Self::check_paused(&env)?;
        Self::require_operator(&env, &caller)?;

        if target_region_ids.len() == 0 {
            return Err(Error::InvalidInput);
        }

        let sync_id: u64 = env.storage().instance().get(&NEXT_SYNC_ID).unwrap();
        let start_time = env.ledger().timestamp();

        // Convert u32 to GeoRegion for storage (simplified)
        let target_regions: Vec<GeoRegion> = Vec::new(&env);

        let sync_op = SyncOperation {
            sync_id,
            source_region: GeoRegion::UsEast, // Simplified
            target_regions,
            data_hash,
            started_at: start_time,
            completed_at: env.ledger().timestamp(),
            success: true,
        };

        let mut sync_ops: Vec<SyncOperation> = env
            .storage()
            .persistent()
            .get(&SYNC_OPS)
            .unwrap_or_else(|| Vec::new(&env));
        sync_ops.push_back(sync_op);
        env.storage().persistent().set(&SYNC_OPS, &sync_ops);
        env.storage().instance().set(&NEXT_SYNC_ID, &(sync_id + 1));

        env.events().publish((symbol_short!("DRO_SYNC"),), sync_id);
        Ok(sync_id)
    }

    pub fn get_sync_operations(env: Env) -> Vec<SyncOperation> {
        env.storage()
            .persistent()
            .get(&SYNC_OPS)
            .unwrap_or_else(|| Vec::new(&env))
    }

    // ========================================================================
    // Health Monitoring & SLA
    // ========================================================================

    pub fn check_health(env: Env, caller: Address) -> Result<bool, Error> {
        Self::require_auditor(&env, &caller)?;

        let regions: Vec<RegionNode> = env
            .storage()
            .persistent()
            .get(&REGIONS)
            .unwrap_or_else(|| Vec::new(&env));

        if regions.len() == 0 {
            return Err(Error::AllRegionsUnavailable);
        }

        let mut active_count = 0;
        for i in 0..regions.len() {
            match &regions.get_unchecked(i).status {
                RegionStatus::Active => active_count += 1,
                _ => {},
            }
        }

        let policy: DRPolicy = env.storage().instance().get(&DR_POLICY).unwrap();
        let health_ok = active_count >= policy.min_replicas_per_region;

        env.storage()
            .instance()
            .set(&LAST_HEALTH_CHECK, &env.ledger().timestamp());

        env.events().publish((symbol_short!("DRO_HLTH"),), health_ok);
        Ok(health_ok)
    }

    pub fn record_uptime_metric(
        env: Env,
        caller: Address,
        start_time: u64,
        end_time: u64,
        uptime_basis_points: u32,
        outages: u32,
        total_outage_ms: u64,
    ) -> Result<(), Error> {
        Self::require_auditor(&env, &caller)?;

        if uptime_basis_points > 10000 || start_time >= end_time {
            return Err(Error::InvalidInput);
        }

        let metric = UptimeMetric {
            start_time,
            end_time,
            uptime_basis_points,
            outages,
            total_outage_ms,
        };

        let mut metrics: Vec<UptimeMetric> = env
            .storage()
            .persistent()
            .get(&UPTIME_METRICS)
            .unwrap_or_else(|| Vec::new(&env));
        metrics.push_back(metric);
        env.storage().persistent().set(&UPTIME_METRICS, &metrics);

        env.events().publish((symbol_short!("DRO_SLAM"),), uptime_basis_points);
        Ok(())
    }

    pub fn get_uptime_metrics(env: Env) -> Vec<UptimeMetric> {
        env.storage()
            .persistent()
            .get(&UPTIME_METRICS)
            .unwrap_or_else(|| Vec::new(&env))
    }

    pub fn get_current_uptime(env: Env) -> u32 {
        let metrics: Vec<UptimeMetric> = env
            .storage()
            .persistent()
            .get(&UPTIME_METRICS)
            .unwrap_or_else(|| Vec::new(&env));

        if metrics.len() == 0 {
            return 10000; // Default to 100%
        }

        metrics.get_unchecked(metrics.len() - 1).uptime_basis_points
    }

    // ========================================================================
    // Policy Management
    // ========================================================================

    pub fn set_policy(env: Env, caller: Address, policy: DRPolicy) -> Result<(), Error> {
        Self::require_admin(&env, &caller)?;

        if policy.max_regions > MAX_REGIONS || policy.max_regions == 0 {
            return Err(Error::InvalidInput);
        }

        env.storage().instance().set(&DR_POLICY, &policy);
        env.events().publish((symbol_short!("DRO_SETP"),), caller);
        Ok(())
    }

    pub fn get_policy(env: Env) -> DRPolicy {
        env.storage()
            .instance()
            .get(&DR_POLICY)
            .unwrap_or_else(|| DRPolicy {
                min_replicas_per_region: 3,
                max_regions: 5,
                failover_timeout_ms: 300000,
                sync_interval_ms: 60000,
                health_check_interval_ms: 30000,
                auto_failover_enabled: true,
                rto_target_ms: RTO_THRESHOLD_MS,
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

    fn require_auditor(env: &Env, caller: &Address) -> Result<(), Error> {
        let roles: Map<Address, u32> = env
            .storage()
            .instance()
            .get(&ROLES)
            .ok_or(Error::NotAuthorized)?;

        let role_mask = roles.get(caller.clone()).ok_or(Error::NotAuthorized)?;
        if (role_mask & ROLE_AUDITOR) == 0 {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    fn check_paused(env: &Env) -> Result<(), Error> {
        let paused: bool = env.storage().instance().get(&PAUSED).unwrap_or(false);
        if paused {
            Err(Error::InvalidInput)
        } else {
            Ok(())
        }
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

        let result = MultiRegionOrchestrator::initialize(env.clone(), admin.clone());
        assert!(result.is_ok());

        // Test double initialization fails
        let result = MultiRegionOrchestrator::initialize(env, admin);
        assert!(matches!(result, Err(Error::AlreadyInitialized)));
    }

    #[test]
    fn test_register_region() {
        let env = Env::default();
        let admin = Address::random(&env);
        let operator = Address::random(&env);

        MultiRegionOrchestrator::initialize(env.clone(), admin.clone()).unwrap();
        MultiRegionOrchestrator::assign_role(env.clone(), admin.clone(), operator.clone(), ROLE_OPERATOR).unwrap();

        let result = MultiRegionOrchestrator::register_region(
            env.clone(),
            operator.clone(),
            GeoRegion::UsEast,
            1,
            12345,
            true,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_failover_event() {
        let env = Env::default();
        let admin = Address::random(&env);
        let operator = Address::random(&env);

        MultiRegionOrchestrator::initialize(env.clone(), admin.clone()).unwrap();
        MultiRegionOrchestrator::assign_role(env.clone(), admin.clone(), operator.clone(), ROLE_OPERATOR).unwrap();

        // Register two regions
        MultiRegionOrchestrator::register_region(
            env.clone(),
            operator.clone(),
            GeoRegion::UsEast,
            1,
            12345,
            true,
        ).unwrap();

        MultiRegionOrchestrator::register_region(
            env.clone(),
            operator.clone(),
            GeoRegion::UsWest,
            2,
            67890,
            false,
        ).unwrap();

        // Trigger failover
        let result = MultiRegionOrchestrator::trigger_failover(
            env.clone(),
            operator.clone(),
            1,
            2,
            symbol_short!("FAULT"),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_policy_management() {
        let env = Env::default();
        let admin = Address::random(&env);

        MultiRegionOrchestrator::initialize(env.clone(), admin.clone()).unwrap();

        let policy = MultiRegionOrchestrator::get_policy(env.clone());
        assert_eq!(policy.min_replicas_per_region, 3);

        let new_policy = DRPolicy {
            min_replicas_per_region: 5,
            max_regions: 10,
            failover_timeout_ms: 600000,
            sync_interval_ms: 30000,
            health_check_interval_ms: 15000,
            auto_failover_enabled: true,
            rto_target_ms: RTO_THRESHOLD_MS,
        };

        let result = MultiRegionOrchestrator::set_policy(env.clone(), admin, new_policy.clone());
        assert!(result.is_ok());

        let updated_policy = MultiRegionOrchestrator::get_policy(env);
        assert_eq!(updated_policy.min_replicas_per_region, 5);
    }
}
