use soroban_sdk::{contracttype, Address, Map, String, Vec};

// ==================== Channel & Priority ====================

/// Preferred delivery channel for notifications.
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum NotificationChannel {
    /// Soroban on-chain event log — always available.
    OnChain,
    /// Off-chain push; URL reference stored off-chain (e.g. IPFS).
    External,
}

/// Severity level. Higher value = higher urgency.
/// Critical bypasses all user filter preferences.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
#[repr(u32)]
pub enum AlertPriority {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

// ==================== Notification Type ====================

/// Maps one-to-one with the existing medical record event types.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
#[repr(u32)]
pub enum NotificationType {
    RecordCreated = 0,
    RecordUpdated = 1,
    RecordDeleted = 2,
    AccessRequested = 3,
    AccessGranted = 4,
    AccessDenied = 5,
    AccessRevoked = 6,
    EmergencyAccessGranted = 7,
    EmergencyAccessRevoked = 8,
    EmergencyAccessExpired = 9,
    AnomalyDetected = 10,
    RiskScoreAlert = 11,
    SystemAlert = 12,
    Custom = 13,
}

// ==================== Notification Status ====================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum NotificationStatus {
    /// Created, not yet viewed.
    Pending,
    /// Delivered to external channel (still unread on-chain).
    Delivered,
    /// User has acknowledged the notification.
    Read,
    /// Dismissed / no longer surfaced in default queries.
    Archived,
}

// ==================== Core Records ====================

/// Per-user preference settings stored on-chain.
#[derive(Clone)]
#[contracttype]
pub struct NotificationPreferences {
    /// When false, only Critical notifications are created (never skipped).
    pub enabled: bool,
    /// Minimum priority to deliver (Critical always bypasses this gate).
    pub min_priority: AlertPriority,
    /// Preferred delivery channel.
    pub channel: NotificationChannel,
    /// Opted-in `NotificationType` repr values. Empty = all types allowed.
    pub enabled_types: Vec<u32>,
    pub updated_at: u64,
}

/// A single immutable notification record.
#[derive(Clone)]
#[contracttype]
pub struct Notification {
    pub id: u64,
    pub recipient: Address,
    pub sender: Address,
    pub notif_type: NotificationType,
    pub priority: AlertPriority,
    pub status: NotificationStatus,
    /// Short summary, max 100 bytes.
    pub title: String,
    /// Full message body, max 500 bytes.
    pub message: String,
    /// Optional linked entity ID (record_id, proposal_id, …).
    pub reference_id: Option<u64>,
    pub created_at: u64,
    pub read_at: Option<u64>,
    /// Ledger timestamp after which the notification is considered stale.
    pub expires_at: Option<u64>,
}

// ==================== Alert Rules ====================

/// Admin-defined rule that triggers batch notifications on matching events.
#[derive(Clone)]
#[contracttype]
pub struct AlertRule {
    pub id: u64,
    /// Human-readable label, max 50 bytes.
    pub name: String,
    /// `NotificationType` repr value this rule monitors.
    pub watches_type: u32,
    pub priority: AlertPriority,
    /// Explicit recipient list. Empty = rule has no pre-defined recipients.
    pub recipients: Vec<Address>,
    pub is_active: bool,
    pub created_by: Address,
    pub created_at: u64,
}

// ==================== Templates ====================

/// Localised message template keyed by `(notif_type, locale)`.
#[derive(Clone)]
#[contracttype]
pub struct NotificationTemplate {
    /// `NotificationType` repr value.
    pub notif_type: u32,
    /// BCP-47 locale tag, e.g. "en", "fr", "pt-BR" (max 10 bytes).
    pub locale: String,
    /// Title pattern, max 100 bytes.
    pub title: String,
    /// Body pattern, max 500 bytes.
    pub message: String,
    pub default_priority: AlertPriority,
    pub updated_at: u64,
}

// ==================== Query / Response ====================

/// Filter applied when querying a user's notification list.
///
/// Enum fields use `u32` repr values (matching the `#[repr(u32)]` variants)
/// rather than the enum types directly, because `Option<ContractTypeEnum>`
/// cannot be XDR-serialized by Soroban. Use `u32::MAX` as the sentinel
/// "no filter" value; the contract ignores any sentinel value.
#[derive(Clone)]
#[contracttype]
pub struct NotificationFilter {
    /// `NotificationStatus` repr value, or `u32::MAX` to skip this filter.
    pub status: u32,
    /// `NotificationType` repr value, or `u32::MAX` to skip.
    pub notif_type: u32,
    /// Minimum `AlertPriority` repr value, or `u32::MAX` to skip.
    pub min_priority: u32,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    /// Page size — capped at MAX_PAGE_SIZE internally.
    pub limit: u32,
    /// Number of matching records to skip (cursor-style pagination).
    pub offset: u32,
}

/// Paginated result set returned by `get_notifications`.
#[derive(Clone)]
#[contracttype]
pub struct NotificationPage {
    pub notifications: Vec<Notification>,
    /// Total matching records before pagination.
    pub total: u32,
    pub offset: u32,
    pub has_more: bool,
}

/// Aggregated counters for the admin analytics view.
#[derive(Clone)]
#[contracttype]
pub struct NotificationAnalytics {
    pub total_sent: u64,
    pub total_read: u64,
    pub total_pending: u64,
    /// `NotificationType` repr → count.
    pub by_type: Map<u32, u64>,
    /// `AlertPriority` repr → count.
    pub by_priority: Map<u32, u64>,
}

// ==================== Rate Limiting ====================

/// Per-sender rolling-window counter for spam prevention.
#[derive(Clone)]
#[contracttype]
pub struct SenderRateLimit {
    pub count: u32,
    pub window_start: u64,
}
