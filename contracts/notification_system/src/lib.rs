#![no_std]
#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]

#[cfg(test)]
mod test;

mod errors;
mod events;
mod types;

pub use errors::Error;
pub use types::{
    AlertPriority, AlertRule, Notification, NotificationAnalytics, NotificationChannel,
    NotificationFilter, NotificationPage, NotificationPreferences, NotificationStatus,
    NotificationTemplate, NotificationType,
};

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Map, String, Vec};

// ==================== Storage Keys ====================

#[contracttype]
pub enum DataKey {
    // Singleton / lifecycle — stored in instance storage
    Initialized,
    Admin,

    // Sender authorization — stored in instance storage
    AuthorizedSenders, // Vec<Address>, bounded by MAX_SENDERS

    // Per-sender rate limiting — persistent
    SenderRate(Address), // SenderRateLimit

    // User preferences — persistent
    UserPrefs(Address), // NotificationPreferences

    // Notification records — persistent
    NotifCount,             // u64 — monotonic ID counter
    Notif(u64),             // Notification
    UserNotifIds(Address),  // Vec<u64> — ordered by insertion (oldest first)
    UserUnreadCount(Address), // u32

    // Alert rules — persistent
    AlertRuleCount,       // u64 — monotonic ID counter
    AlertRule(u64),       // AlertRule
    ActiveAlertRuleIds,   // Vec<u64> — IDs of all non-deleted rules

    // Localised templates — persistent
    // Key: (notif_type_repr, locale) → NotificationTemplate
    Template(u32, String),

    // Analytics counters — persistent
    TotalSent,         // u64
    TotalRead,         // u64
    TotalPending,      // u64
    ByTypeSent(u32),   // u64 — keyed by NotificationType repr
    ByPrioritySent(u32), // u64 — keyed by AlertPriority repr
}

// ==================== Constants ====================

/// Maximum distinct authorized senders (contracts + admin).
const MAX_SENDERS: u32 = 50;
/// Maximum notifications stored per user (ring-buffer eviction after this).
const MAX_USER_NOTIFS: u32 = 200;
/// Maximum active alert rules.
const MAX_ALERT_RULES: u32 = 100;
/// Maximum recipients in a single alert rule.
const MAX_RULE_RECIPIENTS: u32 = 20;
/// Maximum page size for paginated notification queries.
const MAX_PAGE_SIZE: u32 = 50;
/// Maximum recipients for bulk notification (create_bulk_notifications).
const MAX_BULK_RECIPIENTS: u32 = 20;
/// Maximum number of localised templates stored per notification type.
const MAX_TEMPLATES_PER_TYPE: u32 = 10;
/// Maximum enabled-type entries in NotificationPreferences.
const MAX_ENABLED_TYPES: u32 = 14;

// String byte-length ceilings
const MAX_TITLE_LEN: u32 = 100;
const MAX_MESSAGE_LEN: u32 = 500;
const MAX_RULE_NAME_LEN: u32 = 50;
const MAX_LOCALE_LEN: u32 = 10;

// Sender rate-limit: MAX_SENDER_CALLS notifications per RATE_WINDOW_SECS.
const MAX_SENDER_CALLS: u32 = 100;
const RATE_WINDOW_SECS: u64 = 3_600; // 1 hour

// ==================== Contract ====================

#[contract]
pub struct NotificationContract;

#[contractimpl]
impl NotificationContract {
    // ------------------------------------------------------------------
    // Lifecycle
    // ------------------------------------------------------------------

