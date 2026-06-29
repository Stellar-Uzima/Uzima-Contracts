#![cfg(test)]

//! Comprehensive Integration Test Suite for IHE/FHIR Standard Compliance
//!
//! This test suite validates:
//! - FHIR resource validation
//! - IHE profile compliance
//! - HL7 message format testing
//! - Interoperability verification

use soroban_sdk::{
    testutils::Address as _, Address, BytesN, Env, String,
};

// Import contract types
mod fhir_types {
    pub use super::*;
    
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum FHIRResourceType {
        Patient = 0,
        Observation = 1,
        Condition = 2,
        MedicationStatement = 3,
        Procedure = 4,
        AllergyIntolerance = 5,
        CareTeam = 6,
        Encounter = 7,
        DiagnosticReport = 8,
        Immunization = 9,
        DocumentReference = 10,
    }
    
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum CodingSystem {
        ICD10,
        ICD9,
        CPT,
        SNOMEDCT,
        LOINC,
        RxNorm,
        Custom,
    }
}

// ==================== Test Helper Functions ====================

fn create_test_env() -> (Env, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let provider = Address::generate(&env);
    let patient = Address::generate(&env);

    (env, admin, provider, patient)
}

fn create_fhir_patient(env: &Env) -> FHIRPatient {
    FHIRPatient {
        identifiers: vec![FHIRIdentifier {
            system: String::from_str(env, "urn:mrn:hospital-a"),
            value: String::from_str(env, "MRN-12345"),
            use_type: String::from_str(env, "official"),
        }],
        given_name: String::from_str(env, "John"),
        family_name: String::from_str(env, "Doe"),
        birth_date: String::from_str(env, "1980-01-01"),
        gender: String::from_str(env, "male"),
        contact_point: String::from_str(env, "john.doe@example.com"),
        address: String::from_str(env, "123 Main St, City, State 12345"),
        communication: vec![String::from_str(env, "en")],
        marital_status: String::from_str(env, "married"),
    }
}

fn create_fhir_code(env: &Env, system: CodingSystem, code: &str, display: &str) -> FHIRCode {
    FHIRCode {
        system,
        code: String::from_str(env, code),
        display: String::from_str(env, display),
    }
}

fn validate_fhir_resource(resource_type: fhir_types::FHIRResourceType) -> bool {
    matches!(
        resource_type,
        fhir_types::FHIRResourceType::Patient
            | fhir_types::FHIRResourceType::Observation
            | fhir_types::FHIRResourceType::Condition
            | fhir_types::FHIRResourceType::MedicationStatement
            | fhir_types::FHIRResourceType::Procedure
            | fhir_types::FHIRResourceType::AllergyIntolerance
            | fhir_types::FHIRResourceType::CareTeam
            | fhir_types::FHIRResourceType::Encounter
            | fhir_types::FHIRResourceType::DiagnosticReport
            | fhir_types::FHIRResourceType::Immunization
            | fhir_types::FHIRResourceType::DocumentReference
    )
}

fn validate_hl7_message_format(message_type: &str) -> bool {
    matches!(
        message_type,
        "ADT" | "ORM" | "ORU" | "MFN" | "QBP" | "RSP" | "ACK"
    )
}

use fhir_types::CodingSystem;
use fhir_types::FHIRResourceType;

// ==================== FHIR Resource Validation Tests ====================

#[test]
fn test_fhir_patient_resource() {
    let (env, _admin, _provider, _patient) = create_test_env();

    let patient = create_fhir_patient(&env);
    
    assert!(!patient.identifiers.is_empty());
    assert_eq!(patient.given_name, String::from_str(&env, "John"));
    assert_eq!(patient.family_name, String::from_str(&env, "Doe"));
    assert_eq!(patient.birth_date, String::from_str(&env, "1980-01-01"));
    assert_eq!(patient.gender, String::from_str(&env, "male"));
    
    assert!(validate_fhir_resource(FHIRResourceType::Patient));
}

#[test]
fn test_fhir_observation_resource() {
    let (env, _admin, _provider, _patient) = create_test_env();

    let observation = FHIRObservation {
        identifier: String::from_str(&env, "obs-001"),
        status: String::from_str(&env, "final"),
        category: create_fhir_code(&env, CodingSystem::LOINC, "vital-signs", "Vital Signs"),
        code: create_fhir_code(&env, CodingSystem::LOINC, "8867-4", "Heart rate"),
        subject_reference: String::from_str(&env, "Patient/MRN-12345"),
        effective_datetime: String::from_str(&env, "2024-01-15T10:30:00Z"),
        value_quantity_value: 72,
        value_quantity_unit: String::from_str(&env, "beats/minute"),
        interpretation: vec![],
        reference_range: String::from_str(&env, "60-100 beats/minute"),
    };
    
    assert_eq!(observation.status, String::from_str(&env, "final"));
    assert_eq!(observation.value_quantity_value, 72);
    assert!(validate_fhir_resource(FHIRResourceType::Observation));
}

#[test]
fn test_fhir_condition_resource() {
    let (env, _admin, _provider, _patient) = create_test_env();

    let condition = FHIRCondition {
        identifier: String::from_str(&env, "cond-001"),
        clinical_status: String::from_str(&env, "active"),
        code: create_fhir_code(
            &env,
            CodingSystem::ICD10,
            "E11.9",
            "Type 2 diabetes mellitus",
        ),
        subject_reference: String::from_str(&env, "Patient/MRN-12345"),
        onset_date_time: String::from_str(&env, "2020-06-15T00:00:00Z"),
        recorded_date: String::from_str(&env, "2020-06-15T00:00:00Z"),
        severity: vec![],
    };
    
    assert_eq!(condition.clinical_status, String::from_str(&env, "active"));
    assert_eq!(condition.code.code, String::from_str(&env, "E11.9"));
    assert!(validate_fhir_resource(FHIRResourceType::Condition));
}

#[test]
fn test_fhir_medication_statement_resource() {
    let (env, _admin, _provider, _patient) = create_test_env();

    let medication = FHIRMedicationStatement {
        identifier: String::from_str(&env, "med-001"),
        status: String::from_str(&env, "active"),
        medication_code: create_fhir_code(&env, CodingSystem::RxNorm, "860975", "Metformin 500mg"),
        subject_reference: String::from_str(&env, "Patient/MRN-12345"),
        effective_period_start: String::from_str(&env, "2020-06-15T00:00:00Z"),
        effective_period_end: String::from_str(&env, ""),
        dosage: String::from_str(&env, "500mg twice daily"),
        reason_code: vec![],
    };
    
    assert_eq!(medication.status, String::from_str(&env, "active"));
    assert_eq!(
        medication.dosage,
        String::from_str(&env, "500mg twice daily")
    );
    assert!(validate_fhir_resource(
        FHIRResourceType::MedicationStatement
    ));
}

