#![cfg(test)]
#![allow(clippy::unwrap_used)]

use soroban_sdk::{
    testutils::{Address as _, Events},
    Address, Env, String, Vec,
};

use crate::{
    errors::Error, AlertPriority, NotificationChannel, NotificationContract,
    NotificationContractClient, NotificationFilter, NotificationPreferences, NotificationStatus,
    NotificationTemplate, NotificationType,
};

// ==================== Helpers ====================

fn setup(env: &Env) -> (NotificationContractClient<'_>, Address) {
    let contract_id = Address::generate(env);
    env.register_contract(&contract_id, NotificationContract);
    let client = NotificationContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    env.mock_all_auths();
    client.initialize(&admin);
    (client, admin)
}

fn s(env: &Env, text: &str) -> String {
    String::from_str(env, text)
}

fn make_prefs(env: &Env, enabled: bool, min_priority: AlertPriority) -> NotificationPreferences {
    NotificationPreferences {
        enabled,
        min_priority,
        channel: NotificationChannel::OnChain,
        enabled_types: Vec::new(env),
        updated_at: 0,
    }
}

/// Build a filter with u32::MAX sentinels meaning "no filter" for enum fields.
fn all_filter(limit: u32, offset: u32) -> NotificationFilter {
    NotificationFilter {
        status: u32::MAX,
        notif_type: u32::MAX,
        min_priority: u32::MAX,
        start_time: None,
        end_time: None,
        limit,
        offset,
    }
}

fn status_filter(status: NotificationStatus, limit: u32) -> NotificationFilter {
    NotificationFilter {
        status: status as u32,
        notif_type: u32::MAX,
        min_priority: u32::MAX,
        start_time: None,
        end_time: None,
        limit,
        offset: 0,
    }
}

// ==================== Lifecycle ====================

#[test]
fn test_initialize_stores_admin() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    assert_eq!(client.get_admin(), admin);
}

#[test]
fn test_double_initialize_fails() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    env.mock_all_auths();
    assert!(matches!(
        client.try_initialize(&admin),
        Err(Ok(Error::AlreadyInitialized))
    ));
}

#[test]
fn test_get_admin_before_init_fails() {
    let env = Env::default();
    let contract_id = Address::generate(&env);
    env.register_contract(&contract_id, NotificationContract);
    let client = NotificationContractClient::new(&env, &contract_id);
    env.mock_all_auths();
    assert!(client.try_get_admin().is_err());
}

// ==================== Sender Authorization ====================

#[test]
fn test_add_and_list_authorized_senders() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let sender = Address::generate(&env);
    env.mock_all_auths();

    client.add_authorized_sender(&admin, &sender);
    assert!(client.get_authorized_senders().contains(sender));
}

#[test]
fn test_remove_authorized_sender() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let sender = Address::generate(&env);
    env.mock_all_auths();

    client.add_authorized_sender(&admin, &sender);
    client.remove_authorized_sender(&admin, &sender);
    assert!(!client.get_authorized_senders().contains(sender));
}

#[test]
fn test_add_sender_is_idempotent() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let sender = Address::generate(&env);
    env.mock_all_auths();

    client.add_authorized_sender(&admin, &sender);
    client.add_authorized_sender(&admin, &sender);
    assert_eq!(client.get_authorized_senders().len(), 1);
}

#[test]
fn test_remove_unknown_sender_fails() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    env.mock_all_auths();
    assert!(matches!(
        client.try_remove_authorized_sender(&admin, &Address::generate(&env)),
        Err(Ok(Error::SenderNotFound))
    ));
}

#[test]
fn test_non_admin_cannot_add_sender() {
    let env = Env::default();
    let (client, _) = setup(&env);
    env.mock_all_auths();
    assert!(matches!(
        client.try_add_authorized_sender(&Address::generate(&env), &Address::generate(&env)),
        Err(Ok(Error::NotAuthorized))
    ));
}

// ==================== Preferences ====================

#[test]
fn test_set_and_get_preferences() {
    let env = Env::default();
    let (client, _) = setup(&env);
    let user = Address::generate(&env);
    env.mock_all_auths();

    client.set_preferences(&user, &make_prefs(&env, true, AlertPriority::Medium));
    let stored = client.get_preferences(&user).unwrap();
    assert!(stored.enabled);
    assert_eq!(stored.min_priority, AlertPriority::Medium);
}

