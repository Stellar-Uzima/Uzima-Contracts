//! FHIR R4 Consent resource mapping for patient_consent_management.
//!
//! Maps the internal `ConsentRecord` structure to a FHIR R4 `Consent` resource
//! JSON representation.  The output is a deterministic JSON string that can be
//! validated against the FHIR R4 Consent schema (hl7.org/fhir/R4/consent.html).
//!
//! ## FHIR R4 Consent resource mapping
//!
//! | ConsentRecord field | FHIR R4 Consent field              | Notes                              |
//! |---------------------|------------------------------------|------------------------------------|
//! | `patient`           | `patient.reference`                | Format: `Address/<addr>`           |
//! | `provider`          | `provision.actor[0].reference`     | The grantee / performer            |
//! | `granted_at`        | `dateTime`                         | ISO-8601 UTC                       |
//! | `expires_at`        | `provision.period.end`             | Omitted when 0 (no expiry)         |
//! | `revoked_at`        | used to infer `status`             | > 0 → "inactive"                   |
//! | `active`            | `status`                           | "active" or "inactive"             |

use crate::ConsentRecord;
use soroban_sdk::{Bytes, BytesN, Env, String};

// ── FHIR R4 constants ───────────────────────────────────────────────────────

const SCOPE_SYSTEM: &str = "http://terminology.hl7.org/CodeSystem/consentscope";
const SCOPE_CODE: &str = "patient-privacy";
const SCOPE_DISPLAY: &str = "Privacy Consent";

const CATEGORY_SYSTEM: &str = "http://loinc.org";
const CATEGORY_CODE: &str = "59284-0";
const CATEGORY_DISPLAY: &str = "Consent";

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Writes a zero-padded u64 directly to `buf`, digit by digit (safe, no
/// intermediate references).  `width` is the minimum number of chars.
fn write_u64_padded(env: &Env, buf: &mut Bytes, n: u64, width: usize) {
    // Build digits buffer on the stack, track actual length.
    let mut digits: [u8; 20] = [0u8; 20];
    let dlen = u64_fill_digits(n, &mut digits);

    // Leading zeros
    let pad = width.saturating_sub(dlen);
    for _ in 0..pad {
        buf.append(&Bytes::from_slice(env, b"0"));
    }
    // Actual digits
    buf.append(&Bytes::from_slice(env, &digits[..dlen]));
}

/// Fills `out[..]` with the decimal digits of `n` (most-significant first)
/// and returns the number of digits written.  Safe — no references returned.
fn u64_fill_digits(n: u64, out: &mut [u8; 20]) -> usize {
    if n == 0 {
        out[0] = b'0';
        return 1;
    }
    let mut v = n;
    let mut idx: usize = 20;
    while v > 0 {
        idx -= 1;
        out[idx] = (v % 10) as u8 + b'0';
        v /= 10;
    }
    let len = 20 - idx;
    // Shift to start of array
    for i in 0..len {
        out[i] = out[idx + i];
    }
    len
}

/// Serializes a `u64` ledger timestamp into an ISO-8601 UTC datetime string.
fn timestamp_to_iso8601(env: &Env, ts: u64, buf: &mut Bytes) {
    let mut remaining = ts;
    let secs = remaining % 86400;
    remaining /= 86400;

    let hour = secs / 3600;
    let minute = (secs % 3600) / 60;
    let second = secs % 60;

    let mut days = remaining;
    let mut year: u64 = 1970;
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }

    let leap = is_leap_year(year);
    let month_days: [u64; 12] = [
        31u64,
        if leap { 29u64 } else { 28u64 },
        31u64,
        30u64,
        31u64,
        30u64,
        31u64,
        31u64,
        30u64,
        31u64,
        30u64,
        31u64,
    ];
    let mut month: u64 = 1;
    for &md in &month_days {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }
    let day = days + 1;

    write_u64_padded(env, buf, year, 4);
    buf.append(&Bytes::from_slice(env, b"-"));
    write_u64_padded(env, buf, month, 2);
    buf.append(&Bytes::from_slice(env, b"-"));
    write_u64_padded(env, buf, day, 2);
    buf.append(&Bytes::from_slice(env, b"T"));
    write_u64_padded(env, buf, hour, 2);
    buf.append(&Bytes::from_slice(env, b":"));
    write_u64_padded(env, buf, minute, 2);
    buf.append(&Bytes::from_slice(env, b":"));
    write_u64_padded(env, buf, second, 2);
    buf.append(&Bytes::from_slice(env, b"Z"));
}

