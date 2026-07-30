# Architecture Decision Record (ADR) Process

This document describes the process for creating, reviewing, and maintaining Architecture Decision Records (ADRs) for the Stellar Uzima contract portfolio.

## What is an ADR?

An ADR captures a significant architectural decision along with its context and consequences. ADRs are immutable once accepted — if a decision is reversed, a new ADR is created that supersedes the old one.

## When to Write an ADR

Write an ADR when:

- Choosing a new technology, library, or framework
- Changing a contract interface or storage layout
- Modifying governance parameters (quorum, timelock, voting)
- Introducing a new cross-contract integration pattern
- Changing security or authentication boundaries
- Modifying data retention or purging behavior
- Any decision that affects the long-term architecture

## ADR Lifecycle

```
Proposed → Accepted → (Deprecated | Superseded)
```

1. **Proposed**: Draft created, under discussion
2. **Accepted**: Decision approved by maintainers
3. **Deprecated**: No longer relevant (but kept for history)
4. **Superseded**: Replaced by a newer ADR

## Naming Convention

ADRs are numbered sequentially: `ADR-{NNN}-{short-title}.md`

Examples:
- `ADR-001-soroban-platform-choice.md`
- `ADR-008-consent-purging-policy.md`

## Template

Use the template at `docs/adr/ADR-TEMPLATE.md`.

## Process

1. **Create the ADR**: Copy the template, fill in all sections
2. **Open a PR**: Add the ADR to `docs/adr/` with a descriptive title
3. **Discussion**: Maintainers review and discuss in the PR
4. **Decision**: Once consensus is reached, update the status to "Accepted"
5. **Merge**: ADR is merged and becomes part of the project record

## File Location

All ADRs live in `docs/adr/`. The index below is auto-maintained.

## Existing ADRs

| ADR | Title | Status |
|-----|-------|--------|
| ADR-001 | Use Soroban (Stellar) platform | Accepted |
| ADR-002 | Patient consent model | Accepted |
| ADR-003 | Timelock duration design | Accepted |
| ADR-004 | Quorum threshold design | Accepted |
| ADR-005 | Vote weight mechanism | Accepted |
| ADR-006 | Governance proposal lifecycle | Accepted |
| ADR-007 | On-chain vs off-chain governance data | Accepted |
