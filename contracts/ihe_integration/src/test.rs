use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, Map, String, Vec};

fn setup() -> (Env, Address, IHEIntegrationContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, IHEIntegrationContract);
    let client = IHEIntegrationContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, admin, client)
}

// ==================== Initialization ====================

#[test]
fn test_initialize() {
    let (_, _, _) = setup();
}

#[test]
fn test_double_initialize_fails() {
    let (env, admin, client) = setup();
    let _ = env;
    let result = client.try_initialize(&admin);
    assert!(result.is_err());
}

// ==================== XDS Tests ====================

fn make_xds_entry(env: &Env, author: &Address) -> XDSDocumentEntry {
    XDSDocumentEntry {
        document_id: String::from_str(env, "DOC-001"),
        patient_id: String::from_str(env, "PAT-001"),
        content_hash: BytesN::from_array(env, &[1u8; 32]),
        document_class_code: String::from_str(env, "11488-4"),
        document_type_code: String::from_str(env, "34117-2"),
        format_code: String::from_str(env, "urn:ihe:pcc:xds-ms:2007"),
        healthcare_facility_type: String::from_str(env, "OF"),
        practice_setting_code: String::from_str(env, "General Medicine"),
        creation_time: 1_700_000_000u64,
        author: author.clone(),
        confidentiality_code: String::from_str(env, "N"),
        language_code: String::from_str(env, "en-US"),
        hl7_message_type: HL7MessageType::V2ADT,
        status: DocumentStatus::Approved,
        repository_unique_id: String::from_str(env, "1.3.6.1.4.1.21367.2010.1.2.1125"),
        submission_set_id: String::from_str(env, "SS-001"),
        mime_type: String::from_str(env, "text/xml"),
    }
}

#[test]
fn test_xds_register_and_retrieve_document() {
    let (env, _, client) = setup();
    let author = Address::generate(&env);
    let entry = make_xds_entry(&env, &author);

    client.xds_register_document(&author, &entry);

    let retrieved = client.xds_retrieve_document(&author, &String::from_str(&env, "DOC-001"));
    assert_eq!(retrieved.document_id, entry.document_id);
    assert_eq!(retrieved.patient_id, entry.patient_id);
}

#[test]
fn test_xds_duplicate_document_fails() {
    let (env, _, client) = setup();
    let author = Address::generate(&env);
    let entry = make_xds_entry(&env, &author);

    client.xds_register_document(&author, &entry);
    let result = client.try_xds_register_document(&author, &entry);
    assert!(result.is_err());
}

#[test]
fn test_xds_query_documents() {
    let (env, _, client) = setup();
    let author = Address::generate(&env);
    let entry = make_xds_entry(&env, &author);
    client.xds_register_document(&author, &entry);

    let docs = client.xds_query_documents(&author, &String::from_str(&env, "PAT-001"));
    assert_eq!(docs.len(), 1);
}

#[test]
fn test_xds_deprecate_document() {
    let (env, _, client) = setup();
    let author = Address::generate(&env);
    let entry = make_xds_entry(&env, &author);
    client.xds_register_document(&author, &entry);

    client.xds_deprecate_document(&author, &String::from_str(&env, "DOC-001"));

    let result = client.try_xds_retrieve_document(&author, &String::from_str(&env, "DOC-001"));
    assert!(result.is_err());
}

// ==================== PIX Tests ====================

fn make_patient_id(env: &Env, value: &str, authority: &str) -> PatientIdentifier {
    PatientIdentifier {
        id_value: String::from_str(env, value),
        assigning_authority: String::from_str(env, authority),
        identifier_type_code: String::from_str(env, "MR"),
    }
}

#[test]
fn test_pix_register_and_query() {
    let (env, _, client) = setup();
    let actor = Address::generate(&env);

    let local_id = make_patient_id(&env, "PAT-001", "DOMAIN_A");
    let mut cross_ids = Vec::new(&env);
    cross_ids.push_back(make_patient_id(&env, "XPAT-999", "DOMAIN_B"));

    let ref_id = client.pix_register_patient(&actor, &local_id, &cross_ids);
    assert_eq!(ref_id, 0u64);

    let refs = client.pix_query_identifiers(&actor, &String::from_str(&env, "PAT-001"));
    assert_eq!(refs.len(), 1);
    assert_eq!(refs.get(0).unwrap().reference_id, 0u64);
}

