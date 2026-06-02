# Event Standardization Policy

## Overview

This document defines the standardized event emission pattern for all Stellar Uzima contracts. Events are critical for external monitoring, indexing, and cross-contract coordination.

## Objectives

1. **Consistency**: All contracts follow the same event emission patterns
2. **Discoverability**: Event schema is documented and registerable
3. **Auditability**: Every state-changing operation emits events
4. **Correlation**: Events can be correlated across contracts using standard topic naming

## Event Naming Convention

Event names follow the pattern: `CONTRACT_NAME:ACTION`

### Topic Structure

Each event publication uses two-level topics:

```rust
env.events().publish(
    (symbol_short!("CONTRACT"), symbol_short!("ACTION")),
    data
);
```

### Examples

- `MEDRC:CREATE` - Medical Records contract, record creation
- `RBAC:ASSIGN` - RBAC contract, role assignment
- `APPT:BOOK` - Appointment Booking, appointment booked
- `NOTIF:SEND` - Notification System, notification sent

## Standard Event Data Structure

Each contract's `events.rs` MUST define:

1. **EventType enum** - All event types emitted by the contract
2. **OperationCategory enum** - Categories: UserManagement, RecordOperations, AccessControl, Administrative, etc.
3. **ContractEventData struct** - Domain-specific event data
4. **ContractEvent struct** - Envelope containing metadata + data
5. **emit\_\* functions** - Helper functions to emit each event type

### Mandatory Fields in ContractEvent

```rust
pub struct ContractEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,        // env.ledger().timestamp()
    pub user_id: Address,      // Caller or principal address
    pub block_height: u64,     // env.ledger().sequence() as u64
    pub data: ContractEventData,
}
```

## State-Changing Operations

Every state-changing function MUST emit at least one event. This includes:

- Initialization
- User/role management
- Data creation, modification, deletion
- Access grants/revocations
- Administrative actions
- Status changes

## Event Registry

All contracts register their events in `schemas/events/registry.json`:

```json
{
  "contracts": {
    "CONTRACT_NAME": {
      "description": "...",
      "events": [
        {
          "name": "ACTION",
          "topic": "CONTRACT_NAME:ACTION",
          "symbol": "ACTION_SHORT",
          "triggered_by": "function_name()",
          "data_structure": "ContractEventData",
          "description": "..."
        }
      ]
    }
  }
}
```

## Validation

All contracts MUST pass the event validation test:

```bash
cargo test --test event_schema_validation
```

This test verifies:

- Each contract has an `events.rs` file
- All event types are properly documented
- Every state-changing function emits events
- Event names follow the naming convention
- Event registry is complete and accurate

## Migration Guide

For existing contracts without `events.rs`:

1. Create `src/events.rs` following the template pattern
2. Define all event types and data structures
3. Call `emit_*` functions from state-changing operations
4. Register events in `schemas/events/registry.json`
5. Add event documentation to contract README
6. Verify with validation test

## Best Practices

1. **Emit early**: Emit events after state mutations but before returning
2. **Include context**: Always include user_id and timestamp for auditability
3. **Atomic events**: Each logical action = one event
4. **No sensitive data**: Never emit encrypted/confidential data
5. **Immutable records**: Treat events as immutable audit trail
6. **Cross-contract reference**: Include contract name in topic for correlation

## Future Considerations

- Event streaming to external indexers
- Real-time alerting on critical events
- Cross-chain event synchronization
- Event replay for contract state recovery
