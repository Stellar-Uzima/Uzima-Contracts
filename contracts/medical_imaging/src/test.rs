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

// ── Study Workflow Tests ──

#[test]
fn test_create_study() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let patient = Address::generate(&env);
    let tech = Address::generate(&env);
    let physician = Address::generate(&env);
    client.assign_role(&admin, &physician, &4u32);

    let (xray_id, mri_id, _ct_id) = upload_three_modalities(&env, &client, &admin, &tech, &patient);

    let mut image_ids = Vec::new(&env);
    image_ids.push_back(xray_id);
    image_ids.push_back(mri_id);

    let study_id =
        client.create_study(&physician, &patient, &ImagingModality::XRay, &image_ids, &2);
    assert_eq!(study_id, 1);

    let study = client.get_study(&study_id).unwrap();
    assert_eq!(study.status, StudyStatus::Pending);
    assert_eq!(study.required_readers, 2);
    assert_eq!(study.image_ids.len(), 2);

    let by_patient = client.get_studies_by_patient(&patient);
    assert!(by_patient.iter().any(|id| id == study_id));

    let by_status = client.get_studies_by_status(&StudyStatus::Pending);
    assert!(by_status.iter().any(|id| id == study_id));
}

#[test]
#[should_panic(expected = "Error(Contract, #18)")]
fn test_create_study_too_many_readers() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let patient = Address::generate(&env);
    let tech = Address::generate(&env);
    let physician = Address::generate(&env);
    client.assign_role(&admin, &physician, &4u32);

    let (xray_id, _mri_id, _ct_id) =
        upload_three_modalities(&env, &client, &admin, &tech, &patient);

    let mut image_ids = Vec::new(&env);
    image_ids.push_back(xray_id);

    client.create_study(&physician, &patient, &ImagingModality::XRay, &image_ids, &6);
}

#[test]
fn test_assign_reader() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let patient = Address::generate(&env);
    let tech = Address::generate(&env);
    let physician = Address::generate(&env);
    let radiologist = Address::generate(&env);
    client.assign_role(&admin, &physician, &4u32);
    client.assign_role(&admin, &radiologist, &2u32);

    let (xray_id, _mri_id, _ct_id) =
        upload_three_modalities(&env, &client, &admin, &tech, &patient);

    let mut image_ids = Vec::new(&env);
    image_ids.push_back(xray_id);

    let study_id =
        client.create_study(&physician, &patient, &ImagingModality::XRay, &image_ids, &2);

    client.assign_reader(&physician, &study_id, &radiologist);

    let study = client.get_study(&study_id).unwrap();
    assert_eq!(study.status, StudyStatus::Assigned);

    let by_reader = client.get_studies_by_reader(&radiologist);
    assert!(by_reader.iter().any(|id| id == study_id));
}

// ── Reader Report Tests ──

fn setup_study_with_readers(
    env: &Env,
    client: &MedicalImagingContractClient<'_>,
    admin: &Address,
) -> (u64, Address, Address) {
    let patient = Address::generate(env);
    let tech = Address::generate(env);
    let physician = Address::generate(env);
    let reader1 = Address::generate(env);
    let reader2 = Address::generate(env);
    client.assign_role(admin, &physician, &4u32);
    client.assign_role(admin, &reader1, &2u32);
    client.assign_role(admin, &reader2, &2u32);

    let (xray_id, _mri_id, _ct_id) = upload_three_modalities(env, client, admin, &tech, &patient);

    let mut image_ids = Vec::new(env);
    image_ids.push_back(xray_id);

    let study_id =
        client.create_study(&physician, &patient, &ImagingModality::XRay, &image_ids, &2);

    client.assign_reader(&physician, &study_id, &reader1);
    client.assign_reader(&physician, &study_id, &reader2);

    (study_id, reader1, reader2)
}

