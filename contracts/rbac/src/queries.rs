use crate::storage::Storage;
use crate::types::{AddressRoles, Role};
use soroban_sdk::{Address, Env, Vec};

/// Query operations for RBAC contract
pub struct Queries;

impl Queries {
    /// Get all roles for an address
    pub fn get_roles(env: &Env, address: &Address) -> Vec<Role> {
        Storage::get_address_roles(env, address)
    }

    /// Check if an address has any of the specified roles
    pub fn has_any_role(env: &Env, address: &Address, roles: &Vec<Role>) -> bool {
        let address_roles = Storage::get_address_roles(env, address);
        roles.iter().any(|r| address_roles.iter().any(|ar| ar == r))
    }

    /// Check if an address has all of the specified roles
    pub fn has_all_roles(env: &Env, address: &Address, roles: &Vec<Role>) -> bool {
        let address_roles = Storage::get_address_roles(env, address);
        roles.iter().all(|r| address_roles.iter().any(|ar| ar == r))
    }

    /// Get role information for an address
    pub fn get_address_role_info(env: &Env, address: &Address) -> AddressRoles {
        let roles = Storage::get_address_roles(env, address);
        let role_count = roles.len() as u32;

        AddressRoles {
            address: address.clone(),
            roles,
            role_count,
        }
    }

    /// Get all members of a role
    pub fn get_role_members(env: &Env, role: Role) -> Vec<Address> {
        Storage::get_role_members(env, role)
    }

    /// Get count of members in a role
    pub fn get_role_member_count(env: &Env, role: Role) -> u32 {
        Storage::get_role_member_count(env, role)
    }

    /// Check if an address is an admin
    pub fn is_admin(env: &Env, address: &Address) -> bool {
        Storage::has_role(env, address, Role::Admin)
    }

    /// Check if an address is a doctor
    pub fn is_doctor(env: &Env, address: &Address) -> bool {
        Storage::has_role(env, address, Role::Doctor)
    }

    /// Check if an address is a patient
    pub fn is_patient(env: &Env, address: &Address) -> bool {
        Storage::has_role(env, address, Role::Patient)
    }

    /// Check if an address is staff
    pub fn is_staff(env: &Env, address: &Address) -> bool {
        Storage::has_role(env, address, Role::Staff)
    }

    /// Check if an address is an insurer
    pub fn is_insurer(env: &Env, address: &Address) -> bool {
        Storage::has_role(env, address, Role::Insurer)
    }

    /// Check if an address is a researcher
    pub fn is_researcher(env: &Env, address: &Address) -> bool {
        Storage::has_role(env, address, Role::Researcher)
    }

    /// Check if an address is an auditor
    pub fn is_auditor(env: &Env, address: &Address) -> bool {
        Storage::has_role(env, address, Role::Auditor)
    }

    /// Check if an address is a service account
    pub fn is_service(env: &Env, address: &Address) -> bool {
        Storage::has_role(env, address, Role::Service)
    }
}
