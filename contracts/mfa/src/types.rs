use soroban_sdk::{contracttype, Address, Bytes, String, Vec};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum FactorType {
    Password = 0,
    Biometric = 1,
    HardwareKey = 2,
    EmailCode = 3,
    SMSCode = 4,
    AuthenticatorApp = 5, // TOTP
    MultiSig = 6,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum AuthStatus {
    Pending = 0,
    Partial = 1,
    Verified = 2,
    Expired = 3,
    Revoked = 4,
}

#[derive(Clone)]
#[contracttype]
pub struct AuthFactor {
    pub factor_id: u64,
    pub user: Address,
    pub factor_type: FactorType,
    pub provider_address: Option<Address>, // External contract for verification
    pub metadata: String,                  // Public identifier (e.g., "YubiKey 5")
    pub created_at: u64,
    pub is_active: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct AuthSession {
    pub session_id: u64,
    pub user: Address,
    pub required_factors: Vec<FactorType>,
    pub verified_factors: Vec<FactorType>,
    pub expires_at: u64,
    pub status: AuthStatus,
}

#[derive(Clone)]
#[contracttype]
pub struct RecoveryVault {
    pub user: Address,
    pub recovery_hashes: Vec<soroban_sdk::BytesN<32>>, // Salted hashes of recovery codes
    pub backup_address: Option<Address>,
    pub unlock_at: u64, // Time-lock for recovery
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    UserFactors(Address),
    UserSession(Address),
    Recovery(Address),
    NextFactorId,
    NextSessionId,
    GlobalConfig,
    AuditLogCount,
    AuditEntry(u64),
}

#[derive(Clone)]
#[contracttype]
pub struct MFAConfig {
    pub session_ttl: u64,
    pub min_factors_for_critical_op: u32,
    pub recovery_delay: u64,
}
