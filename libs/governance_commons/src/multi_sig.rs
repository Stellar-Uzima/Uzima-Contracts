//! Shared multi-sig approval logic for governance contracts
//!
//! This module consolidates multi-sig approval patterns used across:
//! - UpgradeManager (validator approvals for upgrades)
//! - EmergencyAccessOverride (approver multi-sig for emergency access)
//! - Future governance contracts needing multi-sig
//!
//! # Pattern
//!
//! 1. Initialize with approval set (members + threshold)
//! 2. Members submit approvals for items
//! 3. Once threshold is reached, item is ready for execution
//! 4. Mark as executed once action is taken

use soroban_sdk::{Address, Env, Vec};
use crate::errors::GovernanceError;
use crate::types::{ApprovalRecord, ApprovalStatus};

/// Validates that an approval set configuration is valid
///
/// # Arguments
/// * `members` - Vector of approved members
/// * `threshold` - Number of approvals required
///
/// # Returns
/// `Ok(())` if valid, `Err` if threshold is invalid
pub fn validate_approval_set(
    members: &Vec<Address>,
    threshold: u32,
) -> Result<(), GovernanceError> {
    if threshold == 0 {
        return Err(GovernanceError::InvalidThreshold);
    }
    if threshold as usize > members.len() {
        return Err(GovernanceError::InvalidThreshold);
    }
    Ok(())
}

/// Validates that an address is in the approval set
///
/// # Arguments
/// * `approver` - Address to check
/// * `members` - Vector of approved members
///
/// # Returns
/// `Ok(())` if approver is in the set, `Err(NotApprover)` otherwise
pub fn validate_approver(
    approver: &Address,
    members: &Vec<Address>,
) -> Result<(), GovernanceError> {
    if !members.contains(approver) {
        return Err(GovernanceError::NotApprover);
    }
    Ok(())
}

/// Checks if an address is already in a list (for deduplication)
///
/// # Arguments
/// * `address` - Address to check
/// * `list` - Vector of addresses
///
/// # Returns
/// `true` if address is in the list, `false` otherwise
pub fn is_already_approved(address: &Address, list: &Vec<Address>) -> bool {
    list.contains(address)
}

/// Adds an approval if not already present
///
/// # Arguments
/// * `env` - Soroban environment
/// * `approver` - Address to add
/// * `approvers` - Mutable vector of approvers
///
/// # Returns
/// `true` if approval was added, `false` if already approved
pub fn add_approval(
    env: &Env,
    approver: Address,
    approvers: &mut Vec<Address>,
) -> bool {
    if is_already_approved(&approver, approvers) {
        return false; // Already approved
    }
    approvers.push_back(approver);
    true // Newly added
}

/// Checks the current approval status
///
/// # Arguments
/// * `approvers` - Vector of addresses that have approved
/// * `threshold` - Required number of approvals
/// * `executed` - Whether the item has been executed
///
/// # Returns
/// Current `ApprovalStatus`
pub fn check_approval_status(
    approvers: &Vec<Address>,
    threshold: u32,
    executed: bool,
) -> ApprovalStatus {
    if executed {
        return ApprovalStatus::Executed;
    }

    let approval_count = approvers.len() as u32;
    if approval_count >= threshold {
        ApprovalStatus::Ready
    } else {
        ApprovalStatus::Pending
    }
}

/// Formats approval progress for events/logging
///
/// Example: "2/3 approvals"
///
/// # Arguments
/// * `current` - Current approval count
/// * `required` - Required approval threshold
///
/// # Returns
/// Formatted string (note: returns a simple numeric representation)
pub fn format_approval_progress(current: u32, required: u32) -> u32 {
    // In Soroban, we can't easily format strings
    // Return a packed representation: (current << 16) | required
    (current << 16) | required
}

/// Validates that an approval record is unique (no duplicates)
///
/// This is useful when storing multiple approval records
pub fn validate_unique_approval(
    item_id: u64,
    existing_ids: &Vec<u64>,
) -> Result<(), GovernanceError> {
    if existing_ids.contains(&item_id) {
        return Err(GovernanceError::DuplicateEntry);
    }
    Ok(())
}
