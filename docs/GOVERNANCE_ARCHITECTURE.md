# Governance Architecture & Separation of Concerns

## Overview

The Uzima governance system consists of multiple specialized contracts, each with a single, well-defined responsibility. This document clarifies the separation of concerns and provides guidance on which mechanism to use for different scenarios.

## Issue Reference

- **Issue #769**: Dispute Resolution Contract Overlaps with Governance — Unclear Separation of Concerns
- **Status**: In Progress

## Contract Responsibilities Matrix

| Contract | Primary Responsibility | Approval Type | Key Features |
|----------|----------------------|---------------|--------------|
| **Governor** | Token-based voting & proposal lifecycle | Quorum + voting majority | Voting delay, voting period, voting power tracking |
| **DisputeResolution** | Dispute arbitration | Arbiter consensus | Can override proposals, challenge period |
| **UpgradeManager** | Contract upgrades | Validator multi-sig | Time delay (24h), emergency fast-track |
| **EmergencyAccessOverride** | Emergency medical access | Approver multi-sig | Rate limiting, expiry, cooldown periods |
| **Timelock** | Execution time-delay gate | Time passage | Prevents flash-loan attacks, dual-check (time + sequence) |

## Detailed Contract Descriptions

### 1. Governor Contract
**Soroban Type**: Token-based governance

**Responsibility**: Manage token holder voting on proposals

**Key Functions**:
- `propose()` - Create a new proposal (requires voting power)
- `cast_vote()` - Token holders vote (for/against/abstain)
- `state()` - Get proposal state (pending, active, succeeded, failed, queued, executed, disputed)
- `queue()` - Queue successful proposal for execution
- `execute()` - Execute queued proposal

**Approval Logic**:
- Voting delay period before voting starts
- Voting period during which voting occurs
- Quorum requirement (minimum voting participation)
- Voting majority (for > against)
- Voting power tracked per holder

**Integration Points**:
- Reads voting power from optional reputation contract
- Delegates to DisputeResolution (if configured) to check if proposal is disputed
- Queues proposals via Timelock contract

**Decision**: Use Governor if your action requires **token holder consensus** with voting periods

### 2. DisputeResolution Contract
**Soroban Type**: Arbitration mechanism

**Responsibility**: Provide dispute arbitration on governance decisions

**Key Functions**:
- `dispute()` - Challenge a proposal (requires token bond)
- `resolve()` - Arbiters resolve dispute (valid_proposal = true/false)
- `is_disputed()` - Check if proposal is currently disputed

**Approval Logic**:
- Arbiters are pre-configured set
- Any arbiter can resolve a dispute
- Dispute can be cleared (allow execution) or kept active (block execution)

**Integration Points**:
- Called by Governor to check if proposals are disputed
- Blocks Governor execution if active dispute exists
- Runs in parallel with Governor voting

**Decision**: Use DisputeResolution if you need **challenge/arbitration** capability on governance decisions

### 3. Timelock Contract
**Soroban Type**: Time-delay execution gate

**Responsibility**: Impose mandatory time delay before sensitive operations execute

**Key Functions**:
- `queue()` - Queue transaction for execution after delay
- `execute()` - Execute queued transaction once delay has passed

**Approval Logic**:
- No approvals; purely time-based
- Dual-check: both timestamp AND ledger sequence must advance
- Prevents flash-loan attacks

**Integration Points**:
- Governor queues proposals via Timelock
- Timelock enforces delay before execution
- Acts as circuit breaker for emergency scenarios

**Decision**: Use Timelock for **time-delay execution gates** on sensitive operations

### 4. UpgradeManager Contract
**Soroban Type**: Multi-sig contract upgrade orchestration

**Responsibility**: Orchestrate contract upgrades with validator approvals

**Key Functions**:
- `propose_upgrade()` - Propose an upgrade (normal or emergency)
- `approve_upgrade()` - Validator approves upgrade
- `execute_upgrade()` - Execute upgrade once threshold reached

