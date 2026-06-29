use std::vec::Vec;

use contract_behavior_fuzzing::{
    execute_sequence, run_regressions, BehaviorHarness, OperationOutcome, RegressionCase,
};
use medical_records::{
    MedicalRecord, RecordMetadata, RecordMetadataHistoryEntry,
    DataQualityScore, ValidationIssue, ValidationReport, ValidationSeverity, CorrectionItem,
    CorrectionWorkflow, CorrectionPriority, CorrectionAction, CleanseResult,
    UserProfile, Role, Permission,
    ZkPublicInputs,
    KeyEnvelope, EnvelopeAlgorithm, EncryptedRecord,
    CrossChainRecordRef, ChainId,
    RateLimitConfig,
    BatchResult, FailureInfo,
    AbePolicyMetadata,
    CryptoConfigProposal,
    AIInsight, AIInsightType,
};
use proptest::collection::vec;
use proptest::prelude::*;
use soroban_sdk::{
    testutils::{Address as _, Events as _},
    xdr::{FromXdr, ToXdr},
    Address, Bytes, BytesN, Env, Map, String, Vec as SorobanVec,
};

mod support;

// =============================================================================
// Fuzz operations — each operation exercises serialisation of contract types
// =============================================================================

#[derive(Clone, Debug)]
enum MedicalRecordSerdeOp {
    MedicalRecordRoundTrip { diagnosis_len: u8, treatment_len: u8 },
    RecordMetadataRoundTrip { tag_count: u8 },
    DataQualityScoreRoundTrip,
    ValidationReportRoundTrip { issue_count: u8 },
    CorrectionWorkflowRoundTrip { correction_count: u8 },
    ZkPublicInputsRoundTrip,
    KeyEnvelopeRoundTrip,
    EncryptedRecordRoundTrip { tag_count: u8, envelope_count: u8 },
    CrossChainRecordRefRoundTrip,
    AbePolicyMetadataRoundTrip,
    CryptoConfigProposalRoundTrip,
    AiInsightRoundTrip,
    UserProfileRoundTrip,
    CleanseResultRoundTrip { change_count: u8 },
    BatchResultRoundTrip { success_count: u8, failure_count: u8 },
    RateLimitConfigRoundTrip,
}

// =============================================================================
// Harness
// =============================================================================

struct MedicalRecordSerdeHarness {
    env: Env,
    event_count: usize,
}

impl MedicalRecordSerdeHarness {
    fn new() -> Self {
        let env = Env::default();
        let event_count = env.events().all().len() as usize;
        Self { env, event_count }
    }

    fn account(&self, seed: u8) -> Address {
        Address::from_xdr(
            &self.env,
            &Bytes::from_slice(&self.env, &[seed; 32]),
        )
        .unwrap()
    }

    fn bytes32(&self, seed: u8) -> BytesN<32> {
        BytesN::from_array(&self.env, &[seed; 32])
    }

    fn string(&self, value: &str) -> String {
        String::from_str(&self.env, value)
    }

    fn medical_record(&self, diagnosis: &str, treatment: &str) -> MedicalRecord {
        MedicalRecord {
            patient_id: self.account(1),
            doctor_id: self.account(2),
            timestamp: 1_700_000_000,
            diagnosis: self.string(diagnosis),
            treatment: self.string(treatment),
            is_confidential: false,
            tags: {
                let mut tags = SorobanVec::new(&self.env);
                tags.push_back(self.string("fuzz"));
                tags
            },
            category: self.string("General"),
            treatment_type: self.string("Standard"),
            data_ref: self.string("ipfs://fuzz"),
            doctor_did: Some(self.string("did:example:fuzz")),
        }
    }
}

impl BehaviorHarness for MedicalRecordSerdeHarness {
    type Operation = MedicalRecordSerdeOp;

