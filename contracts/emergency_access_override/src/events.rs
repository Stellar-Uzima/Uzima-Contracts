use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol, Vec};

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum AuditAction {
    Initialized,
    GrantGranted,
    GrantApproved,
    GrantRevoked,
    DuplicateApproval,
    AccessCheck,
    RateLimited,
    CooldownUpdated,
    CircuitReset,
    CircuitTripped,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct AuditLogEntry {
    pub action: Symbol,
    pub actor: Address,
    pub patient: Option<Address>,
    pub provider: Option<Address>,
    pub details: Symbol,
    pub timestamp: u64,
    pub block: u32,
}

pub fn record_audit_entry(
    env: &Env,
    action: AuditAction,
    actor: &Address,
    patient: Option<Address>,
    provider: Option<Address>,
    details: Symbol,
) -> u64 {
    let counter_key = Symbol::new(env, "AuditLogCounter");
    let counter: u64 = env.storage().persistent().get(&counter_key).unwrap_or(0);
    let entry_id = counter + 1;

    let entry = AuditLogEntry {
        action: match action {
            AuditAction::Initialized => symbol_short!("INIT"),
            AuditAction::GrantGranted => symbol_short!("GRANT"),
            AuditAction::GrantApproved => symbol_short!("APPR"),
            AuditAction::GrantRevoked => symbol_short!("REVOKE"),
            AuditAction::DuplicateApproval => symbol_short!("DUPA"),
            AuditAction::AccessCheck => symbol_short!("CHECK"),
            AuditAction::RateLimited => symbol_short!("RATELMT"),
            AuditAction::CooldownUpdated => symbol_short!("CDUPD"),
            AuditAction::CircuitReset => symbol_short!("CBRST"),
            AuditAction::CircuitTripped => symbol_short!("CBTRIP"),
        },
        actor: actor.clone(),
        patient,
        provider,
        details,
        timestamp: env.ledger().timestamp(),
        block: env.ledger().sequence(),
    };

    let entry_key = (counter_key.clone(), entry_id);
    env.storage().persistent().set(&entry_key, &entry);
    env.storage().persistent().set(&counter_key, &entry_id);

    env.events().publish(
        (Symbol::new(env, "EMER"), Symbol::new(env, "AUDIT")),
        (entry_id, entry.action, actor.clone()),
    );

    entry_id
}

pub fn query_audit_logs(
    env: &Env,
    from_id: u64,
    to_id: u64,
    max_results: u32,
) -> Vec<AuditLogEntry> {
    let mut results = Vec::new(env);
    let counter_key = Symbol::new(env, "AuditLogCounter");
    let max: u64 = env.storage().persistent().get(&counter_key).unwrap_or(0);
    let start = core::cmp::max(from_id, 1);
    let end = core::cmp::min(to_id, max);
    let mut count = 0u32;

    let mut i = start;
    while i <= end && count < max_results {
        let entry_key = (counter_key.clone(), i);
        if let Some(entry) = env.storage().persistent().get::<_, AuditLogEntry>(&entry_key) {
            results.push_back(entry);
            count += 1;
        }
        i += 1;
    }

    results
}

pub fn publish_emergency_access_granted(
    env: &Env,
    patient: &Address,
    provider: &Address,
    expiry_at: u64,
    granted_at: u64,
) {
    env.events().publish(
        (symbol_short!("EMER"), symbol_short!("GRANT")),
        (patient, provider, expiry_at, granted_at),
    );
}

pub fn publish_emergency_access_approved(
    env: &Env,
    patient: &Address,
    provider: &Address,
    approver: &Address,
    approved_at: u64,
) {
    env.events().publish(
        (symbol_short!("EMER"), symbol_short!("APPR")),
        (patient, provider, approver, approved_at),
    );
}

pub fn publish_duplicate_approval(
    env: &Env,
    patient: &Address,
    provider: &Address,
    approver: &Address,
    timestamp: u64,
) {
    env.events().publish(
        (symbol_short!("EMER"), symbol_short!("DUPA")),
        (patient, provider, approver, timestamp),
    );
}

pub fn publish_emergency_access_checked(
    env: &Env,
    patient: &Address,
    provider: &Address,
    has_access: bool,
    checked_at: u64,
) {
    env.events().publish(
        (symbol_short!("EMER"), symbol_short!("CHECK")),
        (patient, provider, has_access, checked_at),
    );
}

pub fn publish_emergency_access_revoked(
    env: &Env,
    patient: &Address,
    provider: &Address,
    revoked_at: u64,
) {
    env.events().publish(
        (symbol_short!("EMER"), symbol_short!("REVOKE")),
        (patient, provider, revoked_at),
    );
}

pub fn publish_initialization(env: &Env, admin: &Address) {
    env.events()
        .publish((symbol_short!("EMER"), symbol_short!("INIT")), admin);
}

pub fn publish_rate_limit_exceeded(
    env: &Env,
    caller: &Address,
    next_allowed_at: u64,
    attempted_at: u64,
) {
    env.events().publish(
        (symbol_short!("EMER"), symbol_short!("RATELMT")),
        (caller, next_allowed_at, attempted_at),
    );
}

pub fn publish_cooldown_updated(env: &Env, admin: &Address, new_period: u64) {
    env.events().publish(
        (symbol_short!("EMER"), symbol_short!("CDUPD")),
        (admin, new_period),
    );
}
