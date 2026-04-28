# Contract Deployment Checklist

Use this checklist before deploying any Uzima contract to a production or testnet environment. All items must be checked off and signed by the responsible team member before proceeding.

---

## Pre-Deployment

### Code Quality
- [ ] All tests pass locally: `cargo test --all`
- [ ] No Clippy warnings: `cargo clippy --all -- -D warnings`
- [ ] Code formatted: `cargo fmt --all -- --check`
- [ ] No `unwrap()` or `expect()` in production paths (use `?` or explicit error handling)
- [ ] All public functions have doc comments

### Security Review
- [ ] Security audit completed (internal or external)
- [ ] All admin functions protected with `require_auth()` and `require_admin()`
- [ ] No hardcoded secrets, keys, or addresses
- [ ] Reentrancy risks assessed (Soroban is single-threaded, but cross-contract calls reviewed)
- [ ] Integer overflow/underflow handled (`checked_add`, `saturating_*`, or explicit bounds)
- [ ] Access control matrix reviewed against RBAC policy

### Testing
- [ ] Unit tests cover all public functions
- [ ] Integration tests cover cross-contract interactions
- [ ] Edge cases tested: double-initialization, unauthorized calls, overflow inputs
- [ ] Test coverage ≥ 80% (run `scripts/coverage_report.sh`)
- [ ] Testnet smoke test passed

### Documentation
- [ ] `README.md` updated with new functions/parameters
- [ ] API reference (`docs/api.md`) updated
- [ ] Changelog entry added
- [ ] Migration guide written (if breaking changes)

---

## Deployment

### Environment Setup
- [ ] Correct network configured: `soroban config network ls`
- [ ] Deployer identity funded with sufficient XLM
- [ ] Environment variables set (no secrets in shell history)
- [ ] Previous deployment backed up: `scripts/deploy_with_rollback.sh`

### Deployment Steps
- [ ] Build optimized WASM: `make build-opt`
- [ ] Verify WASM size within limits (see `docs/CONTRACT_RESOURCE_LIMITS.md`)
- [ ] Deploy contract: `./scripts/deploy.sh <contract> <network>`
- [ ] Record contract ID in `deployments/<network>_<contract>.json`
- [ ] Initialize contract with correct admin address
- [ ] Verify deployment: `./scripts/verify_deployment.sh`

### Post-Deployment Verification
- [ ] Contract responds to read-only queries
- [ ] Admin functions accessible only to admin address
- [ ] Events emitted correctly (check via Stellar Expert or local logs)
- [ ] Monitoring configured: `./scripts/monitor_deployments.sh <network>`

---

## Post-Deployment

### Monitoring
- [ ] Health check passing: `./scripts/monitor_deployments.sh <network> --alert-on-failure`
- [ ] Alert thresholds configured in `config/health_alerts_config.json`
- [ ] On-call rotation notified of new deployment

### Rollback Plan
- [ ] Rollback procedure documented and tested
- [ ] Previous contract ID recorded for rollback: `deployments/<network>_<contract>_backup_*.json`
- [ ] Rollback command verified: `./scripts/rollback_deployment.sh <contract> <network>`

### Sign-Off

| Role | Name | Date | Signature |
|---|---|---|---|
| Developer | | | |
| Reviewer | | | |
| Security | | | |
| DevOps | | | |

---

## CI/CD Automation

The following checks run automatically on every PR and must pass before merging:

```yaml
# .github/workflows/ci.yml (excerpt)
- cargo fmt --all -- --check
- cargo clippy --all -- -D warnings
- cargo test --all
- scripts/coverage_report.sh
```

For testnet deployments triggered by merges to `develop`, see `.github/workflows/deploy.yml`.
