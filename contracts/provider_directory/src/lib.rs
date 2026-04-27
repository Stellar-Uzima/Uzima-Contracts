#![no_std]

mod types;

use soroban_sdk::{contract, contractimpl, Address, Env, String, Symbol, Vec};
use types::{Availability, DataKey, Error, PrivacySettings, ProviderProfile};

#[contract]
pub struct ProviderDirectoryContract;

#[contractimpl]
impl ProviderDirectoryContract {
    /// Initialize the provider directory contract
    pub fn initialize(env: Env, admin: Address, identity_registry: Address) -> Result<(), Error> {
        if env.storage().persistent().has(&DataKey::Initialized) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage()
            .persistent()
            .set(&DataKey::IdentityRegistry, &identity_registry);
        env.storage().persistent().set(&DataKey::Initialized, &true);
        env.storage().persistent().set(&DataKey::Paused, &false);
        env.storage()
            .persistent()
            .set(&DataKey::ProviderList, &Vec::<Address>::new(&env));

        Ok(())
    }

    /// Register or update a provider profile
    pub fn update_profile(
        env: Env,
        provider: Address,
        name: String,
        specialties: Vec<Symbol>,
        bio: String,
        location: String,
        contact_info: String,
    ) -> Result<(), Error> {
        provider.require_auth();
        Self::check_paused(&env)?;

        let existing: Option<ProviderProfile> = env
            .storage()
            .persistent()
            .get::<DataKey, ProviderProfile>(&DataKey::Profile(provider.clone()));

        let mut profile = if let Some(prev) = existing {
            // For added specialties, append provider to the index.
            // Stale entries for removed specialties are tolerated (search validates against profile).
            for s in specialties.iter() {
                if !prev.specialties.contains(s.clone()) {
                    Self::index_specialty(&env, &s, &provider);
                }
            }
            prev
        } else {
            // New provider: register in ProviderList and index all specialties
            let mut list = env
                .storage()
                .persistent()
                .get::<DataKey, Vec<Address>>(&DataKey::ProviderList)
                .unwrap_or(Vec::new(&env));
            list.push_back(provider.clone());
            env.storage()
                .persistent()
                .set(&DataKey::ProviderList, &list);

            for s in specialties.iter() {
                Self::index_specialty(&env, &s, &provider);
            }

            ProviderProfile {
                address: provider.clone(),
                name: name.clone(),
                specialties: specialties.clone(),
                bio: bio.clone(),
                location: location.clone(),
                contact_info: contact_info.clone(),
                is_verified: false,
                reputation_score: 0,
                joining_timestamp: env.ledger().timestamp(),
            }
        };

        profile.name = name;
        profile.specialties = specialties;
        profile.bio = bio;
        profile.location = location;
        profile.contact_info = contact_info;

        env.storage()
            .persistent()
            .set(&DataKey::Profile(provider), &profile);
        Ok(())
    }

    /// Get a provider profile
    pub fn get_profile(env: Env, provider: Address) -> Result<ProviderProfile, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Profile(provider))
            .ok_or(Error::ProfileNotFound)
    }

    /// Set provider availability
    pub fn set_availability(
        env: Env,
        provider: Address,
        availability: Vec<Availability>,
    ) -> Result<(), Error> {
        provider.require_auth();
        Self::check_paused(&env)?;

        if !env
            .storage()
            .persistent()
            .has(&DataKey::Profile(provider.clone()))
        {
            return Err(Error::ProfileNotFound);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Availability(provider), &availability);
        Ok(())
    }

    /// Get provider availability
    pub fn get_availability(env: Env, provider: Address) -> Result<Vec<Availability>, Error> {
        Ok(env
            .storage()
            .persistent()
            .get(&DataKey::Availability(provider))
            .unwrap_or_else(|| Vec::new(&env)))
    }

    /// Update privacy settings
    pub fn update_privacy(
        env: Env,
        provider: Address,
        settings: PrivacySettings,
    ) -> Result<(), Error> {
        provider.require_auth();
        Self::check_paused(&env)?;

        if !env
            .storage()
            .persistent()
            .has(&DataKey::Profile(provider.clone()))
        {
            return Err(Error::ProfileNotFound);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Privacy(provider), &settings);
        Ok(())
    }

    /// Search providers by specialty using the specialty index for O(M) reads instead of O(N)
    pub fn search_by_specialty(env: Env, specialty: Symbol) -> Vec<ProviderProfile> {
        let candidates: Vec<Address> = env
            .storage()
            .persistent()
            .get::<DataKey, Vec<Address>>(&DataKey::SpecialtyProviders(specialty.clone()))
            .unwrap_or(Vec::new(&env));

        let mut results = Vec::new(&env);
        for addr in candidates.iter() {
            if let Some(profile) = env
                .storage()
                .persistent()
                .get::<DataKey, ProviderProfile>(&DataKey::Profile(addr))
            {
                // Validate against live profile to handle stale index entries
                if profile.specialties.contains(specialty.clone()) {
                    results.push_back(profile);
                }
            }
        }
        results
    }

    /// Verify a provider (Admin only)
    pub fn verify_provider(env: Env, admin: Address, provider: Address) -> Result<(), Error> {
        admin.require_auth();
        let stored_admin: Address = env
            .storage()
            .persistent()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if admin != stored_admin {
            return Err(Error::NotAuthorized);
        }

        let mut profile = env
            .storage()
            .persistent()
            .get::<DataKey, ProviderProfile>(&DataKey::Profile(provider.clone()))
            .ok_or(Error::ProfileNotFound)?;

        profile.is_verified = true;
        env.storage()
            .persistent()
            .set(&DataKey::Profile(provider), &profile);
        Ok(())
    }

    /// Batch verify multiple providers (Admin only). Reads admin once for the whole batch.
    pub fn batch_verify_providers(
        env: Env,
        admin: Address,
        providers: Vec<Address>,
    ) -> Result<u32, Error> {
        admin.require_auth();
        let stored_admin: Address = env
            .storage()
            .persistent()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if admin != stored_admin {
            return Err(Error::NotAuthorized);
        }

        let mut verified_count: u32 = 0;
        for provider in providers.iter() {
            if let Some(mut profile) = env
                .storage()
                .persistent()
                .get::<DataKey, ProviderProfile>(&DataKey::Profile(provider.clone()))
            {
                if !profile.is_verified {
                    profile.is_verified = true;
                    env.storage()
                        .persistent()
                        .set(&DataKey::Profile(provider), &profile);
                    verified_count = verified_count.saturating_add(1);
                }
            }
        }
        Ok(verified_count)
    }

    /// Append provider to specialty index (deduplicated check skipped for gas; search validates)
    fn index_specialty(env: &Env, specialty: &Symbol, provider: &Address) {
        let key = DataKey::SpecialtyProviders(specialty.clone());
        let mut list: Vec<Address> = env
            .storage()
            .persistent()
            .get::<DataKey, Vec<Address>>(&key)
            .unwrap_or(Vec::new(env));
        list.push_back(provider.clone());
        env.storage().persistent().set(&key, &list);
    }

    /// Private helper to check if contract is paused
    fn check_paused(env: &Env) -> Result<(), Error> {
        let paused = env
            .storage()
            .persistent()
            .get::<DataKey, bool>(&DataKey::Paused)
            .unwrap_or(false);
        if paused {
            return Err(Error::ContractPaused);
        }
        Ok(())
    }
}