#[test]
fn test_submit_reader_report_agreement() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let (study_id, reader1, reader2) = setup_study_with_readers(&env, &client, &admin);

    let same_diag = hash(&env, 200);

    client.submit_reader_report(
        &reader1,
        &study_id,
        &same_diag,
        &hash(&env, 201),
        &String::from_str(&env, "ipfs://findings1"),
        &true,
        &9000,
    );

    let study = client.get_study(&study_id).unwrap();
    assert_eq!(study.status, StudyStatus::InReview);

    client.submit_reader_report(
        &reader2,
        &study_id,
        &same_diag,
        &hash(&env, 202),
        &String::from_str(&env, "ipfs://findings2"),
        &true,
        &8500,
    );

    let study = client.get_study(&study_id).unwrap();
    assert_eq!(study.status, StudyStatus::PreliminaryReport);
}

#[test]
fn test_submit_reader_report_discrepancy() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let (study_id, reader1, reader2) = setup_study_with_readers(&env, &client, &admin);

    client.submit_reader_report(
        &reader1,
        &study_id,
        &hash(&env, 200),
        &hash(&env, 201),
        &String::from_str(&env, "ipfs://findings1"),
        &true,
        &9000,
    );

    client.submit_reader_report(
        &reader2,
        &study_id,
        &hash(&env, 210),
        &hash(&env, 211),
        &String::from_str(&env, "ipfs://findings2"),
        &false,
        &5000,
    );

    let study = client.get_study(&study_id).unwrap();
    assert_eq!(study.status, StudyStatus::DiscrepancyReview);
}

#[test]
#[should_panic(expected = "Error(Contract, #16)")]
fn test_submit_report_not_assigned() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let (study_id, _reader1, _reader2) = setup_study_with_readers(&env, &client, &admin);

    let stranger = Address::generate(&env);

    client.submit_reader_report(
        &stranger,
        &study_id,
        &hash(&env, 200),
        &hash(&env, 201),
        &String::from_str(&env, "ipfs://findings"),
        &true,
        &9000,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #17)")]
fn test_submit_report_duplicate() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let (study_id, reader1, _reader2) = setup_study_with_readers(&env, &client, &admin);

    client.submit_reader_report(
        &reader1,
        &study_id,
        &hash(&env, 200),
        &hash(&env, 201),
        &String::from_str(&env, "ipfs://findings1"),
        &true,
        &9000,
    );

    client.submit_reader_report(
        &reader1,
        &study_id,
        &hash(&env, 200),
        &hash(&env, 201),
        &String::from_str(&env, "ipfs://findings1"),
        &true,
        &9000,
    );
}

#[test]
fn test_single_reader_study() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let patient = Address::generate(&env);
    let tech = Address::generate(&env);
    let physician = Address::generate(&env);
    let reader1 = Address::generate(&env);
    client.assign_role(&admin, &physician, &4u32);
    client.assign_role(&admin, &reader1, &2u32);

    let (xray_id, _mri_id, _ct_id) =
        upload_three_modalities(&env, &client, &admin, &tech, &patient);

    let mut image_ids = Vec::new(&env);
    image_ids.push_back(xray_id);

    let study_id =
        client.create_study(&physician, &patient, &ImagingModality::XRay, &image_ids, &1);

    client.assign_reader(&physician, &study_id, &reader1);

    client.submit_reader_report(
        &reader1,
        &study_id,
        &hash(&env, 200),
        &hash(&env, 201),
        &String::from_str(&env, "ipfs://findings1"),
        &true,
        &9000,
    );

    let study = client.get_study(&study_id).unwrap();
    assert_eq!(study.status, StudyStatus::PreliminaryReport);
}

#[test]
fn test_blind_review_enforcement() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let (study_id, reader1, reader2) = setup_study_with_readers(&env, &client, &admin);

    client.submit_reader_report(
        &reader1,
        &study_id,
        &hash(&env, 200),
        &hash(&env, 201),
        &String::from_str(&env, "ipfs://findings1"),
        &true,
        &9000,
    );

    // Reader2 tries to see reports while still InReview → empty
    let reports = client.get_reader_reports(&reader2, &study_id);
    assert_eq!(reports.len(), 0);

    // Reader1 can still see own report
    let my_report = client.get_my_report(&reader1, &study_id);
    assert_eq!(my_report.study_id, study_id);
}

