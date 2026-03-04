#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{Address, BytesN, Env, String, Vec};

fn setup(env: &Env) -> (MedicalImagingContractClient<'_>, Address) {
    let contract_id = Address::generate(env);
    env.register_contract(&contract_id, MedicalImagingContract);
    let client = MedicalImagingContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.initialize(&admin, &200);
    (client, admin)
}

fn hash(env: &Env, v: u8) -> BytesN<32> {
    BytesN::from_array(env, &[v; 32])
}

fn dicom(env: &Env, v: u8) -> DicomMetadata {
    DicomMetadata {
        study_uid_hash: hash(env, v),
        series_uid_hash: hash(env, v.saturating_add(1)),
        sop_uid_hash: hash(env, v.saturating_add(2)),
        modality_code_hash: hash(env, v.saturating_add(3)),
        body_part_hash: hash(env, v.saturating_add(4)),
        acquisition_timestamp: 1_700_000_000,
        rows: 2048,
        cols: 2048,
        bits_allocated: 16,
        pixel_spacing_microns: 250,
    }
}

fn upload_three_modalities(
    env: &Env,
    client: &MedicalImagingContractClient<'_>,
    admin: &Address,
    tech: &Address,
    patient: &Address,
) -> (u64, u64, u64) {
    client.assign_role(admin, tech, &1u32);

    let xray_id = client.upload_image(
        tech,
        patient,
        &ImagingModality::XRay,
        &String::from_str(env, "ipfs://xray.enc"),
        &CompressionAlgorithm::Jpeg2000Lossless,
        &12_000,
        &8_000,
        &hash(env, 10),
        &hash(env, 11),
        &dicom(env, 12),
    );

    let mri_id = client.upload_image(
        tech,
        patient,
        &ImagingModality::MRI,
        &String::from_str(env, "ipfs://mri.enc"),
        &CompressionAlgorithm::Deflate,
        &20_000,
        &11_000,
        &hash(env, 20),
        &hash(env, 21),
        &dicom(env, 22),
    );

    let ct_id = client.upload_image(
        tech,
        patient,
        &ImagingModality::CT,
        &String::from_str(env, "ipfs://ct.enc"),
        &CompressionAlgorithm::Rle,
        &18_000,
        &9_000,
        &hash(env, 30),
        &hash(env, 31),
        &dicom(env, 32),
    );

    (xray_id, mri_id, ct_id)
}

