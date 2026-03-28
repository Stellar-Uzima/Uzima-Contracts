#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec, map, Map};

// ============================================================================
// Data Types & Constants
// ============================================================================

const ROLE_ADMIN: u32 = 1;
const ROLE_OPERATOR: u32 = 2;
const ROLE_ANALYZER: u32 = 4;
const ALL_ROLES: u32 = 7;

const HEARTBEAT_TIMEOUT_MS: u64 = 30000; // 30 seconds
const FAILURE_THRESHOLD: u32 = 3; // Trigger after 3 consecutive failures

#[derive(Clone)]
#[contracttype]
pub enum FailoverReason {
    NodeFailure = 0,
    HeartbeatTimeout = 1,
    HighLatency = 2,
    ResourceExhaustion = 3,
    DataInconsistency = 4,
    ManualTrigger = 5,
}

#[derive(Clone)]
#[contracttype]
pub enum FailoverState {
    Pending = 0,
    InProgress = 1,
    Completed = 2,
    RolledBack = 3,
    Failed = 4,
}

#[derive(Clone)]
#[contracttype]
pub struct FailoverDetection {
    pub detection_id: u64,
    pub source_node_id: u32,
    pub detected_at: u64,
    pub reason: FailoverReason,
    pub severity_level: u32, // 1-5
    pub consecutive_failures: u32,
    pub is_critical: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct FailoverExecution {
    pub execution_id: u64,
    pub detection_id: u64,
    pub source_node_id: u32,
    pub target_node_id: u32,
    pub initiated_at: u64,
    pub completed_at: u64,
    pub state: FailoverState,
    pub rto_ms: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct NodeFailureMetric {
    pub node_id: u32,
    pub consecutive_failures: u32,
    pub last_failure_at: u64,
    pub total_failures: u64,
    pub recovery_attempts: u32,
    pub last_successful_recovery: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct FailoverPlan {
    pub plan_id: u64,
    pub source_node_id: u32,
    pub target_nodes: Vec<u32>,
    pub priority_order: Vec<u32>,
    pub created_at: u64,
    pub is_active: bool,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    InvalidInput = 4,
    NodeNotFound = 5,
    FailoverNotFound = 6,
    NoAvailableTargets = 7,
    FailoverInProgress = 8,
    MaxFailuresReached = 9,
    RecoveryFailed = 10,
}

// ============================================================================
// Storage Keys
// ============================================================================

const ADMIN: Symbol = symbol_short!("ADMIN");
const INITIALIZED: Symbol = symbol_short!("INIT");
const ROLES: Symbol = symbol_short!("ROLES");
const DETECTIONS: Symbol = symbol_short!("DETC");
const EXECUTIONS: Symbol = symbol_short!("EXEC");
const METRICS: Symbol = symbol_short!("METR");
const PLANS: Symbol = symbol_short!("PLAN");
const NEXT_DETECTION_ID: Symbol = symbol_short!("NDID");
const NEXT_EXECUTION_ID: Symbol = symbol_short!("NEID");
const NEXT_PLAN_ID: Symbol = symbol_short!("NPID");
const FAILOVER_IN_PROGRESS: Symbol = symbol_short!("FIP");

// ============================================================================
// Contract Implementation
// ============================================================================

#[contract]
pub struct FailoverDetector;

#[contractimpl]
impl FailoverDetector {
    // ========================================================================
    // Initialization & Admin
    // ========================================================================

    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&INITIALIZED) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&INITIALIZED, &true);
        env.storage().instance().set(&NEXT_DETECTION_ID, &1u64);
        env.storage().instance().set(&NEXT_EXECUTION_ID, &1u64);
        env.storage().instance().set(&NEXT_PLAN_ID, &1u64);
        env.storage().instance().set(&FAILOVER_IN_PROGRESS, &false);

        env.events().publish((symbol_short!("FD_INIT"),), admin);
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
    // Failover Detection
    // ========================================================================

    pub fn detect_node_failure(
        env: Env,
        caller: Address,
        node_id: u32,
        reason: FailoverReason,
        severity_level: u32,
    ) -> Result<u64, Error> {
        Self::require_operator(&env, &caller)?;

        if severity_level < 1 || severity_level > 5 {
            return Err(Error::InvalidInput);
        }

        let detection_id: u64 = env.storage().instance().get(&NEXT_DETECTION_ID).unwrap();

        // Get or create failure metrics
        let mut metrics: Vec<NodeFailureMetric> = env
            .storage()
            .persistent()
            .get(&METRICS)
            .unwrap_or_else(|| Vec::new(&env));

        let mut node_metric: Option<NodeFailureMetric> = None;
        let mut found_index: Option<usize> = None;

        for i in 0..metrics.len() {
            if metrics.get_unchecked(i).node_id == node_id {
                node_metric = Some(metrics.get_unchecked(i).clone());
                found_index = Some(i);
                break;
            }
        }

        let mut current_failures = if let Some(metric) = &node_metric {
            metric.consecutive_failures + 1
        } else {
            1
        };

        let is_critical = current_failures >= FAILURE_THRESHOLD;

        // Update metrics
        let updated_metric = NodeFailureMetric {
            node_id,
            consecutive_failures: current_failures,
            last_failure_at: env.ledger().timestamp(),
            total_failures: if let Some(metric) = node_metric.clone() {
                metric.total_failures + 1
            } else {
                1
            },
            recovery_attempts: if let Some(metric) = node_metric {
                metric.recovery_attempts
            } else {
                0
            },
            last_successful_recovery: if let Some(metric) = node_metric {
                metric.last_successful_recovery
            } else {
                0
            },
        };

        if let Some(idx) = found_index {
            metrics.set(idx, updated_metric.clone());
        } else {
            metrics.push_back(updated_metric.clone());
        }
        env.storage().persistent().set(&METRICS, &metrics);

        // Create detection record
        let detection = FailoverDetection {
            detection_id,
            source_node_id: node_id,
            detected_at: env.ledger().timestamp(),
            reason,
            severity_level,
            consecutive_failures: current_failures,
            is_critical,
        };

        let mut detections: Vec<FailoverDetection> = env
            .storage()
            .persistent()
            .get(&DETECTIONS)
            .unwrap_or_else(|| Vec::new(&env));
        detections.push_back(detection);
        env.storage().persistent().set(&DETECTIONS, &detections);
        env.storage()
            .instance()
            .set(&NEXT_DETECTION_ID, &(detection_id + 1));

        env.events().publish((symbol_short!("FD_DETC"),), detection_id);

        if is_critical {
            env.events().publish((symbol_short!("FD_CRIT"),), node_id);
        }

        Ok(detection_id)
    }

    pub fn get_detections(env: Env) -> Vec<FailoverDetection> {
        env.storage()
            .persistent()
            .get(&DETECTIONS)
            .unwrap_or_else(|| Vec::new(&env))
    }

    pub fn get_node_metrics(env: Env, node_id: u32) -> Option<NodeFailureMetric> {
        let metrics: Vec<NodeFailureMetric> = env
            .storage()
            .persistent()
            .get(&METRICS)
            .unwrap_or_else(|| Vec::new(&env));

        for i in 0..metrics.len() {
            if metrics.get_unchecked(i).node_id == node_id {
                return Some(metrics.get_unchecked(i).clone());
            }
        }
        None
    }

    // ========================================================================
    // Failover Planning & Execution
    // ========================================================================

    pub fn create_failover_plan(
        env: Env,
        caller: Address,
        source_node_id: u32,
        target_nodes: Vec<u32>,
    ) -> Result<u64, Error> {
        Self::require_operator(&env, &caller)?;

        if target_nodes.len() == 0 {
            return Err(Error::NoAvailableTargets);
        }

        let plan_id: u64 = env.storage().instance().get(&NEXT_PLAN_ID).unwrap();

        let plan = FailoverPlan {
            plan_id,
            source_node_id,
            target_nodes: target_nodes.clone(),
            priority_order: target_nodes.clone(),
            created_at: env.ledger().timestamp(),
            is_active: true,
        };

        let mut plans: Vec<FailoverPlan> = env
            .storage()
            .persistent()
            .get(&PLANS)
            .unwrap_or_else(|| Vec::new(&env));
        plans.push_back(plan);
        env.storage().persistent().set(&PLANS, &plans);
        env.storage().instance().set(&NEXT_PLAN_ID, &(plan_id + 1));

        env.events().publish((symbol_short!("FD_PLAN"),), plan_id);
        Ok(plan_id)
    }

    pub fn execute_failover(
        env: Env,
        caller: Address,
        detection_id: u64,
        target_node_id: u32,
    ) -> Result<u64, Error> {
        Self::require_operator(&env, &caller)?;

        let failover_in_progress: bool = env
            .storage()
            .instance()
            .get(&FAILOVER_IN_PROGRESS)
            .unwrap_or(false);
        if failover_in_progress {
            return Err(Error::FailoverInProgress);
        }

        // Get detection
        let detections: Vec<FailoverDetection> = env
            .storage()
            .persistent()
            .get(&DETECTIONS)
            .unwrap_or_else(|| Vec::new(&env));

        let mut source_node_id: u32 = 0;
        for i in 0..detections.len() {
            if detections.get_unchecked(i).detection_id == detection_id {
                source_node_id = detections.get_unchecked(i).source_node_id;
                break;
            }
        }

        if source_node_id == 0 {
            return Err(Error::FailoverNotFound);
        }

        env.storage()
            .instance()
            .set(&FAILOVER_IN_PROGRESS, &true);

        let execution_id: u64 = env.storage().instance().get(&NEXT_EXECUTION_ID).unwrap();
        let initiated_at = env.ledger().timestamp();
        let completed_at = env.ledger().timestamp();
        let rto_ms = (completed_at - initiated_at) * 1000;

        let execution = FailoverExecution {
            execution_id,
            detection_id,
            source_node_id,
            target_node_id,
            initiated_at,
            completed_at,
            state: FailoverState::Completed,
            rto_ms,
        };

        let mut executions: Vec<FailoverExecution> = env
            .storage()
            .persistent()
            .get(&EXECUTIONS)
            .unwrap_or_else(|| Vec::new(&env));
        executions.push_back(execution);
        env.storage().persistent().set(&EXECUTIONS, &executions);
        env.storage()
            .instance()
            .set(&NEXT_EXECUTION_ID, &(execution_id + 1));

        env.storage()
            .instance()
            .set(&FAILOVER_IN_PROGRESS, &false);

        // Reset consecutive failures for recovered node
        let mut metrics: Vec<NodeFailureMetric> = env
            .storage()
            .persistent()
            .get(&METRICS)
            .unwrap_or_else(|| Vec::new(&env));

        for i in 0..metrics.len() {
            let mut metric = metrics.get_unchecked(i).clone();
            if metric.node_id == source_node_id {
                metric.consecutive_failures = 0;
                metric.recovery_attempts += 1;
                metric.last_successful_recovery = env.ledger().timestamp();
                metrics.set(i, metric);
                break;
            }
        }
        env.storage().persistent().set(&METRICS, &metrics);

        env.events().publish((symbol_short!("FD_EXEC"),), execution_id);
        Ok(execution_id)
    }

    pub fn get_failover_executions(env: Env) -> Vec<FailoverExecution> {
        env.storage()
            .persistent()
            .get(&EXECUTIONS)
            .unwrap_or_else(|| Vec::new(&env))
    }

    pub fn get_failover_plans(env: Env) -> Vec<FailoverPlan> {
        env.storage()
            .persistent()
            .get(&PLANS)
            .unwrap_or_else(|| Vec::new(&env))
    }

    // ========================================================================
    // Recovery Operations
    // ========================================================================

    pub fn mark_recovery_success(
        env: Env,
        caller: Address,
        node_id: u32,
    ) -> Result<(), Error> {
        Self::require_operator(&env, &caller)?;

        let mut metrics: Vec<NodeFailureMetric> = env
            .storage()
            .persistent()
            .get(&METRICS)
            .unwrap_or_else(|| Vec::new(&env));

        let mut found = false;
        for i in 0..metrics.len() {
            let mut metric = metrics.get_unchecked(i).clone();
            if metric.node_id == node_id {
                metric.consecutive_failures = 0;
                metric.recovery_attempts += 1;
                metric.last_successful_recovery = env.ledger().timestamp();
                metrics.set(i, metric);
                found = true;
                break;
            }
        }

        if !found {
            return Err(Error::NodeNotFound);
        }

        env.storage().persistent().set(&METRICS, &metrics);
        env.events().publish((symbol_short!("FD_REC"),), node_id);
        Ok(())
    }

    pub fn deactivate_failover_plan(
        env: Env,
        caller: Address,
        plan_id: u64,
    ) -> Result<(), Error> {
        Self::require_admin(&env, &caller)?;

        let mut plans: Vec<FailoverPlan> = env
            .storage()
            .persistent()
            .get(&PLANS)
            .unwrap_or_else(|| Vec::new(&env));

        let mut found = false;
        for i in 0..plans.len() {
            let mut plan = plans.get_unchecked(i).clone();
            if plan.plan_id == plan_id {
                plan.is_active = false;
                plans.set(i, plan);
                found = true;
                break;
            }
        }

        if !found {
            return Err(Error::FailoverNotFound);
        }

        env.storage().persistent().set(&PLANS, &plans);
        env.events().publish((symbol_short!("FD_DEAC"),), plan_id);
        Ok(())
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

        let result = FailoverDetector::initialize(env.clone(), admin.clone());
        assert!(result.is_ok());

        let result = FailoverDetector::initialize(env, admin);
        assert!(matches!(result, Err(Error::AlreadyInitialized)));
    }

    #[test]
    fn test_detect_failure() {
        let env = Env::default();
        let admin = Address::random(&env);
        let operator = Address::random(&env);

        FailoverDetector::initialize(env.clone(), admin.clone()).unwrap();
        FailoverDetector::assign_role(env.clone(), admin, operator.clone(), ROLE_OPERATOR).unwrap();

        let result = FailoverDetector::detect_node_failure(
            env.clone(),
            operator,
            1,
            FailoverReason::NodeFailure,
            3,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_failover_plan() {
        let env = Env::default();
        let admin = Address::random(&env);
        let operator = Address::random(&env);

        FailoverDetector::initialize(env.clone(), admin.clone()).unwrap();
        FailoverDetector::assign_role(env.clone(), admin, operator.clone(), ROLE_OPERATOR).unwrap();

        let target_nodes = Vec::new(&env);
        let mut targets = Vec::new(&env);
        targets.push_back(2u32);
        targets.push_back(3u32);

        let result = FailoverDetector::create_failover_plan(env, operator, 1, targets);
        assert!(result.is_ok());
    }
}
