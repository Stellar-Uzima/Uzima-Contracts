# Contract Lifecycle State Machine

All upgradeable contracts in the Uzima workspace share a common lifecycle
state machine implemented in `contracts/upgradeability/src/lifecycle.rs`.

## States

| State | Description | Writes allowed | Reads allowed |
|-------|-------------|----------------|---------------|
| `Uninitialized` | Deployed but `initialize()` not yet called | ❌ | ✅ |
| `Active` | Normal operation | ✅ | ✅ |
| `Paused` | Emergency stop | ❌ | ✅ |
| `Upgrading` | Upgrade in progress | ❌ (migration only) | ✅ |
| `Deprecated` | Permanently retired | ❌ | ❌ |

## State Transition Diagram

```
Uninitialized ──initialize()──► Active
                                  │
                         pause()  │  begin_upgrade()
                                  │
                 ┌────────────────┼─────────────────┐
                 ▼                ▼                  │
              Paused          Upgrading         Deprecated
                 │                │
         resume()│ complete/abort │
                 └────────────────┘
                          │
                          ▼
                        Active
```

Any state can transition to `Deprecated` (admin only, irreversible).

## Integration Guide

### Step 1 — Add the dependency

In your contract's `Cargo.toml`:

```toml
[dependencies]
upgradeability = { path = "../../contracts/upgradeability" }
```

### Step 2 — Use lifecycle guards

```rust
use upgradeability::lifecycle::{ContractLifecycle, ContractLifecycleState, LifecycleError};

// In initialize():
pub fn initialize(env: Env, admin: Address) {
    admin.require_auth();
    ContractLifecycle::transition(&env, ContractLifecycleState::Active);
    // ...
}

// In a write entrypoint:
pub fn create_record(env: Env, patient: Address, data: String) -> Result<u64, Error> {
    patient.require_auth();
    ContractLifecycle::require_active(&env)?;
    // ... proceed with write
}

// In admin pause():
pub fn pause(env: Env, admin: Address) {
    admin.require_auth();
    ContractLifecycle::transition(&env, ContractLifecycleState::Paused);
}

// In admin resume():
pub fn resume(env: Env, admin: Address) {
    admin.require_auth();
    ContractLifecycle::transition(&env, ContractLifecycleState::Active);
}
```

### Step 3 — Upgrade flow

```rust
// Admin calls begin_upgrade() — transitions to Upgrading state
pub fn begin_upgrade(env: Env, admin: Address, new_wasm: BytesN<32>) {
    admin.require_auth();
    ContractLifecycle::transition(&env, ContractLifecycleState::Upgrading);
    env.deployer().update_current_contract_wasm(new_wasm);
}

// After WASM is updated, the migration entry point runs, then:
pub fn complete_upgrade(env: Env, admin: Address) {
    admin.require_auth();
    ContractLifecycle::transition(&env, ContractLifecycleState::Active);
}
```

## Events

Every state transition emits a Soroban diagnostic event:

```
topics: ["lifecycle", "transition"]
data:   (old_state: u32, new_state: u32)
```

These events are indexed by the contract monitoring infrastructure and
trigger alerts when unexpected transitions occur.

## Error Codes

| Code | Constant | Meaning |
|------|----------|---------|
| 200 | `NotInitialized` | `initialize()` has not been called |
| 201 | `ContractPaused` | Emergency pause is active |
| 202 | `UpgradeInProgress` | Upgrade in progress |
| 203 | `ContractDeprecated` | Contract permanently retired |
| 204 | `InvalidTransition` | FSM transition not allowed |
