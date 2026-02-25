#![no_std]
#![allow(clippy::too_many_arguments)]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Bytes, BytesN, Env,
    String, Symbol, Vec,
};

const ROLE_OPERATOR: u32 = 1;
const ROLE_AUDITOR: u32 = 2;
const ROLE_RECOVERY: u32 = 4;
const ALL_ROLES: u32 = ROLE_OPERATOR | ROLE_AUDITOR | ROLE_RECOVERY;

const ADMIN: Symbol = symbol_short!("ADMIN");
const PAUSED: Symbol = symbol_short!("PAUSED");
const NEXT_TGT: Symbol = symbol_short!("NEXT_TGT");
const NEXT_BKP: Symbol = symbol_short!("NEXT_BKP");
const NEXT_EXE: Symbol = symbol_short!("NEXT_EXE");
const NEXT_ALT: Symbol = symbol_short!("NEXT_ALT");
const NEXT_TST: Symbol = symbol_short!("NEXT_TST");
const NEXT_RST: Symbol = symbol_short!("NEXT_RST");
const LAST_RUN: Symbol = symbol_short!("LAST_RUN");
const NEXT_RUN: Symbol = symbol_short!("NEXT_RUN");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum BackupNetwork {
    Stellar,
    Ethereum,
    Polygon,
    Arbitrum,
    Optimism,
    Avalanche,
    BinanceSmartChain,
    Ipfs,
    Filecoin,
    Arweave,
    AwsS3,
    AzureBlob,
    GcpStorage,
    Custom(u32),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum GeoRegion {
    UsEast,
    UsWest,
    EuCentral,
    EuWest,
    ApSouth,
    ApNorth,
    SaEast,
    AfSouth,
    Custom(u32),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum BackupStatus {
    Completed,
    Verified,
    Failed,
    Archived,
    Restored,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ReplicaStatus {
    Synced,
    Verified,
    Failed,
    Archived,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum AlertKind {
    BackupFailure,
    TargetFailure,
    GeoRedundancyRisk,
    IntegrityCheckFailed,
    RestoreFailure,
    CostThresholdExceeded,
    ScheduleMissed,
    RecoveryDrillFailed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum RestoreStatus {
    Pending,
    Approved,
    Executed,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct BackupPolicy {
    pub interval_seconds: u64,
    pub retention_seconds: u64,
    pub max_active_backups: u32,
    pub min_targets_per_backup: u32,
    pub min_region_count: u32,
    pub max_total_cost_weight: u32,
    pub verify_on_write: bool,
    pub encryption_required: bool,
    pub auto_cleanup: bool,
    pub min_restore_approvals: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct BackupTarget {
    pub target_id: u32,
    pub network: BackupNetwork,
    pub region: GeoRegion,
    pub endpoint_hash: BytesN<32>,
    pub is_active: bool,
    pub encrypted_only: bool,
    pub cost_weight: u32,
    pub max_capacity_units: u64,
    pub failure_count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct BackupArtifact {
    pub artifact_id: u64,
    pub source_root: BytesN<32>,
    pub checksum: BytesN<32>,
    pub snapshot_ref: String,
    pub encryption_key_version: u32,
    pub encrypted: bool,
    pub created_at: u64,
    pub expires_at: u64,
    pub target_ids: Vec<u32>,
    pub region_count: u32,
    pub total_cost_weight: u32,
    pub status: BackupStatus,
    pub last_verified_at: u64,
    pub last_restored_at: u64,
    pub restore_drill_passed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct BackupReplica {
    pub artifact_id: u64,
    pub target_id: u32,
    pub checksum: BytesN<32>,
    pub synced_at: u64,
    pub status: ReplicaStatus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct BackupExecution {
    pub execution_id: u64,
    pub triggered_by: Address,
    pub started_at: u64,
    pub completed_at: u64,
    pub scheduled: bool,
    pub success_targets: u32,
    pub failed_targets: u32,
    pub artifact_id: Option<u64>,
    pub error_code: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct AlertEntry {
    pub alert_id: u64,
    pub kind: AlertKind,
    pub severity: AlertSeverity,
    pub created_at: u64,
    pub details_hash: BytesN<32>,
    pub resolved: bool,
    pub resolved_at: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct RecoveryTest {
    pub test_id: u64,
    pub artifact_id: u64,
    pub started_by: Address,
    pub executed_at: u64,
    pub validation_hash: BytesN<32>,
    pub passed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct RestoreRequest {
    pub request_id: u64,
    pub artifact_id: u64,
    pub requested_by: Address,
    pub reason_hash: BytesN<32>,
    pub requested_at: u64,
    pub approvals: Vec<Address>,
    pub status: RestoreStatus,
    pub executed_at: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct BackupHealth {
    pub total_runs: u64,
    pub successful_runs: u64,
    pub failed_runs: u64,
    pub consecutive_failures: u32,
    pub last_success_at: u64,
    pub last_failure_at: u64,
    pub last_error_code: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CleanupReport {
    pub archived_backups: u32,
    pub reclaimed_cost_weight: u32,
    pub remaining_active_backups: u32,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Policy,
    Roles(Address),
    Target(u32),
    TargetIds,
    Artifact(u64),
    ArtifactIds,
    Replica(u64, u32),
    Execution(u64),
    Alert(u64),
    AlertIds,
    RecoveryTest(u64),
    RestoreRequest(u64),
    Health,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    ContractPaused = 4,
    InvalidInput = 5,
    TargetNotFound = 6,
    BackupNotFound = 7,
    RestoreRequestNotFound = 8,
    RecoveryTestNotFound = 9,
    ScheduleNotDue = 10,
    InsufficientTargets = 11,
    GeoRedundancyNotMet = 12,
    EncryptionRequired = 13,
    IntegrityMismatch = 14,
    RestoreNotApproved = 15,
    AlreadyExecuted = 16,
    DuplicateApproval = 17,
    CostLimitExceeded = 18,
}

#[contract]
pub struct MedicalRecordBackupContract;

#[contractimpl]
impl MedicalRecordBackupContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&ADMIN) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&PAUSED, &false);
        env.storage().instance().set(&NEXT_TGT, &1u32);
        env.storage().instance().set(&NEXT_BKP, &1u64);
        env.storage().instance().set(&NEXT_EXE, &1u64);
        env.storage().instance().set(&NEXT_ALT, &1u64);
        env.storage().instance().set(&NEXT_TST, &1u64);
        env.storage().instance().set(&NEXT_RST, &1u64);
        env.storage().instance().set(&LAST_RUN, &0u64);
        env.storage()
            .instance()
            .set(&NEXT_RUN, &env.ledger().timestamp());
        env.storage()
            .persistent()
            .set(&DataKey::TargetIds, &Vec::<u32>::new(&env));
        env.storage()
            .persistent()
            .set(&DataKey::ArtifactIds, &Vec::<u64>::new(&env));
        env.storage()
            .persistent()
            .set(&DataKey::AlertIds, &Vec::<u64>::new(&env));

        env.storage().persistent().set(
            &DataKey::Policy,
            &BackupPolicy {
                interval_seconds: 21_600,
                retention_seconds: 2_592_000,
                max_active_backups: 30,
                min_targets_per_backup: 2,
                min_region_count: 2,
                max_total_cost_weight: 1_000,
                verify_on_write: true,
                encryption_required: true,
                auto_cleanup: true,
                min_restore_approvals: 1,
            },
        );

        env.storage().persistent().set(
            &DataKey::Health,
            &BackupHealth {
                total_runs: 0,
                successful_runs: 0,
                failed_runs: 0,
                consecutive_failures: 0,
                last_success_at: 0,
                last_failure_at: 0,
                last_error_code: 0,
            },
        );

        env.events()
            .publish((symbol_short!("BKP_INIT"),), admin.clone());
        Ok(())
    }

    pub fn set_paused(env: Env, caller: Address, paused: bool) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        env.storage().instance().set(&PAUSED, &paused);
        Ok(true)
    }

    pub fn assign_role(
        env: Env,
        caller: Address,
        user: Address,
        role_mask: u32,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        let allowed = role_mask & ALL_ROLES;
        env.storage()
            .persistent()
            .set(&DataKey::Roles(user.clone()), &allowed);
        env.events()
            .publish((symbol_short!("BKP_ROLE"),), (user, allowed));
        Ok(true)
    }

    pub fn set_policy(env: Env, caller: Address, policy: BackupPolicy) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        if policy.interval_seconds == 0
            || policy.retention_seconds == 0
            || policy.max_active_backups == 0
            || policy.min_targets_per_backup == 0
            || policy.min_region_count == 0
            || policy.max_total_cost_weight == 0
        {
            return Err(Error::InvalidInput);
        }

        env.storage().persistent().set(&DataKey::Policy, &policy);
        env.events().publish((symbol_short!("BKP_POL"),), caller);
        Ok(true)
    }

    pub fn get_policy(env: Env) -> Result<BackupPolicy, Error> {
        Self::require_initialized(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::Policy)
            .ok_or(Error::NotInitialized)
    }

    pub fn register_target(
        env: Env,
        caller: Address,
        network: BackupNetwork,
        region: GeoRegion,
        endpoint_hash: BytesN<32>,
        encrypted_only: bool,
        cost_weight: u32,
        max_capacity_units: u64,
    ) -> Result<u32, Error> {
        caller.require_auth();
        Self::require_operator(&env, &caller)?;
        Self::require_not_paused(&env)?;
        if cost_weight == 0 || max_capacity_units == 0 {
            return Err(Error::InvalidInput);
        }

        let target_id = Self::next_target_id(&env);
        let target = BackupTarget {
            target_id,
            network,
            region,
            endpoint_hash,
            is_active: true,
            encrypted_only,
            cost_weight,
            max_capacity_units,
            failure_count: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Target(target_id), &target);
        let mut ids: Vec<u32> = env
            .storage()
            .persistent()
            .get(&DataKey::TargetIds)
            .unwrap_or(Vec::new(&env));
        ids.push_back(target_id);
        env.storage().persistent().set(&DataKey::TargetIds, &ids);
        env.events()
            .publish((symbol_short!("BKP_TGT"),), (target_id,));
        Ok(target_id)
    }

    pub fn set_target_active(
        env: Env,
        caller: Address,
        target_id: u32,
        active: bool,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_operator(&env, &caller)?;
        let mut target: BackupTarget = env
            .storage()
            .persistent()
            .get(&DataKey::Target(target_id))
            .ok_or(Error::TargetNotFound)?;
        target.is_active = active;
        env.storage()
            .persistent()
            .set(&DataKey::Target(target_id), &target);
        Ok(true)
    }

    pub fn get_target(env: Env, target_id: u32) -> Option<BackupTarget> {
        env.storage().persistent().get(&DataKey::Target(target_id))
    }

    pub fn list_targets(env: Env) -> Vec<BackupTarget> {
        let ids: Vec<u32> = env
            .storage()
            .persistent()
            .get(&DataKey::TargetIds)
            .unwrap_or(Vec::new(&env));
        let mut out = Vec::new(&env);
        for id in ids.iter() {
            if let Some(t) = env
                .storage()
                .persistent()
                .get::<DataKey, BackupTarget>(&DataKey::Target(id))
            {
                out.push_back(t);
            }
        }
        out
    }

    pub fn run_scheduled_backup(
        env: Env,
        caller: Address,
        source_root: BytesN<32>,
        snapshot_ref: String,
        encryption_key_version: u32,
    ) -> Result<u64, Error> {
        caller.require_auth();
        Self::require_operator(&env, &caller)?;
        let now = env.ledger().timestamp();
        let next_run: u64 = env.storage().instance().get(&NEXT_RUN).unwrap_or(0);
        if now < next_run {
            return Err(Error::ScheduleNotDue);
        }
        Self::execute_backup(
            env,
            caller,
            source_root,
            snapshot_ref,
            encryption_key_version,
            true,
        )
    }

    pub fn run_backup_now(
        env: Env,
        caller: Address,
        source_root: BytesN<32>,
        snapshot_ref: String,
        encryption_key_version: u32,
    ) -> Result<u64, Error> {
        caller.require_auth();
        Self::require_operator(&env, &caller)?;
        Self::execute_backup(
            env,
            caller,
            source_root,
            snapshot_ref,
            encryption_key_version,
            false,
        )
    }

    pub fn verify_backup_integrity(
        env: Env,
        caller: Address,
        artifact_id: u64,
        observed_checksum: BytesN<32>,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_auditor(&env, &caller)?;

        let mut artifact: BackupArtifact = env
            .storage()
            .persistent()
            .get(&DataKey::Artifact(artifact_id))
            .ok_or(Error::BackupNotFound)?;

        let mut ok = artifact.checksum == observed_checksum;
        if ok {
            for target_id in artifact.target_ids.iter() {
                let replica: BackupReplica = env
                    .storage()
                    .persistent()
                    .get(&DataKey::Replica(artifact_id, target_id))
                    .ok_or(Error::IntegrityMismatch)?;
                if replica.status == ReplicaStatus::Failed || replica.checksum != artifact.checksum
                {
                    ok = false;
                    break;
                }
            }
        }

        if ok {
            artifact.status = BackupStatus::Verified;
            artifact.last_verified_at = env.ledger().timestamp();
        } else {
            artifact.status = BackupStatus::Failed;
            let details =
                Self::compute_reason_hash(&env, Error::IntegrityMismatch as u32, artifact_id);
            Self::append_alert(
                &env,
                AlertKind::IntegrityCheckFailed,
                AlertSeverity::High,
                details,
            );
        }
        env.storage()
            .persistent()
            .set(&DataKey::Artifact(artifact_id), &artifact);
        Ok(ok)
    }

    pub fn request_restore(
        env: Env,
        caller: Address,
        artifact_id: u64,
        reason_hash: BytesN<32>,
    ) -> Result<u64, Error> {
        caller.require_auth();
        Self::require_recovery(&env, &caller)?;
        let _: BackupArtifact = env
            .storage()
            .persistent()
            .get(&DataKey::Artifact(artifact_id))
            .ok_or(Error::BackupNotFound)?;

        let request_id = Self::next_restore_request_id(&env);
        let request = RestoreRequest {
            request_id,
            artifact_id,
            requested_by: caller,
            reason_hash,
            requested_at: env.ledger().timestamp(),
            approvals: Vec::new(&env),
            status: RestoreStatus::Pending,
            executed_at: 0,
        };
        env.storage()
            .persistent()
            .set(&DataKey::RestoreRequest(request_id), &request);
        env.events()
            .publish((symbol_short!("BKP_RREQ"),), (request_id, artifact_id));
        Ok(request_id)
    }

    pub fn approve_restore(env: Env, caller: Address, request_id: u64) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_auditor(&env, &caller)?;
        let mut request: RestoreRequest = env
            .storage()
            .persistent()
            .get(&DataKey::RestoreRequest(request_id))
            .ok_or(Error::RestoreRequestNotFound)?;
        if request.status == RestoreStatus::Executed {
            return Err(Error::AlreadyExecuted);
        }
        if request.approvals.iter().any(|a| a == caller) {
            return Err(Error::DuplicateApproval);
        }
        request.approvals.push_back(caller.clone());
        let policy = Self::get_policy_internal(&env)?;
        if request.approvals.len() >= policy.min_restore_approvals {
            request.status = RestoreStatus::Approved;
        }
        env.storage()
            .persistent()
            .set(&DataKey::RestoreRequest(request_id), &request);
        env.events()
            .publish((symbol_short!("BKP_RAPP"),), (request_id, caller));
        Ok(true)
    }

    pub fn execute_restore(env: Env, caller: Address, request_id: u64) -> Result<String, Error> {
        caller.require_auth();
        Self::require_recovery(&env, &caller)?;
        let mut request: RestoreRequest = env
            .storage()
            .persistent()
            .get(&DataKey::RestoreRequest(request_id))
            .ok_or(Error::RestoreRequestNotFound)?;
        if request.status != RestoreStatus::Approved {
            return Err(Error::RestoreNotApproved);
        }

        let mut artifact: BackupArtifact = env
            .storage()
            .persistent()
            .get(&DataKey::Artifact(request.artifact_id))
            .ok_or(Error::BackupNotFound)?;

        request.status = RestoreStatus::Executed;
        request.executed_at = env.ledger().timestamp();
        artifact.status = BackupStatus::Restored;
        artifact.last_restored_at = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::RestoreRequest(request_id), &request);
        env.storage()
            .persistent()
            .set(&DataKey::Artifact(request.artifact_id), &artifact);

        env.events().publish(
            (symbol_short!("BKP_REST"),),
            (request_id, request.artifact_id),
        );
        Ok(artifact.snapshot_ref)
    }

    pub fn run_recovery_test(
        env: Env,
        caller: Address,
        artifact_id: u64,
        validation_hash: BytesN<32>,
    ) -> Result<u64, Error> {
        caller.require_auth();
        Self::require_recovery(&env, &caller)?;
        let mut artifact: BackupArtifact = env
            .storage()
            .persistent()
            .get(&DataKey::Artifact(artifact_id))
            .ok_or(Error::BackupNotFound)?;

        let passed = validation_hash == artifact.checksum
            && artifact.status != BackupStatus::Archived
            && !artifact.target_ids.is_empty();

        let test_id = Self::next_recovery_test_id(&env);
        let test = RecoveryTest {
            test_id,
            artifact_id,
            started_by: caller,
            executed_at: env.ledger().timestamp(),
            validation_hash,
            passed,
        };

        if passed {
            artifact.restore_drill_passed = true;
            env.storage()
                .persistent()
                .set(&DataKey::Artifact(artifact_id), &artifact);
        } else {
            let details =
                Self::compute_reason_hash(&env, Error::IntegrityMismatch as u32, artifact_id);
            Self::append_alert(
                &env,
                AlertKind::RecoveryDrillFailed,
                AlertSeverity::Medium,
                details,
            );
        }
        env.storage()
            .persistent()
            .set(&DataKey::RecoveryTest(test_id), &test);
        env.events()
            .publish((symbol_short!("BKP_TEST"),), (test_id, passed));
        Ok(test_id)
    }

    pub fn optimize_and_cleanup(env: Env, caller: Address) -> Result<CleanupReport, Error> {
        caller.require_auth();
        Self::require_operator(&env, &caller)?;
        let policy = Self::get_policy_internal(&env)?;
        let report = Self::optimize_and_cleanup_internal(&env, &policy);
        env.events()
            .publish((symbol_short!("BKP_CLEAN"),), report.clone());
        Ok(report)
    }

    pub fn report_target_failure(
        env: Env,
        caller: Address,
        target_id: u32,
        reason_hash: BytesN<32>,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_operator(&env, &caller)?;
        let mut target: BackupTarget = env
            .storage()
            .persistent()
            .get(&DataKey::Target(target_id))
            .ok_or(Error::TargetNotFound)?;
        target.failure_count = target.failure_count.saturating_add(1);
        env.storage()
            .persistent()
            .set(&DataKey::Target(target_id), &target);
        Self::append_alert(
            &env,
            AlertKind::TargetFailure,
            AlertSeverity::High,
            reason_hash,
        );
        Ok(true)
    }

    pub fn resolve_alert(env: Env, caller: Address, alert_id: u64) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_auditor(&env, &caller)?;
        let mut alert: AlertEntry = env
            .storage()
            .persistent()
            .get(&DataKey::Alert(alert_id))
            .ok_or(Error::InvalidInput)?;
        alert.resolved = true;
        alert.resolved_at = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::Alert(alert_id), &alert);
        Ok(true)
    }

    pub fn list_alerts(env: Env, open_only: bool) -> Vec<AlertEntry> {
        let ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::AlertIds)
            .unwrap_or(Vec::new(&env));
        let mut out = Vec::new(&env);
        for id in ids.iter() {
            if let Some(a) = env
                .storage()
                .persistent()
                .get::<DataKey, AlertEntry>(&DataKey::Alert(id))
            {
                if !open_only || !a.resolved {
                    out.push_back(a);
                }
            }
        }
        out
    }

    pub fn list_artifacts(env: Env, include_archived: bool) -> Vec<BackupArtifact> {
        let ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::ArtifactIds)
            .unwrap_or(Vec::new(&env));
        let mut out = Vec::new(&env);
        for id in ids.iter() {
            if let Some(a) = env
                .storage()
                .persistent()
                .get::<DataKey, BackupArtifact>(&DataKey::Artifact(id))
            {
                if include_archived || a.status != BackupStatus::Archived {
                    out.push_back(a);
                }
            }
        }
        out
    }

    pub fn get_artifact(env: Env, artifact_id: u64) -> Option<BackupArtifact> {
        env.storage()
            .persistent()
            .get(&DataKey::Artifact(artifact_id))
    }

    pub fn get_execution(env: Env, execution_id: u64) -> Option<BackupExecution> {
        env.storage()
            .persistent()
            .get(&DataKey::Execution(execution_id))
    }

    pub fn get_restore_request(env: Env, request_id: u64) -> Option<RestoreRequest> {
        env.storage()
            .persistent()
            .get(&DataKey::RestoreRequest(request_id))
    }

    pub fn get_recovery_test(env: Env, test_id: u64) -> Option<RecoveryTest> {
        env.storage()
            .persistent()
            .get(&DataKey::RecoveryTest(test_id))
    }

    pub fn get_health(env: Env) -> BackupHealth {
        env.storage()
            .persistent()
            .get(&DataKey::Health)
            .unwrap_or(BackupHealth {
                total_runs: 0,
                successful_runs: 0,
                failed_runs: 0,
                consecutive_failures: 0,
                last_success_at: 0,
                last_failure_at: 0,
                last_error_code: 0,
            })
    }

    pub fn get_schedule(env: Env) -> (u64, u64) {
        (
            env.storage().instance().get(&LAST_RUN).unwrap_or(0),
            env.storage().instance().get(&NEXT_RUN).unwrap_or(0),
        )
    }

    fn execute_backup(
        env: Env,
        caller: Address,
        source_root: BytesN<32>,
        snapshot_ref: String,
        encryption_key_version: u32,
        scheduled: bool,
    ) -> Result<u64, Error> {
        Self::require_initialized(&env)?;
        Self::require_not_paused(&env)?;
        let policy = Self::get_policy_internal(&env)?;

        if policy.encryption_required && encryption_key_version == 0 {
            Self::record_failed_execution(
                &env,
                caller,
                scheduled,
                Error::EncryptionRequired as u32,
                0,
            );
            return Err(Error::EncryptionRequired);
        }

        let selected = Self::select_targets(&env, &policy);
        let (target_ids, region_count, total_cost) = match selected {
            Ok(v) => v,
            Err(e) => {
                Self::record_failed_execution(&env, caller, scheduled, e as u32, 0);
                return Err(e);
            }
        };

        if total_cost > policy.max_total_cost_weight {
            Self::record_failed_execution(
                &env,
                caller,
                scheduled,
                Error::CostLimitExceeded as u32,
                0,
            );
            return Err(Error::CostLimitExceeded);
        }

        let artifact_id = Self::next_backup_id(&env);
        let now = env.ledger().timestamp();
        let checksum = Self::compute_checksum(&env, &source_root, artifact_id, now);

        let mut status = BackupStatus::Completed;
        let mut verified_at = 0u64;
        for target_id in target_ids.iter() {
            let replica_status = if policy.verify_on_write {
                ReplicaStatus::Verified
            } else {
                ReplicaStatus::Synced
            };
            let replica = BackupReplica {
                artifact_id,
                target_id,
                checksum: checksum.clone(),
                synced_at: now,
                status: replica_status,
            };
            env.storage()
                .persistent()
                .set(&DataKey::Replica(artifact_id, target_id), &replica);
        }
        if policy.verify_on_write {
            status = BackupStatus::Verified;
            verified_at = now;
        }

        let artifact = BackupArtifact {
            artifact_id,
            source_root,
            checksum,
            snapshot_ref,
            encryption_key_version,
            encrypted: encryption_key_version > 0,
            created_at: now,
            expires_at: now.saturating_add(policy.retention_seconds),
            target_ids: target_ids.clone(),
            region_count,
            total_cost_weight: total_cost,
            status,
            last_verified_at: verified_at,
            last_restored_at: 0,
            restore_drill_passed: false,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Artifact(artifact_id), &artifact);
        let mut artifact_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::ArtifactIds)
            .unwrap_or(Vec::new(&env));
        artifact_ids.push_back(artifact_id);
        env.storage()
            .persistent()
            .set(&DataKey::ArtifactIds, &artifact_ids);

        Self::record_success_execution(
            &env,
            caller.clone(),
            scheduled,
            artifact_id,
            target_ids.len(),
            0,
        );
        env.storage().instance().set(&LAST_RUN, &now);
        env.storage()
            .instance()
            .set(&NEXT_RUN, &now.saturating_add(policy.interval_seconds));
        env.events().publish(
            (symbol_short!("BKP_RUN"),),
            (artifact_id, target_ids.len(), region_count),
        );

        if policy.auto_cleanup {
            let _ = Self::optimize_and_cleanup_internal(&env, &policy);
        }

        Ok(artifact_id)
    }

    fn select_targets(env: &Env, policy: &BackupPolicy) -> Result<(Vec<u32>, u32, u32), Error> {
        let ids: Vec<u32> = env
            .storage()
            .persistent()
            .get(&DataKey::TargetIds)
            .unwrap_or(Vec::new(env));
        let mut selected = Vec::new(env);
        let mut regions = Vec::new(env);
        let mut total_cost = 0u32;

        for id in ids.iter() {
            let target: BackupTarget = match env.storage().persistent().get(&DataKey::Target(id)) {
                Some(t) => t,
                None => continue,
            };
            if !target.is_active {
                continue;
            }
            if policy.encryption_required && !target.encrypted_only {
                continue;
            }
            if total_cost.saturating_add(target.cost_weight) > policy.max_total_cost_weight {
                continue;
            }

            selected.push_back(id);
            total_cost = total_cost.saturating_add(target.cost_weight);
            if !Self::contains_region(&regions, target.region) {
                regions.push_back(target.region);
            }
            if selected.len() >= policy.min_targets_per_backup
                && regions.len() >= policy.min_region_count
            {
                break;
            }
        }

        if selected.len() < policy.min_targets_per_backup {
            let details = Self::compute_reason_hash(env, Error::InsufficientTargets as u32, 0);
            Self::append_alert(
                env,
                AlertKind::BackupFailure,
                AlertSeverity::Critical,
                details,
            );
            return Err(Error::InsufficientTargets);
        }
        if regions.len() < policy.min_region_count {
            let details = Self::compute_reason_hash(env, Error::GeoRedundancyNotMet as u32, 0);
            Self::append_alert(
                env,
                AlertKind::GeoRedundancyRisk,
                AlertSeverity::Critical,
                details,
            );
            return Err(Error::GeoRedundancyNotMet);
        }
        Ok((selected, regions.len(), total_cost))
    }

    fn contains_region(regions: &Vec<GeoRegion>, candidate: GeoRegion) -> bool {
        regions.iter().any(|r| r == candidate)
    }

    fn record_success_execution(
        env: &Env,
        caller: Address,
        scheduled: bool,
        artifact_id: u64,
        success_targets: u32,
        failed_targets: u32,
    ) {
        let execution_id = Self::next_execution_id(env);
        let now = env.ledger().timestamp();
        let exec = BackupExecution {
            execution_id,
            triggered_by: caller,
            started_at: now,
            completed_at: now,
            scheduled,
            success_targets,
            failed_targets,
            artifact_id: Some(artifact_id),
            error_code: None,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Execution(execution_id), &exec);

        let mut health: BackupHealth =
            env.storage()
                .persistent()
                .get(&DataKey::Health)
                .unwrap_or(BackupHealth {
                    total_runs: 0,
                    successful_runs: 0,
                    failed_runs: 0,
                    consecutive_failures: 0,
                    last_success_at: 0,
                    last_failure_at: 0,
                    last_error_code: 0,
                });
        health.total_runs = health.total_runs.saturating_add(1);
        health.successful_runs = health.successful_runs.saturating_add(1);
        health.consecutive_failures = 0;
        health.last_success_at = now;
        env.storage().persistent().set(&DataKey::Health, &health);
    }

    fn record_failed_execution(
        env: &Env,
        caller: Address,
        scheduled: bool,
        error_code: u32,
        failed_targets: u32,
    ) {
        let execution_id = Self::next_execution_id(env);
        let now = env.ledger().timestamp();
        let exec = BackupExecution {
            execution_id,
            triggered_by: caller,
            started_at: now,
            completed_at: now,
            scheduled,
            success_targets: 0,
            failed_targets,
            artifact_id: None,
            error_code: Some(error_code),
        };
        env.storage()
            .persistent()
            .set(&DataKey::Execution(execution_id), &exec);

        let mut health: BackupHealth =
            env.storage()
                .persistent()
                .get(&DataKey::Health)
                .unwrap_or(BackupHealth {
                    total_runs: 0,
                    successful_runs: 0,
                    failed_runs: 0,
                    consecutive_failures: 0,
                    last_success_at: 0,
                    last_failure_at: 0,
                    last_error_code: 0,
                });
        health.total_runs = health.total_runs.saturating_add(1);
        health.failed_runs = health.failed_runs.saturating_add(1);
        health.consecutive_failures = health.consecutive_failures.saturating_add(1);
        health.last_failure_at = now;
        health.last_error_code = error_code;
        env.storage().persistent().set(&DataKey::Health, &health);

        let details = Self::compute_reason_hash(env, error_code, 0);
        Self::append_alert(
            env,
            AlertKind::BackupFailure,
            AlertSeverity::Critical,
            details,
        );
        if let Ok(policy) = Self::get_policy_internal(env) {
            env.storage().instance().set(&LAST_RUN, &now);
            env.storage()
                .instance()
                .set(&NEXT_RUN, &now.saturating_add(policy.interval_seconds));
        }
    }

    fn append_alert(env: &Env, kind: AlertKind, severity: AlertSeverity, details_hash: BytesN<32>) {
        let alert_id = Self::next_alert_id(env);
        let now = env.ledger().timestamp();
        let alert = AlertEntry {
            alert_id,
            kind,
            severity,
            created_at: now,
            details_hash,
            resolved: false,
            resolved_at: 0,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Alert(alert_id), &alert);
        let mut ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::AlertIds)
            .unwrap_or(Vec::new(env));
        ids.push_back(alert_id);
        env.storage().persistent().set(&DataKey::AlertIds, &ids);
        env.events()
            .publish((symbol_short!("BKP_ALT"),), (alert_id, kind, severity));
    }

    fn compute_checksum(
        env: &Env,
        source_root: &BytesN<32>,
        artifact_id: u64,
        now: u64,
    ) -> BytesN<32> {
        let mut payload = Bytes::new(env);
        payload.append(&Bytes::from_slice(env, &source_root.to_array()));
        payload.append(&Bytes::from_slice(env, &artifact_id.to_be_bytes()));
        payload.append(&Bytes::from_slice(env, &now.to_be_bytes()));
        env.crypto().sha256(&payload).into()
    }

    fn compute_reason_hash(env: &Env, code: u32, context: u64) -> BytesN<32> {
        let mut payload = Bytes::new(env);
        payload.append(&Bytes::from_slice(env, &code.to_be_bytes()));
        payload.append(&Bytes::from_slice(env, &context.to_be_bytes()));
        payload.append(&Bytes::from_slice(
            env,
            &env.ledger().timestamp().to_be_bytes(),
        ));
        env.crypto().sha256(&payload).into()
    }

    fn optimize_and_cleanup_internal(env: &Env, policy: &BackupPolicy) -> CleanupReport {
        let now = env.ledger().timestamp();
        let ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::ArtifactIds)
            .unwrap_or(Vec::new(env));

        let mut active_count: u32 = 0;
        for id in ids.iter() {
            if let Some(a) = env
                .storage()
                .persistent()
                .get::<DataKey, BackupArtifact>(&DataKey::Artifact(id))
            {
                if a.status != BackupStatus::Archived {
                    active_count = active_count.saturating_add(1);
                }
            }
        }

        let mut archived: u32 = 0;
        let mut reclaimed: u32 = 0;
        for id in ids.iter() {
            if let Some(mut artifact) = env
                .storage()
                .persistent()
                .get::<DataKey, BackupArtifact>(&DataKey::Artifact(id))
            {
                let expired = now >= artifact.expires_at;
                let exceeds_limit = active_count > policy.max_active_backups;
                if artifact.status != BackupStatus::Archived && (expired || exceeds_limit) {
                    artifact.status = BackupStatus::Archived;
                    archived = archived.saturating_add(1);
                    reclaimed = reclaimed.saturating_add(artifact.total_cost_weight);
                    if active_count > 0 {
                        active_count = active_count.saturating_sub(1);
                    }
                    for target_id in artifact.target_ids.iter() {
                        env.storage()
                            .persistent()
                            .remove(&DataKey::Replica(artifact.artifact_id, target_id));
                    }
                    env.storage()
                        .persistent()
                        .set(&DataKey::Artifact(id), &artifact);
                }
            }
        }

        CleanupReport {
            archived_backups: archived,
            reclaimed_cost_weight: reclaimed,
            remaining_active_backups: active_count,
        }
    }

    fn require_initialized(env: &Env) -> Result<(), Error> {
        if !env.storage().instance().has(&ADMIN) {
            return Err(Error::NotInitialized);
        }
        Ok(())
    }

    fn require_not_paused(env: &Env) -> Result<(), Error> {
        if env.storage().instance().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }
        Ok(())
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        Self::require_initialized(env)?;
        let admin: Address = env
            .storage()
            .instance()
            .get(&ADMIN)
            .ok_or(Error::NotInitialized)?;
        if *caller != admin {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    fn require_role(env: &Env, caller: &Address, role: u32) -> Result<(), Error> {
        Self::require_initialized(env)?;
        let admin: Address = env
            .storage()
            .instance()
            .get(&ADMIN)
            .ok_or(Error::NotInitialized)?;
        if *caller == admin {
            return Ok(());
        }
        let mask: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::Roles(caller.clone()))
            .unwrap_or(0u32);
        if (mask & role) == 0 {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    fn require_operator(env: &Env, caller: &Address) -> Result<(), Error> {
        Self::require_role(env, caller, ROLE_OPERATOR)
    }

    fn require_auditor(env: &Env, caller: &Address) -> Result<(), Error> {
        Self::require_role(env, caller, ROLE_AUDITOR)
    }

    fn require_recovery(env: &Env, caller: &Address) -> Result<(), Error> {
        Self::require_role(env, caller, ROLE_RECOVERY)
    }

    fn get_policy_internal(env: &Env) -> Result<BackupPolicy, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Policy)
            .ok_or(Error::NotInitialized)
    }

    fn next_target_id(env: &Env) -> u32 {
        let current: u32 = env.storage().instance().get(&NEXT_TGT).unwrap_or(1);
        env.storage()
            .instance()
            .set(&NEXT_TGT, &current.saturating_add(1));
        current
    }

    fn next_backup_id(env: &Env) -> u64 {
        let current: u64 = env.storage().instance().get(&NEXT_BKP).unwrap_or(1);
        env.storage()
            .instance()
            .set(&NEXT_BKP, &current.saturating_add(1));
        current
    }

    fn next_execution_id(env: &Env) -> u64 {
        let current: u64 = env.storage().instance().get(&NEXT_EXE).unwrap_or(1);
        env.storage()
            .instance()
            .set(&NEXT_EXE, &current.saturating_add(1));
        current
    }

    fn next_alert_id(env: &Env) -> u64 {
        let current: u64 = env.storage().instance().get(&NEXT_ALT).unwrap_or(1);
        env.storage()
            .instance()
            .set(&NEXT_ALT, &current.saturating_add(1));
        current
    }

    fn next_recovery_test_id(env: &Env) -> u64 {
        let current: u64 = env.storage().instance().get(&NEXT_TST).unwrap_or(1);
        env.storage()
            .instance()
            .set(&NEXT_TST, &current.saturating_add(1));
        current
    }

    fn next_restore_request_id(env: &Env) -> u64 {
        let current: u64 = env.storage().instance().get(&NEXT_RST).unwrap_or(1);
        env.storage()
            .instance()
            .set(&NEXT_RST, &current.saturating_add(1));
        current
    }
}
