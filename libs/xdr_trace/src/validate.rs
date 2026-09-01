//! Validation logic for contract traces and event registry conformance.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value;

use crate::error::TraceError;
use crate::json::ContractTraceJson;

#[derive(Debug, Deserialize)]
struct RegistryFile {
    #[serde(default)]
    events: HashMap<String, RegistryEventEntry>,
}

#[derive(Debug, Deserialize)]
struct RegistryEventEntry {
    #[serde(default)]
    topics: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    schema: Option<Value>,
}

/// Validate a `ContractTraceJson` record against schema rules and event registry.
pub fn validate_trace_record(
    record: &ContractTraceJson,
    registry_path: Option<&Path>,
) -> Result<(), TraceError> {
    if record.schema_version != "1.0.0" {
        return Err(TraceError::Serialization(format!(
            "unsupported schema_version '{}', expected '1.0.0'",
            record.schema_version
        )));
    }

    if record.trace_id.len() != 64 || hex::decode(&record.trace_id).is_err() {
        return Err(TraceError::Serialization(format!(
            "invalid trace_id '{}': expected 64-char hex string",
            record.trace_id
        )));
    }

    if record.contract.is_empty() {
        return Err(TraceError::Serialization(
            "contract identifier cannot be empty".to_string(),
        ));
    }

    if record.status != "success" && record.status != "failed" {
        return Err(TraceError::Serialization(format!(
            "invalid status '{}', expected 'success' or 'failed'",
            record.status
        )));
    }

    if let Some(path) = registry_path {
        if path.exists() {
            let data = fs::read_to_string(path)
                .map_err(|e| TraceError::Io(format!("failed to read registry file: {e}")))?;
            let registry: RegistryFile = serde_json::from_str(&data).map_err(|e| {
                TraceError::Serialization(format!("failed to parse registry JSON: {e}"))
            })?;

            for event in &record.events {
                let key = format!("{}.{}", record.contract_name, event.name);
                let entry = registry
                    .events
                    .get(&key)
                    .ok_or_else(|| TraceError::UnknownEvent {
                        contract: record.contract_name.clone(),
                        event: event.name.clone(),
                    })?;

                for topic in &event.topics {
                    if !entry.topics.contains(topic) {
                        return Err(TraceError::InvalidEventTopic {
                            contract: record.contract_name.clone(),
                            event: event.name.clone(),
                            topic: topic.clone(),
                        });
                    }
                }
            }
        }
    }

    Ok(())
}
