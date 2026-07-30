# ADR-005: Vote Weight Mechanism

**Status:** Accepted  
**Date:** 2025-06-29

## Context

The `governor` contract needs a vote weight model that determines how much influence each participant has on proposal outcomes. Healthcare governance must balance several competing concerns:

- **Patient primacy**: Patients whose data is managed should have meaningful input, especially on data-sharing policies.
- **Operational expertise**: Healthcare providers and administrators have domain knowledge about feasibility.
- **Regulatory accountability**: Compliance officers bear legal responsibility and need proportionate influence on policy changes.
- **Plutocracy resistance**: Weight should not be purely proportional to economic stake, as that would exclude patients.

Options evaluated:
1. **Equal weight (1 address = 1 vote)** — simple, fully democratic, ignores expertise
2. **Token-weighted** — aligns incentives with economic stake; not appropriate for healthcare where patients shouldn't need to buy governance tokens
3. **Role-weighted** — weight based on assigned governance role; recognized expertise
4. **Delegation model** — participants can delegate their votes to a trusted representative

## Decision

Use a **role-weighted delegation model**:

| Governance Role | Base Weight | Notes |
|-----------------|:-----------:|-------|
| Patient | 1 | Any registered patient may vote on policies affecting patient data |
| Healthcare Provider (Doctor/Nurse) | 2 | Elevated weight reflects operational expertise |
| Compliance Officer | 3 | Regulatory accountability warrants higher influence on policy proposals |
| Governance Council Member | 4 | Elected representatives with full governance mandate |
| Admin | 1 | Administrative weight equal to Patient to prevent admin capture |

Delegation: Any participant may delegate their vote weight to another participant for a specific proposal or for all future proposals until revoked. Delegated weight stacks (a council member with 5 delegations from patients has weight 4 + 5×1 = 9).

Delegation is recorded on-chain via `delegate_vote(from, to, scope)` and is revocable at any time before a proposal closes.

## Rationale

- Role-based weights are transparent and auditable — every participant can see exactly how much weight any vote carries.
- Delegation enables patients who lack time or technical knowledge to participate through trusted representatives (e.g., a patient advocacy organization), without disenfranchising them.
- Keeping Admin weight equal to Patient weight prevents the system from being captured by the operations team, who manage the contracts but should not dominate patient-data policy.
- Token-weighting was explicitly rejected because it would create economic barriers to patient participation in decisions about their own medical data.

## Consequences

- The governor contract must store and enforce role-to-weight mappings.
- The delegation graph must be stored on-chain; deep delegation chains should be bounded (max depth 3) to prevent unbounded computation on weight resolution.
- Weight assignments for governance roles are themselves subject to governance (requiring a governance configuration change — see ADR-004).
- Off-chain tooling should display effective vote weight (including delegated weight) to participants before they vote.
