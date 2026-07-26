# Governance Commons Library

## Overview

The `governance_commons` library consolidates shared utilities and patterns for governance-related contracts in the Uzima platform. It provides:

- **Multi-Sig Module**: Consolidated approval logic for multi-signature operations
- **Common Types**: Shared data structures for approvals, statuses, and configurations
- **Error Types**: Standardized error handling across governance contracts
- **Decision Tree Documentation**: Guidance on choosing the right mechanism

## Location

- Library: `libs/governance_commons/`
- Version: 0.1.0

## Components

### 1. Multi-Sig Module (`multi_sig.rs`)

Consolidates multi-sig approval patterns previously scattered across:
- `UpgradeManager` - Validator approvals for upgrades
- `EmergencyAccessOverride` - Approver multi-sig for emergency access
- Future governance contracts

#### Functions

##### `validate_approval_set(members, threshold) -> Result<(), GovernanceError>`

Validates that an approval set configuration is valid.

```rust
let members = vec![validator_1, validator_2, validator_3];
governance_commons::multi_sig::validate_approval_set(&members, 3)?;
```

**Validates**:
- Threshold > 0
- Threshold ≤ member count

---

##### `validate_approver(approver, members) -> Result<(), GovernanceError>`

Checks if an address is in the approval set.

```rust
use governance_commons::multi_sig;

if let Err(_) = multi_sig::validate_approver(&caller, &validators) {
    return Err(Error::NotApprover);
}
```

---

##### `is_already_approved(address, approvers) -> bool`

Fast check if address is already in approval list.

```rust
if multi_sig::is_already_approved(&approver, &approvers) {
    return Err(Error::DuplicateApproval);
}
```

---

##### `add_approval(env, approver, approvers) -> bool`

Adds approval if not already present. Returns `true` if newly added, `false` if duplicate.

```rust
let mut approvers = Vec::new(&env);
if multi_sig::add_approval(&env, approver, &mut approvers) {
    // First approval from this approver
    env.events().publish(...);
} else {
    // Already approved
    return Err(Error::AlreadyApproved);
}
```

---

##### `check_approval_status(approvers, threshold, executed) -> ApprovalStatus`

Returns current approval status: `Pending`, `Ready`, `Executed`, or `NotFound`.

```rust
let status = multi_sig::check_approval_status(&approvers, threshold, false);

match status {
    ApprovalStatus::Pending => {
        // Need more approvals
    }
    ApprovalStatus::Ready => {
        // Can execute now
    }
    ApprovalStatus::Executed => {
        // Already executed
    }
    ApprovalStatus::NotFound => {
        // Record not found
    }
}
```

---

### 2. Common Types (`types.rs`)

#### `ApprovalRecord`

Represents a multi-sig approval record.

```rust
#[contracttype]
pub struct ApprovalRecord {
    pub item_id: u64,              // proposal_id, upgrade_id, etc.
    pub approvers: Vec<Address>,    // Who has approved
    pub threshold: u32,             // Required approvals
    pub executed: bool,             // Has this item been executed?
    pub created_at: u64,            // Timestamp
}
```

#### `ApprovalStatus`

Enum representing approval status:
- `Pending = 0` - Not enough approvals yet
- `Ready = 1` - Threshold reached, ready to execute
- `Executed = 2` - Already executed
- `NotFound = 3` - Record not found

#### `ApprovalSetConfig`

Configuration for an approval set:

```rust
#[contracttype]
pub struct ApprovalSetConfig {
    pub members: Vec<Address>,  // Approvers/validators
    pub threshold: u32,         // Required approvals
}
```

---

### 3. Error Types (`errors.rs`)

`GovernanceError` enum for consistent error handling:

| Error | Code | Meaning |
|-------|------|---------|
| `AlreadyInitialized` | 1 | Contract already initialized |
| `NotInitialized` | 2 | Contract not initialized |
| `NotAuthorized` | 3 | Caller not authorized |
| `NotApprover` | 4 | Caller not in approval set |
| `InsufficientApprovals` | 5 | Not enough approvals yet |
| `InvalidThreshold` | 6 | Invalid threshold (0 or > count) |
| `NotFound` | 7 | Item not found |
| `OperationFailed` | 8 | Operation failed (generic) |
| `DuplicateEntry` | 9 | Duplicate entry |
| `InvalidInput` | 10 | Invalid input |

---

## Usage Patterns

### Pattern 1: Multi-Sig Upgrade Approval (UpgradeManager)