    fn apply(&mut self, operation: &Self::Operation) -> OperationOutcome {
        match operation {
            MedicalRecordSerdeOp::MedicalRecordRoundTrip { diagnosis_len, treatment_len } => {
                let diagnosis = "A".repeat((*diagnosis_len as usize).max(1) % 256);
                let treatment = "B".repeat((*treatment_len as usize).max(1) % 256);
                let record = self.medical_record(&diagnosis, &treatment);

                // Serialize then deserialize
                let xdr_bytes = record.to_xdr(&self.env);
                let deserialized =
                    MedicalRecord::from_xdr(&self.env, &xdr_bytes).expect("MedicalRecord round-trip");
                assert_eq!(record.patient_id, deserialized.patient_id);
                assert_eq!(record.doctor_id, deserialized.doctor_id);
                assert_eq!(record.timestamp, deserialized.timestamp);
                assert_eq!(record.diagnosis, deserialized.diagnosis);
                assert_eq!(record.treatment, deserialized.treatment);
                assert_eq!(record.is_confidential, deserialized.is_confidential);
                assert_eq!(record.tags.len(), deserialized.tags.len());
                assert_eq!(record.category, deserialized.category);
                assert_eq!(record.treatment_type, deserialized.treatment_type);
                assert_eq!(record.data_ref, deserialized.data_ref);

                // Re-serialize should match original bytes
                let re_xdr = deserialized.to_xdr(&self.env);
                assert_eq!(xdr_bytes, re_xdr, "MedicalRecord XDR idempotency");

                OperationOutcome::new(0)
            },
            MedicalRecordSerdeOp::RecordMetadataRoundTrip { tag_count } => {
                let count = (*tag_count as usize) % 20;
                let mut tags = SorobanVec::new(&self.env);
                for i in 0..count {
                    tags.push_back(self.string(&format!("tag-{i}")));
                }
                let mut custom_fields = Map::new(&self.env);
                custom_fields.set(self.string("key"), self.string("value"));

                let mut history = SorobanVec::new(&self.env);
                history.push_back(RecordMetadataHistoryEntry {
                    version: 1,
                    timestamp: 1_700_000_000,
                    tags: tags.clone(),
                    custom_fields: custom_fields.clone(),
                });

                let meta = RecordMetadata {
                    record_id: 42,
                    patient_id: self.account(1),
                    timestamp: 1_700_000_000,
                    category: self.string("General"),
                    is_confidential: false,
                    record_hash: self.bytes32(7),
                    tags,
                    custom_fields,
                    version: 1,
                    history,
                };

                let xdr_bytes = meta.to_xdr(&self.env);
                let deserialized =
                    RecordMetadata::from_xdr(&self.env, &xdr_bytes).expect("RecordMetadata round-trip");
                assert_eq!(meta.record_id, deserialized.record_id);
                assert_eq!(meta.version, deserialized.version);
                assert_eq!(meta.category, deserialized.category);

                let re_xdr = deserialized.to_xdr(&self.env);
                assert_eq!(xdr_bytes, re_xdr, "RecordMetadata XDR idempotency");

                OperationOutcome::new(0)
            },
            MedicalRecordSerdeOp::DataQualityScoreRoundTrip => {
                let score = DataQualityScore {
                    overall_score: 8_500,
                    completeness_score: 9_000,
                    format_score: 8_000,
                    consistency_score: 7_500,
                    fhir_compliance_score: 9_500,
                    issue_count: 2,
                };
                let xdr_bytes = score.to_xdr(&self.env);
                let deserialized =
                    DataQualityScore::from_xdr(&self.env, &xdr_bytes).expect("DataQualityScore round-trip");
                assert_eq!(score.overall_score, deserialized.overall_score);
                assert_eq!(score.issue_count, deserialized.issue_count);

                let re_xdr = deserialized.to_xdr(&self.env);
                assert_eq!(xdr_bytes, re_xdr, "DataQualityScore XDR idempotency");

                OperationOutcome::new(0)
            },
            MedicalRecordSerdeOp::ValidationReportRoundTrip { issue_count } => {
                let count = (*issue_count as usize) % 10;
                let mut issues = SorobanVec::new(&self.env);
                for i in 0..count {
                    issues.push_back(ValidationIssue {
                        severity: ValidationSeverity::Warning,
                        field_name: self.string(&format!("field-{i}")),
                        issue_description: self.string(&format!("issue-{i}")),
                        suggestion: self.string(&format!("suggestion-{i}")),
                    });
                }
                let report = ValidationReport {
                    record_id: 1,
                    quality_score: DataQualityScore {
                        overall_score: 8_000,
                        completeness_score: 8_000,
                        format_score: 8_000,
                        consistency_score: 8_000,
                        fhir_compliance_score: 8_000,
                        issue_count: count as u32,
                    },
                    issues,
                    is_fhir_compliant: true,
                    validated_at: 1_700_000_000,
                };
                let xdr_bytes = report.to_xdr(&self.env);
                let deserialized =
                    ValidationReport::from_xdr(&self.env, &xdr_bytes).expect("ValidationReport round-trip");
                assert_eq!(report.record_id, deserialized.record_id);
                assert_eq!(report.issues.len(), deserialized.issues.len());
                assert_eq!(report.is_fhir_compliant, deserialized.is_fhir_compliant);

                let re_xdr = deserialized.to_xdr(&self.env);
                assert_eq!(xdr_bytes, re_xdr, "ValidationReport XDR idempotency");

                OperationOutcome::new(0)
            },
            MedicalRecordSerdeOp::CorrectionWorkflowRoundTrip { correction_count } => {
                let count = (*correction_count as usize) % 10;
                let mut corrections = SorobanVec::new(&self.env);
                for i in 0..count {
                    corrections.push_back(CorrectionItem {
                        field_name: self.string(&format!("field-{i}")),
                        action: CorrectionAction::FixFormat,
                        description: self.string(&format!("desc-{i}")),
                        suggested_value: Some(self.string(&format!("val-{i}"))),
                        priority: CorrectionPriority::High,
                    });
                }
                let workflow = CorrectionWorkflow {
                    record_id: 1,
                    total_issues: count as u32,
                    critical_count: 0,
                    error_count: count as u32,
                    warning_count: 0,
                    info_count: 0,
                    corrections,
                    can_auto_fix: false,
                    workflow_created_at: 1_700_000_000,
                };
                let xdr_bytes = workflow.to_xdr(&self.env);
                let deserialized =
                    CorrectionWorkflow::from_xdr(&self.env, &xdr_bytes).expect("CorrectionWorkflow round-trip");
                assert_eq!(workflow.record_id, deserialized.record_id);
                assert_eq!(workflow.total_issues, deserialized.total_issues);

                let re_xdr = deserialized.to_xdr(&self.env);
                assert_eq!(xdr_bytes, re_xdr, "CorrectionWorkflow XDR idempotency");

                OperationOutcome::new(0)
            },
            MedicalRecordSerdeOp::ZkPublicInputsRoundTrip => {
                let zk = ZkPublicInputs {
                    record_id: 42,
                    record_commitment: self.bytes32(1),
                    credential_root: self.bytes32(2),
                    issuer: self.account(3),
                    requester_commitment: self.bytes32(4),
                    provider_commitment: self.bytes32(5),
                    claim_commitment: self.bytes32(6),
                    min_timestamp: 1_700_000_000,
                    max_timestamp: 1_800_000_000,
                    nullifier: self.bytes32(7),
                    pseudonym: self.bytes32(8),
                    vk_version: 1,
                };
                let xdr_bytes = zk.to_xdr(&self.env);
                let deserialized =
                    ZkPublicInputs::from_xdr(&self.env, &xdr_bytes).expect("ZkPublicInputs round-trip");
                assert_eq!(zk.record_id, deserialized.record_id);
                assert_eq!(zk.vk_version, deserialized.vk_version);

                let re_xdr = deserialized.to_xdr(&self.env);
                assert_eq!(xdr_bytes, re_xdr, "ZkPublicInputs XDR idempotency");

                OperationOutcome::new(0)
            },
            MedicalRecordSerdeOp::KeyEnvelopeRoundTrip => {
                let envelope = KeyEnvelope {
                    recipient: self.account(1),
                    key_version: 1,
                    algorithm: EnvelopeAlgorithm::X25519,
                    wrapped_key: Bytes::from_slice(&self.env, &[0x01, 0x02, 0x03]),
                    pq_wrapped_key: Some(Bytes::from_slice(&self.env, &[0x04, 0x05, 0x06])),
                };
                let xdr_bytes = envelope.to_xdr(&self.env);
                let deserialized =
                    KeyEnvelope::from_xdr(&self.env, &xdr_bytes).expect("KeyEnvelope round-trip");
                assert_eq!(envelope.recipient, deserialized.recipient);
                assert_eq!(envelope.algorithm, deserialized.algorithm);

                let re_xdr = deserialized.to_xdr(&self.env);
                assert_eq!(xdr_bytes, re_xdr, "KeyEnvelope XDR idempotency");

                OperationOutcome::new(0)
            },
            MedicalRecordSerdeOp::EncryptedRecordRoundTrip { tag_count, envelope_count } => {
                let t_count = (*tag_count as usize) % 10;
                let e_count = (*envelope_count as usize).max(1) % 5;
                let mut tags = SorobanVec::new(&self.env);
                for i in 0..t_count {
                    tags.push_back(self.string(&format!("tag-{i}")));
                }
                let mut envelopes = SorobanVec::new(&self.env);
                for i in 0..e_count {
                    envelopes.push_back(KeyEnvelope {
                        recipient: self.account(i as u8 + 1),
                        key_version: 1,
                        algorithm: EnvelopeAlgorithm::X25519,
                        wrapped_key: Bytes::from_slice(&self.env, &[0x01, 0x02, 0x03]),
                        pq_wrapped_key: None,
                    });
                }
                let record = EncryptedRecord {
                    patient_id: self.account(1),
                    doctor_id: self.account(2),
                    timestamp: 1_700_000_000,
                    is_confidential: true,
                    tags,
                    category: self.string("General"),
                    treatment_type: self.string("Standard"),
                    ciphertext_ref: self.string("ipfs://encrypted"),
                    ciphertext_hash: self.bytes32(42),
                    envelopes,
                    doctor_did: None,
                };
                let xdr_bytes = record.to_xdr(&self.env);
                let deserialized =
                    EncryptedRecord::from_xdr(&self.env, &xdr_bytes).expect("EncryptedRecord round-trip");
                assert_eq!(record.patient_id, deserialized.patient_id);
                assert_eq!(record.ciphertext_hash, deserialized.ciphertext_hash);

                let re_xdr = deserialized.to_xdr(&self.env);
                assert_eq!(xdr_bytes, re_xdr, "EncryptedRecord XDR idempotency");

                OperationOutcome::new(0)
            },
            MedicalRecordSerdeOp::CrossChainRecordRefRoundTrip => {
                let ref_record = CrossChainRecordRef {
                    local_record_id: 1,
                    external_chain: ChainId::Ethereum,
                    external_record_hash: self.bytes32(99),
                    sync_timestamp: 1_700_000_000,
                    is_synced: true,
                };
                let xdr_bytes = ref_record.to_xdr(&self.env);
                let deserialized =
                    CrossChainRecordRef::from_xdr(&self.env, &xdr_bytes).expect("CrossChainRecordRef round-trip");
                assert_eq!(ref_record.local_record_id, deserialized.local_record_id);
                assert_eq!(ref_record.external_chain, deserialized.external_chain);

                let re_xdr = deserialized.to_xdr(&self.env);
                assert_eq!(xdr_bytes, re_xdr, "CrossChainRecordRef XDR idempotency");

                OperationOutcome::new(0)
            },
            MedicalRecordSerdeOp::AbePolicyMetadataRoundTrip => {
                let abe = AbePolicyMetadata {
                    policy_ref: self.string("ipfs://policy"),
                    policy_hash: self.bytes32(1),
                    access_ciphertext_ref: self.string("ipfs://access"),
                    access_ciphertext_hash: self.bytes32(2),
                    required_permission: Permission::ReadRecord,
                    attribute_count: 3,
                    compiled_at: 1_700_000_000,
                    valid_until: 1_800_000_000,
                    revocation_epoch: 0,
                };
                let xdr_bytes = abe.to_xdr(&self.env);
                let deserialized =
                    AbePolicyMetadata::from_xdr(&self.env, &xdr_bytes).expect("AbePolicyMetadata round-trip");
                assert_eq!(abe.required_permission, deserialized.required_permission);
                assert_eq!(abe.attribute_count, deserialized.attribute_count);

                let re_xdr = deserialized.to_xdr(&self.env);
                assert_eq!(xdr_bytes, re_xdr, "AbePolicyMetadata XDR idempotency");

                OperationOutcome::new(0)
            },
            MedicalRecordSerdeOp::CryptoConfigProposalRoundTrip => {
                let proposal = CryptoConfigProposal {
                    proposal_id: 1,
                    created_at: 1_700_000_000,
                    executed: false,
                    approvals: {
                        let mut a = SorobanVec::new(&self.env);
                        a.push_back(self.account(1));
                        a
                    },
                    new_crypto_registry: Some(self.account(2)),
                    new_homomorphic_registry: None,
                    new_mpc_manager: None,
                    encryption_required: Some(true),
                    require_pq_envelopes: None,
                };
                let xdr_bytes = proposal.to_xdr(&self.env);
                let deserialized = CryptoConfigProposal::from_xdr(&self.env, &xdr_bytes)
                    .expect("CryptoConfigProposal round-trip");
                assert_eq!(proposal.proposal_id, deserialized.proposal_id);
                assert_eq!(proposal.encryption_required, deserialized.encryption_required);

                let re_xdr = deserialized.to_xdr(&self.env);
                assert_eq!(xdr_bytes, re_xdr, "CryptoConfigProposal XDR idempotency");

                OperationOutcome::new(0)
            },
            MedicalRecordSerdeOp::AiInsightRoundTrip => {
                let insight = AIInsight {
                    patient: self.account(1),
                    record_id: 42,
                    model_id: self.bytes32(1),
                    insight_type: AIInsightType::RiskScore,
                    score_bps: 7_500,
                    explanation_ref: self.string("ipfs://explain"),
                    explanation_summary: self.string("High risk of readmission"),
                    created_at: 1_700_000_000,
                    model_version: self.string("v1.0"),
                };
                let xdr_bytes = insight.to_xdr(&self.env);
                let deserialized =
                    AIInsight::from_xdr(&self.env, &xdr_bytes).expect("AIInsight round-trip");
                assert_eq!(insight.record_id, deserialized.record_id);
                assert_eq!(insight.insight_type, deserialized.insight_type);

                let re_xdr = deserialized.to_xdr(&self.env);
                assert_eq!(xdr_bytes, re_xdr, "AIInsight XDR idempotency");

                OperationOutcome::new(0)
            },
            MedicalRecordSerdeOp::UserProfileRoundTrip => {
                let profile = UserProfile {
                    role: Role::Doctor,
                    active: true,
                    did_reference: Some(self.string("did:example:doctor")),
                    qkd_capable: false,
                };
                let xdr_bytes = profile.to_xdr(&self.env);
                let deserialized =
                    UserProfile::from_xdr(&self.env, &xdr_bytes).expect("UserProfile round-trip");
                assert_eq!(profile.role, deserialized.role);
                assert_eq!(profile.active, deserialized.active);

                let re_xdr = deserialized.to_xdr(&self.env);
                assert_eq!(xdr_bytes, re_xdr, "UserProfile XDR idempotency");

                OperationOutcome::new(0)
            },
            MedicalRecordSerdeOp::CleanseResultRoundTrip { change_count } => {
                let count = (*change_count as usize) % 10;
                let mut changes = SorobanVec::new(&self.env);
                for i in 0..count {
                    changes.push_back(self.string(&format!("change-{i}")));
                }
                let result = CleanseResult {
                    record: self.medical_record("diagnosis", "treatment"),
                    changes_made: changes,
                    was_modified: count > 0,
                };
                let xdr_bytes = result.to_xdr(&self.env);
                let deserialized =
                    CleanseResult::from_xdr(&self.env, &xdr_bytes).expect("CleanseResult round-trip");
                assert_eq!(result.changes_made.len(), deserialized.changes_made.len());
                assert_eq!(result.was_modified, deserialized.was_modified);

                let re_xdr = deserialized.to_xdr(&self.env);
                assert_eq!(xdr_bytes, re_xdr, "CleanseResult XDR idempotency");

                OperationOutcome::new(0)
            },
            MedicalRecordSerdeOp::BatchResultRoundTrip { success_count, failure_count } => {
                let s_count = (*success_count as usize) % 20;
                let f_count = (*failure_count as usize) % 10;
                let mut successes = SorobanVec::new(&self.env);
                for i in 0..s_count {
                    successes.push_back(i as u64);
                }
                let mut failures = SorobanVec::new(&self.env);
                for i in 0..f_count {
                    failures.push_back(FailureInfo {
                        index: i as u32,
                        error_code: 100 + i as u32,
                    });
                }
                let batch = BatchResult {
                    successes,
                    failures,
                };
                let xdr_bytes = batch.to_xdr(&self.env);
                let deserialized =
                    BatchResult::from_xdr(&self.env, &xdr_bytes).expect("BatchResult round-trip");
                assert_eq!(batch.successes.len(), deserialized.successes.len());
                assert_eq!(batch.failures.len(), deserialized.failures.len());

                let re_xdr = deserialized.to_xdr(&self.env);
                assert_eq!(xdr_bytes, re_xdr, "BatchResult XDR idempotency");

                OperationOutcome::new(0)
            },
            MedicalRecordSerdeOp::RateLimitConfigRoundTrip => {
                let config = RateLimitConfig {
                    doctor_max_calls: 50,
                    patient_max_calls: 10,
                    admin_max_calls: 0,
                    window_secs: 3_600,
                };
                let xdr_bytes = config.to_xdr(&self.env);
                let deserialized =
                    RateLimitConfig::from_xdr(&self.env, &xdr_bytes).expect("RateLimitConfig round-trip");
                assert_eq!(config.doctor_max_calls, deserialized.doctor_max_calls);
                assert_eq!(config.window_secs, deserialized.window_secs);

                let re_xdr = deserialized.to_xdr(&self.env);
                assert_eq!(xdr_bytes, re_xdr, "RateLimitConfig XDR idempotency");

                OperationOutcome::new(0)
            },
        }
    }