#[test]
fn test_unset_preferences_returns_none() {
    let env = Env::default();
    let (client, _) = setup(&env);
    env.mock_all_auths();
    assert!(client.get_preferences(&Address::generate(&env)).is_none());
}

#[test]
fn test_set_preferences_emits_event() {
    let env = Env::default();
    let (client, _) = setup(&env);
    let user = Address::generate(&env);
    env.mock_all_auths();

    let before = env.events().all().len();
    client.set_preferences(&user, &make_prefs(&env, true, AlertPriority::Low));
    assert!(env.events().all().len() > before);
}

// ==================== Notification Creation ====================

#[test]
fn test_create_notification_returns_sequential_ids() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let r = Address::generate(&env);
    env.mock_all_auths();

    let id1 = client.create_notification(
        &admin,
        &r,
        &NotificationType::RecordCreated,
        &AlertPriority::Low,
        &s(&env, "T1"),
        &s(&env, "B1"),
        &None,
        &None,
    );
    let id2 = client.create_notification(
        &admin,
        &r,
        &NotificationType::RecordUpdated,
        &AlertPriority::Low,
        &s(&env, "T2"),
        &s(&env, "B2"),
        &None,
        &None,
    );
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
}

#[test]
fn test_create_notification_increments_unread_count() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let r = Address::generate(&env);
    env.mock_all_auths();

    assert_eq!(client.get_unread_count(&r), 0);
    client.create_notification(
        &admin,
        &r,
        &NotificationType::RecordCreated,
        &AlertPriority::Medium,
        &s(&env, "T"),
        &s(&env, "B"),
        &None,
        &None,
    );
    assert_eq!(client.get_unread_count(&r), 1);
}

#[test]
fn test_create_notification_emits_event() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let r = Address::generate(&env);
    env.mock_all_auths();

    let before = env.events().all().len();
    client.create_notification(
        &admin,
        &r,
        &NotificationType::AccessGranted,
        &AlertPriority::High,
        &s(&env, "Access"),
        &s(&env, "Granted."),
        &Some(1u64),
        &None,
    );
    assert!(env.events().all().len() > before);
}

#[test]
fn test_unauthorized_sender_cannot_create() {
    let env = Env::default();
    let (client, _) = setup(&env);
    let rogue = Address::generate(&env);
    let r = Address::generate(&env);
    env.mock_all_auths();

    assert!(matches!(
        client.try_create_notification(
            &rogue,
            &r,
            &NotificationType::SystemAlert,
            &AlertPriority::Low,
            &s(&env, "Hack"),
            &s(&env, "Bad"),
            &None,
            &None,
        ),
        Err(Ok(Error::SenderNotAuthorized))
    ));
}

#[test]
fn test_title_too_long_is_rejected() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let r = Address::generate(&env);
    env.mock_all_auths();

    // 101 bytes — over the 100-byte limit.
    let long = s(&env,
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaab");
    assert!(matches!(
        client.try_create_notification(
            &admin,
            &r,
            &NotificationType::RecordCreated,
            &AlertPriority::Low,
            &long,
            &s(&env, "Body"),
            &None,
            &None,
        ),
        Err(Ok(Error::TitleTooLong))
    ));
}

// ==================== Bulk Notifications ====================

#[test]
fn test_bulk_creates_one_per_recipient() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    env.mock_all_auths();

    let r1 = Address::generate(&env);
    let r2 = Address::generate(&env);
    let r3 = Address::generate(&env);
    let mut recipients = Vec::new(&env);
    recipients.push_back(r1.clone());
    recipients.push_back(r2.clone());
    recipients.push_back(r3.clone());

    let ids = client.create_bulk_notifications(
        &admin,
        &recipients,
        &NotificationType::SystemAlert,
        &AlertPriority::High,
        &s(&env, "Alert"),
        &s(&env, "Maintenance"),
        &None,
        &None,
    );
    assert_eq!(ids.len(), 3);
    assert_eq!(client.get_unread_count(&r1), 1);
    assert_eq!(client.get_unread_count(&r2), 1);
    assert_eq!(client.get_unread_count(&r3), 1);
}

