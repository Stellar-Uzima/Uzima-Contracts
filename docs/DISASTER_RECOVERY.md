# Disaster Recovery Procedures (Issue #422)

Comprehensive runbooks, recovery scripts, and testing schedules for Uzima smart contract failures.

---

## Table of Contents

1. [Overview](#overview)
2. [Recovery Time Objectives](#recovery-time-objectives)
3. [Contact Escalation Matrix](#contact-escalation-matrix)
4. [Runbook 1: Contract Pause Emergency](#runbook-1-contract-pause-emergency)
5. [Runbook 2: State Backup and Restore](#runbook-2-state-backup-and-restore)
6. [Runbook 3: Fund Recovery](#runbook-3-fund-recovery)
7. [Runbook 4: Data Migration](#runbook-4-data-migration)
8. [Runbook 5: Incident Response](#runbook-5-incident-response)
9. [Automation](#automation)
10. [DR Drill Schedule](#dr-drill-schedule)

---

## Overview

This document defines the disaster recovery (DR) procedures for the Uzima smart contract system deployed on Stellar/Soroban. All procedures assume the operator has:

- Access to the deployer identity (`soroban config identity`)
- The `soroban` CLI installed (v21.7.7+)
- Network access to the target Stellar network
- The contract IDs stored in `deployments/`

---

## Recovery Time Objectives

| Scenario | RTO | RPO | Priority |
|----------|-----|-----|----------|
| Contract pause (emergency stop) | 5 min | 0 (no data loss) | P0 |
| Critical bug – rollback to previous WASM | 15 min | 0 | P0 |
| State corruption – restore from backup | 2 hours | Last backup | P1 |
| Fund recovery (stuck escrow) | 4 hours | 0 | P1 |
| Full data migration (schema change) | 8 hours | 0 | P2 |
| Multi-region failover | 30 min | Last sync | P1 |

---

## Contact Escalation Matrix

| Level | Role | Trigger | Action |
|-------|------|---------|--------|
| L1 | On-call Engineer | Any alert | Triage, apply runbook |
| L2 | Senior Engineer | P0/P1 unresolved > 15 min | Escalate, coordinate |
| L3 | Engineering Lead | P0 unresolved > 30 min | Executive decision |
| L4 | Security Team | Suspected exploit | Immediate pause + forensics |

Notification channels:
- PagerDuty: `uzima-contracts-oncall`
- Slack: `#uzima-incidents`
- Email: `security@uzima.health` (P0 only)

---

## Runbook 1: Contract Pause Emergency

**Trigger**: Critical bug detected, exploit in progress, or unexpected behavior.

### Step 1 – Identify the affected contract

```bash
# List all deployed contracts
./scripts/deployment_status.sh <network>

# Check contract health
./scripts/monitor_deployments.sh <network>
```

### Step 2 – Execute emergency pause

```bash
# One-click pause (uses upgradeability freeze mechanism)
./scripts/dr/emergency_pause.sh <contract_id> <network>
```

The script will:
1. Verify admin identity
2. Call the contract's freeze function via `soroban contract invoke`
3. Confirm the frozen state
4. Log the action to `deployments/incident_log.json`

### Step 3 – Notify stakeholders

```bash
# Generate incident report
./scripts/dr/generate_incident_report.sh <contract_id> <network> "Emergency pause"
```

### Step 4 – Investigate and resolve

- Review recent transactions on [Stellar Expert](https://stellar.expert/)
- Check contract logs
- Prepare fix or rollback

### Step 5 – Resume (after fix)

```bash
# Deploy fix and unfreeze
./scripts/dr/resume_contract.sh <contract_id> <network> <new_wasm_hash>
```

---

## Runbook 2: State Backup and Restore

**Trigger**: State corruption detected, or pre-migration backup required.

### Backup

```bash
# Create a state snapshot (stores hashes + encrypted refs, no PHI on-chain)
./scripts/dr/backup_state.sh <contract_id> <network>

# Verify backup integrity
./scripts/dr/verify_backup.sh <contract_id> <network>
```

Backups are stored in `deployments/<network>_<contract>_backup_<timestamp>.json`.

### Restore

```bash
# List available backups
ls deployments/<network>_<contract>_backup_*.json

# Restore from a specific backup
./scripts/dr/restore_state.sh <contract_id> <network> <backup_file>
```

The restore process:
1. Pauses the contract
2. Verifies backup integrity
3. Applies state via migration transaction
4. Runs integrity checks
5. Resumes the contract

---

## Runbook 3: Fund Recovery

**Trigger**: Funds stuck in escrow, payment router failure, or treasury anomaly.

### Step 1 – Identify stuck funds

```bash
soroban contract invoke \
  --id <escrow_contract_id> \
  --network <network> \
  --source <admin_identity> \
  -- list_pending_releases
```

### Step 2 – Emergency release (admin override)

```bash
soroban contract invoke \
  --id <escrow_contract_id> \
  --network <network> \
  --source <admin_identity> \
  -- emergency_release \
  --escrow-id <id> \
  --reason "DR: stuck funds recovery"
```

### Step 3 – Verify and document

```bash
# Confirm funds released
soroban contract invoke \
  --id <escrow_contract_id> \
  --network <network> \
  --source <admin_identity> \
  -- get_escrow \
  --escrow-id <id>

# Log recovery action
./scripts/dr/log_recovery_action.sh "fund_recovery" <contract_id> <network>
```

---

## Runbook 4: Data Migration

**Trigger**: Schema change requiring on-chain data transformation.

### Pre-migration checklist

- [ ] New WASM built and tested locally
- [ ] Migration script reviewed by ≥ 2 engineers
- [ ] State backup completed
- [ ] Rollback plan documented
- [ ] Maintenance window communicated

### Migration steps

```bash
# 1. Install new WASM
soroban contract install \
  --network <network> \
  --source <deployer_identity> \
  --wasm target/wasm32-unknown-unknown/release/<contract>.wasm

# 2. Validate upgrade (dry-run)
./scripts/dr/validate_upgrade.sh <contract_id> <new_wasm_hash> <network>

# 3. Execute upgrade with migration
./scripts/deploy_with_rollback.sh <contract_name> <network>

# 4. Run post-migration integrity checks
./scripts/dr/post_migration_check.sh <contract_id> <network>

# 5. Verify new functionality
cargo test -p <contract_name> -- --test-threads=1
```

### Rollback (if migration fails)

```bash
./scripts/rollback_deployment.sh <contract_name> <network>
```

---

## Runbook 5: Incident Response

**Trigger**: Security incident, exploit, or data breach.

### Immediate actions (first 5 minutes)

1. **Pause all affected contracts**:
   ```bash
   ./scripts/dr/emergency_pause_all.sh <network>
   ```

2. **Preserve evidence** (do not modify logs):
   ```bash
   ./scripts/dr/capture_forensics.sh <network> <incident_id>
   ```

3. **Notify security team**:
   ```bash
   ./scripts/dr/notify_security.sh <incident_id> <severity>
   ```

### Investigation (5–60 minutes)

```bash
# Analyze recent transactions
./scripts/dr/analyze_transactions.sh <contract_id> <network> --last 1000

# Check for anomalies
soroban contract invoke \
  --id <anomaly_detector_id> \
  --network <network> \
  --source <admin_identity> \
  -- get_recent_anomalies \
  --limit 50
```

### Recovery

1. Deploy patched WASM
2. Run full test suite
3. Resume contracts
4. Monitor for 24 hours
5. Publish post-mortem within 72 hours

---

## Automation

### Automated Backups

Backups run automatically via the `medical_record_backup` contract's scheduled backup mechanism. The CI/CD pipeline also triggers backups on every deployment.

Manual trigger:
```bash
./scripts/dr/backup_state.sh all <network>
```

### Health Monitoring

```bash
# Continuous monitoring (runs every 30s)
./scripts/monitor_deployments.sh <network> --alert-on-failure

# One-time health check
./scripts/dr/health_check.sh <network>
```

### One-Click Pause

```bash
# Pause a single contract
./scripts/dr/emergency_pause.sh <contract_id> <network>

# Pause all contracts (nuclear option)
./scripts/dr/emergency_pause_all.sh <network>
```

---

## DR Drill Schedule

| Drill | Frequency | Scope | Owner |
|-------|-----------|-------|-------|
| Contract pause + resume | Monthly | Testnet | On-call Engineer |
| State backup + restore | Quarterly | Testnet | Senior Engineer |
| Fund recovery simulation | Quarterly | Testnet | Engineering Lead |
| Full failover simulation | Semi-annually | Staging | Engineering Lead |
| Security incident simulation | Annually | Staging | Security Team |

### Running a DR Drill

```bash
# Run a specific drill scenario
./scripts/dr/run_drill.sh <scenario> <network>

# Available scenarios:
#   pause_resume      - Contract pause and resume
#   backup_restore    - State backup and restore
#   fund_recovery     - Stuck fund recovery
#   rollback          - WASM rollback
#   full_failover     - Multi-region failover

# Validate drill results
./scripts/dr/validate_drill.sh <drill_id>
```

Drill results are logged to `deployments/dr_drills/` and reviewed in the monthly engineering sync.

---

## Related Documentation

- [Medical Record Backup Contract](./MEDICAL_BACKUP_DISASTER_RECOVERY.md)
- [Multi-Region DR Architecture](../config/multi_region_dr.json)
- [Deployment Scripts](../scripts/)
- [Upgradeability System](./upgradeability.md)
- [Monitoring](./MONITORING.md)
