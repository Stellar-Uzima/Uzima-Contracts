#![no_std]

mod errors;
mod events;
mod types;

#[cfg(test)]
mod test;

pub use errors::Error;
pub use types::{
    DataKey, InvariantCheck, PermissionCheck, ResourceTracker, StateConsistencyCheck,
    ValidationReport, ViolationType,
};

use soroban_sdk::{
    contract, contractimpl, Address, Env, String, Vec,
};

#[contract]
pub struct RuntimeValidation;

#[contractimpl]
impl RuntimeValidation {
    /// Initialize the runtime validation system
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::ViolationCount, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::CheckCount, &0u32);

        events::publish_initialization(&env, &admin);
        Ok(())
    }

    /// Register an invariant check
    pub fn register_invariant(
        env: Env,
        admin: Address,
        check_id: String,
        description: String,
        severity: u32, // 1=low, 2=medium, 3=high, 4=critical
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        if severity < 1 || severity > 4 {
            return Err(Error::InvalidSeverity);
        }

        if env
            .storage()
            .persistent()
            .has(&DataKey::Invariant(check_id.clone()))
        {
            return Err(Error::CheckAlreadyExists);
        }

        let check = InvariantCheck {
            check_id: check_id.clone(),
            description,
            severity,
            is_active: true,
            created_at: env.ledger().timestamp(),
            violation_count: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Invariant(check_id.clone()), &check);

        let count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::CheckCount)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::CheckCount, &(count + 1));

        events::publish_invariant_registered(&env, &check);
        Ok(())
    }

    /// Register a state consistency check
    pub fn register_state_check(
        env: Env,
        admin: Address,
        check_id: String,
        description: String,
        expected_state: String,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        if env
            .storage()
            .persistent()
            .has(&DataKey::StateCheck(check_id.clone()))
        {
            return Err(Error::CheckAlreadyExists);
        }

        let check = StateConsistencyCheck {
            check_id: check_id.clone(),
            description,
            expected_state,
            is_active: true,
            created_at: env.ledger().timestamp(),
            last_verified: 0,
            violation_count: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::StateCheck(check_id.clone()), &check);

        events::publish_state_check_registered(&env, &check);
        Ok(())
    }

    /// Register a permission check
    pub fn register_permission_check(
        env: Env,
        admin: Address,
        check_id: String,
        description: String,
        required_role: String,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        if env
            .storage()
            .persistent()
            .has(&DataKey::PermissionCheck(check_id.clone()))
        {
            return Err(Error::CheckAlreadyExists);
        }

        let check = PermissionCheck {
            check_id: check_id.clone(),
            description,
            required_role,
            is_active: true,
            created_at: env.ledger().timestamp(),
            violation_count: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::PermissionCheck(check_id.clone()), &check);

        events::publish_permission_check_registered(&env, &check);
        Ok(())
    }

    /// Register a resource tracker
    pub fn register_resource_tracker(
        env: Env,
        admin: Address,
        tracker_id: String,
        resource_type: String,
        max_allocation: i128,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        if max_allocation <= 0 {
            return Err(Error::InvalidResourceLimit);
        }

        if env
            .storage()
            .persistent()
            .has(&DataKey::ResourceTracker(tracker_id.clone()))
        {
            return Err(Error::CheckAlreadyExists);
        }

        let tracker = ResourceTracker {
            tracker_id: tracker_id.clone(),
            resource_type,
            max_allocation,
            current_usage: 0,
            created_at: env.ledger().timestamp(),
            last_updated: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::ResourceTracker(tracker_id.clone()), &tracker);

        events::publish_resource_tracker_registered(&env, &tracker);
        Ok(())
    }

    /// Report a validation violation
    pub fn report_violation(
        env: Env,
        reporter: Address,
        check_id: String,
        violation_type: ViolationType,
        details: String,
    ) -> Result<u64, Error> {
        reporter.require_auth();

        let violation_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::ViolationCount)
            .unwrap_or(0);

        let report = ValidationReport {
            violation_id,
            check_id,
            violation_type,
            reporter,
            details,
            timestamp: env.ledger().timestamp(),
            resolved: false,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Violation(violation_id), &report);

        env.storage()
            .instance()
            .set(&DataKey::ViolationCount, &(violation_id + 1));

        events::publish_violation_reported(&env, &report);
        Ok(violation_id)
    }

    /// Verify an invariant check
    pub fn verify_invariant(
        env: Env,
        check_id: String,
        current_value: i128,
        expected_range_min: i128,
        expected_range_max: i128,
    ) -> Result<bool, Error> {
        let mut check: InvariantCheck = env
            .storage()
            .persistent()
            .get(&DataKey::Invariant(check_id.clone()))
            .ok_or(Error::CheckNotFound)?;

        if !check.is_active {
            return Err(Error::CheckNotActive);
        }

        let is_valid = current_value >= expected_range_min && current_value <= expected_range_max;

        if !is_valid {
            check.violation_count += 1;
            env.storage()
                .persistent()
                .set(&DataKey::Invariant(check_id.clone()), &check);

            events::publish_invariant_violation(&env, &check_id, current_value);
        }

        Ok(is_valid)
    }

    /// Verify state consistency
    pub fn verify_state_consistency(
        env: Env,
        check_id: String,
        current_state: String,
    ) -> Result<bool, Error> {
        let mut check: StateConsistencyCheck = env
            .storage()
            .persistent()
            .get(&DataKey::StateCheck(check_id.clone()))
            .ok_or(Error::CheckNotFound)?;

        if !check.is_active {
            return Err(Error::CheckNotActive);
        }

        let is_consistent = current_state == check.expected_state;

        if !is_consistent {
            check.violation_count += 1;
        }

        check.last_verified = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::StateCheck(check_id.clone()), &check);

        if !is_consistent {
            events::publish_state_violation(&env, &check_id, &current_state);
        }

        Ok(is_consistent)
    }

    /// Check permission
    pub fn verify_permission(
        env: Env,
        check_id: String,
        user_role: String,
    ) -> Result<bool, Error> {
        let check: PermissionCheck = env
            .storage()
            .persistent()
            .get(&DataKey::PermissionCheck(check_id.clone()))
            .ok_or(Error::CheckNotFound)?;

        if !check.is_active {
            return Err(Error::CheckNotActive);
        }

        let has_permission = user_role == check.required_role;

        if !has_permission {
            let mut check_mut = check.clone();
            check_mut.violation_count += 1;
            env.storage()
                .persistent()
                .set(&DataKey::PermissionCheck(check_id.clone()), &check_mut);

            events::publish_permission_violation(&env, &check_id, &user_role);
        }

        Ok(has_permission)
    }

    /// Update resource usage
    pub fn update_resource_usage(
        env: Env,
        tracker_id: String,
        usage_delta: i128,
    ) -> Result<(), Error> {
        let mut tracker: ResourceTracker = env
            .storage()
            .persistent()
            .get(&DataKey::ResourceTracker(tracker_id.clone()))
            .ok_or(Error::CheckNotFound)?;

        let new_usage = tracker.current_usage + usage_delta;

        if new_usage < 0 || new_usage > tracker.max_allocation {
            return Err(Error::ResourceLimitExceeded);
        }

        tracker.current_usage = new_usage;
        tracker.last_updated = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::ResourceTracker(tracker_id.clone()), &tracker);

        events::publish_resource_updated(&env, &tracker);
        Ok(())
    }

    /// Get validation report
    pub fn get_violation_report(env: Env, violation_id: u64) -> Result<ValidationReport, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Violation(violation_id))
            .ok_or(Error::ViolationNotFound)
    }

    /// Get total violations
    pub fn get_violation_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::ViolationCount)
            .unwrap_or(0)
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