#[test]
fn test_bulk_empty_recipients_fails() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    env.mock_all_auths();
    assert!(matches!(
        client.try_create_bulk_notifications(
            &admin,
            &Vec::new(&env),
            &NotificationType::SystemAlert,
            &AlertPriority::Low,
            &s(&env, "T"),
            &s(&env, "B"),
            &None,
            &None,
        ),
        Err(Ok(Error::RecipientsEmpty))
    ));
}

// ==================== Notification Retrieval ====================

#[test]
fn test_get_notification_by_recipient() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let r = Address::generate(&env);
    env.mock_all_auths();

    let id = client.create_notification(
        &admin,
        &r,
        &NotificationType::RecordCreated,
        &AlertPriority::Low,
        &s(&env, "Title"),
        &s(&env, "Body"),
        &None,
        &None,
    );
    let notif = client.get_notification(&r, &id);
    assert_eq!(notif.id, id);
    assert_eq!(notif.status, NotificationStatus::Pending);
}

#[test]
fn test_get_notification_by_non_recipient_fails() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let r = Address::generate(&env);
    let stranger = Address::generate(&env);
    env.mock_all_auths();

    let id = client.create_notification(
        &admin,
        &r,
        &NotificationType::RecordCreated,
        &AlertPriority::Low,
        &s(&env, "T"),
        &s(&env, "B"),
        &None,
        &None,
    );
    assert!(matches!(
        client.try_get_notification(&stranger, &id),
        Err(Ok(Error::NotAuthorized))
    ));
}

#[test]
fn test_get_notifications_paginated_newest_first() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let user = Address::generate(&env);
    env.mock_all_auths();

    let mut last_id = 0u64;
    for _ in 0..5u32 {
        last_id = client.create_notification(
            &admin,
            &user,
            &NotificationType::RecordCreated,
            &AlertPriority::Low,
            &s(&env, "T"),
            &s(&env, "B"),
            &None,
            &None,
        );
    }

    let page = client.get_notifications(&user, &user, &all_filter(3, 0));
    assert_eq!(page.notifications.len(), 3);
    assert_eq!(page.total, 5);
    assert!(page.has_more);
    // Newest first.
    assert_eq!(page.notifications.get(0).unwrap().id, last_id);
}

#[test]
fn test_get_notifications_filter_by_status() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let user = Address::generate(&env);
    env.mock_all_auths();

    let id1 = client.create_notification(
        &admin,
        &user,
        &NotificationType::RecordCreated,
        &AlertPriority::Low,
        &s(&env, "T1"),
        &s(&env, "B1"),
        &None,
        &None,
    );
    client.create_notification(
        &admin,
        &user,
        &NotificationType::RecordUpdated,
        &AlertPriority::Low,
        &s(&env, "T2"),
        &s(&env, "B2"),
        &None,
        &None,
    );
    client.mark_read(&user, &id1);

    let page = client.get_notifications(
        &user,
        &user,
        &status_filter(NotificationStatus::Pending, 50),
    );
    assert_eq!(page.notifications.len(), 1);
    assert_eq!(
        page.notifications.get(0).unwrap().status,
        NotificationStatus::Pending
    );
}

#[test]
fn test_get_notifications_second_page() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let user = Address::generate(&env);
    env.mock_all_auths();

    for _ in 0..5u32 {
        client.create_notification(
            &admin,
            &user,
            &NotificationType::RecordCreated,
            &AlertPriority::Low,
            &s(&env, "T"),
            &s(&env, "B"),
            &None,
            &None,
        );
    }
    let page = client.get_notifications(&user, &user, &all_filter(3, 3));
    assert_eq!(page.notifications.len(), 2);
    assert!(!page.has_more);
}

// ==================== Read / Archive ====================

#[test]
fn test_mark_read_transitions_status_and_unread_count() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let user = Address::generate(&env);
    env.mock_all_auths();

    let id = client.create_notification(
        &admin,
        &user,
        &NotificationType::AccessGranted,
        &AlertPriority::Medium,
        &s(&env, "Access"),
        &s(&env, "Granted"),
        &None,
        &None,
    );
    assert_eq!(client.get_unread_count(&user), 1);
    client.mark_read(&user, &id);
    assert_eq!(client.get_unread_count(&user), 0);

    let notif = client.get_notification(&user, &id);
    assert_eq!(notif.status, NotificationStatus::Read);
    assert!(notif.read_at.is_some());
}

