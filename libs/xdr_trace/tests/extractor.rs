use std::process::Command;
use xdr_trace::{validate_trace_record, ContractTraceJson, TraceEventJson, TypedValue};

#[test]
fn test_extractor_cli_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_trace_extractor"))
        .arg("--help")
        .output()
        .expect("failed to execute trace_extractor");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Usage: trace_extractor"));
}

#[test]
fn test_extractor_cli_golden_fixture_no_validate() {
    let hex_fixture = include_str!("fixtures/simple_invoke_trace.sorobanmeta.xdr.hex").trim();

    let output = Command::new(env!("CARGO_BIN_EXE_trace_extractor"))
        .arg("--hex")
        .arg(hex_fixture)
        .arg("--no-validate-registry")
        .output()
        .expect("failed to execute trace_extractor");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let record: ContractTraceJson =
        serde_json::from_str(&stdout).expect("output must be valid ContractTraceJson");

    assert_eq!(record.schema_version, "1.0.0");
    assert_eq!(record.function, "burn");
    assert_eq!(record.status, "success");
    assert_eq!(record.return_value.type_, "u32");
    assert_eq!(record.return_value.value, serde_json::json!(42));
    assert_eq!(record.events.len(), 1);
    assert_eq!(record.events[0].name, "transfer");
    assert_eq!(record.events[0].topics, vec!["transfer", "alice", "bob"]);
}

#[test]
fn test_extractor_cli_unknown_event_fails_registry() {
    let hex_fixture = include_str!("fixtures/simple_invoke_trace.sorobanmeta.xdr.hex").trim();

    let output = Command::new(env!("CARGO_BIN_EXE_trace_extractor"))
        .arg("--hex")
        .arg(hex_fixture)
        .arg("--contract-name")
        .arg("medical_records")
        .output()
        .expect("failed to execute trace_extractor");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("absent from the authoritative registry"),
        "stderr was: {stderr}"
    );
}

#[test]
fn test_extractor_cli_stdin_pipe() {
    use std::io::Write;
    use std::process::Stdio;

    let hex_fixture = include_str!("fixtures/simple_invoke_trace.sorobanmeta.xdr.hex").trim();

    let mut child = Command::new(env!("CARGO_BIN_EXE_trace_extractor"))
        .arg("--no-validate-registry")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn trace_extractor");

    {
        let stdin = child.stdin.as_mut().expect("failed to open stdin");
        stdin
            .write_all(hex_fixture.as_bytes())
            .expect("failed to write to stdin");
    }

    let output = child.wait_with_output().expect("failed to wait on child");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let record: ContractTraceJson =
        serde_json::from_str(&stdout).expect("output must be valid ContractTraceJson");
    assert_eq!(record.function, "burn");
}

#[test]
fn test_validate_trace_record_valid() {
    let record = ContractTraceJson {
        schema_version: "1.0.0".to_string(),
        trace_id: "7f1c2b9e4d8a3f0c6e5b7a9d1c3e5f7a9b1d3c5e7f9a1b3d5c7e9f0a2b4d6c8e".to_string(),
        contract: "CDDEVC3VPQVXMB6U4Q7ADJX4QN2XMFPKS7DCGB2BXTWJA3CLZ4VJOU66".to_string(),
        contract_name: "medical_records".to_string(),
        function: "create_record".to_string(),
        account: Some("GDDEVC3VPQVXMB6U4Q7ADJX4QN2XMFPKS7DCGB2BXTWJA3CLZ4VJOU66".to_string()),
        status: "success".to_string(),
        ledger: Some(48213),
        block_time: Some(1741987200),
        arguments: vec![
            TypedValue {
                type_: "address".to_string(),
                value: serde_json::json!(
                    "GDDEVC3VPQVXMB6U4Q7ADJX4QN2XMFPKS7DCGB2BXTWJA3CLZ4VJOU66"
                ),
            },
            TypedValue {
                type_: "string".to_string(),
                value: serde_json::json!("MR-20941"),
            },
        ],
        return_value: TypedValue {
            type_: "u64".to_string(),
            value: serde_json::json!(441),
        },
        events: vec![TraceEventJson {
            sequence: 0,
            contract: "CDDEVC3VPQVXMB6U4Q7ADJX4QN2XMFPKS7DCGB2BXTWJA3CLZ4VJOU66".to_string(),
            name: "record_created".to_string(),
            version: 1,
            topics: vec!["record_created".to_string(), "medical_records".to_string()],
            body: serde_json::json!({
                "record_id": 441,
                "patient": "GDDEVC3VPQVXMB6U4Q7ADJX4QN2XMFPKS7DCGB2BXTWJA3CLZ4VJOU66",
                "doctor": "GD6LIKZEVGOWQGCP2Y5U7M4XGZGYZAROBB4SQF7UDNGUU7TTYHYUZMPQ",
                "category": "oncology",
                "is_confidential": true,
                "ledger": 48213
            }),
        }],
        result_xdr: None,
    };

    let registry_path = std::path::Path::new("../../schemas/events/event-schema-registry.json");
    let result = validate_trace_record(&record, Some(registry_path));
    assert!(result.is_ok(), "validation failed: {:?}", result.err());
}

#[test]
fn test_validate_trace_record_invalid_topic() {
    let record = ContractTraceJson {
        schema_version: "1.0.0".to_string(),
        trace_id: "7f1c2b9e4d8a3f0c6e5b7a9d1c3e5f7a9b1d3c5e7f9a1b3d5c7e9f0a2b4d6c8e".to_string(),
        contract: "CDDEVC3VPQVXMB6U4Q7ADJX4QN2XMFPKS7DCGB2BXTWJA3CLZ4VJOU66".to_string(),
        contract_name: "medical_records".to_string(),
        function: "create_record".to_string(),
        account: None,
        status: "success".to_string(),
        ledger: None,
        block_time: None,
        arguments: vec![],
        return_value: TypedValue {
            type_: "void".to_string(),
            value: serde_json::Value::Null,
        },
        events: vec![TraceEventJson {
            sequence: 0,
            contract: "CDDEVC3VPQVXMB6U4Q7ADJX4QN2XMFPKS7DCGB2BXTWJA3CLZ4VJOU66".to_string(),
            name: "record_created".to_string(),
            version: 1,
            topics: vec!["totally_bogus_topic".to_string()],
            body: serde_json::json!({}),
        }],
        result_xdr: None,
    };

    let registry_path = std::path::Path::new("../../schemas/events/event-schema-registry.json");
    let result = validate_trace_record(&record, Some(registry_path));
    assert!(result.is_err());
    let err_msg = result.err().unwrap().to_string();
    assert!(err_msg.contains("is not in the registered topics"));
}