    fn assert_invariants(&self) {
        // No persistent state to check — we test serialization round-trips
    }

    fn event_count(&self) -> usize {
        self.event_count
    }
}

// =============================================================================
// Operation strategy for proptest
// =============================================================================

fn serde_operation() -> impl Strategy<Value = MedicalRecordSerdeOp> {
    prop_oneof![
        (any::<u8>(), any::<u8>()).prop_map(|(d, t)| MedicalRecordSerdeOp::MedicalRecordRoundTrip {
            diagnosis_len: d,
            treatment_len: t,
        }),
        any::<u8>().prop_map(|tag_count| MedicalRecordSerdeOp::RecordMetadataRoundTrip { tag_count }),
        Just(MedicalRecordSerdeOp::DataQualityScoreRoundTrip),
        any::<u8>().prop_map(|issue_count| MedicalRecordSerdeOp::ValidationReportRoundTrip { issue_count }),
        any::<u8>().prop_map(|correction_count| {
            MedicalRecordSerdeOp::CorrectionWorkflowRoundTrip { correction_count }
        }),
        Just(MedicalRecordSerdeOp::ZkPublicInputsRoundTrip),
        Just(MedicalRecordSerdeOp::KeyEnvelopeRoundTrip),
        (any::<u8>(), any::<u8>()).prop_map(|(tag_count, envelope_count)| {
            MedicalRecordSerdeOp::EncryptedRecordRoundTrip {
                tag_count,
                envelope_count,
            }
        }),
        Just(MedicalRecordSerdeOp::CrossChainRecordRefRoundTrip),
        Just(MedicalRecordSerdeOp::AbePolicyMetadataRoundTrip),
        Just(MedicalRecordSerdeOp::CryptoConfigProposalRoundTrip),
        Just(MedicalRecordSerdeOp::AiInsightRoundTrip),
        Just(MedicalRecordSerdeOp::UserProfileRoundTrip),
        any::<u8>().prop_map(|change_count| MedicalRecordSerdeOp::CleanseResultRoundTrip { change_count }),
        (any::<u8>(), any::<u8>()).prop_map(|(success_count, failure_count)| {
            MedicalRecordSerdeOp::BatchResultRoundTrip {
                success_count,
                failure_count,
            }
        }),
        Just(MedicalRecordSerdeOp::RateLimitConfigRoundTrip),
    ]
}

