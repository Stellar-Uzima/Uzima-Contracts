use soroban_sdk::{contracttype, Env, Symbol, symbol_short, Vec, String};

use crate::{RegionalNode, NodeStatus, NodeConfiguration};

#[derive(Clone)]
#[contracttype]
pub struct NodeFailureScenario {
    pub scenario_id: u32,
    pub name: Symbol,
    pub failure_type: Symbol,
    pub affected_node_id: u32,
    pub expected_degradation: bool,
    pub recovery_time_ms: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct HealthCheckSimulation {
    pub check_id: u64,
    pub node_id: u32,
    pub simulated_cpu: u32,
    pub simulated_memory: u32,
    pub simulated_disk: u32,
    pub expected_status: NodeStatus,
}

#[derive(Clone)]
#[contracttype]
pub struct ReplicationSimulation {
    pub source_node_id: u32,
    pub target_node_ids: Vec<u32>,
    pub data_hash: u64,
    pub expected_lag_ms: u64,
    pub sync_expected: bool,
}

pub struct NodeSimulator;

impl NodeSimulator {
    pub fn create_failure_scenario(env: &Env, scenario_id: u32, name: Symbol, failure_type: Symbol, affected_node_id: u32, expected_degradation: bool, recovery_time_ms: u64) -> NodeFailureScenario {
        NodeFailureScenario { scenario_id, name, failure_type, affected_node_id, expected_degradation, recovery_time_ms }
    }

    pub fn simulate_health_check(env: &Env, node: &RegionalNode, config: &NodeConfiguration) -> HealthCheckSimulation {
        let expected_status = if node.cpu_usage_percent > config.max_cpu_threshold || node.memory_usage_percent > config.max_memory_threshold || node.disk_usage_percent > config.max_disk_threshold {
            NodeStatus::Degraded
        } else if node.replica_lag_ms > config.max_replica_lag_ms {
            NodeStatus::Degraded
        } else {
            NodeStatus::Healthy
        };
        HealthCheckSimulation { check_id: env.ledger().timestamp(), node_id: node.node_id, simulated_cpu: node.cpu_usage_percent, simulated_memory: node.memory_usage_percent, simulated_disk: node.disk_usage_percent, expected_status }
    }

    pub fn simulate_node_failure(env: &Env, scenario: &NodeFailureScenario, nodes: &Vec<RegionalNode>) -> (bool, Symbol) {
        let mut found = false;
        for i in 0..nodes.len() { if nodes.get_unchecked(i).node_id == scenario.affected_node_id { found = true; break; } }
        if !found { return (false, symbol_short!("NODE_NF")); }
        let total_nodes = nodes.len();
        if total_nodes <= 1 { return (false, symbol_short!("LAST_NODE")); }
        (true, symbol_short!("RECOVER"))
    }

    pub fn simulate_replication(env: &Env, source: &RegionalNode, targets: &Vec<RegionalNode>, config: &NodeConfiguration) -> ReplicationSimulation {
        let mut target_ids = Vec::new(env);
        for i in 0..targets.len() { target_ids.push_back(targets.get_unchecked(i).node_id); }
        let sync_expected = source.status == NodeStatus::Healthy && source.replica_lag_ms <= config.max_replica_lag_ms;
        ReplicationSimulation { source_node_id: source.node_id, target_node_ids: target_ids, data_hash: env.ledger().timestamp(), expected_lag_ms: config.max_replica_lag_ms / 2, sync_expected }
    }

    pub fn simulate_load_balancing(env: &Env, nodes: &Vec<RegionalNode>) -> Vec<u32> {
        let mut healthy_ids = Vec::new(env);
        for i in 0..nodes.len() {
            let node = nodes.get_unchecked(i);
            if node.status == NodeStatus::Healthy && node.cpu_usage_percent < 80 { healthy_ids.push_back(node.node_id); }
        }
        healthy_ids
    }

    pub fn evaluate_threshold_breach(cpu: u32, memory: u32, disk: u32, config: &NodeConfiguration) -> (bool, bool, bool) {
        (cpu > config.max_cpu_threshold, memory > config.max_memory_threshold, disk > config.max_disk_threshold)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Env;

    #[test]
    fn test_failure_scenario_creation() {
        let env = Env::default();
        let scenario = NodeSimulator::create_failure_scenario(&env, 1, symbol_short!("NODE_FAIL"), symbol_short!("CPU_HIGH"), 1, true, 30000);
        assert_eq!(scenario.scenario_id, 1);
        assert!(scenario.expected_degradation);
        assert_eq!(scenario.recovery_time_ms, 30000);
    }

    #[test]
    fn test_threshold_breach() {
        let config = NodeConfiguration { max_cpu_threshold: 85, max_memory_threshold: 80, max_disk_threshold: 90, max_replica_lag_ms: 5000, heartbeat_timeout_ms: 30000, health_check_interval_ms: 10000 };
        let (cpu, mem, disk) = NodeSimulator::evaluate_threshold_breach(50, 60, 40, &config);
        assert!(!cpu); assert!(!mem); assert!(!disk);
        let (cpu, mem, disk) = NodeSimulator::evaluate_threshold_breach(90, 85, 95, &config);
        assert!(cpu); assert!(mem); assert!(disk);
    }

    #[test]
    fn test_load_balancing() {
        let env = Env::default();
        let mut nodes = Vec::new(&env);
        nodes.push_back(RegionalNode { node_id: 1, region_name: String::from_bytes(&env, &b"us-east-1"[..]), status: NodeStatus::Healthy, cpu_usage_percent: 50, memory_usage_percent: 60, disk_usage_percent: 40, last_heartbeat: 0, replica_lag_ms: 100, total_uptime_ms: 86400000, failure_count: 0 });
        nodes.push_back(RegionalNode { node_id: 2, region_name: String::from_bytes(&env, &b"us-west-1"[..]), status: NodeStatus::Degraded, cpu_usage_percent: 90, memory_usage_percent: 85, disk_usage_percent: 70, last_heartbeat: 0, replica_lag_ms: 6000, total_uptime_ms: 86400000, failure_count: 2 });
        let healthy = NodeSimulator::simulate_load_balancing(&env, &nodes);
        assert_eq!(healthy.len(), 1);
        assert_eq!(healthy.get_unchecked(0), 1);
    }
}
