use soroban_sdk::{contracttype, Env, Symbol, symbol_short, Vec};

use crate::{GeoRegion, RegionNode, RegionStatus, DRPolicy};

#[derive(Clone)]
#[contracttype]
pub struct DeploymentScenario {
    pub scenario_id: u32,
    pub name: Symbol,
    pub description: Symbol,
    pub source_region: GeoRegion,
    pub target_regions: Vec<GeoRegion>,
    pub expected_rto_ms: u64,
    pub simulated: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct FailoverScenario {
    pub scenario_id: u32,
    pub name: Symbol,
    pub failure_type: Symbol,
    pub affected_regions: Vec<GeoRegion>,
    pub expected_outcome: Symbol,
    pub recovery_expected: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct SimulationResult {
    pub result_id: u64,
    pub scenario_id: u32,
    pub success: bool,
    pub actual_rto_ms: u64,
    pub regions_affected: u32,
    pub data_loss: bool,
    pub executed_at: u64,
}

pub struct DeploymentSimulator;

impl DeploymentSimulator {
    pub fn create_deployment_scenario(
        env: &Env,
        scenario_id: u32,
        name: Symbol,
        description: Symbol,
        source_region: GeoRegion,
        target_regions: Vec<GeoRegion>,
        expected_rto_ms: u64,
    ) -> DeploymentScenario {
        DeploymentScenario { scenario_id, name, description, source_region, target_regions, expected_rto_ms, simulated: true }
    }

    pub fn create_failover_scenario(
        env: &Env,
        scenario_id: u32,
        name: Symbol,
        failure_type: Symbol,
        affected_regions: Vec<GeoRegion>,
        expected_outcome: Symbol,
        recovery_expected: bool,
    ) -> FailoverScenario {
        FailoverScenario { scenario_id, name, failure_type, affected_regions, expected_outcome, recovery_expected }
    }

    pub fn simulate_deployment(env: &Env, scenario: &DeploymentScenario, regions: &Vec<RegionNode>) -> SimulationResult {
        let mut active_count = 0u32;
        for i in 0..regions.len() {
            if let RegionStatus::Active = regions.get_unchecked(i).status { active_count += 1; }
        }
        let target_count = scenario.target_regions.len();
        let success = active_count >= target_count;
        let actual_rto = if success { scenario.expected_rto_ms / 2 } else { scenario.expected_rto_ms * 2 };
        SimulationResult { result_id: env.ledger().timestamp(), scenario_id: scenario.scenario_id, success, actual_rto_ms: actual_rto, regions_affected: target_count, data_loss: !success, executed_at: env.ledger().timestamp() }
    }

    pub fn simulate_failover(env: &Env, scenario: &FailoverScenario, regions: &Vec<RegionNode>, policy: &DRPolicy) -> SimulationResult {
        let mut healthy_targets = 0u32;
        for i in 0..regions.len() {
            let node = regions.get_unchecked(i);
            if let RegionStatus::Active = node.status {
                for j in 0..scenario.affected_regions.len() {
                    if node.region.clone() as u32 != scenario.affected_regions.get_unchecked(j) as u32 { healthy_targets += 1; break; }
                }
            }
        }
        let success = healthy_targets > 0 && policy.auto_failover_enabled;
        let actual_rto = if success { policy.failover_timeout_ms / 3 } else { policy.failover_timeout_ms * 2 };
        SimulationResult { result_id: env.ledger().timestamp(), scenario_id: scenario.scenario_id, success, actual_rto_ms: actual_rto, regions_affected: scenario.affected_regions.len(), data_loss: !success, executed_at: env.ledger().timestamp() }
    }

    pub fn evaluate_rto_compliance(actual_rto_ms: u64, target_rto_ms: u64) -> bool { actual_rto_ms <= target_rto_ms }

    pub fn count_healthy_regions(regions: &Vec<RegionNode>) -> u32 {
        let mut count = 0u32;
        for i in 0..regions.len() { if let RegionStatus::Active = regions.get_unchecked(i).status { count += 1; } }
        count
    }

    pub fn simulate_cascading_failure(env: &Env, failure_chain: &Vec<GeoRegion>, regions: &mut Vec<RegionNode>) -> Vec<SimulationResult> {
        let mut results = Vec::new(env);
        for i in 0..failure_chain.len() {
            let target_region = failure_chain.get_unchecked(i);
            for j in 0..regions.len() {
                let mut node = regions.get_unchecked(j).clone();
                if node.region.clone() as u32 == target_region as u32 { node.status = RegionStatus::Unavailable; regions.set(j, node); break; }
            }
            let healthy = Self::count_healthy_regions(regions);
            results.push_back(SimulationResult { result_id: env.ledger().timestamp(), scenario_id: i as u32, success: healthy > 0, actual_rto_ms: 60000, regions_affected: (i + 1) as u32, data_loss: healthy == 0, executed_at: env.ledger().timestamp() });
        }
        results
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Env;

    #[test]
    fn test_deployment_scenario_creation() {
        let env = Env::default();
        let targets = Vec::from_array(&env, [GeoRegion::UsWest, GeoRegion::EuCentral]);
        let scenario = DeploymentSimulator::create_deployment_scenario(&env, 1, symbol_short!("SCENARIO1"), symbol_short!("Basic deploy"), GeoRegion::UsEast, targets, 300000);
        assert_eq!(scenario.scenario_id, 1);
        assert!(scenario.simulated);
        assert_eq!(scenario.expected_rto_ms, 300000);
    }

    #[test]
    fn test_failover_scenario_creation() {
        let env = Env::default();
        let affected = Vec::from_array(&env, [GeoRegion::UsEast]);
        let scenario = DeploymentSimulator::create_failover_scenario(&env, 1, symbol_short!("FAILOVER1"), symbol_short!("REGION_DOWN"), affected, symbol_short!("SUCCESS"), true);
        assert_eq!(scenario.scenario_id, 1);
        assert!(scenario.recovery_expected);
    }

    #[test]
    fn test_rto_compliance() {
        assert!(DeploymentSimulator::evaluate_rto_compliance(45000, 300000));
        assert!(!DeploymentSimulator::evaluate_rto_compliance(400000, 300000));
    }

    #[test]
    fn test_count_healthy_regions() {
        let env = Env::default();
        let mut regions = Vec::new(&env);
        regions.push_back(RegionNode { region: GeoRegion::UsEast, node_id: 1, status: RegionStatus::Active, endpoint_hash: 100, last_heartbeat: 0, replica_count: 3, is_primary: true, failure_count: 0 });
        regions.push_back(RegionNode { region: GeoRegion::UsWest, node_id: 2, status: RegionStatus::Unavailable, endpoint_hash: 200, last_heartbeat: 0, replica_count: 2, is_primary: false, failure_count: 1 });
        assert_eq!(DeploymentSimulator::count_healthy_regions(&regions), 1);
    }
}
