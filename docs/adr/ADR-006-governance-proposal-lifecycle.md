# ADR-006: Governance Proposal Lifecycle

**Status:** Accepted  
**Date:** 2025-06-29

## Context

The `governor` contract needs a well-defined lifecycle for proposals — from creation through execution or cancellation. Without a clear lifecycle, proposals can be acted upon at unintended stages (e.g., executed before the voting window closes), and governance participants lack predictability.

Key questions:
- How long should voting be open?
- Who can cancel a proposal and when?
- What happens to a proposal that reaches quorum but is not executed?
- Can a proposal be amended after submission?

## Decision

Proposals follow a **linear state machine** with the following states and transitions:

```
[Created] → [Active] → [Succeeded] → [Queued] → [Executed]
                ↓              ↓           ↓
           [Defeated]    [Cancelled]  [Expired]
```

**State definitions:**

| State | Entry Condition | Duration / Trigger |
|-------|----------------|--------------------|
| `Created` | Proposal submitted with valid calldata and metadata | Immediate |
| `Active` | Voting delay elapsed (24h after creation) | Voting period: 72 hours |
| `Defeated` | Voting period ends; quorum not met or approval threshold not met | Terminal |
| `Succeeded` | Voting period ends; quorum met; approval threshold met | Immediate transition to queue |
| `Queued` | Proposal queued in timelock contract | Timelock delay (see ADR-003) |
| `Executed` | Timelock delay elapsed; execute() called | Terminal |
| `Cancelled` | Proposer cancels (only while `Created` or `Active`); or admin cancels with supermajority | Terminal |
| `Expired` | Queued but not executed within 7 days of timelock expiry | Terminal |

**Amendment rule**: Proposals may **not** be amended after creation. A new proposal must be submitted. This prevents bait-and-switch attacks where a proposal is modified after garnering votes.

**Expiry rule**: A queued proposal that is not executed within 7 days of its timelock window opening expires automatically. This prevents stale approvals from being executed months later.

## Rationale

- The 24-hour voting delay (between creation and active) gives participants time to review the proposal before voting opens, reducing impulsive votes on surprise proposals.
- The 72-hour voting window gives all time zones adequate opportunity to participate.
- The no-amendment rule is essential for integrity: if a proposal could be changed after votes are cast, early voters would have voted on different content.
- The 7-day execution window for queued proposals prevents indefinite "zombie approvals" — passed but never executed proposals that could be executed opportunistically in the future.

## Consequences

- The governor contract must implement the full state machine with explicit state checks on each transition.
- Proposal IDs are immutable; a cancelled proposal cannot be reused.
- Proposers need to be aware that poorly specified proposals cannot be fixed — education and proposal templates are important.
- The execution window (7 days post-timelock) must be clearly communicated in governance tooling so executors don't miss the window.