**Approval Logic**:
- Fixed set of validator addresses
- Configurable approval threshold
- Emergency mode bypasses time delay
- Uses shared `governance_commons::multi_sig` module

**Integration Points**:
- Calls target contract's upgrade function
- No integration with Governor (operates independently)
- Can use Timelock optionally for additional delay

**Decision**: Use UpgradeManager for **contract upgrades** coordinated by validators

### 5. EmergencyAccessOverride Contract
**Soroban Type**: Multi-sig emergency access control

**Responsibility**: Grant emergency medical access with multi-sig controls

**Key Functions**:
- `grant_emergency_access()` - Approver grants emergency access
- `check_emergency_access()` - Verify if access is currently granted
- `revoke_emergency_access()` - Remove granted access

**Approval Logic**:
- Fixed set of trusted approvers
- Configurable approval threshold
- Rate limiting per approver (24h default)
- Time-limited access with expiry
- Uses shared `governance_commons::multi_sig` module

**Integration Points**:
- No integration with other governance contracts
- Operates independently for emergency access
- Can be called by external systems (EHR, telemedicine, etc.)

**Decision**: Use EmergencyAccessOverride for **emergency access control** requiring multi-sig from trusted approvers

## Decision Tree: Which Mechanism to Use

```
┌─ Is this a governance/proposal decision?
│  ├─ YES
│  │  ├─ Do token holders need to vote?
│  │  │  ├─ YES → Use GOVERNOR
│  │  │  │         (voting delay, voting period, quorum, voting power)
│  │  │  │
│  │  │  └─ NO → Use custom vote (e.g., committee-specific)
│  │  │
│  │  └─ Can this decision be challenged/disputed?
│  │     ├─ YES → Also use DISPUTE_RESOLUTION
│  │     │        (arbiters can override via challenge period)
│  │     │
│  │     └─ NO → Continue below
│  │
│  └─ NO
│     ├─ Is this a CONTRACT UPGRADE?
│     │  ├─ YES → Use UPGRADE_MANAGER
│     │  │        (validator multi-sig with optional time-delay)
│     │  │
│     │  └─ NO → Is this EMERGENCY ACCESS control?
│     │     ├─ YES → Use EMERGENCY_ACCESS_OVERRIDE
│     │     │        (approver multi-sig with rate-limiting)
│     │     │
│     │     └─ NO → Does this need TIME-DELAY gate?
│     │        ├─ YES → Use TIMELOCK
│     │        │        (queue, then execute after delay)
│     │        │
│     │        └─ NO → Use custom authorization
│     │               (require_auth, specific roles)
│
└─ End: Choose mechanism(s) based on path
```

## Usage Patterns by Scenario

### Scenario 1: Standard Governance Decision
**Example**: Update protocol parameters

1. **Proposer** calls `Governor::propose()` with parameter changes
2. **Token holders** call `Governor::cast_vote()` during voting period
3. **Anyone** calls `Governor::queue()` if vote succeeded
4. **Anyone** calls `Governor::execute()` after timelock delay

**Contracts Used**: Governor → Timelock → Target Contract

**Dispute**: If disputed, `DisputeResolution::dispute()` by challenger, `DisputeResolution::resolve()` by arbiter

**Contracts Used**: DisputeResolution

### Scenario 2: Contract Upgrade
**Example**: Deploy new logic for medical records contract

1. **Upgrader** calls `UpgradeManager::propose_upgrade()`
2. **Validators** call `UpgradeManager::approve_upgrade()` (multiple calls)
3. Once threshold reached, anyone calls `UpgradeManager::execute_upgrade()`

**Contracts Used**: UpgradeManager

**Note**: Governor is NOT involved; UpgradeManager operates independently

### Scenario 3: Emergency Medical Access
**Example**: Cardiologist needs emergency access to patient records

1. **Cardiologist** calls `EmergencyAccessOverride::grant_emergency_access()`
2. **System** checks approval threshold from configured approvers
3. Access is granted for specified duration
4. Access auto-revokes after expiry or manual revocation

**Contracts Used**: EmergencyAccessOverride

