# Contract Resource Limits and Constraints

## Purpose

This document captures the core resource limits and constraints that apply to all Stellar Uzima Soroban contracts. It is intended to help developers keep contract code, storage, execution, and batch workflows within safe limits for Stellar/Soroban deployment, while also providing monitoring and performance guidance.

## Scope

Applies to all contracts in `contracts/*` and all deployments built using the Stellar Uzima workspace.

## 1. Core Soroban / Stellar Limits

### Contract size
- **Max contract size:** 64 KB (65,536 bytes)
- **Deployment warning threshold:** 80% of limit (~51.2 KB)
- **Deployment critical threshold:** 95% of limit (~61.4 KB)
- **Implication:** Contracts larger than the limit cannot be deployed. Keep contract source, dependencies, and build artifacts optimized.

### Transaction size
- **Max transaction size:** 64 KB (65,536 bytes)
- **Implication:** Large transactions with many arguments, events, or contract calls can fail due to size. Keep transaction payloads compact.

### Operation count
- **Max operations per Stellar transaction:** 100 operations
- **Implication:** Multi-contract batch workflows should not exceed the 100-operation cap. Complex workflows should be split into smaller transactions.

### Gas and execution
- **Soroban metering model:** WASM instructions are metered and charged as gas
- **Recommended alert threshold:** 3,000,000 instructions for business-critical operations
- **Implication:** Heavy computations, large loops, and expensive serialization raise gas cost and can make contracts expensive or slow.

### Event frequency
- **Event limits:** bounded by transaction size and ledger footprint
- **Implication:** Emitting too many events in a single transaction can cause the transaction to exceed the size limit. Use events for key state changes only.

### State and storage
- **Storage capacity:** contract data is bounded by ledger limits and network policies
- **Implication:** On-chain storage is limited and costly. Do not store large encrypted medical payloads directly in contract data.

## 2. Storage Capacity Guidelines

### Contract data size
- Store only small identifiers, hashes, pointers, and metadata on-chain.
- Use off-chain encrypted storage for large medical records and save only a reference or proof on-chain.
- Keep individual contract data entries compact and avoid storing binary payloads larger than a few kilobytes.

### Record design
- For patient records, use a normalized key structure such as `patient_id`, `record_id`, and `version`.
- Preserve auditability by storing timestamps, author IDs, and permission metadata without storing entire document bodies.

### Total state growth
- Use bounded collections and periodic cleanup patterns where appropriate.
- Avoid unbounded append-only storage for high-volume data such as event logs, telemetry, or temporary records.

## 3. Computation and Performance Limits

### Function complexity
- Keep contract functions deterministic and avoid unbounded loops.
- Prefer simple control flow and fixed-size data structures.
- Large branches, nested loops, or repeated cryptographic operations can dramatically increase gas.

### Batch operations
- Split large batch updates into multiple transactions if they risk exceeding transaction size, operation count, or instruction budgets.
- Recommended batch size: keep contract calls and state writes under 10-20 operations for complex workflows.
- For large migrations or bulk updates, use incremental processing with checkpoints.

### Event emission
- Emit events only for meaningful changes, such as access grants, consent updates, or audit actions.
- Avoid using events as a substitute for high-volume record storage.
- Event payloads should be minimal and structured for easy indexing.

## 4. Performance Implications

### Deployment risk
- Large WASM files increase deployment risk and make upgrades harder.
- Contracts near the 64 KB limit are fragile and require tighter dependency control.

### Cost and latency
- High instruction usage directly increases transaction cost and validation time.
- Large on-chain state and many events can slow ledger validation and contract invocations.

### Maintainability
- Contracts designed to fit within Soroban limits are easier to audit and upgrade.
- Smaller, modular contracts make it simpler to isolate failures and optimize performance.

## 5. Monitoring Recommendations

### WASM size monitoring
- Use the repository monitoring script: `scripts/wasm_size_monitor.sh`
- Recommended Make targets:
  - `make monitor-wasm`
  - `make check-wasm-size`
- Track size trends in `.wasm_size_trends.json` and keep contract sizes below the warning threshold.

### Gas and execution monitoring
- Monitor transaction instruction usage in test and staging environments.
- Watch for operations that consistently exceed 3,000,000 instructions.
- Use benchmarking and profiling tools such as `cargo bloat` when available.

### Event and batch monitoring
- Audit event frequency in transactions and correlate with transaction size.
- Validate batch workflows against operation limits and transaction payload size.
- Implement alerts for transactions that generate excessive events or state writes.

## 6. Best Practices for All Contracts

- Keep smart contracts modular and purpose-specific.
- Remove unused dependencies and dead code before release builds.
- Prefer `--release` optimized builds for deployment: `cargo build --target wasm32-unknown-unknown --release`.
- Keep event and state payloads minimal.
- Use bounded and paginated patterns for high-volume data.
- Keep code size within safe limits, not just deployable limits.

## 7. Recommended Actions

1. Build optimized contracts regularly and check WASM size with `make check-wasm-size`.
2. Use `make monitor-wasm` to detect growth trends and critical contracts.
3. Review any contract above 80% of the 64 KB limit for optimization.
4. Avoid storing large medical record payloads directly in contract state; use off-chain storage references instead.
5. Split large workflows into smaller transactions to stay within operation and size limits.

## 8. Notes

- These limits are based on current Stellar/Soroban runtime constraints and the repository's deployment practices.
- Always verify network-specific limits before production deployment, since network policies may evolve.
