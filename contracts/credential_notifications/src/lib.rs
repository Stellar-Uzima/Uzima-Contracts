#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, vec, Address, BytesN, Env,
    String, Symbol, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    NotificationNotFound = 4,
    InvalidNotificationType = 5,
    DuplicateNotification = 6,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum NotificationType {
    ExpirationWarning = 0,
    Expired = 1,
    RenewalRequired = 2,
    VerificationRequired = 3,
    SuspensionWarning = 4,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CredentialNotification {
    pub notification_id: BytesN<32>,
    pub provider: Address,
    pub credential_id: BytesN<32>,
    pub notification_type: NotificationType,
    pub message: String,
    pub timestamp: u64,
    pub is_read: bool,
    pub action_required: bool,
    pub deadline: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotificationSettings {
    pub provider: Address,
    pub expiration_warning_days: u32, // Days before expiration to warn
    pub renewal_reminder_days: u32,   // Days after expiration to remind
    pub enable_notifications: bool,
    pub notification_channels: Vec<Symbol>, // email, sms, in_app
}

#[contracttype]
pub enum DataKey {
    Admin,
    Initialized,
    Notification(BytesN<32>),
    ProviderNotifications(Address), // Vec<BytesN<32>>
    NotificationSettings(Address),
    ScheduledNotification(Address, u64), // timestamp -> notification_id
    CredentialNotifications(Address, BytesN<32>), // Vec<BytesN<32>>
}

#[contract]
pub struct CredentialNotificationSystem;

#[contractimpl]
impl CredentialNotificationSystem {
    // Initialize notification system
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Initialized, &true);

        env.events()
            .publish((symbol_short!("CREDNTIF"), symbol_short!("INIT")), admin);
        Ok(())
    }

    // Set notification preferences for provider
    pub fn set_notification_settings(
        env: Env,
        provider: Address,
        settings: NotificationSettings,
    ) -> Result<(), Error> {
        provider.require_auth();
        Self::require_initialized(&env)?;

        env.storage()
            .persistent()
            .set(&DataKey::NotificationSettings(provider.clone()), &settings);

        env.events().publish(
            (symbol_short!("CREDNTIF"), symbol_short!("SETTINGS")),
            provider,
        );
        Ok(())
    }

    // Create expiration warning notification
    pub fn create_expiration_warning(
        env: Env,
        admin: Address,
        provider: Address,
        credential_id: BytesN<32>,
        expiration_date: u64,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let settings = Self::get_notification_settings(&env, &provider)?;
        if !settings.enable_notifications {
            return Ok(());
        }

        let current_time = env.ledger().timestamp();
        let days_until_expiration = (expiration_date.saturating_sub(current_time)) / (24 * 60 * 60);

        if days_until_expiration <= settings.expiration_warning_days as u64 {
            let notification_id = Self::generate_notification_id(&env, &provider, 1);

            let message = String::from_str(
                &env,
                "Your credential is expiring soon. Please renew before the deadline.",
            );

            let notification = CredentialNotification {
                notification_id: notification_id.clone(),
                provider: provider.clone(),
                credential_id,
                notification_type: NotificationType::ExpirationWarning,
                message,
                timestamp: current_time,
                is_read: false,
                action_required: true,
                deadline: expiration_date,
            };

            Self::store_notification(&env, notification)?;
        }

        Ok(())
    }

    // Create expired credential notification
    pub fn create_expired_notification(
        env: Env,
        admin: Address,
        provider: Address,
        credential_id: BytesN<32>,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let settings = Self::get_notification_settings(&env, &provider)?;
        if !settings.enable_notifications {
            return Ok(());
        }

        let notification_id = Self::generate_notification_id(&env, &provider, 2);
        let current_time = env.ledger().timestamp();

        let message = String::from_str(
            &env,
            "Your credential has expired. Please renew immediately to maintain your verified status.",
        );

        let deadline =
            current_time.saturating_add(settings.renewal_reminder_days as u64 * 24 * 60 * 60);

        let notification = CredentialNotification {
            notification_id: notification_id.clone(),
            provider: provider.clone(),
            credential_id: credential_id.clone(),
            notification_type: NotificationType::Expired,
            message,
            timestamp: current_time,
            is_read: false,
            action_required: true,
            deadline,
        };

        Self::store_notification(&env, notification)?;

        // Schedule renewal reminders
        Self::schedule_renewal_reminders(
            &env,
            provider,
            credential_id,
            settings.renewal_reminder_days,
        )?;

        Ok(())
    }

    // Create renewal required notification
    pub fn create_renewal_notification(
        env: Env,
        admin: Address,
        provider: Address,
        credential_id: BytesN<32>,
        renewal_deadline: u64,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let settings = Self::get_notification_settings(&env, &provider)?;
        if !settings.enable_notifications {
            return Ok(());
        }

        let notification_id = Self::generate_notification_id(&env, &provider, 3);
        let current_time = env.ledger().timestamp();

        let message = String::from_str(
            &env,
            "Credential renewal required. Please complete renewal before the deadline.",
        );

        let notification = CredentialNotification {
            notification_id: notification_id.clone(),
            provider: provider.clone(),
            credential_id,
            notification_type: NotificationType::RenewalRequired,
            message,
            timestamp: current_time,
            is_read: false,
            action_required: true,
            deadline: renewal_deadline,
        };

        Self::store_notification(&env, notification)?;
        Ok(())
    }

    // Create verification required notification
    pub fn create_verification_notification(
        env: Env,
        admin: Address,
        provider: Address,
        credential_id: BytesN<32>,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let settings = Self::get_notification_settings(&env, &provider)?;
        if !settings.enable_notifications {
            return Ok(());
        }

        let notification_id = Self::generate_notification_id(&env, &provider, 4);
        let current_time = env.ledger().timestamp();

        let message = String::from_str(
            &env,
            "Your credential requires verification. Please submit required documentation.",
        );

        let notification = CredentialNotification {
            notification_id: notification_id.clone(),
            provider: provider.clone(),
            credential_id,
            notification_type: NotificationType::VerificationRequired,
            message,
            timestamp: current_time,
            is_read: false,
            action_required: true,
            deadline: current_time.saturating_add(30 * 24 * 60 * 60), // 30 days
        };

        Self::store_notification(&env, notification)?;
        Ok(())
    }

    // Mark notification as read
    pub fn mark_notification_read(
        env: Env,
        provider: Address,
        notification_id: BytesN<32>,
    ) -> Result<(), Error> {
        provider.require_auth();
        Self::require_initialized(&env)?;

        let mut notification: CredentialNotification = env
            .storage()
            .persistent()
            .get(&DataKey::Notification(notification_id.clone()))
            .ok_or(Error::NotificationNotFound)?;

        if notification.provider != provider {
            return Err(Error::NotAuthorized);
        }

        notification.is_read = true;
        env.storage().persistent().set(
            &DataKey::Notification(notification_id.clone()),
            &notification,
        );

        env.events().publish(
            (symbol_short!("CREDNTIF"), symbol_short!("READ")),
            notification_id,
        );
        Ok(())
    }

    // Get provider notifications
    pub fn get_provider_notifications(
        env: Env,
        provider: Address,
        unread_only: bool,
    ) -> Result<Vec<CredentialNotification>, Error> {
        Self::require_initialized(&env)?;

        let notification_ids: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::ProviderNotifications(provider))
            .unwrap_or(Vec::new(&env));

        let mut notifications = Vec::new(&env);
        for notification_id in notification_ids.iter() {
            if let Some(notification) = env
                .storage()
                .persistent()
                .get::<DataKey, CredentialNotification>(&DataKey::Notification(
                    notification_id.clone(),
                ))
            {
                if !unread_only || !notification.is_read {
                    notifications.push_back(notification);
                }
            }
        }

        Ok(notifications)
    }

    // Get unread notification count
    pub fn get_unread_count(env: Env, provider: Address) -> Result<u32, Error> {
        Self::require_initialized(&env)?;

        let notifications = Self::get_provider_notifications(env, provider, true)?;
        Ok(notifications.len())
    }

    // Get notifications by credential
    pub fn get_credential_notifications(
        env: Env,
        provider: Address,
        credential_id: BytesN<32>,
    ) -> Result<Vec<CredentialNotification>, Error> {
        Self::require_initialized(&env)?;

        let notification_ids: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::CredentialNotifications(
                provider.clone(),
                credential_id.clone(),
            ))
            .unwrap_or(Vec::new(&env));

        let mut notifications = Vec::new(&env);
        for notification_id in notification_ids.iter() {
            if let Some(notification) = env
                .storage()
                .persistent()
                .get::<DataKey, CredentialNotification>(&DataKey::Notification(
                    notification_id.clone(),
                ))
            {
                notifications.push_back(notification);
            }
        }

        Ok(notifications)
    }

    // Process scheduled notifications (called by cron job)
    pub fn process_scheduled_notifications(env: Env, admin: Address) -> Result<u32, Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let processed_count: u32 = 0;

        env.events().publish(
            (symbol_short!("CREDNTIF"), symbol_short!("PROCESSD")),
            processed_count,
        );
        Ok(processed_count)
    }

    // Store notification helper
    fn store_notification(env: &Env, notification: CredentialNotification) -> Result<(), Error> {
        let notification_id = notification.notification_id.clone();
        let provider = notification.provider.clone();
        let credential_id = notification.credential_id.clone();

        // Store notification
        env.storage().persistent().set(
            &DataKey::Notification(notification_id.clone()),
            &notification,
        );

        // Update provider's notification list
        let mut provider_notifications: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::ProviderNotifications(provider.clone()))
            .unwrap_or(Vec::new(env));
        provider_notifications.push_back(notification_id.clone());
        env.storage().persistent().set(
            &DataKey::ProviderNotifications(provider.clone()),
            &provider_notifications,
        );

        // Update credential's notification list
        let mut credential_notifications: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::CredentialNotifications(
                provider.clone(),
                credential_id.clone(),
            ))
            .unwrap_or(Vec::new(env));
        credential_notifications.push_back(notification_id.clone());
        env.storage().persistent().set(
            &DataKey::CredentialNotifications(provider, credential_id),
            &credential_notifications,
        );

        env.events().publish(
            (symbol_short!("CREDNTIF"), symbol_short!("CREATED")),
            notification_id,
        );
        Ok(())
    }

    // Schedule renewal reminders helper
    fn schedule_renewal_reminders(
        env: &Env,
        provider: Address,
        _credential_id: BytesN<32>,
        reminder_days: u32,
    ) -> Result<(), Error> {
        let current_time = env.ledger().timestamp();

        // Schedule reminders at different intervals (in days, as u32)
        let intervals: [u32; 3] = [7, 14, 30];

        for days in intervals {
            if days <= reminder_days {
                let reminder_time = current_time.saturating_add(days as u64 * 24 * 60 * 60);
                let notification_id = Self::generate_notification_id(env, &provider, days);

                env.storage().persistent().set(
                    &DataKey::ScheduledNotification(provider.clone(), reminder_time),
                    &notification_id,
                );
            }
        }

        Ok(())
    }

    // Generate unique notification ID
    fn generate_notification_id(env: &Env, _provider: &Address, sequence: u32) -> BytesN<32> {
        let timestamp = env.ledger().timestamp();
        let mut data = [0u8; 32];

        // Use timestamp and sequence to create unique ID
        data[0..8].copy_from_slice(&timestamp.to_be_bytes());
        data[8..12].copy_from_slice(&sequence.to_be_bytes());

        BytesN::from_array(env, &data)
    }

    // Get notification settings with defaults
    fn get_notification_settings(
        env: &Env,
        provider: &Address,
    ) -> Result<NotificationSettings, Error> {
        if let Some(settings) = env
            .storage()
            .persistent()
            .get::<DataKey, NotificationSettings>(&DataKey::NotificationSettings(provider.clone()))
        {
            Ok(settings)
        } else {
            // Return default settings
            Ok(NotificationSettings {
                provider: provider.clone(),
                expiration_warning_days: 30,
                renewal_reminder_days: 14,
                enable_notifications: true,
                notification_channels: vec![env, symbol_short!("email"), symbol_short!("in_app")],
            })
        }
    }

    // Helper functions
    fn require_initialized(env: &Env) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Initialized) {
            Ok(())
        } else {
            Err(Error::NotInitialized)
        }
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if admin == *caller {
            Ok(())
        } else {
            Err(Error::NotAuthorized)
        }
    }
}
