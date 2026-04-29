#![no_std]

use soroban_sdk::{Env, String};

// Maximum byte length the internal buffer can hold. All MAX_*_LEN constants
// must be <= this value, enforced by the runtime check in sanitize_string.
const BUFFER_SIZE: usize = 1024;

pub const MAX_NAME_LEN: u32 = 128;
pub const MAX_EMAIL_LEN: u32 = 254; // RFC 5321
pub const MAX_ID_LEN: u32 = 64;
pub const MAX_URL_LEN: u32 = 512;
pub const MAX_BIO_LEN: u32 = 1024;
pub const MAX_GENERAL_LEN: u32 = 256;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum SanitizationError {
    InputTooLong = 1,
    EmptyInput = 2,
    NullByte = 3,
    InvalidCharacter = 4,
    InvalidFormat = 5,
}

/// Validates a general-purpose string: non-empty, within `max_len` bytes,
/// no null bytes, no ASCII control characters (allows tab/LF/CR).
pub fn sanitize_string(_env: &Env, input: &String, max_len: u32) -> Result<(), SanitizationError> {
    let len = input.len();

    if len == 0 {
        return Err(SanitizationError::EmptyInput);
    }
    // Guard: reject inputs that would overflow the static buffer.
    if len > BUFFER_SIZE as u32 || len > max_len {
        return Err(SanitizationError::InputTooLong);
    }

    let len_usize = len as usize;
    let mut buf = [0u8; BUFFER_SIZE];
    input.copy_into_slice(&mut buf[..len_usize]);

    for i in 0..len_usize {
        let b = buf[i];
        if b == 0x00 {
            return Err(SanitizationError::NullByte);
        }
        // Block C0 control chars except HT(0x09), LF(0x0A), CR(0x0D).
        if b < 0x20 && b != 0x09 && b != 0x0A && b != 0x0D {
            return Err(SanitizationError::InvalidCharacter);
        }
        // DEL (0x7F) is also a control character.
        if b == 0x7F {
            return Err(SanitizationError::InvalidCharacter);
        }
    }

    Ok(())
}

/// Validates a human name: letters (any UTF-8), digits, spaces, hyphens,
/// apostrophes, commas, and periods only (ASCII subset).
pub fn sanitize_name(_env: &Env, input: &String) -> Result<(), SanitizationError> {
    let len = input.len();

    if len == 0 {
        return Err(SanitizationError::EmptyInput);
    }
    if len > BUFFER_SIZE as u32 || len > MAX_NAME_LEN {
        return Err(SanitizationError::InputTooLong);
    }

    let len_usize = len as usize;
    let mut buf = [0u8; BUFFER_SIZE];
    input.copy_into_slice(&mut buf[..len_usize]);

    for i in 0..len_usize {
        let b = buf[i];
        if b == 0x00 {
            return Err(SanitizationError::NullByte);
        }
        // Allow printable ASCII name chars and UTF-8 multi-byte continuations.
        let ascii_name_char = matches!(b,
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9'
            | b' ' | b'-' | b'\'' | b',' | b'.' | b'_'
        );
        let utf8_continuation = b >= 0x80;
        if !ascii_name_char && !utf8_continuation {
            return Err(SanitizationError::InvalidCharacter);
        }
    }

    Ok(())
}

/// Validates an email address: single '@', non-empty local and domain parts,
/// domain contains at least one '.', all chars from the RFC 5321 allowed set.
pub fn sanitize_email(_env: &Env, input: &String) -> Result<(), SanitizationError> {
    let len = input.len();

    if len == 0 {
        return Err(SanitizationError::EmptyInput);
    }
    if len > BUFFER_SIZE as u32 || len > MAX_EMAIL_LEN {
        return Err(SanitizationError::InputTooLong);
    }

    let len_usize = len as usize;
    let mut buf = [0u8; BUFFER_SIZE];
    input.copy_into_slice(&mut buf[..len_usize]);

    let mut at_count: u32 = 0;
    let mut at_pos: usize = 0;

    for i in 0..len_usize {
        let b = buf[i];
        if b == 0x00 {
            return Err(SanitizationError::NullByte);
        }
        if b == b'@' {
            at_count += 1;
            at_pos = i;
        }
        if !is_email_char(b) {
            return Err(SanitizationError::InvalidCharacter);
        }
    }

    if at_count != 1 {
        return Err(SanitizationError::InvalidFormat);
    }
    // Local part must be non-empty.
    if at_pos == 0 {
        return Err(SanitizationError::InvalidFormat);
    }
    // Domain part must be non-empty and contain at least one dot.
    let domain_start = at_pos + 1;
    if domain_start >= len_usize {
        return Err(SanitizationError::InvalidFormat);
    }

    let mut domain_has_dot = false;
    for i in domain_start..len_usize {
        if buf[i] == b'.' {
            domain_has_dot = true;
            break;
        }
    }
    if !domain_has_dot {
        return Err(SanitizationError::InvalidFormat);
    }

    Ok(())
}

