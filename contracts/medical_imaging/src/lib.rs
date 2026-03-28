#![no_std]
#![allow(clippy::too_many_arguments)]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
    String, Symbol, Vec,
};

const ROLE_TECHNICIAN: u32 = 1;
const ROLE_RADIOLOGIST: u32 = 2;
const ROLE_PHYSICIAN: u32 = 4;
const ROLE_RESEARCHER: u32 = 8;
const ROLE_AUDITOR: u32 = 16;
const ALL_ROLES: u32 =
    ROLE_TECHNICIAN | ROLE_RADIOLOGIST | ROLE_PHYSICIAN | ROLE_RESEARCHER | ROLE_AUDITOR;

const ADMIN: Symbol = symbol_short!("ADMIN");
const PAUSED: Symbol = symbol_short!("PAUSED");
const NEXT_IMG: Symbol = symbol_short!("NIMG");
const NEXT_ANN: Symbol = symbol_short!("NANN");
const NEXT_DGN: Symbol = symbol_short!("NDIAG");
const NEXT_DSE: Symbol = symbol_short!("NDOSE");
const SAFE_DSE: Symbol = symbol_short!("SAFE_DSE");
const NEXT_STD: Symbol = symbol_short!("NSTD");
const NEXT_RPT: Symbol = symbol_short!("NRPT");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ImagingModality {
    XRay,
    MRI,
    CT,
    Ultrasound,
    PET,
    Mammography,
    Custom(u32),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum CompressionAlgorithm {
    None,
    LosslessJpeg,
    Jpeg2000Lossless,
    Rle,
    Deflate,
    Custom(u32),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ProcessingKind {
    EdgeDetection,
    Segmentation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ShareScope {
    ViewOnly,
    Diagnostics,
    Research,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum AnnotationVisibility {
    Private,
    CareTeam,
    MultiInstitution,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct DicomMetadata {
    pub study_uid_hash: BytesN<32>,
    pub series_uid_hash: BytesN<32>,
    pub sop_uid_hash: BytesN<32>,
    pub modality_code_hash: BytesN<32>,
    pub body_part_hash: BytesN<32>,
    pub acquisition_timestamp: u64,
    pub rows: u32,
    pub cols: u32,
    pub bits_allocated: u32,
    pub pixel_spacing_microns: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct MedicalImage {
    pub image_id: u64,
    pub patient: Address,
    pub uploaded_by: Address,
    pub modality: ImagingModality,
    pub encrypted_ref: String,
    pub compression: CompressionAlgorithm,
    pub original_size_bytes: u64,
    pub compressed_size_bytes: u64,
    pub content_hash: BytesN<32>,
    pub encrypted_key_commitment: BytesN<32>,
    pub dicom_sop_uid_hash: BytesN<32>,
    pub uploaded_at: u64,
    pub integrity_verified_at: u64,
    pub tamper_detected: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ImageMetadataIndex {
    pub image_id: u64,
    pub extracted_by: Address,
    pub extracted_at: u64,
    pub token_hashes: Vec<BytesN<32>>,
    pub finding_hashes: Vec<BytesN<32>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ProcessingResult {
    pub image_id: u64,
    pub kind: ProcessingKind,
    pub processor: Address,
    pub algorithm_version: u32,
    pub output_ref: String,
    pub output_hash: BytesN<32>,
    pub quality_score_bps: u32,
    pub created_at: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct AiDiagnosticModel {
    pub model_id: BytesN<32>,
    pub owner: Address,
    pub model_name_hash: BytesN<32>,
    pub version: u32,
    pub modality: ImagingModality,
    pub is_active: bool,
    pub created_at: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct DiagnosticAssistance {
    pub diagnosis_id: u64,
    pub image_id: u64,
    pub model_id: BytesN<32>,
    pub clinician: Address,
    pub condition_hash: BytesN<32>,
    pub confidence_bps: u32,
    pub explanation_ref: String,
    pub recommended_action_hash: BytesN<32>,
    pub created_at: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ImageShareGrant {
    pub image_id: u64,
    pub patient: Address,
    pub grantee: Address,
    pub granted_by: Address,
    pub scope: ShareScope,
    pub expires_at: u64,
    pub zk_access_commitment: BytesN<32>,
    pub watermark_hash: BytesN<32>,
    pub revoked: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ImageAnnotation {
    pub annotation_id: u64,
    pub image_id: u64,
    pub author: Address,
    pub visibility: AnnotationVisibility,
    pub encrypted_note_ref: String,
    pub note_hash: BytesN<32>,
    pub region_hash: BytesN<32>,
    pub collaborators: Vec<Address>,
    pub created_at: u64,
    pub resolved: bool,
    pub resolved_by: Option<Address>,
    pub replies: Vec<BytesN<32>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ImageRecordLink {
    pub image_id: u64,
    pub record_contract: Address,
    pub medical_record_id: u64,
    pub linked_by: Address,
    pub linked_at: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct RadiationDoseEntry {
    pub dose_id: u64,
    pub patient: Address,
    pub image_id: u64,
    pub modality: ImagingModality,
    pub dose_mgy: u32,
    pub warning_threshold_mgy: u32,
    pub accumulated_mgy: u64,
    pub recorded_at: u64,
    pub threshold_exceeded: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct DoseSummary {
    pub patient: Address,
    pub total_mgy: u64,
    pub event_count: u32,
    pub last_recorded_at: u64,
    pub safety_alerts: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum StudyStatus {
    Pending,
    Assigned,
    InReview,
    PreliminaryReport,
    DiscrepancyReview,
    FinalReport,
    Amended,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ImagingStudy {
    pub study_id: u64,
    pub patient: Address,
    pub created_by: Address,
    pub modality: ImagingModality,
    pub image_ids: Vec<u64>,
    pub ai_result_ids: Vec<u64>,
    pub required_readers: u32,
    pub status: StudyStatus,
    pub created_at: u64,
    pub finalized_at: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ReaderReport {
    pub report_id: u64,
    pub study_id: u64,
    pub reader: Address,
    pub diagnosis_hash: BytesN<32>,
    pub findings_hash: BytesN<32>,
    pub findings_ref: String,
    pub agrees_with_ai: bool,
    pub ai_accuracy_feedback_bps: u32,
    pub submitted_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Roles(Address),
    Image(u64),
    ImageIds,
    Dicom(u64),
    ImageByPatient(Address),
    ImageByModality(BytesN<32>),
    ImageByBodyPart(BytesN<32>),
    SopLookup(BytesN<32>),
    MetadataIndex(u64),
    Processing(u64, ProcessingKind),
    Model(BytesN<32>),
    Diagnosis(u64),
    Share(u64, Address),
    Annotation(u64),
    ImageAnnotations(u64),
    Link(u64),
    DoseEntry(u64),
    DoseSummary(Address),
    Study(u64),
    ReaderReportEntry(u64),
    StudyReports(u64),
    StudyReaders(u64),
    ReaderStudies(Address),
    StatusStudies(u32),
    PatientStudies(Address),
    StudyArbitrator(u64),
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
    ImageNotFound = 6,
    ModelNotFound = 7,
    ShareNotFound = 8,
    ShareExpired = 9,
    AnnotationNotFound = 10,
    LinkNotFound = 11,
    DuplicateDicomSop = 12,
    IntegrityMismatch = 13,
    StudyNotFound = 14,
    StudyNotInExpectedStatus = 15,
    ReaderNotAssigned = 16,
    ReaderAlreadySubmitted = 17,
    TooManyReaders = 18,
    TooManyImages = 19,
    AllReadersNotSubmitted = 20,
    ArbitratorNotAssigned = 21,
    InvalidStatusTransition = 22,
    ReportsNotYetAvailable = 23,
}

#[contract]
pub struct MedicalImagingContract;

#[contractimpl]
impl MedicalImagingContract {
    pub fn initialize(env: Env, admin: Address, safety_threshold_mgy: u32) -> Result<bool, Error> {
        admin.require_auth();
        if env.storage().instance().has(&ADMIN) {
            return Err(Error::AlreadyInitialized);
        }
        if safety_threshold_mgy == 0 {
            return Err(Error::InvalidInput);
        }

        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&PAUSED, &false);
        env.storage().instance().set(&NEXT_IMG, &1u64);
        env.storage().instance().set(&NEXT_ANN, &1u64);
        env.storage().instance().set(&NEXT_DGN, &1u64);
        env.storage().instance().set(&NEXT_DSE, &1u64);
        env.storage()
            .instance()
            .set(&SAFE_DSE, &safety_threshold_mgy);
        env.storage().instance().set(&NEXT_STD, &1u64);
        env.storage().instance().set(&NEXT_RPT, &1u64);
        env.storage()
            .persistent()
            .set(&DataKey::ImageIds, &Vec::<u64>::new(&env));
        Ok(true)
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
        env.storage()
            .persistent()
            .set(&DataKey::Roles(user), &(role_mask & ALL_ROLES));
        Ok(true)
    }

    pub fn set_safety_threshold(
        env: Env,
        caller: Address,
        safety_threshold_mgy: u32,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        if safety_threshold_mgy == 0 {
            return Err(Error::InvalidInput);
        }
        env.storage()
            .instance()
            .set(&SAFE_DSE, &safety_threshold_mgy);
        Ok(true)
    }

    pub fn upload_image(
        env: Env,
        caller: Address,
        patient: Address,
        modality: ImagingModality,
        encrypted_ref: String,
        compression: CompressionAlgorithm,
        original_size_bytes: u64,
        compressed_size_bytes: u64,
        content_hash: BytesN<32>,
        encrypted_key_commitment: BytesN<32>,
        dicom: DicomMetadata,
    ) -> Result<u64, Error> {
        caller.require_auth();
        Self::require_not_paused(&env)?;
        Self::require_role_or_admin(&env, &caller, ROLE_TECHNICIAN)?;

        if encrypted_ref.len() < 8 || original_size_bytes == 0 || compressed_size_bytes == 0 {
            return Err(Error::InvalidInput);
        }
        if compressed_size_bytes > original_size_bytes {
            return Err(Error::InvalidInput);
        }
        if dicom.rows == 0 || dicom.cols == 0 || dicom.bits_allocated == 0 {
            return Err(Error::InvalidInput);
        }
        if env
            .storage()
            .persistent()
            .has(&DataKey::SopLookup(dicom.sop_uid_hash.clone()))
        {
            return Err(Error::DuplicateDicomSop);
        }

        let id = Self::next_counter(&env, &NEXT_IMG);
        let now = env.ledger().timestamp();

        let image = MedicalImage {
            image_id: id,
            patient: patient.clone(),
            uploaded_by: caller.clone(),
            modality,
            encrypted_ref,
            compression,
            original_size_bytes,
            compressed_size_bytes,
            content_hash,
            encrypted_key_commitment,
            dicom_sop_uid_hash: dicom.sop_uid_hash.clone(),
            uploaded_at: now,
            integrity_verified_at: 0,
            tamper_detected: false,
        };

        env.storage().persistent().set(&DataKey::Image(id), &image);
        env.storage().persistent().set(&DataKey::Dicom(id), &dicom);
        env.storage()
            .persistent()
            .set(&DataKey::SopLookup(dicom.sop_uid_hash), &id);

        Self::append_u64(&env, DataKey::ImageIds, id);
        Self::append_u64(&env, DataKey::ImageByPatient(patient), id);
        Self::append_u64(&env, DataKey::ImageByModality(dicom.modality_code_hash), id);
        Self::append_u64(&env, DataKey::ImageByBodyPart(dicom.body_part_hash), id);

        env.events()
            .publish((symbol_short!("IMG_UPLD"),), (id, caller));
        Ok(id)
    }

    pub fn extract_and_index_metadata(
        env: Env,
        caller: Address,
        image_id: u64,
        token_hashes: Vec<BytesN<32>>,
        finding_hashes: Vec<BytesN<32>>,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_role_or_admin(&env, &caller, ROLE_RADIOLOGIST)?;
        Self::require_image_exists(&env, image_id)?;
        if token_hashes.is_empty() {
            return Err(Error::InvalidInput);
        }

        let index = ImageMetadataIndex {
            image_id,
            extracted_by: caller.clone(),
            extracted_at: env.ledger().timestamp(),
            token_hashes,
            finding_hashes,
        };

        env.storage()
            .persistent()
            .set(&DataKey::MetadataIndex(image_id), &index);
        env.events()
            .publish((symbol_short!("IMG_META"),), (image_id, caller));
        Ok(true)
    }

    pub fn run_edge_detection(
        env: Env,
        caller: Address,
        image_id: u64,
        bins: Vec<u32>,
        gradient_threshold: u32,
        output_ref: String,
        output_hash: BytesN<32>,
        algorithm_version: u32,
    ) -> Result<ProcessingResult, Error> {
        caller.require_auth();
        Self::require_role_or_admin(&env, &caller, ROLE_RADIOLOGIST)?;
        Self::require_image_exists(&env, image_id)?;

        if bins.len() < 2 || gradient_threshold == 0 || output_ref.is_empty() {
            return Err(Error::InvalidInput);
        }

        let mut edges = 0u32;
        let mut prev = bins.get(0).unwrap_or(0);
        for idx in 1..bins.len() {
            let current = bins.get(idx).unwrap_or(0);
            let diff = if current > prev {
                current.saturating_sub(prev)
            } else {
                prev.saturating_sub(current)
            };
            if diff >= gradient_threshold {
                edges = edges.saturating_add(1);
            }
            prev = current;
        }

        let denominator = bins.len().saturating_sub(1);
        let quality = if denominator == 0 {
            0
        } else {
            edges
                .checked_mul(10_000)
                .and_then(|value| value.checked_div(denominator))
                .unwrap_or(0)
        };

        let result = ProcessingResult {
            image_id,
            kind: ProcessingKind::EdgeDetection,
            processor: caller.clone(),
            algorithm_version,
            output_ref,
            output_hash,
            quality_score_bps: quality,
            created_at: env.ledger().timestamp(),
        };

        env.storage().persistent().set(
            &DataKey::Processing(image_id, ProcessingKind::EdgeDetection),
            &result,
        );
        env.events()
            .publish((symbol_short!("IMG_EDGE"),), (image_id, caller));
        Ok(result)
    }

    pub fn run_segmentation(
        env: Env,
        caller: Address,
        image_id: u64,
        bins: Vec<u32>,
        lower_bound: u32,
        upper_bound: u32,
        output_ref: String,
        output_hash: BytesN<32>,
        algorithm_version: u32,
    ) -> Result<ProcessingResult, Error> {
        caller.require_auth();
        Self::require_role_or_admin(&env, &caller, ROLE_RADIOLOGIST)?;
        Self::require_image_exists(&env, image_id)?;

        if bins.is_empty() || lower_bound > upper_bound || output_ref.is_empty() {
            return Err(Error::InvalidInput);
        }

        let mut in_segment = 0u32;
        for value in bins.iter() {
            if value >= lower_bound && value <= upper_bound {
                in_segment = in_segment.saturating_add(1);
            }
        }

        let quality = in_segment
            .checked_mul(10_000)
            .and_then(|value| value.checked_div(bins.len()))
            .unwrap_or(0);

        let result = ProcessingResult {
            image_id,
            kind: ProcessingKind::Segmentation,
            processor: caller.clone(),
            algorithm_version,
            output_ref,
            output_hash,
            quality_score_bps: quality,
            created_at: env.ledger().timestamp(),
        };

        env.storage().persistent().set(
            &DataKey::Processing(image_id, ProcessingKind::Segmentation),
            &result,
        );
        env.events()
            .publish((symbol_short!("IMG_SEGM"),), (image_id, caller));
        Ok(result)
    }

    pub fn register_ai_model(
        env: Env,
        caller: Address,
        model_id: BytesN<32>,
        model_name_hash: BytesN<32>,
        version: u32,
        modality: ImagingModality,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_role_or_admin(&env, &caller, ROLE_PHYSICIAN)?;
        if version == 0 {
            return Err(Error::InvalidInput);
        }

        let model = AiDiagnosticModel {
            model_id: model_id.clone(),
            owner: caller.clone(),
            model_name_hash,
            version,
            modality,
            is_active: true,
            created_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::Model(model_id), &model);
        env.events().publish((symbol_short!("IMG_MDL"),), caller);
        Ok(true)
    }

    pub fn submit_diagnostic_assistance(
        env: Env,
        caller: Address,
        image_id: u64,
        model_id: BytesN<32>,
        condition_hash: BytesN<32>,
        confidence_bps: u32,
        explanation_ref: String,
        recommended_action_hash: BytesN<32>,
    ) -> Result<u64, Error> {
        caller.require_auth();
        Self::require_role_or_admin(&env, &caller, ROLE_PHYSICIAN)?;
        Self::require_image_exists(&env, image_id)?;

        let model: AiDiagnosticModel = env
            .storage()
            .persistent()
            .get(&DataKey::Model(model_id.clone()))
            .ok_or(Error::ModelNotFound)?;
        if !model.is_active || confidence_bps > 10_000 || explanation_ref.is_empty() {
            return Err(Error::InvalidInput);
        }

        let diagnosis_id = Self::next_counter(&env, &NEXT_DGN);
        let diagnosis = DiagnosticAssistance {
            diagnosis_id,
            image_id,
            model_id,
            clinician: caller.clone(),
            condition_hash,
            confidence_bps,
            explanation_ref,
            recommended_action_hash,
            created_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::Diagnosis(diagnosis_id), &diagnosis);
        env.events()
            .publish((symbol_short!("IMG_AIDI"),), (image_id, diagnosis_id));
        Ok(diagnosis_id)
    }

    pub fn grant_image_access(
        env: Env,
        caller: Address,
        image_id: u64,
        grantee: Address,
        scope: ShareScope,
        expires_at: u64,
        zk_access_commitment: BytesN<32>,
        watermark_hash: BytesN<32>,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_not_paused(&env)?;

        let image: MedicalImage = env
            .storage()
            .persistent()
            .get(&DataKey::Image(image_id))
            .ok_or(Error::ImageNotFound)?;

        if expires_at <= env.ledger().timestamp() {
            return Err(Error::InvalidInput);
        }

        if caller != image.patient {
            Self::require_admin(&env, &caller)?;
        }

        let grant = ImageShareGrant {
            image_id,
            patient: image.patient,
            grantee: grantee.clone(),
            granted_by: caller.clone(),
            scope,
            expires_at,
            zk_access_commitment,
            watermark_hash,
            revoked: false,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Share(image_id, grantee.clone()), &grant);
        env.events()
            .publish((symbol_short!("IMG_SHAR"),), (image_id, grantee));
        Ok(true)
    }

    pub fn revoke_image_access(
        env: Env,
        caller: Address,
        image_id: u64,
        grantee: Address,
    ) -> Result<bool, Error> {
        caller.require_auth();
        let image: MedicalImage = env
            .storage()
            .persistent()
            .get(&DataKey::Image(image_id))
            .ok_or(Error::ImageNotFound)?;

        if caller != image.patient {
            Self::require_admin(&env, &caller)?;
        }

        let mut grant: ImageShareGrant = env
            .storage()
            .persistent()
            .get(&DataKey::Share(image_id, grantee.clone()))
            .ok_or(Error::ShareNotFound)?;
        grant.revoked = true;
        env.storage()
            .persistent()
            .set(&DataKey::Share(image_id, grantee.clone()), &grant);

        env.events()
            .publish((symbol_short!("IMG_RVOK"),), (image_id, grantee));
        Ok(true)
    }

    pub fn verify_share_access(env: Env, image_id: u64, viewer: Address) -> Result<bool, Error> {
        let grant: ImageShareGrant = env
            .storage()
            .persistent()
            .get(&DataKey::Share(image_id, viewer))
            .ok_or(Error::ShareNotFound)?;

        if grant.revoked {
            return Ok(false);
        }
        if grant.expires_at <= env.ledger().timestamp() {
            return Err(Error::ShareExpired);
        }

        Ok(true)
    }

    pub fn verify_image_integrity(
        env: Env,
        caller: Address,
        image_id: u64,
        observed_hash: BytesN<32>,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_role_or_admin(&env, &caller, ROLE_AUDITOR)?;

        let mut image: MedicalImage = env
            .storage()
            .persistent()
            .get(&DataKey::Image(image_id))
            .ok_or(Error::ImageNotFound)?;

        image.integrity_verified_at = env.ledger().timestamp();
        let matched = image.content_hash == observed_hash;
        image.tamper_detected = !matched;

        env.storage()
            .persistent()
            .set(&DataKey::Image(image_id), &image);

        if !matched {
            env.events()
                .publish((symbol_short!("IMG_TMPR"),), (image_id, caller));
            return Ok(false);
        }

        Ok(true)
    }

    pub fn add_annotation(
        env: Env,
        caller: Address,
        image_id: u64,
        visibility: AnnotationVisibility,
        encrypted_note_ref: String,
        note_hash: BytesN<32>,
        region_hash: BytesN<32>,
        collaborators: Vec<Address>,
    ) -> Result<u64, Error> {
        caller.require_auth();
        Self::require_role_or_admin(&env, &caller, ROLE_PHYSICIAN)?;
        Self::require_image_exists(&env, image_id)?;
        if encrypted_note_ref.is_empty() {
            return Err(Error::InvalidInput);
        }

        let annotation_id = Self::next_counter(&env, &NEXT_ANN);
        let ann = ImageAnnotation {
            annotation_id,
            image_id,
            author: caller.clone(),
            visibility,
            encrypted_note_ref,
            note_hash,
            region_hash,
            collaborators,
            created_at: env.ledger().timestamp(),
            resolved: false,
            resolved_by: None,
            replies: Vec::new(&env),
        };

        env.storage()
            .persistent()
            .set(&DataKey::Annotation(annotation_id), &ann);
        Self::append_u64(&env, DataKey::ImageAnnotations(image_id), annotation_id);
        env.events()
            .publish((symbol_short!("IMG_ANN"),), (image_id, annotation_id));
        Ok(annotation_id)
    }

    pub fn add_annotation_reply(
        env: Env,
        caller: Address,
        annotation_id: u64,
        reply_hash: BytesN<32>,
    ) -> Result<bool, Error> {
        caller.require_auth();
        let mut ann: ImageAnnotation = env
            .storage()
            .persistent()
            .get(&DataKey::Annotation(annotation_id))
            .ok_or(Error::AnnotationNotFound)?;

        let is_collaborator = ann.collaborators.iter().any(|c| c == caller);
        if caller != ann.author && !is_collaborator {
            Self::require_admin(&env, &caller)?;
        }

        ann.replies.push_back(reply_hash);
        env.storage()
            .persistent()
            .set(&DataKey::Annotation(annotation_id), &ann);
        Ok(true)
    }

    pub fn resolve_annotation(
        env: Env,
        caller: Address,
        annotation_id: u64,
    ) -> Result<bool, Error> {
        caller.require_auth();
        let mut ann: ImageAnnotation = env
            .storage()
            .persistent()
            .get(&DataKey::Annotation(annotation_id))
            .ok_or(Error::AnnotationNotFound)?;

        if caller != ann.author {
            Self::require_admin(&env, &caller)?;
        }

        ann.resolved = true;
        ann.resolved_by = Some(caller.clone());
        env.storage()
            .persistent()
            .set(&DataKey::Annotation(annotation_id), &ann);

        env.events()
            .publish((symbol_short!("ANN_RSLV"),), annotation_id);
        Ok(true)
    }

    pub fn link_image_to_record(
        env: Env,
        caller: Address,
        image_id: u64,
        record_contract: Address,
        medical_record_id: u64,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_role_or_admin(&env, &caller, ROLE_PHYSICIAN)?;
        Self::require_image_exists(&env, image_id)?;

        let link = ImageRecordLink {
            image_id,
            record_contract,
            medical_record_id,
            linked_by: caller.clone(),
            linked_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::Link(image_id), &link);
        env.events()
            .publish((symbol_short!("IMG_LINK"),), (image_id, medical_record_id));
        Ok(true)
    }

    pub fn record_radiation_dose(
        env: Env,
        caller: Address,
        patient: Address,
        image_id: u64,
        modality: ImagingModality,
        dose_mgy: u32,
    ) -> Result<u64, Error> {
        caller.require_auth();
        Self::require_role_or_admin(&env, &caller, ROLE_TECHNICIAN)?;
        Self::require_image_exists(&env, image_id)?;
        if dose_mgy == 0 {
            return Err(Error::InvalidInput);
        }

        let threshold: u32 = env
            .storage()
            .instance()
            .get(&SAFE_DSE)
            .ok_or(Error::NotInitialized)?;

        let mut summary: DoseSummary = env
            .storage()
            .persistent()
            .get(&DataKey::DoseSummary(patient.clone()))
            .unwrap_or(DoseSummary {
                patient: patient.clone(),
                total_mgy: 0,
                event_count: 0,
                last_recorded_at: 0,
                safety_alerts: 0,
            });

        summary.total_mgy = summary.total_mgy.saturating_add(dose_mgy as u64);
        summary.event_count = summary.event_count.saturating_add(1);
        summary.last_recorded_at = env.ledger().timestamp();
        let exceeded = summary.total_mgy >= threshold as u64;
        if exceeded {
            summary.safety_alerts = summary.safety_alerts.saturating_add(1);
        }

        let dose_id = Self::next_counter(&env, &NEXT_DSE);
        let dose = RadiationDoseEntry {
            dose_id,
            patient: patient.clone(),
            image_id,
            modality,
            dose_mgy,
            warning_threshold_mgy: threshold,
            accumulated_mgy: summary.total_mgy,
            recorded_at: summary.last_recorded_at,
            threshold_exceeded: exceeded,
        };

        env.storage()
            .persistent()
            .set(&DataKey::DoseEntry(dose_id), &dose);
        env.storage()
            .persistent()
            .set(&DataKey::DoseSummary(patient.clone()), &summary);
        env.events()
            .publish((symbol_short!("IMG_DOSE"),), (patient, dose_id));

        Ok(dose_id)
    }

    pub fn get_image(env: Env, image_id: u64) -> Option<MedicalImage> {
        env.storage().persistent().get(&DataKey::Image(image_id))
    }

    pub fn get_dicom(env: Env, image_id: u64) -> Option<DicomMetadata> {
        env.storage().persistent().get(&DataKey::Dicom(image_id))
    }

    pub fn get_image_by_sop(env: Env, sop_uid_hash: BytesN<32>) -> Option<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::SopLookup(sop_uid_hash))
    }

    pub fn list_images_by_patient(env: Env, patient: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::ImageByPatient(patient))
            .unwrap_or(Vec::new(&env))
    }

    pub fn list_images_by_modality_hash(env: Env, modality_code_hash: BytesN<32>) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::ImageByModality(modality_code_hash))
            .unwrap_or(Vec::new(&env))
    }

    pub fn list_images_by_body_part_hash(env: Env, body_part_hash: BytesN<32>) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::ImageByBodyPart(body_part_hash))
            .unwrap_or(Vec::new(&env))
    }

    pub fn get_compression_ratio_bps(env: Env, image_id: u64) -> Result<u32, Error> {
        let image: MedicalImage = env
            .storage()
            .persistent()
            .get(&DataKey::Image(image_id))
            .ok_or(Error::ImageNotFound)?;

        if image.original_size_bytes == 0 {
            return Err(Error::InvalidInput);
        }

        let ratio = image
            .compressed_size_bytes
            .checked_mul(10_000)
            .and_then(|value| value.checked_div(image.original_size_bytes))
            .ok_or(Error::InvalidInput)?;

        let ratio_u32 = u32::try_from(ratio).map_err(|_| Error::InvalidInput)?;
        Ok(ratio_u32)
    }

    pub fn get_processing_result(
        env: Env,
        image_id: u64,
        kind: ProcessingKind,
    ) -> Option<ProcessingResult> {
        env.storage()
            .persistent()
            .get(&DataKey::Processing(image_id, kind))
    }

    pub fn get_metadata_index(env: Env, image_id: u64) -> Option<ImageMetadataIndex> {
        env.storage()
            .persistent()
            .get(&DataKey::MetadataIndex(image_id))
    }

    pub fn get_model(env: Env, model_id: BytesN<32>) -> Option<AiDiagnosticModel> {
        env.storage().persistent().get(&DataKey::Model(model_id))
    }

    pub fn get_diagnostic(env: Env, diagnosis_id: u64) -> Option<DiagnosticAssistance> {
        env.storage()
            .persistent()
            .get(&DataKey::Diagnosis(diagnosis_id))
    }

    pub fn get_share_grant(env: Env, image_id: u64, grantee: Address) -> Option<ImageShareGrant> {
        env.storage()
            .persistent()
            .get(&DataKey::Share(image_id, grantee))
    }

    pub fn get_annotation(env: Env, annotation_id: u64) -> Option<ImageAnnotation> {
        env.storage()
            .persistent()
            .get(&DataKey::Annotation(annotation_id))
    }

    pub fn list_annotations_for_image(env: Env, image_id: u64) -> Vec<ImageAnnotation> {
        let ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::ImageAnnotations(image_id))
            .unwrap_or(Vec::new(&env));

        let mut out = Vec::new(&env);
        for id in ids.iter() {
            if let Some(ann) = env.storage().persistent().get(&DataKey::Annotation(id)) {
                out.push_back(ann);
            }
        }
        out
    }

    pub fn get_image_record_link(env: Env, image_id: u64) -> Option<ImageRecordLink> {
        env.storage().persistent().get(&DataKey::Link(image_id))
    }

    pub fn get_dose_entry(env: Env, dose_id: u64) -> Option<RadiationDoseEntry> {
        env.storage().persistent().get(&DataKey::DoseEntry(dose_id))
    }

    pub fn get_dose_summary(env: Env, patient: Address) -> Option<DoseSummary> {
        env.storage()
            .persistent()
            .get(&DataKey::DoseSummary(patient))
    }

    // ── Study Creation & Reader Assignment ──

    pub fn create_study(
        env: Env,
        caller: Address,
        patient: Address,
        modality: ImagingModality,
        image_ids: Vec<u64>,
        required_readers: u32,
    ) -> Result<u64, Error> {
        caller.require_auth();
        Self::require_not_paused(&env)?;
        Self::require_role_or_admin(&env, &caller, ROLE_PHYSICIAN)?;

        if required_readers == 0 || required_readers > 5 {
            return Err(Error::TooManyReaders);
        }
        if image_ids.is_empty() || image_ids.len() > 500 {
            return Err(Error::TooManyImages);
        }
        for img_id in image_ids.iter() {
            Self::require_image_exists(&env, img_id)?;
        }

        let study_id = Self::next_counter(&env, &NEXT_STD);
        let now = env.ledger().timestamp();

        let study = ImagingStudy {
            study_id,
            patient: patient.clone(),
            created_by: caller.clone(),
            modality,
            image_ids,
            ai_result_ids: Vec::new(&env),
            required_readers,
            status: StudyStatus::Pending,
            created_at: now,
            finalized_at: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Study(study_id), &study);
        Self::append_u64(
            &env,
            DataKey::PatientStudies(patient),
            study_id,
        );
        Self::append_u64(
            &env,
            DataKey::StatusStudies(Self::status_to_u32(&StudyStatus::Pending)),
            study_id,
        );

        env.events()
            .publish((symbol_short!("STDY_NEW"),), (study_id, caller));
        Ok(study_id)
    }

    pub fn assign_reader(
        env: Env,
        caller: Address,
        study_id: u64,
        reader: Address,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_role_or_admin(&env, &caller, ROLE_PHYSICIAN)?;
        Self::require_role_or_admin(&env, &reader, ROLE_RADIOLOGIST)?;

        let mut study: ImagingStudy = env
            .storage()
            .persistent()
            .get(&DataKey::Study(study_id))
            .ok_or(Error::StudyNotFound)?;

        if study.status != StudyStatus::Pending && study.status != StudyStatus::Assigned {
            return Err(Error::StudyNotInExpectedStatus);
        }

        let readers: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::StudyReaders(study_id))
            .unwrap_or(Vec::new(&env));

        if readers.iter().any(|r| r == reader) {
            return Err(Error::InvalidInput);
        }
        if readers.len() >= study.required_readers {
            return Err(Error::TooManyReaders);
        }

        let mut new_readers = readers;
        new_readers.push_back(reader.clone());
        env.storage()
            .persistent()
            .set(&DataKey::StudyReaders(study_id), &new_readers);

        Self::append_u64(&env, DataKey::ReaderStudies(reader.clone()), study_id);

        if study.status == StudyStatus::Pending {
            let old_status = study.status;
            study.status = StudyStatus::Assigned;
            env.storage()
                .persistent()
                .set(&DataKey::Study(study_id), &study);
            Self::update_status_index(&env, study_id, &old_status, &StudyStatus::Assigned);
        }

        env.events()
            .publish((symbol_short!("STDY_ASG"),), (study_id, reader));
        Ok(true)
    }

    pub fn assign_arbitrator(
        env: Env,
        caller: Address,
        study_id: u64,
        arbitrator: Address,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_role_or_admin(&env, &caller, ROLE_PHYSICIAN)?;

        let study: ImagingStudy = env
            .storage()
            .persistent()
            .get(&DataKey::Study(study_id))
            .ok_or(Error::StudyNotFound)?;

        if study.status != StudyStatus::DiscrepancyReview {
            return Err(Error::StudyNotInExpectedStatus);
        }

        env.storage()
            .persistent()
            .set(&DataKey::StudyArbitrator(study_id), &arbitrator);
        Ok(true)
    }

    pub fn link_ai_results(
        env: Env,
        caller: Address,
        study_id: u64,
        result_ids: Vec<u64>,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_role_or_admin(&env, &caller, ROLE_PHYSICIAN)?;

        let mut study: ImagingStudy = env
            .storage()
            .persistent()
            .get(&DataKey::Study(study_id))
            .ok_or(Error::StudyNotFound)?;

        for rid in result_ids.iter() {
            study.ai_result_ids.push_back(rid);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Study(study_id), &study);
        Ok(true)
    }

    pub fn get_study(env: Env, study_id: u64) -> Option<ImagingStudy> {
        env.storage().persistent().get(&DataKey::Study(study_id))
    }

    pub fn get_studies_by_reader(env: Env, reader: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::ReaderStudies(reader))
            .unwrap_or(Vec::new(&env))
    }

    pub fn get_studies_by_status(env: Env, status: StudyStatus) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::StatusStudies(Self::status_to_u32(&status)))
            .unwrap_or(Vec::new(&env))
    }

    pub fn get_studies_by_patient(env: Env, patient: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::PatientStudies(patient))
            .unwrap_or(Vec::new(&env))
    }

    // ── Reader Report Submission ──

    pub fn submit_reader_report(
        env: Env,
        reader: Address,
        study_id: u64,
        diagnosis_hash: BytesN<32>,
        findings_hash: BytesN<32>,
        findings_ref: String,
        agrees_with_ai: bool,
        ai_accuracy_feedback_bps: u32,
    ) -> Result<u64, Error> {
        reader.require_auth();
        Self::require_not_paused(&env)?;

        let mut study: ImagingStudy = env
            .storage()
            .persistent()
            .get(&DataKey::Study(study_id))
            .ok_or(Error::StudyNotFound)?;

        if study.status != StudyStatus::Assigned
            && study.status != StudyStatus::InReview
            && study.status != StudyStatus::DiscrepancyReview
        {
            return Err(Error::StudyNotInExpectedStatus);
        }

        // Check reader is assigned or is the arbitrator
        let readers: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::StudyReaders(study_id))
            .unwrap_or(Vec::new(&env));

        let is_reader = readers.iter().any(|r| r == reader);
        let is_arbitrator = env
            .storage()
            .persistent()
            .get::<_, Address>(&DataKey::StudyArbitrator(study_id))
            .map(|a| a == reader)
            .unwrap_or(false);

        if !is_reader && !is_arbitrator {
            return Err(Error::ReaderNotAssigned);
        }

        // Check for duplicate submission
        let existing_report_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::StudyReports(study_id))
            .unwrap_or(Vec::new(&env));

        for rid in existing_report_ids.iter() {
            let existing: ReaderReport = env
                .storage()
                .persistent()
                .get(&DataKey::ReaderReportEntry(rid))
                .unwrap();
            if existing.reader == reader {
                return Err(Error::ReaderAlreadySubmitted);
            }
        }

        if ai_accuracy_feedback_bps > 10_000 {
            return Err(Error::InvalidInput);
        }

        let report_id = Self::next_counter(&env, &NEXT_RPT);
        let report = ReaderReport {
            report_id,
            study_id,
            reader: reader.clone(),
            diagnosis_hash: diagnosis_hash.clone(),
            findings_hash,
            findings_ref,
            agrees_with_ai,
            ai_accuracy_feedback_bps,
            submitted_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::ReaderReportEntry(report_id), &report);
        Self::append_u64(&env, DataKey::StudyReports(study_id), report_id);

        // Auto-transition logic
        let old_status = study.status;

        if old_status == StudyStatus::Assigned {
            study.status = StudyStatus::InReview;
            env.storage()
                .persistent()
                .set(&DataKey::Study(study_id), &study);
            Self::update_status_index(&env, study_id, &old_status, &study.status);
        }

        // Check if arbitrator submitted in DiscrepancyReview
        if is_arbitrator && old_status == StudyStatus::DiscrepancyReview {
            study.status = StudyStatus::PreliminaryReport;
            env.storage()
                .persistent()
                .set(&DataKey::Study(study_id), &study);
            Self::update_status_index(&env, study_id, &old_status, &study.status);
        }

        // Check if all required readers have submitted
        let updated_report_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::StudyReports(study_id))
            .unwrap_or(Vec::new(&env));

        // Count only reports from assigned readers (not arbitrator)
        let mut reader_report_count = 0u32;
        let mut all_diagnosis_hashes: Vec<BytesN<32>> = Vec::new(&env);
        for rid in updated_report_ids.iter() {
            let rpt: ReaderReport = env
                .storage()
                .persistent()
                .get(&DataKey::ReaderReportEntry(rid))
                .unwrap();
            if readers.iter().any(|r| r == rpt.reader) {
                reader_report_count = reader_report_count.saturating_add(1);
                all_diagnosis_hashes.push_back(rpt.diagnosis_hash);
            }
        }

        if reader_report_count >= study.required_readers
            && study.status != StudyStatus::PreliminaryReport
        {
            // Reload study in case it was updated
            let mut study: ImagingStudy = env
                .storage()
                .persistent()
                .get(&DataKey::Study(study_id))
                .unwrap();
            let current_status = study.status;

            // Check if all hashes match
            let first_hash = all_diagnosis_hashes.get(0).unwrap();
            let all_match = all_diagnosis_hashes.iter().all(|h| h == first_hash);

            if all_match {
                study.status = StudyStatus::PreliminaryReport;
                env.storage()
                    .persistent()
                    .set(&DataKey::Study(study_id), &study);
                Self::update_status_index(
                    &env,
                    study_id,
                    &current_status,
                    &StudyStatus::PreliminaryReport,
                );
            } else {
                study.status = StudyStatus::DiscrepancyReview;
                env.storage()
                    .persistent()
                    .set(&DataKey::Study(study_id), &study);
                Self::update_status_index(
                    &env,
                    study_id,
                    &current_status,
                    &StudyStatus::DiscrepancyReview,
                );
                env.events()
                    .publish((symbol_short!("DISCREP"),), study_id);
            }
        }

        env.events()
            .publish((symbol_short!("RPT_SUB"),), (study_id, report_id, reader));
        Ok(report_id)
    }

    pub fn get_reader_reports(env: Env, caller: Address, study_id: u64) -> Vec<ReaderReport> {
        let study: Option<ImagingStudy> = env
            .storage()
            .persistent()
            .get(&DataKey::Study(study_id));

        let can_view = match &study {
            Some(s) => {
                matches!(
                    s.status,
                    StudyStatus::PreliminaryReport
                        | StudyStatus::DiscrepancyReview
                        | StudyStatus::FinalReport
                        | StudyStatus::Amended
                ) || Self::require_admin(&env, &caller).is_ok()
            }
            None => false,
        };

        if !can_view {
            return Vec::new(&env);
        }

        let report_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::StudyReports(study_id))
            .unwrap_or(Vec::new(&env));

        let mut out = Vec::new(&env);
        for rid in report_ids.iter() {
            if let Some(rpt) = env
                .storage()
                .persistent()
                .get(&DataKey::ReaderReportEntry(rid))
            {
                out.push_back(rpt);
            }
        }
        out
    }

    pub fn get_my_report(env: Env, reader: Address, study_id: u64) -> ReaderReport {
        reader.require_auth();
        let report_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::StudyReports(study_id))
            .unwrap_or(Vec::new(&env));

        for rid in report_ids.iter() {
            let rpt: ReaderReport = env
                .storage()
                .persistent()
                .get(&DataKey::ReaderReportEntry(rid))
                .unwrap();
            if rpt.reader == reader {
                return rpt;
            }
        }
        panic!("report not found");
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&ADMIN)
            .ok_or(Error::NotInitialized)?;
        if &admin != caller {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    fn require_not_paused(env: &Env) -> Result<(), Error> {
        let paused: bool = env.storage().instance().get(&PAUSED).unwrap_or(false);
        if paused {
            return Err(Error::ContractPaused);
        }
        Ok(())
    }

    fn require_role_or_admin(env: &Env, caller: &Address, role: u32) -> Result<(), Error> {
        if Self::require_admin(env, caller).is_ok() {
            return Ok(());
        }
        let roles: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::Roles(caller.clone()))
            .unwrap_or(0u32);
        if (roles & role) == 0 {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    fn require_image_exists(env: &Env, image_id: u64) -> Result<(), Error> {
        if !env.storage().persistent().has(&DataKey::Image(image_id)) {
            return Err(Error::ImageNotFound);
        }
        Ok(())
    }

    fn next_counter(env: &Env, key: &Symbol) -> u64 {
        let current: u64 = env.storage().instance().get(key).unwrap_or(1u64);
        env.storage()
            .instance()
            .set(key, &current.saturating_add(1));
        current
    }

    fn append_u64(env: &Env, key: DataKey, value: u64) {
        let mut values: Vec<u64> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));
        if !values.iter().any(|item| item == value) {
            values.push_back(value);
            env.storage().persistent().set(&key, &values);
        }
    }

    fn status_to_u32(status: &StudyStatus) -> u32 {
        match status {
            StudyStatus::Pending => 0,
            StudyStatus::Assigned => 1,
            StudyStatus::InReview => 2,
            StudyStatus::PreliminaryReport => 3,
            StudyStatus::DiscrepancyReview => 4,
            StudyStatus::FinalReport => 5,
            StudyStatus::Amended => 6,
        }
    }

    fn update_status_index(
        env: &Env,
        study_id: u64,
        old_status: &StudyStatus,
        new_status: &StudyStatus,
    ) {
        // Remove from old status index
        let old_key = DataKey::StatusStudies(Self::status_to_u32(old_status));
        let old_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&old_key)
            .unwrap_or(Vec::new(env));
        let mut new_vec = Vec::new(env);
        for id in old_ids.iter() {
            if id != study_id {
                new_vec.push_back(id);
            }
        }
        env.storage().persistent().set(&old_key, &new_vec);
        // Add to new status index
        Self::append_u64(env, DataKey::StatusStudies(Self::status_to_u32(new_status)), study_id);
    }
}
