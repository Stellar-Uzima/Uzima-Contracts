#![no_std]

mod errors;
mod events;
mod types;

#[cfg(test)]
mod test;

pub use errors::Error;
pub use types::{DataKey, ModuleOwnership, OwnershipMatrix, ReviewRoute};

use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};

#[contract]
pub struct CodeOwnership;

#[contractimpl]
impl CodeOwnership {
    /// Initialize the code ownership tracking system
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::ModuleCount, &0u32);

        events::publish_initialization(&env, &admin);
        Ok(())
    }

    /// Register a module with ownership information
    pub fn register_module(
        env: Env,
        admin: Address,
        module_id: String,
        module_name: String,
        primary_owner: Address,
        secondary_owners: Vec<Address>,
        expertise_areas: Vec<String>,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        if env
            .storage()
            .persistent()
            .has(&DataKey::Module(module_id.clone()))
        {
            return Err(Error::ModuleAlreadyExists);
        }

        let ownership = ModuleOwnership {
            module_id: module_id.clone(),
            module_name,
            primary_owner: primary_owner.clone(),
            secondary_owners,
            expertise_areas,
            registered_at: env.ledger().timestamp(),
            last_updated: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::Module(module_id.clone()), &ownership);

        // Update module count
        let count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::ModuleCount)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::ModuleCount, &(count + 1));

        events::publish_module_registered(&env, &ownership);
        Ok(())
    }

    /// Update module ownership
    pub fn update_module_ownership(
        env: Env,
        admin: Address,
        module_id: String,
        new_primary_owner: Address,
        new_secondary_owners: Vec<Address>,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let mut ownership: ModuleOwnership = env
            .storage()
            .persistent()
            .get(&DataKey::Module(module_id.clone()))
            .ok_or(Error::ModuleNotFound)?;

        ownership.primary_owner = new_primary_owner;
        ownership.secondary_owners = new_secondary_owners;
        ownership.last_updated = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::Module(module_id.clone()), &ownership);

        events::publish_ownership_updated(&env, &ownership);
        Ok(())
    }

    /// Configure review routing for a module
    pub fn configure_review_route(
        env: Env,
        admin: Address,
        module_id: String,
        required_reviewers: u32,
        escalation_threshold: u32,
        escalation_owner: Address,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        // Verify module exists
        let _ownership: ModuleOwnership = env
            .storage()
            .persistent()
            .get(&DataKey::Module(module_id.clone()))
            .ok_or(Error::ModuleNotFound)?;

        let route = ReviewRoute {
            module_id: module_id.clone(),
            required_reviewers,
            escalation_threshold,
            escalation_owner,
            created_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::ReviewRoute(module_id.clone()), &route);

        events::publish_review_route_configured(&env, &route);
        Ok(())
    }

    /// Get module ownership information
    pub fn get_module_ownership(env: Env, module_id: String) -> Result<ModuleOwnership, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Module(module_id))
            .ok_or(Error::ModuleNotFound)
    }

    /// Get review routing for a module
    pub fn get_review_route(env: Env, module_id: String) -> Result<ReviewRoute, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::ReviewRoute(module_id))
            .ok_or(Error::ReviewRouteNotFound)
    }

    /// Get expertise matrix for all modules
    pub fn get_expertise_matrix(env: Env) -> OwnershipMatrix {
        let count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::ModuleCount)
            .unwrap_or(0);

        let mut modules: Vec<ModuleOwnership> = Vec::new(&env);

        // Note: In production, this would need pagination for large datasets
        // For now, we iterate through stored modules
        for i in 0..count {
            // This is a simplified approach; real implementation would need better indexing
            if let Some(module) = env
                .storage()
                .persistent()
                .get::<DataKey, ModuleOwnership>(&DataKey::ModuleIndex(i))
            {
                modules.push_back(module);
            }
        }

        OwnershipMatrix {
            total_modules: count,
            modules,
            generated_at: env.ledger().timestamp(),
        }
    }

    /// Check if an address is an owner of a module
    pub fn is_module_owner(env: Env, module_id: String, address: Address) -> Result<bool, Error> {
        let ownership: ModuleOwnership = env
            .storage()
            .persistent()
            .get(&DataKey::Module(module_id))
            .ok_or(Error::ModuleNotFound)?;

        Ok(ownership.primary_owner == address
            || ownership.secondary_owners.iter().any(|o| o == address))
    }

    /// Get all modules owned by an address
    pub fn get_owned_modules(env: Env, owner: Address) -> Vec<String> {
        let count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::ModuleCount)
            .unwrap_or(0);

        let mut owned_modules: Vec<String> = Vec::new(&env);

        for i in 0..count {
            if let Some(module) = env
                .storage()
                .persistent()
                .get::<DataKey, ModuleOwnership>(&DataKey::ModuleIndex(i))
            {
                if module.primary_owner == owner
                    || module.secondary_owners.iter().any(|o| o == owner)
                {
                    owned_modules.push_back(module.module_id);
                }
            }
        }

        owned_modules
    }

    fn require_admin(env: &Env, actor: &Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;

        if admin != *actor {
            return Err(Error::NotAuthorized);
        }

        Ok(())
    }
}