```rust
use governance_commons::multi_sig;
use governance_commons::{ApprovalStatus, GovernanceError};

pub fn approve_upgrade(
    env: Env,
    validator: Address,
    upgrade_id: u64,
) -> Result<ApprovalStatus, GovernanceError> {
    // 1. Verify validator is authorized
    let config = get_config(&env)?;
    multi_sig::validate_approver(&validator, &config.validators)?;

    // 2. Load existing approvals for this upgrade
    let mut approvals = load_approvals(&env, upgrade_id)
        .unwrap_or(Vec::new(&env));

    // 3. Add this approval (prevents duplicates)
    multi_sig::add_approval(&env, validator, &mut approvals)?;

    // 4. Save updated approvals
    save_approvals(&env, upgrade_id, &approvals);

    // 5. Check if we've reached threshold
    let status = multi_sig::check_approval_status(
        &approvals,
        config.threshold,
        false,
    );

    if status == ApprovalStatus::Ready {
        env.events().publish(
            (symbol_short!("ready"), upgrade_id),
            approvals.len(),
        );
    }

    Ok(status)
}
```

### Pattern 2: Multi-Sig Emergency Access (EmergencyAccessOverride)

```rust
use governance_commons::multi_sig;
use governance_commons::ApprovalStatus;

pub fn grant_emergency_access(
    env: Env,
    approver: Address,
    patient: Address,
    provider: Address,
    duration: u64,
) -> Result<bool, Error> {
    // 1. Verify approver is in trusted set
    let config = get_config(&env)?;
    multi_sig::validate_approver(&approver, &config.approvers)?;

    // 2. Check rate limiting
    check_cooldown(&env, &approver)?;

    // 3. Load existing approvals
    let access_key = get_access_key(&patient, &provider);
    let mut approvals = load_approvals(&env, &access_key)
        .unwrap_or(Vec::new(&env));

    // 4. Add approval
    multi_sig::add_approval(&env, approver, &mut approvals)?;

    // 5. Check approval status
    let status = multi_sig::check_approval_status(
        &approvals,
        config.threshold,
        false,
    );

    let can_grant = status == ApprovalStatus::Ready;

    if can_grant {
        // Grant access with expiry
        set_access_grant(&env, &patient, &provider, duration);
        env.events().publish((symbol_short!("access"), &patient), can_grant);
    }

    Ok(can_grant)
}
```

---

## Migration Guide: Using Governance Commons

### For UpgradeManager

**Before**: Custom multi-sig approval logic

```rust
// Old way: inline approval tracking
let mut approvals: Vec<Address> = env
    .storage()
    .persistent()
    .get(&DataKey::Approvals(proposal_id))
    .unwrap_or(Vec::new(&env));

if approvals.contains(&validator) {
    return Err(Error::AlreadyApproved);
}

approvals.push_back(validator);

if approvals.len() >= config.required_approvals {
    // Ready to execute
}
```

**After**: Using governance_commons::multi_sig

```rust
use governance_commons::multi_sig;

// New way: use shared validation
multi_sig::validate_approver(&validator, &config.validators)?;

let mut approvals = load_approvals(&env, upgrade_id)?;
multi_sig::add_approval(&env, validator, &mut approvals)?;

let status = multi_sig::check_approval_status(
    &approvals,
    config.threshold,
    false,
);

if status == ApprovalStatus::Ready {
    // Ready to execute
}
```

### For EmergencyAccessOverride

**Before**: Custom multi-sig with rate limiting

```rust
// Old way: manual approval tracking
let mut record = env
    .storage()
    .persistent()
    .get(&key)
    .unwrap_or_default();

if record.approvers.contains(&approver) {
    return Err(Error::AlreadyApproved);
}

record.approvers.push_back(approver);
```

**After**: Using governance_commons::multi_sig

```rust
use governance_commons::multi_sig;

let mut approvers = record.approvers.clone();
multi_sig::add_approval(&env, approver, &mut approvers)?;

let status = multi_sig::check_approval_status(
    &approvers,
    config.threshold,
    false,
);
```

---

## Benefits of Consolidation

1. **Code Reuse**: Eliminate duplicate multi-sig logic
2. **Consistency**: All contracts use same approval patterns
3. **Maintenance**: Bug fixes and improvements in one place
4. **Testing**: Shared test cases for multi-sig logic
5. **Documentation**: Clear, centralized documentation
6. **Extensibility**: Easy to add new multi-sig contracts

---

## Dependencies

Add to `Cargo.toml`:

```toml
[dependencies]
governance_commons = { path = "../../../libs/governance_commons" }
```

---

## Future Enhancements

- [ ] Add weighted voting (different approval weights)
- [ ] Add time-based approval expiry
- [ ] Add approval withdrawal/revocation
- [ ] Add approval quorum (minimum participation)
- [ ] Add role-based approval sets
- [ ] Add approval delegation

---

## References

- [Governance Architecture](../GOVERNANCE_ARCHITECTURE.md)
- [Issue #769](https://github.com/Stellar-Uzima/Uzima-Contracts/issues/769)
