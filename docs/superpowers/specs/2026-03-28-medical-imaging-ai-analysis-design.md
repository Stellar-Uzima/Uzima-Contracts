# Medical Imaging AI Analysis — Design Spec

**Issue:** #192 Implement Medical Imaging AI Analysis
**Date:** 2026-03-28
**Status:** Draft

## Overview

Extend the existing `medical_imaging` contract and create a new `medical_imaging_ai` contract to deliver CNN-based diagnostic analysis, structured anomaly detection and segmentation, radiologist workflow integration, and performance benchmarking with tiered enforcement.

Off-chain AI inference is out of scope. The contracts define the interface for submitting, verifying, storing, and evaluating AI-generated results.

## Architecture

Two contracts, one clean split:

- **`medical_imaging`** (extend existing) — study lifecycle, radiologist workflow with configurable multi-reader review, structured edge detection/segmentation output, query indexes by reader/status/patient.
- **`medical_imaging_ai`** (new contract) — CNN model metadata with ed25519 signing keys, oracle-attested analysis and segmentation results, rolling-window performance benchmarking with tiered enforcement.

**No cross-contract calls.** Model status is enforced at submission time in `medical_imaging_ai`. The `medical_imaging` contract references AI result IDs but does not call across contracts. This keeps coupling minimal and gas costs predictable.

**Role management:** Both contracts maintain their own RBAC. The `medical_imaging_ai` contract has a `register_evaluator(admin, evaluator)` method to authorize radiologists who can call `record_evaluation`. This avoids cross-contract role lookups.

### Interaction Flow

```
Off-chain AI service
        |
        v
medical_imaging_ai  <--  register model (with signing pubkey),
        |                 submit attested results (signature verified),
        |                 track performance, auto-deactivate if degraded
        |
        v (referenced by result IDs, no cross-contract call)
medical_imaging  <--  create study, assign readers, blind review,
                      consensus/discrepancy detection, finalize reports,
                      store spatial anomaly/segmentation data
```

## Data Models

### `medical_imaging_ai` Types

#### CnnModelMetadata

Extends the existing `AiDiagnosticModel` concept with CNN-specific fields.

| Field | Type | Description |
|---|---|---|
| model_id | BytesN<32> | Unique model identifier |
| owner | Address | Model registrant |
| version | u32 | Model version number |
| modality | ImagingModality | Target imaging modality |
| architecture_hash | BytesN<32> | Hash of architecture name (e.g. "ResNet50") |
| layer_count | u32 | Number of layers |
| input_rows | u32 | Expected input height in pixels |
| input_cols | u32 | Expected input width in pixels |
| input_channels | u32 | 1 for grayscale, 3 for RGB |
| training_samples | u64 | Number of training samples |
| validation_accuracy_bps | u32 | Accuracy at registration time (basis points) |
| training_dataset_hash | BytesN<32> | Hash of training dataset for reproducibility |
| signing_pubkey | BytesN<32> | Ed25519 public key for attestation verification |
| status | ModelStatus | Active, Degraded, Deactivated, or Retired |
| registered_at | u64 | Registration timestamp |
| last_evaluated_at | u64 | Last performance evaluation timestamp |

#### ModelStatus

```rust
pub enum ModelStatus {
    Active,       // performing within thresholds
    Degraded,     // below warning threshold, still usable but flagged
    Deactivated,  // below critical threshold or admin-disabled
    Retired,      // manually retired by owner
}
```

#### AnalysisResult

| Field | Type | Description |
|---|---|---|
| result_id | u64 | Auto-incremented result identifier |
| image_id | u64 | Reference to image in medical_imaging contract |
| model_id | BytesN<32> | Model that produced the result |
| submitter | Address | Address that submitted the result |
| attestation_hash | BytesN<32> | hash(model_id \|\| image_hash \|\| result_hash) |
| signature | BytesN<64> | Ed25519 signature over attestation_hash |
| findings | Vec\<Finding\> | Structured findings, max 20 |
| overall_confidence_bps | u32 | Overall confidence (0-10000) |
| processing_time_ms | u32 | Time taken for inference |
| created_at | u64 | Submission timestamp |

#### Finding

| Field | Type | Description |
|---|---|---|
| finding_id | u32 | Sequential within the analysis |
| condition_hash | BytesN<32> | Hash of condition name (e.g. "pneumothorax") |
| confidence_bps | u32 | Confidence for this finding (0-10000) |
| severity | u32 | 1-5 scale |
| region | BoundingBox | Spatial location in image |
| explanation_ref | String | IPFS CID to SHAP/attention map |

#### BoundingBox

