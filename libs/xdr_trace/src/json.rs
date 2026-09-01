//! Schema-compliant JSON and NDJSON serialization for contract traces.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map as JsonMap, Value};
use sha2::{Digest, Sha256};
use soroban_sdk::xdr::{
    ContractEvent, ContractEventBody, Hash, Int128Parts, Int256Parts, ScAddress, ScMapEntry, ScVal,
    TimePoint, UInt128Parts, UInt256Parts,
};

use crate::error::TraceError;
use crate::model::{ContractId, ContractTrace};

/// A typed value in `ContractTrace` JSON schema (`$defs.typedValue`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TypedValue {
    #[serde(rename = "type")]
    pub type_: String,
    pub value: Value,
}

/// A trace event in `ContractTrace` JSON schema (`$defs.traceEvent`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TraceEventJson {
    pub sequence: u32,
    pub contract: String,
    pub name: String,
    pub version: u32,
    pub topics: Vec<String>,
    pub body: Value,
}

/// The full NDJSON record representing a contract invocation trace.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContractTraceJson {
    pub schema_version: String,
    pub trace_id: String,
    pub contract: String,
    pub contract_name: String,
    pub function: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_time: Option<u64>,
    pub arguments: Vec<TypedValue>,
    pub return_value: TypedValue,
    pub events: Vec<TraceEventJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_xdr: Option<String>,
}

/// Options to customize JSON emission.
#[derive(Clone, Debug, Default)]
pub struct TraceJsonOptions {
    pub contract_name: Option<String>,
    pub account: Option<String>,
    pub ledger: Option<u64>,
    pub block_time: Option<u64>,
    pub trace_id: Option<String>,
    pub status: Option<String>,
    pub raw_xdr: Option<Vec<u8>>,
    pub include_result_xdr: bool,
}

/// Convert a `ContractId` to its canonical C-prefixed StrKey string.
pub fn contract_id_to_strkey(contract_id: &ContractId) -> String {
    stellar_strkey::Contract(contract_id.0).to_string()
}

/// Convert a `ScAddress` to its canonical StrKey string (`G...` or `C...`).
pub fn address_to_strkey(addr: &ScAddress) -> String {
    match addr {
        ScAddress::Account(account_id) => {
            let soroban_sdk::xdr::AccountId(soroban_sdk::xdr::PublicKey::PublicKeyTypeEd25519(
                soroban_sdk::xdr::Uint256(bytes),
            )) = account_id;
            stellar_strkey::ed25519::PublicKey(*bytes).to_string()
        },
        ScAddress::Contract(Hash(bytes)) => stellar_strkey::Contract(*bytes).to_string(),
    }
}

/// Convert an `ScVal` to its `TypedValue` representation (`{"type": "...", "value": ...}`).
pub fn scval_to_typed_value(val: &ScVal) -> TypedValue {
    match val {
        ScVal::Void => TypedValue {
            type_: "void".to_string(),
            value: Value::Null,
        },
        ScVal::Bool(b) => TypedValue {
            type_: "bool".to_string(),
            value: Value::Bool(*b),
        },
        ScVal::U32(v) => TypedValue {
            type_: "u32".to_string(),
            value: json!(*v),
        },
        ScVal::I32(v) => TypedValue {
            type_: "i32".to_string(),
            value: json!(*v),
        },
        ScVal::U64(v) => TypedValue {
            type_: "u64".to_string(),
            value: json!(*v),
        },
        ScVal::I64(v) => TypedValue {
            type_: "i64".to_string(),
            value: json!(*v),
        },
        ScVal::Timepoint(TimePoint(v)) => TypedValue {
            type_: "u64".to_string(),
            value: json!(*v),
        },
        ScVal::Duration(soroban_sdk::xdr::Duration(v)) => TypedValue {
            type_: "u64".to_string(),
            value: json!(*v),
        },
        ScVal::U128(UInt128Parts { hi, lo }) => {
            let val = ((*hi as u128) << 64) | (*lo as u128);
            TypedValue {
                type_: "u128".to_string(),
                value: Value::String(val.to_string()),
            }
        },
        ScVal::I128(Int128Parts { hi, lo }) => {
            let val = ((*hi as i128) << 64) | (*lo as i128);
            TypedValue {
                type_: "i128".to_string(),
                value: Value::String(val.to_string()),
            }
        },
        ScVal::U256(UInt256Parts {
            hi_hi,
            hi_lo,
            lo_hi,
            lo_lo,
        }) => {
            let hex_str = format!("{hi_hi:016x}{hi_lo:016x}{lo_hi:016x}{lo_lo:016x}");
            TypedValue {
                type_: "u256".to_string(),
                value: Value::String(hex_str),
            }
        },
        ScVal::I256(Int256Parts {
            hi_hi,
            hi_lo,
            lo_hi,
            lo_lo,
        }) => {
            let hex_str = format!("{hi_hi:016x}{hi_lo:016x}{lo_hi:016x}{lo_lo:016x}");
            TypedValue {
                type_: "i256".to_string(),
                value: Value::String(hex_str),
            }
        },
        ScVal::Bytes(b) => TypedValue {
            type_: "bytes".to_string(),
            value: Value::String(hex::encode(b.0.as_vec())),
        },
        ScVal::String(s) => TypedValue {
            type_: "string".to_string(),
            value: Value::String(String::from_utf8_lossy(s.0.as_vec()).to_string()),
        },
        ScVal::Symbol(s) => TypedValue {
            type_: "symbol".to_string(),
            value: Value::String(String::from_utf8_lossy(s.0.as_vec()).to_string()),
        },
        ScVal::Address(addr) => TypedValue {
            type_: "address".to_string(),
            value: Value::String(address_to_strkey(addr)),
        },
        ScVal::Vec(Some(v)) => {
            let items: Vec<Value> = v.0.as_vec().iter().map(scval_to_native_json).collect();
            TypedValue {
                type_: "vec".to_string(),
                value: Value::Array(items),
            }
        },
        ScVal::Vec(None) => TypedValue {
            type_: "vec".to_string(),
            value: Value::Array(vec![]),
        },
        ScVal::Map(Some(m)) => {
            let map = scval_map_to_json(m.0.as_vec());
            TypedValue {
                type_: "map".to_string(),
                value: Value::Object(map),
            }
        },
        ScVal::Map(None) => TypedValue {
            type_: "map".to_string(),
            value: Value::Object(JsonMap::new()),
        },
        ScVal::ContractInstance(_) => TypedValue {
            type_: "contract_instance".to_string(),
            value: Value::Null,
        },
        ScVal::LedgerKeyContractInstance => TypedValue {
            type_: "ledger_key_contract_instance".to_string(),
            value: Value::Null,
        },
        ScVal::LedgerKeyNonce(n) => TypedValue {
            type_: "ledger_key_nonce".to_string(),
            value: json!(n.nonce),
        },
        ScVal::Error(err) => TypedValue {
            type_: "error".to_string(),
            value: Value::String(format!("{err:?}")),
        },
    }
}

