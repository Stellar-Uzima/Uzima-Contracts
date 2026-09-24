#![no_std]
#![forbid(alloc)]
//! failover_detector - Healthcare smart contract on Stellar blockchain.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Map, Symbol,
    Vec,
};

// ============================================================================
// Data Types & Constants
// ============================================================================

const ROLE_ADMIN: u32 = 1;
const ROLE_OPERATOR: u32 = 2;
const ALL_ROLES: u32 = 3;

const FAILURE_THRESHOLD: u32 = 3; // Trigger after 3 consecutive failures

#[derive(Clone, Copy)]
#[contracttype]
pub enum FailoverReason {
    NodeFailure = 0,
    HeartbeatTimeout = 1,
    HighLatency = 2,
    ResourceExhaustion = 3,
    DataInconsistency = 4,
    ManualTrigger = 5,
}

#[derive(Clone, Copy)]
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

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::AlreadyInitialized => write!(f, "already initialized"),
            Error::NotInitialized => write!(f, "not initialized"),
            Error::NotAuthorized => write!(f, "not authorized"),
            Error::InvalidInput => write!(f, "invalid input"),
            Error::NodeNotFound => write!(f, "node not found"),
            Error::FailoverNotFound => write!(f, "failover not found"),
            Error::NoAvailableTargets => write!(f, "no available targets"),
            Error::FailoverInProgress => write!(f, "failover in progress"),
            Error::MaxFailuresReached => write!(f, "max failures reached"),
            Error::RecoveryFailed => write!(f, "recovery failed"),
        }
    }
}

// ============================================================================
// Storage Keys
// ============================================================================

