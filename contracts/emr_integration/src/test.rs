use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env, Map, String};

fn setup(env: &Env) -> (EMRIntegrationContractClient<'_>, Address, Address) {
    let id = Address::generate(env);
    env.register_contract(&id, EMRIntegrationContract);
    let client = EMRIntegrationContractClient::new(env, &id);

    let admin = Address::generate(env);
    let fhir_contract = Address::generate(env);
    client.mock_all_auths().initialize(&admin, &fhir_contract);

    client.mock_all_auths().register_emr_system(
        &admin,
        &String::from_str(env, "epic-prod"),
        &String::from_str(env, "Epic"),
        &String::from_str(env, "interop@epic.example"),
        &String::from_str(env, "2026.1"),
        &soroban_sdk::vec![
            env,
            String::from_str(env, "HL7 v2"),
            String::from_str(env, "HL7 v3"),
            String::from_str(env, "CDA")
        ],
        &soroban_sdk::vec![env, String::from_str(env, "mllp://epic-prod"),],
    );

    (client, admin, fhir_contract)
}

fn sample_metadata(env: &Env) -> Map<String, String> {
    let mut metadata = Map::new(env);
    metadata.set(
        String::from_str(env, "control_id"),
        String::from_str(env, "MSG-100"),
    );
    metadata.set(
        String::from_str(env, "patient_id"),
        String::from_str(env, "PAT-001"),
    );
    metadata.set(
        String::from_str(env, "patient_name"),
        String::from_str(env, "DOE^JANE"),
    );
    metadata.set(
        String::from_str(env, "document_title"),
        String::from_str(env, "Continuity of Care"),
    );
    metadata.set(
        String::from_str(env, "document_text"),
        String::from_str(env, "Patient is stable."),
    );
    metadata
}

#[test]
fn initialize_and_generate_hl7_v2_message() {
    let env = Env::default();
    let (client, _admin, _) = setup(&env);

    let generated = client.mock_all_auths().generate_message(
        &Address::generate(&env),
        &String::from_str(&env, "msg-1"),
        &String::from_str(&env, "epic-prod"),
        &MessagingStandard::HL7v2,
        &String::from_str(&env, "2.5.1"),
        &String::from_str(&env, "ADT^A01"),
        &CharacterEncoding::UTF8,
        &TransportProtocol::MLLP,
        &String::from_str(&env, "application/hl7-v2"),
        &sample_metadata(&env),
    );

    assert_eq!(generated.standard, MessagingStandard::HL7v2);
    assert_eq!(generated.message_type, String::from_str(&env, "ADT^A01"));
    assert!(generated.raw_payload.len() > 20);
}

#[test]
fn parse_hl7_v2_message_extracts_header_fields() {
    let env = Env::default();
    let (client, _admin, _) = setup(&env);

    let payload = String::from_str(
        &env,
        "MSH|^~\\&|Uzima|Main|EMR|Receiving|20260328090000||ORU^R01|CTRL-99|P|2.5.1||||||UTF-8\rPID|1||PAT-001||DOE^JANE\rOBX|1|TX|NOTE||All good",
    );

    let parsed = client.mock_all_auths().parse_message(
        &Address::generate(&env),
        &String::from_str(&env, "msg-2"),
        &String::from_str(&env, "epic-prod"),
        &CharacterEncoding::UTF8,
        &TransportProtocol::MLLP,
        &String::from_str(&env, "application/hl7-v2"),
        &payload,
    );

    assert_eq!(parsed.message_type, String::from_str(&env, "ORU^R01"));
    assert_eq!(parsed.control_id, String::from_str(&env, "CTRL-99"));
    assert!(parsed.segment_count >= 3);
}

#[test]
fn supports_hl7_v3_and_cda_documents() {
    let env = Env::default();
    let (client, _admin, _) = setup(&env);

    let v3 = client.mock_all_auths().generate_message(
        &Address::generate(&env),
        &String::from_str(&env, "msg-v3"),
        &String::from_str(&env, "epic-prod"),
        &MessagingStandard::HL7v3,
        &String::from_str(&env, "3.0"),
        &String::from_str(&env, "PRPA_IN201301UV02"),
        &CharacterEncoding::UTF16,
        &TransportProtocol::HTTP,
        &String::from_str(&env, "application/xml"),
        &sample_metadata(&env),
    );
    assert_eq!(v3.standard, MessagingStandard::HL7v3);

    let cda = client.mock_all_auths().generate_message(
        &Address::generate(&env),
        &String::from_str(&env, "msg-cda"),
        &String::from_str(&env, "epic-prod"),
        &MessagingStandard::CDA,
        &String::from_str(&env, "R2"),
        &String::from_str(&env, "ClinicalDocument"),
        &CharacterEncoding::ISO88591,
        &TransportProtocol::HTTP,
        &String::from_str(&env, "application/xml+cda"),
        &sample_metadata(&env),
    );
    assert_eq!(cda.standard, MessagingStandard::CDA);
    assert!(cda.raw_payload.len() > 50);
}

