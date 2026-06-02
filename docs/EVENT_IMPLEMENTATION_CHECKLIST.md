# Event Implementation Checklist

Use this checklist during contract development and review to ensure proper event standardization.

## Pre-Implementation

- [ ] Identify all state-changing functions in contract
- [ ] Define EventType enum with all possible events
- [ ] Define OperationCategory values for contract
- [ ] Design event data structures for each event type
- [ ] Plan contract symbol (3-6 uppercase chars)
- [ ] Plan action symbols (3-7 uppercase chars) for each event

## Implementation

### events.rs File

- [ ] Create `src/events.rs` file
- [ ] Define `EventType` enum with all event types
- [ ] Define `OperationCategory` enum with relevant categories
- [ ] Define `*EventData` struct with domain-specific fields
- [ ] Define `*Event` struct with required metadata fields:
  - [ ] `event_type: EventType`
  - [ ] `category: OperationCategory`
  - [ ] `timestamp: u64` (from `env.ledger().timestamp()`)
  - [ ] `user_id: Address` (caller or principal)
  - [ ] `block_height: u64` (from `env.ledger().sequence() as u64`)
  - [ ] `data: *EventData`
- [ ] Implement `emit_*` function for each event type
- [ ] Use `symbol_short!()` for topics (max 10 chars each)
- [ ] Document each event function with doc comments

### lib.rs File

- [ ] Add `mod events;` declaration
- [ ] Import emit functions: `use events::*;`
- [ ] Call appropriate `emit_*` function in each state-changing function
- [ ] Emit events AFTER state changes (but within same transaction)
- [ ] Ensure events are emitted on success path only
- [ ] Document which events are emitted by each function

### Testing

- [ ] Write unit tests for event emission
- [ ] Verify events contain correct data
- [ ] Test events with different input values
- [ ] Verify error conditions don't emit events
- [ ] Run event validation test: `cargo test event_schema_validation`

## Documentation

### Contract README

- [ ] Add "Events" section to README
- [ ] List all events with descriptions
- [ ] Document when each event is emitted
- [ ] Include example event data (if helpful)

### Schema Registry

- [ ] Update `schemas/events/registry.json`
- [ ] Add contract entry with description
- [ ] List all events with:
  - [ ] `name`: Event name
  - [ ] `topic`: Full topic string (CONTRACT:ACTION)
  - [ ] `symbol`: Action symbol
  - [ ] `triggered_by`: Function name that emits it
  - [ ] `data_structure`: Name of event data struct
  - [ ] `description`: Clear description

## Code Review - Event Implementation

### Structure Validation

- [ ] EventType enum exists
- [ ] OperationCategory enum exists
- [ ] Event data struct has appropriate fields
- [ ] Event envelope struct has all required fields
- [ ] emit\_\* functions exist for all event types
- [ ] No inline event emissions (all use emit\_\* functions)

### Best Practices

- [ ] All state-changing functions emit events ✓
- [ ] Events emitted AFTER state changes ✓
- [ ] No sensitive/encrypted data in events ✓
- [ ] Correct timestamp and user_id values ✓
- [ ] Symbol_short! used consistently ✓
- [ ] Event topics follow naming convention ✓
- [ ] Data structures are Soroban-compatible ✓

### Naming Convention

- [ ] Contract symbol: 3-6 uppercase letters
- [ ] Action symbol: 3-7 uppercase letters
- [ ] Pattern: `symbol_short!("SYMBOL"), symbol_short!("ACTION")`
- [ ] Examples follow existing patterns

### Error Handling

- [ ] Events not emitted on failed operations ✓
- [ ] Events not emitted on unauthorized operations ✓
- [ ] Error paths don't call emit\_\* functions ✓

## Integration

### Event Registry

- [ ] Contract added to `schemas/events/registry.json`
- [ ] All events documented in registry
- [ ] Event descriptions clear and accurate
- [ ] Triggered_by functions are correct

### Cross-Contract References

- [ ] Event topics are unique across contracts
- [ ] No symbol collisions with other contracts
- [ ] Can correlate events across contracts if needed

## Testing After Implementation

```bash
# Run event validation tests
cargo test event_schema_validation -- --nocapture

# Run contract-specific tests
cargo test contracts::your_contract --test '*'

# Check event emissions
grep -r "emit_" contracts/your_contract/src/
```

## Common Issues to Avoid

### ❌ Don't

- [ ] Don't emit before validation ✓
- [ ] Don't use symbols longer than 10 chars ✓
- [ ] Don't forget to emit for state-changing functions ✓
- [ ] Don't include sensitive data ✓
- [ ] Don't use inconsistent naming conventions ✓
- [ ] Don't emit from read-only functions ✓

### ✅ Do

- [ ] Do emit after state successfully changes ✓
- [ ] Do include all relevant context ✓
- [ ] Do document event purposes ✓
- [ ] Do test event emission ✓
- [ ] Do follow naming conventions ✓
- [ ] Do register events in schema ✓

## Sign-Off

- [ ] All requirements met
- [ ] Tests passing
- [ ] Code review completed
- [ ] Documentation updated
- [ ] Ready for merge

---

## Event Validation Script

```bash
#!/bin/bash
# Check contract events

CONTRACT=$1

if [ -z "$CONTRACT" ]; then
    echo "Usage: $0 <contract_name>"
    exit 1
fi

echo "Checking events in contracts/$CONTRACT..."

# Check for events.rs
if [ -f "contracts/$CONTRACT/src/events.rs" ]; then
    echo "✓ events.rs exists"
else
    echo "✗ events.rs missing"
fi

# Check for emit_* functions
EMITS=$(grep -c "pub fn emit_" contracts/$CONTRACT/src/events.rs 2>/dev/null || echo "0")
echo "  Found $EMITS emit_* functions"

# Check for EventType enum
if grep -q "enum EventType" contracts/$CONTRACT/src/events.rs; then
    echo "✓ EventType enum defined"
else
    echo "✗ EventType enum missing"
fi

# Check for Event struct
if grep -q "struct.*Event {" contracts/$CONTRACT/src/events.rs; then
    echo "✓ Event struct defined"
else
    echo "✗ Event struct missing"
fi

# Check lib.rs imports
if grep -q "mod events" contracts/$CONTRACT/src/lib.rs; then
    echo "✓ events module imported in lib.rs"
else
    echo "✗ events module not imported in lib.rs"
fi

# Check for emit calls in lib.rs
CALLS=$(grep -c "emit_" contracts/$CONTRACT/src/lib.rs 2>/dev/null || echo "0")
echo "  Found $CALLS emit_* calls in lib.rs"

echo ""
echo "✓ Validation complete"
```

Save as `scripts/check_events.sh` and run:

```bash
chmod +x scripts/check_events.sh
./scripts/check_events.sh your_contract
```