| Field | Type | Description |
|---|---|---|
| x_min | u32 | Left pixel coordinate |
| y_min | u32 | Top pixel coordinate |
| x_max | u32 | Right pixel coordinate |
| y_max | u32 | Bottom pixel coordinate |

Invariant: x_min < x_max, y_min < y_max. Validated at submission.

#### SegmentationResult

| Field | Type | Description |
|---|---|---|
| seg_id | u64 | Auto-incremented segmentation identifier |
| image_id | u64 | Reference to image in medical_imaging contract |
| model_id | BytesN<32> | Model that produced the result |
| submitter | Address | Address that submitted the result |
| attestation_hash | BytesN<32> | hash(model_id \|\| image_hash \|\| result_hash) |
| signature | BytesN<64> | Ed25519 signature over attestation_hash |
| regions | Vec\<SegmentedRegion\> | Segmented regions, max 30 |
| processing_time_ms | u32 | Time taken for inference |
| created_at | u64 | Submission timestamp |

#### SegmentedRegion

| Field | Type | Description |
|---|---|---|
| label_hash | BytesN<32> | Hash of region label (e.g. "left_lung") |
| pixel_count | u64 | Number of pixels in region |
| volume_mm3 | u64 | Volume in cubic mm (0 if 2D modality) |
| mean_intensity | u32 | Mean pixel intensity in region |
| mask_ref | String | IPFS CID to binary mask |
| bounds | BoundingBox | Bounding box of region |

#### ModelPerformance

| Field | Type | Description |
|---|---|---|
| model_id | BytesN<32> | Model being tracked |
| modality | ImagingModality | Modality of tracked performance |
| total_evaluated | u64 | Lifetime evaluation count |
| correct_count | u64 | Lifetime correct count |
| lifetime_accuracy_bps | u32 | Lifetime accuracy for reference |
| window_size | u64 | Rolling window size (default 100) |
| window_correct | u64 | Correct within current window |
| window_total | u64 | Evaluated within current window |
| rolling_accuracy_bps | u32 | Accuracy within current window |
| avg_processing_time_ms | u32 | Average processing time |
| warning_threshold_bps | u32 | Degraded trigger (default 9200) |
| critical_threshold_bps | u32 | Deactivation trigger (default 8500) |
| min_sample_size | u64 | Min evaluations before enforcement (default 50) |
| last_updated | u64 | Last update timestamp |

**Rolling window behavior:** When `window_total` exceeds `window_size`, the window resets (counters zeroed). This is an approximation — exact sliding windows are too expensive on-chain. The reset provides periodic fresh measurement of model performance.

#### Storage Keys

```rust
pub enum DataKey {
    Admin,
    Initialized,
    Paused,
    DefaultWarningBps,
    DefaultCriticalBps,
    DefaultMinSamples,
    CnnModel(BytesN<32>),           // model_id -> CnnModelMetadata
    AnalysisResult(u64),             // result_id -> AnalysisResult
    SegResult(u64),                  // seg_id -> SegmentationResult
    Performance(BytesN<32>),         // model_id -> ModelPerformance
    ImageResults(u64),               // image_id -> Vec<u64> (result_ids)
    ImageSegResults(u64),            // image_id -> Vec<u64> (seg_ids)
    Evaluator(Address),              // evaluator -> bool (authorized)
    NextResultId,
    NextSegId,
}
```

### `medical_imaging` New Types

#### ImagingStudy

| Field | Type | Description |
|---|---|---|
| study_id | u64 | Auto-incremented study identifier |
| patient | Address | Patient address |
| created_by | Address | Study creator |
| modality | ImagingModality | Imaging modality |
| image_ids | Vec\<u64\> | Image IDs in this study, max 500 |
| ai_result_ids | Vec\<u64\> | AI result IDs from medical_imaging_ai |
| required_readers | u32 | Number of readers required (1-5) |
| status | StudyStatus | Current workflow status |
| created_at | u64 | Creation timestamp |
| finalized_at | u64 | Finalization timestamp (0 if not finalized) |

#### StudyStatus

```rust
pub enum StudyStatus {
    Pending,             // created, awaiting reader assignment
    Assigned,            // readers assigned, awaiting review
    InReview,            // at least one reader has started
    PreliminaryReport,   // all readers submitted, no discrepancy
    DiscrepancyReview,   // readers disagree on diagnosis, arbitrator needed
    FinalReport,         // finalized by lead reader or admin
    Amended,             // amended after finalization
}
```

**Valid transitions:**

