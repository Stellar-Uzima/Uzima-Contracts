# Contract Logging Standards

This document defines the standardized logging patterns for all smart contracts in the Uzima Healthcare platform.

## Overview

All contracts use Soroban's event emission system for logging. Events serve as the primary mechanism for off-chain logging, monitoring, and log aggregation.

## Log Level Usage Guidelines

Sorbon contracts emit events instead of traditional log levels. Event types map to log levels as follows:

| Log Level | Event Type | Usage |
|-----------|------------|-------|
| Error | SystemEvent with status "error" | Operation failures, validation errors |
| Warning | SystemEvent with status "warning" | Non-critical issues, deprecated usage |
| Info | Operation-specific events | Normal operations (record created, access granted, etc.) |
| Debug | Detailed data events | Complex operations with full data payloads |

### Priority Mapping

- **Critical**: ContractPaused, ContractUnpaused, RecoveryExecuted, EmergencyAccessGranted
- **High**: UserCreated, RecordCreated, RecordDeleted, AccessGranted, AccessDenied
- **Medium**: RecordAccessed, UserRoleUpdated, PermissionGranted, MetadataUpdated
- **Low**: HealthCheck, MetricUpdate, AIAnalysisTriggered

## Message Format Standards

All events must follow the structured format defined in `contracts/medical_records/src/events.rs`:

### Event Structure

```rust
pub struct EventMetadata {
    pub event_type: EventType,      // Categorized event type
    pub category: OperationCategory, // Operation category
    pub timestamp: u64,             // Ledger timestamp
    pub user_id: Address,           // Acting user
    pub session_id: Option<String>,  // Optional session reference
    pub ipfs_ref: Option<String>,   // Optional IPFS reference
    pub gas_used: Option<u64>,       // Gas consumed (for expensive ops)
    pub block_height: u64,           // Ledger sequence
}

pub enum EventData {
    // ... operation-specific data
}
```

### Naming Conventions

- Topic (first parameter): 4-10 character symbol_short (e.g., `USER_ADD`, `REC_NEW`, `ACC_REQ`)
- Event type names: PascalCase (e.g., `UserCreated`, `RecordAccessed`)
- Category names: PascalCase (e.g., `UserManagement`, `RecordOperations`)

## Structured Logging Adoption

### Event Type Categories

All contracts must use events from these categories:

1. **UserManagement**: UserCreated, UserRoleUpdated, UserDeactivated, UserActivated
2. **RecordOperations**: RecordCreated, RecordAccessed, RecordUpdated, RecordDeleted
3. **AccessControl**: AccessRequested, AccessGranted, AccessDenied, AccessRevoked
4. **EmergencyAccess**: EmergencyAccessGranted, EmergencyAccessRevoked, EmergencyAccessExpired
5. **Administrative**: ContractPaused, ContractUnpaused, RecoveryProposed, RecoveryApproved, RecoveryExecuted
6. **AIIntegration**: AIConfigUpdated, AnomalyScoreSubmitted, RiskScoreSubmitted, AIAnalysisTriggered
7. **CrossChain**: CrossChainSyncInitiated, CrossChainSyncCompleted, CrossChainRecordReferenced
8. **System**: HealthCheck, MetricUpdate, DataQualityValidated
9. **DataQuality**: DataQualityValidated

### Required Metadata

All events must include:
- `timestamp`: Current ledger timestamp (`env.ledger().timestamp()`)
- `block_height`: Current ledger sequence (`env.ledger().sequence() as u64`)
- `user_id`: The address triggering the event
- `event_type`: Specific event type from the defined enum
- `category`: Operation category for filtering

## Sensitive Data Handling

### Data Classification

Events are classified into three sensitivity levels:

1. **Public** (emit without restrictions):
   - Event type (e.g., RecordCreated)
   - Timestamp and block height
   - User roles (without PII)

2. **Restricted** (requires authorization):
   - Record IDs
   - Access grants/revocations
   - Permission changes

3. **Confidential** (never emit):
   - Patient medical data
   - Diagnosis information
   - Treatment details
   - PHI/PII beyond roles

### Never Log

The following data must NEVER be included in events:
- Patient names
- Medical diagnoses
- Treatment plans
- Medication details
- Lab results
- SSNs or other identifiers
- Private keys or signatures
- Authentication credentials

## Event Emission Pattern

### Standard Emitter Function

```rust
pub fn emit_<action>(
    env: &Env,
    actor: Address,
    // ... operation-specific parameters
) {
    let event = BaseEvent {
        metadata: EventMetadata {
            event_type: EventType::<Action>,
            category: OperationCategory::<Category>,
            timestamp: env.ledger().timestamp(),
            user_id: actor,
            session_id: None,
            ipfs_ref: None,
            gas_used: None,
            block_height: env.ledger().sequence() as u64,
        },
        data: EventData::<EventDataType> {
            // ... operation-specific data
        },
    };
    env.events()
        .publish(("EVENT", symbol_short!("TOPIC")), event);
}
```

## Log Retention Policies