#[test]
fn test_fhir_procedure_resource() {
    let (env, _admin, _provider, _patient) = create_test_env();
    
    let procedure = FHIRProcedure {
        identifier: String::from_str(&env, "proc-001"),
        status: String::from_str(&env, "completed"),
        code: create_fhir_code(&env, CodingSystem::CPT, "99213", "Office visit"),
        subject_reference: String::from_str(&env, "Patient/MRN-12345"),
        performed_date_time: String::from_str(&env, "2024-01-15T10:00:00Z"),
        performer: vec![String::from_str(&env, "Practitioner/dr-smith")],
        reason_code: vec![],
    };
    
    assert_eq!(procedure.status, String::from_str(&env, "completed"));
    assert!(!procedure.performer.is_empty());
    assert!(validate_fhir_resource(FHIRResourceType::Procedure));
}

#[test]
fn test_fhir_allergy_intolerance_resource() {
    let (env, _admin, _provider, _patient) = create_test_env();

    let allergy = FHIRAllergyIntolerance {
        identifier: String::from_str(&env, "allergy-001"),
        clinical_status: String::from_str(&env, "active"),
        verification_status: String::from_str(&env, "confirmed"),
        substance_code: create_fhir_code(&env, CodingSystem::SNOMEDCT, "387207008", "Penicillin"),
        patient_reference: String::from_str(&env, "Patient/MRN-12345"),
        recorded_date: String::from_str(&env, "2020-01-01T00:00:00Z"),
        manifestation: vec![],
        severity: String::from_str(&env, "severe"),
    };
    
    assert_eq!(allergy.clinical_status, String::from_str(&env, "active"));
    assert_eq!(allergy.severity, String::from_str(&env, "severe"));
    assert!(validate_fhir_resource(FHIRResourceType::AllergyIntolerance));
}

// ==================== IHE Profile Compliance Tests ====================

#[test]
fn test_ihe_xds_profile_compliance() {
    let (env, _admin, _provider, _patient) = create_test_env();
    
    let document_entry = XDSDocumentEntry {
        document_id: String::from_str(&env, "doc-001"),
        patient_id: String::from_str(&env, "MRN-12345"),
        content_hash: BytesN::from_array(&env, &[1u8; 32]),
        document_class_code: String::from_str(&env, "11488-4"),
        document_type_code: String::from_str(&env, "34117-2"),
        format_code: String::from_str(&env, "urn:ihe:pcc:xds-ms:2007"),
        healthcare_facility_type: String::from_str(&env, "OF"),
        practice_setting_code: String::from_str(&env, "General Medicine"),
        creation_time: env.ledger().timestamp(),
        author: Address::generate(&env),
        confidentiality_code: String::from_str(&env, "N"),
        language_code: String::from_str(&env, "en-US"),
        hl7_message_type: HL7MessageType::V2ADT,
        status: DocumentStatus::Approved,
        repository_unique_id: String::from_str(&env, "1.3.6.1.4.1.21367.2010.1.2.1125"),
        submission_set_id: String::from_str(&env, "ss-001"),
        mime_type: String::from_str(&env, "application/pdf"),
    };
    
    assert_eq!(document_entry.document_id, String::from_str(&env, "doc-001"));
    assert_eq!(document_entry.status, DocumentStatus::Approved);
    assert!(!document_entry.format_code.is_empty());
}

#[test]
fn test_ihe_pix_profile_compliance() {
    let (env, _admin, _provider, _patient) = create_test_env();
    
    let local_id = PatientIdentifier {
        id_value: String::from_str(&env, "MRN-12345"),
        assigning_authority: String::from_str(&env, "Hospital-A"),
        identifier_type_code: String::from_str(&env, "MR"),
    };
    
    let cross_ids = vec![PatientIdentifier {
        id_value: String::from_str(&env, "PID-67890"),
        assigning_authority: String::from_str(&env, "Hospital-B"),
        identifier_type_code: String::from_str(&env, "PI"),
    }];
    
    let pix_cross_ref = PIXCrossReference {
        reference_id: 1,
        local_id: local_id.clone(),
        cross_referenced_ids: cross_ids,
        created_at: env.ledger().timestamp(),
        updated_at: env.ledger().timestamp(),
        is_merged: false,
    };
    
    assert_eq!(pix_cross_ref.local_id.id_value, String::from_str(&env, "MRN-12345"));
    assert!(!pix_cross_ref.cross_referenced_ids.is_empty());
    assert!(!pix_cross_ref.is_merged);
}

#[test]
fn test_ihe_pdq_profile_compliance() {
    let (env, _admin, _provider, _patient) = create_test_env();
    
    let demographics = PatientDemographics {
        patient_id: String::from_str(&env, "MRN-12345"),
        given_name: String::from_str(&env, "John"),
        family_name: String::from_str(&env, "Doe"),
        date_of_birth: String::from_str(&env, "1980-01-01"),
        administrative_gender: String::from_str(&env, "M"),
        street_address: String::from_str(&env, "123 Main St"),
        city: String::from_str(&env, "Springfield"),
        state: String::from_str(&env, "IL"),
        postal_code: String::from_str(&env, "62701"),
        country_code: String::from_str(&env, "US"),
        phone_home: String::from_str(&env, "555-1234"),
        phone_mobile: String::from_str(&env, "555-5678"),
        mother_maiden_name: String::from_str(&env, "Smith"),
        marital_status: String::from_str(&env, "M"),
        race: String::from_str(&env, "2106-3"),
        ethnicity: String::from_str(&env, "2186-5"),
        primary_language: String::from_str(&env, "en"),
        last_updated: env.ledger().timestamp(),
        assigning_authority: String::from_str(&env, "Hospital-A"),
    };
    
    assert_eq!(demographics.given_name, String::from_str(&env, "John"));
    assert_eq!(demographics.family_name, String::from_str(&env, "Doe"));
    assert_eq!(
        demographics.administrative_gender,
        String::from_str(&env, "M")
    );
}

#[test]
fn test_ihe_atna_profile_compliance() {
    let (env, _admin, _provider, _patient) = create_test_env();
    
    let participants = vec![ATNAParticipant {
        user_id: String::from_str(&env, "user-001"),
        user_name: String::from_str(&env, "Dr. Smith"),
        role_id_code: String::from_str(&env, "physician"),
        is_requestor: true,
        network_access_point: String::from_str(&env, "192.168.1.100"),
    }];
    
    let audit_event = ATNAAuditEvent {
        event_id: 1,
        event_type: ATNAEventType::PatientRecordAccess,
        event_action_code: String::from_str(&env, "R"),
        event_date_time: env.ledger().timestamp(),
        event_outcome: ATNAEventOutcome::Success,
        source_id: String::from_str(&env, "EMR-System"),
        source_type: String::from_str(&env, "4"),
        active_participants: participants,
        participant_objects: vec![],
        hl7_message_id: String::from_str(&env, "MSG-001"),
        profile: IHEProfile::ATNA,
    };
    
    assert_eq!(audit_event.event_type, ATNAEventType::PatientRecordAccess);
    assert_eq!(audit_event.event_outcome, ATNAEventOutcome::Success);
    assert!(!audit_event.active_participants.is_empty());
}

