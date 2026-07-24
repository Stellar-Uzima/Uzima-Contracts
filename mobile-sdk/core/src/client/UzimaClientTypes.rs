//! Standardized Uzima client with ergonomic interface for Rust SDK.
//!
//! Provides a builder pattern for initialization and consistent
//! error handling across all operations.

use soroban_sdk::{Address, Env};

/// SDK configuration with builder pattern.
pub struct UzimaConfig {
    pub api_endpoint: String,
    pub contract_id: String,
    pub network_passphrase: String,
    pub server_url: String,
    pub encryption_key: Option<String>,
    pub offline_enabled: bool,
    pub notifications_enabled: bool,
    pub biometric_enabled: bool,
    pub request_timeout: u32,
    pub cache_enabled: bool,
    pub cache_ttl: u32,
}

impl UzimaConfig {
    pub fn builder() -> UzimaConfigBuilder {
        UzimaConfigBuilder::default()
    }
}

/// Builder for UzimaConfig.
pub struct UzimaConfigBuilder {
    api_endpoint: Option<String>,
    contract_id: Option<String>,
    network_passphrase: Option<String>,
    server_url: Option<String>,
    encryption_key: Option<String>,
    offline_enabled: bool,
    notifications_enabled: bool,
    biometric_enabled: bool,
    request_timeout: u32,
    cache_enabled: bool,
    cache_ttl: u32,
}

impl Default for UzimaConfigBuilder {
    fn default() -> Self {
        Self {
            api_endpoint: None,
            contract_id: None,
            network_passphrase: None,
            server_url: None,
            encryption_key: None,
            offline_enabled: false,
            notifications_enabled: false,
            biometric_enabled: false,
            request_timeout: 30_000,
            cache_enabled: true,
            cache_ttl: 300_000,
        }
    }
}

impl UzimaConfigBuilder {
    pub fn api_endpoint(mut self, val: &str) -> Self {
        self.api_endpoint = Some(val.to_string());
        self
    }

    pub fn contract_id(mut self, val: &str) -> Self {
        self.contract_id = Some(val.to_string());
        self
    }

    pub fn network_passphrase(mut self, val: &str) -> Self {
        self.network_passphrase = Some(val.to_string());
        self
    }

    pub fn server_url(mut self, val: &str) -> Self {
        self.server_url = Some(val.to_string());
        self
    }

    pub fn encryption_key(mut self, val: &str) -> Self {
        self.encryption_key = Some(val.to_string());
        self
    }

    pub fn offline_enabled(mut self, val: bool) -> Self {
        self.offline_enabled = val;
        self
    }

    pub fn notifications_enabled(mut self, val: bool) -> Self {
        self.notifications_enabled = val;
        self
    }

    pub fn biometric_enabled(mut self, val: bool) -> Self {
        self.biometric_enabled = val;
        self
    }

    pub fn request_timeout(mut self, val: u32) -> Self {
        self.request_timeout = val;
        self
    }

    pub fn cache_enabled(mut self, val: bool) -> Self {
        self.cache_enabled = val;
        self
    }

    pub fn cache_ttl(mut self, val: u32) -> Self {
        self.cache_ttl = val;
        self
    }

    pub fn build(self) -> Result<UzimaConfig, UzimaError> {
        Ok(UzimaConfig {
            api_endpoint: self.api_endpoint.ok_or(UzimaError::MissingConfig("api_endpoint"))?,
            contract_id: self.contract_id.ok_or(UzimaError::MissingConfig("contract_id"))?,
            network_passphrase: self.network_passphrase.ok_or(UzimaError::MissingConfig("network_passphrase"))?,
            server_url: self.server_url.ok_or(UzimaError::MissingConfig("server_url"))?,
            encryption_key: self.encryption_key,
            offline_enabled: self.offline_enabled,
            notifications_enabled: self.notifications_enabled,
            biometric_enabled: self.biometric_enabled,
            request_timeout: self.request_timeout,
            cache_enabled: self.cache_enabled,
            cache_ttl: self.cache_ttl,
        })
    }
}

/// SDK error types.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UzimaError {
    MissingConfig(&'static str),
    NotInitialized,
    AuthenticationFailed,
    NetworkError,
    ContractError(u32),
    SerializationError,
    EncryptionError,
}

impl core::fmt::Display for UzimaError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::MissingConfig(field) => write!(f, "missing required config: {}", field),
            Self::NotInitialized => write!(f, "SDK not initialized"),
            Self::AuthenticationFailed => write!(f, "authentication failed"),
            Self::NetworkError => write!(f, "network error"),
            Self::ContractError(code) => write!(f, "contract error: {}", code),
            Self::SerializationError => write!(f, "serialization error"),
            Self::EncryptionError => write!(f, "encryption error"),
        }
    }
}

/// SDK status.
pub struct SDKStatus {
    pub ready: bool,
    pub authenticated: bool,
    pub online: bool,
    pub offline_queue_size: u32,
    pub cache_size: u32,
}

/// Main Uzima client.
pub struct UzimaClient {
    config: UzimaConfig,
    initialized: bool,
    public_key: Option<String>,
}

impl UzimaClient {
    /// Create a new client from config.
    pub fn new(config: UzimaConfig) -> Self {
        Self {
            config,
            initialized: false,
            public_key: None,
        }
    }

    /// Initialize SDK with authentication.
    pub fn initialize(&mut self, public_key: &str, _secret_key: Option<&str>) {
        self.public_key = Some(public_key.to_string());
        self.initialized = true;
    }

    /// Check if SDK is ready.
    pub fn is_ready(&self) -> bool {
        self.initialized
    }

    /// Get SDK status.
    pub fn get_status(&self) -> SDKStatus {
        SDKStatus {
            ready: self.initialized,
            authenticated: self.public_key.is_some(),
            online: true,
            offline_queue_size: 0,
            cache_size: 0,
        }
    }

    /// Logout and clear session.
    pub fn logout(&mut self) {
        self.initialized = false;
        self.public_key = None;
    }

    /// Get config reference.
    pub fn config(&self) -> &UzimaConfig {
        &self.config
    }
}