#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Vec};

// ── Retention periods (seconds) ──────────────────────────────────────────────
const CREDENTIAL_RETENTION: u64 = 365 * 24 * 3600;   // 1 year
const AUDIT_LOG_RETENTION: u64  = 7 * 365 * 24 * 3600; // 7 years
const ESCROW_RETENTION: u64     = 90 * 24 * 3600;    // 90 days after settlement
const CONSENT_RETENTION: u64    = 2 * 365 * 24 * 3600; // 2 years after revocation
const SCHEDULE_RETENTION: u64   = 30 * 24 * 3600;    // 30 days past end date

// Safety margin: never delete items newer than this (seconds)
const SAFETY_MARGIN: u64 = 24 * 3600; // 1 day

const MAX_BATCH: u32 = 100;

// ── Storage keys ─────────────────────────────────────────────────────────────
#[contracttype]
pub enum DataKey {
    Admin,
    Paused,
    // Indexed lists of item IDs per category
    CredentialIds,
    AuditLogIds,
    EscrowIds,
    ConsentIds,
    ScheduleIds,
    // Per-item metadata (expiry timestamp)
    CredentialExpiry(u64),
    AuditLogExpiry(u64),
    EscrowSettledAt(u64),
    ConsentRevokedAt(u64),
    ScheduleEndAt(u64),
    // Cleanup audit trail
    CleanupLog,
    // Configurable retention overrides
    RetentionConfig,
}

#[contracttype]
#[derive(Clone)]
pub struct RetentionConfig {
    pub credential_secs: u64,
    pub audit_log_secs: u64,
    pub escrow_secs: u64,
    pub consent_secs: u64,
    pub schedule_secs: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct CleanupEntry {
    pub timestamp: u64,
    pub caller: Address,
    pub category: u32, // 0=cred,1=audit,2=escrow,3=consent,4=schedule
    pub count: u32,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    NotAuthorized = 3,
    Paused = 4,
    BatchTooLarge = 5,
}

#[contract]
pub struct StorageCleanup;

#[contractimpl]
impl StorageCleanup {
    // ── Admin ─────────────────────────────────────────────────────────────────

    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
        Ok(())
    }

