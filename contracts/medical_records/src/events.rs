use soroban_sdk::{contracttype, Address, BytesN, Env, String, Symbol, Vec, Map, symbol_short};

// ==================== Event Schema Definitions ====================

#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum EventType {
    // User Management Events
    UserCreated,
    UserRoleUpdated,
    UserDeactivated,
    UserActivated,

    // Record Events
    RecordCreated,
    RecordAccessed,
    RecordUpdated,
    RecordDeleted,

    // Access Control Events
    AccessRequested,
    AccessGranted,
    AccessDenied,
    AccessRevoked,

    // Emergency Access Events
    EmergencyAccessGranted,
    EmergencyAccessRevoked,
    EmergencyAccessExpired,

    // Admin Events
    ContractPaused,
    ContractUnpaused,
    RecoveryProposed,
    RecoveryApproved,
    RecoveryExecuted,
    RecoveryRejected,

    // AI Events
    AIConfigUpdated,
    AnomalyScoreSubmitted,
    RiskScoreSubmitted,
    AIAnalysisTriggered,

    // Cross-chain Events
    CrossChainSyncInitiated,
    CrossChainSyncCompleted,
    CrossChainRecordReferenced,

    // System Events
    HealthCheck,
    MetricUpdate,
}

#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum OperationCategory {
    UserManagement,
    RecordOperations,
    AccessControl,
    EmergencyAccess,
    Administrative,
    AIIntegration,
    CrossChain,
    System,
}

#[derive(Clone)]
#[contracttype]
pub struct EventMetadata {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub session_id: Option<BytesN<32>>,
    pub ipfs_ref: Option<String>,
    pub gas_used: Option<u64>,
    pub block_height: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct BaseEvent {
    pub metadata: EventMetadata,
    pub data: EventData,
}

#[derive(Clone)]
#[contracttype]
pub enum EventData {
    // User Management
    UserEvent {
        target_user: Address,
        role: Option<String>,
        previous_role: Option<String>,
        did_reference: Option<String>,
    },

    // Record Events
    RecordEvent {
        record_id: u64,
        patient_id: Address,
        doctor_id: Option<Address>,
        is_confidential: bool,
        category: String,
        tags: Vec<String>,
    },

    // Access Events
    AccessEvent {
        record_id: u64,
        requester: Address,
        patient: Address,
        purpose: String,
        granted: bool,
        credential_hash: Option<BytesN<32>>,
    },

    // Emergency Access
    EmergencyEvent {
        grantee: Address,
        patient: Address,
        record_scope: Vec<u64>,
        expires_at: u64,
        is_active: bool,
    },

    // Recovery Events
    RecoveryEvent {
        proposal_id: u64,
        token_contract: Address,
        recipient: Address,
        amount: i128,
        executed: bool,
        approver_count: u32,
    },

    // AI Events
    AIEvent {
        record_id: Option<u64>,
        patient_id: Option<Address>,
        model_id: BytesN<32>,
        score_bps: u32,
        model_version: String,
        analysis_type: String,
    },

    // Cross-chain Events
    CrossChainEvent {
        local_record_id: u64,
        external_chain: String,
        external_record_hash: BytesN<32>,
        sync_status: String,
    },

