use soroban_sdk::{contracttype, symbol_short, Address, Env, String};

// ==================== Event Payload Structs ====================
// Each struct is a compact, typed payload published to the Soroban event log.
// External indexers subscribe via topic pattern ("NOTIF", symbol_short!("…")).

#[derive(Clone)]
#[contracttype]
pub struct NotifCreatedEvent {
    pub notif_id: u64,
    pub recipient: Address,
    pub sender: Address,
    /// NotificationType repr value.
    pub notif_type: u32,
    /// AlertPriority repr value.
    pub priority: u32,
    pub reference_id: Option<u64>,
    pub timestamp: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct NotifStatusEvent {
    pub notif_id: u64,
    pub user: Address,
    pub timestamp: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct AlertRuleEvent {
    pub rule_id: u64,
    /// NotificationType repr value this rule monitors.
    pub watches_type: u32,
    /// AlertPriority repr value.
    pub priority: u32,
    pub is_active: bool,
    pub admin: Address,
    pub timestamp: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct AlertTriggeredEvent {
    pub rule_id: u64,
    pub sender: Address,
    pub recipient_count: u32,
    pub reference_id: Option<u64>,
    pub timestamp: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct PrefsUpdatedEvent {
    pub user: Address,
    pub enabled: bool,
    /// AlertPriority repr value.
    pub min_priority: u32,
    pub timestamp: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct SenderAuthEvent {
    pub sender: Address,
    pub admin: Address,
    /// true = authorized, false = revoked.
    pub authorized: bool,
    pub timestamp: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct TemplateSetEvent {
    /// NotificationType repr value.
    pub notif_type: u32,
    pub locale: String,
    pub admin: Address,
    pub timestamp: u64,
}

// ==================== Emit Functions ====================

pub fn emit_notification_created(
    env: &Env,
    notif_id: u64,
    recipient: Address,
    sender: Address,
    notif_type: u32,
    priority: u32,
    reference_id: Option<u64>,
) {
    env.events().publish(
        ("NOTIF", symbol_short!("NOTIF_NEW")),
        NotifCreatedEvent {
            notif_id,
            recipient,
            sender,
            notif_type,
            priority,
            reference_id,
            timestamp: env.ledger().timestamp(),
        },
    );
}

pub fn emit_notification_read(env: &Env, notif_id: u64, user: Address) {
    env.events().publish(
        ("NOTIF", symbol_short!("NOTIF_RD")),
        NotifStatusEvent {
            notif_id,
            user,
            timestamp: env.ledger().timestamp(),
        },
    );
}

pub fn emit_notification_archived(env: &Env, notif_id: u64, user: Address) {
    env.events().publish(
        ("NOTIF", symbol_short!("NOTIF_ARC")),
        NotifStatusEvent {
            notif_id,
            user,
            timestamp: env.ledger().timestamp(),
        },
    );
}

pub fn emit_alert_rule_created(
    env: &Env,
    rule_id: u64,
    watches_type: u32,
    priority: u32,
    admin: Address,
) {
    env.events().publish(
        ("NOTIF", symbol_short!("ALRT_NEW")),
        AlertRuleEvent {
            rule_id,
            watches_type,
            priority,
            is_active: true,
            admin,
            timestamp: env.ledger().timestamp(),
        },
    );
}

pub fn emit_alert_rule_updated(
    env: &Env,
    rule_id: u64,
    watches_type: u32,
    priority: u32,
    is_active: bool,
    admin: Address,
) {
    env.events().publish(
        ("NOTIF", symbol_short!("ALRT_UPD")),
        AlertRuleEvent {
            rule_id,
            watches_type,
            priority,
            is_active,
            admin,
            timestamp: env.ledger().timestamp(),
        },
    );
}

pub fn emit_alert_rule_deleted(env: &Env, rule_id: u64, admin: Address) {
    env.events().publish(
        ("NOTIF", symbol_short!("ALRT_DEL")),
        AlertRuleEvent {
            rule_id,
            watches_type: 0,
            priority: 0,
            is_active: false,
            admin,
            timestamp: env.ledger().timestamp(),
        },
    );
}

pub fn emit_alert_triggered(
    env: &Env,
    rule_id: u64,
    sender: Address,
    recipient_count: u32,
    reference_id: Option<u64>,
) {
    env.events().publish(
        ("NOTIF", symbol_short!("ALRT_TRG")),
        AlertTriggeredEvent {
            rule_id,
            sender,
            recipient_count,
            reference_id,
            timestamp: env.ledger().timestamp(),
        },
    );
}

pub fn emit_preferences_updated(env: &Env, user: Address, enabled: bool, min_priority: u32) {
    env.events().publish(
        ("NOTIF", symbol_short!("PREF_UPD")),
        PrefsUpdatedEvent {
            user,
            enabled,
            min_priority,
            timestamp: env.ledger().timestamp(),
        },
    );
}

pub fn emit_sender_authorized(env: &Env, sender: Address, admin: Address) {
    env.events().publish(
        ("NOTIF", symbol_short!("SNDR_ADD")),
        SenderAuthEvent {
            sender,
            admin,
            authorized: true,
            timestamp: env.ledger().timestamp(),
        },
    );
}

pub fn emit_sender_revoked(env: &Env, sender: Address, admin: Address) {
    env.events().publish(
        ("NOTIF", symbol_short!("SNDR_RMV")),
        SenderAuthEvent {
            sender,
            admin,
            authorized: false,
            timestamp: env.ledger().timestamp(),
        },
    );
}

pub fn emit_template_set(env: &Env, notif_type: u32, locale: String, admin: Address) {
    env.events().publish(
        ("NOTIF", symbol_short!("TMPL_SET")),
        TemplateSetEvent {
            notif_type,
            locale,
            admin,
            timestamp: env.ledger().timestamp(),
        },
    );
}