// ── Finalization & Amendment Tests ──

#[test]
fn test_finalize_study() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let (study_id, reader1, reader2) = setup_study_with_readers(&env, &client, &admin);

    let same_diag = hash(&env, 200);

    client.submit_reader_report(
        &reader1,
        &study_id,
        &same_diag,
        &hash(&env, 201),
        &String::from_str(&env, "ipfs://findings1"),
        &true,
        &9000,
    );

    client.submit_reader_report(
        &reader2,
        &study_id,
        &same_diag,
        &hash(&env, 202),
        &String::from_str(&env, "ipfs://findings2"),
        &true,
        &8500,
    );

    assert_eq!(
        client.get_study(&study_id).unwrap().status,
        StudyStatus::PreliminaryReport
    );

    env.ledger().set_timestamp(1000);

    client.finalize_study(
        &reader1,
        &study_id,
        &String::from_str(&env, "ipfs://final-report"),
    );

    let study = client.get_study(&study_id).unwrap();
    assert_eq!(study.status, StudyStatus::FinalReport);
    assert!(study.finalized_at > 0);
}

#[test]
fn test_amend_study() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let (study_id, reader1, reader2) = setup_study_with_readers(&env, &client, &admin);

    let same_diag = hash(&env, 200);

    client.submit_reader_report(
        &reader1,
        &study_id,
        &same_diag,
        &hash(&env, 201),
        &String::from_str(&env, "ipfs://findings1"),
        &true,
        &9000,
    );

    client.submit_reader_report(
        &reader2,
        &study_id,
        &same_diag,
        &hash(&env, 202),
        &String::from_str(&env, "ipfs://findings2"),
        &true,
        &8500,
    );

    client.finalize_study(
        &reader1,
        &study_id,
        &String::from_str(&env, "ipfs://final-report"),
    );

    client.amend_study(
        &reader1,
        &study_id,
        &String::from_str(&env, "ipfs://amendment"),
        &hash(&env, 250),
    );

    let study = client.get_study(&study_id).unwrap();
    assert_eq!(study.status, StudyStatus::Amended);
}

#[test]
fn test_amend_study_multiple() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let (study_id, reader1, reader2) = setup_study_with_readers(&env, &client, &admin);

    let same_diag = hash(&env, 200);

    client.submit_reader_report(
        &reader1,
        &study_id,
        &same_diag,
        &hash(&env, 201),
        &String::from_str(&env, "ipfs://findings1"),
        &true,
        &9000,
    );

    client.submit_reader_report(
        &reader2,
        &study_id,
        &same_diag,
        &hash(&env, 202),
        &String::from_str(&env, "ipfs://findings2"),
        &true,
        &8500,
    );

    client.finalize_study(
        &reader1,
        &study_id,
        &String::from_str(&env, "ipfs://final-report"),
    );

    client.amend_study(
        &reader1,
        &study_id,
        &String::from_str(&env, "ipfs://amendment1"),
        &hash(&env, 250),
    );

    client.amend_study(
        &reader1,
        &study_id,
        &String::from_str(&env, "ipfs://amendment2"),
        &hash(&env, 251),
    );

    let study = client.get_study(&study_id).unwrap();
    assert_eq!(study.status, StudyStatus::Amended);
}

#[test]
#[should_panic(expected = "Error(Contract, #15)")]
fn test_finalize_wrong_status() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let (study_id, reader1, _reader2) = setup_study_with_readers(&env, &client, &admin);

    // Only 1 of 2 readers submitted → InReview
    client.submit_reader_report(
        &reader1,
        &study_id,
        &hash(&env, 200),
        &hash(&env, 201),
        &String::from_str(&env, "ipfs://findings1"),
        &true,
        &9000,
    );

    // Study is in InReview, not PreliminaryReport → should fail
    client.finalize_study(
        &reader1,
        &study_id,
        &String::from_str(&env, "ipfs://final-report"),
    );
}
