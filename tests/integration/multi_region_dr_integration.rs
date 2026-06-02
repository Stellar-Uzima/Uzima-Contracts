// Multi-Region Disaster Recovery System Integration Tests

#[cfg(test)]
mod multi_region_dr_tests {
    use failover_detector::{
        Error as FdError, FailoverDetector, FailoverDetectorClient, FailoverReason, FailoverState,
    };
    use multi_region_orchestrator::{
        DRPolicy, Error as OrchError, GeoRegion, MultiRegionOrchestrator,
        MultiRegionOrchestratorClient,
    };
    use regional_node_manager::{
        Error as RnmError, NodeStatus, RegionalNodeManager, RegionalNodeManagerClient,
    };
    use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};
    use sync_manager::{
        ConsistencyLevel, Error as SmError, SyncManager, SyncManagerClient, SyncStatus,
    };

    #[test]
    fn test_multi_region_deployment() {
        let env = Env::default();
        let admin1 = Address::generate(&env);
        let admin2 = Address::generate(&env);
        let admin3 = Address::generate(&env);
        let admin4 = Address::generate(&env);

        let orch_addr = env.register_contract(None, MultiRegionOrchestrator);
        let fd_addr = env.register_contract(None, FailoverDetector);
        let sm_addr = env.register_contract(None, SyncManager);
        let rnm_addr = env.register_contract(None, RegionalNodeManager);

        let orch = MultiRegionOrchestratorClient::new(&env, &orch_addr);
        let fd = FailoverDetectorClient::new(&env, &fd_addr);
        let sm = SyncManagerClient::new(&env, &sm_addr);
        let rnm = RegionalNodeManagerClient::new(&env, &rnm_addr);

        assert!(orch.try_initialize(&admin1).is_ok());
        assert!(fd.try_initialize(&admin2).is_ok());
        assert!(sm.try_initialize(&admin3).is_ok());
        assert!(rnm.try_initialize(&admin4).is_ok());

        assert_eq!(
            orch.try_initialize(&admin1),
            Err(Ok(OrchError::AlreadyInitialized))
        );
        assert_eq!(
            fd.try_initialize(&admin2),
            Err(Ok(FdError::AlreadyInitialized))
        );
        assert_eq!(
            sm.try_initialize(&admin3),
            Err(Ok(SmError::AlreadyInitialized))
        );
        assert_eq!(
            rnm.try_initialize(&admin4),
            Err(Ok(RnmError::AlreadyInitialized))
        );
    }

    #[test]
    fn test_region_registration_5_regions() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let operator = Address::generate(&env);

        let addr = env.register_contract(None, MultiRegionOrchestrator);
        let client = MultiRegionOrchestratorClient::new(&env, &addr);

        client.initialize(&admin);
        client.assign_role(&admin, &operator, &2u32); // ROLE_OPERATOR=2

        client.register_region(&operator, &GeoRegion::UsEast, &1u32, &10001u64, &true);
        client.register_region(&operator, &GeoRegion::UsWest, &2u32, &10002u64, &false);
        client.register_region(&operator, &GeoRegion::EuCentral, &3u32, &10003u64, &false);
        client.register_region(&operator, &GeoRegion::EuWest, &4u32, &10004u64, &false);
        client.register_region(&operator, &GeoRegion::ApSouth, &5u32, &10005u64, &false);

        let regions = client.list_regions();
        assert_eq!(regions.len(), 5u32, "exactly 5 regions must be registered");

        let mut primary_count = 0u32;
        for i in 0u32..regions.len() {
            if regions.get_unchecked(i).is_primary {
                primary_count += 1;
            }
        }
        assert_eq!(primary_count, 1u32, "exactly 1 primary region");

        assert_eq!(
            client.try_register_region(&operator, &GeoRegion::ApNorth, &6u32, &10006u64, &false),
            Err(Ok(OrchError::MaxRegionsExceeded)),
            "6th region should exceed max_regions=5"
        );
    }

    #[test]
    fn test_automatic_failover_detection() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let operator = Address::generate(&env);

        let addr = env.register_contract(None, FailoverDetector);
        let client = FailoverDetectorClient::new(&env, &addr);

        client.initialize(&admin);
        client.assign_role(&admin, &operator, &2u32); // ROLE_OPERATOR=2

        let reasons = [
            FailoverReason::NodeFailure,
            FailoverReason::HeartbeatTimeout,
            FailoverReason::HighLatency,
            FailoverReason::ResourceExhaustion,
            FailoverReason::DataInconsistency,
        ];

        for reason in &reasons {
            client.detect_node_failure(&operator, &1u32, reason, &3u32);
        }

        let detections = client.get_detections();
        assert_eq!(detections.len(), 5u32, "5 failure detections recorded");

        // Below FAILURE_THRESHOLD=3
        assert!(
            !detections.get_unchecked(0u32).is_critical,
            "detection 1 should not be critical (1 consecutive failure)"
        );
        assert!(
            !detections.get_unchecked(1u32).is_critical,
            "detection 2 should not be critical (2 consecutive failures)"
        );
        // At threshold
        assert!(
            detections.get_unchecked(2u32).is_critical,
            "detection 3 should be critical (3 == FAILURE_THRESHOLD)"
        );

        let metrics = client.get_node_metrics(&1u32).unwrap();
        assert_eq!(
            metrics.consecutive_failures, 5u32,
            "node 1 should have 5 consecutive failures"
        );
    }

    #[test]
    fn test_rto_less_than_15_minutes() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let operator = Address::generate(&env);

        let addr = env.register_contract(None, FailoverDetector);
        let client = FailoverDetectorClient::new(&env, &addr);

        client.initialize(&admin);
        client.assign_role(&admin, &operator, &2u32);

        let detection_id =
            client.detect_node_failure(&operator, &1u32, &FailoverReason::HeartbeatTimeout, &3u32);

        let mut targets = Vec::new(&env);
        targets.push_back(2u32);
        client.create_failover_plan(&operator, &1u32, &targets);

        let execution_id = client.execute_failover(&operator, &detection_id, &2u32);
        assert_eq!(execution_id, 1u64, "first execution should have id=1");

        let executions = client.get_failover_executions();
        let execution = executions.get_unchecked(0u32);

        assert!(
            execution.rto_ms <= 900_000u64,
            "failover RTO {}ms exceeds 15-min SLA of 900_000ms",
            execution.rto_ms
        );
        assert!(
            matches!(execution.state, FailoverState::Completed),
            "execution state should be Completed"
        );

        let metrics = client.get_node_metrics(&1u32).unwrap();
        assert_eq!(
            metrics.consecutive_failures, 0u32,
            "node should have 0 consecutive failures after recovery"
        );
    }

    #[test]
    fn test_99_99_percent_uptime_sla() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let auditor = Address::generate(&env);

        let addr = env.register_contract(None, MultiRegionOrchestrator);
        let client = MultiRegionOrchestratorClient::new(&env, &addr);

        client.initialize(&admin);
        client.assign_role(&admin, &auditor, &4u32); // ROLE_AUDITOR=4

        let windows = [
            (1000u64, 2000u64, 9999u32),
            (2000u64, 3000u64, 9998u32),
            (3000u64, 4000u64, 9999u32),
            (4000u64, 5000u64, 9999u32),
            (5000u64, 6000u64, 9997u32),
        ];

        for (start, end, bp) in windows {
            client.record_uptime_metric(&auditor, &start, &end, &bp, &0u32, &0u64);
        }

        let metrics = client.get_uptime_metrics();
        assert_eq!(metrics.len(), 5u32, "5 uptime windows recorded");

        let mut total = 0u64;
        for i in 0u32..metrics.len() {
            total += metrics.get_unchecked(i).uptime_basis_points as u64;
        }
        let avg = total / 5u64;
        assert!(
            avg >= 9989u64,
            "average uptime {}bp should be within 10bp of 9999 target",
            avg
        );

        let current = client.get_current_uptime();
        assert_eq!(
            current, 9997u32,
            "current uptime should be last recorded value"
        );
    }

    #[test]
    fn test_data_synchronization_across_regions() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let operator = Address::generate(&env);

        let addr = env.register_contract(None, SyncManager);
        let client = SyncManagerClient::new(&env, &addr);

        client.initialize(&admin);
        client.assign_role(&admin, &operator, &2u32);

        let mut targets1 = Vec::new(&env);
        targets1.push_back(2u32);
        targets1.push_back(3u32);
        targets1.push_back(4u32);
        targets1.push_back(5u32);

        let mut targets2 = Vec::new(&env);
        targets2.push_back(1u32);
        targets2.push_back(2u32);
        targets2.push_back(4u32);
        targets2.push_back(5u32);

        let mut targets3 = Vec::new(&env);
        targets3.push_back(6u32);
        targets3.push_back(3u32);
        targets3.push_back(2u32);

        let op_id1 = client.initiate_sync(
            &operator,
            &1u32,
            &targets1,
            &1000u64,
            &ConsistencyLevel::Eventual,
        );
        assert_eq!(op_id1, 1u64, "first sync op should have id=1");
        let op_id2 = client.initiate_sync(
            &operator,
            &3u32,
            &targets2,
            &2000u64,
            &ConsistencyLevel::Strong,
        );
        assert_eq!(op_id2, 2u64, "second sync op should have id=2");
        let op_id3 = client.initiate_sync(
            &operator,
            &5u32,
            &targets3,
            &3000u64,
            &ConsistencyLevel::Causal,
        );
        assert_eq!(op_id3, 3u64, "third sync op should have id=3");

        assert!(client.execute_sync(&operator, &op_id1));
        assert!(client.execute_sync(&operator, &op_id2));
        assert!(client.execute_sync(&operator, &op_id3));

        let ops = client.list_sync_operations();
        assert_eq!(ops.len(), 3u32, "exactly 3 sync operations");
        for i in 0u32..ops.len() {
            let op = ops.get_unchecked(i);
            assert!(
                matches!(op.status, SyncStatus::Completed),
                "op {} should be Completed",
                i
            );
            assert_eq!(op.failure_count, 0u32, "op {} should have no failures", i);
        }

        let empty: Vec<u32> = Vec::new(&env);
        assert_eq!(
            client.try_initiate_sync(
                &operator,
                &1u32,
                &empty,
                &9999u64,
                &ConsistencyLevel::Eventual
            ),
            Err(Ok(SmError::InvalidInput)),
            "empty target list should return InvalidInput"
        );
    }

    #[test]
    fn test_multi_region_failover_workflow() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let operator = Address::generate(&env);

        let addr = env.register_contract(None, FailoverDetector);
        let client = FailoverDetectorClient::new(&env, &addr);

        client.initialize(&admin);
        client.assign_role(&admin, &operator, &2u32);

        // Step 1: 3 consecutive HeartbeatTimeout failures on node 1
        client.detect_node_failure(&operator, &1u32, &FailoverReason::HeartbeatTimeout, &3u32);
        client.detect_node_failure(&operator, &1u32, &FailoverReason::HeartbeatTimeout, &3u32);
        let detection_id =
            client.detect_node_failure(&operator, &1u32, &FailoverReason::HeartbeatTimeout, &3u32);

        // Step 2: 3rd detection is critical
        let detections = client.get_detections();
        assert!(
            detections.get_unchecked(2u32).is_critical,
            "3rd consecutive failure should be critical"
        );

        // Step 3: Create failover plan and verify it is active
        let mut targets = Vec::new(&env);
        targets.push_back(2u32);
        targets.push_back(3u32);
        let _plan_id = client.create_failover_plan(&operator, &1u32, &targets);
        let plans = client.get_failover_plans();
        assert!(plans.get_unchecked(0u32).is_active, "plan should be active");

        // Step 4: Execute failover
        let execution_id = client.execute_failover(&operator, &detection_id, &2u32);
        assert_eq!(execution_id, 1u64, "first execution should have id=1");

        // Step 5: Assert execution details
        let executions = client.get_failover_executions();
        let execution = executions.get_unchecked(0u32);
        assert!(
            matches!(execution.state, FailoverState::Completed),
            "execution state should be Completed"
        );
        assert_eq!(execution.target_node_id, 2u32, "target should be node 2");
        assert_eq!(execution.source_node_id, 1u32, "source should be node 1");

        // Step 6: Node 1 consecutive_failures reset, recovery_attempts incremented
        let metrics = client.get_node_metrics(&1u32).unwrap();
        assert_eq!(metrics.consecutive_failures, 0u32);
        assert_eq!(metrics.recovery_attempts, 1u32);

        // Step 7: RTO within SLA
        assert!(
            execution.rto_ms <= 900_000u64,
            "RTO {}ms exceeds 15-min SLA",
            execution.rto_ms
        );
    }

    #[test]
    fn test_conflict_detection_and_resolution() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let operator = Address::generate(&env);

        let addr = env.register_contract(None, SyncManager);
        let client = SyncManagerClient::new(&env, &addr);

        client.initialize(&admin);
        client.assign_role(&admin, &operator, &2u32);

        let mut targets = Vec::new(&env);
        targets.push_back(3u32);
        targets.push_back(4u32);
        let op_id = client.initiate_sync(
            &operator,
            &1u32,
            &targets,
            &1000u64,
            &ConsistencyLevel::Strong,
        );

        let mut conflicting = Vec::new(&env);
        conflicting.push_back(1u32);
        conflicting.push_back(3u32);
        let conflict_id = client.detect_sync_conflict(&operator, &op_id, &conflicting);
        assert_eq!(conflict_id, 1u64, "first conflict should have id=1");

        let conflicts = client.get_conflicts();
        assert!(
            !conflicts.get_unchecked(0u32).resolved,
            "conflict should not yet be resolved"
        );

        // Resolve with Last-Write-Wins (strategy=1)
        client.resolve_conflict(&operator, &conflict_id, &1u32);

        let conflicts = client.get_conflicts();
        assert!(
            conflicts.get_unchecked(0u32).resolved,
            "conflict should be resolved"
        );
        assert_eq!(
            conflicts.get_unchecked(0u32).resolution_strategy,
            1u32,
            "resolution strategy should be LWW=1"
        );
    }

    #[test]
    fn test_health_monitoring_and_alerting() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let operator = Address::generate(&env);
        let monitor = Address::generate(&env);

        let addr = env.register_contract(None, RegionalNodeManager);
        let client = RegionalNodeManagerClient::new(&env, &addr);

        client.initialize(&admin);
        client.assign_role(&admin, &operator, &2u32); // ROLE_OPERATOR=2
        client.assign_role(&admin, &monitor, &4u32); // ROLE_MONITOR=4

        let node1 = client.register_node(&operator, &String::from_str(&env, "us-east-1"));
        let node2 = client.register_node(&operator, &String::from_str(&env, "us-west-1"));
        let node3 = client.register_node(&operator, &String::from_str(&env, "eu-central-1"));
        let node4 = client.register_node(&operator, &String::from_str(&env, "eu-west-1"));
        let node5 = client.register_node(&operator, &String::from_str(&env, "ap-south-1"));

        client.update_node_metrics(&operator, &node1, &45u32, &60u32, &50u32, &0u64);
        client.update_node_metrics(&operator, &node2, &38u32, &58u32, &51u32, &0u64);
        // eu-central-1: cpu=89 > max_cpu_threshold=85 → Degraded
        client.update_node_metrics(&operator, &node3, &89u32, &60u32, &50u32, &0u64);
        client.update_node_metrics(&operator, &node4, &52u32, &71u32, &64u32, &0u64);
        client.update_node_metrics(&operator, &node5, &41u32, &55u32, &48u32, &0u64);

        let eu_central = client.get_node(&node3).unwrap();
        assert!(
            matches!(eu_central.status, NodeStatus::Degraded),
            "eu-central-1 should be Degraded due to cpu=89 > threshold=85"
        );

        for nid in [node1, node2, node4, node5] {
            let node = client.get_node(&nid).unwrap();
            assert!(
                matches!(node.status, NodeStatus::Healthy),
                "node {} should be Healthy",
                nid
            );
        }

        client.perform_health_check(&monitor, &node1);
        client.perform_health_check(&monitor, &node2);
        client.perform_health_check(&monitor, &node3);
        client.perform_health_check(&monitor, &node4);
        client.perform_health_check(&monitor, &node5);

        assert_eq!(
            client.get_health_checks().len(),
            5u32,
            "5 health checks must be recorded"
        );

        let check = client.get_recent_health_check(&node3).unwrap();
        assert!(
            matches!(check.status, NodeStatus::Degraded),
            "recent health check for eu-central-1 should show Degraded"
        );
        assert_eq!(
            check.cpu_usage, 89u32,
            "cpu_usage in health check should be 89"
        );
    }

    #[test]
    fn test_backup_and_recovery_drills() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let operator = Address::generate(&env);

        let addr = env.register_contract(None, FailoverDetector);
        let client = FailoverDetectorClient::new(&env, &addr);

        client.initialize(&admin);
        client.assign_role(&admin, &operator, &2u32);

        // Drill 1: node 1
        client.detect_node_failure(&operator, &1u32, &FailoverReason::HeartbeatTimeout, &3u32);
        let metrics1 = client.get_node_metrics(&1u32).unwrap();
        assert_eq!(
            metrics1.consecutive_failures, 1u32,
            "node 1 should have 1 consecutive failure"
        );
        client.mark_recovery_success(&operator, &1u32);
        let metrics1 = client.get_node_metrics(&1u32).unwrap();
        assert_eq!(
            metrics1.consecutive_failures, 0u32,
            "node 1 consecutive failures should reset to 0"
        );
        assert_eq!(
            metrics1.recovery_attempts, 1u32,
            "node 1 should have 1 recovery attempt"
        );

        // Drill 2: node 2
        client.detect_node_failure(&operator, &2u32, &FailoverReason::HeartbeatTimeout, &3u32);
        let metrics2 = client.get_node_metrics(&2u32).unwrap();
        assert_eq!(
            metrics2.consecutive_failures, 1u32,
            "node 2 should have 1 consecutive failure"
        );
        client.mark_recovery_success(&operator, &2u32);
        let metrics2 = client.get_node_metrics(&2u32).unwrap();
        assert_eq!(
            metrics2.consecutive_failures, 0u32,
            "node 2 consecutive failures should reset to 0"
        );
        assert_eq!(
            metrics2.recovery_attempts, 1u32,
            "node 2 should have 1 recovery attempt"
        );

        assert_eq!(
            client.get_detections().len(),
            2u32,
            "2 detection records total"
        );
    }

    #[test]
    fn test_integration_with_medical_record_backup() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let operator = Address::generate(&env);

        let addr = env.register_contract(None, RegionalNodeManager);
        let client = RegionalNodeManagerClient::new(&env, &addr);

        client.initialize(&admin);
        client.assign_role(&admin, &operator, &2u32);

        let _primary = client.register_node(&operator, &String::from_str(&env, "us-east-1"));
        let rep1 = client.register_node(&operator, &String::from_str(&env, "us-west-1"));
        let rep2 = client.register_node(&operator, &String::from_str(&env, "eu-central-1"));
        let rep3 = client.register_node(&operator, &String::from_str(&env, "ap-south-1"));

        let data_hash = 0xDEAD_BEEF_u64;
        client.register_replica(&operator, &101u32, &rep1, &data_hash);
        client.register_replica(&operator, &102u32, &rep2, &data_hash);
        client.register_replica(&operator, &103u32, &rep3, &data_hash);

        // Lags all below max_replica_lag_ms=5000
        client.update_replica_sync(&operator, &101u32, &120u64);
        client.update_replica_sync(&operator, &102u32, &340u64);
        client.update_replica_sync(&operator, &103u32, &580u64);

        for (nid, rid) in [(rep1, 101u32), (rep2, 102u32), (rep3, 103u32)] {
            let replicas = client.get_replicas_for_node(&nid);
            assert_eq!(replicas.len(), 1u32);
            let replica = replicas.get_unchecked(0u32);
            assert!(replica.is_in_sync, "replica {} should be in sync", rid);
            assert_eq!(
                replica.data_hash, data_hash,
                "replica {} data_hash mismatch",
                rid
            );
        }

        assert_eq!(
            client.list_nodes().len(),
            4u32,
            "4 nodes total (1 primary + 3 replicas)"
        );
    }

    #[test]
    fn test_security_and_rbac() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let operator = Address::generate(&env);
        let auditor = Address::generate(&env);
        let unauthorized = Address::generate(&env);

        let addr = env.register_contract(None, MultiRegionOrchestrator);
        let client = MultiRegionOrchestratorClient::new(&env, &addr);

        client.initialize(&admin);
        client.assign_role(&admin, &operator, &2u32); // ROLE_OPERATOR=2
        client.assign_role(&admin, &auditor, &4u32); // ROLE_AUDITOR=4

        // Unauthorized calls must fail
        assert_eq!(
            client.try_register_region(&unauthorized, &GeoRegion::UsEast, &1u32, &1000u64, &true),
            Err(Ok(OrchError::NotAuthorized)),
            "unauthorized cannot register region"
        );
        assert_eq!(
            client.try_assign_role(&operator, &unauthorized, &2u32),
            Err(Ok(OrchError::NotAuthorized)),
            "operator cannot assign roles (admin-only)"
        );
        assert_eq!(
            client.try_check_health(&operator),
            Err(Ok(OrchError::NotAuthorized)),
            "operator lacks ROLE_AUDITOR for check_health"
        );
        assert_eq!(
            client.try_set_policy(
                &unauthorized,
                &DRPolicy {
                    min_replicas_per_region: 3u32,
                    max_regions: 5u32,
                    failover_timeout_ms: 300_000u64,
                    sync_interval_ms: 60_000u64,
                    health_check_interval_ms: 30_000u64,
                    auto_failover_enabled: true,
                    rto_target_ms: 900_000u64,
                }
            ),
            Err(Ok(OrchError::NotAuthorized)),
            "unauthorized cannot set policy"
        );

        // Authorized calls must succeed
        let region_id =
            client.register_region(&operator, &GeoRegion::UsEast, &1u32, &1000u64, &true);
        assert!(region_id > 0u32, "register_region should return a valid id");

        // 1 active region < min_replicas=3 → health is false
        let healthy = client.check_health(&auditor);
        assert!(
            !healthy,
            "health should be false with only 1 active region vs min_replicas=3"
        );

        client.set_policy(
            &admin,
            &DRPolicy {
                min_replicas_per_region: 3u32,
                max_regions: 5u32,
                failover_timeout_ms: 300_000u64,
                sync_interval_ms: 60_000u64,
                health_check_interval_ms: 30_000u64,
                auto_failover_enabled: true,
                rto_target_ms: 900_000u64,
            },
        );

        // role_mask=255 > ALL_ROLES=7 → InvalidInput
        assert_eq!(
            client.try_assign_role(&admin, &unauthorized, &255u32),
            Err(Ok(OrchError::InvalidInput)),
            "role_mask=255 exceeds ALL_ROLES=7"
        );
    }
}

