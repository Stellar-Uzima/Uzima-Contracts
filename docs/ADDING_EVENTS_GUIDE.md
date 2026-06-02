# Event Standardization Implementation Guide

## Overview

This guide walks developers through adding standardized events to Stellar Uzima contracts.

## Quick Start: Adding Events to Your Contract

### Step 1: Create `src/events.rs`

Copy the template from `docs/EVENTS_TEMPLATE.rs` and customize:

```rust
use soroban_sdk::{contracttype, symbol_short, Address, Env, String};

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum EventType {
    Initialized,
    YourEventType,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum OperationCategory {
    UserManagement,
    // Add relevant categories
}

#[derive(Clone)]
#[contracttype]
pub struct YourContractEventData {
    pub field1: SomeType,
    pub field2: Option<Address>,
}

#[derive(Clone)]
#[contracttype]
pub struct YourContractEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: YourContractEventData,
}

pub fn emit_initialized(env: &Env, admin: Address) {
    let event = YourContractEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: admin.clone(),
        block_height: env.ledger().sequence() as u64,
        data: YourContractEventData {
            field1: value1,
            field2: Some(admin),
        },
    };
    env.events()
        .publish((symbol_short!("YOUR"), symbol_short!("INIT")), event);
}
```

### Step 2: Update `src/lib.rs`

1. Add `mod events;` at the top
2. Call emit functions from state-changing operations

```rust
mod events;
use events::*;

#[contractimpl]
impl YourContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        // Checks and effects
        env.storage().instance().set(&DataKey::Admin, &admin);

        // Emit event
        emit_initialized(&env, admin);
        Ok(())
    }

    pub fn your_state_changing_function(
        env: Env,
        caller: Address,
        data: String,
    ) -> Result<(), Error> {
        caller.require_auth();

        // Validate and update state
        // ... your logic ...

        // Emit event - ALWAYS after successful state change
        emit_your_event(&env, caller, data);
        Ok(())
    }
}
```

### Step 3: Register Events in Schema

Update `schemas/events/registry.json`:

```json
"your_contract": {
  "description": "Your contract description",
  "symbol": "YOUR",
  "events": [
    {
      "name": "Initialized",
      "topic": "YOUR:INIT",
      "symbol": "INIT",
      "triggered_by": "initialize()",
      "data_structure": "YourContractEventData",
      "description": "Emitted when contract initialized"
    },
    {
      "name": "YourEventType",
      "topic": "YOUR:ACTION",
      "symbol": "ACTION",
      "triggered_by": "your_state_changing_function()",
      "data_structure": "YourContractEventData",
      "description": "Description of when this event is emitted"
    }
  ]
}
```

### Step 4: Document in Contract README

Add to `contracts/your_contract/README.md`:

```markdown
## Events

This contract emits the following events:

- **Initialized**: Emitted when contract is initialized
- **YourEventType**: Emitted when [operation] occurs

### Event Naming Convention

Events follow the pattern `CONTRACT:ACTION`:

- `YOUR:INIT` - Initialize
- `YOUR:ACTION` - Your action

### Event Schema

See `schemas/events/registry.json` for complete event schema.
```

## Event Emission Best Practices

### ✅ DO:

1. **Emit after state changes**: Emit events after updating storage to ensure consistency
2. **Include context**: Always include timestamp, user_id, and block_height
3. **Use descriptive data**: Include all relevant information for external indexers
4. **Atomic events**: One logical operation = one event (or multiple if multiple state changes)
5. **Document events**: Add inline comments explaining what triggers each event

```rust
// ✅ Good
pub fn update_status(env: &Env, user: Address, new_status: String) {
    // Validate
    user.require_auth();

    // Update state
    env.storage().persistent().set(&DataKey::Status, &new_status);

    // Emit event after successful state change
    emit_status_changed(&env, user, "pending", new_status);
}
```

### ❌ DON'T:

1. **Emit before validation**: Events should only be emitted for successful operations
2. **Include sensitive data**: Never emit encrypted or confidential information
3. **Create one event per field change**: Batch related changes into one event
4. **Forget to call emit functions**: Every state change needs an event
5. **Use undefined event types**: Always define new event types in your EventType enum

```rust
// ❌ Bad - emits before validating user
pub fn bad_update(env: &Env, user: Address, new_value: String) {
    emit_something(&env, user, new_value);  // Too early!

    if !user.require_auth().is_ok() {
        return;  // Authorization failed but event already emitted
    }
}
```

## Testing Your Events

### Unit Test

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{Env, testutils::Address as TestAddress};

    #[test]
    fn test_events_emitted() {
        let env = Env::default();
        let admin = TestAddress::random(&env);

        // Call your function
        initialize(&env, admin.clone()).unwrap();

        // Check events were published
        let events = env.events().all();
        assert!(!events.is_empty(), "No events emitted");

        // Verify event content
        let event = events.get(0).unwrap();
        assert_eq!(event.topics.get(0), &symbol_short!("YOUR"));
        assert_eq!(event.topics.get(1), &symbol_short!("INIT"));
    }
}
```

### Integration Test

```bash
# Run event validation tests
cargo test --test event_schema_validation

# Run contract-specific tests
cargo test your_contract --test '*'
```

## Troubleshooting

### Events not appearing

1. **Check event call**: Verify `emit_*` function is called
2. **Check placement**: Event should be emitted after state changes
3. **Check return path**: Function might be returning early before emit
4. **Check env scope**: Ensure you're using the same `env` reference

### "Event data is missing"

1. **Verify struct fields**: All required fields must be populated
2. **Check serialization**: Ensure data types are Soroban-compatible
3. **Verify address encoding**: Use `address.clone()` when needed

### "Symbol too long"

1. **Keep symbols short**: Max 10 characters recommended
2. **Use symbol_short!**: Enforces 10-char limit
3. **Examples**:
   - ✅ `symbol_short!("INIT")` - Good
   - ❌ `symbol_short!("INITIALIZATION")` - Too long

## Event Naming Reference

| Operation     | Pattern            | Example                      |
| ------------- | ------------------ | ---------------------------- |
| Create        | CREATE             | `MED:CREATE`                 |
| Update        | UPDATE or specific | `MED:UPDATE` or `MED:STATUS` |
| Delete        | DELETE             | `MED:DELETE`                 |
| Grant access  | GRANT              | `AC:GRANT`                   |
| Revoke access | REVOKE             | `AC:REVOKE`                  |
| Initialize    | INIT               | `SYS:INIT`                   |
| Submit        | SUBMIT             | `CLAIM:SUBMIT`               |
| Approve       | APPROVE            | `CLAIM:APPROVE`              |
| Execute       | EXECUTE            | `GOV:EXEC`                   |

## Validating Your Implementation

Run the validation suite:

```bash
# Test all event implementations
cargo test event_schema_validation -- --nocapture

# Check your specific contract
grep -r "emit_" contracts/your_contract/src/
```

## Resources

- [Stellar Soroban Events Documentation](https://developers.stellar.org/soroban/events)
- [Event Standardization Policy](EVENT_STANDARDIZATION_POLICY.md)
- [Event Registry](../../schemas/events/registry.json)
- [Event Template](EVENTS_TEMPLATE.rs)

## Support

Questions? Issues?

1. Check existing contracts for examples
2. Review event validation test failures
3. Ask in GitHub Discussions
4. Open an issue with your contract code
