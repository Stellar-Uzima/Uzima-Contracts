#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    ProviderNotFound = 4,
    ReputationContractNotFound = 5,
    HealthcareReputationContractNotFound = 6,
    InvalidScoreMapping = 7,
    SyncFailed = 8,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScoreMapping {
    pub base_reputation_weight: u32, // Weight for base reputation score (0-100)
    pub healthcare_reputation_weight: u32, // Weight for healthcare reputation score (0-100)
    pub adjustment_factor: i32,      // Adjustment factor for healthcare-specific factors
    pub last_sync_timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyncRecord {
    pub provider: Address,
    pub base_score: i128,
    pub healthcare_score: u32,
    pub combined_score: i128,
    pub timestamp: u64,
    pub sync_type: SyncType,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SyncType {
    Manual = 0,
    Automatic = 1,
    CredentialUpdate = 2,
    FeedbackUpdate = 3,
    ConductUpdate = 4,
}

#[contracttype]
pub enum DataKey {
    Admin,
    Initialized,
    BaseReputationContract,
    HealthcareReputationContract,
    ScoreMapping,
    SyncRecord(Address, u64),  // provider, timestamp
    ProviderSyncList(Address), // Vec<u64> timestamps
    LastSyncTime(Address),     // u64 timestamp
    SyncSettings,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyncSettings {
    pub auto_sync_enabled: bool,
    pub sync_interval_hours: u32,
    pub sync_on_credential_change: bool,
    pub sync_on_feedback_change: bool,
    pub sync_on_conduct_change: bool,
}

#[contract]
pub struct ReputationIntegration;

#[contractimpl]
impl ReputationIntegration {
    // Initialize integration system
    pub fn initialize(
        env: Env,
        admin: Address,
        base_reputation_contract: Address,
        healthcare_reputation_contract: Address,
    ) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage()
            .instance()
            .set(&DataKey::BaseReputationContract, &base_reputation_contract);
        env.storage().instance().set(
            &DataKey::HealthcareReputationContract,
            &healthcare_reputation_contract,
        );

        // Set default score mapping
        let default_mapping = ScoreMapping {
            base_reputation_weight: 60,       // 60% weight to base reputation
            healthcare_reputation_weight: 40, // 40% weight to healthcare reputation
            adjustment_factor: 0,             // No adjustment initially
            last_sync_timestamp: env.ledger().timestamp(),
        };
        env.storage()
            .instance()
            .set(&DataKey::ScoreMapping, &default_mapping);

        // Set default sync settings
        let default_settings = SyncSettings {
            auto_sync_enabled: true,
            sync_interval_hours: 24,
            sync_on_credential_change: true,
            sync_on_feedback_change: true,
            sync_on_conduct_change: true,
        };
        env.storage()
            .instance()
            .set(&DataKey::SyncSettings, &default_settings);

        env.events()
            .publish((symbol_short!("REPUTINT"), symbol_short!("INIT")), admin);
        Ok(())
    }

    // Sync provider reputation scores
    pub fn sync_provider_reputation(
        env: Env,
        admin: Address,
        provider: Address,
    ) -> Result<i128, Error> {
        admin.require_auth();
        Self::require_initialized(&env)?;

        let base_score = Self::get_base_reputation_score(&env, provider.clone())?;
        let healthcare_score = Self::get_healthcare_reputation_score(&env, provider.clone())?;
        let combined_score = Self::calculate_combined_score(&env, base_score, healthcare_score)?;

        // Record sync
        let sync_record = SyncRecord {
            provider: provider.clone(),
            base_score,
            healthcare_score,
            combined_score,
            timestamp: env.ledger().timestamp(),
            sync_type: SyncType::Manual,
        };

        Self::store_sync_record(&env, sync_record)?;

        // Update base reputation contract with combined score
        Self::update_base_reputation_score(&env, provider.clone(), combined_score)?;

        env.events().publish(
            (symbol_short!("REPUTINT"), symbol_short!("SYNC")),
            (provider, combined_score),
        );
        Ok(combined_score)
    }

    // Batch sync multiple providers
    pub fn batch_sync_providers(
        env: Env,
        admin: Address,
        providers: Vec<Address>,
    ) -> Result<Vec<i128>, Error> {
        admin.require_auth();
        Self::require_initialized(&env)?;

        let mut results = Vec::new(&env);
        for provider in providers.iter() {
            match Self::sync_provider_reputation(env.clone(), admin.clone(), provider.clone()) {
                Ok(score) => results.push_back(score),
                Err(_) => {
                    // Continue with other providers even if one fails
                    results.push_back(0);
                },
            }
        }

        Ok(results)
    }

    // Auto-sync all providers (called by cron job)
    pub fn auto_sync_all_providers(env: Env, admin: Address) -> Result<u32, Error> {
        admin.require_auth();
        Self::require_initialized(&env)?;

        let settings: SyncSettings = env
            .storage()
            .instance()
            .get(&DataKey::SyncSettings)
            .ok_or(Error::NotInitialized)?;

        if !settings.auto_sync_enabled {
            return Ok(0);
        }

        // In a real implementation, you'd get list of all providers from provider directory
        // For now, we'll return 0 as placeholder
        let synced_count = 0;

        env.events().publish(
            (symbol_short!("REPUTINT"), symbol_short!("AUTO_SYNC")),
            synced_count,
        );
        Ok(synced_count)
    }

    // Update score mapping
    pub fn update_score_mapping(
        env: Env,
        admin: Address,
        base_weight: u32,
        healthcare_weight: u32,
        adjustment_factor: i32,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        if base_weight.checked_add(healthcare_weight) != Some(100) {
            return Err(Error::InvalidScoreMapping);
        }

        let mapping = ScoreMapping {
            base_reputation_weight: base_weight,
            healthcare_reputation_weight: healthcare_weight,
            adjustment_factor,
            last_sync_timestamp: env.ledger().timestamp(),
        };

        env.storage()
            .instance()
            .set(&DataKey::ScoreMapping, &mapping);

        env.events().publish(
            (symbol_short!("REPUTINT"), symbol_short!("MAP_UPD")),
            mapping,
        );
        Ok(())
    }

    // Update sync settings
    pub fn update_sync_settings(
        env: Env,
        admin: Address,
        settings: SyncSettings,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        env.storage()
            .instance()
            .set(&DataKey::SyncSettings, &settings);

        env.events().publish(
            (symbol_short!("REPUTINT"), symbol_short!("SET_UPD")),
            settings,
        );
        Ok(())
    }

    // Get combined reputation score
    pub fn get_combined_score(env: Env, provider: Address) -> Result<i128, Error> {
        Self::require_initialized(&env)?;

        let base_score = Self::get_base_reputation_score(&env, provider.clone())?;
        let healthcare_score = Self::get_healthcare_reputation_score(&env, provider)?;
        Self::calculate_combined_score(&env, base_score, healthcare_score)
    }

    // Get provider sync history
    pub fn get_sync_history(
        env: Env,
        provider: Address,
        limit: u32,
    ) -> Result<Vec<SyncRecord>, Error> {
        Self::require_initialized(&env)?;

        let sync_timestamps: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::ProviderSyncList(provider.clone()))
            .unwrap_or(Vec::new(&env));

        let mut records = Vec::new(&env);
        let mut count = 0;

        // Get most recent records (reverse order)
        for timestamp in sync_timestamps.iter().rev() {
            if count >= limit {
                break;
            }

            if let Some(record) = env
                .storage()
                .persistent()
                .get::<DataKey, SyncRecord>(&DataKey::SyncRecord(provider.clone(), timestamp))
            {
                records.push_back(record);
                count = count.saturating_add(1);
            }
        }

        Ok(records)
    }

    // Trigger sync on credential change
    pub fn trigger_credential_sync(env: Env, provider: Address) -> Result<(), Error> {
        Self::require_initialized(&env)?;

        let settings: SyncSettings = env
            .storage()
            .instance()
            .get(&DataKey::SyncSettings)
            .ok_or(Error::NotInitialized)?;

        if settings.sync_on_credential_change {
            // Get admin address (in real implementation, this might be a system account)
            let admin: Address = env
                .storage()
                .instance()
                .get(&DataKey::Admin)
                .ok_or(Error::NotInitialized)?;

            Self::sync_provider_reputation(env, admin, provider)?;
        }
        Ok(())
    }

    // Trigger sync on feedback change
    pub fn trigger_feedback_sync(env: Env, provider: Address) -> Result<(), Error> {
        Self::require_initialized(&env)?;

        let settings: SyncSettings = env
            .storage()
            .instance()
            .get(&DataKey::SyncSettings)
            .ok_or(Error::NotInitialized)?;

        if settings.sync_on_feedback_change {
            let admin: Address = env
                .storage()
                .instance()
                .get(&DataKey::Admin)
                .ok_or(Error::NotInitialized)?;

            Self::sync_provider_reputation(env, admin, provider)?;
        }

        Ok(())
    }

    // Trigger sync on conduct change
    pub fn trigger_conduct_sync(env: Env, provider: Address) -> Result<(), Error> {
        Self::require_initialized(&env)?;

        let settings: SyncSettings = env
            .storage()
            .instance()
            .get(&DataKey::SyncSettings)
            .ok_or(Error::NotInitialized)?;

        if settings.sync_on_conduct_change {
            let admin: Address = env
                .storage()
                .instance()
                .get(&DataKey::Admin)
                .ok_or(Error::NotInitialized)?;

            Self::sync_provider_reputation(env, admin, provider)?;
        }

        Ok(())
    }

    // Helper functions
    fn get_base_reputation_score(_env: &Env, _provider: Address) -> Result<i128, Error> {
        // Cross-contract call placeholder
        Ok(50)
    }

    fn get_healthcare_reputation_score(_env: &Env, _provider: Address) -> Result<u32, Error> {
        // Cross-contract call placeholder
        Ok(75)
    }

    fn calculate_combined_score(
        env: &Env,
        base_score: i128,
        healthcare_score: u32,
    ) -> Result<i128, Error> {
        let mapping: ScoreMapping = env
            .storage()
            .instance()
            .get(&DataKey::ScoreMapping)
            .ok_or(Error::InvalidScoreMapping)?;

        let base_weighted = base_score
            .checked_mul(mapping.base_reputation_weight as i128)
            .unwrap_or(0)
            .checked_div(100)
            .unwrap_or(0);
        let healthcare_weighted = (healthcare_score as i128)
            .checked_mul(mapping.healthcare_reputation_weight as i128)
            .unwrap_or(0)
            .checked_div(100)
            .unwrap_or(0);

        let combined = base_weighted
            .saturating_add(healthcare_weighted)
            .saturating_add(mapping.adjustment_factor as i128);

        Ok(combined.max(0)) // Ensure non-negative score
    }

    fn update_base_reputation_score(
        env: &Env,
        provider: Address,
        new_score: i128,
    ) -> Result<(), Error> {
        let _contract_address: Address = env
            .storage()
            .instance()
            .get(&DataKey::BaseReputationContract)
            .ok_or(Error::ReputationContractNotFound)?;

        // In a real implementation, you'd make a cross-contract call to update the score
        // For now, we'll just emit an event
        env.events().publish(
            (symbol_short!("REPUTINT"), symbol_short!("BASE_UPD")),
            (provider, new_score),
        );

        Ok(())
    }

    fn store_sync_record(env: &Env, record: SyncRecord) -> Result<(), Error> {
        let provider = record.provider.clone();
        let timestamp = record.timestamp;

        // Store the sync record
        env.storage()
            .persistent()
            .set(&DataKey::SyncRecord(provider.clone(), timestamp), &record);

        // Update provider's sync list
        let mut sync_list: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::ProviderSyncList(provider.clone()))
            .unwrap_or(Vec::new(env));
        sync_list.push_back(timestamp);

        // Keep only last 100 sync records to prevent unlimited growth
        if sync_list.len() > 100 {
            sync_list.remove(0);
        }

        env.storage()
            .persistent()
            .set(&DataKey::ProviderSyncList(provider.clone()), &sync_list);

        // Update last sync time
        env.storage()
            .persistent()
            .set(&DataKey::LastSyncTime(provider), &timestamp);

        Ok(())
    }

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