/// Convert an `ScVal` to its native JSON value (used for event bodies, arguments, and vector elements).
pub fn scval_to_native_json(val: &ScVal) -> Value {
    match val {
        ScVal::Void => Value::Null,
        ScVal::Bool(b) => Value::Bool(*b),
        ScVal::U32(v) => json!(*v),
        ScVal::I32(v) => json!(*v),
        ScVal::U64(v) => json!(*v),
        ScVal::I64(v) => json!(*v),
        ScVal::Timepoint(TimePoint(v)) => json!(*v),
        ScVal::Duration(soroban_sdk::xdr::Duration(v)) => json!(*v),
        ScVal::U128(UInt128Parts { hi, lo }) => {
            let val = ((*hi as u128) << 64) | (*lo as u128);
            Value::String(val.to_string())
        },
        ScVal::I128(Int128Parts { hi, lo }) => {
            let val = ((*hi as i128) << 64) | (*lo as i128);
            Value::String(val.to_string())
        },
        ScVal::U256(UInt256Parts {
            hi_hi,
            hi_lo,
            lo_hi,
            lo_lo,
        }) => {
            let hex_str = format!("{hi_hi:016x}{hi_lo:016x}{lo_hi:016x}{lo_lo:016x}");
            Value::String(hex_str)
        },
        ScVal::I256(Int256Parts {
            hi_hi,
            hi_lo,
            lo_hi,
            lo_lo,
        }) => {
            let hex_str = format!("{hi_hi:016x}{hi_lo:016x}{lo_hi:016x}{lo_lo:016x}");
            Value::String(hex_str)
        },
        ScVal::Bytes(b) => Value::String(hex::encode(b.0.as_vec())),
        ScVal::String(s) => Value::String(String::from_utf8_lossy(s.0.as_vec()).to_string()),
        ScVal::Symbol(s) => Value::String(String::from_utf8_lossy(s.0.as_vec()).to_string()),
        ScVal::Address(addr) => Value::String(address_to_strkey(addr)),
        ScVal::Vec(Some(v)) => {
            Value::Array(v.0.as_vec().iter().map(scval_to_native_json).collect())
        },
        ScVal::Vec(None) => Value::Array(vec![]),
        ScVal::Map(Some(m)) => Value::Object(scval_map_to_json(m.0.as_vec())),
        ScVal::Map(None) => Value::Object(JsonMap::new()),
        ScVal::ContractInstance(_) => Value::Null,
        ScVal::LedgerKeyContractInstance => Value::Null,
        ScVal::LedgerKeyNonce(n) => json!(n.nonce),
        ScVal::Error(err) => Value::String(format!("{err:?}")),
    }
}

