#![no_std]

mod errors;
mod events;
mod types;

#[cfg(test)]
mod test;

pub use errors::Error;
pub use types::{DataKey, DeprecationPhase, DeprecationStatus, MigrationGuide, SunsetTimeline};

use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};

#[contract]
pub struct DeprecationFramework;

#[contractimpl]
impl DeprecationFramework {
    /// Initialize the deprecation framework
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::ContractCount, &0u32);

        events::publish_initialization(&env, &admin);
        Ok(())
    }

    /// Mark a contract for deprecation
    pub fn mark_for_deprecation(
        env: Env,
        admin: Address,
        contract_id: String,
        contract_name: String,
        reason: String,
        replacement_contract: Option<String>,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        if env
            .storage()
            .persistent()
            .has(&DataKey::DeprecatedContract(contract_id.clone()))
        {
            return Err(Error::ContractAlreadyDeprecated);
        }

        let status = DeprecationStatus {
            contract_id: contract_id.clone(),
            contract_name,
            reason,
            replacement_contract,
            phase: DeprecationPhase::Announced,
            marked_at: env.ledger().timestamp(),
            last_updated: env.ledger().timestamp(),
            is_active: true,
        };

        env.storage()
            .persistent()
            .set(&DataKey::DeprecatedContract(contract_id.clone()), &status);

        let count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::ContractCount)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::ContractCount, &(count + 1));

        events::publish_deprecation_marked(&env, &status);
        Ok(())
    }

    /// Set sunset timeline for a contract
    pub fn set_sunset_timeline(
        env: Env,
        admin: Address,
        contract_id: String,
        announcement_date: u64,
        support_end_date: u64,
        removal_date: u64,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        // Validate dates are in chronological order
        if announcement_date >= support_end_date || support_end_date >= removal_date {
            return Err(Error::InvalidTimeline);
        }

        let _status: DeprecationStatus = env
            .storage()
            .persistent()
            .get(&DataKey::DeprecatedContract(contract_id.clone()))
            .ok_or(Error::ContractNotFound)?;

        let timeline = SunsetTimeline {
            contract_id: contract_id.clone(),
            announcement_date,
            support_end_date,
            removal_date,
            created_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::SunsetTimeline(contract_id.clone()), &timeline);

        events::publish_sunset_timeline_set(&env, &timeline);
        Ok(())
    }

    /// Add migration guidance
    pub fn add_migration_guide(
        env: Env,
        admin: Address,
        contract_id: String,
        guide_title: String,
        guide_content: String,
        code_examples: Vec<String>,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let _status: DeprecationStatus = env
            .storage()
            .persistent()
            .get(&DataKey::DeprecatedContract(contract_id.clone()))
            .ok_or(Error::ContractNotFound)?;

        let guide = MigrationGuide {
            contract_id: contract_id.clone(),
            guide_title,
            guide_content,
            code_examples,
            created_at: env.ledger().timestamp(),
            last_updated: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::MigrationGuide(contract_id.clone()), &guide);

        events::publish_migration_guide_added(&env, &guide);
        Ok(())
    }

    /// Update deprecation phase
    pub fn update_deprecation_phase(
        env: Env,
        admin: Address,
        contract_id: String,
        new_phase: DeprecationPhase,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let mut status: DeprecationStatus = env
            .storage()
            .persistent()
            .get(&DataKey::DeprecatedContract(contract_id.clone()))
            .ok_or(Error::ContractNotFound)?;

        // Validate phase progression
        let current_phase_value = Self::phase_to_value(&status.phase);
        let new_phase_value = Self::phase_to_value(&new_phase);

        if new_phase_value < current_phase_value {
            return Err(Error::InvalidPhaseTransition);
        }

        status.phase = new_phase;
        status.last_updated = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::DeprecatedContract(contract_id.clone()), &status);

        events::publish_phase_updated(&env, &status);
        Ok(())
    }

    /// Publish user communication
    pub fn publish_user_communication(
        env: Env,
        admin: Address,
        contract_id: String,
        message: String,
        communication_type: String, // "email", "notification", "announcement"
    ) -> Result<u64, Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let _status: DeprecationStatus = env
            .storage()
            .persistent()
            .get(&DataKey::DeprecatedContract(contract_id.clone()))
            .ok_or(Error::ContractNotFound)?;

        let comm_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::CommunicationCount)
            .unwrap_or(0);

        env.storage()
            .instance()
            .set(&DataKey::CommunicationCount, &(comm_id + 1));

        env.storage().persistent().set(
            &DataKey::Communication(comm_id),
            &(
                contract_id.clone(),
                message.clone(),
                communication_type,
                env.ledger().timestamp(),
            ),
        );

        events::publish_communication_sent(&env, &contract_id, comm_id);
        Ok(comm_id)
    }

    /// Create removal checklist
    pub fn create_removal_checklist(
        env: Env,
        admin: Address,
        contract_id: String,
        checklist_items: Vec<String>,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let _status: DeprecationStatus = env
            .storage()
            .persistent()
            .get(&DataKey::DeprecatedContract(contract_id.clone()))
            .ok_or(Error::ContractNotFound)?;

        env.storage().persistent().set(
            &DataKey::RemovalChecklist(contract_id.clone()),
            &checklist_items,
        );

        events::publish_removal_checklist_created(&env, &contract_id);
        Ok(())
    }

    /// Mark checklist item as complete
    pub fn mark_checklist_item_complete(
        env: Env,
        admin: Address,
        contract_id: String,
        item_index: u32,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let _checklist: Vec<String> = env
            .storage()
            .persistent()
            .get(&DataKey::RemovalChecklist(contract_id.clone()))
            .ok_or(Error::ChecklistNotFound)?;

        // Store completion status separately
        env.storage().persistent().set(
            &DataKey::ChecklistItemComplete(contract_id.clone(), item_index),
            &true,
        );

        events::publish_checklist_item_completed(&env, &contract_id, item_index);
        Ok(())
    }

    /// Get deprecation status
    pub fn get_deprecation_status(
        env: Env,
        contract_id: String,
    ) -> Result<DeprecationStatus, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::DeprecatedContract(contract_id))
            .ok_or(Error::ContractNotFound)
    }

    /// Get sunset timeline
    pub fn get_sunset_timeline(env: Env, contract_id: String) -> Result<SunsetTimeline, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::SunsetTimeline(contract_id))
            .ok_or(Error::TimelineNotFound)
    }

    /// Get migration guide
    pub fn get_migration_guide(env: Env, contract_id: String) -> Result<MigrationGuide, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::MigrationGuide(contract_id))
            .ok_or(Error::GuideNotFound)
    }

    /// Check if contract is deprecated
    pub fn is_deprecated(env: Env, contract_id: String) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::DeprecatedContract(contract_id))
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

    fn phase_to_value(phase: &DeprecationPhase) -> u32 {
        match phase {
            DeprecationPhase::Announced => 1,
            DeprecationPhase::Supported => 2,
            DeprecationPhase::Sunset => 3,
            DeprecationPhase::Removed => 4,
        }
    }
}
