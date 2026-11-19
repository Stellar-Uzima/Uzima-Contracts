# Definition of Done

This is the single canonical checklist a pull request must satisfy before it
is merged. It consolidates the items previously spread across
[CONTRIBUTING.md](../CONTRIBUTING.md#before-requesting-review),
[contract-review-checklist.md](contract-review-checklist.md),
[SECURITY_CHECKLIST.md](SECURITY_CHECKLIST.md) and
[PERFORMANCE_BUDGETS.md](PERFORMANCE_BUDGETS.md). Those documents remain the
detailed references; this one is the gate.

## Checklist

### Build, format and lint
- [ ] `cargo fmt --all` passes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [ ] `cargo test --all` passes locally (`make check` runs fmt + clippy + test).
- [ ] `make check-gates` passes (event, generated-artifact, template and
      dead-code gates; see [development.md](development.md#repository-gates)).

### Tests
- [ ] Unit tests cover normal behaviour, edge cases and invalid inputs,
      including negative paths.
- [ ] Integration tests cover cross-contract behaviour where relevant.
- [ ] Regression tests exist for any bug being fixed.

### Contract conventions
- [ ] Storage keys use the `DataKey` enum pattern; errors use `#[contracterror]`.
- [ ] Events are emitted for state-changing operations.
- [ ] Privileged functions call `require_auth()` and verify the role.
- [ ] Integer arithmetic uses `checked_*` / `saturating_*` where overflow is possible.

### ADRs and docs
- [ ] Breaking ABI or storage-layout changes include an ADR in the same PR
      (`python3 scripts/check_adr_required.py`).
- [ ] Docs, `docs/api.md` and `CHANGELOG.md` are updated for changed behaviour.
- [ ] Excluded-contract changes update `docs/ONRAMP_OWNERSHIP.md`.
- [ ] The PR description references the relevant issue(s).

### Security review
- [ ] Every item in [SECURITY_CHECKLIST.md](SECURITY_CHECKLIST.md) is checked
      or explicitly waived with a justification.
- [ ] No secrets, private keys or key-like literals are committed (see
      [Script secret policy](#script-secret-policy)).

### WASM budgets and telemetry
- [ ] The release WASM is **at most 64 KB (65,536 bytes)**. This is the hard
      Soroban deploy limit and the only blocking size threshold.
- [ ] **60 KB (61,440 bytes)** is a *warning* threshold only
      (`warning_threshold_bytes` in `scripts/wasm_size_baselines.json`);
      crossing it should be justified in the PR, not a failure.
- [ ] A new deployable contract has an entry in
      `scripts/wasm_size_baselines.json` (or is listed in `"excluded"`);
      CI fails otherwise (`scripts/check_wasm_baseline_staleness.sh`).
- [ ] Storage budgets and WASM size telemetry from `scripts/measure_storage.sh`
      show no regression beyond the configured tolerance
      (see [PERFORMANCE_BUDGETS.md](PERFORMANCE_BUDGETS.md)).

## Resolved threshold conflict

Earlier docs quoted both "60 KB" and "64 KB" as the WASM limit. They are not
alternatives: 64 KB is the hard limit and 60 KB is the early-warning level
below it. The README's "must be <= 64 KB" is consistent with this.

## Script secret policy

No script (`scripts/**`, including `.sh` and `.ts`) may contain an embedded
secret, or a fake key literal that looks like one. A fake literal can be
copied into a real deployment by mistake and also trips secret scanners
(see `.github/workflows/secret-scan.yml`).

- Read secrets and identities at runtime from the environment or from the
  per-network, git-ignored `.secrets/` directory managed by
  `scripts/manage_secrets.sh`.
- Generate throwaway keys at runtime (for example `Keypair.random()`), never
  as a literal.
- Where a non-secret placeholder is unavoidable in a template or output, use
  an obviously non-key token such as `<AUDITOR_PUBKEY_PENDING>` (as
  `scripts/verify_deployment.sh` does) and never a string shaped like a
  Stellar secret seed (`S` followed by 55 base32 characters) or public key.

Known violation to clean up: `scripts/deploy_forensics.ts` assigns a
truncated seed-shaped literal (`'SAKLGIB3G6P7E5YV...'`, described as "just a
placeholder") to an unused variable. It should be removed or replaced with
`process.env` / `Keypair.random()`.