/// Convert a list of `ScMapEntry` to a `serde_json::Map`.
fn scval_map_to_json(entries: &[ScMapEntry]) -> JsonMap<String, Value> {
    let mut map = JsonMap::new();
    for entry in entries {
        let key_str = match &entry.key {
            ScVal::Symbol(s) => String::from_utf8_lossy(s.0.as_vec()).to_string(),
            ScVal::String(s) => String::from_utf8_lossy(s.0.as_vec()).to_string(),
            ScVal::Bytes(b) => hex::encode(b.0.as_vec()),
            other => scval_to_native_json(other).to_string(),
        };
        map.insert(key_str, scval_to_native_json(&entry.val));
    }
    map
}

/// Convert an `ScVal` in an event topic to its string topic identifier.
fn scval_to_topic_string(val: &ScVal) -> String {
    match val {
        ScVal::Symbol(s) => String::from_utf8_lossy(s.0.as_vec()).to_string(),
        ScVal::String(s) => String::from_utf8_lossy(s.0.as_vec()).to_string(),
        ScVal::Bytes(b) => hex::encode(b.0.as_vec()),
        ScVal::Address(addr) => address_to_strkey(addr),
        ScVal::U32(v) => v.to_string(),
        ScVal::I32(v) => v.to_string(),
        ScVal::U64(v) => v.to_string(),
        ScVal::I64(v) => v.to_string(),
        _ => "topic".to_string(),
    }
}

/// Format contract events into canonical `TraceEventJson` records.
pub fn format_events(events: &[ContractEvent], root_contract_strkey: &str) -> Vec<TraceEventJson> {
    events
        .iter()
        .enumerate()
        .map(|(sequence, ev)| {
            let contract_str = ev
                .contract_id
                .as_ref()
                .map(|Hash(h)| stellar_strkey::Contract(*h).to_string())
                .unwrap_or_else(|| root_contract_strkey.to_string());

            let ContractEventBody::V0(v0) = &ev.body;
            let topics: Vec<String> = v0
                .topics
                .as_vec()
                .iter()
                .map(scval_to_topic_string)
                .collect();

            let name = topics
                .first()
                .cloned()
                .unwrap_or_else(|| "event".to_string());
            let body_val = scval_to_native_json(&v0.data);
            let body = match body_val {
                Value::Object(_) => body_val,
                Value::Null => json!({}),
                other => json!({ "value": other }),
            };

            TraceEventJson {
                sequence: sequence as u32,
                contract: contract_str,
                name,
                version: 1,
                topics,
                body,
            }
        })
        .collect()
}

/// Derive a deterministic 64-hex trace ID.
pub fn derive_trace_id(
    account: Option<&str>,
    ledger: Option<u64>,
    raw_xdr: Option<&[u8]>,
    contract_id: &ContractId,
    function_name: &str,
) -> String {
    let mut hasher = Sha256::new();
    if let (Some(acc), Some(led)) = (account, ledger) {
        hasher.update(acc.as_bytes());
        hasher.update(led.to_be_bytes());
        hasher.update(contract_id.0);
        hasher.update(function_name.as_bytes());
    } else if let Some(xdr) = raw_xdr {
        hasher.update(xdr);
    } else {
        hasher.update(contract_id.0);
        hasher.update(function_name.as_bytes());
    }
    hex::encode(hasher.finalize())
}

impl ContractTrace {
    /// Convert this `ContractTrace` into a canonical `ContractTraceJson` record.
    pub fn to_json_record(&self, options: TraceJsonOptions) -> ContractTraceJson {
        let contract_str = contract_id_to_strkey(&self.contract_id);
        let contract_name = options
            .contract_name
            .unwrap_or_else(|| "unknown_contract".to_string());

        let trace_id = options.trace_id.unwrap_or_else(|| {
            derive_trace_id(
                options.account.as_deref(),
                options.ledger,
                options.raw_xdr.as_deref(),
                &self.contract_id,
                &self.function_name,
            )
        });

        let status = options.status.unwrap_or_else(|| {
            if matches!(self.return_value, ScVal::Error(_)) {
                "failed".to_string()
            } else {
                "success".to_string()
            }
        });

        let arguments: Vec<TypedValue> = self.arguments.iter().map(scval_to_typed_value).collect();
        let return_value = scval_to_typed_value(&self.return_value);
        let events = format_events(&self.events, &contract_str);

        let result_xdr = if options.include_result_xdr {
            options.raw_xdr.as_ref().map(hex::encode)
        } else {
            None
        };

        ContractTraceJson {
            schema_version: "1.0.0".to_string(),
            trace_id,
            contract: contract_str,
            contract_name,
            function: self.function_name.clone(),
            account: options.account,
            status,
            ledger: options.ledger,
            block_time: options.block_time,
            arguments,
            return_value,
            events,
            result_xdr,
        }
    }

    /// Serialize this `ContractTrace` as a single-line NDJSON string.
    pub fn to_ndjson_line(&self, options: TraceJsonOptions) -> Result<String, TraceError> {
        let record = self.to_json_record(options);
        serde_json::to_string(&record).map_err(|e| TraceError::Serialization(e.to_string()))
    }
}