```
Pending --> Assigned            (on assign_reader)
Assigned --> InReview           (on first submit_reader_report)
InReview --> PreliminaryReport  (all readers submitted, diagnosis_hash match)
InReview --> DiscrepancyReview  (all readers submitted, diagnosis_hash mismatch)
DiscrepancyReview --> PreliminaryReport  (arbitrator submits resolving report)
PreliminaryReport --> FinalReport        (finalize_study)
FinalReport --> Amended                  (amend_study)
Amended --> Amended                      (subsequent amendments)
```

Invalid transitions return `StudyNotInExpectedStatus`.

**Index maintenance:** On every status transition, the study ID is removed from `StatusStudies(old_status)` and added to `StatusStudies(new_status)`. Similarly, `ReaderStudies(reader)` is updated on `assign_reader`. This keeps query indexes consistent without requiring full scans.

#### ReaderReport

| Field | Type | Description |
|---|---|---|
| report_id | u64 | Auto-incremented report identifier |
| study_id | u64 | Study this report belongs to |
| reader | Address | Radiologist who submitted |
| diagnosis_hash | BytesN<32> | Hash of structured diagnosis (used for agreement) |
| findings_hash | BytesN<32> | Hash of full detailed findings (audit trail) |
| findings_ref | String | Encrypted reference to full report |
| agrees_with_ai | bool | Whether reader concurs with AI findings |
| ai_accuracy_feedback_bps | u32 | Reader's assessment of AI accuracy |
| submitted_at | u64 | Submission timestamp |

**Discrepancy detection:** Compares `diagnosis_hash` across all reader reports. If all match, readers agree. If any differ, study moves to `DiscrepancyReview`. This avoids on-chain semantic comparison — hash equality is deterministic and cheap.

**Blind review enforcement:** `get_reader_reports` only returns reports when study is in PreliminaryReport or later status. Before that, readers can only retrieve their own report via `get_my_report`.

#### EdgeDetectionResult (replaces current u32 return)

| Field | Type | Description |
|---|---|---|
| image_id | u64 | Image processed |
| quality_score_bps | u32 | Overall quality score |
| edge_count | u32 | Number of edges detected |
| histogram_bins | Vec\<u32\> | Intensity histogram |
| regions_of_interest | Vec\<BoundingBox\> | Where edges cluster |
| processed_at | u64 | Processing timestamp |

#### New Storage Keys (added to existing DataKey enum)

```rust
Study(u64),                  // study_id -> ImagingStudy
ReaderReport(u64),           // report_id -> ReaderReport
StudyReports(u64),           // study_id -> Vec<u64> (report_ids)
StudyReaders(u64),           // study_id -> Vec<Address>
ReaderStudies(Address),      // reader -> Vec<u64> (study_ids)
StatusStudies(StudyStatus),  // status -> Vec<u64> (study_ids)
PatientStudies(Address),     // patient -> Vec<u64> (study_ids)
NextStudyId,
NextReportId,
```

## Contract Interfaces

### `medical_imaging_ai` — 17 Methods

**Initialization & Admin:**
- `initialize(admin, default_warning_bps, default_critical_bps, default_min_samples)` — one-time setup
- `pause(admin)` / `unpause(admin)` — emergency controls

**Model Lifecycle:**
- `register_cnn_model(caller, model_id, modality, architecture_hash, layer_count, input_rows, input_cols, input_channels, training_samples, validation_accuracy_bps, training_dataset_hash, signing_pubkey)` — register with ed25519 public key
- `register_evaluator(admin, evaluator)` — authorize an address to call `record_evaluation`
- `revoke_evaluator(admin, evaluator)` — remove evaluator authorization
- `update_model_status(admin, model_id, new_status)` — admin override (reactivate, retire)
- `get_model(model_id)` — retrieve model metadata
- `is_model_active(model_id)` — returns bool, checks status is Active or Degraded

**Analysis Submission (oracle-attested):**
- `submit_analysis(caller, image_id, model_id, attestation_hash, signature, findings, overall_confidence_bps, processing_time_ms)` — verifies ed25519 signature against model's signing key, validates findings count <= 20 and bounding box invariants, rejects if model not active. Returns result_id.
- `submit_segmentation(caller, image_id, model_id, attestation_hash, signature, regions, processing_time_ms)` — same attestation verification, validates regions count <= 30. Returns seg_id.
- `get_analysis(result_id)` — retrieve analysis result
- `get_segmentation(seg_id)` — retrieve segmentation result
- `get_image_analyses(image_id)` — retrieve all result_ids for an image