const ADMIN: Symbol = symbol_short!("ADMIN");
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
        governance_commons::try_init_guard(&env).map_err(|_| Error::AlreadyInitialized)?;

        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&NEXT_DETECTION_ID, &1u64);
        env.storage().instance().set(&NEXT_EXECUTION_ID, &1u64);
        env.storage().instance().set(&NEXT_PLAN_ID, &1u64);
        env.storage().instance().set(&FAILOVER_IN_PROGRESS, &false);

        env.events().publish((symbol_short!("FD_INIT"),), admin);
        Ok(())
    }

    pub fn assign_role(
        env: Env,
        caller: Address,
        user: Address,
        role_mask: u32,
    ) -> Result<(), Error> {
        Self::require_admin(&env, &caller)?;
        if role_mask > ALL_ROLES {
            return Err(Error::InvalidInput);
        }

        let mut roles: Map<Address, u32> = env
            .storage()
            .instance()
            .get(&ROLES)
            .unwrap_or_else(|| Map::new(&env));
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

        if !(1..=5).contains(&severity_level) {
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
        let mut found_index: Option<u32> = None;

        for i in 0u32..metrics.len() {
            if metrics.get_unchecked(i).node_id == node_id {
                node_metric = Some(metrics.get_unchecked(i).clone());
                found_index = Some(i);
                break;
            }
        }

        let current_failures = if let Some(metric) = &node_metric {
            metric.consecutive_failures + 1
        } else {
            1
        };

        let is_critical = current_failures >= FAILURE_THRESHOLD;

        let (total_failures, recovery_attempts, last_successful_recovery) =
            if let Some(metric) = &node_metric {
                (
                    metric.total_failures + 1,
                    metric.recovery_attempts,
                    metric.last_successful_recovery,
                )
            } else {
                (1, 0, 0)
            };

        let updated_metric = NodeFailureMetric {
            node_id,
            consecutive_failures: current_failures,
            last_failure_at: env.ledger().timestamp(),
            total_failures,
            recovery_attempts,
            last_successful_recovery,
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

        env.events()
            .publish((symbol_short!("FD_DETC"),), detection_id);

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

        if target_nodes.is_empty() {
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

        env.storage().instance().set(&FAILOVER_IN_PROGRESS, &true);

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

        env.storage().instance().set(&FAILOVER_IN_PROGRESS, &false);

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

        env.events()
            .publish((symbol_short!("FD_EXEC"),), execution_id);
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

    pub fn mark_recovery_success(env: Env, caller: Address, node_id: u32) -> Result<(), Error> {
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

    pub fn deactivate_failover_plan(env: Env, caller: Address, plan_id: u64) -> Result<(), Error> {
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

    #[must_use]
    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&ADMIN)
            .ok_or(Error::NotInitialized)?;
        if admin != *caller {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    #[must_use]
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
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_initialize() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let contract = env.register_contract(None, FailoverDetector);

        let result = env.as_contract(&contract, || {
            FailoverDetector::initialize(env.clone(), admin.clone())
        });
        assert!(result.is_ok());

        let result = env.as_contract(&contract, || {
            FailoverDetector::initialize(env.clone(), admin)
        });
        assert!(matches!(result, Err(Error::AlreadyInitialized)));
    }

    #[test]
    fn test_detect_failure() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let operator = Address::generate(&env);
        let contract = env.register_contract(None, FailoverDetector);

        env.as_contract(&contract, || {
            FailoverDetector::initialize(env.clone(), admin.clone())
        })
        .unwrap();
        env.as_contract(&contract, || {
            FailoverDetector::assign_role(env.clone(), admin, operator.clone(), ROLE_OPERATOR)
        })
        .unwrap();

        let result = env.as_contract(&contract, || {
            FailoverDetector::detect_node_failure(
                env.clone(),
                operator,
                1,
                FailoverReason::NodeFailure,
                3,
            )
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_failover_plan() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let operator = Address::generate(&env);
        let contract = env.register_contract(None, FailoverDetector);

        env.as_contract(&contract, || {
            FailoverDetector::initialize(env.clone(), admin.clone())
        })
        .unwrap();
        env.as_contract(&contract, || {
            FailoverDetector::assign_role(env.clone(), admin, operator.clone(), ROLE_OPERATOR)
        })
        .unwrap();

        let mut targets = Vec::new(&env);
        targets.push_back(2u32);
        targets.push_back(3u32);

        let result = env.as_contract(&contract, || {
            FailoverDetector::create_failover_plan(env.clone(), operator, 1, targets)
        });
        assert!(result.is_ok());
    }

    // ========================================================================
    // Setup helper
    // ========================================================================

    /// Initializes the contract and assigns the operator role to `operator`.
    /// Returns the contract address so callers can wrap further calls in
    /// `env.as_contract`.
    fn setup(env: &Env) -> (Address, Address, Address) {
        let admin = Address::generate(env);
        let operator = Address::generate(env);
        let contract = env.register_contract(None, FailoverDetector);

        env.as_contract(&contract, || {
            FailoverDetector::initialize(env.clone(), admin.clone())
        })
        .unwrap();
        env.as_contract(&contract, || {
            FailoverDetector::assign_role(env.clone(), admin.clone(), operator.clone(), ROLE_OPERATOR)
        })
        .unwrap();

        (contract, admin, operator)
    }

    // ========================================================================
    // assign_role
    // ========================================================================

    #[test]
    fn test_assign_role_rejects_invalid_role_mask() {
        let env = Env::default();
        let (contract, admin, _operator) = setup(&env);
        let stranger = Address::generate(&env);

        let result = env.as_contract(&contract, || {
            FailoverDetector::assign_role(env.clone(), admin, stranger, ALL_ROLES + 1)
        });
        assert!(matches!(result, Err(Error::InvalidInput)));
    }

    #[test]
    fn test_assign_role_rejects_non_admin_caller() {
        let env = Env::default();
        let (contract, _admin, operator) = setup(&env);
        let stranger = Address::generate(&env);

        // `operator` has the operator role but not admin, so it must not be
        // able to grant roles to others.
        let result = env.as_contract(&contract, || {
            FailoverDetector::assign_role(env.clone(), operator, stranger, ROLE_OPERATOR)
        });
        assert!(matches!(result, Err(Error::NotAuthorized)));
    }

    // ========================================================================
    // detect_node_failure
    // ========================================================================

    #[test]
    fn test_detect_node_failure_rejects_unauthorized_caller() {
        let env = Env::default();
        let (contract, _admin, _operator) = setup(&env);
        let stranger = Address::generate(&env);

        let result = env.as_contract(&contract, || {
            FailoverDetector::detect_node_failure(
                env.clone(),
                stranger,
                1,
                FailoverReason::NodeFailure,
                3,
            )
        });
        assert!(matches!(result, Err(Error::NotAuthorized)));
    }

    #[test]
    fn test_detect_node_failure_rejects_out_of_range_severity() {
        let env = Env::default();
        let (contract, _admin, operator) = setup(&env);

        for invalid_severity in [0u32, 6u32] {
            let result = env.as_contract(&contract, || {
                FailoverDetector::detect_node_failure(
                    env.clone(),
                    operator.clone(),
                    1,
                    FailoverReason::NodeFailure,
                    invalid_severity,
                )
            });
            assert!(matches!(result, Err(Error::InvalidInput)));
        }
    }

    /// Table-driven check of the FAILURE_THRESHOLD (3) invariant: a node's
    /// `is_critical` detection flag must stay false for the first two
    /// consecutive failures and flip true from the third onward, with
    /// `consecutive_failures` tracking the call count exactly.
    #[test]
    fn test_detect_node_failure_critical_threshold_invariant() {
        let env = Env::default();
        let (contract, _admin, operator) = setup(&env);
        let node_id = 42u32;

        let expected_is_critical = [false, false, true, true];

        for (i, expected_critical) in expected_is_critical.iter().enumerate() {
            env.as_contract(&contract, || {
                FailoverDetector::detect_node_failure(
                    env.clone(),
                    operator.clone(),
                    node_id,
                    FailoverReason::HeartbeatTimeout,
                    3,
                )
            })
            .unwrap();

            let metric = env
                .as_contract(&contract, || {
                    FailoverDetector::get_node_metrics(env.clone(), node_id)
                })
                .unwrap();
            assert_eq!(metric.consecutive_failures, (i as u32) + 1);

            let detections = env.as_contract(&contract, || {
                FailoverDetector::get_detections(env.clone())
            });
            let last = detections.get_unchecked(detections.len() - 1);
            assert_eq!(last.is_critical, *expected_critical);
        }
    }

    #[test]
    fn test_get_node_metrics_returns_none_for_unknown_node() {
        let env = Env::default();
        let (contract, _admin, _operator) = setup(&env);

        let result = env.as_contract(&contract, || {
            FailoverDetector::get_node_metrics(env.clone(), 999)
        });
        assert!(result.is_none());
    }

    // ========================================================================
    // create_failover_plan
    // ========================================================================

    #[test]
    fn test_create_failover_plan_rejects_empty_targets() {
        let env = Env::default();
        let (contract, _admin, operator) = setup(&env);
        let targets = Vec::new(&env);

        let result = env.as_contract(&contract, || {
            FailoverDetector::create_failover_plan(env.clone(), operator, 1, targets)
        });
        assert!(matches!(result, Err(Error::NoAvailableTargets)));
    }

    #[test]
    fn test_create_failover_plan_rejects_unauthorized_caller() {
        let env = Env::default();
        let (contract, _admin, _operator) = setup(&env);
        let stranger = Address::generate(&env);
        let mut targets = Vec::new(&env);
        targets.push_back(2u32);

        let result = env.as_contract(&contract, || {
            FailoverDetector::create_failover_plan(env.clone(), stranger, 1, targets)
        });
        assert!(matches!(result, Err(Error::NotAuthorized)));
    }

    // ========================================================================
    // execute_failover
    // ========================================================================

    #[test]
    fn test_execute_failover_rejects_unknown_detection() {
        let env = Env::default();
        let (contract, _admin, operator) = setup(&env);

        let result = env.as_contract(&contract, || {
            FailoverDetector::execute_failover(env.clone(), operator, 999, 2)
        });
        assert!(matches!(result, Err(Error::FailoverNotFound)));
    }

    #[test]
    fn test_execute_failover_rejects_when_already_in_progress() {
        let env = Env::default();
        let (contract, _admin, operator) = setup(&env);

        let detection_id = env
            .as_contract(&contract, || {
                FailoverDetector::detect_node_failure(
                    env.clone(),
                    operator.clone(),
                    1,
                    FailoverReason::NodeFailure,
                    3,
                )
            })
            .unwrap();

        // Directly flip the internal in-progress flag to simulate a
        // concurrent failover, since execute_failover clears it again
        // before returning within a single call.
        env.as_contract(&contract, || {
            env.storage().instance().set(&FAILOVER_IN_PROGRESS, &true);
        });

        let result = env.as_contract(&contract, || {
            FailoverDetector::execute_failover(env.clone(), operator, detection_id, 2)
        });
        assert!(matches!(result, Err(Error::FailoverInProgress)));
    }

    #[test]
    fn test_execute_failover_happy_path_resets_consecutive_failures() {
        let env = Env::default();
        let (contract, _admin, operator) = setup(&env);
        let node_id = 7u32;

        // Two failures short of the critical threshold; execute_failover
        // should reset the counter back to zero on success.
        for _ in 0..2 {
            env.as_contract(&contract, || {
                FailoverDetector::detect_node_failure(
                    env.clone(),
                    operator.clone(),
                    node_id,
                    FailoverReason::NodeFailure,
                    2,
                )
            })
            .unwrap();
        }

        let detections = env.as_contract(&contract, || {
            FailoverDetector::get_detections(env.clone())
        });
        let detection_id = detections.get_unchecked(detections.len() - 1).detection_id;

        let execution_id = env
            .as_contract(&contract, || {
                FailoverDetector::execute_failover(env.clone(), operator.clone(), detection_id, 9)
            })
            .unwrap();
        assert!(execution_id > 0);

        let executions = env.as_contract(&contract, || {
            FailoverDetector::get_failover_executions(env.clone())
        });
        assert_eq!(executions.len(), 1);
        assert!(matches!(
            executions.get_unchecked(0).state,
            FailoverState::Completed
        ));

        let metric = env
            .as_contract(&contract, || {
                FailoverDetector::get_node_metrics(env.clone(), node_id)
            })
            .unwrap();
        assert_eq!(metric.consecutive_failures, 0);
        assert_eq!(metric.recovery_attempts, 1);
    }

    // ========================================================================
    // mark_recovery_success
    // ========================================================================

    #[test]
    fn test_mark_recovery_success_rejects_unknown_node() {
        let env = Env::default();
        let (contract, _admin, operator) = setup(&env);

        let result = env.as_contract(&contract, || {
            FailoverDetector::mark_recovery_success(env.clone(), operator, 999)
        });
        assert!(matches!(result, Err(Error::NodeNotFound)));
    }

    #[test]
    fn test_mark_recovery_success_resets_consecutive_failures() {
        let env = Env::default();
        let (contract, _admin, operator) = setup(&env);
        let node_id = 5u32;

        env.as_contract(&contract, || {
            FailoverDetector::detect_node_failure(
                env.clone(),
                operator.clone(),
                node_id,
                FailoverReason::HighLatency,
                4,
            )
        })
        .unwrap();

        env.as_contract(&contract, || {
            FailoverDetector::mark_recovery_success(env.clone(), operator, node_id)
        })
        .unwrap();

        let metric = env
            .as_contract(&contract, || {
                FailoverDetector::get_node_metrics(env.clone(), node_id)
            })
            .unwrap();
        assert_eq!(metric.consecutive_failures, 0);
        assert_eq!(metric.recovery_attempts, 1);
    }

    // ========================================================================
    // deactivate_failover_plan
    // ========================================================================

    #[test]
    fn test_deactivate_failover_plan_rejects_non_admin_caller() {
        let env = Env::default();
        let (contract, _admin, operator) = setup(&env);
        let mut targets = Vec::new(&env);
        targets.push_back(2u32);

        let plan_id = env
            .as_contract(&contract, || {
                FailoverDetector::create_failover_plan(env.clone(), operator.clone(), 1, targets)
            })
            .unwrap();

        // Operators may create plans, but only admin may deactivate one.
        let result = env.as_contract(&contract, || {
            FailoverDetector::deactivate_failover_plan(env.clone(), operator, plan_id)
        });
        assert!(matches!(result, Err(Error::NotAuthorized)));
    }

    #[test]
    fn test_deactivate_failover_plan_rejects_unknown_plan() {
        let env = Env::default();
        let (contract, admin, _operator) = setup(&env);

        let result = env.as_contract(&contract, || {
            FailoverDetector::deactivate_failover_plan(env.clone(), admin, 999)
        });
        assert!(matches!(result, Err(Error::FailoverNotFound)));
    }

    #[test]
    fn test_deactivate_failover_plan_happy_path() {
        let env = Env::default();
        let (contract, admin, operator) = setup(&env);
        let mut targets = Vec::new(&env);
        targets.push_back(2u32);

        let plan_id = env
            .as_contract(&contract, || {
                FailoverDetector::create_failover_plan(env.clone(), operator, 1, targets)
            })
            .unwrap();

        env.as_contract(&contract, || {
            FailoverDetector::deactivate_failover_plan(env.clone(), admin, plan_id)
        })
        .unwrap();

        let plans = env.as_contract(&contract, || {
            FailoverDetector::get_failover_plans(env.clone())
        });
        let plan = plans.get_unchecked(plans.len() - 1);
        assert_eq!(plan.plan_id, plan_id);
        assert!(!plan.is_active);
    }
}