#[cfg(test)]
mod performance_tests {
    use failover_detector::{
        FailoverDetector, FailoverDetectorClient, FailoverReason, FailoverState,
    };
    use soroban_sdk::{testutils::Address as _, Address, Env, Vec};
    use sync_manager::{ConsistencyLevel, SyncManager, SyncManagerClient, SyncStatus};

    #[test]
    fn test_failover_performance_metrics() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let operator = Address::generate(&env);

        let addr = env.register_contract(None, FailoverDetector);
        let client = FailoverDetectorClient::new(&env, &addr);

        client.initialize(&admin);
        client.assign_role(&admin, &operator, &2u32);

        let detection_id =
            client.detect_node_failure(&operator, &1u32, &FailoverReason::HeartbeatTimeout, &4u32);

        let mut targets = Vec::new(&env);
        targets.push_back(2u32);
        client.create_failover_plan(&operator, &1u32, &targets);

        let exec_id = client.execute_failover(&operator, &detection_id, &2u32);
        assert_eq!(exec_id, 1u64, "first execution id should be 1");

        let executions = client.get_failover_executions();
        let execution = executions.get_unchecked(0u32);

        assert!(
            execution.rto_ms <= 900_000u64,
            "failover RTO {}ms exceeds 15-min SLA of 900_000ms",
            execution.rto_ms
        );
        assert!(
            matches!(execution.state, FailoverState::Completed),
            "execution state should be Completed"
        );