**Performance Benchmarking:**
- `record_evaluation(caller, result_id, is_correct)` — caller must have ROLE_RADIOLOGIST. Updates rolling window. If window_total > window_size, resets window. Checks thresholds when min_sample_size reached: below warning → Degraded + emit `MDL_WARN`; below critical → Deactivated + emit `MDL_DEACT`. Returns updated ModelPerformance.
- `get_performance(model_id)` — retrieve current performance metrics
- `configure_thresholds(admin, model_id, warning_bps, critical_bps, min_samples, window_size)` — rejects if warning_bps <= critical_bps

### `medical_imaging` — 13 New/Modified Methods

**Study Management:**
- `create_study(caller, patient, modality, image_ids, required_readers)` — validates image_ids <= 500, required_readers 1-5. Status = Pending. Returns study_id.
- `assign_reader(caller, study_id, reader)` — caller must be admin or physician. Reader must have ROLE_RADIOLOGIST. Transitions Pending → Assigned when first reader assigned.
- `assign_arbitrator(caller, study_id, arbitrator)` — only valid in DiscrepancyReview status
- `link_ai_results(caller, study_id, result_ids)` — stores references to medical_imaging_ai result IDs

**Reader Workflow:**
- `submit_reader_report(reader, study_id, diagnosis_hash, findings_hash, findings_ref, agrees_with_ai, ai_accuracy_feedback_bps)` — reader must be assigned. Rejects duplicates. Auto-transitions: first submission → InReview; all submitted + diagnosis match → PreliminaryReport; all submitted + diagnosis mismatch → DiscrepancyReview. Returns report_id.
- `finalize_study(caller, study_id, final_report_ref)` — only from PreliminaryReport → FinalReport
- `amend_study(caller, study_id, amendment_ref, reason_hash)` — only from FinalReport or Amended → Amended

**Query Methods:**
- `get_study(study_id)` — retrieve study
- `get_reader_reports(caller, study_id)` — returns reports only if study in PreliminaryReport or later, or caller is admin
- `get_my_report(reader, study_id)` — reader retrieves own report regardless of study status
- `get_studies_by_reader(reader)` — all study IDs assigned to this reader
- `get_studies_by_status(status)` — all study IDs with this status
- `get_studies_by_patient(patient)` — all study IDs for this patient

**Modified Existing:**
- `run_edge_detection(caller, image_id, num_bins)` — now returns `EdgeDetectionResult` with structured spatial data instead of bare `u32`
- `run_segmentation(caller, image_id, region_count)` — expanded input validation

### Events

**`medical_imaging_ai`:**

| Event | Data | Trigger |
|---|---|---|
| MDL_REG | model_id | Model registered |
| ANALYSIS | result_id, image_id | Analysis submitted |
| SEG | seg_id, image_id | Segmentation submitted |
| MDL_WARN | model_id, rolling_accuracy_bps | Warning threshold crossed |
| MDL_DEACT | model_id, rolling_accuracy_bps | Auto-deactivated |
| MDL_REACT | model_id | Admin reactivated |
| MDL_RET | model_id | Model retired |

**`medical_imaging` (new):**

| Event | Data | Trigger |
|---|---|---|
| STUDY_NEW | study_id, patient | Study created |
| STUDY_ASGN | study_id, reader | Reader assigned |
| RPT_SUB | report_id, study_id, reader | Report submitted |
| DISCREP | study_id | Discrepancy detected |
| STUDY_FIN | study_id | Study finalized |
| STUDY_AMD | study_id | Study amended |

## Error Handling

### `medical_imaging_ai` Errors

```rust
#[contracterror]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    ContractPaused = 4,
    InvalidInput = 5,
    ModelNotFound = 6,
    ModelNotActive = 7,
    ModelAlreadyExists = 8,
    ResultNotFound = 9,
    SegmentationNotFound = 10,
    TooManyFindings = 11,        // > 20
    TooManyRegions = 12,         // > 30
    InvalidConfidence = 13,      // > 10000 bps
    InvalidSeverity = 14,        // not 1-5
    InvalidThreshold = 15,       // warning <= critical
    AttestationInvalid = 16,     // ed25519 signature verification failed
    DuplicateResult = 17,        // same model + same image already analyzed
    InsufficientSamples = 18,    // can't enforce thresholds yet
}
```

### `medical_imaging` New Errors (extend existing enum)

```rust
StudyNotFound = 14,
StudyNotInExpectedStatus = 15,
ReaderNotAssigned = 16,
ReaderAlreadySubmitted = 17,
TooManyReaders = 18,           // > 5
TooManyImages = 19,            // > 500
AllReadersNotSubmitted = 20,
ArbitratorNotAssigned = 21,
InvalidStatusTransition = 22,
ReportsNotYetAvailable = 23,
```