**Note**: Governor/UpgradeManager are NOT involved; fully independent

### Scenario 4: Sensitive Transaction with Time Delay
**Example**: Large fund transfer that needs safety buffer

1. **Authorized party** calls `Timelock::queue()` with transaction details
2. **System** calculates execution time (current time + delay)
3. After delay passes, `Timelock::execute()` can be called

**Contracts Used**: Timelock

**Note**: Often used BY Governor or UpgradeManager, not directly by users

## Integration Constraints

### ✅ Allowed Integrations

| Caller | Called | Reason |
|--------|--------|--------|
| Governor | Timelock | Governor queues proposals via timelock |
| Governor | DisputeResolution | Governor checks if proposals are disputed |
| UpgradeManager | Timelock | UpgradeManager can optionally use timelock |
| Any | Timelock | Timelock is a generic execution gate |
| Emergency System | EmergencyAccessOverride | Direct calls from emergency systems |

### ❌ Avoid These Cross-Contracts

| Anti-Pattern | Why | Alternative |
|--------------|-----|-------------|
| EmergencyAccessOverride calls Governor | Emergency access ≠ governance decision | EmergencyAccessOverride is independent |
| UpgradeManager calls Governor | Upgrades shouldn't need voting | UpgradeManager has own validator approval |
| DisputeResolution calls Governor | Would create circular dependency | Governor calls DisputeResolution (one-way) |
| Governor calls UpgradeManager | Protocol upgrades ≠ governance | Use UpgradeManager independently |

## Multi-Sig Pattern Usage

The shared `governance_commons::multi_sig` module consolidates multi-sig patterns used by:

1. **UpgradeManager** - Validator multi-sig for upgrades
2. **EmergencyAccessOverride** - Approver multi-sig for emergency access
3. **Future governance contracts** - Any contract needing multi-sig

### Using the Multi-Sig Module

```rust
use governance_commons::multi_sig;

// In initialize:
multi_sig::validate_approval_set(&members, threshold)?;

// When an approver submits:
multi_sig::validate_approver(&approver, &members)?;
if multi_sig::add_approval(&env, approver, &mut approvers) {
    // First approval, track it
}

// Check if ready:
let status = multi_sig::check_approval_status(&approvers, threshold, executed);
if status == ApprovalStatus::Ready {
    // Can execute now
}
```

## Error Handling Strategy

### Governor Errors
- `ProposalNotFound` - Proposal doesn't exist
- `ProposalDisputed` - Execution blocked by active dispute
- `ProposalThresholdNotMet` - Proposer lacks voting power
- `QuorumNotMet` - Not enough votes cast
- `ProposalNotSuccessful` - Voting didn't pass

### DisputeResolution Errors
- `NotArbiter` - Caller not in arbiter set
- `DisputeNotFound` - No active dispute for proposal

### UpgradeManager Errors
- `InvalidThreshold` - Threshold invalid for validators
- `NotApprover` - Caller not a validator

### EmergencyAccessOverride Errors
- `InvalidThreshold` - Threshold invalid for approvers
- `RateLimitExceeded` - Approver exceeded rate limit

## Migration Guide for Existing Contracts

If integrating with governance system:

1. **Determine your use case** (see Decision Tree above)
2. **Choose primary mechanism** (Governor, UpgradeManager, etc.)
3. **Add optional dispute checking** if applicable
4. **Add optional timelock delay** if needed
5. **Test integration paths** (see Integration Tests)

## Next Steps

- [ ] Review existing contracts for governance patterns
- [ ] Identify contracts that need integration
- [ ] Create integration test suite
- [ ] Document additional integration points
- [ ] Create runbook for governance operations

## References

- [Governor Contract](/contracts/governor/)
- [DisputeResolution Contract](/contracts/dispute_resolution/)
- [UpgradeManager Contract](/contracts/upgrade_manager/)
- [EmergencyAccessOverride Contract](/contracts/emergency_access_override/)
- [Timelock Contract](/contracts/timelock/)
- [Governance Commons Library](/libs/governance_commons/)
