//! trace_extractor CLI binary.
//!
//! Decodes Soroban transaction metadata XDR into canonical NDJSON records conforming
//! to `schemas/trace/contract_trace.schema.json` and verifies event schema compliance.

use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process;

use xdr_trace::{decode_trace, validate_trace_record, TraceJsonOptions};

fn print_usage() {
    eprintln!(
        r#"Usage: trace_extractor [OPTIONS]

Options:
  --xdr <FILE>             Path to XDR file (raw binary, hex, or base64). Pass '-' for stdin.
  --hex <HEX>              Hex-encoded XDR string.
  --base64 <B64>           Base64-encoded XDR string.
  --contract-name <NAME>   Contract name (e.g., medical_records).
  --account <ACCOUNT>      Top-level caller account public key (G... address).
  --ledger <LEDGER>        Ledger sequence number.
  --block-time <TIMESTAMP> Unix timestamp of the block/transaction.
  --trace-id <HEX_64>      Correlation trace ID (64 hex characters).
  --status <STATUS>        Transaction status (success | failed).
  --registry <PATH>        Path to event-schema-registry.json.
  --no-validate-registry   Skip event schema registry check.
  --raw-xdr                Include base64/hex raw XDR in output.
  --pretty                 Pretty-print JSON output.
  -h, --help               Print this help message.
"#
    );
}

fn parse_xdr_bytes(input: &[u8]) -> Result<Vec<u8>, String> {
    if input.is_empty() {
        return Err("input XDR buffer is empty".to_string());
    }

    let trimmed = match std::str::from_utf8(input) {
        Ok(s) => s.trim(),
        Err(_) => return Ok(input.to_vec()),
    };

    // Try hex decoding first
    if trimmed.chars().all(|c| c.is_ascii_hexdigit())
        && trimmed.len() % 2 == 0
        && trimmed.len() >= 8
    {
        if let Ok(bytes) = hex::decode(trimmed) {
            return Ok(bytes);
        }
    }

    // Try base64 decoding
    // Standard base64 characters: A-Z, a-z, 0-9, +, /, =
    let is_b64 = trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=');
    if is_b64 && trimmed.len() >= 4 {
        use base64_compat::decode_base64;
        if let Ok(bytes) = decode_base64(trimmed) {
            return Ok(bytes);
        }
    }

    // Default to raw binary
    Ok(input.to_vec())
}

mod base64_compat {
    pub fn decode_base64(s: &str) -> Result<Vec<u8>, ()> {
        let mut table = [255u8; 256];
        for (i, &b) in b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
            .iter()
            .enumerate()
        {
            table[b as usize] = i as u8;
        }

        let mut out = Vec::with_capacity(s.len() * 3 / 4);
        let mut buf = 0u32;
        let mut bits = 0;

        for &b in s.as_bytes() {
            if b == b'=' || b.is_ascii_whitespace() {
                continue;
            }
            let val = table[b as usize];
            if val == 255 {
                return Err(());
            }
            buf = (buf << 6) | (val as u32);
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                out.push((buf >> bits) as u8);
            }
        }

        Ok(out)
    }
}

