#![no_std]

//! # Soft Delete Library
//!
//! Provides soft-delete and archival primitives for Uzima contracts.
//! Records can transition between Active, SoftDeleted, and Archived states
//! with full audit metadata.
//!
//! ## Record Lifecycle
//!
//! ```text
//!   Active ──soft_delete()──> SoftDeleted
//!     │                           │
//!     │ restore()                 │ archive()
//!     │                           │
//!     └─────── archive() ─────> Archived
//! ```

use soroban_sdk::{contracterror, contracttype, Address, String};

/// Current lifecycle status of a record.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub enum RecordStatus {
    Active = 0,
    SoftDeleted = 1,
    Archived = 2,
}

/// Error types for soft-delete operations.
#[contracterror]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum SoftDeleteError {
    AlreadyDeleted = 1,
    AlreadyArchived = 2,
    InvalidStateTransition = 3,
    Unauthorized = 4,
    RetentionExpired = 5,
}

impl core::fmt::Display for SoftDeleteError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            SoftDeleteError::AlreadyDeleted => write!(f, "record already deleted"),
            SoftDeleteError::AlreadyArchived => write!(f, "record already archived"),
            SoftDeleteError::InvalidStateTransition => write!(f, "invalid state transition"),
            SoftDeleteError::Unauthorized => write!(f, "unauthorized"),
            SoftDeleteError::RetentionExpired => write!(f, "retention period expired"),
        }
    }
}

/// Metadata attached when a record is soft-deleted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub struct DeletionMetadata {
    pub deleted_by: Address,
    pub deleted_at: u64,
    pub reason: String,
    pub retention_days: u32,
}

/// Policy defining when records should be auto-archived.
#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub struct ArchivePolicy {
    pub auto_archive_after_secs: u64,
    pub auto_purge_after_secs: u64,
}

impl ArchivePolicy {
    pub fn after_days(archive_days: u32, purge_days: u32) -> Self {
        Self {
            auto_archive_after_secs: (archive_days as u64) * 86_400,
            auto_purge_after_secs: (purge_days as u64) * 86_400,
        }
    }
}

/// Trait for types that support soft-delete lifecycle operations.
pub trait SoftDeletable {
    type Error;

    fn soft_delete(
        &mut self,
        deleted_by: Address,
        deleted_at: u64,
        reason: String,
        retention_days: u32,
    ) -> Result<(), Self::Error>;

    fn restore(&mut self) -> Result<(), Self::Error>;

    fn archive(&mut self) -> Result<(), Self::Error>;

    fn is_deleted(&self) -> bool;

    fn is_archived(&self) -> bool;

    fn is_active(&self) -> bool;

    fn status(&self) -> RecordStatus;
}

/// A generic soft-deletable wrapper that can hold any record.
#[derive(Clone, Debug)]
#[contracttype]
pub struct SoftDeleteRecord {
    pub status: RecordStatus,
    pub deletion_metadata: Option<DeletionMetadata>,
    pub archived_at: Option<u64>,
}

impl SoftDeleteRecord {
    pub fn new() -> Self {
        Self {
            status: RecordStatus::Active,
            deletion_metadata: None,
            archived_at: None,
        }
    }
}

impl SoftDeletable for SoftDeleteRecord {
    type Error = SoftDeleteError;

    fn soft_delete(
        &mut self,
        deleted_by: Address,
        deleted_at: u64,
        reason: String,
        retention_days: u32,
    ) -> Result<(), SoftDeleteError> {
        if self.status == RecordStatus::SoftDeleted {
            return Err(SoftDeleteError::AlreadyDeleted);
        }
        if self.status == RecordStatus::Archived {
            return Err(SoftDeleteError::AlreadyArchived);
        }

        self.status = RecordStatus::SoftDeleted;
        self.deletion_metadata = Some(DeletionMetadata {
            deleted_by,
            deleted_at,
            reason,
            retention_days,
        });
        Ok(())
    }

    fn restore(&mut self) -> Result<(), SoftDeleteError> {
        if self.status == RecordStatus::Archived {
            return Err(SoftDeleteError::AlreadyArchived);
        }
        if self.status == RecordStatus::Active {
            return Err(SoftDeleteError::InvalidStateTransition);
        }

        self.status = RecordStatus::Active;
        self.deletion_metadata = None;
        Ok(())
    }

    fn archive(&mut self) -> Result<(), SoftDeleteError> {
        match self.status {
            RecordStatus::Archived => Err(SoftDeleteError::AlreadyArchived),
            RecordStatus::SoftDeleted => {
                self.status = RecordStatus::Archived;
                Ok(())
            }
            RecordStatus::Active => {
                self.status = RecordStatus::Archived;
                Ok(())
            }
        }
    }

    fn is_deleted(&self) -> bool {
        self.status == RecordStatus::SoftDeleted
    }

    fn is_archived(&self) -> bool {
        self.status == RecordStatus::Archived
    }

    fn is_active(&self) -> bool {
        self.status == RecordStatus::Active
    }

    fn status(&self) -> RecordStatus {
        self.status
    }
}

/// Check whether a soft-deleted record's retention period has expired.
pub fn is_retention_expired(metadata: &DeletionMetadata, current_time: u64) -> bool {
    check_retention(metadata.deleted_at, metadata.retention_days, current_time)
}

/// Check retention expiry from raw fields (no struct dependency).
pub fn check_retention(deleted_at: u64, retention_days: u32, current_time: u64) -> bool {
    if retention_days == 0 {
        return false;
    }
    let expiry = deleted_at + (retention_days as u64) * 86_400;
    current_time >= expiry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_record_is_active() {
        let record = SoftDeleteRecord::new();
        assert!(record.is_active());
        assert!(!record.is_deleted());
        assert!(!record.is_archived());
        assert_eq!(record.status, RecordStatus::Active);
    }

    #[test]
    fn test_archive_from_active() {
        let mut record = SoftDeleteRecord::new();
        record.archive().unwrap();
        assert!(record.is_archived());
    }

    #[test]
    fn test_restore_on_active_fails() {
        let mut record = SoftDeleteRecord::new();
        let result = record.restore();
        assert_eq!(result, Err(SoftDeleteError::InvalidStateTransition));
    }

    #[test]
    fn test_restore_on_archived_fails() {
        let mut record = SoftDeleteRecord::new();
        record.archive().unwrap();
        let result = record.restore();
        assert_eq!(result, Err(SoftDeleteError::AlreadyArchived));
    }

    #[test]
    fn test_double_archive_fails() {
        let mut record = SoftDeleteRecord::new();
        record.archive().unwrap();
        let result = record.archive();
        assert_eq!(result, Err(SoftDeleteError::AlreadyArchived));
    }

    #[test]
    fn test_archive_policy_after_days() {
        let policy = ArchivePolicy::after_days(90, 365);
        assert_eq!(policy.auto_archive_after_secs, 90 * 86_400);
        assert_eq!(policy.auto_purge_after_secs, 365 * 86_400);
    }

    #[test]
    fn test_status_transitions() {
        let record = SoftDeleteRecord::new();
        assert_eq!(record.status(), RecordStatus::Active);
    }

    #[test]
    fn test_retention_expired() {
        assert!(!check_retention(1000, 30, 1000 + 29 * 86400));
        assert!(check_retention(1000, 30, 1000 + 30 * 86400));
    }

    #[test]
    fn test_retention_zero_never_expires() {
        assert!(!check_retention(1000, 0, 1000 + 999_999_999));
    }
}