#[test]
fn test_pix_query_nonexistent_patient_fails() {
    let (env, _, client) = setup();
    let actor = Address::generate(&env);
    let result = client.try_pix_query_identifiers(&actor, &String::from_str(&env, "UNKNOWN"));
    assert!(result.is_err());
}

#[test]
fn test_pix_merge_patients() {
    let (env, _, client) = setup();
    let actor = Address::generate(&env);

    let local_a = make_patient_id(&env, "PAT-A", "DOMAIN_A");
    let local_b = make_patient_id(&env, "PAT-B", "DOMAIN_A");
    let cross_a = Vec::new(&env);
    let cross_b = Vec::new(&env);

    let ref_a = client.pix_register_patient(&actor, &local_a, &cross_a);
    let ref_b = client.pix_register_patient(&actor, &local_b, &cross_b);

    client.pix_merge_patients(&actor, &ref_a, &ref_b);
}

// ==================== PDQ Tests ====================

fn make_demographics(env: &Env) -> PatientDemographics {
    PatientDemographics {
        patient_id: String::from_str(env, "PAT-001"),
        given_name: String::from_str(env, "Jane"),
        family_name: String::from_str(env, "Doe"),
        date_of_birth: String::from_str(env, "19900101"),
        administrative_gender: String::from_str(env, "F"),
        street_address: String::from_str(env, "123 Main St"),
        city: String::from_str(env, "Springfield"),
        state: String::from_str(env, "IL"),
        postal_code: String::from_str(env, "62701"),
        country_code: String::from_str(env, "US"),
        phone_home: String::from_str(env, "2175550100"),
        phone_mobile: String::from_str(env, "2175550101"),
        mother_maiden_name: String::from_str(env, "Smith"),
        marital_status: String::from_str(env, "S"),
        race: String::from_str(env, "2106-3"),
        ethnicity: String::from_str(env, "2186-5"),
        primary_language: String::from_str(env, "en"),
        last_updated: 0u64,
        assigning_authority: String::from_str(env, "DOMAIN_A"),
    }
}

#[test]
fn test_pdq_register_and_get_demographics() {
    let (env, _, client) = setup();
    let actor = Address::generate(&env);
    let demographics = make_demographics(&env);

    client.pdq_register_demographics(&actor, &demographics);

    let retrieved = client.pdq_get_demographics(&actor, &String::from_str(&env, "PAT-001"));
    assert_eq!(retrieved.given_name, demographics.given_name);
    assert_eq!(retrieved.family_name, demographics.family_name);
}

#[test]
fn test_pdq_query() {
    let (env, _, client) = setup();
    let actor = Address::generate(&env);
    let mut params = Map::new(&env);
    params.set(
        String::from_str(&env, "family_name"),
        String::from_str(&env, "Doe"),
    );

    let query_id = client.pdq_query(
        &actor,
        &params,
        &String::from_str(&env, "HIS_SYSTEM"),
        &HL7MessageType::V2QBP,
        &String::from_str(&env, "DOMAIN_A"),
    );
    assert_eq!(query_id, 0u64);
}

// ==================== ATNA Tests ====================

#[test]
fn test_atna_log_event() {
    let (env, _, client) = setup();
    let actor = Address::generate(&env);

    let event_id = client.atna_log_event(
        &actor,
        &ATNAEventType::PatientRecordAccess,
        &String::from_str(&env, "R"),
        &ATNAEventOutcome::Success,
        &String::from_str(&env, "HIS_NODE"),
        &String::from_str(&env, "4"),
        &Vec::new(&env),
        &Vec::new(&env),
        &String::from_str(&env, "MSG-001"),
        &IHEProfile::ATNA,
    );
    assert_eq!(event_id, 0u64);

    let event = client.atna_get_event(&event_id);
    assert_eq!(event.event_type, ATNAEventType::PatientRecordAccess);
    assert_eq!(event.event_outcome, ATNAEventOutcome::Success);
}

#[test]
fn test_atna_authenticate_node() {
    let (env, _, client) = setup();
    let node = Address::generate(&env);

    let event_id = client.atna_authenticate_node(
        &node,
        &String::from_str(&env, "NODE-001"),
        &BytesN::from_array(&env, &[2u8; 32]),
    );
    assert_eq!(event_id, 0u64);
}

// ==================== XCA Tests ====================