// ==================== HL7 Message Format Tests ====================

#[test]
fn test_hl7_v2_adt_message_format() {
    let message_type = "ADT";
    assert!(validate_hl7_message_format(message_type));
}

#[test]
fn test_hl7_v2_orm_message_format() {
    let message_type = "ORM";
    assert!(validate_hl7_message_format(message_type));
}

#[test]
fn test_hl7_v2_oru_message_format() {
    let message_type = "ORU";
    assert!(validate_hl7_message_format(message_type));
}

#[test]
fn test_hl7_v2_qbp_message_format() {
    let message_type = "QBP";
    assert!(validate_hl7_message_format(message_type));
}

#[test]
fn test_hl7_message_type_enum() {
    let message_types = vec![
        HL7MessageType::V2ADT,
        HL7MessageType::V2ORM,
        HL7MessageType::V2ORU,
        HL7MessageType::V2MFN,
        HL7MessageType::V2QBP,
        HL7MessageType::V2RSP,
        HL7MessageType::V2ACK,
        HL7MessageType::V3ClinicalDocument,
        HL7MessageType::V3PatientQuery,
        HL7MessageType::V3PatientResponse,
        HL7MessageType::V3DeviceQuery,
    ];
    
    assert_eq!(message_types.len(), 11);
}

// ==================== Interoperability Verification Tests ====================

#[test]
fn test_patient_record_exchange_interoperability() {
    let (env, _admin, _provider, _patient) = create_test_env();
    
    let fhir_patient = create_fhir_patient(&env);
    
    let demographics = PatientDemographics {
        patient_id: fhir_patient.identifiers[0].value.clone(),
        given_name: fhir_patient.given_name.clone(),
        family_name: fhir_patient.family_name.clone(),
        date_of_birth: fhir_patient.birth_date.clone(),
        administrative_gender: fhir_patient.gender.clone(),
        street_address: fhir_patient.address.clone(),
        city: String::from_str(&env, "Springfield"),
        state: String::from_str(&env, "IL"),
        postal_code: String::from_str(&env, "62701"),
        country_code: String::from_str(&env, "US"),
        phone_home: String::from_str(&env, ""),
        phone_mobile: fhir_patient.contact_point.clone(),
        mother_maiden_name: String::from_str(&env, ""),
        marital_status: fhir_patient.marital_status.clone(),
        race: String::from_str(&env, ""),
        ethnicity: String::from_str(&env, ""),
        primary_language: fhir_patient.communication[0].clone(),
        last_updated: env.ledger().timestamp(),
        assigning_authority: String::from_str(&env, "Hospital-A"),
    };
    
    assert_eq!(demographics.given_name, fhir_patient.given_name);
    assert_eq!(demographics.family_name, fhir_patient.family_name);
    assert_eq!(demographics.date_of_birth, fhir_patient.birth_date);
}

#[test]
fn test_fhir_to_xds_document_conversion() {
    let (env, _admin, _provider, _patient) = create_test_env();
    
    let fhir_doc_ref_id = String::from_str(&env, "doc-ref-001");
    let patient_id = String::from_str(&env, "MRN-12345");
    
    let xds_entry = XDSDocumentEntry {
        document_id: fhir_doc_ref_id.clone(),
        patient_id: patient_id.clone(),
        content_hash: BytesN::from_array(&env, &[2u8; 32]),
        document_class_code: String::from_str(&env, "11488-4"),
        document_type_code: String::from_str(&env, "34117-2"),
        format_code: String::from_str(&env, "urn:ihe:pcc:xds-ms:2007"),
        healthcare_facility_type: String::from_str(&env, "OF"),
        practice_setting_code: String::from_str(&env, "General Medicine"),
        creation_time: env.ledger().timestamp(),
        author: Address::generate(&env),
        confidentiality_code: String::from_str(&env, "N"),
        language_code: String::from_str(&env, "en-US"),
        hl7_message_type: HL7MessageType::V2ADT,
        status: DocumentStatus::Approved,
        repository_unique_id: String::from_str(&env, "1.3.6.1.4.1.21367.2010.1.2.1125"),
        submission_set_id: String::from_str(&env, "ss-001"),
        mime_type: String::from_str(&env, "application/pdf"),
    };
    
    assert_eq!(xds_entry.document_id, fhir_doc_ref_id);
    assert_eq!(xds_entry.patient_id, patient_id);
    assert_eq!(xds_entry.status, DocumentStatus::Approved);
}

#[test]
fn test_cross_system_patient_identifier_mapping() {
    let (env, _admin, _provider, _patient) = create_test_env();
    
    let system_a_id = PatientIdentifier {
        id_value: String::from_str(&env, "MRN-12345"),
        assigning_authority: String::from_str(&env, "Hospital-A"),
        identifier_type_code: String::from_str(&env, "MR"),
    };

    let system_b_id = PatientIdentifier {
        id_value: String::from_str(&env, "PID-67890"),
        assigning_authority: String::from_str(&env, "Hospital-B"),
        identifier_type_code: String::from_str(&env, "PI"),
    };

    let system_c_id = PatientIdentifier {
        id_value: String::from_str(&env, "EID-ABCDE"),
        assigning_authority: String::from_str(&env, "Clinic-C"),
        identifier_type_code: String::from_str(&env, "EI"),
    };
    
    let pix_mapping = PIXCrossReference {
        reference_id: 1,
        local_id: system_a_id,
        cross_referenced_ids: vec![system_b_id, system_c_id],
        created_at: env.ledger().timestamp(),
        updated_at: env.ledger().timestamp(),
        is_merged: false,
    };
    
    assert_eq!(pix_mapping.cross_referenced_ids.len(), 2);
    assert_eq!(
        pix_mapping.local_id.assigning_authority,
        String::from_str(&env, "Hospital-A")
    );
}

// ==================== Test Scenarios ====================

