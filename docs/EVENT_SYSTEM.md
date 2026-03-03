# Uzima Medical Records Event System

## Overview

The Uzima Medical Records contract implements a comprehensive event system for monitoring, auditing, and integration with external systems. This document provides detailed information about the event schema, implementation, and usage.

## Event Architecture

### Event Types

The system defines comprehensive event types covering all contract operations:

#### User Management Events
- `UserCreated` - New user account creation
- `UserRoleUpdated` - User role changes
- `UserDeactivated` - User account deactivation
- `UserActivated` - User account reactivation

#### Record Events
- `RecordCreated` - Medical record creation
- `RecordAccessed` - Record access/view operations
- `RecordUpdated` - Record modification
- `RecordDeleted` - Record deletion

#### Access Control Events
- `AccessRequested` - Access permission requests
- `AccessGranted` - Access permission approvals
- `AccessDenied` - Access permission denials
- `AccessRevoked` - Access permission revocation

#### Emergency Access Events
- `EmergencyAccessGranted` - Emergency access authorization
- `EmergencyAccessRevoked` - Emergency access removal
- `EmergencyAccessExpired` - Emergency access timeout

#### Administrative Events
- `ContractPaused` - Contract emergency pause
- `ContractUnpaused` - Contract resumption
- `RecoveryProposed` - Recovery proposal creation
- `RecoveryApproved` - Recovery proposal approval
- `RecoveryExecuted` - Recovery proposal execution
- `RecoveryRejected` - Recovery proposal rejection

#### AI Integration Events
- `AIConfigUpdated` - AI configuration changes
- `AnomalyScoreSubmitted` - AI anomaly detection results
- `RiskScoreSubmitted` - AI risk assessment results
- `AIAnalysisTriggered` - AI analysis initiation

#### Cross-chain Events
- `CrossChainSyncInitiated` - Cross-chain sync start
- `CrossChainSyncCompleted` - Cross-chain sync completion
- `CrossChainRecordReferenced` - Cross-chain record linking

#### System Events
- `HealthCheck` - System health monitoring
- `MetricUpdate` - Performance metrics

### Operation Categories

Events are categorized for easier filtering and analysis:

- `UserManagement` - User account operations
- `RecordOperations` - Medical record CRUD
- `AccessControl` - Permission management
- `EmergencyAccess` - Emergency protocols
- `Administrative` - Admin operations
- `AIIntegration` - AI system interactions
- `CrossChain` - Cross-chain operations
- `System` - System maintenance

## Event Schema

### Base Event Structure

```rust
pub struct BaseEvent {
    pub metadata: EventMetadata,
    pub data: EventData,
}
```

### Event Metadata

```rust
pub struct EventMetadata {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub session_id: Option<String>,
    pub ipfs_ref: Option<String>,
    pub gas_used: Option<u64>,
    pub block_height: u64,
}
```

### Event Data Variants

The `EventData` enum contains operation-specific information:

- `UserEvent` - User management data
- `RecordEvent` - Medical record data
- `AccessEvent` - Access control data
- `EmergencyEvent` - Emergency access data
- `RecoveryEvent` - Recovery proposal data
- `AIEvent` - AI operation data
- `CrossChainEvent` - Cross-chain operation data
- `SystemEvent` - System operation data

## Event Publishing

Events are published using Soroban SDK's event system with structured topics:

```rust
env.events().publish(("EVENT", symbol_short!("RECORD_CREATED")), event);
```

## Structured Logging (Soroban Events)

The medical records contract now includes a structured logging layer that emits dedicated log events for major operations.

### Log levels

- `Info` -> topic `("LOG", symbol_short!("LOG_INFO"))`
- `Warning` -> topic `("LOG", symbol_short!("LOG_WARN"))`
- `Error` -> topic `("LOG", symbol_short!("LOG_ERROR"))`

### Log payload schema

```rust
pub struct StructuredLog {
    pub timestamp: u64,
    pub level: LogLevel,
    pub operation: String,
    pub actor: Option<Address>,
    pub target_id: Option<Address>,
    pub record_id: Option<u64>,
    pub message: String,
}
```

### Coverage

Structured logs are emitted for:

- User management operations: `initialize`, `manage_user`, `deactivate_user`, permission grant/revoke
- Record operations: `add_record`, `add_record_with_did`, `get_record`, `get_record_with_did`, metadata update/import
- Administrative actions: pause/unpause, audit forensics configuration, rate-limit configuration, emergency/recovery flows

### Metadata included in every log

- `timestamp` from ledger time
- `actor` (caller when available)
- `operation` type string
- optional `target_id` and `record_id`
- human-readable `message`

### Testing

Logging behavior is verified in:

- `contracts/medical_records/tests/logging_tests.rs`

## Event Filtering and Querying

### Event Filter

```rust
pub struct EventFilter {
    pub event_types: Option<Vec<EventType>>,
    pub categories: Option<Vec<OperationCategory>>,
    pub user_id: Option<Address>,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub limit: Option<u32>,
}
```