#[test]
fn test_xca_register_and_query_gateway() {
    let (env, admin, client) = setup();

    let mut profiles = Vec::new(&env);
    profiles.push_back(IHEProfile::XDS);
    profiles.push_back(IHEProfile::PDQ);

    let gateway = XCAGateway {
        gateway_id: String::from_str(&env, "GW-001"),
        community_id: String::from_str(&env, "COMM-001"),
        gateway_address: String::from_str(&env, "https://gateway.example.com"),
        supported_profiles: profiles,
        registered_by: admin.clone(),
        registration_time: 0u64,
        is_active: true,
    };

    client.xca_register_gateway(&admin, &gateway);

    let result = client.xca_initiate_query(
        &admin,
        &String::from_str(&env, "GW-001"),
        &String::from_str(&env, "PAT-001"),
    );
    assert_eq!(result.gateway_id, gateway.gateway_id);
}

// ==================== MPI Tests ====================

#[test]
fn test_mpi_register_and_find_patient() {
    let (env, _, client) = setup();
    let actor = Address::generate(&env);
    let demographics = make_demographics(&env);

    let master_id = client.mpi_register_master_patient(
        &actor,
        &String::from_str(&env, "GLOBAL-PAT-001"),
        &demographics,
        &Vec::new(&env),
        &95u32,
    );
    assert_eq!(master_id, 0u64);

    let master = client.mpi_find_patient(&actor, &String::from_str(&env, "GLOBAL-PAT-001"));
    assert_eq!(master.master_id, 0u64);
    assert_eq!(master.confidence_score, 95u32);
}

// ==================== BPPC Tests ====================

#[test]
fn test_bppc_register_verify_revoke() {
    let (env, _, client) = setup();
    let author = Address::generate(&env);

    let mut acl = Vec::new(&env);
    acl.push_back(String::from_str(&env, "POLICY-ALLOW-TREATMENT"));

    let consent_id = client.bppc_register_consent(
        &author,
        &String::from_str(&env, "PAT-001"),
        &String::from_str(&env, "POLICY-001"),
        &acl,
        &9_999_999_999u64,
        &String::from_str(&env, "ipfs://QmABC"),
    );

    let consent = client.bppc_verify_consent(&consent_id);
    assert_eq!(consent.consent_status, ConsentStatus::Active);

    client.bppc_revoke_consent(&author, &consent_id);
    let result = client.try_bppc_verify_consent(&consent_id);
    assert!(result.is_err());
}

// ==================== DSG Tests ====================

#[test]
fn test_dsg_sign_and_verify() {
    let (env, _, client) = setup();
    let signer = Address::generate(&env);

    let sig_id = client.dsg_sign_document(
        &signer,
        &String::from_str(&env, "DOC-001"),
        &BytesN::from_array(&env, &[3u8; 32]),
        &String::from_str(&env, "RSA-SHA256"),
        &String::from_str(&env, "CERT-REF-001"),
        &String::from_str(&env, "1.2.840.10065.1.12.1.1"),
    );

    let sig = client.dsg_verify_signature(&sig_id);
    assert!(sig.is_valid);
    assert_eq!(sig.signer, signer);
}

#[test]
fn test_dsg_get_document_signatures() {
    let (env, _, client) = setup();
    let signer = Address::generate(&env);

    client.dsg_sign_document(
        &signer,
        &String::from_str(&env, "DOC-002"),
        &BytesN::from_array(&env, &[4u8; 32]),
        &String::from_str(&env, "RSA-SHA256"),
        &String::from_str(&env, "CERT-002"),
        &String::from_str(&env, "author"),
    );

    let sigs = client.dsg_get_document_signatures(&String::from_str(&env, "DOC-002"));
    assert_eq!(sigs.len(), 1);
}

// ==================== HPD Tests ====================

#[test]
fn test_hpd_register_and_get_provider() {
    let (env, _, client) = setup();
    let actor = Address::generate(&env);

    let provider = HPDProvider {
        provider_id: 0u64,
        provider_type: ProviderType::Individual,
        given_name: String::from_str(&env, "Alice"),
        family_name: String::from_str(&env, "Smith"),
        organization_name: String::from_str(&env, "City Hospital"),
        specialty_code: String::from_str(&env, "394814009"),
        license_number: String::from_str(&env, "LIC-12345"),
        npi: String::from_str(&env, "1234567890"),
        address: String::from_str(&env, "456 Hospital Ave"),
        electronic_service_info: String::from_str(&env, "urn:oid:1.2.3.4.5"),
        registered_by: actor.clone(),
        registration_time: 0u64,
        is_active: true,
    };

    let provider_id = client.hpd_register_provider(&actor, &provider);
    let retrieved = client.hpd_get_provider(&provider_id);
    assert_eq!(retrieved.npi, provider.npi);
    assert_eq!(retrieved.given_name, provider.given_name);
}

