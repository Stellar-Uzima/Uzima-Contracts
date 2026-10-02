# ADR Adoption Notes

Concrete examples of the Architecture Decision Record process described in
[ADR-PROCESS.md](ADR-PROCESS.md) in use, so the process is demonstrated rather
than only described. Each entry names the decision, the ADR that records it, and
the outcome.

---

## 2026-09-30 — Requirement formalized for breaking changes

Before this date the process said when to *consider* an ADR; nothing required
one and nothing checked that the ADR directory was internally consistent. Issue
[#1651](https://github.com/Stellar-Uzima/Uzima-Contracts/issues/1651) closed
that gap:

- **Required vs recommended is now explicit.** [ADR-PROCESS.md](ADR-PROCESS.md#when-an-adr-is-required)
  lists the changes that cannot merge without an ADR — removing or re-signing an
  exported function, and removing a `DataKey` variant or `#[contracttype]` field.
- **The template records the classification.** [ADR-TEMPLATE.md](ADR-TEMPLATE.md)
  gained a *Change Classification* section (breaking ABI / storage / migration)
  so the reviewer can see the blast radius without reading the diff.
- **CI enforces it.** [`scripts/check_adr_required.py`](../../scripts/check_adr_required.py)
  fails a pull request that breaks an interface or a storage schema without an
  ADR, and fails if an ADR is missing its required header or is absent from the
  index.

### Worked example: what a breaking change looks like now

A change that re-signs `transfer(to, amount)` to `transfer(to)` produces:

```text
[adr] 7 ADR file(s); 1 breaking ABI / 0 storage change(s) detected.
[adr] adr-required FAILED:
  - breaking ABI change: payments: pub fn transfer(to: Address, amount: i128) {
  - no ADR was added or modified in this change set. Add one under docs/adr/ ...
```

Adding `docs/adr/ADR-008-transfer-signature.md` to the same pull request, with
`**Status:** Proposed`, clears the gate — CI then reports
`ADRs present in this change set: ADR-008`. A maintainer who has verified the
change is *not* breaking can pass `--allow-breaking`, which prints the waiver
into the job log instead of hiding it.

### The existing ADRs already satisfy the format

ADR-001 through ADR-007 predate the enforcement. They were written with a
`# ADR-NNN: ...` heading, a `**Status:**` and an ISO `**Date:**`, so the
hygiene and index checks pass against them unchanged — no retroactive edits were
needed. The gate exists to keep ADR-008 onward in the same shape.

---

## Prior decisions this process already covers

These records show the intended granularity: one decision, its alternatives, and
its consequences.

| ADR | Decision | Why it needed a record |
| --- | --- | --- |
| [ADR-001](ADR-001-soroban-platform-choice.md) | Use Soroban over EVM/Solana | Platform choice constrains every later contract. |
| [ADR-004](ADR-004-quorum-threshold-design.md) | Tiered quorum by proposal type | Governance parameters are a breaking change to off-chain tooling. |
| [ADR-007](ADR-007-on-chain-vs-offchain-governance-data.md) | On-chain vs off-chain governance data | Storage-layout decision with cost and privacy consequences. |

When one of these decisions is revisited, the process does not edit the old
record: it creates a new ADR that supersedes it, so the reasoning trail stays
intact.
