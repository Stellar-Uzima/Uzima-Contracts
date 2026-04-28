#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, Env, String, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    RateLimitExceeded = 3,
    NotAuthorized = 4,
}

#[contracttype]
pub enum DataKey {
    Admin,
    RateLimitConfig,
    SearchRateLimit(Address),
    ExemptInstitution(Address),
}

#[contracttype]
pub struct RateLimitConfig {
    pub max_searches: u32,
    pub window_secs: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct Provider {
    pub id: Address,
    pub name: String,
    pub specialty: String,
}

#[contract]
pub struct ProviderDirectoryContract;

#[contractimpl]
impl ProviderDirectoryContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        
        // Set default rate limit: 10 searches per hour (3600 seconds)
        let default_config = RateLimitConfig {
            max_searches: 10,
            window_secs: 3600,
        };
        env.storage().instance().set(&DataKey::RateLimitConfig, &default_config);
        
        Ok(())
    }

    pub fn set_rate_limit_config(env: Env, admin: Address, max_searches: u32, window_secs: u64) -> Result<(), Error> {
        admin.require_auth();
        let current_admin: Address = env.storage().instance().get(&DataKey::Admin).ok_or(Error::NotInitialized)?;
        if current_admin != admin {
            return Err(Error::NotAuthorized);
        }
        
        let config = RateLimitConfig {
            max_searches,
            window_secs,
        };
        env.storage().instance().set(&DataKey::RateLimitConfig, &config);
        Ok(())
    }

    pub fn set_institution_exemption(env: Env, admin: Address, institution: Address, is_exempt: bool) -> Result<(), Error> {
        admin.require_auth();
        let current_admin: Address = env.storage().instance().get(&DataKey::Admin).ok_or(Error::NotInitialized)?;
        if current_admin != admin {
            return Err(Error::NotAuthorized);
        }
        
        if is_exempt {
            env.storage().persistent().set(&DataKey::ExemptInstitution(institution), &true);
        } else {
            env.storage().persistent().remove(&DataKey::ExemptInstitution(institution));
        }
        Ok(())
    }

    pub fn search_providers(env: Env, caller: Address, _query: String) -> Result<Vec<Provider>, Error> {
        caller.require_auth();
        
        // Enforce sliding window rate limits (throws Error::RateLimitExceeded)
        Self::check_search_rate_limit(&env, &caller)?;

        // TODO: Implement actual provider search matching the _query
        Ok(Vec::new(&env))
    }
    
    fn check_search_rate_limit(env: &Env, caller: &Address) -> Result<(), Error> {
        let is_exempt: bool = env.storage().persistent().get(&DataKey::ExemptInstitution(caller.clone())).unwrap_or(false);
        if is_exempt { return Ok(()); }

        let config: RateLimitConfig = env.storage().instance().get(&DataKey::RateLimitConfig).unwrap_or(RateLimitConfig { max_searches: 10, window_secs: 3600 });
        let now = env.ledger().timestamp();
        let timestamps: Vec<u64> = env.storage().persistent().get(&DataKey::SearchRateLimit(caller.clone())).unwrap_or(Vec::new(env));
        let mut active_timestamps = Vec::new(env);
        for ts in timestamps.iter() {
            if now.saturating_sub(ts) < config.window_secs { active_timestamps.push_back(ts); }
        }

        if active_timestamps.len() >= config.max_searches { return Err(Error::RateLimitExceeded); }
        active_timestamps.push_back(now);
        env.storage().persistent().set(&DataKey::SearchRateLimit(caller.clone()), &active_timestamps);
        Ok(())
    }
}