// ==================== SVS Tests ====================

#[test]
fn test_svs_register_and_get_value_set() {
    let (env, _, client) = setup();
    let actor = Address::generate(&env);

    let mut concepts = Vec::new(&env);
    concepts.push_back(SVSConcept {
        code: String::from_str(&env, "N"),
        code_system: String::from_str(&env, "2.16.840.1.113883.5.25"),
        code_system_name: String::from_str(&env, "Confidentiality"),
        display_name: String::from_str(&env, "Normal"),
        level: 0u32,
        type_code: String::from_str(&env, "L"),
    });

    let value_set = SVSValueSet {
        value_set_id: 0u64,
        oid: String::from_str(&env, "1.3.6.1.4.1.21367.100.1"),
        name: String::from_str(&env, "XDS Confidentiality Codes"),
        version: String::from_str(&env, "2023-01-01"),
        status: String::from_str(&env, "active"),
        description: String::from_str(&env, "Confidentiality codes for XDS documents"),
        concepts,
        effective_date: 1_672_531_200u64,
        source_url: String::from_str(&env, "http://ihe.net/"),
        registered_by: actor.clone(),
    };

    let vs_id = client.svs_register_value_set(&actor, &value_set);

    let retrieved =
        client.svs_get_value_set_by_oid(&String::from_str(&env, "1.3.6.1.4.1.21367.100.1"));
    assert_eq!(retrieved.value_set_id, vs_id);
    assert_eq!(retrieved.concepts.len(), 1);
}

// ==================== Connectathon Tests ====================

#[test]
fn test_connectathon_record_and_check_compliance() {
    let (env, _, client) = setup();
    let tester = Address::generate(&env);

    client.connectathon_record_test(
        &tester,
        &IHEProfile::XDS,
        &String::from_str(&env, "Document Registry"),
        &String::from_str(&env, "ITI-42 Register Document Set-b"),
        &true,
        &String::from_str(&env, "All assertions passed"),
    );

    let compliant = client.connectathon_is_compliant(&IHEProfile::XDS);
    assert!(compliant);
}

#[test]
fn test_connectathon_failing_test_is_not_compliant() {
    let (env, _, client) = setup();
    let tester = Address::generate(&env);

    client.connectathon_record_test(
        &tester,
        &IHEProfile::PIX,
        &String::from_str(&env, "Patient Identity Source"),
        &String::from_str(&env, "ITI-8 Patient Identity Feed"),
        &false,
        &String::from_str(&env, "Assertion failed: missing PID-3"),
    );

    let compliant = client.connectathon_is_compliant(&IHEProfile::PIX);
    assert!(!compliant);
}

#[test]
fn test_connectathon_no_tests_is_not_compliant() {
    let (env, _, _client) = setup();
    let _ = env;
    let (env2, _, client2) = setup();
    let compliant = client2.connectathon_is_compliant(&IHEProfile::SVS);
    let _ = env2;
    assert!(!compliant);
}

// ==================== CT Tests ====================

#[test]
fn test_ct_record_time_sync() {
    let (env, _, client) = setup();
    let actor = Address::generate(&env);

    let drift = client.ct_record_time_sync(
        &actor,
        &String::from_str(&env, "NODE-CT-001"),
        &1_700_000_100u64,
    );
    // drift value depends on ledger time in test env; just assert it's u64
    let _ = drift;
}

// ==================== XDR / XDM Tests ====================

#[test]
fn test_xdr_send_document() {
    let (env, _, client) = setup();
    let sender = Address::generate(&env);
    let entry = make_xds_entry(&env, &sender);

    client.xdr_send_document(&sender, &entry, &String::from_str(&env, "RECIPIENT-001"));
}

#[test]
fn test_xdm_record_media_package() {
    let (env, _, client) = setup();
    let actor = Address::generate(&env);

    let mut doc_ids = Vec::new(&env);
    doc_ids.push_back(String::from_str(&env, "DOC-001"));
    doc_ids.push_back(String::from_str(&env, "DOC-002"));

    client.xdm_record_media_package(
        &actor,
        &String::from_str(&env, "PKG-001"),
        &String::from_str(&env, "PAT-001"),
        &BytesN::from_array(&env, &[5u8; 32]),
        &String::from_str(&env, "CD-ROM"),
        &doc_ids,
    );
}
