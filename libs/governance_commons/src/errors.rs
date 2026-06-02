//! Error types for governance contracts

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum GovernanceError {
    /// Contract has already been initialized
    AlreadyInitialized = 1,
    /// Contract is not initialized
    NotInitialized = 2,
    /// Caller is not authorized
    NotAuthorized = 3,
    /// Caller is not in the approver set
    NotApprover = 4,
    /// Insufficient approvals for the operation
    InsufficientApprovals = 5,
    /// Threshold is invalid (0 or > count)
    InvalidThreshold = 6,
    /// Item not found
    NotFound = 7,
    /// Operation failed (generic)
    OperationFailed = 8,
    /// Duplicate entry
    DuplicateEntry = 9,
    /// Input validation failed
    InvalidInput = 10,
}

impl GovernanceError {
    pub fn as_u32(self) -> u32 {
        self as u32
    }
}