#[test]
fn test_mark_read_twice_fails() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let user = Address::generate(&env);
    env.mock_all_auths();

    let id = client.create_notification(
        &admin,
        &user,
        &NotificationType::RecordCreated,
        &AlertPriority::Low,
        &s(&env, "T"),
        &s(&env, "B"),
        &None,
        &None,
    );
    client.mark_read(&user, &id);
    assert!(matches!(
        client.try_mark_read(&user, &id),
        Err(Ok(Error::AlreadyRead))
    ));
}

#[test]
fn test_mark_all_read_clears_all_unread() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let user = Address::generate(&env);
    env.mock_all_auths();

    for _ in 0..4u32 {
        client.create_notification(
            &admin,
            &user,
            &NotificationType::SystemAlert,
            &AlertPriority::Low,
            &s(&env, "T"),
            &s(&env, "B"),
            &None,
            &None,
        );
    }
    assert_eq!(client.get_unread_count(&user), 4);
    assert_eq!(client.mark_all_read(&user), 4);
    assert_eq!(client.get_unread_count(&user), 0);
}

#[test]
fn test_archive_notification() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let user = Address::generate(&env);
    env.mock_all_auths();

    let id = client.create_notification(
        &admin,
        &user,
        &NotificationType::RecordDeleted,
        &AlertPriority::Low,
        &s(&env, "T"),
        &s(&env, "B"),
        &None,
        &None,
    );
    client.archive_notification(&user, &id);
    assert_eq!(
        client.get_notification(&user, &id).status,
        NotificationStatus::Archived
    );
    assert_eq!(client.get_unread_count(&user), 0);
}

#[test]
fn test_archive_twice_fails() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let user = Address::generate(&env);
    env.mock_all_auths();

    let id = client.create_notification(
        &admin,
        &user,
        &NotificationType::RecordDeleted,
        &AlertPriority::Low,
        &s(&env, "T"),
        &s(&env, "B"),
        &None,
        &None,
    );
    client.archive_notification(&user, &id);
    assert!(matches!(
        client.try_archive_notification(&user, &id),
        Err(Ok(Error::AlreadyArchived))
    ));
}

#[test]
fn test_non_recipient_cannot_archive() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let user = Address::generate(&env);
    let stranger = Address::generate(&env);
    env.mock_all_auths();

    let id = client.create_notification(
        &admin,
        &user,
        &NotificationType::RecordCreated,
        &AlertPriority::Low,
        &s(&env, "T"),
        &s(&env, "B"),
        &None,
        &None,
    );
    assert!(matches!(
        client.try_archive_notification(&stranger, &id),
        Err(Ok(Error::NotAuthorized))
    ));
}

// ==================== Preference Filtering ====================

#[test]
fn test_low_priority_filtered_below_threshold() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let user = Address::generate(&env);
    env.mock_all_auths();

    client.set_preferences(&user, &make_prefs(&env, true, AlertPriority::High));

    let id = client.create_notification(
        &admin,
        &user,
        &NotificationType::RecordCreated,
        &AlertPriority::Low,
        &s(&env, "Low"),
        &s(&env, "Archived"),
        &None,
        &None,
    );
    assert_eq!(
        client.get_notification(&user, &id).status,
        NotificationStatus::Archived
    );
    assert_eq!(client.get_unread_count(&user), 0);
}

#[test]
fn test_critical_bypasses_disabled_preferences() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let user = Address::generate(&env);
    env.mock_all_auths();

    client.set_preferences(&user, &make_prefs(&env, false, AlertPriority::Critical));

    let id = client.create_notification(
        &admin,
        &user,
        &NotificationType::EmergencyAccessGranted,
        &AlertPriority::Critical,
        &s(&env, "Emergency"),
        &s(&env, "Emergency access"),
        &None,
        &None,
    );
    assert_eq!(
        client.get_notification(&user, &id).status,
        NotificationStatus::Pending
    );
    assert_eq!(client.get_unread_count(&user), 1);
}