fn is_leap_year(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

/// Appends an escaped JSON string value to the buffer.
/// Takes a `soroban_sdk::String` and escapes special chars.
fn json_escape_string(env: &Env, s: &String, buf: &mut Bytes) {
    let s_bytes = s.to_buffer::<256>();
    let slice = s_bytes.as_slice();
    for i in 0..s_bytes.len() {
        let b = slice[i as usize];
        match b {
            b'"' => buf.append(&Bytes::from_slice(env, b"\\\"")),
            b'\\' => buf.append(&Bytes::from_slice(env, b"\\\\")),
            b'\n' => buf.append(&Bytes::from_slice(env, b"\\n")),
            b'\r' => buf.append(&Bytes::from_slice(env, b"\\r")),
            b'\t' => buf.append(&Bytes::from_slice(env, b"\\t")),
            other => buf.append(&Bytes::from_slice(env, &[other])),
        }
    }
}

// ── Public API ──────────────────────────────────────────────────────────────

/// Converts a `ConsentRecord` into a **FHIR R4 Consent** resource JSON string.
///
/// # FHIR R4 Consent resource fields populated
///
/// | FHIR field                   | Source / Value                               |
/// |------------------------------|----------------------------------------------|
/// | `resourceType`               | `"Consent"`                                  |
/// | `id`                         | `<patient>:<provider>:<granted_at>` hash     |
/// | `status`                     | `"active"` or `"inactive"`                   |
/// | `scope`                      | patient-privacy (LOINC 59284-0)              |
/// | `category`                   | Consent (LOINC 59284-0)                      |
/// | `patient.reference`          | `"Address/<patient>"`                        |
/// | `dateTime`                   | `granted_at` as ISO-8601                     |
/// | `provision.type`             | `"deny"` when inactive, otherwise `"permit"` |
/// | `provision.period.start`     | `granted_at` as ISO-8601                     |
/// | `provision.period.end`       | `expires_at` when > 0                        |
/// | `provision.actor[0].role`    | `{coding: [{system: ..., code: "IRCP"}]}`    |
/// | `provision.actor[0].reference` | `"Address/<provider>"`                    |
///
/// # Parameters
/// * `env`      – Soroban environment (for string/buffer allocation).
/// * `record`   – The on-chain consent record to serialize.
///
/// # Returns
/// A valid FHIR R4 JSON `String` representing the Consent resource.
pub fn to_fhir_json(env: &Env, record: &ConsentRecord) -> String {
    let mut buf = Bytes::new(env);

    // ── Open object ─────────────────────────────────────────────────────
    buf.append(&Bytes::from_slice(env, b"{"));

    // resourceType
    buf.append(&Bytes::from_slice(
        env,
        b"\"resourceType\":\"Consent\",",
    ));

    // id
    let id = generate_consent_id(env, record);
    buf.append(&Bytes::from_slice(env, b"\"id\":\""));
    json_escape_string(env, &id, &mut buf);
    buf.append(&Bytes::from_slice(env, b"\","));

    // status
    let status = if record.active { "active" } else { "inactive" };
    buf.append(&Bytes::from_slice(env, b"\"status\":\""));
    buf.append(&Bytes::from_slice(env, status.as_bytes()));
    buf.append(&Bytes::from_slice(env, b"\","));

    // scope
    buf.append(&Bytes::from_slice(env, b"\"scope\":{"));
    buf.append(&Bytes::from_slice(env, b"\"coding\":[{"));
    buf.append(&Bytes::from_slice(env, b"\"system\":\""));
    buf.append(&Bytes::from_slice(env, SCOPE_SYSTEM.as_bytes()));
    buf.append(&Bytes::from_slice(env, b"\",\"code\":\""));
    buf.append(&Bytes::from_slice(env, SCOPE_CODE.as_bytes()));
    buf.append(&Bytes::from_slice(env, b"\",\"display\":\""));
    buf.append(&Bytes::from_slice(env, SCOPE_DISPLAY.as_bytes()));
    buf.append(&Bytes::from_slice(env, b"\"}]},"));

    // category
    buf.append(&Bytes::from_slice(env, b"\"category\":[{"));
    buf.append(&Bytes::from_slice(env, b"\"coding\":[{"));
    buf.append(&Bytes::from_slice(env, b"\"system\":\""));
    buf.append(&Bytes::from_slice(env, CATEGORY_SYSTEM.as_bytes()));
    buf.append(&Bytes::from_slice(env, b"\",\"code\":\""));
    buf.append(&Bytes::from_slice(env, CATEGORY_CODE.as_bytes()));
    buf.append(&Bytes::from_slice(env, b"\",\"display\":\""));
    buf.append(&Bytes::from_slice(env, CATEGORY_DISPLAY.as_bytes()));
    buf.append(&Bytes::from_slice(env, b"\"}]}],"));

    // patient
    buf.append(&Bytes::from_slice(env, b"\"patient\":{\"reference\":\"Address/"));
    let patient_str = record.patient.to_string();
    json_escape_string(env, &patient_str, &mut buf);
    buf.append(&Bytes::from_slice(env, b"\"},"));

    // dateTime
    buf.append(&Bytes::from_slice(env, b"\"dateTime\":\""));
    timestamp_to_iso8601(env, record.granted_at, &mut buf);
    buf.append(&Bytes::from_slice(env, b"\","));

    // provision
    let provision_type = if record.active { "permit" } else { "deny" };
    buf.append(&Bytes::from_slice(env, b"\"provision\":{"));
    buf.append(&Bytes::from_slice(env, b"\"type\":\""));
    buf.append(&Bytes::from_slice(env, provision_type.as_bytes()));
    buf.append(&Bytes::from_slice(env, b"\","));

    // provision.period
    buf.append(&Bytes::from_slice(env, b"\"period\":{"));
    buf.append(&Bytes::from_slice(env, b"\"start\":\""));
    timestamp_to_iso8601(env, record.granted_at, &mut buf);
    buf.append(&Bytes::from_slice(env, b"\""));
    if record.expires_at > 0 {
        buf.append(&Bytes::from_slice(env, b",\"end\":\""));
        timestamp_to_iso8601(env, record.expires_at, &mut buf);
        buf.append(&Bytes::from_slice(env, b"\""));
    }
    buf.append(&Bytes::from_slice(env, b"},"));

    // provision.actor
    buf.append(&Bytes::from_slice(env, b"\"actor\":[{"));
    buf.append(&Bytes::from_slice(env, b"\"role\":{\"coding\":[{"));
    buf.append(&Bytes::from_slice(
        env,
        b"\"system\":\"http://terminology.hl7.org/CodeSystem/v3-ParticipationType\",",
    ));
    buf.append(&Bytes::from_slice(env, b"\"code\":\"IRCP\"}]},"));
    buf.append(&Bytes::from_slice(env, b"\"reference\":\"Address/"));
    let provider_str = record.provider.to_string();
    json_escape_string(env, &provider_str, &mut buf);
    buf.append(&Bytes::from_slice(env, b"\""));

    // provision.action
    buf.append(&Bytes::from_slice(env, b"}],\"action\":[{"));
    buf.append(&Bytes::from_slice(env, b"\"coding\":[{"));
    buf.append(&Bytes::from_slice(
        env,
        b"\"system\":\"http://terminology.hl7.org/CodeSystem/consentaction\",",
    ));
    buf.append(&Bytes::from_slice(env, b"\"code\":\"access\"}]}]}"));

    // Close provision and root object
    buf.append(&Bytes::from_slice(env, b"}"));

    // ── Close object ─────────────────────────────────────────────────────
    buf.append(&Bytes::from_slice(env, b"}"));

    let result_buf = buf.to_buffer::<8192>();
    String::from_bytes(env, result_buf.as_slice())
}

/// Generates a deterministic FHIR resource ID from the consent record.
/// The ID is the first 16 hex characters of the SHA-256 hash.
fn generate_consent_id(env: &Env, record: &ConsentRecord) -> String {
    let mut payload = Bytes::new(env);

    let patient_s = record.patient.to_string();
    let patient_bytes = patient_s.to_buffer::<256>();
    payload.append(&Bytes::from_slice(env, patient_bytes.as_slice()));
    payload.append(&Bytes::from_slice(env, b":"));

    let provider_s = record.provider.to_string();
    let provider_bytes = provider_s.to_buffer::<256>();
    payload.append(&Bytes::from_slice(env, provider_bytes.as_slice()));
    payload.append(&Bytes::from_slice(env, b":"));

    payload.append(&Bytes::from_slice(env, &record.granted_at.to_be_bytes()));
    payload.append(&Bytes::from_slice(env, b":"));
    payload.append(&Bytes::from_slice(env, &record.expires_at.to_be_bytes()));

    let hash_bytes: BytesN<32> = env.crypto().sha256(&payload).into();
    bytes_to_hex_prefix(env, &hash_bytes)
}

/// Converts the first 8 bytes of a 32-byte hash to 16 hex characters.
fn bytes_to_hex_prefix(env: &Env, bytes: &BytesN<32>) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut buf = Bytes::new(env);
    let raw = bytes.as_array();
    for &b in raw.iter().take(8) {
        buf.append(&Bytes::from_slice(env, &[HEX[(b >> 4) as usize]]));
        buf.append(&Bytes::from_slice(env, &[HEX[(b & 0x0F) as usize]]));
    }
    let result_buf = buf.to_buffer::<32>();
    String::from_bytes(env, result_buf.as_slice())
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ConsentRecord;
    use soroban_sdk::testutils::{Address as _, Ledger as _};
    use soroban_sdk::{Address, Env};

    fn make_record(env: &Env, active: bool, expires_at: u64, revoked_at: u64) -> ConsentRecord {
        ConsentRecord {
            patient: Address::generate(env),
            provider: Address::generate(env),
            granted_at: 1_700_000_000, // 2023-11-14T22:13:20Z
            expires_at,
            revoked_at,
            active,
        }
    }

    /// Checks whether the JSON string contains the given substring.
    fn json_contains(json: &String, needle: &str) -> bool {
        let buf = json.to_buffer::<8192>();
        let bytes = buf.as_slice();
        let s = core::str::from_utf8(bytes).unwrap_or("");
        s.contains(needle)
    }

    #[test]
    fn test_active_consent_fhir_json_contains_required_fields() {
        let env = Env::default();
        let record = make_record(&env, true, 0, 0);
        let json = to_fhir_json(&env, &record);

        assert!(json_contains(&json, "Consent"));
        assert!(json_contains(&json, "\"active\""));
        assert!(json_contains(&json, "Address/"));
        assert!(json_contains(&json, "\"permit\""));
        assert!(json_contains(&json, "patient-privacy"));
    }

    #[test]
    fn test_inactive_revoked_consent_fhir_json() {
        let env = Env::default();
        let record = make_record(&env, false, 0, 1_700_500_000);
        let json = to_fhir_json(&env, &record);

        assert!(json_contains(&json, "\"inactive\""));
        assert!(json_contains(&json, "\"deny\""));
    }

    #[test]
    fn test_consent_with_expiry_has_period_end() {
        let env = Env::default();
        let expiry = 1_710_000_000;
        let record = make_record(&env, true, expiry, 0);
        let json = to_fhir_json(&env, &record);

        assert!(json_contains(&json, "\"end\""));
    }

    #[test]
    fn test_consent_without_expiry_has_no_period_end() {
        let env = Env::default();
        let record = make_record(&env, true, 0, 0);
        let json = to_fhir_json(&env, &record);

        // When expires_at == 0, no "end" field should appear anywhere in the JSON.
        assert!(!json_contains(&json, "\"end\""));
    }

    #[test]
    fn test_fhir_json_has_valid_id() {
        let env = Env::default();
        let record = make_record(&env, true, 0, 0);
        let json = to_fhir_json(&env, &record);

        // Verify id field is present with a 16-char hex value.
        let buf = json.to_buffer::<8192>();
        let s = core::str::from_utf8(buf.as_slice()).unwrap_or("");
        assert!(s.contains("\"id\":\""));
        // Extract 16 chars after "id":"
        let id_start = s.find("\"id\":\"").unwrap() + 6;
        let id_end = id_start + 16;
        let id = &s[id_start..id_end];
        assert_eq!(id.len(), 16);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_fhir_json_starts_and_ends_with_braces() {
        let env = Env::default();
        let record = make_record(&env, true, 0, 0);
        let json = to_fhir_json(&env, &record);

        let buf = json.to_buffer::<8192>();
        let s = core::str::from_utf8(buf.as_slice()).unwrap_or("");
        assert!(s.starts_with('{'));
        assert!(s.ends_with('}'));
    }

    #[test]
    fn test_fhir_json_contains_date_time() {
        let env = Env::default();
        let record = make_record(&env, true, 0, 0);
        let json = to_fhir_json(&env, &record);

        assert!(json_contains(&json, "\"dateTime\":\""));
        assert!(json_contains(&json, "T"));
        assert!(json_contains(&json, "Z\""));
    }

    #[test]
    fn test_fhir_json_contains_actor_reference() {
        let env = Env::default();
        let record = make_record(&env, true, 0, 0);
        let json = to_fhir_json(&env, &record);

        assert!(json_contains(&json, "Address/"));
        assert!(json_contains(&json, "IRCP"));
    }

    #[test]
    fn test_different_records_produce_different_ids() {
        let env = Env::default();
        let record1 = make_record(&env, true, 0, 0);
        let record2 = {
            let mut r = make_record(&env, true, 0, 0);
            r.provider = Address::generate(&env);
            r
        };

        let json1 = to_fhir_json(&env, &record1);
        let json2 = to_fhir_json(&env, &record2);

        let buf1 = json1.to_buffer::<8192>();
        let buf2 = json2.to_buffer::<8192>();
        let s1 = core::str::from_utf8(buf1.as_slice()).unwrap_or("");
        let s2 = core::str::from_utf8(buf2.as_slice()).unwrap_or("");

        let id1_start = s1.find("\"id\":\"").unwrap() + 6;
        let id2_start = s2.find("\"id\":\"").unwrap() + 6;
        let id1 = &s1[id1_start..id1_start + 16];
        let id2 = &s2[id2_start..id2_start + 16];
        assert_ne!(id1, id2);
    }
}