fn find_default_registry_path() -> Option<PathBuf> {
    let candidates = [
        "schemas/events/event-schema-registry.json",
        "../schemas/events/event-schema-registry.json",
        "../../schemas/events/event-schema-registry.json",
    ];
    for cand in candidates {
        let p = Path::new(cand);
        if p.exists() {
            return Some(p.to_path_buf());
        }
    }
    None
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_usage();
        process::exit(0);
    }

    let mut xdr_file: Option<String> = None;
    let mut hex_input: Option<String> = None;
    let mut base64_input: Option<String> = None;
    let mut contract_name: Option<String> = None;
    let mut account: Option<String> = None;
    let mut ledger: Option<u64> = None;
    let mut block_time: Option<u64> = None;
    let mut trace_id: Option<String> = None;
    let mut status: Option<String> = None;
    let mut registry_path: Option<PathBuf> = None;
    let mut no_validate_registry = false;
    let mut include_raw_xdr = false;
    let mut pretty = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--xdr" => {
                i += 1;
                if i < args.len() {
                    xdr_file = Some(args[i].clone());
                }
            },
            "--hex" => {
                i += 1;
                if i < args.len() {
                    hex_input = Some(args[i].clone());
                }
            },
            "--base64" => {
                i += 1;
                if i < args.len() {
                    base64_input = Some(args[i].clone());
                }
            },
            "--contract-name" => {
                i += 1;
                if i < args.len() {
                    contract_name = Some(args[i].clone());
                }
            },
            "--account" => {
                i += 1;
                if i < args.len() {
                    account = Some(args[i].clone());
                }
            },
            "--ledger" => {
                i += 1;
                if i < args.len() {
                    ledger = args[i].parse().ok();
                }
            },
            "--block-time" => {
                i += 1;
                if i < args.len() {
                    block_time = args[i].parse().ok();
                }
            },
            "--trace-id" => {
                i += 1;
                if i < args.len() {
                    trace_id = Some(args[i].clone());
                }
            },
            "--status" => {
                i += 1;
                if i < args.len() {
                    status = Some(args[i].clone());
                }
            },
            "--registry" => {
                i += 1;
                if i < args.len() {
                    registry_path = Some(PathBuf::from(&args[i]));
                }
            },
            "--no-validate-registry" => {
                no_validate_registry = true;
            },
            "--raw-xdr" => {
                include_raw_xdr = true;
            },
            "--pretty" => {
                pretty = true;
            },
            other => {
                // If positional argument and ends with .xdr or .hex
                if xdr_file.is_none() && (other.ends_with(".xdr") || other.ends_with(".hex")) {
                    xdr_file = Some(other.to_string());
                } else {
                    eprintln!("Unknown argument: {other}");
                    print_usage();
                    process::exit(1);
                }
            },
        }
        i += 1;
    }

    let raw_bytes = if let Some(hex_str) = hex_input {
        hex::decode(hex_str.trim()).unwrap_or_else(|e| {
            eprintln!("Error: failed to decode hex input: {e}");
            process::exit(1);
        })
    } else if let Some(b64_str) = base64_input {
        base64_compat::decode_base64(b64_str.trim()).unwrap_or_else(|_| {
            eprintln!("Error: failed to decode base64 input");
            process::exit(1);
        })
    } else if let Some(file_path) = xdr_file {
        if file_path == "-" {
            let mut buffer = Vec::new();
            io::stdin().read_to_end(&mut buffer).unwrap_or_else(|e| {
                eprintln!("Error: failed to read stdin: {e}");
                process::exit(1);
            });
            parse_xdr_bytes(&buffer).unwrap_or_else(|e| {
                eprintln!("Error: {e}");
                process::exit(1);
            })
        } else {
            let data = fs::read(&file_path).unwrap_or_else(|e| {
                eprintln!("Error: failed to read file '{file_path}': {e}");
                process::exit(1);
            });
            parse_xdr_bytes(&data).unwrap_or_else(|e| {
                eprintln!("Error: {e}");
                process::exit(1);
            })
        }
    } else {
        // Check if stdin has data
        let mut buffer = Vec::new();
        let bytes_read = io::stdin().read_to_end(&mut buffer).unwrap_or(0);
        if bytes_read > 0 {
            parse_xdr_bytes(&buffer).unwrap_or_else(|e| {
                eprintln!("Error: {e}");
                process::exit(1);
            })
        } else {
            eprintln!(
                "Error: no XDR input provided (use --xdr <FILE>, --hex <HEX>, or pipe to stdin)"
            );
            print_usage();
            process::exit(1);
        }
    };

    let trace = match decode_trace(&raw_bytes) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error: failed to decode trace XDR: {e}");
            process::exit(1);
        },
    };

    let options = TraceJsonOptions {
        contract_name,
        account,
        ledger,
        block_time,
        trace_id,
        status,
        raw_xdr: Some(raw_bytes),
        include_result_xdr: include_raw_xdr,
    };

    let record = trace.to_json_record(options);

    if !no_validate_registry {
        let reg_path = registry_path.or_else(find_default_registry_path);
        if let Err(e) = validate_trace_record(&record, reg_path.as_deref()) {
            eprintln!("Error: trace validation failed: {e}");
            process::exit(1);
        }
    }

    let json_output = if pretty {
        serde_json::to_string_pretty(&record)
    } else {
        serde_json::to_string(&record)
    };

    match json_output {
        Ok(out) => {
            println!("{out}");
        },
        Err(e) => {
            eprintln!("Error: JSON serialization error: {e}");
            process::exit(1);
        },
    }
}