/// Validates an identifier: alphanumeric chars, hyphens, underscores, colons,
/// dots, and forward slashes (covers DIDs, slugs, and resource paths).
pub fn sanitize_id(_env: &Env, input: &String) -> Result<(), SanitizationError> {
    let len = input.len();

    if len == 0 {
        return Err(SanitizationError::EmptyInput);
    }
    if len > BUFFER_SIZE as u32 || len > MAX_ID_LEN {
        return Err(SanitizationError::InputTooLong);
    }

    let len_usize = len as usize;
    let mut buf = [0u8; BUFFER_SIZE];
    input.copy_into_slice(&mut buf[..len_usize]);

    for i in 0..len_usize {
        let b = buf[i];
        if b == 0x00 {
            return Err(SanitizationError::NullByte);
        }
        if !is_id_char(b) {
            return Err(SanitizationError::InvalidCharacter);
        }
    }

    Ok(())
}

/// Validates a URL: printable ASCII only, length within MAX_URL_LEN.
pub fn sanitize_url(_env: &Env, input: &String) -> Result<(), SanitizationError> {
    let len = input.len();

    if len == 0 {
        return Err(SanitizationError::EmptyInput);
    }
    if len > BUFFER_SIZE as u32 || len > MAX_URL_LEN {
        return Err(SanitizationError::InputTooLong);
    }

    let len_usize = len as usize;
    let mut buf = [0u8; BUFFER_SIZE];
    input.copy_into_slice(&mut buf[..len_usize]);

    for i in 0..len_usize {
        let b = buf[i];
        if b == 0x00 {
            return Err(SanitizationError::NullByte);
        }
        // Printable ASCII only for URLs (0x21-0x7E, no space).
        if b < 0x21 || b > 0x7E {
            return Err(SanitizationError::InvalidCharacter);
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Character class helpers
// ---------------------------------------------------------------------------

#[inline]
fn is_email_char(b: u8) -> bool {
    matches!(b,
        b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9'
        | b'!' | b'#' | b'$' | b'%' | b'&' | b'\'' | b'*' | b'+'
        | b'/' | b'=' | b'?' | b'^' | b'_' | b'`' | b'{' | b'|'
        | b'}' | b'~' | b'.' | b'-' | b'@'
    )
}

#[inline]
fn is_id_char(b: u8) -> bool {
    matches!(b,
        b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9'
        | b'-' | b'_' | b':' | b'.' | b'/' | b'#'
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{Env, String};

    // --- sanitize_string ---

    #[test]
    fn test_sanitize_string_valid() {
        let env = Env::default();
        let s = String::from_str(&env, "Hello World 123");
        assert!(sanitize_string(&env, &s, MAX_GENERAL_LEN).is_ok());
    }

    #[test]
    fn test_sanitize_string_empty() {
        let env = Env::default();
        let s = String::from_str(&env, "");
        assert_eq!(
            sanitize_string(&env, &s, MAX_GENERAL_LEN),
            Err(SanitizationError::EmptyInput)
        );
    }

    #[test]
    fn test_sanitize_string_too_long() {
        let env = Env::default();
        // Build a 10-char string and reject it with max_len=5.
        let s = String::from_str(&env, "HelloWorld");
        assert_eq!(
            sanitize_string(&env, &s, 5),
            Err(SanitizationError::InputTooLong)
        );
    }

    #[test]
    fn test_sanitize_string_null_byte() {
        let env = Env::default();
        let s = String::from_bytes(&env, b"hello\x00world");
        assert_eq!(
            sanitize_string(&env, &s, MAX_GENERAL_LEN),
            Err(SanitizationError::NullByte)
        );
    }

    #[test]
    fn test_sanitize_string_control_char() {
        let env = Env::default();
        let s = String::from_bytes(&env, b"bad\x01char");
        assert_eq!(
            sanitize_string(&env, &s, MAX_GENERAL_LEN),
            Err(SanitizationError::InvalidCharacter)
        );
    }

    #[test]
    fn test_sanitize_string_allows_tab_lf_cr() {
        let env = Env::default();
        let s = String::from_bytes(&env, b"line1\nline2\ttab\r");
        assert!(sanitize_string(&env, &s, MAX_GENERAL_LEN).is_ok());
    }

    // --- sanitize_name ---

    #[test]
    fn test_sanitize_name_valid() {
        let env = Env::default();
        let s = String::from_str(&env, "Dr. Jane O'Brien-Smith");
        assert!(sanitize_name(&env, &s).is_ok());
    }

    #[test]
    fn test_sanitize_name_rejects_angle_bracket() {
        let env = Env::default();
        let s = String::from_str(&env, "<script>");
        assert_eq!(
            sanitize_name(&env, &s),
            Err(SanitizationError::InvalidCharacter)
        );
    }

    #[test]
    fn test_sanitize_name_too_long() {
        let env = Env::default();
        // 129 'a' characters
        let long: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\
             aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        assert_eq!(long.len(), 130);
        let s = String::from_str(&env, long);
        assert_eq!(
            sanitize_name(&env, &s),
            Err(SanitizationError::InputTooLong)
        );
    }

    // --- sanitize_email ---

    #[test]
    fn test_sanitize_email_valid() {
        let env = Env::default();
        let s = String::from_str(&env, "user@example.com");
        assert!(sanitize_email(&env, &s).is_ok());
    }

    #[test]
    fn test_sanitize_email_no_at() {
        let env = Env::default();
        let s = String::from_str(&env, "userexample.com");
        assert_eq!(
            sanitize_email(&env, &s),
            Err(SanitizationError::InvalidFormat)
        );
    }

    #[test]
    fn test_sanitize_email_double_at() {
        let env = Env::default();
        let s = String::from_str(&env, "a@@b.com");
        assert_eq!(
            sanitize_email(&env, &s),
            Err(SanitizationError::InvalidFormat)
        );
    }

    #[test]
    fn test_sanitize_email_empty_local() {
        let env = Env::default();
        let s = String::from_str(&env, "@example.com");
        assert_eq!(
            sanitize_email(&env, &s),
            Err(SanitizationError::InvalidFormat)
        );
    }

    #[test]
    fn test_sanitize_email_no_dot_in_domain() {
        let env = Env::default();
        let s = String::from_str(&env, "user@localhost");
        assert_eq!(
            sanitize_email(&env, &s),
            Err(SanitizationError::InvalidFormat)
        );
    }

    #[test]
    fn test_sanitize_email_invalid_char() {
        let env = Env::default();
        let s = String::from_str(&env, "user name@example.com");
        assert_eq!(
            sanitize_email(&env, &s),
            Err(SanitizationError::InvalidCharacter)
        );
    }

    // --- sanitize_id ---

    #[test]
    fn test_sanitize_id_valid_did_fragment() {
        let env = Env::default();
        let s = String::from_str(&env, "key-1");
        assert!(sanitize_id(&env, &s).is_ok());
    }

    #[test]
    fn test_sanitize_id_valid_with_colon() {
        let env = Env::default();
        let s = String::from_str(&env, "did:stellar:key-1");
        assert!(sanitize_id(&env, &s).is_ok());
    }

    #[test]
    fn test_sanitize_id_rejects_space() {
        let env = Env::default();
        let s = String::from_str(&env, "bad id");
        assert_eq!(
            sanitize_id(&env, &s),
            Err(SanitizationError::InvalidCharacter)
        );
    }

    #[test]
    fn test_sanitize_id_rejects_injection() {
        let env = Env::default();
        let s = String::from_str(&env, "id'; DROP TABLE");
        assert_eq!(
            sanitize_id(&env, &s),
            Err(SanitizationError::InvalidCharacter)
        );
    }

    #[test]
    fn test_sanitize_id_too_long() {
        let env = Env::default();
        let long = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"; // 66 chars
        assert_eq!(long.len(), 66);
        let s = String::from_str(&env, long);
        assert_eq!(sanitize_id(&env, &s), Err(SanitizationError::InputTooLong));
    }

    // --- sanitize_url ---

    #[test]
    fn test_sanitize_url_valid() {
        let env = Env::default();
        let s = String::from_str(&env, "https://example.com/path?q=1&r=2");
        assert!(sanitize_url(&env, &s).is_ok());
    }

    #[test]
    fn test_sanitize_url_rejects_space() {
        let env = Env::default();
        let s = String::from_str(&env, "https://example.com/bad path");
        assert_eq!(
            sanitize_url(&env, &s),
            Err(SanitizationError::InvalidCharacter)
        );
    }

    #[test]
    fn test_sanitize_url_rejects_null() {
        let env = Env::default();
        let s = String::from_bytes(&env, b"https://evil.com/\x00");
        assert_eq!(sanitize_url(&env, &s), Err(SanitizationError::NullByte));
    }

    // --- boundary conditions ---

    #[test]
    fn test_exactly_at_max_len_is_ok() {
        let env = Env::default();
        // 5-char string accepted with max_len=5.
        let s = String::from_str(&env, "Hello");
        assert!(sanitize_string(&env, &s, 5).is_ok());
    }

    #[test]
    fn test_one_over_max_len_rejected() {
        let env = Env::default();
        let s = String::from_str(&env, "Hello!");
        assert_eq!(
            sanitize_string(&env, &s, 5),
            Err(SanitizationError::InputTooLong)
        );
    }
}
