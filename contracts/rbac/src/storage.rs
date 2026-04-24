use crate::types::{AddressRoles, DataKey, RBACConfig, Role, RoleAssignment};
use soroban_sdk::{Address, Env, Vec};

/// Storage operations for RBAC contract
pub struct Storage;

impl Storage {
    /// Get the next assignment ID
    pub fn get_next_assignment_id(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::AssignmentCount)
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::AssignmentCount, &(count + 1));
        count
    }

    /// Get all roles for an address
    pub fn get_address_roles(env: &Env, address: &Address) -> Vec<Role> {
        env.storage()
            .persistent()
            .get(&DataKey::AddressRoles(address.clone()))
            .unwrap_or_else(|| Vec::new(env))
    }

    /// Check if an address has a specific role
    pub fn has_role(env: &Env, address: &Address, role: Role) -> bool {
        let roles = Self::get_address_roles(env, address);
        roles.iter().any(|r| r == role)
    }

    /// Add a role to an address
    pub fn add_role(env: &Env, address: &Address, role: Role) -> bool {
        let mut roles = Self::get_address_roles(env, address);

        // Check if role already exists
        if roles.iter().any(|r| r == role) {
            return false;
        }

        // Get max roles per address
        let max_roles = Self::get_config(env)
            .map(|c| c.max_roles_per_address)
            .unwrap_or(10);

        // Check if we've reached the limit
        if roles.len() as u32 >= max_roles {
            return false;
        }

        roles.push_back(role);
        env.storage()
            .persistent()
            .set(&DataKey::AddressRoles(address.clone()), &roles);

        // Add address to role members list
        let mut members: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::RoleMembers(role as u32))
            .unwrap_or_else(|| Vec::new(env));

        if !members.iter().any(|a| a == *address) {
            members.push_back(address.clone());
            env.storage()
                .persistent()
                .set(&DataKey::RoleMembers(role as u32), &members);
        }

        true
    }

    /// Remove a role from an address
    pub fn remove_role(env: &Env, address: &Address, role: Role) -> bool {
        let mut roles = Self::get_address_roles(env, address);

        // Find and remove the role
        let mut index_to_remove: Option<u32> = None;
        for (i, r) in roles.iter().enumerate() {
            if r == role {
                index_to_remove = Some(i as u32);
                break;
            }
        }

        if let Some(index) = index_to_remove {
            // Create new vec without the removed role
            let mut new_roles: Vec<Role> = Vec::new(env);
            for (i, r) in roles.iter().enumerate() {
                if i as u32 != index {
                    new_roles.push_back(r);
                }
            }

            env.storage()
                .persistent()
                .set(&DataKey::AddressRoles(address.clone()), &new_roles);

            // Remove address from role members list
            let mut members: Vec<Address> = env
                .storage()
                .persistent()
                .get(&DataKey::RoleMembers(role as u32))
                .unwrap_or_else(|| Vec::new(env));

            let mut member_index_to_remove: Option<u32> = None;
            for (i, m) in members.iter().enumerate() {
                if m == *address {
                    member_index_to_remove = Some(i as u32);
                    break;
                }
            }

            if let Some(idx) = member_index_to_remove {
                let mut new_members: Vec<Address> = Vec::new(env);
                for (i, m) in members.iter().enumerate() {
                    if i as u32 != idx {
                        new_members.push_back(m);
                    }
                }
                env.storage()
                    .persistent()
                    .set(&DataKey::RoleMembers(role as u32), &new_members);
            }

            true
        } else {
            false
        }
    }

    /// Get all addresses with a specific role
    pub fn get_role_members(env: &Env, role: Role) -> Vec<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::RoleMembers(role as u32))
            .unwrap_or_else(|| Vec::new(env))
    }

    /// Get count of members with a role
    pub fn get_role_member_count(env: &Env, role: Role) -> u32 {
        Self::get_role_members(env, role).len() as u32
    }

    /// Save a role assignment record
    pub fn save_assignment(env: &Env, assignment: &RoleAssignment) {
        let id = Self::get_next_assignment_id(env);
        env.storage()
            .persistent()
            .set(&DataKey::Assignment(id), assignment);
    }

    /// Get a role assignment record
    pub fn get_assignment(env: &Env, id: u64) -> Option<RoleAssignment> {
        env.storage().persistent().get(&DataKey::Assignment(id))
    }

    /// Get admin address
    pub fn get_admin(env: &Env) -> Address {
        env.storage()
            .persistent()
            .get(&DataKey::Admin)
            .expect("Admin not set")
    }

    /// Set admin address
    pub fn set_admin(env: &Env, admin: &Address) {
        env.storage().persistent().set(&DataKey::Admin, admin);
    }

    /// Get RBAC configuration
    pub fn get_config(env: &Env) -> Option<RBACConfig> {
        env.storage().persistent().get(&DataKey::Config)
    }

    /// Set RBAC configuration
    pub fn set_config(env: &Env, config: &RBACConfig) {
        env.storage().persistent().set(&DataKey::Config, config);
    }

    /// Check if contract is initialized
    pub fn is_initialized(env: &Env) -> bool {
        env.storage()
            .persistent()
            .get::<_, bool>(&DataKey::IsInitialized)
            .unwrap_or(false)
    }

    /// Mark contract as initialized
    pub fn set_initialized(env: &Env) {
        env.storage()
            .persistent()
            .set(&DataKey::IsInitialized, &true);
    }
}
