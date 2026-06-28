# Governance Contracts - Detailed Refactoring Guide

## Overview

This document provides specific guidance for completing the refactoring of governance contracts to properly separate concerns and eliminate duplicate logic.

## Issue Reference

- **Issue #769**: Dispute Resolution Contract Overlaps with Governance — Unclear Separation of Concerns

## Current State Analysis

### Governor Contract

**Current Status**: ✅ PARTIALLY GOOD - Correctly delegates dispute checking to DisputeResolution

**Governor's Role**:
- Manages voting (proposal, cast_vote, queue, execute)
- Checks proposal state, including disputes
- Delegates actual dispute logic to DisputeResolution

**Dispatch Pattern in Governor** (lines 228-232 and 288-292):

```rust
// When checking proposal state
if let Some(dispute_addr) = cfg.dispute_contract {
    let args = vec![&env, proposal_id.into_val(&env)];
    let is_disputed: bool = env.invoke_contract(
        &dispute_addr,
        &Symbol::new(&env, "is_disputed"),
        args
    );
    if is_disputed {
        return Ok(6); // State: DISPUTED
    }
}
```

**Verdict**: ✅ Good - Follows clean delegation pattern
- Governor does NOT have duplicate dispute logic
- Governor delegates to DisputeResolution
- One-way dependency: Governor → DisputeResolution (not circular)

---

### DisputeResolution Contract

**Current Status**: ✅ GOOD - Focused responsibility

**Responsibility**:
- Manage disputes on proposals
- Allow arbiters to resolve disputes
- Track dispute status

**Key Functions**:
- `dispute(proposal_id, challenger)` - Challenge a proposal
- `resolve(proposal_id, arbiter, valid_proposal)` - Arbiter resolves
- `is_disputed(proposal_id)` - Check status

**Verdict**: ✅ Good - Single, well-defined responsibility
- Clean interface for Governor to use
- No cross-contract dependencies to other governance contracts
- Works independently

---

### UpgradeManager Contract

**Current Status**: ⚠️ NEEDS REFACTORING - Uses custom multi-sig logic

