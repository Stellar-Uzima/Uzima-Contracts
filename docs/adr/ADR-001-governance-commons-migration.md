# ADR-001: Governance Commons Migration

**Status:** Accepted
**Date:** 2025-06-29

## Context

The governance system includes multiple contracts with overlapping approval
patterns. `UpgradeManager` coordinates contract upgrades with validator
approvals, while `EmergencyAccessOverride` grants emergency medical access with
approver consent and rate limiting. Before the governance separation work, each
contract carried its own multi-signature approval tracking and threshold logic.

Issue #769 documented the broader concern: governance responsibilities were hard
to distinguish, and duplicated approval logic made future changes more expensive
to audit.

The relevant existing documentation is:

- `docs/ISSUE_769_RESOLUTION.md`
- `docs/GOVERNANCE_ARCHITECTURE.md`
- `docs/GOVERNANCE_REFACTORING_GUIDE.md`
- `libs/governance_commons/README.md`

## Decision

Create and use `libs/governance_commons` as the shared home for reusable
governance primitives, starting with multi-signature approval helpers, common
types, and common errors.

The `governance_commons::multi_sig` module is the canonical implementation for:

- approval set validation
- approver validation
- duplicate-safe approval insertion
- threshold status calculation

`UpgradeManager` and `EmergencyAccessOverride` remain separate contracts with
separate responsibilities. They should use the shared approval helpers where the
approval mechanics are common, while retaining domain-specific behavior in their
own contracts.

## Consequences

Shared multi-signature logic reduces duplicate code and gives reviewers one
place to inspect approval semantics.

Contract-specific responsibilities remain explicit:

- `UpgradeManager` owns contract upgrade orchestration.
- `EmergencyAccessOverride` owns emergency medical access, expiry, and
  rate-limiting behavior.
- `Governor` owns token-holder proposal voting.
- `DisputeResolution` owns proposal challenge and arbitration.

Future changes to approval behavior must consider every caller of
`governance_commons`. Contract-specific constraints, such as emergency-access
rate limits, must not be moved into the shared library unless they become truly
generic.

The initial migration created the shared library and documented the intended
refactoring path. Follow-up PRs can migrate remaining contract-local approval
logic into the shared helpers when doing so does not change behavior.

## References

- `docs/ISSUE_769_RESOLUTION.md`
- `docs/GOVERNANCE_ARCHITECTURE.md`
- `docs/GOVERNANCE_REFACTORING_GUIDE.md`
- `libs/governance_commons/README.md`
- Original governance separation issue: #769
