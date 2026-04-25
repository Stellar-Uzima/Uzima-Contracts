// Medical Consent NFT - Advanced Patient consent management with dynamic features
#![no_std]
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Map, String,
    Vec,
};

// Storage keys

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Issuers,
    TokenCounter,
    TokenOwner(u64),
    TokenMetadata(u64),
    TokenRevoked(u64),
    OwnerTokens(Address),
    ConsentHistory(u64),
    PatientConsents(Address), // Track tokens issued for a patient (for revoke access)
    // Advanced features storage keys
    GranularPermissions(u64), // Granular permissions per token
    AccessControls(u64),      // Time-based and condition-based access controls
    ConsentDelegations(u64),  // Delegation mappings
    ConsentInheritance(u64),  // Parent-child consent relationships
    EmergencyOverrides(u64),  // Emergency override records
    MarketplaceListings(u64), // Research marketplace listings
    VersionHistory(u64),      // Full version history for dynamic updates
    AnalyticsData,            // Aggregated analytics data
    EmergencyAuthorities,     // Authorized emergency override addresses
    MarketplaceEnabled,       // Marketplace feature flag
}

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractError {
    NotAuthorized = 1,
    TokenNotFound = 2,
    ConsentRevoked = 3,
    AlreadyInitialized = 4,
    NotTokenOwner = 5,
    InvalidPermission = 6,
    AccessDenied = 7,
    InvalidDelegation = 8,
    EmergencyOverrideFailed = 9,
    MarketplaceNotEnabled = 10,
    InvalidCondition = 11,
    InheritanceCycle = 12,
}

// Data type enum for granular permissions
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataType {
    Demographics,
    MedicalHistory,
    LabResults,
    Imaging,
    Medications,
    Procedures,
    Allergies,
    Research,
    Financial,
}

// Permission level enum
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PermissionLevel {
    None,  // No access
    Read,  // Read-only access
    Write, // Read and write access
    Full,  // Full access including deletion
}

// Granular permissions structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GranularPermissions {
    pub permissions: Map<DataType, PermissionLevel>, // Data type -> permission level mapping
}

// Access condition types - using tuple variants for Soroban compatibility
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AccessCondition {
    TimeWindow(u64, u64),       // (start, end) - Time-based access window
    DayOfWeek(Vec<u32>),        // Specific days of week (0-6)
    TimeOfDay(u32, u32),        // (start_hour, end_hour) - Time of day restrictions
    LocationBased(Vec<String>), // Location-based access
    PurposeBased(Vec<String>),  // Purpose-based restrictions
    EmergencyOnly,              // Emergency access only
}

// Access control structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccessControl {
    pub conditions: Vec<AccessCondition>, // Multiple conditions (AND logic)
    pub max_access_count: u32,            // Maximum number of accesses (0 = unlimited)
    pub current_access_count: u32,        // Current access count
    pub last_access_timestamp: u64,       // Last access time
}

// Delegation structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Delegation {
    pub delegate: Address,                // Who is delegated to
    pub permissions: GranularPermissions, // What permissions are delegated
    pub expiry_timestamp: u64,            // When delegation expires
    pub created_timestamp: u64,           // When delegation was created
}

// Consent inheritance structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Inheritance {
    pub parent_token_id: u64,                       // Parent consent token ID
    pub inherited_permissions: GranularPermissions, // Inherited permissions
}

// Emergency override record
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmergencyOverride {
    pub override_id: u64,       // Unique override ID
    pub authorized_by: Address, // Who authorized the override
    pub reason: String,         // Reason for override
    pub timestamp: u64,         // When override occurred
    pub duration: u64,          // How long override is valid (0 = single use)
    pub used: bool,             // Whether override has been used
}

// Marketplace listing structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarketplaceListing {
    pub token_id: u64,
    pub price: i128,               // Price in tokens (using i128 for Soroban)
    pub data_types: Vec<DataType>, // Which data types are included
    pub research_purpose: String,  // Purpose of research
    pub duration: u64,             // How long access is granted
    pub listed_by: Address,        // Who listed it
    pub listed_timestamp: u64,     // When it was listed
    pub active: bool,              // Whether listing is active
}

// Version history entry
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionHistoryEntry {
    pub version: u32,
    pub metadata_uri: String,
    pub updated_by: Address,
    pub timestamp: u64,
    pub change_summary: String, // Summary of changes
}

// Consent metadata structure - Enhanced with advanced features
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsentMetadata {
    pub metadata_uri: String,          // IPFS hash or secure storage pointer
    pub consent_type: String,          // Type of consent (treatment, research, etc.)
    pub issued_timestamp: u64,         // When consent was issued
    pub expiry_timestamp: u64,         // When consent expires (0 = no expiry)
    pub issuer: Address,               // Who issued the consent
    pub patient: Address,              // The patient this consent is for
    pub version: u32,                  // Current metadata version for updates
    pub dynamic_updates_enabled: bool, // Whether dynamic updates are allowed
}

// Consent history entry for audit trail - Enhanced
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsentHistoryEntry {
    pub action: String, // "issued", "updated", "revoked", "delegated", "inherited", "emergency_override", "marketplace_listed", etc.
    pub timestamp: u64,
    pub actor: Address,
    pub metadata_uri: String,
    pub details: String, // Additional details about the action
}

// Analytics data structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalyticsData {
    pub total_consents: u64,
    pub active_consents: u64,
    pub revoked_consents: u64,
    pub total_delegations: u64,
    pub total_emergency_overrides: u64,
    pub marketplace_listings: u64,
    pub total_access_count: u64,
}