    // System Events
    SystemEvent {
        status: String,
        metric_name: Option<String>,
        metric_value: Option<u64>,
    },
}

// ==================== Event Publishing Functions ====================

pub fn emit_user_created(env: &Env, admin: Address, new_user: Address, role: &str, did_ref: Option<String>) {
    let event = BaseEvent {
        metadata: EventMetadata {
            event_type: EventType::UserCreated,
            category: OperationCategory::UserManagement,
            timestamp: env.ledger().timestamp(),
            user_id: admin,
            session_id: None,
            ipfs_ref: None,
            gas_used: None,
            block_height: env.ledger().sequence(),
        },
        data: EventData::UserEvent {
            target_user: new_user,
            role: Some(String::from_str(env, role)),
            previous_role: None,
            did_reference: did_ref,
        },
    };
    env.events().publish(("EVENT", symbol_short!("USER_CREATED")), event);
}

pub fn emit_user_role_updated(env: &Env, admin: Address, target_user: Address, new_role: &str, previous_role: Option<&str>) {
    let event = BaseEvent {
        metadata: EventMetadata {
            event_type: EventType::UserRoleUpdated,
            category: OperationCategory::UserManagement,
            timestamp: env.ledger().timestamp(),
            user_id: admin,
            session_id: None,
            ipfs_ref: None,
            gas_used: None,
            block_height: env.ledger().sequence(),
        },
        data: EventData::UserEvent {
            target_user,
            role: Some(String::from_str(env, new_role)),
            previous_role: previous_role.map(|r| String::from_str(env, r)),
            did_reference: None,
        },
    };
    env.events().publish(("EVENT", symbol_short!("USER_ROLE_UPD")), event);
}

pub fn emit_user_deactivated(env: &Env, admin: Address, target_user: Address) {
    let event = BaseEvent {
        metadata: EventMetadata {
            event_type: EventType::UserDeactivated,
            category: OperationCategory::UserManagement,
            timestamp: env.ledger().timestamp(),
            user_id: admin,
            session_id: None,
            ipfs_ref: None,
            gas_used: None,
            block_height: env.ledger().sequence(),
        },
        data: EventData::UserEvent {
            target_user,
            role: None,
            previous_role: None,
            did_reference: None,
        },
    };
    env.events().publish(("EVENT", symbol_short!("USER_DEACT")), event);
}

pub fn emit_record_created(
    env: &Env,
    doctor: Address,
    record_id: u64,
    patient: Address,
    is_confidential: bool,
    category: String,
    tags: Vec<String>
) {
    let event = BaseEvent {
        metadata: EventMetadata {
            event_type: EventType::RecordCreated,
            category: OperationCategory::RecordOperations,
            timestamp: env.ledger().timestamp(),
            user_id: doctor,
            session_id: None,
            ipfs_ref: None,
            gas_used: None,
            block_height: env.ledger().sequence(),
        },
        data: EventData::RecordEvent {
            record_id,
            patient_id: patient,
            doctor_id: Some(doctor),
            is_confidential,
            category,
            tags,
        },
    };
    env.events().publish(("EVENT", symbol_short!("RECORD_CREATED")), event);
}

pub fn emit_record_accessed(env: &Env, accessor: Address, record_id: u64, patient: Address) {
    let event = BaseEvent {
        metadata: EventMetadata {
            event_type: EventType::RecordAccessed,
            category: OperationCategory::RecordOperations,
            timestamp: env.ledger().timestamp(),
            user_id: accessor,
            session_id: None,
            ipfs_ref: None,
            gas_used: None,
            block_height: env.ledger().sequence(),
        },
        data: EventData::RecordEvent {
            record_id,
            patient_id: patient,
            doctor_id: None,
            is_confidential: false,
            category: String::from_str(env, ""),
            tags: Vec::new(env),
        },
    };
    env.events().publish(("EVENT", symbol_short!("RECORD_ACCESS")), event);
}

pub fn emit_access_requested(
    env: &Env,
    requester: Address,
    patient: Address,
    record_id: u64,
    purpose: String,
    credential_hash: Option<BytesN<32>>
) {
    let event = BaseEvent {
        metadata: EventMetadata {
            event_type: EventType::AccessRequested,
            category: OperationCategory::AccessControl,
            timestamp: env.ledger().timestamp(),
            user_id: requester,
            session_id: None,
            ipfs_ref: None,
            gas_used: None,
            block_height: env.ledger().sequence(),
        },
        data: EventData::AccessEvent {
            record_id,
            requester,
            patient,
            purpose,
            granted: false,
            credential_hash,
        },
    };
    env.events().publish(("EVENT", symbol_short!("ACCESS_REQ")), event);
}

pub fn emit_access_granted(
    env: &Env,
    granter: Address,
    requester: Address,
    patient: Address,
    record_id: u64,
    purpose: String,
    credential_hash: Option<BytesN<32>>
) {
    let event = BaseEvent {
        metadata: EventMetadata {
            event_type: EventType::AccessGranted,
            category: OperationCategory::AccessControl,
            timestamp: env.ledger().timestamp(),
            user_id: granter,
            session_id: None,
            ipfs_ref: None,
            gas_used: None,
            block_height: env.ledger().sequence(),
        },
        data: EventData::AccessEvent {
            record_id,
            requester,
            patient,
            purpose,
            granted: true,
            credential_hash,
        },
    };
    env.events().publish(("EVENT", symbol_short!("ACCESS_GRANT")), event);
}

pub fn emit_emergency_access_granted(
    env: &Env,
    granter: Address,
    grantee: Address,
    patient: Address,
    record_scope: Vec<u64>,
    expires_at: u64
) {
    let event = BaseEvent {
        metadata: EventMetadata {
            event_type: EventType::EmergencyAccessGranted,
            category: OperationCategory::EmergencyAccess,
            timestamp: env.ledger().timestamp(),
            user_id: granter,
            session_id: None,
            ipfs_ref: None,
            gas_used: None,
            block_height: env.ledger().sequence(),
        },
        data: EventData::EmergencyEvent {
            grantee,
            patient,
            record_scope,
            expires_at,
            is_active: true,
        },
    };
    env.events().publish(("EVENT", symbol_short!("EMERGENCY_GRANT")), event);
}

pub fn emit_contract_paused(env: &Env, admin: Address) {
    let event = BaseEvent {
        metadata: EventMetadata {
            event_type: EventType::ContractPaused,
            category: OperationCategory::Administrative,
            timestamp: env.ledger().timestamp(),
            user_id: admin,
            session_id: None,
            ipfs_ref: None,
            gas_used: None,
            block_height: env.ledger().sequence(),
        },
        data: EventData::SystemEvent {
            status: String::from_str(env, "paused"),
            metric_name: None,
            metric_value: None,
        },
    };
    env.events().publish(("EVENT", symbol_short!("CONTRACT_PAUSE")), event);
}

pub fn emit_contract_unpaused(env: &Env, admin: Address) {
    let event = BaseEvent {
        metadata: EventMetadata {
            event_type: EventType::ContractUnpaused,
            category: OperationCategory::Administrative,
            timestamp: env.ledger().timestamp(),
            user_id: admin,
            session_id: None,
            ipfs_ref: None,
            gas_used: None,
            block_height: env.ledger().sequence(),
        },
        data: EventData::SystemEvent {
            status: String::from_str(env, "active"),
            metric_name: None,
            metric_value: None,
        },
    };
    env.events().publish(("EVENT", symbol_short!("CONTRACT_UNPAUSE")), event);
}

pub fn emit_recovery_proposed(
    env: &Env,
    proposer: Address,
    proposal_id: u64,
    token_contract: Address,
    recipient: Address,
    amount: i128
) {
    let event = BaseEvent {
        metadata: EventMetadata {
            event_type: EventType::RecoveryProposed,
            category: OperationCategory::Administrative,
            timestamp: env.ledger().timestamp(),
            user_id: proposer,
            session_id: None,
            ipfs_ref: None,
            gas_used: None,
            block_height: env.ledger().sequence(),
        },
        data: EventData::RecoveryEvent {
            proposal_id,
            token_contract,
            recipient,
            amount,
            executed: false,
            approver_count: 1,
        },
    };
    env.events().publish(("EVENT", symbol_short!("RECOVERY_PROP")), event);
}

pub fn emit_recovery_approved(env: &Env, approver: Address, proposal_id: u64) {
    let event = BaseEvent {
        metadata: EventMetadata {
            event_type: EventType::RecoveryApproved,
            category: OperationCategory::Administrative,
            timestamp: env.ledger().timestamp(),
            user_id: approver,
            session_id: None,
            ipfs_ref: None,
            gas_used: None,
            block_height: env.ledger().sequence(),
        },
        data: EventData::RecoveryEvent {
            proposal_id,
            token_contract: Address::generate(env), // Placeholder
            recipient: Address::generate(env), // Placeholder
            amount: 0,
            executed: false,
            approver_count: 0, // Will be updated by caller
        },
    };
    env.events().publish(("EVENT", symbol_short!("RECOVERY_APPR")), event);
}

pub fn emit_recovery_executed(
    env: &Env,
    executor: Address,
    proposal_id: u64,
    token_contract: Address,
    recipient: Address,
    amount: i128
) {
    let event = BaseEvent {
        metadata: EventMetadata {
            event_type: EventType::RecoveryExecuted,
            category: OperationCategory::Administrative,
            timestamp: env.ledger().timestamp(),
            user_id: executor,
            session_id: None,
            ipfs_ref: None,
            gas_used: None,
            block_height: env.ledger().sequence(),
        },
        data: EventData::RecoveryEvent {
            proposal_id,
            token_contract,
            recipient,
            amount,
            executed: true,
            approver_count: 0, // Not relevant for execution
        },
    };
    env.events().publish(("EVENT", symbol_short!("RECOVERY_EXEC")), event);
}

pub fn emit_ai_config_updated(env: &Env, admin: Address, ai_coordinator: Address) {
    let event = BaseEvent {
        metadata: EventMetadata {
            event_type: EventType::AIConfigUpdated,
            category: OperationCategory::AIIntegration,
            timestamp: env.ledger().timestamp(),
            user_id: admin,
            session_id: None,
            ipfs_ref: None,
            gas_used: None,
            block_height: env.ledger().sequence(),
        },
        data: EventData::AIEvent {
            record_id: None,
            patient_id: None,
            model_id: BytesN::from_array(env, &[0u8; 32]), // Placeholder
            score_bps: 0,
            model_version: String::from_str(env, ""),
            analysis_type: String::from_str(env, "config_update"),
        },
    };
    env.events().publish(("EVENT", symbol_short!("AI_CONFIG_UPD")), event);
}

pub fn emit_anomaly_score_submitted(
    env: &Env,
    ai_coordinator: Address,
    record_id: u64,
    patient: Address,
    model_id: BytesN<32>,
    score_bps: u32,
    model_version: String
) {
    let event = BaseEvent {
        metadata: EventMetadata {
            event_type: EventType::AnomalyScoreSubmitted,
            category: OperationCategory::AIIntegration,
            timestamp: env.ledger().timestamp(),
            user_id: ai_coordinator,
            session_id: None,
            ipfs_ref: None,
            gas_used: None,
            block_height: env.ledger().sequence(),
        },
        data: EventData::AIEvent {
            record_id: Some(record_id),
            patient_id: Some(patient),
            model_id,
            score_bps,
            model_version,
            analysis_type: String::from_str(env, "anomaly_detection"),
        },
    };
    env.events().publish(("EVENT", symbol_short!("ANOMALY_SCORE")), event);
}

pub fn emit_risk_score_submitted(
    env: &Env,
    ai_coordinator: Address,
    patient: Address,
    model_id: BytesN<32>,
    score_bps: u32,
    model_version: String
) {
    let event = BaseEvent {
        metadata: EventMetadata {
            event_type: EventType::RiskScoreSubmitted,
            category: OperationCategory::AIIntegration,
            timestamp: env.ledger().timestamp(),
            user_id: ai_coordinator,
            session_id: None,
            ipfs_ref: None,
            gas_used: None,
            block_height: env.ledger().sequence(),
        },
        data: EventData::AIEvent {
            record_id: None,
            patient_id: Some(patient),
            model_id,
            score_bps,
            model_version,
            analysis_type: String::from_str(env, "risk_assessment"),
        },
    };
    env.events().publish(("EVENT", symbol_short!("RISK_SCORE")), event);
}

pub fn emit_ai_analysis_triggered(env: &Env, record_id: u64, patient: Address) {
    let event = BaseEvent {
        metadata: EventMetadata {
            event_type: EventType::AIAnalysisTriggered,
            category: OperationCategory::AIIntegration,
            timestamp: env.ledger().timestamp(),
            user_id: Address::generate(env), // System triggered
            session_id: None,
            ipfs_ref: None,
            gas_used: None,
            block_height: env.ledger().sequence(),
        },
        data: EventData::AIEvent {
            record_id: Some(record_id),
            patient_id: Some(patient),
            model_id: BytesN::from_array(env, &[0u8; 32]), // Placeholder
            score_bps: 0,
            model_version: String::from_str(env, ""),
            analysis_type: String::from_str(env, "analysis_triggered"),
        },
    };
    env.events().publish(("EVENT", symbol_short!("AI_TRIGGER")), event);
}

pub fn emit_health_check(env: &Env, status: &str, gas_used: u64) {
    let event = BaseEvent {
        metadata: EventMetadata {
            event_type: EventType::HealthCheck,
            category: OperationCategory::System,
            timestamp: env.ledger().timestamp(),
            user_id: Address::generate(env), // System
            session_id: None,
            ipfs_ref: None,
            gas_used: Some(gas_used),
            block_height: env.ledger().sequence(),
        },
        data: EventData::SystemEvent {
            status: String::from_str(env, status),
            metric_name: Some(String::from_str(env, "health_check")),
            metric_value: Some(1),
        },
    };
    env.events().publish(("EVENT", symbol_short!("HEALTH_CHECK")), event);
}

pub fn emit_metric_update(env: &Env, metric_name: &str, value: u64) {
    let event = BaseEvent {
        metadata: EventMetadata {
            event_type: EventType::MetricUpdate,
            category: OperationCategory::System,
            timestamp: env.ledger().timestamp(),
            user_id: Address::generate(env), // System
            session_id: None,
            ipfs_ref: None,
            gas_used: None,
            block_height: env.ledger().sequence(),
        },
        data: EventData::SystemEvent {
            status: String::from_str(env, "active"),
            metric_name: Some(String::from_str(env, metric_name)),
            metric_value: Some(value),
        },
    };
    env.events().publish(("EVENT", symbol_short!("METRIC_UPD")), event);
}

// ==================== Event Filtering and Querying ====================

#[derive(Clone)]
#[contracttype]
pub struct EventFilter {
    pub event_types: Option<Vec<EventType>>,
    pub categories: Option<Vec<OperationCategory>>,
    pub user_id: Option<Address>,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub limit: Option<u32>,
}

#[derive(Clone)]
#[contracttype]
pub struct EventQueryResult {
    pub events: Vec<BaseEvent>,
    pub total_count: u64,
    pub has_more: bool,
}

// ==================== Event Aggregation ====================

#[derive(Clone)]
#[contracttype]
pub struct EventStats {
    pub total_events: u64,
    pub events_by_type: Map<EventType, u64>,
    pub events_by_category: Map<OperationCategory, u64>,
    pub events_by_user: Map<Address, u64>,
    pub time_range: (u64, u64), // (start, end)
}

#[derive(Clone)]
#[contracttype]
pub struct MonitoringDashboard {
    pub stats: EventStats,
    pub recent_events: Vec<BaseEvent>,
    pub alerts: Vec<String>,
    pub health_status: String,
}

// ==================== Event Querying and Aggregation Implementation ====================

use soroban_sdk::{Env, Vec};

pub fn filter_events(events: &Vec<BaseEvent>, filter: &EventFilter) -> Vec<BaseEvent> {
    let mut filtered = Vec::new(&events.env());

    for event in events.iter() {
        let metadata = &event.metadata;

        // Filter by event types
        if let Some(ref types) = filter.event_types {
            let mut found = false;
            for event_type in types.iter() {
                if metadata.event_type == *event_type {
                    found = true;
                    break;
                }
            }
            if !found { continue; }
        }

        // Filter by categories
        if let Some(ref categories) = filter.categories {
            let mut found = false;
            for category in categories.iter() {
                if metadata.category == *category {
                    found = true;
                    break;
                }
            }
            if !found { continue; }
        }

        // Filter by user
        if let Some(ref user_filter) = filter.user_id {
            if metadata.user_id != *user_filter { continue; }
        }

        // Filter by time range
        if let Some(start_time) = filter.start_time {
            if metadata.timestamp < start_time { continue; }
        }
        if let Some(end_time) = filter.end_time {
            if metadata.timestamp > end_time { continue; }
        }

        filtered.push_back(event.clone());
    }

    // Apply limit
    if let Some(limit) = filter.limit {
        let mut limited = Vec::new(&events.env());
        let len = filtered.len().min(limit as usize);
        for i in 0..len {
            if let Some(event) = filtered.get(i as u32) {
                limited.push_back(event);
            }
        }
        limited
    } else {
        filtered
    }
}

pub fn aggregate_events(events: &Vec<BaseEvent>) -> EventStats {
    let env = &events.env();
    let mut events_by_type: Map<EventType, u64> = Map::new(env);
    let mut events_by_category: Map<OperationCategory, u64> = Map::new(env);
    let mut events_by_user: Map<Address, u64> = Map::new(env);

    let mut min_time = u64::MAX;
    let mut max_time = 0u64;

    for event in events.iter() {
        let metadata = &event.metadata;

        // Track time range
        if metadata.timestamp < min_time { min_time = metadata.timestamp; }
        if metadata.timestamp > max_time { max_time = metadata.timestamp; }

        // Count by type
        let curr_type = metadata.event_type.clone();
        let type_count = events_by_type.get(curr_type.clone()).unwrap_or(0) + 1;
        events_by_type.set(curr_type.clone(), type_count);

        // Count by category
        let curr_cat = metadata.category.clone();
        let category_count = events_by_category.get(curr_cat.clone()).unwrap_or(0) + 1;
        events_by_category.set(curr_cat.clone(), category_count);

        // Count by user
        let user = metadata.user_id.clone();
        let user_count = events_by_user.get(user.clone()).unwrap_or(0) + 1;
        events_by_user.set(user.clone(), user_count);
    }

    EventStats {
        total_events: events.len() as u64,
        events_by_type,
        events_by_category,
        events_by_user,
        time_range: (min_time, max_time),
    }
}

pub fn create_monitoring_dashboard(
    env: &Env,
    all_events: &Vec<BaseEvent>,
    recent_limit: u32
) -> MonitoringDashboard {
    let stats = aggregate_events(all_events);

    // Get recent events (last N events)
    let mut recent_events = Vec::new(env);
    let start = if all_events.len() > recent_limit {
        all_events.len() - recent_limit as usize
    } else {
        0
    };

    for i in start..all_events.len() {
        if let Some(event) = all_events.get(i as u32) {
            recent_events.push_back(event);
        }
    }

    // Generate alerts based on patterns
    let alerts = generate_alerts(&stats, env);

    // Determine health status
    let health_status = determine_health_status(&stats, &alerts);

    MonitoringDashboard {
        stats,
        recent_events,
        alerts,
        health_status: String::from_str(env, &health_status),
    }
}

fn generate_alerts(stats: &EventStats, env: &Env) -> Vec<String> {
    let mut alerts = Vec::new(env);

    // Alert on high error rates (simplified example)
    if let Some(error_count) = stats.events_by_type.get(EventType::ContractPaused) {
        if error_count > 5 {
            alerts.push_back(String::from_str(env, "High number of contract pauses detected"));
        }
    }

    // Alert on unusual user activity
    for (user, count) in stats.events_by_user.iter() {
        if count > 100 {  // Arbitrary threshold
            alerts.push_back(String::from_str(env, "High activity detected for user"));
        }
    }

    // Alert on system issues
    if stats.total_events == 0 {
        alerts.push_back(String::from_str(env, "No events recorded - system may be offline"));
    }

    alerts
}

fn determine_health_status(stats: &EventStats, alerts: &Vec<String>) -> String {
    if alerts.len() > 0 {
        "warning".to_string()
    } else if stats.total_events > 0 {
        "healthy".to_string()
    } else {
        "unknown".to_string()
    }
}

// ==================== Event Storage and Retrieval ====================

// Note: In a real implementation, events would be stored in contract storage
// For now, this is a simplified version that could be extended

#[derive(Clone)]
#[contracttype]
pub struct EventStore {
    pub events: Vec<BaseEvent>,
    pub max_events: u32,
}

impl EventStore {
    pub fn new(env: &Env, max_events: u32) -> Self {
        Self {
            events: Vec::new(env),
            max_events,
        }
    }