#[test]
fn end_to_end_imaging_flow_with_privacy_ai_and_safety() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let patient = Address::generate(&env);
    let tech = Address::generate(&env);
    let radiologist = Address::generate(&env);
    let physician = Address::generate(&env);
    let auditor = Address::generate(&env);
    let specialist = Address::generate(&env);

    client.assign_role(&admin, &radiologist, &2u32);
    client.assign_role(&admin, &physician, &4u32);
    client.assign_role(&admin, &auditor, &16u32);

    let (xray_id, _mri_id, _ct_id) =
        upload_three_modalities(&env, &client, &admin, &tech, &patient);

    let ratio = client.get_compression_ratio_bps(&xray_id);
    assert_eq!(ratio, 6666);

    let mut tokens = Vec::new(&env);
    tokens.push_back(hash(&env, 40));
    tokens.push_back(hash(&env, 41));
    let mut findings = Vec::new(&env);
    findings.push_back(hash(&env, 42));
    assert!(client.extract_and_index_metadata(&radiologist, &xray_id, &tokens, &findings));

    let mut edge_bins = Vec::new(&env);
    edge_bins.push_back(1);
    edge_bins.push_back(12);
    edge_bins.push_back(3);
    edge_bins.push_back(24);
    let edge = client.run_edge_detection(
        &radiologist,
        &xray_id,
        &edge_bins,
        &5,
        &String::from_str(&env, "ipfs://edge-mask"),
        &hash(&env, 50),
        &1,
    );
    assert_eq!(edge.kind, ProcessingKind::EdgeDetection);

    let mut seg_bins = Vec::new(&env);
    seg_bins.push_back(3);
    seg_bins.push_back(8);
    seg_bins.push_back(9);
    seg_bins.push_back(30);
    let seg = client.run_segmentation(
        &radiologist,
        &xray_id,
        &seg_bins,
        &5,
        &10,
        &String::from_str(&env, "ipfs://seg-mask"),
        &hash(&env, 51),
        &1,
    );
    assert_eq!(seg.kind, ProcessingKind::Segmentation);

    client.register_ai_model(
        &physician,
        &hash(&env, 60),
        &hash(&env, 61),
        &1,
        &ImagingModality::XRay,
    );
    let diag_id = client.submit_diagnostic_assistance(
        &physician,
        &xray_id,
        &hash(&env, 60),
        &hash(&env, 62),
        &8800,
        &String::from_str(&env, "ipfs://explainability"),
        &hash(&env, 63),
    );
    let diag = client.get_diagnostic(&diag_id).unwrap();
    assert_eq!(diag.image_id, xray_id);

    env.ledger().set_timestamp(1000);
    client.grant_image_access(
        &patient,
        &xray_id,
        &specialist,
        &ShareScope::Diagnostics,
        &2000,
        &hash(&env, 70),
        &hash(&env, 71),
    );
    assert!(client.verify_share_access(&xray_id, &specialist));

    client.revoke_image_access(&patient, &xray_id, &specialist);
    assert!(!client.verify_share_access(&xray_id, &specialist));

    let integrity_ok = client.verify_image_integrity(&auditor, &xray_id, &hash(&env, 255));
    assert!(!integrity_ok);

    let image = client.get_image(&xray_id).unwrap();
    assert!(image.tamper_detected);

    let mut collaborators = Vec::new(&env);
    collaborators.push_back(specialist.clone());
    let annotation_id = client.add_annotation(
        &physician,
        &xray_id,
        &AnnotationVisibility::CareTeam,
        &String::from_str(&env, "ipfs://annot.enc"),
        &hash(&env, 80),
        &hash(&env, 81),
        &collaborators,
    );
    client.add_annotation_reply(&specialist, &annotation_id, &hash(&env, 82));
    client.resolve_annotation(&physician, &annotation_id);

    let ann = client.get_annotation(&annotation_id).unwrap();
    assert!(ann.resolved);
    assert_eq!(ann.replies.len(), 1);

    let records_contract = Address::generate(&env);
    client.link_image_to_record(&physician, &xray_id, &records_contract, &9001);
    let link = client.get_image_record_link(&xray_id).unwrap();
    assert_eq!(link.medical_record_id, 9001);

    client.record_radiation_dose(&tech, &patient, &xray_id, &ImagingModality::XRay, &90);
    client.record_radiation_dose(&tech, &patient, &xray_id, &ImagingModality::XRay, &130);

    let summary = client.get_dose_summary(&patient).unwrap();
    assert_eq!(summary.total_mgy, 220);
    assert!(summary.safety_alerts >= 1);
}

#[test]
fn supports_dicom_lookup_and_indexes() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let patient = Address::generate(&env);
    let tech = Address::generate(&env);
    client.assign_role(&admin, &tech, &1u32);

    let md = dicom(&env, 100);
    let modality_hash = md.modality_code_hash.clone();
    let body_hash = md.body_part_hash.clone();
    let sop_hash = md.sop_uid_hash.clone();

    let image_id = client.upload_image(
        &tech,
        &patient,
        &ImagingModality::CT,
        &String::from_str(&env, "ipfs://ct2.enc"),
        &CompressionAlgorithm::LosslessJpeg,
        &5_000,
        &2_500,
        &hash(&env, 101),
        &hash(&env, 102),
        &md,
    );

    let by_patient = client.list_images_by_patient(&patient);
    assert!(by_patient.iter().any(|id| id == image_id));

    let by_modality = client.list_images_by_modality_hash(&modality_hash);
    assert!(by_modality.iter().any(|id| id == image_id));

    let by_body = client.list_images_by_body_part_hash(&body_hash);
    assert!(by_body.iter().any(|id| id == image_id));

    let looked_up = client.get_image_by_sop(&sop_hash).unwrap();
    assert_eq!(looked_up, image_id);
}

#[test]
fn duplicate_dicom_sop_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let patient = Address::generate(&env);
    let tech = Address::generate(&env);
    client.assign_role(&admin, &tech, &1u32);

    let md = dicom(&env, 150);

    client.upload_image(
        &tech,
        &patient,
        &ImagingModality::MRI,
        &String::from_str(&env, "ipfs://mri-a.enc"),
        &CompressionAlgorithm::Deflate,
        &8_000,
        &6_000,
        &hash(&env, 151),
        &hash(&env, 152),
        &md.clone(),
    );

    let err = client.try_upload_image(
        &tech,
        &patient,
        &ImagingModality::MRI,
        &String::from_str(&env, "ipfs://mri-b.enc"),
        &CompressionAlgorithm::Deflate,
        &8_100,
        &6_100,
        &hash(&env, 153),
        &hash(&env, 154),
        &md,
    );

    assert_eq!(err, Err(Ok(Error::DuplicateDicomSop)));
}