## Testing Strategy

### Unit Tests — `medical_imaging_ai` (~20 tests)

**Model lifecycle:**
- Register valid CNN model — metadata stored correctly
- Reject duplicate model_id
- Reject invalid inputs (0 layers, confidence > 10000, zero-dimension inputs)

**Attested submission:**
- Submit analysis with valid ed25519 signature — accepted
- Submit analysis with invalid signature — rejected (AttestationInvalid)
- Submit analysis with wrong model's key — rejected
- Reject submissions to inactive/deactivated model
- Reject > 20 findings, > 30 regions
- Reject invalid bounding box (x_min >= x_max)
- Reject invalid confidence (> 10000) and severity (not 1-5)

**Performance benchmarking:**
- Record evaluation — rolling window updates correctly
- Accuracy drops below warning → status = Degraded, MDL_WARN emitted
- Accuracy drops below critical → status = Deactivated, MDL_DEACT emitted
- No enforcement below min sample size
- Window resets when window_total exceeds window_size
- Admin reactivates deactivated model
- Admin retires active model
- Threshold configuration rejects warning <= critical

**Admin:**
- Pause blocks all submissions, unpause restores

### Unit Tests — `medical_imaging` Workflow (~18 tests)

**Study lifecycle:**
- Create study — valid inputs, status = Pending
- Reject > 5 readers, > 500 images
- Assign reader — must have ROLE_RADIOLOGIST, transitions to Assigned

**Blind review:**
- Submit report — first submission transitions → InReview
- Submit report — all readers agree (same diagnosis_hash) → PreliminaryReport
- Submit report — disagreement (different diagnosis_hash) → DiscrepancyReview
- Reject if reader not assigned
- Reject duplicate submission from same reader
- get_reader_reports blocked during InReview, allowed after PreliminaryReport
- get_my_report always works for the submitting reader

**Consensus variations:**
- Single-reader study — submit → PreliminaryReport directly
- Arbitrator flow — assign, submit, resolves DiscrepancyReview → PreliminaryReport
- Arbitrator assignment rejected if not in DiscrepancyReview

**Finalization:**
- Finalize — only from PreliminaryReport → FinalReport
- Amend — only from FinalReport or Amended → Amended
- Multiple amendments chain correctly
- Invalid transitions return StudyNotInExpectedStatus

**Queries:**
- get_studies_by_reader returns correct study IDs
- get_studies_by_status returns correct study IDs
- get_studies_by_patient returns correct study IDs

### Integration Test

End-to-end flow exercising both contracts:

1. Initialize both contracts
2. Register CNN model with ed25519 signing keypair in medical_imaging_ai
3. Upload image in medical_imaging
4. Submit attested analysis (3 findings with bounding boxes, valid signature)
5. Submit attested segmentation (5 regions with masks)
6. Create study linking image + AI results
7. Assign 2 readers
8. Reader 1 submits report (agrees_with_ai=true, feedback=9500 bps)
9. Reader 2 submits report (same diagnosis_hash, feedback=9200 bps)
10. Verify auto-transition to PreliminaryReport
11. Verify blind review was enforced (reports hidden during InReview)
12. Record evaluations in medical_imaging_ai → verify rolling window updated
13. Finalize study
14. Verify all data queryable and consistent across both contracts

### Edge Case Tests

- Model degrades mid-workflow — study with already-submitted results still completes
- Concurrent studies sharing same images — no interference
- Performance window reset — accuracy recovers after retraining and reactivation
- Discrepancy with 3 readers — 2 agree, 1 differs → still DiscrepancyReview (requires unanimous agreement)

## Acceptance Criteria Mapping

| Criterion | How It's Met |
|---|---|
| CNN-based image analysis | CnnModelMetadata with architecture, layers, dimensions; attested analysis results |
| Multiple imaging modalities | Existing ImagingModality enum (X-Ray, MRI, CT, Ultrasound, PET, Mammography) |
| Anomaly detection and segmentation | AnalysisResult with spatial findings + BoundingBox; SegmentationResult with labeled regions |
| Radiologist workflow integration | ImagingStudy state machine, configurable multi-reader blind review, discrepancy resolution |
| Diagnostic accuracy > 92% | ModelPerformance rolling window with warning at 92%, auto-deactivate at 85% |
| Analysis time < 30 seconds | processing_time_ms tracked per result, queryable via ModelPerformance |
| DICOM format support | Existing DicomMetadata in medical_imaging contract |
| Integration with existing contract | Extends medical_imaging, references existing types and patterns |
| GPU acceleration | Out of scope (off-chain concern), but processing_time_ms tracking enables monitoring |