**Current Implementation**:
- Has its own approval tracking: `approvals: Vec<Address>`
- Has its own threshold checking
- Has its own validation logic
- Does NOT use governance_commons library (which didn't exist before)

**Duplicate Logic**:
```rust
// UpgradeManager has custom version of multi-sig approval tracking
pub fn approve_upgrade(...) {
    // Manual approval collection
    let mut approvals = ...;
    if approvals.contains(&validator) {
        return Err(Error::DuplicateApproval);
    }
    approvals.push_back(validator);
    
    // Manual threshold check
    if approvals.len() >= config.required_approvals {
        // ready to execute
    }
}
```

**Verdict**: ⚠️ Needs Refactoring
- **Action**: Update to use `governance_commons::multi_sig` module
- **Benefit**: Eliminates duplicate code, improves consistency

---

### EmergencyAccessOverride Contract

**Current Status**: ⚠️ NEEDS REFACTORING - Uses custom multi-sig logic + rate limiting

**Current Implementation**:
- Has own approval tracking
- Has own threshold checking
- Has rate limiting (per-approver cooldown)
- Does NOT use governance_commons library

**Duplicate Logic**:
```rust
// EmergencyAccessOverride has custom version of multi-sig
pub fn grant_emergency_access(...) {
    // Manual approval collection + rate limiting
    let mut approvers = record.approvers.clone();
    if approvers.contains(&approver) {
        return Err(Error::AlreadyApproved);
    }
    
    // Check cooldown
    let cooldown_period = ...;
    let last_used = ...;
    if now < last_used.saturating_add(cooldown_period) {
        return Err(Error::RateLimitExceeded);
    }
    
    approvers.push_back(approver);
    // ... rest
}
```

**Verdict**: ⚠️ Needs Refactoring
- **Action**: Use `governance_commons::multi_sig` for core approval logic
- **Note**: Retain rate limiting logic (it's specific to emergency access)

---

## Refactoring Plan

### Phase 1: Verify Governor (Already Good)

**Status**: ✅ No changes needed

**What's correct**:
- Governor properly delegates to DisputeResolution
- No duplicate dispute logic in Governor
- Clean one-way dependency

**Verification**:
- [x] Governor calls `dispute_contract.is_disputed()` before state transitions
- [x] Governor properly handles dispute state (state = 6 when disputed)
- [x] Governor allows proposal queue only when not disputed

---

### Phase 2: Refactor UpgradeManager

**Current File**: `contracts/upgrade_manager/src/lib.rs`

**Changes Required**:

1. **Add Dependency** in `Cargo.toml`:
```toml
[dependencies]
governance_commons = { path = "../../libs/governance_commons" }
```

2. **Import Module** in `lib.rs`:
```rust
use governance_commons::multi_sig;
use governance_commons::{ApprovalStatus, ApprovalRecord};
```

3. **Update `approve_upgrade()` Function**:

**Before**:
```rust
pub fn approve_upgrade(env: Env, validator: Address, proposal_id: u64) -> Result<(), Error> {
    let mut approvals = load_approvals(&env, proposal_id)?;
    
    if approvals.contains(&validator) {
        return Err(Error::DuplicateApproval);
    }
    approvals.push_back(validator);
    
    if approvals.len() >= config.required_approvals {
        // Ready
    }
}
```

**After**:
```rust
pub fn approve_upgrade(env: Env, validator: Address, proposal_id: u64) -> Result<(), Error> {
    let config = get_config(&env)?;
    
    // Use shared validation
    multi_sig::validate_approver(&validator, &config.validators)?;
    
    let mut approvals = load_approvals(&env, proposal_id)?;
    
    // Use shared approval logic
    if !multi_sig::add_approval(&env, validator, &mut approvals) {
        return Err(Error::DuplicateApproval);
    }
    
    save_approvals(&env, proposal_id, &approvals);
    
    // Use shared status checking
    let status = multi_sig::check_approval_status(
        &approvals,
        config.required_approvals,
        false,
    );
    
    if status == ApprovalStatus::Ready {
        emit_ready_event(&env, proposal_id);
    }
    
    Ok(())
}
```

4. **Update Error Types** in `errors.rs`:
   - Use `GovernanceError::NotApprover` instead of custom error
   - Consolidate other error types

---

### Phase 3: Refactor EmergencyAccessOverride

**Current File**: `contracts/emergency_access_override/src/lib.rs`

**Changes Required**:

1. **Add Dependency** in `Cargo.toml`:
```toml
[dependencies]
governance_commons = { path = "../../libs/governance_commons" }
```

2. **Import Module** in `lib.rs`:
```rust
use governance_commons::multi_sig;
use governance_commons::ApprovalStatus;
```

3. **Update `grant_emergency_access()` Function**:

**Before**:
```rust
pub fn grant_emergency_access(
    env: Env,
    approver: Address,
    patient: Address,
    provider: Address,
    duration_seconds: u64,
) -> Result<bool, Error> {
    // Manual approver validation
    let is_trusted = env.storage().persistent().get(
        &DataKey::TrustedApprover(approver.clone())
    );
    if is_trusted != Some(true) {
        return Err(Error::Unauthorized);
    }
    
    // Manual approval tracking
    let mut approvers = record.approvers.clone();
    if approvers.contains(&approver) {
        return Err(Error::AlreadyApproved);
    }
    approvers.push_back(approver);
    
    // Manual threshold check
    if approvers.len() >= config.approval_threshold {
        // Grant access
    }
}
```

**After**:
```rust
pub fn grant_emergency_access(
    env: Env,
    approver: Address,
    patient: Address,
    provider: Address,
    duration_seconds: u64,
) -> Result<bool, Error> {
    let config = get_config(&env)?;
    
    // Use shared approver validation
    multi_sig::validate_approver(&approver, &config.approvers)
        .map_err(|_| Error::Unauthorized)?;
    
    // Rate limiting (KEEP THIS - specific to emergency access)
    check_cooldown(&env, &approver)?;
    
    let mut approvers = record.approvers.clone();
    
    // Use shared approval logic
    multi_sig::add_approval(&env, approver, &mut approvers);
    
    // Use shared status checking
    let status = multi_sig::check_approval_status(
        &approvers,
        config.approval_threshold,
        false,
    );
    
    let can_grant = status == ApprovalStatus::Ready;
    
    if can_grant {
        set_access_grant(&env, &patient, &provider, duration_seconds);
    }
    
    Ok(can_grant)
}
```

4. **Consolidate Error Types**:
   - Use `GovernanceError::NotApprover` where applicable
   - Keep emergency-specific errors (e.g., `RateLimitExceeded`)

---

## Summary of Contracts After Refactoring

| Contract | Status | Responsibility | Dependencies |
|----------|--------|-----------------|--------------|
| Governor | ✅ No changes | Voting & proposals | → DisputeResolution, Timelock |
| DisputeResolution | ✅ No changes | Dispute arbitration | (none - independent) |
| UpgradeManager | ⚠️ Refactor | Contract upgrades | → governance_commons (multi_sig) |
| EmergencyAccessOverride | ⚠️ Refactor | Emergency access | → governance_commons (multi_sig) |
| Timelock | ✅ No changes | Time-delay gate | (none - independent) |

---

## Benefits After Refactoring

1. **Reduced Code Duplication**: 50+ lines of duplicate multi-sig logic eliminated
2. **Improved Consistency**: Both UpgradeManager and EmergencyAccessOverride use same patterns
3. **Easier Maintenance**: Multi-sig bugs fixed in one place
4. **Better Testing**: Shared test cases for multi-sig logic
5. **Clear Separation**: Each contract has single responsibility
6. **Documentation**: Clear decision tree for choosing mechanism

---

## Testing Strategy

### Unit Tests (Per Contract)

- Governor: Vote, dispute, state transitions
- DisputeResolution: Dispute, resolve, is_disputed
- UpgradeManager: Propose, approve, execute (using multi_sig)
- EmergencyAccessOverride: Grant, rate limiting, approval (using multi_sig)

### Integration Tests

See `contracts/governance_integration_tests/mod.rs` for:
- Governor → DisputeResolution integration
- Governor → Timelock integration
- UpgradeManager independent operation
- EmergencyAccessOverride independent operation

---

## Rollout Plan

### Step 1: Create Governance Commons Library
- [x] Create `libs/governance_commons`
- [x] Implement multi_sig module
- [x] Add tests

### Step 2: Document Architecture
- [x] Create `GOVERNANCE_ARCHITECTURE.md` with decision tree
- [x] Create `governance_commons/README.md` with usage patterns
- [x] Create integration tests documentation

### Step 3: Refactor UpgradeManager (Future PR)
- [ ] Add governance_commons dependency
- [ ] Update `approve_upgrade()` to use multi_sig module
- [ ] Update error types
- [ ] Run tests, verify no regressions

### Step 4: Refactor EmergencyAccessOverride (Future PR)
- [ ] Add governance_commons dependency
- [ ] Update `grant_emergency_access()` to use multi_sig module
- [ ] Keep rate limiting logic
- [ ] Run tests, verify no regressions

### Step 5: Verify Governor (Final Check)
- [ ] Confirm Governor properly delegates to DisputeResolution
- [ ] No duplicate dispute logic in Governor
- [ ] Update Governor tests if needed

---

## Success Criteria

- [x] Clear separation of concerns documented
- [x] Decision tree guides users to correct mechanism
- [x] Shared multi-sig module eliminates duplication
- [x] Governor properly delegates to DisputeResolution
- [x] Integration tests demonstrate correct interactions
- [x] Each contract has single, well-defined responsibility
- [ ] (Future) UpgradeManager and EmergencyAccessOverride refactored to use shared module
- [ ] (Future) All governance contracts integrated tested

---

## References

- [Governor Contract](../../contracts/governor/src/)
- [DisputeResolution Contract](../../contracts/dispute_resolution/src/)
- [UpgradeManager Contract](../../contracts/upgrade_manager/src/)
- [EmergencyAccessOverride Contract](../../contracts/emergency_access_override/src/)
- [Governance Commons Library](../governance_commons/)
- [Governance Architecture](../GOVERNANCE_ARCHITECTURE.md)