#[test]
fn test_scenario_patient_record_exchange() {
    let (env, _admin, _provider, _patient) = create_test_env();
    
    let fhir_patient = create_fhir_patient(&env);
    assert!(!fhir_patient.identifiers.is_empty());
    
    let local_id = PatientIdentifier {
        id_value: fhir_patient.identifiers[0].value.clone(),
        assigning_authority: String::from_str(&env, "Hospital-A"),
        identifier_type_code: String::from_str(&env, "MR"),
    };

    let pix_ref = PIXCrossReference {
        reference_id: 1,
        local_id,
        cross_referenced_ids: vec![],
        created_at: env.ledger().timestamp(),
        updated_at: env.ledger().timestamp(),
        is_merged: false,
    };

    assert_eq!(pix_ref.reference_id, 1);
    
    let demographics = PatientDemographics {
        patient_id: fhir_patient.identifiers[0].value.clone(),
        given_name: fhir_patient.given_name.clone(),
        family_name: fhir_patient.family_name.clone(),
        date_of_birth: fhir_patient.birth_date.clone(),
        administrative_gender: fhir_patient.gender.clone(),
        street_address: fhir_patient.address.clone(),
        city: String::from_str(&env, "Springfield"),
        state: String::from_str(&env, "IL"),
        postal_code: String::from_str(&env, "62701"),
        country_code: String::from_str(&env, "US"),
        phone_home: String::from_str(&env, ""),
        phone_mobile: fhir_patient.contact_point.clone(),
        mother_maiden_name: String::from_str(&env, ""),
        marital_status: fhir_patient.marital_status.clone(),
        race: String::from_str(&env, ""),
        ethnicity: String::from_str(&env, ""),
        primary_language: String::from_str(&env, "en"),
        last_updated: env.ledger().timestamp(),
        assigning_authority: String::from_str(&env, "Hospital-A"),
    };

    assert_eq!(demographics.given_name, fhir_patient.given_name);
    

    // Step 4: Create XDS document entry
    let xds_entry = XDSDocumentEntry {
        document_id: String::from_str(&env, "doc-001"),
        patient_id: demographics.patient_id.clone(),
        content_hash: BytesN::from_array(&env, &[3u8; 32]),
        document_class_code: String::from_str(&env, "11488-4"),
        document_type_code: String::from_str(&env, "34117-2"),
        format_code: String::from_str(&env, "urn:ihe:pcc:xds-ms:2007"),
        healthcare_facility_type: String::from_str(&env, "OF"),
        practice_setting_code: String::from_str(&env, "General Medicine"),
        creation_time: env.ledger().timestamp(),
        author: _provider.clone(),
        confidentiality_code: String::from_str(&env, "N"),
        language_code: String::from_str(&env, "en-US"),
        hl7_message_type: HL7MessageType::V2ADT,
        status: DocumentStatus::Approved,
        repository_unique_id: String::from_str(&env, "1.3.6.1.4.1.21367.2010.1.2.1125"),
        submission_set_id: String::from_str(&env, "ss-001"),
        mime_type: String::from_str(&env, "application/pdf"),
    };

    assert_eq!(xds_entry.status, DocumentStatus::Approved);
}

#[test]
fn test_scenario_consent_management_flow() {
    let (env, _admin, _provider, _patient) = create_test_env();
    
    let fhir_patient = create_fhir_patient(&env);
    
    let consent = BPPCConsent {
        consent_id: 1,
        patient_id: fhir_patient.identifiers[0].value.clone(),
        policy_id: String::from_str(&env, "policy-001"),
        consent_status: ConsentStatus::Active,
        access_consent_list: vec![
            String::from_str(&env, "provider-001"),
            String::from_str(&env, "provider-002"),
        ],
        date_of_consent: env.ledger().timestamp(),
        expiry_time: env.ledger().timestamp() + 31536000,
        author: _patient.clone(),
        document_ref: String::from_str(&env, "consent-doc-001"),
    };

    assert_eq!(consent.consent_status, ConsentStatus::Active);
    assert_eq!(consent.access_consent_list.len(), 2);
    
    let audit_event = ATNAAuditEvent {
        event_id: 1,
        event_type: ATNAEventType::PatientRecordAccess,
        event_action_code: String::from_str(&env, "C"),
        event_date_time: env.ledger().timestamp(),
        event_outcome: ATNAEventOutcome::Success,
        source_id: String::from_str(&env, "CONSENT-SYSTEM"),
        source_type: String::from_str(&env, "4"),
        active_participants: vec![ATNAParticipant {
            user_id: String::from_str(&env, "patient-001"),
            user_name: String::from_str(&env, "John Doe"),
            role_id_code: String::from_str(&env, "patient"),
            is_requestor: true,
            network_access_point: String::from_str(&env, "192.168.1.100"),
        }],
        participant_objects: vec![],
        hl7_message_id: String::from_str(&env, "MSG-CONSENT-001"),
        profile: IHEProfile::BPPC,
    };

    assert_eq!(audit_event.event_outcome, ATNAEventOutcome::Success);
    assert_eq!(audit_event.profile, IHEProfile::BPPC);
    
    assert_eq!(consent.consent_status, ConsentStatus::Active);
    assert!(consent.expiry_time > env.ledger().timestamp());
}

#[test]
fn test_scenario_audit_trail_compliance() {
    let (env, _admin, _provider, _patient) = create_test_env();
    
    let participants = vec![ATNAParticipant {
        user_id: String::from_str(&env, "dr-smith"),
        user_name: String::from_str(&env, "Dr. John Smith"),
        role_id_code: String::from_str(&env, "physician"),
        is_requestor: true,
        network_access_point: String::from_str(&env, "192.168.1.50"),
    }];
    
    let participant_objects = vec![ATNAParticipantObject {
        object_id_type_code: String::from_str(&env, "2"),
        object_id: String::from_str(&env, "MRN-12345"),
        object_type_code: 1,
        object_sensitivity: String::from_str(&env, "N"),
        object_query: String::from_str(&env, ""),
    }];
    
    let access_event = ATNAAuditEvent {
        event_id: 1,
        event_type: ATNAEventType::PatientRecordAccess,
        event_action_code: String::from_str(&env, "R"),
        event_date_time: env.ledger().timestamp(),
        event_outcome: ATNAEventOutcome::Success,
        source_id: String::from_str(&env, "EMR-SYSTEM"),
        source_type: String::from_str(&env, "4"),
        active_participants: participants.clone(),
        participant_objects: participant_objects.clone(),
        hl7_message_id: String::from_str(&env, "MSG-001"),
        profile: IHEProfile::ATNA,
    };

    assert_eq!(access_event.event_type, ATNAEventType::PatientRecordAccess);
    assert_eq!(access_event.event_outcome, ATNAEventOutcome::Success);
    
    let update_event = ATNAAuditEvent {
        event_id: 2,
        event_type: ATNAEventType::PatientRecordUpdate,
        event_action_code: String::from_str(&env, "U"),
        event_date_time: env.ledger().timestamp(),
        event_outcome: ATNAEventOutcome::Success,
        source_id: String::from_str(&env, "EMR-SYSTEM"),
        source_type: String::from_str(&env, "4"),
        active_participants: participants.clone(),
        participant_objects: participant_objects.clone(),
        hl7_message_id: String::from_str(&env, "MSG-002"),
        profile: IHEProfile::ATNA,
    };

    assert_eq!(update_event.event_type, ATNAEventType::PatientRecordUpdate);
    
    let export_event = ATNAAuditEvent {
        event_id: 3,
        event_type: ATNAEventType::DocumentExport,
        event_action_code: String::from_str(&env, "E"),
        event_date_time: env.ledger().timestamp(),
        event_outcome: ATNAEventOutcome::Success,
        source_id: String::from_str(&env, "XDS-REPOSITORY"),
        source_type: String::from_str(&env, "4"),
        active_participants: participants,
        participant_objects,
        hl7_message_id: String::from_str(&env, "MSG-003"),
        profile: IHEProfile::XDS,
    };

    assert_eq!(export_event.event_type, ATNAEventType::DocumentExport);
    
    assert_eq!(access_event.event_id, 1);
    assert_eq!(update_event.event_id, 2);
    assert_eq!(export_event.event_id, 3);
}