#[contract]
pub struct PatientConsentToken;

#[contractimpl]
impl PatientConsentToken {
    /// Initialize the contract with an admin
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(ContractError::AlreadyInitialized);
        }

        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TokenCounter, &0u64);

        // Initialize empty issuers list
        let issuers: Vec<Address> = Vec::new(&env);
        env.storage().instance().set(&DataKey::Issuers, &issuers);

        // Initialize analytics
        let analytics = AnalyticsData {
            total_consents: 0,
            active_consents: 0,
            revoked_consents: 0,
            total_delegations: 0,
            total_emergency_overrides: 0,
            marketplace_listings: 0,
            total_access_count: 0,
        };
        env.storage()
            .instance()
            .set(&DataKey::AnalyticsData, &analytics);

        // Initialize emergency authorities
        let authorities: Vec<Address> = Vec::new(&env);
        env.storage()
            .instance()
            .set(&DataKey::EmergencyAuthorities, &authorities);

        // Marketplace disabled by default
        env.storage()
            .instance()
            .set(&DataKey::MarketplaceEnabled, &false);

        Ok(())
    }

    /// Add an authorized issuer (clinic/healthcare provider)
    pub fn add_issuer(env: Env, issuer: Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Not initialized");
        admin.require_auth();

        let mut issuers: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Issuers)
            .unwrap_or(Vec::new(&env));

        issuers.push_back(issuer);
        env.storage().instance().set(&DataKey::Issuers, &issuers);
    }

    /// Remove an authorized issuer
    pub fn remove_issuer(env: Env, issuer: Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Not initialized");
        admin.require_auth();

        let issuers: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Issuers)
            .expect("No issuers found");

        let mut new_issuers = Vec::new(&env);
        for i in 0..issuers.len() {
            if let Some(current) = issuers.get(i) {
                if current != issuer {
                    new_issuers.push_back(current);
                }
            }
        }

        env.storage()
            .instance()
            .set(&DataKey::Issuers, &new_issuers);
    }

    /// Check if address is an authorized issuer
    pub fn is_issuer(env: Env, address: Address) -> bool {
        let issuers: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Issuers)
            .unwrap_or(Vec::new(&env));

        for i in 0..issuers.len() {
            if let Some(issuer) = issuers.get(i) {
                if issuer == address {
                    return true;
                }
            }
        }
        false
    }

    /// Mint a new consent token - FIXED: Add issuer: Address param, require_auth on it, use for check & metadata (no env.invoker())
    pub fn mint_consent(
        env: Env,
        issuer: Address, // FIXED: Passed by caller (must be their own Address::AccountId)
        patient: Address, // Renamed from 'to' for clarity
        metadata_uri: String,
        consent_type: String,
        expiry_timestamp: u64,
    ) -> Result<u64, ContractError> {
        // FIXED: Verify caller is authorized issuer via passed address + auth
        issuer.require_auth();
        if !Self::is_issuer(env.clone(), issuer.clone()) {
            return Err(ContractError::NotAuthorized);
        }

        // Get and increment token counter
        let token_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TokenCounter)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::TokenCounter, &(token_id + 1));

        // Create consent metadata
        let metadata = ConsentMetadata {
            metadata_uri: metadata_uri.clone(),
            consent_type: consent_type.clone(),
            issued_timestamp: env.ledger().timestamp(),
            expiry_timestamp,
            issuer: issuer.clone(),
            patient: patient.clone(),
            version: 1,
            dynamic_updates_enabled: false, // Default to false, can be enabled later
        };

        // Store token data
        env.storage()
            .instance()
            .set(&DataKey::TokenOwner(token_id), &patient);
        env.storage()
            .instance()
            .set(&DataKey::TokenMetadata(token_id), &metadata);
        env.storage()
            .instance()
            .set(&DataKey::TokenRevoked(token_id), &false);

        // Add to patient's token list (initial owner)
        let owner_key = DataKey::OwnerTokens(patient.clone());
        let mut owner_tokens: Vec<u64> = env
            .storage()
            .instance()
            .get(&owner_key)
            .unwrap_or(Vec::new(&env));
        owner_tokens.push_back(token_id);
        env.storage().instance().set(&owner_key, &owner_tokens);

        // Add to patient's consents list (for revoke access)
        let patient_key = DataKey::PatientConsents(patient.clone());
        let mut patient_consents: Vec<u64> = env
            .storage()
            .instance()
            .get(&patient_key)
            .unwrap_or(Vec::new(&env));
        patient_consents.push_back(token_id);
        env.storage()
            .instance()
            .set(&patient_key, &patient_consents);

        // Initialize consent history
        let history_entry = ConsentHistoryEntry {
            action: String::from_str(&env, "issued"),
            timestamp: env.ledger().timestamp(),
            actor: issuer.clone(),
            metadata_uri: metadata_uri.clone(),
            details: String::from_str(&env, "Consent issued"),
        };
        let mut history = Vec::new(&env);
        history.push_back(history_entry);
        env.storage()
            .instance()
            .set(&DataKey::ConsentHistory(token_id), &history);

        // Update analytics
        let mut analytics: AnalyticsData = env
            .storage()
            .instance()
            .get(&DataKey::AnalyticsData)
            .unwrap_or(AnalyticsData {
                total_consents: 0,
                active_consents: 0,
                revoked_consents: 0,
                total_delegations: 0,
                total_emergency_overrides: 0,
                marketplace_listings: 0,
                total_access_count: 0,
            });
        analytics.total_consents += 1;
        analytics.active_consents += 1;
        env.storage()
            .instance()
            .set(&DataKey::AnalyticsData, &analytics);

        // Emit event
        env.events().publish(
            (symbol_short!("consent"), symbol_short!("issued")),
            (token_id, patient, consent_type, metadata_uri),
        );

        Ok(token_id)
    }

    /// Update consent metadata (creates new version)
    pub fn update_consent(
        env: Env,
        token_id: u64,
        new_metadata_uri: String,
    ) -> Result<(), ContractError> {
        // Verify token exists and is not revoked
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenOwner(token_id))
            .expect("Token does not exist");

        let is_revoked: bool = env
            .storage()
            .instance()
            .get(&DataKey::TokenRevoked(token_id))
            .unwrap_or(false);

        if is_revoked {
            return Err(ContractError::ConsentRevoked);
        }

        // Verify caller is owner (or tighten to issuer/patient if needed)
        owner.require_auth();

        // Get and update metadata
        let mut metadata: ConsentMetadata = env
            .storage()
            .instance()
            .get(&DataKey::TokenMetadata(token_id))
            .expect("Metadata not found");

        metadata.metadata_uri = new_metadata_uri.clone();
        metadata.version += 1;

        env.storage()
            .instance()
            .set(&DataKey::TokenMetadata(token_id), &metadata);

        // Add to history
        let history_entry = ConsentHistoryEntry {
            action: String::from_str(&env, "updated"),
            timestamp: env.ledger().timestamp(),
            actor: owner.clone(),
            metadata_uri: new_metadata_uri.clone(),
            details: String::from_str(&env, "Consent metadata updated"),
        };

        let mut history: Vec<ConsentHistoryEntry> = env
            .storage()
            .instance()
            .get(&DataKey::ConsentHistory(token_id))
            .unwrap_or(Vec::new(&env));
        history.push_back(history_entry);
        env.storage()
            .instance()
            .set(&DataKey::ConsentHistory(token_id), &history);

        // Emit event
        env.events().publish(
            (symbol_short!("consent"), symbol_short!("updated")),
            (token_id, metadata.version, new_metadata_uri),
        );
        Ok(())
    }

    /// Revoke consent (marks as revoked, prevents transfers) - Patient authorizes via require_auth on their address from metadata
    pub fn revoke_consent(env: Env, token_id: u64) -> Result<(), ContractError> {
        // Verify token exists
        let _: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenOwner(token_id))
            .ok_or(ContractError::TokenNotFound)?;

        let metadata: ConsentMetadata = env
            .storage()
            .instance()
            .get(&DataKey::TokenMetadata(token_id))
            .ok_or(ContractError::TokenNotFound)?;

        let patient = metadata.patient;

        // Patient must authorize revoke (controls their consent)
        patient.require_auth();

        let is_revoked: bool = env
            .storage()
            .instance()
            .get(&DataKey::TokenRevoked(token_id))
            .unwrap_or(false);

        if is_revoked {
            return Err(ContractError::ConsentRevoked);
        }

        // Mark as revoked
        env.storage()
            .instance()
            .set(&DataKey::TokenRevoked(token_id), &true);

        // Update analytics
        let mut analytics: AnalyticsData = env
            .storage()
            .instance()
            .get(&DataKey::AnalyticsData)
            .unwrap_or(AnalyticsData {
                total_consents: 0,
                active_consents: 0,
                revoked_consents: 0,
                total_delegations: 0,
                total_emergency_overrides: 0,
                marketplace_listings: 0,
                total_access_count: 0,
            });
        analytics.active_consents = analytics.active_consents.saturating_sub(1);
        analytics.revoked_consents += 1;
        env.storage()
            .instance()
            .set(&DataKey::AnalyticsData, &analytics);

        // Add to history
        let history_entry = ConsentHistoryEntry {
            action: String::from_str(&env, "revoked"),
            timestamp: env.ledger().timestamp(),
            actor: patient.clone(),
            metadata_uri: metadata.metadata_uri.clone(),
            details: String::from_str(&env, "Consent revoked by patient"),
        };

        let mut history: Vec<ConsentHistoryEntry> = env
            .storage()
            .instance()
            .get(&DataKey::ConsentHistory(token_id))
            .unwrap_or(Vec::new(&env));
        history.push_back(history_entry);
        env.storage()
            .instance()
            .set(&DataKey::ConsentHistory(token_id), &history);

        // Emit event
        env.events().publish(
            (symbol_short!("consent"), symbol_short!("revoked")),
            (token_id, patient),
        );

        Ok(())
    }

    /// Transfer consent token (blocked if revoked)
    pub fn transfer(
        env: Env,
        from: Address,
        to: Address,
        token_id: u64,
    ) -> Result<(), ContractError> {
        from.require_auth();

        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenOwner(token_id))
            .expect("Token does not exist");

        if owner != from {
            return Err(ContractError::NotTokenOwner);
        }

        let is_revoked: bool = env
            .storage()
            .instance()
            .get(&DataKey::TokenRevoked(token_id))
            .unwrap_or(false);

        if is_revoked {
            return Err(ContractError::ConsentRevoked);
        }

        // Update ownership
        env.storage()
            .instance()
            .set(&DataKey::TokenOwner(token_id), &to);

        // Update token lists
        let from_key = DataKey::OwnerTokens(from.clone());
        let from_tokens: Vec<u64> = env
            .storage()
            .instance()
            .get(&from_key)
            .unwrap_or(Vec::new(&env));

        let mut new_from_tokens = Vec::new(&env);
        for i in 0..from_tokens.len() {
            if let Some(tid) = from_tokens.get(i) {
                if tid != token_id {
                    new_from_tokens.push_back(tid);
                }
            }
        }
        env.storage().instance().set(&from_key, &new_from_tokens);

        let to_key = DataKey::OwnerTokens(to.clone());
        let mut to_tokens: Vec<u64> = env
            .storage()
            .instance()
            .get(&to_key)
            .unwrap_or(Vec::new(&env));
        to_tokens.push_back(token_id);
        env.storage().instance().set(&to_key, &to_tokens);

        // PatientConsents list unchanged - patient still tracks/revokes it

        // Emit event
        env.events().publish(
            (symbol_short!("consent"), symbol_short!("transfer")),
            (token_id, from, to),
        );
        Ok(())
    }

    /// Get token owner
    pub fn owner_of(env: Env, token_id: u64) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::TokenOwner(token_id))
            .expect("Token does not exist")
    }

    /// Get consent metadata
    pub fn get_metadata(env: Env, token_id: u64) -> ConsentMetadata {
        env.storage()
            .instance()
            .get(&DataKey::TokenMetadata(token_id))
            .expect("Token does not exist")
    }

    /// Check if consent is revoked
    pub fn is_revoked(env: Env, token_id: u64) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::TokenRevoked(token_id))
            .unwrap_or(false)
    }

    /// Get consent history (audit trail)
    pub fn get_history(env: Env, token_id: u64) -> Vec<ConsentHistoryEntry> {
        env.storage()
            .instance()
            .get(&DataKey::ConsentHistory(token_id))
            .unwrap_or(Vec::new(&env))
    }

    /// Get all tokens owned by an address
    pub fn tokens_of_owner(env: Env, owner: Address) -> Vec<u64> {
        env.storage()
            .instance()
            .get(&DataKey::OwnerTokens(owner))
            .unwrap_or(Vec::new(&env))
    }

    /// Check if doctor has valid consent for patient and type (for cross-contract access control)
    pub fn has_consent(env: Env, patient: Address, doctor: Address, consent_type: String) -> bool {
        let tokens = Self::tokens_of_owner(env.clone(), doctor);
        for i in 0..tokens.len() {
            let token_id = tokens.get(i).unwrap();
            if Self::is_revoked(env.clone(), token_id) {
                continue;
            }
            let metadata = Self::get_metadata(env.clone(), token_id);
            if metadata.patient == patient
                && metadata.consent_type == consent_type
                && (metadata.expiry_timestamp == 0
                    || env.ledger().timestamp() < metadata.expiry_timestamp)
            {
                return true;
            }
        }
        false
    }

    /// Check if consent is valid (not revoked and not expired)
    pub fn is_valid(env: Env, token_id: u64) -> bool {
        let is_revoked: bool = env
            .storage()
            .instance()
            .get(&DataKey::TokenRevoked(token_id))
            .unwrap_or(false);

        if is_revoked {
            return false;
        }

        let metadata: ConsentMetadata = env
            .storage()
            .instance()
            .get(&DataKey::TokenMetadata(token_id))
            .expect("Token does not exist");

        if metadata.expiry_timestamp == 0 {
            return true; // No expiry
        }

        env.ledger().timestamp() < metadata.expiry_timestamp
    }

    // ========== ADVANCED FEATURES ==========

    /// Set granular permissions for a consent token
    pub fn set_granular_permissions(
        env: Env,
        caller: Address,
        token_id: u64,
        permissions: GranularPermissions,
    ) -> Result<(), ContractError> {
        let metadata: ConsentMetadata = env
            .storage()
            .instance()
            .get(&DataKey::TokenMetadata(token_id))
            .ok_or(ContractError::TokenNotFound)?;

        // Only patient or issuer can set permissions
        if caller != metadata.patient && caller != metadata.issuer {
            return Err(ContractError::NotAuthorized);
        }

        caller.require_auth();

        env.storage()
            .instance()
            .set(&DataKey::GranularPermissions(token_id), &permissions);

        // Add to history
        let history_entry = ConsentHistoryEntry {
            action: String::from_str(&env, "permissions_updated"),
            timestamp: env.ledger().timestamp(),
            actor: caller.clone(),
            metadata_uri: metadata.metadata_uri.clone(),
            details: String::from_str(&env, "Granular permissions updated"),
        };

        let mut history: Vec<ConsentHistoryEntry> = env
            .storage()
            .instance()
            .get(&DataKey::ConsentHistory(token_id))
            .unwrap_or(Vec::new(&env));
        history.push_back(history_entry);
        env.storage()
            .instance()
            .set(&DataKey::ConsentHistory(token_id), &history);

        env.events().publish(
            (symbol_short!("consent"), symbol_short!("perm_upd")),
            (token_id, caller.clone()),
        );

        Ok(())
    }

    /// Get granular permissions for a consent token
    pub fn get_granular_permissions(
        env: Env,
        token_id: u64,
    ) -> Result<GranularPermissions, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::GranularPermissions(token_id))
            .ok_or(ContractError::TokenNotFound)
    }

    /// Check if requester has permission for specific data type
    pub fn has_permission(
        env: Env,
        token_id: u64,
        requester: Address,
        data_type: DataType,
        required_level: PermissionLevel,
    ) -> bool {
        // Check if token is valid
        if !Self::is_valid(env.clone(), token_id) {
            return false;
        }

        let metadata: ConsentMetadata = env
            .storage()
            .instance()
            .get(&DataKey::TokenMetadata(token_id))
            .ok_or(ContractError::TokenNotFound)
            .unwrap();

        // Owner always has full access
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenOwner(token_id))
            .unwrap();
        if requester == owner || requester == metadata.patient {
            return true;
        }

        // Check granular permissions
        let permissions: GranularPermissions = env
            .storage()
            .instance()
            .get(&DataKey::GranularPermissions(token_id))
            .unwrap_or(GranularPermissions {
                permissions: Map::new(&env),
            });

        let permission_level = permissions
            .permissions
            .get(data_type)
            .unwrap_or(PermissionLevel::None);

        // Check if permission level meets requirement
        matches!(
            (permission_level, required_level),
            (PermissionLevel::Full, _)
                | (PermissionLevel::Write, PermissionLevel::Read)
                | (PermissionLevel::Write, PermissionLevel::Write)
                | (PermissionLevel::Read, PermissionLevel::Read)
        )
    }

    /// Set access controls for a consent token
    pub fn set_access_controls(
        env: Env,
        token_id: u64,
        access_control: AccessControl,
    ) -> Result<(), ContractError> {
        let metadata: ConsentMetadata = env
            .storage()
            .instance()
            .get(&DataKey::TokenMetadata(token_id))
            .ok_or(ContractError::TokenNotFound)?;

        metadata.patient.require_auth();

        env.storage()
            .instance()
            .set(&DataKey::AccessControls(token_id), &access_control);

        // Add to history
        let history_entry = ConsentHistoryEntry {
            action: String::from_str(&env, "access_controls_updated"),
            timestamp: env.ledger().timestamp(),
            actor: metadata.patient.clone(),
            metadata_uri: metadata.metadata_uri.clone(),
            details: String::from_str(&env, "Access controls updated"),
        };

        let mut history: Vec<ConsentHistoryEntry> = env
            .storage()
            .instance()
            .get(&DataKey::ConsentHistory(token_id))
            .unwrap_or(Vec::new(&env));
        history.push_back(history_entry);
        env.storage()
            .instance()
            .set(&DataKey::ConsentHistory(token_id), &history);

        Ok(())
    }

    /// Check if access is allowed based on access controls
    pub fn check_access_allowed(
        env: Env,
        token_id: u64,
        _requester: Address,
    ) -> Result<bool, ContractError> {
        if !Self::is_valid(env.clone(), token_id) {
            return Ok(false);
        }

        let access_control: AccessControl = env
            .storage()
            .instance()
            .get(&DataKey::AccessControls(token_id))
            .unwrap_or(AccessControl {
                conditions: Vec::new(&env),
                max_access_count: 0,
                current_access_count: 0,
                last_access_timestamp: 0,
            });

        // Check access count limit
        if access_control.max_access_count > 0
            && access_control.current_access_count >= access_control.max_access_count
        {
            return Ok(false);
        }

        // Check conditions
        let current_time = env.ledger().timestamp();
        for i in 0..access_control.conditions.len() {
            let condition = access_control.conditions.get(i).unwrap();
            match condition {
                AccessCondition::TimeWindow(start, end) => {
                    if current_time < start || current_time > end {
                        return Ok(false);
                    }
                }
                AccessCondition::DayOfWeek(days) => {
                    // Simple day check (assuming timestamp % 7 gives day of week)
                    let day = (current_time / 86400) % 7;
                    let mut found = false;
                    for j in 0..days.len() {
                        if days.get(j).unwrap() == day as u32 {
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        return Ok(false);
                    }
                }
                AccessCondition::TimeOfDay(start_hour, end_hour) => {
                    let hour = (current_time % 86400) / 3600;
                    if hour < (start_hour as u64) || hour > (end_hour as u64) {
                        return Ok(false);
                    }
                }
                AccessCondition::EmergencyOnly => {
                    // Only emergency overrides can access
                    return Ok(false);
                }
                _ => {
                    // Other conditions would need additional context
                }
            }
        }

        Ok(true)
    }

    /// Record access attempt
    pub fn record_access(
        env: Env,
        token_id: u64,
        _requester: Address,
    ) -> Result<(), ContractError> {
        let mut access_control: AccessControl = env
            .storage()
            .instance()
            .get(&DataKey::AccessControls(token_id))
            .unwrap_or(AccessControl {
                conditions: Vec::new(&env),
                max_access_count: 0,
                current_access_count: 0,
                last_access_timestamp: 0,
            });

        access_control.current_access_count += 1;
        access_control.last_access_timestamp = env.ledger().timestamp();

        env.storage()
            .instance()
            .set(&DataKey::AccessControls(token_id), &access_control);

        // Update analytics
        let mut analytics: AnalyticsData = env
            .storage()
            .instance()
            .get(&DataKey::AnalyticsData)
            .unwrap_or(AnalyticsData {
                total_consents: 0,
                active_consents: 0,
                revoked_consents: 0,
                total_delegations: 0,
                total_emergency_overrides: 0,
                marketplace_listings: 0,
                total_access_count: 0,
            });
        analytics.total_access_count += 1;
        env.storage()
            .instance()
            .set(&DataKey::AnalyticsData, &analytics);

        Ok(())
    }

    /// Delegate consent to another address
    pub fn delegate_consent(
        env: Env,
        token_id: u64,
        delegate: Address,
        permissions: GranularPermissions,
        expiry_timestamp: u64,
    ) -> Result<(), ContractError> {
        let metadata: ConsentMetadata = env
            .storage()
            .instance()
            .get(&DataKey::TokenMetadata(token_id))
            .ok_or(ContractError::TokenNotFound)?;

        metadata.patient.require_auth();

        let delegation = Delegation {
            delegate: delegate.clone(),
            permissions: permissions.clone(),
            expiry_timestamp,
            created_timestamp: env.ledger().timestamp(),
        };

        let mut delegations: Vec<Delegation> = env
            .storage()
            .instance()
            .get(&DataKey::ConsentDelegations(token_id))
            .unwrap_or(Vec::new(&env));
        delegations.push_back(delegation);
        env.storage()
            .instance()
            .set(&DataKey::ConsentDelegations(token_id), &delegations);

        // Update analytics
        let mut analytics: AnalyticsData = env
            .storage()
            .instance()
            .get(&DataKey::AnalyticsData)
            .unwrap_or(AnalyticsData {
                total_consents: 0,
                active_consents: 0,
                revoked_consents: 0,
                total_delegations: 0,
                total_emergency_overrides: 0,
                marketplace_listings: 0,
                total_access_count: 0,
            });
        analytics.total_delegations += 1;
        env.storage()
            .instance()
            .set(&DataKey::AnalyticsData, &analytics);

        // Add to history
        let history_entry = ConsentHistoryEntry {
            action: String::from_str(&env, "delegated"),
            timestamp: env.ledger().timestamp(),
            actor: metadata.patient.clone(),
            metadata_uri: metadata.metadata_uri.clone(),
            details: String::from_str(&env, "Consent delegated"),
        };

        let mut history: Vec<ConsentHistoryEntry> = env
            .storage()
            .instance()
            .get(&DataKey::ConsentHistory(token_id))
            .unwrap_or(Vec::new(&env));
        history.push_back(history_entry);
        env.storage()
            .instance()
            .set(&DataKey::ConsentHistory(token_id), &history);

        env.events().publish(
            (symbol_short!("consent"), symbol_short!("delegated")),
            (token_id, metadata.patient, delegate),
        );

        Ok(())
    }

    /// Revoke delegation
    pub fn revoke_delegation(
        env: Env,
        token_id: u64,
        delegate: Address,
    ) -> Result<(), ContractError> {
        let metadata: ConsentMetadata = env
            .storage()
            .instance()
            .get(&DataKey::TokenMetadata(token_id))
            .ok_or(ContractError::TokenNotFound)?;

        metadata.patient.require_auth();

        let delegations: Vec<Delegation> = env
            .storage()
            .instance()
            .get(&DataKey::ConsentDelegations(token_id))
            .unwrap_or(Vec::new(&env));

        let mut new_delegations = Vec::new(&env);
        for i in 0..delegations.len() {
            let d = delegations.get(i).unwrap();
            if d.delegate != delegate {
                new_delegations.push_back(d);
            }
        }

        env.storage()
            .instance()
            .set(&DataKey::ConsentDelegations(token_id), &new_delegations);

        Ok(())
    }

    /// Get active delegations for a token
    pub fn get_delegations(env: Env, token_id: u64) -> Vec<Delegation> {
        let delegations: Vec<Delegation> = env
            .storage()
            .instance()
            .get(&DataKey::ConsentDelegations(token_id))
            .unwrap_or(Vec::new(&env));

        let current_time = env.ledger().timestamp();
        let mut active_delegations = Vec::new(&env);

        for i in 0..delegations.len() {
            let d = delegations.get(i).unwrap();
            if d.expiry_timestamp == 0 || current_time < d.expiry_timestamp {
                active_delegations.push_back(d);
            }
        }

        active_delegations
    }

    /// Set consent inheritance (child consent inherits from parent)
    pub fn set_inheritance(
        env: Env,
        child_token_id: u64,
        parent_token_id: u64,
        inherited_permissions: GranularPermissions,
    ) -> Result<(), ContractError> {
        // Check for cycles
        let mut current = parent_token_id;
        let mut visited = Vec::new(&env);
        visited.push_back(current);

        loop {
            let inheritance: Option<Inheritance> = env
                .storage()
                .instance()
                .get(&DataKey::ConsentInheritance(current));
            match inheritance {
                Some(inh) => {
                    // Check if already visited
                    let mut found = false;
                    for i in 0..visited.len() {
                        if visited.get(i).unwrap() == inh.parent_token_id {
                            found = true;
                            break;
                        }
                    }
                    if found {
                        return Err(ContractError::InheritanceCycle);
                    }
                    if inh.parent_token_id == child_token_id {
                        return Err(ContractError::InheritanceCycle);
                    }
                    current = inh.parent_token_id;
                    visited.push_back(current);
                }
                None => break,
            }
        }

        let child_metadata: ConsentMetadata = env
            .storage()
            .instance()
            .get(&DataKey::TokenMetadata(child_token_id))
            .ok_or(ContractError::TokenNotFound)?;

        child_metadata.patient.require_auth();

        let inheritance = Inheritance {
            parent_token_id,
            inherited_permissions,
        };

        env.storage()
            .instance()
            .set(&DataKey::ConsentInheritance(child_token_id), &inheritance);

        // Add to history
        let history_entry = ConsentHistoryEntry {
            action: String::from_str(&env, "inheritance_set"),
            timestamp: env.ledger().timestamp(),
            actor: child_metadata.patient.clone(),
            metadata_uri: child_metadata.metadata_uri.clone(),
            details: String::from_str(&env, "Inheritance set"),
        };

        let mut history: Vec<ConsentHistoryEntry> = env
            .storage()
            .instance()
            .get(&DataKey::ConsentHistory(child_token_id))
            .unwrap_or(Vec::new(&env));
        history.push_back(history_entry);
        env.storage()
            .instance()
            .set(&DataKey::ConsentHistory(child_token_id), &history);

        Ok(())
    }

    /// Add emergency authority
    pub fn add_emergency_authority(env: Env, authority: Address) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Not initialized");
        admin.require_auth();

        let mut authorities: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::EmergencyAuthorities)
            .unwrap_or(Vec::new(&env));

        authorities.push_back(authority);
        env.storage()
            .instance()
            .set(&DataKey::EmergencyAuthorities, &authorities);

        Ok(())
    }

    /// Emergency override access
    pub fn emergency_override(
        env: Env,
        caller: Address,
        token_id: u64,
        reason: String,
        duration: u64,
    ) -> Result<u64, ContractError> {
        caller.require_auth();

        let authorities: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::EmergencyAuthorities)
            .unwrap_or(Vec::new(&env));

        let mut is_authorized = false;
        for i in 0..authorities.len() {
            if authorities.get(i).unwrap() == caller {
                is_authorized = true;
                break;
            }
        }

        if !is_authorized {
            return Err(ContractError::NotAuthorized);
        }

        let metadata: ConsentMetadata = env
            .storage()
            .instance()
            .get(&DataKey::TokenMetadata(token_id))
            .ok_or(ContractError::TokenNotFound)?;

        // Generate override ID
        let override_id = env
            .storage()
            .instance()
            .get(&DataKey::TokenCounter)
            .unwrap_or(0);

        let override_record = EmergencyOverride {
            override_id,
            authorized_by: caller.clone(),
            reason: reason.clone(),
            timestamp: env.ledger().timestamp(),
            duration,
            used: false,
        };

        let mut overrides: Vec<EmergencyOverride> = env
            .storage()
            .instance()
            .get(&DataKey::EmergencyOverrides(token_id))
            .unwrap_or(Vec::new(&env));
        overrides.push_back(override_record);
        env.storage()
            .instance()
            .set(&DataKey::EmergencyOverrides(token_id), &overrides);

        // Update analytics
        let mut analytics: AnalyticsData = env
            .storage()
            .instance()
            .get(&DataKey::AnalyticsData)
            .unwrap_or(AnalyticsData {
                total_consents: 0,
                active_consents: 0,
                revoked_consents: 0,
                total_delegations: 0,
                total_emergency_overrides: 0,
                marketplace_listings: 0,
                total_access_count: 0,
            });
        analytics.total_emergency_overrides += 1;
        env.storage()
            .instance()
            .set(&DataKey::AnalyticsData, &analytics);

        // Add to history
        let history_entry = ConsentHistoryEntry {
            action: String::from_str(&env, "emergency_override"),
            timestamp: env.ledger().timestamp(),
            actor: caller.clone(),
            metadata_uri: metadata.metadata_uri.clone(),
            details: reason.clone(),
        };

        let mut history: Vec<ConsentHistoryEntry> = env
            .storage()
            .instance()
            .get(&DataKey::ConsentHistory(token_id))
            .unwrap_or(Vec::new(&env));
        history.push_back(history_entry);
        env.storage()
            .instance()
            .set(&DataKey::ConsentHistory(token_id), &history);

        env.events().publish(
            (symbol_short!("consent"), symbol_short!("emerg_ovr")),
            (token_id, caller, reason),
        );

        Ok(override_id)
    }

    /// Enable/disable marketplace
    pub fn set_marketplace_enabled(env: Env, enabled: bool) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Not initialized");
        admin.require_auth();

        env.storage()
            .instance()
            .set(&DataKey::MarketplaceEnabled, &enabled);

        Ok(())
    }

    /// List consent on marketplace for research
    pub fn list_on_marketplace(
        env: Env,
        token_id: u64,
        price: i128,
        data_types: Vec<DataType>,
        research_purpose: String,
        duration: u64,
    ) -> Result<(), ContractError> {
        let marketplace_enabled: bool = env
            .storage()
            .instance()
            .get(&DataKey::MarketplaceEnabled)
            .unwrap_or(false);

        if !marketplace_enabled {
            return Err(ContractError::MarketplaceNotEnabled);
        }

        let metadata: ConsentMetadata = env
            .storage()
            .instance()
            .get(&DataKey::TokenMetadata(token_id))
            .ok_or(ContractError::TokenNotFound)?;

        // Only patient can list their consent
        metadata.patient.require_auth();

        let listing = MarketplaceListing {
            token_id,
            price,
            data_types: data_types.clone(),
            research_purpose: research_purpose.clone(),
            duration,
            listed_by: metadata.patient.clone(),
            listed_timestamp: env.ledger().timestamp(),
            active: true,
        };

        env.storage()
            .instance()
            .set(&DataKey::MarketplaceListings(token_id), &listing);

        // Update analytics
        let mut analytics: AnalyticsData = env
            .storage()
            .instance()
            .get(&DataKey::AnalyticsData)
            .unwrap_or(AnalyticsData {
                total_consents: 0,
                active_consents: 0,
                revoked_consents: 0,
                total_delegations: 0,
                total_emergency_overrides: 0,
                marketplace_listings: 0,
                total_access_count: 0,
            });
        analytics.marketplace_listings += 1;
        env.storage()
            .instance()
            .set(&DataKey::AnalyticsData, &analytics);

        // Add to history
        let history_entry = ConsentHistoryEntry {
            action: String::from_str(&env, "marketplace_listed"),
            timestamp: env.ledger().timestamp(),
            actor: metadata.patient.clone(),
            metadata_uri: metadata.metadata_uri.clone(),
            details: research_purpose.clone(),
        };

        let mut history: Vec<ConsentHistoryEntry> = env
            .storage()
            .instance()
            .get(&DataKey::ConsentHistory(token_id))
            .unwrap_or(Vec::new(&env));
        history.push_back(history_entry);
        env.storage()
            .instance()
            .set(&DataKey::ConsentHistory(token_id), &history);

        env.events().publish(
            (symbol_short!("consent"), symbol_short!("mkt_list")),
            (token_id, price, research_purpose),
        );

        Ok(())
    }

    /// Get marketplace listing
    pub fn get_marketplace_listing(
        env: Env,
        token_id: u64,
    ) -> Result<MarketplaceListing, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::MarketplaceListings(token_id))
            .ok_or(ContractError::TokenNotFound)
    }

    /// Purchase marketplace listing (simplified - would need payment integration)
    pub fn purchase_marketplace_listing(
        env: Env,
        token_id: u64,
        buyer: Address,
    ) -> Result<(), ContractError> {
        buyer.require_auth();

        let mut listing: MarketplaceListing = env
            .storage()
            .instance()
            .get(&DataKey::MarketplaceListings(token_id))
            .ok_or(ContractError::TokenNotFound)?;

        if !listing.active {
            return Err(ContractError::TokenNotFound);
        }

        // In a real implementation, this would handle payment
        // For now, we just transfer access permissions

        listing.active = false;
        env.storage()
            .instance()
            .set(&DataKey::MarketplaceListings(token_id), &listing);

        // Create delegation for buyer
        let permissions = GranularPermissions {
            permissions: {
                let mut perms = Map::new(&env);
                for i in 0..listing.data_types.len() {
                    perms.set(listing.data_types.get(i).unwrap(), PermissionLevel::Read);
                }
                perms
            },
        };

        Self::delegate_consent(
            env.clone(),
            token_id,
            buyer.clone(),
            permissions,
            env.ledger().timestamp() + listing.duration,
        )?;

        env.events().publish(
            (symbol_short!("consent"), symbol_short!("mkt_purch")),
            (token_id, listing.listed_by, buyer),
        );

        Ok(())
    }

    /// Enhanced dynamic consent update with version history
    pub fn update_consent_dynamic(
        env: Env,
        caller: Address,
        token_id: u64,
        new_metadata_uri: String,
        change_summary: String,
    ) -> Result<(), ContractError> {
        let metadata: ConsentMetadata = env
            .storage()
            .instance()
            .get(&DataKey::TokenMetadata(token_id))
            .ok_or(ContractError::TokenNotFound)?;

        if !metadata.dynamic_updates_enabled {
            return Err(ContractError::NotAuthorized);
        }

        if caller != metadata.patient && caller != metadata.issuer {
            return Err(ContractError::NotAuthorized);
        }
        caller.require_auth();

        // Save current version to history
        let version_entry = VersionHistoryEntry {
            version: metadata.version,
            metadata_uri: metadata.metadata_uri.clone(),
            updated_by: caller.clone(),
            timestamp: env.ledger().timestamp(),
            change_summary: change_summary.clone(),
        };

        let mut version_history: Vec<VersionHistoryEntry> = env
            .storage()
            .instance()
            .get(&DataKey::VersionHistory(token_id))
            .unwrap_or(Vec::new(&env));
        version_history.push_back(version_entry);

        env.storage()
            .instance()
            .set(&DataKey::VersionHistory(token_id), &version_history);

        // Update metadata
        let mut new_metadata = metadata.clone();
        new_metadata.metadata_uri = new_metadata_uri.clone();
        new_metadata.version += 1;

        env.storage()
            .instance()
            .set(&DataKey::TokenMetadata(token_id), &new_metadata);

        // Add to history
        let history_entry = ConsentHistoryEntry {
            action: String::from_str(&env, "updated_dynamic"),
            timestamp: env.ledger().timestamp(),
            actor: caller.clone(),
            metadata_uri: new_metadata_uri.clone(),
            details: change_summary.clone(),
        };

        let mut history: Vec<ConsentHistoryEntry> = env
            .storage()
            .instance()
            .get(&DataKey::ConsentHistory(token_id))
            .unwrap_or(Vec::new(&env));
        history.push_back(history_entry);
        env.storage()
            .instance()
            .set(&DataKey::ConsentHistory(token_id), &history);

        env.events().publish(
            (symbol_short!("consent"), symbol_short!("upd_dyn")),
            (token_id, new_metadata.version, change_summary),
        );

        Ok(())
    }

    /// Get version history
    pub fn get_version_history(env: Env, token_id: u64) -> Vec<VersionHistoryEntry> {
        env.storage()
            .instance()
            .get(&DataKey::VersionHistory(token_id))
            .unwrap_or(Vec::new(&env))
    }

    /// Enable dynamic updates for a consent
    pub fn enable_dynamic_updates(env: Env, token_id: u64) -> Result<(), ContractError> {
        let mut metadata: ConsentMetadata = env
            .storage()
            .instance()
            .get(&DataKey::TokenMetadata(token_id))
            .ok_or(ContractError::TokenNotFound)?;

        metadata.patient.require_auth();

        metadata.dynamic_updates_enabled = true;
        env.storage()
            .instance()
            .set(&DataKey::TokenMetadata(token_id), &metadata);

        Ok(())
    }

    /// Get analytics data
    pub fn get_analytics(env: Env) -> AnalyticsData {
        env.storage()
            .instance()
            .get(&DataKey::AnalyticsData)
            .unwrap_or(AnalyticsData {
                total_consents: 0,
                active_consents: 0,
                revoked_consents: 0,
                total_delegations: 0,
                total_emergency_overrides: 0,
                marketplace_listings: 0,
                total_access_count: 0,
            })
    }

    /// Generate consent report for a patient
    pub fn generate_consent_report(env: Env, patient: Address) -> Vec<u64> {
        env.storage()
            .instance()
            .get(&DataKey::PatientConsents(patient))
            .unwrap_or(Vec::new(&env))
    }
}

// Tests moved to test.rs module to avoid direct contract function calls
// that cause storage access issues in test environment
