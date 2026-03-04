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
}
