# Medical Record Backup and Disaster Recovery

Contract: `contracts/medical_record_backup`

## Core capabilities
- Automated backup schedule state (`run_scheduled_backup`, `get_schedule`) plus manual execution (`run_backup_now`)
- Multi-target redundancy across networks and geo regions (`register_target`, `set_policy`)
- Integrity verification (`verify_backup_integrity`) and restore workflows (`request_restore`, `approve_restore`, `execute_restore`)
- Recovery drills (`run_recovery_test`)
- Cost-aware retention cleanup (`optimize_and_cleanup`)
- Monitoring and alerting (`list_alerts`, `report_target_failure`, `get_health`)
- Access controls via role masks (operator, auditor, recovery) and admin override

## Design notes
- Backup artifacts only store hashes and encrypted references (`snapshot_ref`) to avoid on-chain PHI leakage.
- Geo-resilience is enforced by `min_region_count` and `min_targets_per_backup`.
- Encryption enforcement is controlled by policy (`encryption_required` + non-zero key version).
