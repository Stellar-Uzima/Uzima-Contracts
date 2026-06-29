//! Common types for governance contracts

use soroban_sdk::{contracttype, Address, Vec};

/// Represents a multi-sig approval record
#[derive(Clone)]
#[contracttype]
pub struct ApprovalRecord {
    /// The item being approved (e.g., proposal_id, upgrade_id, etc.)
    pub item_id: u64,
    /// Addresses that have approved
    pub approvers: Vec<Address>,
    /// Required number of approvals
    pub threshold: u32,
    /// Whether this item has been executed/finalized
    pub executed: bool,
    /// Timestamp when approval was created
    pub created_at: u64,
}

/// Result of checking approval status
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ApprovalStatus {
    /// Not enough approvals yet
    Pending = 0,
    /// Threshold reached, ready to execute
    Ready = 1,
    /// Already executed
    Executed = 2,
    /// Not found
    NotFound = 3,
}

/// Information about an approval set (validators, arbiters, approvers, etc.)
#[derive(Clone)]
#[contracttype]
pub struct ApprovalSetConfig {
    /// Members of the approval set
    pub members: Vec<Address>,
    /// Number of approvals required
    pub threshold: u32,
}
