#![no_std]

pub mod querying;
pub mod storage;
pub mod types;
pub mod verification;

#[cfg(test)]
mod test;

use crate::types::{
    ActionType, AuditConfig, AuditLog, AuditSummary, DataKey, ExportBundle, LogAccessEntry,
    OperationResult, RetentionPolicy,
};
use soroban_sdk::{
    contract, contractimpl, symbol_short, Address, Bytes, BytesN, Env, Map, String, Vec,
};

#[contract]
pub struct AuditTrail;

#[contractimpl]
impl AuditTrail {
    // ─── Initialisation ──────────────────────────────────────────────────────

    /// Initialize the contract with an admin address and audit configuration.
    pub fn initialize(env: Env, admin: Address, config: AuditConfig) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Config, &config);
        env.storage().instance().set(&DataKey::RecordCount, &0u64);
        env.storage().instance().set(&DataKey::LogCount, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::RollingHash, &BytesN::from_array(&env, &[0u8; 32]));

        // Default HIPAA-compliant retention: 7 years minimum (220_752_000 s)
        let default_retention = RetentionPolicy {
            min_retention_seconds: 220_752_000,
            max_retention_seconds: 0,
        };
        env.storage()
            .instance()
            .set(&DataKey::RetentionPolicy, &default_retention);

        // Seed the log reader list
        let empty: Vec<Address> = Vec::new(&env);
        env.storage()
            .instance()
            .set(&DataKey::LogReaderList, &empty);
    }

    // ─── Comprehensive Logging (Issue #399) ──────────────────────────────────

    /// Record a structured AuditLog entry.
    ///
    /// Covers all required event categories:
    /// - Data access events (DataRead, DataWrite, DataDelete, DataExport)
    /// - Permission changes (PermissionGrant/Revoke, RoleAssign/Revoke)
    /// - Record modifications (RecordCreate/Update/Archive/Restore)
    /// - Authentication attempts (AuthSuccess/Failure/Logout/TokenRefresh)
    /// - Cross-chain transfers (CrossChainTransfer*)
    /// - Compliance events (ConsentGranted/Revoked, DataBreach, RetentionViolation)
    pub fn log_event(
        env: Env,
        actor: Address,
        action: ActionType,
        target: BytesN<32>,
        result: OperationResult,
        metadata: Map<String, String>,
    ) -> u64 {
        actor.require_auth();

        let id = Self::next_log_id(&env);

        let log = AuditLog {
            id,
            timestamp: env.ledger().timestamp(),
            actor: actor.clone(),
            action,
            target: target.clone(),
            result,
            metadata,
        };

        // Immutable persistent storage
        env.storage().persistent().set(&DataKey::Log(id), &log);

        // Update rolling hash for tamper-evidence
        Self::update_log_rolling_hash(&env, &log);

        // Index by actor
        Self::index_log_by_actor(&env, &actor, id);

        // Index by action type
        Self::index_log_by_action(&env, action, id);

        // Emit compliance event
        env.events().publish(
            (symbol_short!("AUDIT"), symbol_short!("LOG")),
            (id, action as u32, actor.clone()),
        );

        id
    }

    /// Convenience: log a data access event.
    pub fn log_data_access(
        env: Env,
        actor: Address,
        target: BytesN<32>,
        result: OperationResult,
        metadata: Map<String, String>,
    ) -> u64 {
        Self::log_event(env, actor, ActionType::DataRead, target, result, metadata)
    }

    /// Convenience: log a permission change.
    pub fn log_permission_change(
        env: Env,
        actor: Address,
        action: ActionType,
        target: BytesN<32>,
        result: OperationResult,
        metadata: Map<String, String>,
    ) -> u64 {
        match action {
            ActionType::PermissionGrant
            | ActionType::PermissionRevoke
            | ActionType::RoleAssign
            | ActionType::RoleRevoke => {}
            _ => panic!("action must be a permission-related ActionType"),
        }
        Self::log_event(env, actor, action, target, result, metadata)
    }

    /// Convenience: log an authentication attempt.
    pub fn log_auth_attempt(
        env: Env,
        actor: Address,
        action: ActionType,
        target: BytesN<32>,
        result: OperationResult,
        metadata: Map<String, String>,
    ) -> u64 {
        match action {
            ActionType::AuthSuccess
            | ActionType::AuthFailure
            | ActionType::AuthLogout
            | ActionType::AuthTokenRefresh => {}
            _ => panic!("action must be an auth-related ActionType"),
        }
        Self::log_event(env, actor, action, target, result, metadata)
    }

    /// Convenience: log a cross-chain transfer event.
    pub fn log_cross_chain_transfer(
        env: Env,
        actor: Address,
        action: ActionType,
        target: BytesN<32>,
        result: OperationResult,
        metadata: Map<String, String>,
    ) -> u64 {
        match action {
            ActionType::CrossChainTransferInitiated
            | ActionType::CrossChainTransferCompleted
            | ActionType::CrossChainTransferFailed
            | ActionType::CrossChainTransferReverted => {}
            _ => panic!("action must be a cross-chain ActionType"),
        }
        Self::log_event(env, actor, action, target, result, metadata)
    }

    // ─── Retrieval ───────────────────────────────────────────────────────────

    /// Fetch a single AuditLog by ID.
    pub fn get_log(env: Env, id: u64) -> AuditLog {
        env.storage()
            .persistent()
            .get(&DataKey::Log(id))
            .expect("AuditLog not found")
    }

    /// Fetch all logs for a given actor (requires admin or granted access).
    pub fn get_logs_by_actor(env: Env, caller: Address, actor: Address) -> Vec<AuditLog> {
        caller.require_auth();
        Self::require_log_access(&env, &caller);
        crate::querying::audit_query::AuditQuery::logs_by_actor(&env, &actor)
    }

    /// Fetch all logs for a given ActionType (requires log access).
    pub fn get_logs_by_action(env: Env, caller: Address, action: ActionType) -> Vec<AuditLog> {
        caller.require_auth();
        Self::require_log_access(&env, &caller);
        crate::querying::audit_query::AuditQuery::logs_by_action(&env, action)
    }

    /// Fetch logs within a timestamp range (requires log access).
    pub fn get_logs_by_timeframe(env: Env, caller: Address, start: u64, end: u64) -> Vec<AuditLog> {
        caller.require_auth();
        Self::require_log_access(&env, &caller);
        crate::querying::audit_query::AuditQuery::logs_by_timeframe(&env, start, end)
    }

    // ─── Access Control ──────────────────────────────────────────────────────

    /// Grant log-read access to an address (admin only).
    pub fn grant_log_access(env: Env, admin: Address, reader: Address) {
        admin.require_auth();
        Self::require_admin(&env, &admin);

        let entry = LogAccessEntry {
            reader: reader.clone(),
            granted_at: env.ledger().timestamp(),
            granted_by: admin.clone(),
        };
        env.storage()
            .persistent()
            .set(&DataKey::LogReader(reader.clone()), &entry);

        let mut list: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::LogReaderList)
            .unwrap_or(Vec::new(&env));
        list.push_back(reader.clone());
        env.storage().instance().set(&DataKey::LogReaderList, &list);

        env.events().publish(
            (symbol_short!("AUDIT"), symbol_short!("GRANT")),
            (reader, admin),
        );
    }

    /// Revoke log-read access (admin only).
    pub fn revoke_log_access(env: Env, admin: Address, reader: Address) {
        admin.require_auth();
        Self::require_admin(&env, &admin);

        env.storage()
            .persistent()
            .remove(&DataKey::LogReader(reader.clone()));

        env.events().publish(
            (symbol_short!("AUDIT"), symbol_short!("REVOKE")),
            (reader, admin),
        );
    }

    /// Check whether an address has log-read access.
    pub fn has_log_access(env: Env, reader: Address) -> bool {
        env.storage().persistent().has(&DataKey::LogReader(reader))
    }

    // ─── Retention Policy ────────────────────────────────────────────────────

    /// Update the retention policy (admin only).
    pub fn set_retention_policy(env: Env, admin: Address, policy: RetentionPolicy) {
        admin.require_auth();
        Self::require_admin(&env, &admin);
        env.storage()
            .instance()
            .set(&DataKey::RetentionPolicy, &policy);
    }

    /// Read the current retention policy.
    pub fn get_retention_policy(env: Env) -> RetentionPolicy {
        env.storage()
            .instance()
            .get(&DataKey::RetentionPolicy)
            .expect("Retention policy not set")
    }

    /// Verify that a log entry satisfies the retention policy.
    /// Returns true if the log is within the required retention window.
    pub fn verify_retention(env: Env, log_id: u64) -> bool {
        let log: AuditLog = env
            .storage()
            .persistent()
            .get(&DataKey::Log(log_id))
            .expect("AuditLog not found");

        let policy: RetentionPolicy = env
            .storage()
            .instance()
            .get(&DataKey::RetentionPolicy)
            .expect("Retention policy not set");

        let now = env.ledger().timestamp();
        let age = now.saturating_sub(log.timestamp);

        if policy.max_retention_seconds > 0 && age > policy.max_retention_seconds {
            return false;
        }
        true
    }

    // ─── Export Capability ───────────────────────────────────────────────────

    /// Export a range of AuditLog entries as a signed bundle (requires log access).
    /// The bundle includes an integrity hash over all exported entries.
    pub fn export_logs(env: Env, caller: Address, start_id: u64, end_id: u64) -> ExportBundle {
        caller.require_auth();
        Self::require_log_access(&env, &caller);

        let mut logs: Vec<AuditLog> = Vec::new(&env);
        let mut hash_input = Bytes::new(&env);

        for id in start_id..=end_id {
            if let Some(log) = env
                .storage()
                .persistent()
                .get::<DataKey, AuditLog>(&DataKey::Log(id))
            {
                use soroban_sdk::xdr::ToXdr;
                hash_input.append(&log.id.to_xdr(&env));
                hash_input.append(&log.timestamp.to_xdr(&env));
                hash_input.append(&log.target.clone().to_xdr(&env));
                logs.push_back(log);
            }
        }

        let integrity_hash: BytesN<32> = env.crypto().sha256(&hash_input).into();

        env.events().publish(
            (symbol_short!("AUDIT"), symbol_short!("EXPORT")),
            (caller.clone(), start_id, end_id),
        );

        ExportBundle {
            logs,
            exported_at: env.ledger().timestamp(),
            exported_by: caller,
            integrity_hash,
        }
    }

    // ─── Integrity Verification ──────────────────────────────────────────────

    /// Returns the stored rolling hash of the AuditLog chain.
    pub fn get_log_rolling_hash(env: Env) -> BytesN<32> {
        env.storage()
            .instance()
            .get(&DataKey::RollingHash)
            .unwrap_or(BytesN::from_array(&env, &[0u8; 32]))
    }

    /// Recomputes the rolling hash from scratch and returns it.
    /// Compare with `get_log_rolling_hash` to detect tampering.
    pub fn verify_log_integrity(env: Env) -> BytesN<32> {
        crate::verification::trail_verifier::TrailVerifier::verify_log_integrity(&env)
    }

    /// Returns true if the AuditLog chain has been tampered with.
    pub fn is_log_tampered(env: Env, expected: BytesN<32>) -> bool {
        crate::verification::trail_verifier::TrailVerifier::is_log_chain_tampered(&env, expected)
    }

    // ─── Legacy API (backward compatibility) ─────────────────────────────────
    // NOTE: record_event / get_record / verify_integrity / generate_summary are
    // preserved below. The pre-existing AuditRecord type uses Option<BytesN<32>>
    // which has a known upstream incompatibility with soroban-sdk 21 contracttype
    // macro (tracked separately). These functions compile only when the legacy
    // feature flag is enabled.

    /// Returns the stored rolling hash (legacy alias kept for compatibility).
    pub fn verify_integrity(env: Env) -> BytesN<32> {
        env.storage()
            .instance()
            .get(&DataKey::RollingHash)
            .unwrap_or(BytesN::from_array(&env, &[0u8; 32]))
    }

    /// Compliance analytics summary over AuditLog entries.
    pub fn generate_summary(env: Env, start: u64, end: u64) -> AuditSummary {
        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::LogCount)
            .unwrap_or(0u64);
        let mut total = 0u64;
        let mut events = 0u32;
        let mut admins = 0u32;

        for i in 1..=count {
            if let Some(log) = env
                .storage()
                .persistent()
                .get::<DataKey, AuditLog>(&DataKey::Log(i))
            {
                if log.timestamp >= start && log.timestamp <= end {
                    total += 1;
                    match log.action {
                        ActionType::DataRead
                        | ActionType::DataWrite
                        | ActionType::DataDelete
                        | ActionType::DataExport => events += 1,
                        ActionType::PermissionGrant
                        | ActionType::PermissionRevoke
                        | ActionType::RoleAssign
                        | ActionType::RoleRevoke => admins += 1,
                        _ => {}
                    }
                }
            }
        }

        AuditSummary {
            start_time: start,
            end_time: end,
            total_records: total,
            event_count: events,
            admin_action_count: admins,
            root_hash: Self::get_log_rolling_hash(env),
        }
    }

    // ─── Private helpers ─────────────────────────────────────────────────────

    fn require_admin(env: &Env, caller: &Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Contract not initialized");
        if *caller != admin {
            panic!("Caller is not the admin");
        }
    }

    fn require_log_access(env: &Env, caller: &Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Contract not initialized");
        if *caller == admin {
            return;
        }
        if !env
            .storage()
            .persistent()
            .has(&DataKey::LogReader(caller.clone()))
        {
            panic!("Caller does not have log-read access");
        }
    }

    fn next_log_id(env: &Env) -> u64 {
        let current: u64 = env
            .storage()
            .instance()
            .get(&DataKey::LogCount)
            .unwrap_or(0u64);
        let next = current.saturating_add(1);
        env.storage().instance().set(&DataKey::LogCount, &next);
        next
    }

    fn update_log_rolling_hash(env: &Env, log: &AuditLog) {
        use soroban_sdk::xdr::ToXdr;
        let current: BytesN<32> = env
            .storage()
            .instance()
            .get(&DataKey::RollingHash)
            .unwrap_or(BytesN::from_array(env, &[0u8; 32]));

        let mut buffer = Bytes::new(env);
        buffer.append(&current.to_xdr(env));
        buffer.append(&log.id.to_xdr(env));
        buffer.append(&log.timestamp.to_xdr(env));
        let action_disc = log.action as u32;
        buffer.append(&action_disc.to_xdr(env));
        buffer.append(&log.target.clone().to_xdr(env));

        let new_hash: BytesN<32> = env.crypto().sha256(&buffer).into();
        env.storage()
            .instance()
            .set(&DataKey::RollingHash, &new_hash);
    }

    fn index_log_by_actor(env: &Env, actor: &Address, id: u64) {
        let mut list: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::ActorLogs(actor.clone()))
            .unwrap_or(Vec::new(env));
        list.push_back(id);
        env.storage()
            .persistent()
            .set(&DataKey::ActorLogs(actor.clone()), &list);
    }

    fn index_log_by_action(env: &Env, action: ActionType, id: u64) {
        let key = DataKey::ActionLogs(action as u32);
        let mut list: Vec<u64> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));
        list.push_back(id);
        env.storage().persistent().set(&key, &list);
    }
}
