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

2. Upload the new WASM to the network:
   ```bash
   soroban contract upload \
     --source deployer \
     --network testnet \
     --wasm target/wasm32-unknown-unknown/release/<contract>.wasm
   ```
   Note the returned `WASM_HASH`.

3. Upgrade the deployed contract:
   ```bash
   soroban contract invoke \
     --id <CONTRACT_ID> \
     --source admin \
     --network testnet \
     -- upgrade \
     --new_wasm_hash <WASM_HASH>
   ```

4. Verify the upgrade:
   ```bash
   soroban contract invoke \
     --id <CONTRACT_ID> \
     --source admin \
     --network testnet \
     -- version
   ```

5. Update `deployments/<network>_<contract>.json` with the new WASM hash and timestamp.

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