### Query Interface

```rust
pub fn query_events(env: Env, caller: Address, filter: EventFilter) -> EventQueryResult;
```

## Event Aggregation and Monitoring

### Event Statistics

```rust
pub struct EventStats {
    pub total_events: u64,
    pub events_by_type: Map<EventType, u64>,
    pub events_by_category: Map<OperationCategory, u64>,
    pub events_by_user: Map<Address, u64>,
    pub time_range: (u64, u64),
}
```

### Monitoring Dashboard

```rust
pub struct MonitoringDashboard {
    pub stats: EventStats,
    pub recent_events: Vec<BaseEvent>,
    pub alerts: Vec<String>,
    pub health_status: String,
}
```

## Gas Efficiency Considerations

The event system is designed for gas efficiency:

1. **Selective Emission**: Events are emitted only for significant operations
2. **Compact Data**: Event data is structured to minimize storage
3. **Batch Processing**: Support for batch operations to reduce per-event overhead
4. **Indexed Topics**: Events use indexed topics for efficient querying

## Integration with External Systems

### Event Consumption

External systems can consume events through:

1. **Blockchain Event Logs**: Standard Soroban event logs
2. **Off-chain Indexers**: Third-party indexing services
3. **Real-time Monitoring**: WebSocket connections to nodes

### Event Topics

Events are published with consistent topic structures:

- Topic 0: `"EVENT"`
- Topic 1: Event-specific symbol (e.g., `symbol_short!("RECORD_CREATED")`)

### Example Event Consumption

```javascript
// Listen for record creation events
contract.events.RecordCreated({}, (error, event) => {
    console.log('New record created:', event.returnValues);
});
```

## Security and Privacy

### Access Control

- Events include user identification for audit trails
- Sensitive data is not exposed in event logs
- Access to event querying is restricted by user roles

### Data Privacy

- Patient data is not directly exposed in events
- Record references use IDs, not actual medical data
- Compliance with healthcare privacy regulations

## Monitoring and Alerting

### Built-in Alerts

The system includes automatic alert generation for:

- High contract pause frequency
- Unusual user activity patterns
- System offline detection
- Performance metric anomalies

### Custom Alert Configuration

Operators can configure custom alerts based on:

- Event frequency thresholds
- Specific event type patterns
- User behavior anomalies
- System performance metrics

## Event Replay Functionality

### Replay Interface

```rust
pub fn replay_events(
    env: &Env,
    start_time: u64,
    end_time: u64,
    event_types: Option<Vec<EventType>>
) -> Vec<BaseEvent>;
```

### Use Cases

1. **Debugging**: Replay events to understand system behavior
2. **Audit**: Reconstruct historical state for compliance
3. **Recovery**: Replay operations for system restoration
4. **Analysis**: Historical data analysis for insights

## Testing

### Event Verification Tests

The system includes comprehensive tests for:

- Event emission verification
- Event data structure validation
- Filtering and querying functionality
- Gas efficiency validation
- Integration with contract operations

### Example Test

```rust
#[test]
fn test_record_creation_event() {
    let env = Env::default();
    let (client, admin) = create_contract(&env);

    // Create a record
    let record_id = client.add_record(/* ... */);

    // Verify event was emitted
    let events = env.events().all();
    assert!(events.len() > 0);

    // Check event structure
    let event = &events[0];
    assert_eq!(event.topics[0], symbol_short!("EVENT"));
    assert_eq!(event.topics[1], symbol_short!("RECORD_CREATED"));
}
```

## Performance Metrics

### Event Emission Costs

- Average gas cost per event: ~2,000-5,000 instructions
- Batch operations reduce per-event overhead
- Optimized data structures minimize storage costs

### Query Performance

- Event filtering: O(n) complexity with n = total events
- Aggregation: O(n) with efficient Map operations
- Recommended event retention limits for performance

## Configuration

### Event Storage Limits

```rust
const MAX_EVENTS: u32 = 10_000; // Configurable limit
```

### Retention Policies

- Automatic cleanup of old events
- Configurable retention periods
- Archive to off-chain storage for long-term retention

## Future Enhancements

### Planned Features

1. **Event Compression**: Reduce storage costs for large event volumes
2. **Real-time Streaming**: WebSocket-based event streaming
3. **Advanced Analytics**: ML-based anomaly detection on event patterns
4. **Cross-chain Event Relay**: Event propagation across chains
5. **Event-driven Automation**: Trigger external actions based on events

### Extensibility

The event system is designed to be extensible:

- New event types can be added without breaking changes
- Custom event data structures support
- Plugin architecture for specialized event handlers

## Conclusion

The Uzima Medical Records event system provides comprehensive monitoring, auditing, and integration capabilities essential for healthcare applications. Its structured approach ensures consistency, efficiency, and compliance with healthcare requirements while maintaining gas efficiency and data privacy.