        // Second failover proves FAILOVER_IN_PROGRESS was reset after first
        let detection_id2 =
            client.detect_node_failure(&operator, &3u32, &FailoverReason::NodeFailure, &4u32);
        let mut targets2 = Vec::new(&env);
        targets2.push_back(4u32);
        client.create_failover_plan(&operator, &3u32, &targets2);
        let exec_id2 = client.execute_failover(&operator, &detection_id2, &4u32);
        assert_eq!(exec_id2, 2u64, "second execution id should be 2");
    }

    #[test]
    fn test_sync_throughput() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let operator = Address::generate(&env);

        let addr = env.register_contract(None, SyncManager);
        let client = SyncManagerClient::new(&env, &addr);

        client.initialize(&admin);
        client.assign_role(&admin, &operator, &2u32);

        // All lags below max_lag_ms=5000
        let lags = [100u64, 500u64, 1000u64, 2000u64];

        for (i, &lag) in lags.iter().enumerate() {
            let target_id = (i as u32) + 2u32;
            let mut targets = Vec::new(&env);
            targets.push_back(target_id);
            let op_id = client.initiate_sync(
                &operator,
                &1u32,
                &targets,
                &((i as u64) * 1000u64 + 1u64),
                &ConsistencyLevel::Eventual,
            );
            assert!(client.execute_sync(&operator, &op_id));
            client.record_replication_lag(&operator, &1u32, &target_id, &lag);
        }

        let ops = client.list_sync_operations();
        assert_eq!(ops.len(), 4u32, "4 sync operations recorded");
        for i in 0u32..ops.len() {
            let op = ops.get_unchecked(i);
            assert!(
                matches!(op.status, SyncStatus::Completed),
                "op {} should be Completed",
                i
            );
            assert_eq!(op.failure_count, 0u32, "op {} should have no failures", i);
        }

        let recorded_lags = client.get_replication_lags();
        assert_eq!(recorded_lags.len(), 4u32, "4 replication lags recorded");
        for i in 0u32..recorded_lags.len() {
            assert!(
                recorded_lags.get_unchecked(i).acceptable,
                "lag {} should be acceptable (< 5000ms)",
                i
            );
        }
    }
}
