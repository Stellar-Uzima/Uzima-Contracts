# Architecture Decision Records

This directory keeps a chronological record of governance-related architecture
decisions for Uzima Contracts.

ADRs are intended to answer: "Why did we choose this direction?" They complement
the longer governance guides by preserving the context, decision, and
consequences of accepted or rejected proposals.

## Numbering

- ADR numbers are monotonically increasing.
- Numbers are never reused, even if an ADR is later superseded.
- New ADRs should use the next available number in this directory.
- Existing ADR filenames are preserved for compatibility with older links.

## Status Values

Use one of these statuses:

- `Proposed`
- `Accepted`
- `Rejected`
- `Deprecated`
- `Superseded by ADR-XXXX`

## Index

| ADR | Status | Title |
| --- | --- | --- |
| [ADR-001](ADR-001-governance-commons-migration.md) | Accepted | Governance Commons Migration |
| [ADR-002](ADR-002-real-zkp-verification.md) | Accepted | Real ZKP Verification |
| [ADR-003](ADR-003-excluded-contracts-triage.md) | Accepted | Excluded Contracts Triage |
| [ADR-006](ADR-006-governance-proposal-lifecycle.md) | Accepted | Governance Proposal Lifecycle |
| [ADR-007](ADR-007-on-chain-vs-offchain-governance-data.md) | Accepted | On-Chain vs Off-Chain Storage of Governance Data |

## Creating a New ADR

1. Copy [`0000-template.md`](0000-template.md).
2. Rename it to the next available number and a short kebab-case title, for
   example `ADR-008-upgrade-quorum-policy.md`.
3. Fill in the Context, Decision, and Consequences sections.
4. Add the ADR to the index above.
5. Link the ADR from the related issue or pull request.