#[test]
fn test_type_filter_allows_opted_types_only() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let user = Address::generate(&env);
    env.mock_all_auths();

    let mut enabled_types = Vec::new(&env);
    enabled_types.push_back(NotificationType::AccessGranted as u32);
    let prefs = NotificationPreferences {
        enabled: true,
        min_priority: AlertPriority::Low,
        channel: NotificationChannel::OnChain,
        enabled_types,
        updated_at: 0,
    };
    client.set_preferences(&user, &prefs);

    // RecordCreated not in allow-list → archived.
    let id1 = client.create_notification(
        &admin,
        &user,
        &NotificationType::RecordCreated,
        &AlertPriority::Medium,
        &s(&env, "Rec"),
        &s(&env, "Not opted-in"),
        &None,
        &None,
    );
    assert_eq!(
        client.get_notification(&user, &id1).status,
        NotificationStatus::Archived
    );

    // AccessGranted is in allow-list → pending.
    let id2 = client.create_notification(
        &admin,
        &user,
        &NotificationType::AccessGranted,
        &AlertPriority::Medium,
        &s(&env, "Access"),
        &s(&env, "Opted-in"),
        &None,
        &None,
    );
    assert_eq!(
        client.get_notification(&user, &id2).status,
        NotificationStatus::Pending
    );
}

// ==================== Alert Rules ====================

#[test]
fn test_create_and_list_alert_rules() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    env.mock_all_auths();

    let rule_id = client.create_alert_rule(
        &admin,
        &s(&env, "Record Watch"),
        &(NotificationType::RecordCreated as u32),
        &AlertPriority::High,
        &Vec::new(&env),
    );
    assert_eq!(rule_id, 1);
    assert_eq!(client.get_alert_rules(&admin).len(), 1);
}

#[test]
fn test_update_alert_rule_changes_state() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    env.mock_all_auths();

    let rule_id = client.create_alert_rule(
        &admin,
        &s(&env, "Rule"),
        &(NotificationType::AnomalyDetected as u32),
        &AlertPriority::Medium,
        &Vec::new(&env),
    );
    client.update_alert_rule(
        &admin,
        &rule_id,
        &false,
        &AlertPriority::High,
        &Vec::new(&env),
    );

    let updated = client.get_alert_rules(&admin).get(0).unwrap();
    assert!(!updated.is_active);
    assert_eq!(updated.priority, AlertPriority::High);
}

#[test]
fn test_delete_alert_rule_removes_it() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    env.mock_all_auths();

    let rule_id = client.create_alert_rule(
        &admin,
        &s(&env, "Temp"),
        &(NotificationType::SystemAlert as u32),
        &AlertPriority::Low,
        &Vec::new(&env),
    );
    client.delete_alert_rule(&admin, &rule_id);
    assert_eq!(client.get_alert_rules(&admin).len(), 0);
}

#[test]
fn test_delete_missing_rule_fails() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    env.mock_all_auths();
    assert!(matches!(
        client.try_delete_alert_rule(&admin, &999u64),
        Err(Ok(Error::AlertRuleNotFound))
    ));
}

#[test]
fn test_trigger_alert_creates_notifications_for_all_recipients() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let r1 = Address::generate(&env);
    let r2 = Address::generate(&env);
    env.mock_all_auths();

    let mut recipients = Vec::new(&env);
    recipients.push_back(r1.clone());
    recipients.push_back(r2.clone());

    let rule_id = client.create_alert_rule(
        &admin,
        &s(&env, "Emergency Rule"),
        &(NotificationType::EmergencyAccessGranted as u32),
        &AlertPriority::Critical,
        &recipients,
    );
    let ids = client.trigger_alert(&admin, &rule_id, &Some(42u64), &None);
    assert_eq!(ids.len(), 2);
    assert_eq!(client.get_unread_count(&r1), 1);
    assert_eq!(client.get_unread_count(&r2), 1);
}

