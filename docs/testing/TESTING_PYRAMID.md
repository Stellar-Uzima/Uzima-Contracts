# Contract testing pyramid

This repo follows a testing pyramid to keep feedback fast and maintain confidence in behavior changes.

## Target distribution

- **Unit tests (70%)**
  - Fast, isolated, minimal dependencies
  - Focus on pure logic, validation, edge cases, and invariants
- **Integration tests (20%)**
  - Exercise contract entrypoints end-to-end in a single `Env` (often with multiple modules/helpers)
  - Validate cross-module flows and important error cases
- **End-to-end tests (10%)**
  - Cross-contract workflows and “realistic” scenarios (may span `tests/` modules)
  - Use sparingly due to runtime cost and flakiness risk

## Conventions (how tests are classified)

The automated pyramid report classifies tests by **file location**:

### Unit tests

- `contracts/<contract>/src/**/test*.rs`
- `contracts/<contract>/src/**/tests.rs`
- `contracts/<contract>/src/**` modules behind `#[cfg(test)]`
- `tests/unit/**/*.rs`

### Integration tests

- `contracts/<contract>/tests/**/*.rs`
- `tests/integration/**/*.rs`

### End-to-end tests

- `tests/e2e/**/*.rs`

> If you need a new E2E suite, create `tests/e2e/` and keep it small and scenario-focused.

## Examples

### Unit test (contract-local)

Put small, fast tests in `contracts/<contract>/src/test.rs` (or a `#[cfg(test)] mod tests { ... }` inside a module).

### Integration test (contract-local)

Put flow tests in `contracts/<contract>/tests/*.rs` to keep them out of the contract crate’s `src/` tree.

### Cross-contract / E2E test (repo-level)

Put system workflow tests in `tests/e2e/*.rs` and keep them few (happy path + key failure cases).

## CI enforcement

CI generates a report and enforces the target ratios (with a small tolerance) using:

- `npm run test:pyramid:report`

Generated artifacts:

- `reports/testing/pyramid.json`
- `reports/testing/pyramid.md`