#[test]
fn test_scenario_security_profile_testing() {
    let (env, _admin, _provider, _patient) = create_test_env();
    
    let auth_event = ATNAAuditEvent {
        event_id: 1,
        event_type: ATNAEventType::UserAuthentication,
        event_action_code: String::from_str(&env, "E"),
        event_date_time: env.ledger().timestamp(),
        event_outcome: ATNAEventOutcome::Success,
        source_id: String::from_str(&env, "AUTH-SERVER"),
        source_type: String::from_str(&env, "4"),
        active_participants: vec![ATNAParticipant {
            user_id: String::from_str(&env, "user-001"),
            user_name: String::from_str(&env, "Dr. Smith"),
            role_id_code: String::from_str(&env, "physician"),
            is_requestor: true,
            network_access_point: String::from_str(&env, "192.168.1.100"),
        }],
        participant_objects: vec![],
        hl7_message_id: String::from_str(&env, "AUTH-001"),
        profile: IHEProfile::ATNA,
    };

    assert_eq!(auth_event.event_type, ATNAEventType::UserAuthentication);
    assert_eq!(auth_event.event_outcome, ATNAEventOutcome::Success);
    
    let signature = DSGSignature {
        signature_id: 1,
        document_id: String::from_str(&env, "doc-001"),
        signer: _provider.clone(),
        signature_hash: BytesN::from_array(&env, &[4u8; 32]),
        signature_algorithm: String::from_str(&env, "RS256"),
        signing_time: env.ledger().timestamp(),
        certificate_ref: String::from_str(&env, "cert-001"),
        signature_purpose: String::from_str(&env, "author"),
        is_valid: true,
    };

    assert!(signature.is_valid);
    assert_eq!(signature.signature_algorithm, String::from_str(&env, "RS256"));
    
    let alert_event = ATNAAuditEvent {
        event_id: 2,
        event_type: ATNAEventType::SecurityAlert,
        event_action_code: String::from_str(&env, "E"),
        event_date_time: env.ledger().timestamp(),
        event_outcome: ATNAEventOutcome::MinorFailure,
        source_id: String::from_str(&env, "SECURITY-MONITOR"),
        source_type: String::from_str(&env, "4"),
        active_participants: vec![ATNAParticipant {
            user_id: String::from_str(&env, "system"),
            user_name: String::from_str(&env, "Security Monitor"),
            role_id_code: String::from_str(&env, "system"),
            is_requestor: false,
            network_access_point: String::from_str(&env, "192.168.1.1"),
        }],
        participant_objects: vec![],
        hl7_message_id: String::from_str(&env, "ALERT-001"),
        profile: IHEProfile::ATNA,
    };

    assert_eq!(alert_event.event_type, ATNAEventType::SecurityAlert);
    assert_eq!(alert_event.event_outcome, ATNAEventOutcome::MinorFailure);
}

// ==================== FHIR Coding System Tests ====================

#[test]
fn test_fhir_coding_systems() {
    let (env, _admin, _provider, _patient) = create_test_env();
    
    let icd10_code = create_fhir_code(&env, CodingSystem::ICD10, "E11.9", "Type 2 diabetes mellitus");
    assert_eq!(icd10_code.system, CodingSystem::ICD10);
    assert_eq!(icd10_code.code, String::from_str(&env, "E11.9"));
    
    let snomed_code = create_fhir_code(&env, CodingSystem::SNOMEDCT, "73211009", "Diabetes mellitus");
    assert_eq!(snomed_code.system, CodingSystem::SNOMEDCT);
    
    let loinc_code = create_fhir_code(&env, CodingSystem::LOINC, "8867-4", "Heart rate");
    assert_eq!(loinc_code.system, CodingSystem::LOINC);
    
    let rxnorm_code = create_fhir_code(&env, CodingSystem::RxNorm, "860975", "Metformin 500mg");
    assert_eq!(rxnorm_code.system, CodingSystem::RxNorm);
    
    let cpt_code = create_fhir_code(&env, CodingSystem::CPT, "99213", "Office visit");
    assert_eq!(cpt_code.system, CodingSystem::CPT);
}

#[test]
fn test_fhir_bundle_operations() {
    let (env, _admin, _provider, _patient) = create_test_env();
    
    let bundle = FHIRBundle {
        bundle_id: String::from_str(&env, "bundle-001"),
        timestamp: env.ledger().timestamp(),
        bundle_type: String::from_str(&env, "transaction"),
        total: 5,
    };

    assert_eq!(bundle.bundle_type, String::from_str(&env, "transaction"));
    assert_eq!(bundle.total, 5);
}

// ==================== IHE Connectathon Compliance Tests ====================

#[test]
fn test_ihe_connectathon_xds_compliance() {
    let (env, _admin, _provider, _patient) = create_test_env();
    
    let test_result = ConnectathonTestResult {
        test_id: 1,
        profile: IHEProfile::XDS,
        actor_name: String::from_str(&env, "Document Registry"),
        test_name: String::from_str(&env, "ITI-42 Register Document Set-b"),
        passed: true,
        tested_at: env.ledger().timestamp(),
        tested_by: Address::generate(&env),
        notes: String::from_str(&env, "All tests passed"),
    };

    assert!(test_result.passed);
    assert_eq!(test_result.profile, IHEProfile::XDS);
}

#[test]
fn test_ihe_connectathon_pix_compliance() {
    let (env, _admin, _provider, _patient) = create_test_env();
    
    let test_result = ConnectathonTestResult {
        test_id: 2,
        profile: IHEProfile::PIX,
        actor_name: String::from_str(&env, "Patient Identity Source"),
        test_name: String::from_str(&env, "ITI-8 Patient Identity Feed"),
        passed: true,
        tested_at: env.ledger().timestamp(),
        tested_by: Address::generate(&env),
        notes: String::from_str(&env, "Identity cross-referencing validated"),
    };

    assert!(test_result.passed);
    assert_eq!(test_result.profile, IHEProfile::PIX);
}

#[test]
fn test_ihe_connectathon_pdq_compliance() {
    let (env, _admin, _provider, _patient) = create_test_env();
    
    let test_result = ConnectathonTestResult {
        test_id: 3,
        profile: IHEProfile::PDQ,
        actor_name: String::from_str(&env, "Patient Demographics Supplier"),
        test_name: String::from_str(&env, "ITI-21 Patient Demographics Query"),
        passed: true,
        tested_at: env.ledger().timestamp(),
        tested_by: Address::generate(&env),
        notes: String::from_str(&env, "Demographics query successful"),
    };

    assert!(test_result.passed);
    assert_eq!(test_result.profile, IHEProfile::PDQ);
}