#[test]
fn transforms_messages_between_supported_standards() {
    let env = Env::default();
    let (client, _admin, _) = setup(&env);
    let sender = Address::generate(&env);

    client.mock_all_auths().generate_message(
        &sender,
        &String::from_str(&env, "src-msg"),
        &String::from_str(&env, "epic-prod"),
        &MessagingStandard::HL7v2,
        &String::from_str(&env, "2.5.1"),
        &String::from_str(&env, "ADT^A08"),
        &CharacterEncoding::UTF8,
        &TransportProtocol::MLLP,
        &String::from_str(&env, "application/hl7-v2"),
        &sample_metadata(&env),
    );

    let transform = client.mock_all_auths().transform_message(
        &sender,
        &String::from_str(&env, "xfm-1"),
        &String::from_str(&env, "src-msg"),
        &String::from_str(&env, "target-msg"),
        &MessagingStandard::CDA,
        &String::from_str(&env, "R2"),
        &String::from_str(&env, "ClinicalDocument"),
        &CharacterEncoding::UTF8,
        &TransportProtocol::HTTP,
        &String::from_str(&env, "application/xml+cda"),
    );

    let target = client.get_message(&String::from_str(&env, "target-msg"));
    assert_eq!(transform.target_standard, MessagingStandard::CDA);
    assert_eq!(target.standard, MessagingStandard::CDA);
}

#[test]
fn validates_messages_and_surfaces_issues() {
    let env = Env::default();
    let (client, _admin, _) = setup(&env);
    let sender = Address::generate(&env);

    client.mock_all_auths().parse_message(
        &sender,
        &String::from_str(&env, "bad-msg"),
        &String::from_str(&env, "epic-prod"),
        &CharacterEncoding::UTF8,
        &TransportProtocol::MLLP,
        &String::from_str(&env, "application/hl7-v2"),
        &String::from_str(
            &env,
            "MSH|^~\\&|Uzima|Main|EMR|Receiving|20260328090000||ACK|CTRL-88|P|2.5.1",
        ),
    );

    let report = client.mock_all_auths().validate_message(
        &sender,
        &String::from_str(&env, "report-1"),
        &String::from_str(&env, "bad-msg"),
    );

    assert!(!report.is_valid);
    assert!(!report.issues.is_empty());
}

#[test]
fn wraps_payload_for_mllp_and_http_transport() {
    let env = Env::default();
    let (client, _admin, _) = setup(&env);
    let sender = Address::generate(&env);

    client.mock_all_auths().generate_message(
        &sender,
        &String::from_str(&env, "mllp-msg"),
        &String::from_str(&env, "epic-prod"),
        &MessagingStandard::HL7v2,
        &String::from_str(&env, "2.5.1"),
        &String::from_str(&env, "ORM^O01"),
        &CharacterEncoding::ASCII,
        &TransportProtocol::MLLP,
        &String::from_str(&env, "application/hl7-v2"),
        &sample_metadata(&env),
    );

    let mllp = client.wrap_transport_payload(&String::from_str(&env, "mllp-msg"));
    assert!(mllp.len() > 10);

    client.mock_all_auths().generate_message(
        &sender,
        &String::from_str(&env, "http-msg"),
        &String::from_str(&env, "epic-prod"),
        &MessagingStandard::HL7v3,
        &String::from_str(&env, "3.0"),
        &String::from_str(&env, "PRPA_IN201302UV02"),
        &CharacterEncoding::UTF8,
        &TransportProtocol::HTTP,
        &String::from_str(&env, "application/xml"),
        &sample_metadata(&env),
    );

    let http = client.wrap_transport_payload(&String::from_str(&env, "http-msg"));
    assert!(http.len() > 30);
}

#[test]
fn exposes_more_than_fifty_supported_message_types() {
    let env = Env::default();
    let (client, _, _) = setup(&env);

    let message_types = client.get_supported_message_types();
    assert!(message_types.len() > 50);
}

#[test]
fn benchmark_meets_throughput_target() {
    let env = Env::default();
    let (client, _, _) = setup(&env);

    let benchmark = client.benchmark_message_processing(
        &String::from_str(&env, "bm-1"),
        &String::from_str(&env, "ADT^A01"),
        &CharacterEncoding::UTF8,
        &TransportProtocol::MLLP,
        &2048,
    );

    assert!(benchmark.messages_per_second > 1000);
}
