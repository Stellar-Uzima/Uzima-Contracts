# Testing and Coverage

This repository uses automated coverage enforcement for Soroban contracts in CI. The policy is aligned with [SECURITY_CHECKLIST.md](SECURITY_CHECKLIST.md), section 9.

## Coverage thresholds

CI enforces the following per-contract minimums:

- `medical_records`: 80%
- `governor`: 80%
- `zkp_registry`: 80%
- all other contracts: 60%

## Local workflow

Run the workspace coverage job locally:

```bash
./scripts/coverage_report.sh
```

The script generates:

- `coverage/lcov.info` for LCOV consumers
- `coverage/html/index.html` for an annotated HTML report
- `reports/coverage_report.txt` with the threshold evaluation
- `reports/coverage_badge.svg` for the README badge

## CI behavior

The GitHub Actions workflow runs the coverage job on every pull request and push to the main branches. A PR fails if any contract is below the configured threshold.