### Retention Periods by Category

| Category | Retention Period | Rationale |
|----------|-----------------|-----------|
| Administrative | 7 years | Audit compliance, regulatory requirements |
| AccessControl | 7 years | HIPAA compliance for healthcare |
| EmergencyAccess | 7 years | Emergency access audit trail |
| RecordOperations | 5 years | Medical record retention |
| UserManagement | 5 years | User lifecycle tracking |
| System | 1 year | Operational metrics |
| AIIntegration | 3 years | Model performance tracking |
| CrossChain | 3 years | Cross-chain reconciliation |
| DataQuality | 3 years | Quality assurance |

### Implementation

Event retention is managed through:
1. **Event indexing**: Store events with timestamp for time-based queries
2. **Filtering by age**: Use `filter_events` with time range
3. **Archival policy**: Offload old events to long-term storage

### Query Pattern for Retention

```rust
fn get_retainable_events(
    events: &Vec<BaseEvent>,
    max_age_seconds: u64,
    current_time: u64,
) -> Vec<BaseEvent> {
    let cutoff = current_time.saturating_sub(max_age_seconds);
    let filter = EventFilter {
        event_types: None,
        categories: None,
        user_id: None,
        start_time: Some(cutoff),
        end_time: None,
        limit: None,
    };
    filter_events(events, &filter)
}
```

### Off-Chain Retention

Off-chain systems should:
- Export events older than retention period to cold storage
- Maintain searchable index for compliance queries
- Implement data minimization (remove non-essential fields after X years)

## Log Aggregation Configuration

### Event Collection

Events can be aggregated using:
- **Topic filtering**: Filter by event topic (e.g., `"EVENT"`)
- **Type filtering**: Filter by EventType enum
- **Category filtering**: Filter by OperationCategory
- **Time range**: Filter by timestamp
- **User filtering**: Filter by user_id

### Dashboard Integration

The MonitoringDashboard structure supports:
- EventStats: Total events, events by type/category/user
- Recent events: Last N events
- Alerts: Active alerts
- Health status: System health

### Query Pattern

```rust
pub fn filter_events(events: &Vec<BaseEvent>, filter: &EventFilter) -> Vec<BaseEvent>
pub fn aggregate_events(events: &Vec<BaseEvent>) -> EventStats
```

### Aggregation Scripts

Off-chain log aggregation can be configured using the provided scripts:

```bash
# Aggregate events by type
cargo run --bin aggregate-events -- --type user_management

# Generate compliance report
cargo run --bin compliance-report -- --start-date 2026-01-01 --end-date 2026-04-25
```

## Example Contract Implementation

The RBAC contract (`contracts/rbac/`) demonstrates the standardized logging pattern:

```rust
// events.rs - Define event types and emit functions
pub mod events {
    use soroban_sdk::{contracttype, symbol_short, Address, Env, String};
    
    #[derive(Clone, Copy, PartialEq, Eq)]
    #[contracttype]
    pub enum EventType {
        Initialized,
        RoleAssigned,
        RoleRemoved,
    }
    
    #[derive(Clone, Copy, PartialEq, Eq)]
    #[contracttype]
    pub enum OperationCategory {
        UserManagement,
        Administrative,
    }
    
    // Standardized event structure with all required metadata
    pub struct RBACEvent {
        pub event_type: EventType,
        pub category: OperationCategory,
        pub timestamp: u64,
        pub user_id: Address,
        pub block_height: u64,
        pub data: RBACEventData,
    }
    
    // Emit functions follow naming convention: emit_<action>
    pub fn emit_role_assigned(env: &Env, admin: Address, target: Address, role: String, success: bool) {
        let event = RBACEvent {
            event_type: EventType::RoleAssigned,
            category: OperationCategory::UserManagement,
            timestamp: env.ledger().timestamp(),
            user_id: admin,
            block_height: env.ledger().sequence() as u64,
            data: RBACEventData { target_address: target, role: Some(role), success },
        };
        env.events().publish(("EVENT", symbol_short!("ROLE_ADD")), event);
    }
}
```

## Implementation Requirements

### New Contracts

New contracts must:
1. Import or define events from `contracts/medical_records/src/events.rs`
2. Use appropriate EventType and OperationCategory enums
3. Emit events for all state-changing operations
4. Include all required metadata fields

### Existing Contracts

Existing contracts should be updated to:
1. Adopt the standardized BaseEvent structure
2. Include all required metadata
3. Follow naming conventions
4. Remove any sensitive data from events

## Exceptions

Some contracts may have custom event types for domain-specific operations:
- `health_data_access_logging`: Dedicated compliance logging
- `notification_system`: Notification delivery tracking
- `audit`: Comprehensive audit trail

These contracts maintain domain-specific event types while adhering to core standards.

## References

- Event system implementation: `contracts/medical_records/src/events.rs`
- Healthcare compliance: `docs/HEALTHCARE_COMPLIANCE_FRAMEWORK.md`
- Access control patterns: `docs/RBAC.md`

---

Last Updated: 2026-04-25