    pub fn set_paused(env: Env, caller: Address, paused: bool) -> Result<(), Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        env.storage().instance().set(&DataKey::Paused, &paused);
        Ok(())
    }

    pub fn set_retention_config(env: Env, caller: Address, config: RetentionConfig) -> Result<(), Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        env.storage().instance().set(&DataKey::RetentionConfig, &config);
        Ok(())
    }

    // ── Registration helpers (called by other contracts or admin) ─────────────

    pub fn register_credential(env: Env, id: u64, expires_at: u64) {
        let mut ids: Vec<u64> = env.storage().persistent()
            .get(&DataKey::CredentialIds).unwrap_or(Vec::new(&env));
        ids.push_back(id);
        env.storage().persistent().set(&DataKey::CredentialIds, &ids);
        env.storage().persistent().set(&DataKey::CredentialExpiry(id), &expires_at);
    }

    pub fn register_audit_log(env: Env, id: u64, logged_at: u64) {
        let mut ids: Vec<u64> = env.storage().persistent()
            .get(&DataKey::AuditLogIds).unwrap_or(Vec::new(&env));
        ids.push_back(id);
        env.storage().persistent().set(&DataKey::AuditLogIds, &ids);
        env.storage().persistent().set(&DataKey::AuditLogExpiry(id), &logged_at);
    }

    pub fn register_escrow(env: Env, id: u64, settled_at: u64) {
        let mut ids: Vec<u64> = env.storage().persistent()
            .get(&DataKey::EscrowIds).unwrap_or(Vec::new(&env));
        ids.push_back(id);
        env.storage().persistent().set(&DataKey::EscrowIds, &ids);
        env.storage().persistent().set(&DataKey::EscrowSettledAt(id), &settled_at);
    }

    pub fn register_consent(env: Env, id: u64, revoked_at: u64) {
        let mut ids: Vec<u64> = env.storage().persistent()
            .get(&DataKey::ConsentIds).unwrap_or(Vec::new(&env));
        ids.push_back(id);
        env.storage().persistent().set(&DataKey::ConsentIds, &ids);
        env.storage().persistent().set(&DataKey::ConsentRevokedAt(id), &revoked_at);
    }

    pub fn register_schedule(env: Env, id: u64, end_at: u64) {
        let mut ids: Vec<u64> = env.storage().persistent()
            .get(&DataKey::ScheduleIds).unwrap_or(Vec::new(&env));
        ids.push_back(id);
        env.storage().persistent().set(&DataKey::ScheduleIds, &ids);
        env.storage().persistent().set(&DataKey::ScheduleEndAt(id), &end_at);
    }

    // ── Core cleanup ──────────────────────────────────────────────────────────

    /// Clean up expired items across all categories.
    /// Returns total number of items removed.
    pub fn cleanup_expired(env: Env, caller: Address, max_items: u32) -> Result<u32, Error> {
        caller.require_auth();
        Self::require_initialized(&env)?;
        Self::require_not_paused(&env)?;
        if max_items > MAX_BATCH {
            return Err(Error::BatchTooLarge);
        }

        let per_cat = (max_items / 5).max(1);
        let mut total = 0u32;

        total += Self::cleanup_expired_credentials(&env, per_cat);
        total += Self::cleanup_old_audit_logs(&env, per_cat);
        total += Self::cleanup_completed_escrows(&env, per_cat);
        total += Self::cleanup_expired_consents(&env, per_cat);
        total += Self::cleanup_outdated_schedules(&env, per_cat);

        Self::record_cleanup(&env, &caller, 255, total);

        env.events().publish(
            (symbol_short!("CLEANUP"), symbol_short!("ALL")),
            (caller, total),
        );

        Ok(total)
    }

    /// Preview how many items would be cleaned without removing them.
    pub fn preview_cleanup(env: Env, max_items: u32) -> Result<u32, Error> {
        Self::require_initialized(&env)?;
        if max_items > MAX_BATCH {
            return Err(Error::BatchTooLarge);
        }
        let now = env.ledger().timestamp();
        let cfg = Self::get_config(&env);
        let per_cat = (max_items / 5).max(1);

        let c = Self::count_expired_in_list(
            &env, &DataKey::CredentialIds,
            |id| env.storage().persistent().get::<_, u64>(&DataKey::CredentialExpiry(id)).unwrap_or(0),
            cfg.credential_secs, now, per_cat,
        );
        let a = Self::count_expired_in_list(
            &env, &DataKey::AuditLogIds,
            |id| env.storage().persistent().get::<_, u64>(&DataKey::AuditLogExpiry(id)).unwrap_or(0),
            cfg.audit_log_secs, now, per_cat,
        );
        let e = Self::count_expired_in_list(
            &env, &DataKey::EscrowIds,
            |id| env.storage().persistent().get::<_, u64>(&DataKey::EscrowSettledAt(id)).unwrap_or(0),
            cfg.escrow_secs, now, per_cat,
        );
        let co = Self::count_expired_in_list(
            &env, &DataKey::ConsentIds,
            |id| env.storage().persistent().get::<_, u64>(&DataKey::ConsentRevokedAt(id)).unwrap_or(0),
            cfg.consent_secs, now, per_cat,
        );
        let s = Self::count_expired_in_list(
            &env, &DataKey::ScheduleIds,
            |id| env.storage().persistent().get::<_, u64>(&DataKey::ScheduleEndAt(id)).unwrap_or(0),
            cfg.schedule_secs, now, per_cat,
        );

        Ok(c + a + e + co + s)
    }

    // ── Category-specific cleanup ─────────────────────────────────────────────

    pub fn cleanup_credentials(env: Env, caller: Address, max_items: u32) -> Result<u32, Error> {
        caller.require_auth();
        Self::require_initialized(&env)?;
        Self::require_not_paused(&env)?;
        let n = Self::cleanup_expired_credentials(&env, max_items.min(MAX_BATCH));
        Self::record_cleanup(&env, &caller, 0, n);
        Ok(n)
    }

    pub fn cleanup_audit_logs(env: Env, caller: Address, max_items: u32) -> Result<u32, Error> {
        caller.require_auth();
        Self::require_initialized(&env)?;
        Self::require_not_paused(&env)?;
        let n = Self::cleanup_old_audit_logs(&env, max_items.min(MAX_BATCH));
        Self::record_cleanup(&env, &caller, 1, n);
        Ok(n)
    }

    pub fn cleanup_escrows(env: Env, caller: Address, max_items: u32) -> Result<u32, Error> {
        caller.require_auth();
        Self::require_initialized(&env)?;
        Self::require_not_paused(&env)?;
        let n = Self::cleanup_completed_escrows(&env, max_items.min(MAX_BATCH));
        Self::record_cleanup(&env, &caller, 2, n);
        Ok(n)
    }

    pub fn cleanup_consents(env: Env, caller: Address, max_items: u32) -> Result<u32, Error> {
        caller.require_auth();
        Self::require_initialized(&env)?;
        Self::require_not_paused(&env)?;
        let n = Self::cleanup_expired_consents(&env, max_items.min(MAX_BATCH));
        Self::record_cleanup(&env, &caller, 3, n);
        Ok(n)
    }

    pub fn cleanup_schedules(env: Env, caller: Address, max_items: u32) -> Result<u32, Error> {
        caller.require_auth();
        Self::require_initialized(&env)?;
        Self::require_not_paused(&env)?;
        let n = Self::cleanup_outdated_schedules(&env, max_items.min(MAX_BATCH));
        Self::record_cleanup(&env, &caller, 4, n);
        Ok(n)
    }

    // ── Queries ───────────────────────────────────────────────────────────────

    pub fn get_cleanup_log(env: Env) -> Vec<CleanupEntry> {
        env.storage().persistent()
            .get(&DataKey::CleanupLog)
            .unwrap_or(Vec::new(&env))
    }

    pub fn is_paused(env: Env) -> bool {
        env.storage().instance().get(&DataKey::Paused).unwrap_or(false)
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

impl StorageCleanup {
    fn cleanup_expired_credentials(env: &Env, max_items: u32) -> u32 {
        let cfg = Self::get_config(env);
        Self::remove_expired(
            env,
            &DataKey::CredentialIds,
            |id| env.storage().persistent().get::<_, u64>(&DataKey::CredentialExpiry(id)).unwrap_or(0),
            |id| { env.storage().persistent().remove(&DataKey::CredentialExpiry(id)); },
            cfg.credential_secs,
            max_items,
        )
    }

    fn cleanup_old_audit_logs(env: &Env, max_items: u32) -> u32 {
        let cfg = Self::get_config(env);
        Self::remove_expired(
            env,
            &DataKey::AuditLogIds,
            |id| env.storage().persistent().get::<_, u64>(&DataKey::AuditLogExpiry(id)).unwrap_or(0),
            |id| { env.storage().persistent().remove(&DataKey::AuditLogExpiry(id)); },
            cfg.audit_log_secs,
            max_items,
        )
    }

    fn cleanup_completed_escrows(env: &Env, max_items: u32) -> u32 {
        let cfg = Self::get_config(env);
        Self::remove_expired(
            env,
            &DataKey::EscrowIds,
            |id| env.storage().persistent().get::<_, u64>(&DataKey::EscrowSettledAt(id)).unwrap_or(0),
            |id| { env.storage().persistent().remove(&DataKey::EscrowSettledAt(id)); },
            cfg.escrow_secs,
            max_items,
        )
    }

    fn cleanup_expired_consents(env: &Env, max_items: u32) -> u32 {
        let cfg = Self::get_config(env);
        Self::remove_expired(
            env,
            &DataKey::ConsentIds,
            |id| env.storage().persistent().get::<_, u64>(&DataKey::ConsentRevokedAt(id)).unwrap_or(0),
            |id| { env.storage().persistent().remove(&DataKey::ConsentRevokedAt(id)); },
            cfg.consent_secs,
            max_items,
        )
    }

    fn cleanup_outdated_schedules(env: &Env, max_items: u32) -> u32 {
        let cfg = Self::get_config(env);
        Self::remove_expired(
            env,
            &DataKey::ScheduleIds,
            |id| env.storage().persistent().get::<_, u64>(&DataKey::ScheduleEndAt(id)).unwrap_or(0),
            |id| { env.storage().persistent().remove(&DataKey::ScheduleEndAt(id)); },
            cfg.schedule_secs,
            max_items,
        )
    }

    /// Generic removal: iterates the ID list, removes expired entries, rewrites the list.
    fn remove_expired<FGet, FDel>(
        env: &Env,
        list_key: &DataKey,
        get_ts: FGet,
        del_data: FDel,
        retention: u64,
        max_items: u32,
    ) -> u32
    where
        FGet: Fn(u64) -> u64,
        FDel: Fn(u64),
    {
        let now = env.ledger().timestamp();
        let cutoff = now.saturating_sub(retention).saturating_sub(SAFETY_MARGIN);

        let ids: Vec<u64> = env.storage().persistent()
            .get(list_key).unwrap_or(Vec::new(env));

        let mut kept = Vec::new(env);
        let mut cleaned = 0u32;

        for id in ids.iter() {
            if cleaned < max_items {
                let ts = get_ts(id);
                // ts == 0 means not yet set (active item) — keep it
                if ts > 0 && ts < cutoff {
                    del_data(id);
                    cleaned += 1;
                    continue;
                }
            }
            kept.push_back(id);
        }

        env.storage().persistent().set(list_key, &kept);
        cleaned
    }

    /// Count-only variant (no mutations) used by preview_cleanup.
    fn count_expired_in_list<FGet>(
        env: &Env,
        list_key: &DataKey,
        get_ts: FGet,
        retention: u64,
        now: u64,
        max_items: u32,
    ) -> u32
    where
        FGet: Fn(u64) -> u64,
    {
        let cutoff = now.saturating_sub(retention).saturating_sub(SAFETY_MARGIN);
        let ids: Vec<u64> = env.storage().persistent()
            .get(list_key).unwrap_or(Vec::new(env));
        let mut count = 0u32;
        for id in ids.iter() {
            if count >= max_items { break; }
            let ts = get_ts(id);
            if ts > 0 && ts < cutoff {
                count += 1;
            }
        }
        count
    }

    fn record_cleanup(env: &Env, caller: &Address, category: u32, count: u32) {
        if count == 0 { return; }
        let entry = CleanupEntry {
            timestamp: env.ledger().timestamp(),
            caller: caller.clone(),
            category,
            count,
        };
        let mut log: Vec<CleanupEntry> = env.storage().persistent()
            .get(&DataKey::CleanupLog).unwrap_or(Vec::new(env));
        log.push_back(entry);
        env.storage().persistent().set(&DataKey::CleanupLog, &log);
    }

    fn get_config(env: &Env) -> RetentionConfig {
        env.storage().instance()
            .get(&DataKey::RetentionConfig)
            .unwrap_or(RetentionConfig {
                credential_secs: CREDENTIAL_RETENTION,
                audit_log_secs:  AUDIT_LOG_RETENTION,
                escrow_secs:     ESCROW_RETENTION,
                consent_secs:    CONSENT_RETENTION,
                schedule_secs:   SCHEDULE_RETENTION,
            })
    }

    fn require_initialized(env: &Env) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            Ok(())
        } else {
            Err(Error::NotInitialized)
        }
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env.storage().instance()
            .get(&DataKey::Admin).ok_or(Error::NotInitialized)?;
        if admin == *caller { Ok(()) } else { Err(Error::NotAuthorized) }
    }

    fn require_not_paused(env: &Env) -> Result<(), Error> {
        if env.storage().instance().get(&DataKey::Paused).unwrap_or(false) {
            Err(Error::Paused)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    fn setup() -> (Env, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, StorageCleanup);
        (env, admin, contract_id)
    }

    fn init(env: &Env, admin: &Address, contract_id: &Address) -> StorageCleanupClient {
        let client = StorageCleanupClient::new(env, contract_id);
        client.initialize(admin);
        client
    }

    // ── Initialization ────────────────────────────────────────────────────────

    #[test]
    fn test_initialize_sets_not_paused() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        assert!(!client.is_paused());
    }

    #[test]
    fn test_double_initialize_fails() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        assert_eq!(client.try_initialize(&admin), Err(Ok(Error::AlreadyInitialized)));
    }

    #[test]
    fn test_cleanup_before_init_fails() {
        let (env, admin, contract_id) = setup();
        let client = StorageCleanupClient::new(&env, &contract_id);
        assert_eq!(
            client.try_cleanup_expired(&admin, &10u32),
            Err(Ok(Error::NotInitialized))
        );
    }

    // ── Pause mechanism ───────────────────────────────────────────────────────

    #[test]
    fn test_pause_and_unpause() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        client.set_paused(&admin, &true);
        assert!(client.is_paused());
        client.set_paused(&admin, &false);
        assert!(!client.is_paused());
    }

    #[test]
    fn test_paused_blocks_all_cleanups() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        client.set_paused(&admin, &true);
        assert_eq!(client.try_cleanup_expired(&admin, &10u32),     Err(Ok(Error::Paused)));
        assert_eq!(client.try_cleanup_credentials(&admin, &10u32), Err(Ok(Error::Paused)));
        assert_eq!(client.try_cleanup_audit_logs(&admin, &10u32),  Err(Ok(Error::Paused)));
        assert_eq!(client.try_cleanup_escrows(&admin, &10u32),     Err(Ok(Error::Paused)));
        assert_eq!(client.try_cleanup_consents(&admin, &10u32),    Err(Ok(Error::Paused)));
        assert_eq!(client.try_cleanup_schedules(&admin, &10u32),   Err(Ok(Error::Paused)));
    }

    // ── Batch limits ──────────────────────────────────────────────────────────

    #[test]
    fn test_batch_too_large_rejected() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        assert_eq!(
            client.try_cleanup_expired(&admin, &(MAX_BATCH + 1)),
            Err(Ok(Error::BatchTooLarge))
        );
    }

    #[test]
    fn test_max_batch_exactly_accepted() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        assert!(client.try_cleanup_expired(&admin, &MAX_BATCH).is_ok());
    }

    #[test]
    fn test_preview_batch_too_large_rejected() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        assert_eq!(
            client.try_preview_cleanup(&(MAX_BATCH + 1)),
            Err(Ok(Error::BatchTooLarge))
        );
    }

    // ── Per-category cleanup ──────────────────────────────────────────────────

    #[test]
    fn test_cleanup_credentials_expired_removed_active_kept() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        client.register_credential(&1u64, &1u64); // expired (ts=1, far past)
        client.register_credential(&2u64, &0u64); // active  (ts=0 sentinel)
        client.register_credential(&3u64, &1u64); // expired
        assert_eq!(client.cleanup_credentials(&admin, &10u32), 2);
    }

    #[test]
    fn test_cleanup_audit_logs() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        client.register_audit_log(&10u64, &1u64);
        client.register_audit_log(&11u64, &0u64); // active
        assert_eq!(client.cleanup_audit_logs(&admin, &10u32), 1);
    }

    #[test]
    fn test_cleanup_escrows() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        client.register_escrow(&20u64, &1u64);
        client.register_escrow(&21u64, &0u64); // active
        assert_eq!(client.cleanup_escrows(&admin, &10u32), 1);
    }

    #[test]
    fn test_cleanup_consents() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        client.register_consent(&30u64, &1u64);
        client.register_consent(&31u64, &0u64); // active
        assert_eq!(client.cleanup_consents(&admin, &10u32), 1);
    }

    #[test]
    fn test_cleanup_schedules() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        client.register_schedule(&40u64, &1u64);
        client.register_schedule(&41u64, &0u64); // active
        assert_eq!(client.cleanup_schedules(&admin, &10u32), 1);
    }

    // ── cleanup_expired (all categories) ─────────────────────────────────────

    #[test]
    fn test_cleanup_expired_all_categories() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        client.register_credential(&1u64, &1u64);
        client.register_audit_log(&2u64, &1u64);
        client.register_escrow(&3u64, &1u64);
        client.register_consent(&4u64, &1u64);
        client.register_schedule(&5u64, &1u64);
        assert_eq!(client.cleanup_expired(&admin, &100u32), 5);
    }

    #[test]
    fn test_cleanup_expired_empty_returns_zero() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        assert_eq!(client.cleanup_expired(&admin, &100u32), 0);
    }

    // ── Preview (read-only) ───────────────────────────────────────────────────

    #[test]
    fn test_preview_does_not_mutate() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        client.register_credential(&1u64, &1u64);
        client.register_escrow(&2u64, &1u64);
        assert_eq!(client.preview_cleanup(&100u32), 2);
        // Still 2 after preview — nothing was removed
        assert_eq!(client.preview_cleanup(&100u32), 2);
    }

    // ── Cleanup audit log ─────────────────────────────────────────────────────

    #[test]
    fn test_cleanup_log_records_entry() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        client.register_escrow(&50u64, &1u64);
        client.cleanup_escrows(&admin, &10u32);
        let log = client.get_cleanup_log();
        assert_eq!(log.len(), 1);
        let entry = log.get(0).unwrap();
        assert_eq!(entry.count, 1);
        assert_eq!(entry.category, 2); // escrow = 2
    }

    #[test]
    fn test_cleanup_log_not_recorded_when_nothing_removed() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        client.cleanup_escrows(&admin, &10u32);
        assert_eq!(client.get_cleanup_log().len(), 0);
    }

    // ── Retention config ──────────────────────────────────────────────────────

    #[test]
    fn test_custom_retention_config_applied() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        let cfg = RetentionConfig {
            credential_secs: 1,
            audit_log_secs: 1,
            escrow_secs: 1,
            consent_secs: 1,
            schedule_secs: 1,
        };
        client.set_retention_config(&admin, &cfg);
        client.register_audit_log(&99u64, &1u64);
        assert_eq!(client.cleanup_audit_logs(&admin, &10u32), 1);
    }

    #[test]
    fn test_non_admin_cannot_set_retention_config() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        let other = Address::generate(&env);
        let cfg = RetentionConfig {
            credential_secs: 1,
            audit_log_secs: 1,
            escrow_secs: 1,
            consent_secs: 1,
            schedule_secs: 1,
        };
        assert_eq!(
            client.try_set_retention_config(&other, &cfg),
            Err(Ok(Error::NotAuthorized))
        );
    }

    // ── Data integrity ────────────────────────────────────────────────────────

    #[test]
    fn test_active_items_never_removed() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        // 10 items: even indices active (ts=0), odd indices expired (ts=1)
        for i in 0u64..10 {
            client.register_credential(&i, &(i % 2)); // 0 or 1
        }
        assert_eq!(client.cleanup_credentials(&admin, &100u32), 5);
        // No expired items remain
        assert_eq!(client.preview_cleanup(&100u32), 0);
    }

    #[test]
    fn test_batch_limit_caps_removal() {
        let (env, admin, contract_id) = setup();
        let client = init(&env, &admin, &contract_id);
        for i in 0u64..20 {
            client.register_credential(&i, &1u64);
        }
        // Cap at 5
        assert_eq!(client.cleanup_credentials(&admin, &5u32), 5);
        // 15 remain
        assert_eq!(client.preview_cleanup(&100u32), 15);
    }
}