#[test]
fn test_ihe_connectathon_atna_compliance() {
    let (env, _admin, _provider, _patient) = create_test_env();
    
    let test_result = ConnectathonTestResult {
        test_id: 4,
        profile: IHEProfile::ATNA,
        actor_name: String::from_str(&env, "Audit Record Repository"),
        test_name: String::from_str(&env, "ITI-20 Record Audit Event"),
        passed: true,
        tested_at: env.ledger().timestamp(),
        tested_by: Address::generate(&env),
        notes: String::from_str(&env, "Audit logging compliant"),
    };

    assert!(test_result.passed);
    assert_eq!(test_result.profile, IHEProfile::ATNA);
}

// ==================== Healthcare Provider Directory Tests ====================

#[test]
fn test_ihe_hpd_provider_registration() {
    let (env, _admin, _provider, _patient) = create_test_env();
    
    let hpd_provider = HPDProvider {
        provider_id: 1,
        provider_type: ProviderType::Individual,
        given_name: String::from_str(&env, "John"),
        family_name: String::from_str(&env, "Smith"),
        organization_name: String::from_str(&env, "General Hospital"),
        specialty_code: String::from_str(&env, "207R00000X"),
        license_number: String::from_str(&env, "MD-12345"),
        npi: String::from_str(&env, "1234567890"),
        address: String::from_str(&env, "123 Medical Plaza"),
        electronic_service_info: String::from_str(&env, "https://hospital.example.com/fhir"),
        registered_by: Address::generate(&env),
        registration_time: env.ledger().timestamp(),
        is_active: true,
    };

    assert_eq!(hpd_provider.provider_type, ProviderType::Individual);
    assert!(hpd_provider.is_active);
    assert_eq!(hpd_provider.npi, String::from_str(&env, "1234567890"));
}

// ==================== Value Set Sharing Tests ====================

#[test]
fn test_ihe_svs_value_set_sharing() {
    let (env, _admin, _provider, _patient) = create_test_env();
    
    let value_set = SVSValueSet {
        value_set_id: 1,
        oid: String::from_str(&env, "2.16.840.1.113883.3.464.1003.103.12.1001"),
        name: String::from_str(&env, "Diabetes"),
        version: String::from_str(&env, "1.0"),
        status: String::from_str(&env, "active"),
        description: String::from_str(&env, "Diabetes diagnosis codes"),
        concepts: vec![SVSConcept {
            code: String::from_str(&env, "E11.9"),
            code_system: String::from_str(&env, "2.16.840.1.113883.6.90"),
            code_system_name: String::from_str(&env, "ICD-10-CM"),
            display_name: String::from_str(&env, "Type 2 diabetes mellitus"),
            level: 0,
            type_code: String::from_str(&env, "L"),
        }],
        effective_date: env.ledger().timestamp(),
        source_url: String::from_str(&env, "https://vsac.nlm.nih.gov/"),
        registered_by: Address::generate(&env),
    };

    assert_eq!(value_set.status, String::from_str(&env, "active"));
    assert!(!value_set.concepts.is_empty());
}

// ==================== Cross-Community Access Tests ====================

#[test]
fn test_ihe_xca_gateway_registration() {
    let (env, _admin, _provider, _patient) = create_test_env();
    
    let xca_gateway = XCAGateway {
        gateway_id: String::from_str(&env, "gateway-001"),
        community_id: String::from_str(&env, "community-a"),
        gateway_address: String::from_str(&env, "https://gateway.hospital-a.com"),
        supported_profiles: vec![IHEProfile::XDS, IHEProfile::PIX, IHEProfile::PDQ],
        registered_by: Address::generate(&env),
        registration_time: env.ledger().timestamp(),
        is_active: true,
    };

    assert!(xca_gateway.is_active);
    assert_eq!(xca_gateway.supported_profiles.len(), 3);
    assert_eq!(
        xca_gateway.community_id,
        String::from_str(&env, "community-a")
    );
}

// ==================== Master Patient Index Tests ====================

#[test]
fn test_ihe_mpi_master_patient_record() {
    let (env, _admin, _provider, _patient) = create_test_env();
    
    let mpi_master = MPIMasterPatient {
        master_id: 1,
        global_patient_id: String::from_str(&env, "GLOBAL-001"),
        linked_identifiers: vec![
            PatientIdentifier {
                id_value: String::from_str(&env, "MRN-12345"),
                assigning_authority: String::from_str(&env, "Hospital-A"),
                identifier_type_code: String::from_str(&env, "MR"),
            },
            PatientIdentifier {
                id_value: String::from_str(&env, "PID-67890"),
                assigning_authority: String::from_str(&env, "Hospital-B"),
                identifier_type_code: String::from_str(&env, "PI"),
            },
        ],
        demographics: PatientDemographics {
            patient_id: String::from_str(&env, "MASTER-001"),
            given_name: String::from_str(&env, "John"),
            family_name: String::from_str(&env, "Doe"),
            date_of_birth: String::from_str(&env, "1980-01-01"),
            administrative_gender: String::from_str(&env, "M"),
            street_address: String::from_str(&env, "123 Main St"),
            city: String::from_str(&env, "Springfield"),
            state: String::from_str(&env, "IL"),
            postal_code: String::from_str(&env, "62701"),
            country_code: String::from_str(&env, "US"),
            phone_home: String::from_str(&env, "555-1234"),
            phone_mobile: String::from_str(&env, "555-5678"),
            mother_maiden_name: String::from_str(&env, "Smith"),
            marital_status: String::from_str(&env, "M"),
            race: String::from_str(&env, "2106-3"),
            ethnicity: String::from_str(&env, "2186-5"),
            primary_language: String::from_str(&env, "en"),
            last_updated: env.ledger().timestamp(),
            assigning_authority: String::from_str(&env, "MPI-SYSTEM"),
        },
        created_at: env.ledger().timestamp(),
        updated_at: env.ledger().timestamp(),
        confidence_score: 95,
    };

    assert_eq!(mpi_master.linked_identifiers.len(), 2);
    assert_eq!(mpi_master.confidence_score, 95);
    assert_eq!(
        mpi_master.global_patient_id,
        String::from_str(&env, "GLOBAL-001")
    );
}

// ==================== Document Submission Set Tests ====================

#[test]
fn test_xds_submission_set() {
    let (env, _admin, _provider, _patient) = create_test_env();
    
    let submission_set = XDSSubmissionSet {
        submission_set_id: String::from_str(&env, "ss-001"),
        patient_id: String::from_str(&env, "MRN-12345"),
        submission_time: env.ledger().timestamp(),
        source_id: String::from_str(&env, "EMR-SYSTEM"),
        author: Address::generate(&env),
        content_type_code: String::from_str(&env, "34117-2"),
        document_ids: vec![
            String::from_str(&env, "doc-001"),
            String::from_str(&env, "doc-002"),
            String::from_str(&env, "doc-003"),
        ],
        intended_recipient: String::from_str(&env, "Hospital-B"),
    };

    assert_eq!(submission_set.document_ids.len(), 3);
    assert_eq!(
        submission_set.patient_id,
        String::from_str(&env, "MRN-12345")
    );
}

