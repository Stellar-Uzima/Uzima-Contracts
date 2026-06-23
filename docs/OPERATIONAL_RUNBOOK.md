# Operational Runbook

This runbook covers common operational tasks for Uzima Contracts in production.

---

## Key Rotation

### Admin Key Rotation

1. Generate a new identity:
   ```bash
   soroban config identity generate new-admin
   soroban config identity address new-admin
   ```

2. Transfer admin role on each contract (example for `medical_records`):
   ```bash
   soroban contract invoke \
     --id <CONTRACT_ID> \
     --source current-admin \
     --network testnet \
     -- transfer_admin \
     --new_admin <NEW_ADMIN_ADDRESS>
   ```

3. Verify the new admin is active:
   ```bash
   soroban contract invoke \
     --id <CONTRACT_ID> \
     --source new-admin \
     --network testnet \
     -- get_admin
   ```

4. Revoke the old identity from all systems and update CI secrets.

### Deployer Key Rotation

1. Fund the new deployer account on the target network.
2. Update `TESTNET_DEPLOYER_SECRET_KEY` in GitHub repository secrets.
3. Re-run the deployment workflow to verify the new key works.

---

## Contract Upgrade

1. Build the new WASM:
   ```bash
   make build-opt
   ```

2. Install the new WASM and scaffold the migration plan:
   ```bash
   ./scripts/upgrade_contract.sh <contract> testnet \
     target/wasm32-unknown-unknown/release/<contract>.wasm \
     <current_version> \
     <next_version> \
     --identity admin
   ```
   This writes `deployments/testnet/<contract>/plan.json` with the installed WASM hash.

3. Review the plan and perform a dry run:
   ```bash
   ./scripts/migrate_contract.sh <contract> --network testnet --dry-run
   ```

4. Execute the live migration only after the dry-run is approved:
   ```bash
   ./scripts/migrate_contract.sh <contract> \
     --network testnet \
     --identity admin \
     --i-understand-this-is-live
   ```
   The script requires an explicit `LIVE` confirmation at the prompt before it submits.

5. Verify the upgrade:
   ```bash
   soroban contract invoke \
     --id <CONTRACT_ID> \
     --source admin \
     --network testnet \
     -- version
   ```

6. Update deployment metadata and attach the approved dry-run output to the change record.

---

## Emergency Pause
If a contract must be halted immediately:

1. Invoke the pause function:
   ```bash
   soroban contract invoke \
     --id <CONTRACT_ID> \
     --source admin \
     --network testnet \
     -- pause
   ```

2. Confirm the contract is paused:
   ```bash
   soroban contract invoke \
     --id <CONTRACT_ID> \
     --source admin \
     --network testnet \
     -- is_paused
   ```
   Expected output: `true`

3. Notify stakeholders and open an incident report.

4. To resume after the issue is resolved:
   ```bash
   soroban contract invoke \
     --id <CONTRACT_ID> \
     --source admin \
     --network testnet \
     -- unpause
   ```

---

## Monitoring & Alerts

- Run `./scripts/monitor_deployments.sh testnet` to check all contract health endpoints.
- Alerts are written to `deployments/alerts.log`.
- Review `deployments/rollback_log.json` for recent rollback history.

---

## Rollback

If a deployment causes issues:

```bash
./scripts/rollback_deployment.sh <contract_name> testnet
```

See [DEPLOYMENT_GUIDE.md](DEPLOYMENT_GUIDE.md) for full rollback procedures.

