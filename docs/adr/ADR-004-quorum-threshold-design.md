# ADR-004: Quorum Threshold Design for Governance Proposals

**Status:** Accepted  
**Date:** 2025-06-29

## Context

The `governor` contract requires a minimum participation level (quorum) before a proposal can pass, preventing a small minority from enacting changes when most stakeholders are absent or unaware. The quorum must be high enough to ensure legitimacy but low enough that governance doesn't deadlock during periods of low participation.

Options evaluated:
1. **Fixed absolute quorum** (e.g., at least 10 distinct voters) — simple but doesn't scale
2. **Fixed percentage quorum** (e.g., 20% of registered governance participants) — scales with system size
3. **Tiered quorum by proposal type** — higher thresholds for higher-impact decisions

Healthcare governance involves a relatively small, known set of participants (hospital admins, department heads, compliance officers) rather than anonymous token holders, making participation rates more predictable and absolute counts more meaningful.

## Decision

Use a **tiered quorum model** based on proposal impact:

| Proposal Type | Quorum Required | Approval Threshold |
|---------------|:--------------:|:-----------------:|
| Parameter change (minor) | 30% | Simple majority (>50%) |
| Contract upgrade | 50% | Supermajority (>66%) |
| Role hierarchy change | 50% | Supermajority (>66%) |
| Emergency action | 66% | Supermajority (>66%) |
| Governance configuration change | 66% | Supermajority (>66%) |

Quorum is calculated over **active governance participants** — addresses that have cast at least one vote in the preceding 90-day window. Inactive addresses do not count against quorum.

## Rationale

- Tiered thresholds match the stakes of each decision: minor parameter tweaks require less consensus than contract upgrades that affect all patients.
- Calculating quorum over **active participants** (rather than all registered addresses) avoids governance deadlock from dormant accounts — a common failure mode in on-chain governance.
- The 90-day activity window ensures the active set reflects current stakeholders, not legacy participants who are no longer engaged.
- A fixed 20% flat quorum was rejected because it would allow minor admin changes to pass with dangerously low participation in a large deployment.

## Consequences

- The governor contract must maintain an activity index per participant, updated on each vote.
- The 90-day window introduces a bootstrapping challenge for new governance participants — they must cast one vote before they count toward quorum.
- Quorum parameters themselves are a "governance configuration change" requiring 66% quorum to modify, providing strong protection against lowering safety thresholds.
- Off-chain governance tooling must display the current active participant count to proposal creators.