    pub fn add_event(&mut self, event: BaseEvent) {
        self.events.push_back(event);

        // Maintain max size by removing oldest events
        while self.events.len() > self.max_events {
            self.events.pop_front();
        }
    }

    pub fn query_events(&self, filter: &EventFilter) -> EventQueryResult {
        let filtered = filter_events(&self.events, filter);
        let has_more = filtered.len() == filter.limit.unwrap_or(u32::MAX) as usize;

        EventQueryResult {
            events: filtered,
            total_count: self.events.len() as u64,
            has_more,
        }
    }

    pub fn get_dashboard(&self, env: &Env, recent_limit: u32) -> MonitoringDashboard {
        create_monitoring_dashboard(env, &self.events, recent_limit)
    }

    pub fn replay_events(&self, start_time: u64, end_time: u64, event_types: Option<Vec<EventType>>) -> Vec<BaseEvent> {
        let mut replayed = Vec::new(&self.events.env());

        for event in self.events.iter() {
            // Filter by time range
            if event.metadata.timestamp < start_time || event.metadata.timestamp > end_time {
                continue;
            }

            // Filter by event types if specified
            if let Some(ref types) = event_types {
                let mut found = false;
                for event_type in types.iter() {
                    if matches!(&event.metadata.event_type, event_type) {
                        found = true;
                        break;
                    }
                }
                if !found {
                    continue;
                }
            }

            replayed.push_back(event.clone());
        }

        replayed
    }
}