#[test]
fn test_trigger_inactive_rule_creates_no_notifications() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let r = Address::generate(&env);
    env.mock_all_auths();

    let mut recipients = Vec::new(&env);
    recipients.push_back(r.clone());

    let rule_id = client.create_alert_rule(
        &admin,
        &s(&env, "Inactive"),
        &(NotificationType::SystemAlert as u32),
        &AlertPriority::Low,
        &recipients,
    );
    client.update_alert_rule(
        &admin,
        &rule_id,
        &false,
        &AlertPriority::Low,
        &Vec::new(&env),
    );

    assert_eq!(
        client.trigger_alert(&admin, &rule_id, &None, &None).len(),
        0
    );
    assert_eq!(client.get_unread_count(&r), 0);
}

#[test]
fn test_trigger_rule_with_no_recipients_returns_empty() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    env.mock_all_auths();

    let rule_id = client.create_alert_rule(
        &admin,
        &s(&env, "No Recip"),
        &(NotificationType::SystemAlert as u32),
        &AlertPriority::Low,
        &Vec::new(&env),
    );
    assert_eq!(
        client.trigger_alert(&admin, &rule_id, &None, &None).len(),
        0
    );
}

// ==================== Templates ====================

#[test]
fn test_set_and_get_template() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    env.mock_all_auths();

    client.set_template(
        &admin,
        &NotificationTemplate {
            notif_type: NotificationType::RecordCreated as u32,
            locale: s(&env, "en"),
            title: s(&env, "New Record"),
            message: s(&env, "A record was created."),
            default_priority: AlertPriority::Medium,
            updated_at: 0,
        },
    );
    let fetched = client.get_template(&(NotificationType::RecordCreated as u32), &s(&env, "en"));
    assert_eq!(fetched.title, s(&env, "New Record"));
}

#[test]
fn test_get_missing_template_fails() {
    let env = Env::default();
    let (client, _) = setup(&env);
    env.mock_all_auths();
    assert!(matches!(
        client.try_get_template(&42u32, &s(&env, "xx")),
        Err(Ok(Error::TemplateNotFound))
    ));
}

#[test]
fn test_multiple_locale_templates_are_independent() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    env.mock_all_auths();

    let ntype = NotificationType::AccessGranted as u32;
    client.set_template(
        &admin,
        &NotificationTemplate {
            notif_type: ntype,
            locale: s(&env, "en"),
            title: s(&env, "Access Granted"),
            message: s(&env, "You have access."),
            default_priority: AlertPriority::High,
            updated_at: 0,
        },
    );
    client.set_template(
        &admin,
        &NotificationTemplate {
            notif_type: ntype,
            locale: s(&env, "fr"),
            title: s(&env, "Acces accorde"),
            message: s(&env, "Vous avez acces."),
            default_priority: AlertPriority::High,
            updated_at: 0,
        },
    );

    assert_eq!(
        client.get_template(&ntype, &s(&env, "en")).title,
        s(&env, "Access Granted")
    );
    assert_eq!(
        client.get_template(&ntype, &s(&env, "fr")).title,
        s(&env, "Acces accorde")
    );
}

// ==================== Analytics ====================

#[test]
fn test_analytics_counts_total_sent_and_pending() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let user = Address::generate(&env);
    env.mock_all_auths();

    client.create_notification(
        &admin,
        &user,
        &NotificationType::RecordCreated,
        &AlertPriority::Medium,
        &s(&env, "T1"),
        &s(&env, "B1"),
        &None,
        &None,
    );
    client.create_notification(
        &admin,
        &user,
        &NotificationType::RecordUpdated,
        &AlertPriority::High,
        &s(&env, "T2"),
        &s(&env, "B2"),
        &None,
        &None,
    );
    let a = client.get_analytics(&admin);
    assert_eq!(a.total_sent, 2);
    assert_eq!(a.total_pending, 2);
}

#[test]
fn test_analytics_updates_read_and_pending_on_mark_read() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let user = Address::generate(&env);
    env.mock_all_auths();

    let id = client.create_notification(
        &admin,
        &user,
        &NotificationType::RecordCreated,
        &AlertPriority::Medium,
        &s(&env, "T"),
        &s(&env, "B"),
        &None,
        &None,
    );
    client.create_notification(
        &admin,
        &user,
        &NotificationType::RecordUpdated,
        &AlertPriority::High,
        &s(&env, "T2"),
        &s(&env, "B2"),
        &None,
        &None,
    );
    client.mark_read(&user, &id);

    let a = client.get_analytics(&admin);
    assert_eq!(a.total_sent, 2);
    assert_eq!(a.total_read, 1);
    assert_eq!(a.total_pending, 1);
}

