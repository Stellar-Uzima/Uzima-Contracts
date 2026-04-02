#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec, map, Map, String};

// ============================================================================
// Data Types & Constants
// ============================================================================

const ROLE_ADMIN: u32 = 1;
const ROLE_OPERATOR: u32 = 2;
const ROLE_MONITOR: u32 = 4;
const ALL_ROLES: u32 = 7;

#[derive(Clone)]
#[contracttype]
pub enum NodeStatus {
    Healthy = 0,
    Degraded = 1,
    Unhealthy = 2,
    Unreachable = 3,
}

#[derive(Clone)]
#[contracttype]
pub struct RegionalNode {
    pub node_id: u32,
    pub region_name: String,
    pub status: NodeStatus,
    pub cpu_usage_percent: u32,
    pub memory_usage_percent: u32,
    pub disk_usage_percent: u32,
    pub last_heartbeat: u64,
    pub replica_lag_ms: u64,
    pub total_uptime_ms: u64,
    pub failure_count: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct HealthCheckResult {
    pub check_id: u64,
    pub node_id: u32,
    pub checked_at: u64,
    pub status: NodeStatus,
    pub cpu_usage: u32,
    pub memory_usage: u32,
    pub disk_usage: u32,
    pub response_time_ms: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct ReplicaInfo {
    pub replica_id: u32,
    pub node_id: u32,
    pub data_hash: u64,
    pub last_synced: u64,
    pub lag_ms: u64,
    pub is_in_sync: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct NodeConfiguration {
    pub max_cpu_threshold: u32,
    pub max_memory_threshold: u32,
    pub max_disk_threshold: u32,
    pub max_replica_lag_ms: u64,
    pub heartbeat_timeout_ms: u64,
    pub health_check_interval_ms: u64,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    InvalidInput = 4,
    NodeNotFound = 5,
    HealthCheckFailed = 6,
    ReplicaOutOfSync = 7,
    NodeUnreachable = 8,
    InvalidThreshold = 9,
    DuplicateNode = 10,
}

// ============================================================================
// Storage Keys
// ============================================================================

const ADMIN: Symbol = symbol_short!("ADMIN");
const INITIALIZED: Symbol = symbol_short!("INIT");
const ROLES: Symbol = symbol_short!("ROLES");
const NODES: Symbol = symbol_short!("NODES");
const CONFIG: Symbol = symbol_short!("CFG");
const NEXT_NODE_ID: Symbol = symbol_short!("NNID");
const NEXT_CHECK_ID: Symbol = symbol_short!("NCID");
const HEALTH_CHECKS: Symbol = symbol_short!("HCKS");
const REPLICAS: Symbol = symbol_short!("REPL");

// ============================================================================
// Contract Implementation
// ============================================================================

#[contract]
pub struct RegionalNodeManager;

#[contractimpl]
impl RegionalNodeManager {
    // ========================================================================
    // Initialization & Admin
    // ========================================================================

    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&INITIALIZED) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&INITIALIZED, &true);
        env.storage().instance().set(&NEXT_NODE_ID, &1u32);
        env.storage().instance().set(&NEXT_CHECK_ID, &1u64);

        // Default configuration
        let default_config = NodeConfiguration {
            max_cpu_threshold: 85,
            max_memory_threshold: 80,
            max_disk_threshold: 90,
            max_replica_lag_ms: 5000,
            heartbeat_timeout_ms: 30000,
            health_check_interval_ms: 10000,
        };
        env.storage().instance().set(&CONFIG, &default_config);

        env.events().publish((symbol_short!("RNM_INIT"),), admin);
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
    // Node Management
    // ========================================================================

    pub fn register_node(
        env: Env,
        caller: Address,
        region_name: String,
    ) -> Result<u32, Error> {
        Self::require_operator(&env, &caller)?;

        let mut nodes: Vec<RegionalNode> = env
            .storage()
            .persistent()
            .get(&NODES)
            .unwrap_or_else(|| Vec::new(&env));

        // Check for duplicate
        for i in 0..nodes.len() {
            if nodes.get_unchecked(i).region_name == region_name {
                return Err(Error::DuplicateNode);
            }
        }

        let node_id: u32 = env.storage().instance().get(&NEXT_NODE_ID).unwrap();
        let current_time = env.ledger().timestamp();

        let node = RegionalNode {
            node_id,
            region_name,
            status: NodeStatus::Healthy,
            cpu_usage_percent: 0,
            memory_usage_percent: 0,
            disk_usage_percent: 0,
            last_heartbeat: current_time,
            replica_lag_ms: 0,
            total_uptime_ms: 0,
            failure_count: 0,
        };

        nodes.push_back(node);
        env.storage().persistent().set(&NODES, &nodes);
        env.storage().instance().set(&NEXT_NODE_ID, &(node_id + 1));

        env.events().publish((symbol_short!("RNM_REG"),), node_id);
        Ok(node_id)
    }

    pub fn get_node(env: Env, node_id: u32) -> Option<RegionalNode> {
        let nodes: Vec<RegionalNode> = env
            .storage()
            .persistent()
            .get(&NODES)
            .unwrap_or_else(|| Vec::new(&env));

        for i in 0..nodes.len() {
            if nodes.get_unchecked(i).node_id == node_id {
                return Some(nodes.get_unchecked(i).clone());
            }
        }
        None
    }

    pub fn list_nodes(env: Env) -> Vec<RegionalNode> {
        env.storage()
            .persistent()
            .get(&NODES)
            .unwrap_or_else(|| Vec::new(&env))
    }

    pub fn update_node_metrics(
        env: Env,
        caller: Address,
        node_id: u32,
        cpu_usage: u32,
        memory_usage: u32,
        disk_usage: u32,
        replica_lag_ms: u64,
    ) -> Result<(), Error> {
        Self::require_operator(&env, &caller)?;

        let config: NodeConfiguration = env.storage().instance().get(&CONFIG).unwrap();

        if cpu_usage > 100 || memory_usage > 100 || disk_usage > 100 {
            return Err(Error::InvalidInput);
        }

        let mut nodes: Vec<RegionalNode> = env
            .storage()
            .persistent()
            .get(&NODES)
            .unwrap_or_else(|| Vec::new(&env));

        let mut found = false;
        for i in 0..nodes.len() {
            let mut node = nodes.get_unchecked(i).clone();
            if node.node_id == node_id {
                node.cpu_usage_percent = cpu_usage;
                node.memory_usage_percent = memory_usage;
                node.disk_usage_percent = disk_usage;
                node.replica_lag_ms = replica_lag_ms;
                node.last_heartbeat = env.ledger().timestamp();

                // Determine health status
                node.status = if cpu_usage > config.max_cpu_threshold
                    || memory_usage > config.max_memory_threshold
                    || disk_usage > config.max_disk_threshold
                {
                    NodeStatus::Degraded
                } else if replica_lag_ms > config.max_replica_lag_ms {
                    NodeStatus::Degraded
                } else {
                    NodeStatus::Healthy
                };

                nodes.set(i, node);
                found = true;
                break;
            }
        }

        if !found {
            return Err(Error::NodeNotFound);
        }

        env.storage().persistent().set(&NODES, &nodes);
        env.events().publish((symbol_short!("RNM_UPD"),), node_id);
        Ok(())
    }

    // ========================================================================
    // Health Checking
    // ========================================================================

    pub fn perform_health_check(
        env: Env,
        caller: Address,
        node_id: u32,
    ) -> Result<u64, Error> {
        Self::require_monitor(&env, &caller)?;

        let node = Self::get_node(env.clone(), node_id).ok_or(Error::NodeNotFound)?;

        let check_id: u64 = env.storage().instance().get(&NEXT_CHECK_ID).unwrap();
        let current_time = env.ledger().timestamp();
        let response_time_ms = 100; // Simulated

        // Simulate health check
        let status = if node.cpu_usage_percent > 85 || node.memory_usage_percent > 80 {
            NodeStatus::Degraded
        } else {
            NodeStatus::Healthy
        };

        let check_result = HealthCheckResult {
            check_id,
            node_id,
            checked_at: current_time,
            status,
            cpu_usage: node.cpu_usage_percent,
            memory_usage: node.memory_usage_percent,
            disk_usage: node.disk_usage_percent,
            response_time_ms,
        };

        let mut checks: Vec<HealthCheckResult> = env
            .storage()
            .persistent()
            .get(&HEALTH_CHECKS)
            .unwrap_or_else(|| Vec::new(&env));
        checks.push_back(check_result);
        env.storage().persistent().set(&HEALTH_CHECKS, &checks);
        env.storage().instance().set(&NEXT_CHECK_ID, &(check_id + 1));

        env.events().publish((symbol_short!("RNM_HLTH"),), check_id);
        Ok(check_id)
    }

    pub fn get_health_checks(env: Env) -> Vec<HealthCheckResult> {
        env.storage()
            .persistent()
            .get(&HEALTH_CHECKS)
            .unwrap_or_else(|| Vec::new(&env))
    }

    pub fn get_recent_health_check(env: Env, node_id: u32) -> Option<HealthCheckResult> {
        let checks: Vec<HealthCheckResult> = env
            .storage()
            .persistent()
            .get(&HEALTH_CHECKS)
            .unwrap_or_else(|| Vec::new(&env));

        if checks.len() == 0 {
            return None;
        }

        for i in (0..checks.len()).rev() {
            if checks.get_unchecked(i).node_id == node_id {
                return Some(checks.get_unchecked(i).clone());
            }
        }
        None
    }

    // ========================================================================
    // Replica Management
    // ========================================================================

    pub fn register_replica(
        env: Env,
        caller: Address,
        replica_id: u32,
        node_id: u32,
        data_hash: u64,
    ) -> Result<(), Error> {
        Self::require_operator(&env, &caller)?;

        // Verify node exists
        Self::get_node(env.clone(), node_id).ok_or(Error::NodeNotFound)?;

        let replica = ReplicaInfo {
            replica_id,
            node_id,
            data_hash,
            last_synced: env.ledger().timestamp(),
            lag_ms: 0,
            is_in_sync: true,
        };

        let mut replicas: Vec<ReplicaInfo> = env
            .storage()
            .persistent()
            .get(&REPLICAS)
            .unwrap_or_else(|| Vec::new(&env));
        replicas.push_back(replica);
        env.storage().persistent().set(&REPLICAS, &replicas);

        env.events().publish((symbol_short!("RNM_REPL"),), replica_id);
        Ok(())
    }

    pub fn update_replica_sync(
        env: Env,
        caller: Address,
        replica_id: u32,
        lag_ms: u64,
    ) -> Result<(), Error> {
        Self::require_operator(&env, &caller)?;

        let config: NodeConfiguration = env.storage().instance().get(&CONFIG).unwrap();
        let is_in_sync = lag_ms <= config.max_replica_lag_ms;

        let mut replicas: Vec<ReplicaInfo> = env
            .storage()
            .persistent()
            .get(&REPLICAS)
            .unwrap_or_else(|| Vec::new(&env));

        let mut found = false;
        for i in 0..replicas.len() {
            let mut replica = replicas.get_unchecked(i).clone();
            if replica.replica_id == replica_id {
                replica.lag_ms = lag_ms;
                replica.is_in_sync = is_in_sync;
                replica.last_synced = env.ledger().timestamp();
                replicas.set(i, replica);
                found = true;
                break;
            }
        }

        if !found {
            return Err(Error::NodeNotFound);
        }

        env.storage().persistent().set(&REPLICAS, &replicas);
        env.events().publish((symbol_short!("RNM_SYNC"),), replica_id);
        Ok(())
    }

    pub fn get_replicas_for_node(env: Env, node_id: u32) -> Vec<ReplicaInfo> {
        let replicas: Vec<ReplicaInfo> = env
            .storage()
            .persistent()
            .get(&REPLICAS)
            .unwrap_or_else(|| Vec::new(&env));

        let mut result = Vec::new(&env);
        for i in 0..replicas.len() {
            if replicas.get_unchecked(i).node_id == node_id {
                result.push_back(replicas.get_unchecked(i).clone());
            }
        }
        result
    }

    // ========================================================================
    // Configuration
    // ========================================================================

    pub fn set_configuration(env: Env, caller: Address, config: NodeConfiguration) -> Result<(), Error> {
        Self::require_admin(&env, &caller)?;

        if config.max_cpu_threshold > 100
            || config.max_memory_threshold > 100
            || config.max_disk_threshold > 100
        {
            return Err(Error::InvalidThreshold);
        }

        env.storage().instance().set(&CONFIG, &config);
        env.events().publish((symbol_short!("RNM_CFG"),), caller);
        Ok(())
    }

    pub fn get_configuration(env: Env) -> NodeConfiguration {
        env.storage()
            .instance()
            .get(&CONFIG)
            .unwrap_or_else(|| NodeConfiguration {
                max_cpu_threshold: 85,
                max_memory_threshold: 80,
                max_disk_threshold: 90,
                max_replica_lag_ms: 5000,
                heartbeat_timeout_ms: 30000,
                health_check_interval_ms: 10000,
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

    fn require_monitor(env: &Env, caller: &Address) -> Result<(), Error> {
        let roles: Map<Address, u32> = env
            .storage()
            .instance()
            .get(&ROLES)
            .ok_or(Error::NotAuthorized)?;

        let role_mask = roles.get(caller.clone()).ok_or(Error::NotAuthorized)?;
        if (role_mask & (ROLE_ADMIN | ROLE_MONITOR)) == 0 {
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

        let result = RegionalNodeManager::initialize(env.clone(), admin.clone());
        assert!(result.is_ok());

        let result = RegionalNodeManager::initialize(env, admin);
        assert!(matches!(result, Err(Error::AlreadyInitialized)));
    }

    #[test]
    fn test_register_node() {
        let env = Env::default();
        let admin = Address::random(&env);
        let operator = Address::random(&env);

        RegionalNodeManager::initialize(env.clone(), admin.clone()).unwrap();
        RegionalNodeManager::assign_role(env.clone(), admin, operator.clone(), ROLE_OPERATOR).unwrap();

        let result = RegionalNodeManager::register_node(
            env.clone(),
            operator,
            String::from_bytes(&env, &b"us-east-1"[..]),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_health_check() {
        let env = Env::default();
        let admin = Address::random(&env);
        let operator = Address::random(&env);
        let monitor = Address::random(&env);

        RegionalNodeManager::initialize(env.clone(), admin.clone()).unwrap();
        RegionalNodeManager::assign_role(env.clone(), admin.clone(), operator.clone(), ROLE_OPERATOR)
            .unwrap();
        RegionalNodeManager::assign_role(env.clone(), admin, monitor.clone(), ROLE_MONITOR).unwrap();

        let node_id = RegionalNodeManager::register_node(
            env.clone(),
            operator.clone(),
            String::from_bytes(&env, &b"us-west-1"[..]),
        )
        .unwrap();

        RegionalNodeManager::update_node_metrics(env.clone(), operator, node_id, 50, 60, 40, 100)
            .unwrap();

        let result = RegionalNodeManager::perform_health_check(env, monitor, node_id);
        assert!(result.is_ok());
    }
}