    /// Initialise the contract. Must be called exactly once.
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::AuthorizedSenders, &Vec::<Address>::new(&env));
        Ok(())
    }

    /// Returns the current admin address.
    pub fn get_admin(env: Env) -> Result<Address, Error> {
        Self::require_initialized(&env)?;
        Ok(Self::read_admin(&env))
    }

    // ------------------------------------------------------------------
    // Sender Authorization
    // ------------------------------------------------------------------

    /// Authorise `sender` to create notifications on behalf of integrated contracts.
    pub fn add_authorized_sender(
        env: Env,
        caller: Address,
        sender: Address,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        let mut senders = Self::read_authorized_senders(&env);
        if senders.contains(sender.clone()) {
            return Ok(()); // Idempotent
        }
        if senders.len() >= MAX_SENDERS {
            return Err(Error::MaxSendersReached);
        }
        senders.push_back(sender.clone());
        env.storage()
            .instance()
            .set(&DataKey::AuthorizedSenders, &senders);

        events::emit_sender_authorized(&env, sender, caller);
        Ok(())
    }

    /// Revoke a sender's authorisation.
    pub fn remove_authorized_sender(
        env: Env,
        caller: Address,
        sender: Address,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        let senders = Self::read_authorized_senders(&env);
        let mut updated = Vec::new(&env);
        let mut found = false;
        for s in senders.iter() {
            if s == sender {
                found = true;
            } else {
                updated.push_back(s);
            }
        }
        if !found {
            return Err(Error::SenderNotFound);
        }
        env.storage()
            .instance()
            .set(&DataKey::AuthorizedSenders, &updated);

        events::emit_sender_revoked(&env, sender, caller);
        Ok(())
    }

    /// Returns the list of all currently authorised sender addresses.
    pub fn get_authorized_senders(env: Env) -> Result<Vec<Address>, Error> {
        Self::require_initialized(&env)?;
        Ok(Self::read_authorized_senders(&env))
    }

    // ------------------------------------------------------------------
    // User Preferences
    // ------------------------------------------------------------------

    /// Upsert `user`'s notification preferences. The user must sign the call.
    pub fn set_preferences(
        env: Env,
        user: Address,
        prefs: NotificationPreferences,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        user.require_auth();

        if prefs.enabled_types.len() > MAX_ENABLED_TYPES {
            return Err(Error::TooManyEnabledTypes);
        }

        let stored = NotificationPreferences {
            enabled: prefs.enabled,
            min_priority: prefs.min_priority,
            channel: prefs.channel,
            enabled_types: prefs.enabled_types,
            updated_at: env.ledger().timestamp(),
        };
        env.storage()
            .persistent()
            .set(&DataKey::UserPrefs(user.clone()), &stored);

        events::emit_preferences_updated(
            &env,
            user,
            stored.enabled,
            stored.min_priority as u32,
        );
        Ok(())
    }

    /// Returns the preferences for `user`, or `None` if not configured.
    pub fn get_preferences(
        env: Env,
        user: Address,
    ) -> Result<Option<NotificationPreferences>, Error> {
        Self::require_initialized(&env)?;
        Ok(env
            .storage()
            .persistent()
            .get(&DataKey::UserPrefs(user)))
    }

    // ------------------------------------------------------------------
    // Notification Creation
    // ------------------------------------------------------------------

    /// Create a single notification for `recipient`.
    /// Caller must be the admin or an authorised sender.
    /// Returns the assigned notification ID.
    pub fn create_notification(
        env: Env,
        sender: Address,
        recipient: Address,
        notif_type: NotificationType,
        priority: AlertPriority,
        title: String,
        message: String,
        reference_id: Option<u64>,
        expires_at: Option<u64>,
    ) -> Result<u64, Error> {
        Self::require_initialized(&env)?;
        sender.require_auth();
        Self::require_authorized(&env, &sender)?;
        Self::check_and_update_sender_rate(&env, &sender)?;

        Self::validate_title(&title)?;
        Self::validate_message(&message)?;

        let notif_id = Self::next_notif_id(&env);
        let status = Self::resolve_status(&env, &recipient, notif_type, priority);

        let notif = Notification {
            id: notif_id,
            recipient: recipient.clone(),
            sender: sender.clone(),
            notif_type,
            priority,
            status,
            title,
            message,
            reference_id,
            created_at: env.ledger().timestamp(),
            read_at: None,
            expires_at,
        };

        Self::store_notification(&env, notif.clone());
        Self::increment_analytics(&env, notif_type, priority, status);

        events::emit_notification_created(
            &env,
            notif_id,
            recipient,
            sender,
            notif_type as u32,
            priority as u32,
            reference_id,
        );
        Ok(notif_id)
    }

    /// Create one notification per recipient in `recipients`.
    /// Bounded by MAX_BULK_RECIPIENTS to cap gas cost.
    pub fn create_bulk_notifications(
        env: Env,
        sender: Address,
        recipients: Vec<Address>,
        notif_type: NotificationType,
        priority: AlertPriority,
        title: String,
        message: String,
        reference_id: Option<u64>,
        expires_at: Option<u64>,
    ) -> Result<Vec<u64>, Error> {
        Self::require_initialized(&env)?;
        sender.require_auth();
        Self::require_authorized(&env, &sender)?;
        Self::check_and_update_sender_rate(&env, &sender)?;

        if recipients.is_empty() {
            return Err(Error::RecipientsEmpty);
        }
        if recipients.len() > MAX_BULK_RECIPIENTS {
            return Err(Error::BatchTooLarge);
        }

        Self::validate_title(&title)?;
        Self::validate_message(&message)?;

        let mut ids = Vec::new(&env);
        let timestamp = env.ledger().timestamp();

        for recipient in recipients.iter() {
            let notif_id = Self::next_notif_id(&env);
            let status = Self::resolve_status(&env, &recipient, notif_type, priority);
            let notif = Notification {
                id: notif_id,
                recipient: recipient.clone(),
                sender: sender.clone(),
                notif_type,
                priority,
                status,
                title: title.clone(),
                message: message.clone(),
                reference_id,
                created_at: timestamp,
                read_at: None,
                expires_at,
            };
            Self::store_notification(&env, notif.clone());
            Self::increment_analytics(&env, notif_type, priority, status);
            events::emit_notification_created(
                &env,
                notif_id,
                recipient,
                sender.clone(),
                notif_type as u32,
                priority as u32,
                reference_id,
            );
            ids.push_back(notif_id);
        }
        Ok(ids)
    }

    // ------------------------------------------------------------------
    // Notification Retrieval
    // ------------------------------------------------------------------

    /// Fetch a single notification by ID.
    /// Only the recipient or admin may view it.
    pub fn get_notification(
        env: Env,
        caller: Address,
        notif_id: u64,
    ) -> Result<Notification, Error> {
        Self::require_initialized(&env)?;
        caller.require_auth();

        let notif = Self::load_notification(&env, notif_id)?;
        if notif.recipient != caller && !Self::is_admin(&env, &caller) {
            return Err(Error::NotAuthorized);
        }
        Ok(notif)
    }

    /// Paginated query over a user's notification history.
    /// Caller must be the user or admin.
    /// Results are returned newest-first; `filter.offset` skips matching records.
    pub fn get_notifications(
        env: Env,
        caller: Address,
        user: Address,
        filter: NotificationFilter,
    ) -> Result<NotificationPage, Error> {
        Self::require_initialized(&env)?;
        caller.require_auth();
        if caller != user && !Self::is_admin(&env, &caller) {
            return Err(Error::NotAuthorized);
        }

        let limit = filter.limit.min(MAX_PAGE_SIZE);
        let ids = Self::read_user_notif_ids(&env, &user);
        let total_ids = ids.len(); // u32

        let mut matched: Vec<Notification> = Vec::new(&env);
        let mut total_matched: u32 = 0;
        let mut skipped: u32 = 0;

        // Iterate newest-first (push_back → last element is newest).
        let mut idx = total_ids;
        loop {
            if idx == 0 {
                break;
            }
            idx -= 1;

            let notif_id = match ids.get(idx) {
                Some(id) => id,
                None => break,
            };
            let notif = match env
                .storage()
                .persistent()
                .get::<DataKey, Notification>(&DataKey::Notif(notif_id))
            {
                Some(n) => n,
                None => continue, // Evicted or missing — skip
            };

            // Apply filter predicates (u32::MAX = sentinel "no filter").
            if filter.status != u32::MAX && (notif.status as u32) != filter.status {
                continue;
            }
            if filter.notif_type != u32::MAX && (notif.notif_type as u32) != filter.notif_type {
                continue;
            }
            if filter.min_priority != u32::MAX
                && (notif.priority as u32) < filter.min_priority
            {
                continue;
            }
            if let Some(start) = filter.start_time {
                if notif.created_at < start {
                    continue;
                }
            }
            if let Some(end) = filter.end_time {
                if notif.created_at > end {
                    continue;
                }
            }

            total_matched += 1;

            if skipped < filter.offset {
                skipped += 1;
                continue;
            }
            if (matched.len() as u32) < limit {
                matched.push_back(notif);
            }
        }

        let has_more = total_matched > filter.offset.saturating_add(matched.len() as u32);
        Ok(NotificationPage {
            notifications: matched,
            total: total_matched,
            offset: filter.offset,
            has_more,
        })
    }

    /// Returns the number of unread (Pending + Delivered) notifications for a user.
    pub fn get_unread_count(env: Env, user: Address) -> Result<u32, Error> {
        Self::require_initialized(&env)?;
        Ok(Self::read_unread_count(&env, &user))
    }

    // ------------------------------------------------------------------
    // Notification State Transitions
    // ------------------------------------------------------------------

    /// Mark a single notification as Read.
    /// Only the recipient may call this.
    pub fn mark_read(env: Env, caller: Address, notif_id: u64) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        caller.require_auth();

        let mut notif = Self::load_notification(&env, notif_id)?;
        if notif.recipient != caller {
            return Err(Error::NotAuthorized);
        }
        if notif.status == NotificationStatus::Read {
            return Err(Error::AlreadyRead);
        }
        if notif.status == NotificationStatus::Archived {
            return Err(Error::AlreadyArchived);
        }

        let was_pending = matches!(
            notif.status,
            NotificationStatus::Pending | NotificationStatus::Delivered
        );
        notif.status = NotificationStatus::Read;
        notif.read_at = Some(env.ledger().timestamp());

        env.storage()
            .persistent()
            .set(&DataKey::Notif(notif_id), &notif);

        if was_pending {
            Self::decrement_unread(&env, &caller);
            Self::increment_total_read(&env);
            Self::decrement_total_pending(&env);
        }

        events::emit_notification_read(&env, notif_id, caller);
        Ok(())
    }

    /// Mark all Pending / Delivered notifications for the caller as Read.
    /// Returns the count of newly-read notifications.
    pub fn mark_all_read(env: Env, caller: Address) -> Result<u32, Error> {
        Self::require_initialized(&env)?;
        caller.require_auth();

        let ids = Self::read_user_notif_ids(&env, &caller);
        let mut newly_read: u32 = 0;
        let timestamp = env.ledger().timestamp();

        for notif_id in ids.iter() {
            let notif_opt = env
                .storage()
                .persistent()
                .get::<DataKey, Notification>(&DataKey::Notif(notif_id));
            if let Some(mut notif) = notif_opt {
                if matches!(
                    notif.status,
                    NotificationStatus::Pending | NotificationStatus::Delivered
                ) {
                    notif.status = NotificationStatus::Read;
                    notif.read_at = Some(timestamp);
                    env.storage()
                        .persistent()
                        .set(&DataKey::Notif(notif_id), &notif);
                    newly_read += 1;
                }
            }
        }

        if newly_read > 0 {
            // Reset unread counter to 0 (all reads processed).
            env.storage()
                .persistent()
                .set(&DataKey::UserUnreadCount(caller.clone()), &0u32);
            Self::add_to_total_read(&env, newly_read as u64);
            Self::sub_from_total_pending(&env, newly_read as u64);
        }

        Ok(newly_read)
    }

    /// Archive a notification so it no longer appears in default queries.
    /// Caller must be the recipient or admin.
    pub fn archive_notification(
        env: Env,
        caller: Address,
        notif_id: u64,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        caller.require_auth();

        let mut notif = Self::load_notification(&env, notif_id)?;
        if notif.recipient != caller && !Self::is_admin(&env, &caller) {
            return Err(Error::NotAuthorized);
        }
        if notif.status == NotificationStatus::Archived {
            return Err(Error::AlreadyArchived);
        }

        let was_pending = matches!(
            notif.status,
            NotificationStatus::Pending | NotificationStatus::Delivered
        );
        notif.status = NotificationStatus::Archived;
        env.storage()
            .persistent()
            .set(&DataKey::Notif(notif_id), &notif);

        if was_pending {
            Self::decrement_unread(&env, &notif.recipient);
            Self::decrement_total_pending(&env);
        }

        events::emit_notification_archived(&env, notif_id, caller);
        Ok(())
    }

    // ------------------------------------------------------------------
    // Alert Rules
    // ------------------------------------------------------------------

    /// Create a new alert rule. Only admin may call this.
    pub fn create_alert_rule(
        env: Env,
        caller: Address,
        name: String,
        watches_type: u32,
        priority: AlertPriority,
        recipients: Vec<Address>,
    ) -> Result<u64, Error> {
        Self::require_initialized(&env)?;
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        if name.len() > MAX_RULE_NAME_LEN {
            return Err(Error::NameTooLong);
        }
        if recipients.len() > MAX_RULE_RECIPIENTS {
            return Err(Error::BatchTooLarge);
        }

        let mut rule_ids = Self::read_active_rule_ids(&env);
        if rule_ids.len() >= MAX_ALERT_RULES {
            return Err(Error::MaxRulesReached);
        }

        let rule_id = Self::next_rule_id(&env);
        let rule = AlertRule {
            id: rule_id,
            name,
            watches_type,
            priority,
            recipients,
            is_active: true,
            created_by: caller.clone(),
            created_at: env.ledger().timestamp(),
        };
        env.storage()
            .persistent()
            .set(&DataKey::AlertRule(rule_id), &rule);

        rule_ids.push_back(rule_id);
        env.storage()
            .persistent()
            .set(&DataKey::ActiveAlertRuleIds, &rule_ids);

        events::emit_alert_rule_created(&env, rule_id, watches_type, priority as u32, caller);
        Ok(rule_id)
    }

    /// Update the active state, priority, and recipients of an existing rule.
    pub fn update_alert_rule(
        env: Env,
        caller: Address,
        rule_id: u64,
        is_active: bool,
        priority: AlertPriority,
        recipients: Vec<Address>,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        if recipients.len() > MAX_RULE_RECIPIENTS {
            return Err(Error::BatchTooLarge);
        }

        let mut rule = Self::load_rule(&env, rule_id)?;
        rule.is_active = is_active;
        rule.priority = priority;
        rule.recipients = recipients;
        let watches_type = rule.watches_type;
        env.storage()
            .persistent()
            .set(&DataKey::AlertRule(rule_id), &rule);

        events::emit_alert_rule_updated(
            &env,
            rule_id,
            watches_type,
            priority as u32,
            is_active,
            caller,
        );
        Ok(())
    }

    /// Permanently delete an alert rule.
    pub fn delete_alert_rule(env: Env, caller: Address, rule_id: u64) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        // Verify it exists before deleting.
        Self::load_rule(&env, rule_id)?;

        env.storage()
            .persistent()
            .remove(&DataKey::AlertRule(rule_id));

        // Remove from the active IDs index.
        let ids = Self::read_active_rule_ids(&env);
        let mut updated = Vec::new(&env);
        for id in ids.iter() {
            if id != rule_id {
                updated.push_back(id);
            }
        }
        env.storage()
            .persistent()
            .set(&DataKey::ActiveAlertRuleIds, &updated);

        events::emit_alert_rule_deleted(&env, rule_id, caller);
        Ok(())
    }

    /// Returns all non-deleted alert rules. Admin only.
    pub fn get_alert_rules(env: Env, caller: Address) -> Result<Vec<AlertRule>, Error> {
        Self::require_initialized(&env)?;
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        let ids = Self::read_active_rule_ids(&env);
        let mut rules = Vec::new(&env);
        for id in ids.iter() {
            if let Some(rule) = env
                .storage()
                .persistent()
                .get::<DataKey, AlertRule>(&DataKey::AlertRule(id))
            {
                rules.push_back(rule);
            }
        }
        Ok(rules)
    }

    /// Trigger a specific alert rule: creates notifications for each of its recipients.
    /// Caller must be admin or an authorised sender.
    /// Returns the IDs of all created notifications.
    pub fn trigger_alert(
        env: Env,
        sender: Address,
        rule_id: u64,
        reference_id: Option<u64>,
        custom_message: Option<String>,
    ) -> Result<Vec<u64>, Error> {
        Self::require_initialized(&env)?;
        sender.require_auth();
        Self::require_authorized(&env, &sender)?;
        Self::check_and_update_sender_rate(&env, &sender)?;

        let rule = Self::load_rule(&env, rule_id)?;
        if !rule.is_active {
            // Rule is disabled — emit event but create no notifications.
            events::emit_alert_triggered(&env, rule_id, sender, 0, reference_id);
            return Ok(Vec::new(&env));
        }
        if rule.recipients.is_empty() {
            // No pre-defined recipients — emit event for external indexers only.
            events::emit_alert_triggered(&env, rule_id, sender, 0, reference_id);
            return Ok(Vec::new(&env));
        }

        // Use custom_message if provided, else rule name as fallback.
        let message = custom_message.unwrap_or_else(|| rule.name.clone());
        Self::validate_message(&message)?;

        let mut ids = Vec::new(&env);
        let timestamp = env.ledger().timestamp();
        let recipient_count = rule.recipients.len();

        for recipient in rule.recipients.iter() {
            let notif_id = Self::next_notif_id(&env);
            let status = Self::resolve_status(&env, &recipient, NotificationType::Custom, rule.priority);
            let notif = Notification {
                id: notif_id,
                recipient: recipient.clone(),
                sender: sender.clone(),
                notif_type: NotificationType::Custom,
                priority: rule.priority,
                status,
                title: rule.name.clone(),
                message: message.clone(),
                reference_id,
                created_at: timestamp,
                read_at: None,
                expires_at: None,
            };
            Self::store_notification(&env, notif.clone());
            Self::increment_analytics(&env, NotificationType::Custom, rule.priority, status);
            events::emit_notification_created(
                &env,
                notif_id,
                recipient,
                sender.clone(),
                NotificationType::Custom as u32,
                rule.priority as u32,
                reference_id,
            );
            ids.push_back(notif_id);
        }

        events::emit_alert_triggered(&env, rule_id, sender, recipient_count, reference_id);
        Ok(ids)
    }

    // ------------------------------------------------------------------
    // Templates
    // ------------------------------------------------------------------

    /// Upsert a localised notification template. Admin only.
    pub fn set_template(
        env: Env,
        caller: Address,
        template: NotificationTemplate,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        if template.locale.len() > MAX_LOCALE_LEN {
            return Err(Error::LocaleTooLong);
        }
        if template.title.len() > MAX_TITLE_LEN {
            return Err(Error::TitleTooLong);
        }
        if template.message.len() > MAX_MESSAGE_LEN {
            return Err(Error::MessageTooLong);
        }

        let stored = NotificationTemplate {
            notif_type: template.notif_type,
            locale: template.locale.clone(),
            title: template.title,
            message: template.message,
            default_priority: template.default_priority,
            updated_at: env.ledger().timestamp(),
        };

        env.storage().persistent().set(
            &DataKey::Template(template.notif_type, template.locale.clone()),
            &stored,
        );

        events::emit_template_set(&env, template.notif_type, template.locale, caller);
        Ok(())
    }

    /// Retrieve a template by notification type and locale.
    pub fn get_template(
        env: Env,
        notif_type: u32,
        locale: String,
    ) -> Result<NotificationTemplate, Error> {
        Self::require_initialized(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::Template(notif_type, locale))
            .ok_or(Error::TemplateNotFound)
    }

    // ------------------------------------------------------------------
    // Analytics
    // ------------------------------------------------------------------

    /// Returns aggregated send/read/pending counters. Admin only.
    pub fn get_analytics(env: Env, caller: Address) -> Result<NotificationAnalytics, Error> {
        Self::require_initialized(&env)?;
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        let total_sent: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalSent)
            .unwrap_or(0);
        let total_read: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalRead)
            .unwrap_or(0);
        let total_pending: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalPending)
            .unwrap_or(0);

        // Rebuild per-type and per-priority maps from counters.
        let mut by_type: Map<u32, u64> = Map::new(&env);
        let type_reprs: [u32; 14] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];
        for t in type_reprs.iter() {
            let count: u64 = env
                .storage()
                .persistent()
                .get(&DataKey::ByTypeSent(*t))
                .unwrap_or(0);
            if count > 0 {
                by_type.set(*t, count);
            }
        }

        let mut by_priority: Map<u32, u64> = Map::new(&env);
        let prio_reprs: [u32; 4] = [0, 1, 2, 3];
        for p in prio_reprs.iter() {
            let count: u64 = env
                .storage()
                .persistent()
                .get(&DataKey::ByPrioritySent(*p))
                .unwrap_or(0);
            if count > 0 {
                by_priority.set(*p, count);
            }
        }

        Ok(NotificationAnalytics {
            total_sent,
            total_read,
            total_pending,
            by_type,
            by_priority,
        })
    }

    // ------------------------------------------------------------------
    // Private helpers
    // ------------------------------------------------------------------

    fn require_initialized(env: &Env) -> Result<(), Error> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::NotInitialized);
        }
        Ok(())
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        if !Self::is_admin(env, caller) {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    /// Caller must be admin OR in the authorised-senders list.
    fn require_authorized(env: &Env, caller: &Address) -> Result<(), Error> {
        if Self::is_admin(env, caller) {
            return Ok(());
        }
        if Self::is_authorized_sender(env, caller) {
            return Ok(());
        }
        Err(Error::SenderNotAuthorized)
    }

    fn is_admin(env: &Env, addr: &Address) -> bool {
        match env
            .storage()
            .instance()
            .get::<DataKey, Address>(&DataKey::Admin)
        {
            Some(admin) => admin == *addr,
            None => false,
        }
    }

    fn is_authorized_sender(env: &Env, addr: &Address) -> bool {
        Self::read_authorized_senders(env).contains(addr.clone())
    }

    fn read_admin(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap()
    }

    fn read_authorized_senders(env: &Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&DataKey::AuthorizedSenders)
            .unwrap_or_else(|| Vec::new(env))
    }

    // ------ Notification ID counter ------

    fn next_notif_id(env: &Env) -> u64 {
        let id: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::NotifCount)
            .unwrap_or(0u64)
            .saturating_add(1);
        env.storage().persistent().set(&DataKey::NotifCount, &id);
        id
    }

    // ------ Alert rule ID counter ------

    fn next_rule_id(env: &Env) -> u64 {
        let id: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::AlertRuleCount)
            .unwrap_or(0u64)
            .saturating_add(1);
        env.storage()
            .persistent()
            .set(&DataKey::AlertRuleCount, &id);
        id
    }

    // ------ Notification storage (ring-buffer eviction) ------

    fn store_notification(env: &Env, notif: Notification) {
        let recipient = notif.recipient.clone();
        let notif_id = notif.id;

        // Store the notification record.
        env.storage()
            .persistent()
            .set(&DataKey::Notif(notif_id), &notif);

        // Append ID to user's list; evict oldest if capacity exceeded.
        let mut ids = Self::read_user_notif_ids(env, &recipient);
        if ids.len() >= MAX_USER_NOTIFS {
            // Remove the oldest notification to maintain the ring buffer.
            if let Some(oldest_id) = ids.get(0) {
                env.storage()
                    .persistent()
                    .remove(&DataKey::Notif(oldest_id));
                // Rebuild ids without the first element.
                let mut trimmed = Vec::new(env);
                for i in 1..ids.len() {
                    if let Some(id) = ids.get(i) {
                        trimmed.push_back(id);
                    }
                }
                ids = trimmed;
            }
        }
        ids.push_back(notif_id);
        env.storage()
            .persistent()
            .set(&DataKey::UserNotifIds(recipient.clone()), &ids);

        // Maintain unread counter for Pending / Delivered statuses.
        if matches!(
            notif.status,
            NotificationStatus::Pending | NotificationStatus::Delivered
        ) {
            let current = Self::read_unread_count(env, &recipient);
            env.storage()
                .persistent()
                .set(&DataKey::UserUnreadCount(recipient), &current.saturating_add(1));
        }
    }

    fn read_user_notif_ids(env: &Env, user: &Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::UserNotifIds(user.clone()))
            .unwrap_or_else(|| Vec::new(env))
    }

    fn read_unread_count(env: &Env, user: &Address) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::UserUnreadCount(user.clone()))
            .unwrap_or(0)
    }

    fn decrement_unread(env: &Env, user: &Address) {
        let current = Self::read_unread_count(env, user);
        env.storage()
            .persistent()
            .set(&DataKey::UserUnreadCount(user.clone()), &current.saturating_sub(1));
    }

    fn load_notification(env: &Env, notif_id: u64) -> Result<Notification, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Notif(notif_id))
            .ok_or(Error::NotificationNotFound)
    }

    fn load_rule(env: &Env, rule_id: u64) -> Result<AlertRule, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::AlertRule(rule_id))
            .ok_or(Error::AlertRuleNotFound)
    }

    fn read_active_rule_ids(env: &Env) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::ActiveAlertRuleIds)
            .unwrap_or_else(|| Vec::new(env))
    }

    // ------ Delivery resolution ------

    /// Determines the initial `NotificationStatus` for a new notification
    /// based on the recipient's stored preferences.
    ///
    /// - Critical priority → always `Pending` (bypasses all filters).
    /// - No preferences stored → `Pending` (opt-in by default).
    /// - Filtered by preferences → `Archived` (kept for history but hidden).
    fn resolve_status(
        env: &Env,
        recipient: &Address,
        notif_type: NotificationType,
        priority: AlertPriority,
    ) -> NotificationStatus {
        if priority == AlertPriority::Critical {
            return NotificationStatus::Pending;
        }
        let prefs_opt: Option<NotificationPreferences> = env
            .storage()
            .persistent()
            .get(&DataKey::UserPrefs(recipient.clone()));

        match prefs_opt {
            None => NotificationStatus::Pending,
            Some(prefs) => {
                if !prefs.enabled {
                    return NotificationStatus::Archived;
                }
                // Priority threshold gate.
                if (priority as u32) < (prefs.min_priority as u32) {
                    return NotificationStatus::Archived;
                }
                // Type allow-list gate (empty = all types allowed).
                if !prefs.enabled_types.is_empty() {
                    let type_id = notif_type as u32;
                    let mut allowed = false;
                    for t in prefs.enabled_types.iter() {
                        if t == type_id {
                            allowed = true;
                            break;
                        }
                    }
                    if !allowed {
                        return NotificationStatus::Archived;
                    }
                }
                NotificationStatus::Pending
            }
        }
    }

    // ------ Analytics counters ------

    fn increment_analytics(
        env: &Env,
        notif_type: NotificationType,
        priority: AlertPriority,
        status: NotificationStatus,
    ) {
        let type_key = DataKey::ByTypeSent(notif_type as u32);
        let prio_key = DataKey::ByPrioritySent(priority as u32);

        let type_count: u64 = env
            .storage()
            .persistent()
            .get(&type_key)
            .unwrap_or(0u64)
            .saturating_add(1);
        env.storage().persistent().set(&type_key, &type_count);

        let prio_count: u64 = env
            .storage()
            .persistent()
            .get(&prio_key)
            .unwrap_or(0u64)
            .saturating_add(1);
        env.storage().persistent().set(&prio_key, &prio_count);

        let total: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalSent)
            .unwrap_or(0u64)
            .saturating_add(1);
        env.storage().persistent().set(&DataKey::TotalSent, &total);

        if matches!(
            status,
            NotificationStatus::Pending | NotificationStatus::Delivered
        ) {
            let pending: u64 = env
                .storage()
                .persistent()
                .get(&DataKey::TotalPending)
                .unwrap_or(0u64)
                .saturating_add(1);
            env.storage()
                .persistent()
                .set(&DataKey::TotalPending, &pending);
        }
    }

    fn increment_total_read(env: &Env) {
        let v: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalRead)
            .unwrap_or(0u64)
            .saturating_add(1);
        env.storage().persistent().set(&DataKey::TotalRead, &v);
    }

    fn add_to_total_read(env: &Env, delta: u64) {
        let v: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalRead)
            .unwrap_or(0u64)
            .saturating_add(delta);
        env.storage().persistent().set(&DataKey::TotalRead, &v);
    }

    fn decrement_total_pending(env: &Env) {
        let v: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalPending)
            .unwrap_or(0u64)
            .saturating_sub(1);
        env.storage().persistent().set(&DataKey::TotalPending, &v);
    }

    fn sub_from_total_pending(env: &Env, delta: u64) {
        let v: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalPending)
            .unwrap_or(0u64)
            .saturating_sub(delta);
        env.storage().persistent().set(&DataKey::TotalPending, &v);
    }

    // ------ Rate limiting ------

    /// Enforce the per-sender rolling-window rate limit.
    fn check_and_update_sender_rate(env: &Env, sender: &Address) -> Result<(), Error> {
        let key = DataKey::SenderRate(sender.clone());
        let now = env.ledger().timestamp();

        let entry: types::SenderRateLimit = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(types::SenderRateLimit {
                count: 0,
                window_start: now,
            });

        let (count, window_start) = if now.saturating_sub(entry.window_start) >= RATE_WINDOW_SECS {
            // Window expired — reset counter.
            (0u32, now)
        } else {
            (entry.count, entry.window_start)
        };

        if count >= MAX_SENDER_CALLS {
            return Err(Error::RateLimitExceeded);
        }

        env.storage().persistent().set(
            &key,
            &types::SenderRateLimit {
                count: count.saturating_add(1),
                window_start,
            },
        );
        Ok(())
    }

    // ------ Validation ------

    fn validate_title(title: &String) -> Result<(), Error> {
        if title.len() > MAX_TITLE_LEN {
            return Err(Error::TitleTooLong);
        }
        Ok(())
    }

    fn validate_message(message: &String) -> Result<(), Error> {
        if message.len() > MAX_MESSAGE_LEN {
            return Err(Error::MessageTooLong);
        }
        Ok(())
    }
}
