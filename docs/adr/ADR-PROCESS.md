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

## When an ADR is Required

Some changes cannot be merged without an ADR. These are the **breaking**
changes: an integrator or a deployed contract has to do something before the
change is safe. The distinction matters because "required" is enforced in CI
(see [Enforcement](#enforcement)), while the list above is review guidance.

| Change | ADR | Why |
| --- | --- | --- |
| Removing or re-signing an exported contract function (`pub fn` in a `#[contractimpl]`) | **Required** | Breaks every caller of the contract. |
| Removing a `DataKey` enum variant | **Required** | Orphans persisted entries and changes the storage layout. |
| Removing or renaming a field of a `#[contracttype]` struct | **Required** | Changes the serialized shape of stored values. |
| Removing a `#[contracttype]` type or the `DataKey` enum itself | **Required** | Changes the storage schema. |
| Adding a function, a `DataKey` variant, or a `#[contracttype]` field | Recommended | Additive and usually backward compatible; record it if the decision is non-obvious. |
| Internal refactors with no interface or storage change | Not required | Nothing observable changes. |

A change in the left column and the ADR that justifies it belong in the **same
pull request**. The record is part of the change, not a follow-up.

See [docs/CHANGE_IMPACT_MATRIX.md](../CHANGE_IMPACT_MATRIX.md) for the full
ABI/storage/migration checklist this rule is drawn from.

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

Where a change is breaking, the ADR is created in step 1 of the *change*
itself rather than before it: the contract edit and the ADR land together, so
the reasoning is available to the reviewer at the moment it is needed.

## Enforcement

[`scripts/check_adr_required.py`](../../scripts/check_adr_required.py) turns the
rules above into a gate. It runs on every pull request
([`.github/workflows/adr-required.yml`](../../.github/workflows/adr-required.yml))
and as part of `make check-gates` / `make check-everything`. It checks three
things:

1. **ADR hygiene** — every numbered ADR has a title, a valid `**Status:**`, and an ISO `**Date:**`.
2. **Index sync** — the "Existing ADRs" table below lists exactly the ADR files on disk.
3. **ADR required for breaking changes** — a detected ABI or storage break with no ADR in the change set fails the build.

Run it locally before opening a PR:

```bash
python3 scripts/check_adr_required.py                 # compare HEAD with origin/main
python3 scripts/check_adr_required.py --diff-file d.patch
python3 scripts/check_adr_required.py --allow-breaking  # maintainer waiver, printed to the log
```

`--allow-breaking` exists for the rare case where the detector is too eager. It
prints exactly what it waived, so a waiver is a visible line in CI rather than
a silent skip.

## File Location

All ADRs live in `docs/adr/`. The index below is auto-maintained and verified
by `scripts/check_adr_required.py`.

## Adoption Notes

Examples of the process in action, including how the requirement was
introduced, are recorded in [ADOPTION_NOTES.md](ADOPTION_NOTES.md).

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