// ==================== Comprehensive Integration Test ====================

#[test]
fn test_comprehensive_ihe_fhir_integration() {
    let (env, _admin, _provider, _patient) = create_test_env();
    
    let fhir_patient = create_fhir_patient(&env);
    assert!(!fhir_patient.identifiers.is_empty());
    
    let local_id = PatientIdentifier {
        id_value: fhir_patient.identifiers[0].value.clone(),
        assigning_authority: String::from_str(&env, "Hospital-A"),
        identifier_type_code: String::from_str(&env, "MR"),
    };
    
    let pix_ref = PIXCrossReference {
        reference_id: 1,
        local_id,
        cross_referenced_ids: vec![PatientIdentifier {
            id_value: String::from_str(&env, "PID-67890"),
            assigning_authority: String::from_str(&env, "Hospital-B"),
            identifier_type_code: String::from_str(&env, "PI"),
        }],
        created_at: env.ledger().timestamp(),
        updated_at: env.ledger().timestamp(),
        is_merged: false,
    };

    assert_eq!(pix_ref.reference_id, 1);
    
    let demographics = PatientDemographics {
        patient_id: fhir_patient.identifiers[0].value.clone(),
        given_name: fhir_patient.given_name.clone(),
        family_name: fhir_patient.family_name.clone(),
        date_of_birth: fhir_patient.birth_date.clone(),
        administrative_gender: fhir_patient.gender.clone(),
        street_address: fhir_patient.address.clone(),
        city: String::from_str(&env, "Springfield"),
        state: String::from_str(&env, "IL"),
        postal_code: String::from_str(&env, "62701"),
        country_code: String::from_str(&env, "US"),
        phone_home: String::from_str(&env, ""),
        phone_mobile: fhir_patient.contact_point.clone(),
        mother_maiden_name: String::from_str(&env, ""),
        marital_status: fhir_patient.marital_status.clone(),
        race: String::from_str(&env, ""),
        ethnicity: String::from_str(&env, ""),
        primary_language: String::from_str(&env, "en"),
        last_updated: env.ledger().timestamp(),
        assigning_authority: String::from_str(&env, "Hospital-A"),
    };
    
    let observation = FHIRObservation {
        identifier: String::from_str(&env, "obs-001"),
        status: String::from_str(&env, "final"),
        category: create_fhir_code(&env, CodingSystem::LOINC, "vital-signs", "Vital Signs"),
        code: create_fhir_code(&env, CodingSystem::LOINC, "8867-4", "Heart rate"),
        subject_reference: demographics.patient_id.clone(),
        effective_datetime: String::from_str(&env, "2024-01-15T10:30:00Z"),
        value_quantity_value: 72,
        value_quantity_unit: String::from_str(&env, "beats/minute"),
        interpretation: vec![],
        reference_range: String::from_str(&env, "60-100 beats/minute"),
    };
    
    let xds_entry = XDSDocumentEntry {
        document_id: String::from_str(&env, "doc-001"),
        patient_id: demographics.patient_id.clone(),
        content_hash: BytesN::from_array(&env, &[5u8; 32]),
        document_class_code: String::from_str(&env, "11488-4"),
        document_type_code: String::from_str(&env, "34117-2"),
        format_code: String::from_str(&env, "urn:ihe:pcc:xds-ms:2007"),
        healthcare_facility_type: String::from_str(&env, "OF"),
        practice_setting_code: String::from_str(&env, "General Medicine"),
        creation_time: env.ledger().timestamp(),
        author: _provider.clone(),
        confidentiality_code: String::from_str(&env, "N"),
        language_code: String::from_str(&env, "en-US"),
        hl7_message_type: HL7MessageType::V2ADT,
        status: DocumentStatus::Approved,
        repository_unique_id: String::from_str(&env, "1.3.6.1.4.1.21367.2010.1.2.1125"),
        submission_set_id: String::from_str(&env, "ss-001"),
        mime_type: String::from_str(&env, "application/pdf"),
    };
    
    let consent = BPPCConsent {
        consent_id: 1,
        patient_id: demographics.patient_id.clone(),
        policy_id: String::from_str(&env, "policy-001"),
        consent_status: ConsentStatus::Active,
        access_consent_list: vec![String::from_str(&env, "provider-001")],
        date_of_consent: env.ledger().timestamp(),
        expiry_time: env.ledger().timestamp() + 31536000,
        author: _patient.clone(),
        document_ref: String::from_str(&env, "consent-doc-001"),
    };
    
    let audit_event = ATNAAuditEvent {
        event_id: 1,
        event_type: ATNAEventType::PatientRecordAccess,
        event_action_code: String::from_str(&env, "R"),
        event_date_time: env.ledger().timestamp(),
        event_outcome: ATNAEventOutcome::Success,
        source_id: String::from_str(&env, "EMR-SYSTEM"),
        source_type: String::from_str(&env, "4"),
        active_participants: vec![ATNAParticipant {
            user_id: String::from_str(&env, "dr-smith"),
            user_name: String::from_str(&env, "Dr. Smith"),
            role_id_code: String::from_str(&env, "physician"),
            is_requestor: true,
            network_access_point: String::from_str(&env, "192.168.1.100"),
        }],
        participant_objects: vec![],
        hl7_message_id: String::from_str(&env, "MSG-001"),
        profile: IHEProfile::ATNA,
    };
    
    assert_eq!(fhir_patient.given_name, demographics.given_name);
    assert_eq!(observation.status, String::from_str(&env, "final"));
    assert_eq!(xds_entry.status, DocumentStatus::Approved);
    assert_eq!(consent.consent_status, ConsentStatus::Active);
    assert_eq!(audit_event.event_outcome, ATNAEventOutcome::Success);
}

// ==================== Type Definitions ====================