#[test]
fn test_analytics_by_type_tracks_counts() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let user = Address::generate(&env);
    env.mock_all_auths();

    let ntype = NotificationType::AccessDenied;
    for _ in 0..3u32 {
        client.create_notification(
            &admin,
            &user,
            &ntype,
            &AlertPriority::High,
            &s(&env, "Denied"),
            &s(&env, "Denied"),
            &None,
            &None,
        );
    }
    assert_eq!(
        client
            .get_analytics(&admin)
            .by_type
            .get(ntype as u32)
            .unwrap_or(0),
        3u64
    );
}

#[test]
fn test_analytics_by_priority_tracks_counts() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let user = Address::generate(&env);
    env.mock_all_auths();

    client.create_notification(
        &admin,
        &user,
        &NotificationType::EmergencyAccessGranted,
        &AlertPriority::Critical,
        &s(&env, "Emergency"),
        &s(&env, "Emergency"),
        &None,
        &None,
    );
    let a = client.get_analytics(&admin);
    assert_eq!(
        a.by_priority
            .get(AlertPriority::Critical as u32)
            .unwrap_or(0),
        1u64
    );
}

#[test]
fn test_non_admin_cannot_get_analytics() {
    let env = Env::default();
    let (client, _) = setup(&env);
    env.mock_all_auths();
    assert!(matches!(
        client.try_get_analytics(&Address::generate(&env)),
        Err(Ok(Error::NotAuthorized))
    ));
}

// ==================== Rate Limiting ====================

#[test]
fn test_rate_limit_blocks_after_100_calls() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let sender = Address::generate(&env);
    let user = Address::generate(&env);
    env.mock_all_auths();

    client.add_authorized_sender(&admin, &sender);
    for _ in 0..100u32 {
        client.create_notification(
            &sender,
            &user,
            &NotificationType::SystemAlert,
            &AlertPriority::Low,
            &s(&env, "T"),
            &s(&env, "B"),
            &None,
            &None,
        );
    }
    assert!(matches!(
        client.try_create_notification(
            &sender,
            &user,
            &NotificationType::SystemAlert,
            &AlertPriority::Low,
            &s(&env, "T"),
            &s(&env, "B"),
            &None,
            &None,
        ),
        Err(Ok(Error::RateLimitExceeded))
    ));
}

// ==================== Integration ====================

#[test]
fn test_authorized_external_sender_can_create() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);
    env.mock_all_auths();

    client.add_authorized_sender(&admin, &sender);
    let id = client.create_notification(
        &sender,
        &recipient,
        &NotificationType::RecordCreated,
        &AlertPriority::Medium,
        &s(&env, "Record Created"),
        &s(&env, "Your record was created."),
        &Some(1u64),
        &None,
    );
    assert_eq!(client.get_notification(&recipient, &id).sender, sender);
}

#[test]
fn test_all_major_operations_emit_events() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let user = Address::generate(&env);
    let sender = Address::generate(&env);
    env.mock_all_auths();

    let count_before = env.events().all().len();

    client.add_authorized_sender(&admin, &sender);
    client.set_preferences(&user, &make_prefs(&env, true, AlertPriority::Low));
    let id = client.create_notification(
        &sender,
        &user,
        &NotificationType::RecordCreated,
        &AlertPriority::Low,
        &s(&env, "T"),
        &s(&env, "B"),
        &None,
        &None,
    );
    client.mark_read(&user, &id);
    client.create_alert_rule(
        &admin,
        &s(&env, "Rule"),
        &0u32,
        &AlertPriority::High,
        &Vec::new(&env),
    );
    client.set_template(
        &admin,
        &NotificationTemplate {
            notif_type: 0,
            locale: s(&env, "en"),
            title: s(&env, "T"),
            message: s(&env, "M"),
            default_priority: AlertPriority::Low,
            updated_at: 0,
        },
    );

    let new_events = env.events().all().len() - count_before;
    // SNDR_ADD, PREF_UPD, NOTIF_NEW, NOTIF_RD, ALRT_NEW, TMPL_SET = 6 minimum
    assert!(new_events >= 6, "Expected ≥6 events, got {}", new_events);
}