// =============================================================================
// Regression cases
// =============================================================================

#[test]
fn medical_records_serde_regressions() {
    let cases: Vec<RegressionCase<MedicalRecordSerdeOp>> = vec![
        RegressionCase {
            name: "basic-round-trip",
            operations: vec![
                MedicalRecordSerdeOp::MedicalRecordRoundTrip {
                    diagnosis_len: 10,
                    treatment_len: 20,
                },
                MedicalRecordSerdeOp::RecordMetadataRoundTrip { tag_count: 3 },
                MedicalRecordSerdeOp::DataQualityScoreRoundTrip,
            ],
        },
        RegressionCase {
            name: "zk-and-encryption-round-trips",
            operations: vec![
                MedicalRecordSerdeOp::ZkPublicInputsRoundTrip,
                MedicalRecordSerdeOp::KeyEnvelopeRoundTrip,
                MedicalRecordSerdeOp::EncryptedRecordRoundTrip {
                    tag_count: 5,
                    envelope_count: 3,
                },
            ],
        },
        RegressionCase {
            name: "validation-and-correction-round-trips",
            operations: vec![
                MedicalRecordSerdeOp::ValidationReportRoundTrip { issue_count: 5 },
                MedicalRecordSerdeOp::CorrectionWorkflowRoundTrip { correction_count: 3 },
                MedicalRecordSerdeOp::CleanseResultRoundTrip { change_count: 2 },
            ],
        },
        RegressionCase {
            name: "cross-chain-and-admin-round-trips",
            operations: vec![
                MedicalRecordSerdeOp::CrossChainRecordRefRoundTrip,
                MedicalRecordSerdeOp::AbePolicyMetadataRoundTrip,
                MedicalRecordSerdeOp::CryptoConfigProposalRoundTrip,
                MedicalRecordSerdeOp::BatchResultRoundTrip {
                    success_count: 5,
                    failure_count: 2,
                },
            ],
        },
        RegressionCase {
            name: "ai-and-user-round-trips",
            operations: vec![
                MedicalRecordSerdeOp::AiInsightRoundTrip,
                MedicalRecordSerdeOp::UserProfileRoundTrip,
                MedicalRecordSerdeOp::RateLimitConfigRoundTrip,
            ],
        },
    ];

    run_regressions::<MedicalRecordSerdeHarness, _>(&cases, MedicalRecordSerdeHarness::new);
}

// =============================================================================
// Property-based fuzz tests
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 24,
        max_shrink_iters: 0,
        .. ProptestConfig::default()
    })]

    #[test]
    fn fuzz_medical_records_serde(operations in vec(serde_operation(), 1..20)) {
        let mut harness = MedicalRecordSerdeHarness::new();
        let report = execute_sequence(&mut harness, &operations);
        prop_assert!(report.is_ok(), "{report:?}");
    }
}