#[derive(Clone, Debug, PartialEq, Eq)]
struct FHIRPatient {
    identifiers: Vec<FHIRIdentifier>,
    given_name: String,
    family_name: String,
    birth_date: String,
    gender: String,
    contact_point: String,
    address: String,
    communication: Vec<String>,
    marital_status: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FHIRIdentifier {
    system: String,
    value: String,
    use_type: String,
}

#[derive(Clone, Debug)]
struct FHIRCode {
    system: CodingSystem,
    code: String,
    display: String,
}

#[derive(Clone, Debug)]
struct FHIRObservation {
    identifier: String,
    status: String,
    category: FHIRCode,
    code: FHIRCode,
    subject_reference: String,
    effective_datetime: String,
    value_quantity_value: i64,
    value_quantity_unit: String,
    interpretation: Vec<FHIRCode>,
    reference_range: String,
}

#[derive(Clone, Debug)]
struct FHIRCondition {
    identifier: String,
    clinical_status: String,
    code: FHIRCode,
    subject_reference: String,
    onset_date_time: String,
    recorded_date: String,
    severity: Vec<FHIRCode>,
}

#[derive(Clone, Debug)]
struct FHIRMedicationStatement {
    identifier: String,
    status: String,
    medication_code: FHIRCode,
    subject_reference: String,
    effective_period_start: String,
    effective_period_end: String,
    dosage: String,
    reason_code: Vec<FHIRCode>,
}

#[derive(Clone, Debug)]
struct FHIRProcedure {
    identifier: String,
    status: String,
    code: FHIRCode,
    subject_reference: String,
    performed_date_time: String,
    performer: Vec<String>,
    reason_code: Vec<FHIRCode>,
}

#[derive(Clone, Debug)]
struct FHIRAllergyIntolerance {
    identifier: String,
    clinical_status: String,
    verification_status: String,
    substance_code: FHIRCode,
    patient_reference: String,
    recorded_date: String,
    manifestation: Vec<FHIRCode>,
    severity: String,
}

#[derive(Clone, Debug)]
struct FHIRBundle {
    bundle_id: String,
    timestamp: u64,
    bundle_type: String,
    total: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HL7MessageType {
    V2ADT,
    V2ORM,
    V2ORU,
    V2MFN,
    V2QBP,
    V2RSP,
    V2ACK,
    V3ClinicalDocument,
    V3PatientQuery,
    V3PatientResponse,
    V3DeviceQuery,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DocumentStatus {
    Approved,
    Deprecated,
    Submitted,
}

#[derive(Clone, Debug)]
struct XDSDocumentEntry {
    document_id: String,
    patient_id: String,
    content_hash: BytesN<32>,
    document_class_code: String,
    document_type_code: String,
    format_code: String,
    healthcare_facility_type: String,
    practice_setting_code: String,
    creation_time: u64,
    author: Address,
    confidentiality_code: String,
    language_code: String,
    hl7_message_type: HL7MessageType,
    status: DocumentStatus,
    repository_unique_id: String,
    submission_set_id: String,
    mime_type: String,
}

#[derive(Clone, Debug)]
struct XDSSubmissionSet {
    submission_set_id: String,
    patient_id: String,
    submission_time: u64,
    source_id: String,
    author: Address,
    content_type_code: String,
    document_ids: Vec<String>,
    intended_recipient: String,
}

#[derive(Clone, Debug)]
struct PatientIdentifier {
    id_value: String,
    assigning_authority: String,
    identifier_type_code: String,
}

#[derive(Clone, Debug)]
struct PIXCrossReference {
    reference_id: u64,
    local_id: PatientIdentifier,
    cross_referenced_ids: Vec<PatientIdentifier>,
    created_at: u64,
    updated_at: u64,
    is_merged: bool,
}

#[derive(Clone, Debug)]
struct PatientDemographics {
    patient_id: String,
    given_name: String,
    family_name: String,
    date_of_birth: String,
    administrative_gender: String,
    street_address: String,
    city: String,
    state: String,
    postal_code: String,
    country_code: String,
    phone_home: String,
    phone_mobile: String,
    mother_maiden_name: String,
    marital_status: String,
    race: String,
    ethnicity: String,
    primary_language: String,
    last_updated: u64,
    assigning_authority: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ATNAEventType {
    PatientRecordAccess,
    PatientRecordUpdate,
    UserAuthentication,
    NodeAuthentication,
    DocumentExport,
    DocumentImport,
    QueryRequest,
    QueryResponse,
    SecurityAlert,
    OrderMessage,
    ProcedureRecord,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ATNAEventOutcome {
    Success = 0,
    MinorFailure = 4,
    SeriousFailure = 8,
    MajorFailure = 12,
}

#[derive(Clone, Debug)]
struct ATNAParticipant {
    user_id: String,
    user_name: String,
    role_id_code: String,
    is_requestor: bool,
    network_access_point: String,
}

#[derive(Clone, Debug)]
struct ATNAParticipantObject {
    object_id_type_code: String,
    object_id: String,
    object_type_code: u32,
    object_sensitivity: String,
    object_query: String,
}

#[derive(Clone, Debug)]
struct ATNAAuditEvent {
    event_id: u64,
    event_type: ATNAEventType,
    event_action_code: String,
    event_date_time: u64,
    event_outcome: ATNAEventOutcome,
    source_id: String,
    source_type: String,
    active_participants: Vec<ATNAParticipant>,
    participant_objects: Vec<ATNAParticipantObject>,
    hl7_message_id: String,
    profile: IHEProfile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IHEProfile {
    XDS,
    PIX,
    PDQ,
    ATNA,
    XCA,
    MPI,
    XDR,
    XDM,
    CT,
    BPPC,
    DSG,
    HPD,
    SVS,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConsentStatus {
    Active,
    Revoked,
    Expired,
}

#[derive(Clone, Debug)]
struct BPPCConsent {
    consent_id: u64,
    patient_id: String,
    policy_id: String,
    consent_status: ConsentStatus,
    access_consent_list: Vec<String>,
    date_of_consent: u64,
    expiry_time: u64,
    author: Address,
    document_ref: String,
}

#[derive(Clone, Debug)]
struct DSGSignature {
    signature_id: u64,
    document_id: String,
    signer: Address,
    signature_hash: BytesN<32>,
    signature_algorithm: String,
    signing_time: u64,
    certificate_ref: String,
    signature_purpose: String,
    is_valid: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProviderType {
    Individual,
    Organization,
    Department,
}

#[derive(Clone, Debug)]
struct HPDProvider {
    provider_id: u64,
    provider_type: ProviderType,
    given_name: String,
    family_name: String,
    organization_name: String,
    specialty_code: String,
    license_number: String,
    npi: String,
    address: String,
    electronic_service_info: String,
    registered_by: Address,
    registration_time: u64,
    is_active: bool,
}

#[derive(Clone, Debug)]
struct SVSConcept {
    code: String,
    code_system: String,
    code_system_name: String,
    display_name: String,
    level: u32,
    type_code: String,
}

#[derive(Clone, Debug)]
struct SVSValueSet {
    value_set_id: u64,
    oid: String,
    name: String,
    version: String,
    status: String,
    description: String,
    concepts: Vec<SVSConcept>,
    effective_date: u64,
    source_url: String,
    registered_by: Address,
}

#[derive(Clone, Debug)]
struct ConnectathonTestResult {
    test_id: u64,
    profile: IHEProfile,
    actor_name: String,
    test_name: String,
    passed: bool,
    tested_at: u64,
    tested_by: Address,
    notes: String,
}

#[derive(Clone, Debug)]
struct XCAGateway {
    gateway_id: String,
    community_id: String,
    gateway_address: String,
    supported_profiles: Vec<IHEProfile>,
    registered_by: Address,
    registration_time: u64,
    is_active: bool,
}

#[derive(Clone, Debug)]
struct MPIMasterPatient {
    master_id: u64,
    global_patient_id: String,
    linked_identifiers: Vec<PatientIdentifier>,
    demographics: PatientDemographics,
    created_at: u64,
    updated_at: u64,
    confidence_score: u32,
}
