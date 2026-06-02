# Contract Review Checklist

A shared checklist for reviewing contract submissions helps ensure consistency, correctness, safety, and testing coverage across the repository.

## Purpose

This checklist is intended for reviewers and contributors working on smart contract code, contract interfaces, and related contract logic.

## How to use

- Review this checklist before submitting a contract-related PR.
- Confirm the checklist items during review discussions.
- Update contract documentation and tests whenever new risks or failure modes are identified.

## Correctness

- [ ] Contract behavior matches the specification and design requirements.
- [ ] All public and internal functions handle expected and unexpected inputs correctly.
- [ ] State transitions and invariants are well-defined and preserved.
- [ ] Access control and authorization checks are correct for every sensitive operation.
- [ ] Numeric operations use safe arithmetic and guard against overflow / underflow.
- [ ] Error messages are clear, consistent, and do not leak sensitive implementation details.
- [ ] Edge cases are handled explicitly and documented where needed.

## Safety

- [ ] Inputs are validated and invalid values are rejected safely.
- [ ] Contract invariants and safety properties are documented and enforced.
- [ ] No unauthorized state changes are possible through reentrancy, callback, or indirect calls.
- [ ] Sensitive data and permissions are protected according to privacy and security expectations.
- [ ] Resource usage (gas, storage) is bounded and does not allow abuse.
- [ ] Upgrade and migration paths are safe, or upgradeability is explicitly prohibited and documented.
- [ ] External dependencies and cross-contract calls are treated as potentially untrusted.
- [ ] Failure paths fail safely and leave the contract in a consistent state.

## Testing

- [ ] Unit tests cover normal behavior, edge cases, and invalid inputs.
- [ ] Integration tests verify contract interactions and cross-contract behavior.
- [ ] Regression tests exist for previously discovered bugs or vulnerabilities.
- [ ] Test coverage includes both positive and negative paths.
- [ ] Test cases document expected outcomes and the contract invariants they exercise.
- [ ] Fuzz tests, property-based tests, or scenario tests are included when applicable.
- [ ] Automated CI checks pass for code formatting, linting, build, and tests.

## Review reminders

- [ ] Confirm documentation is updated for any changed behavior or new contract rules.
- [ ] Check that the PR description clearly states the intent and risk areas.
- [ ] Verify the contract uses the project’s accepted coding and security practices.
- [ ] If the change affects external integrations or network behavior, ensure stakeholders are notified.

## Notes

This checklist is a companion to the project’s existing contribution and testing policies. It is not a substitute for a deeper security or audit review when needed.
