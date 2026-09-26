#![no_std]
//! The canonical ICD-10 / CPT mapping, shared by clinical contracts and
//! off-chain tooling.
//!
//! `contracts/clinical_nlp/src/icd_cpt_codes.rs` used to hold a ~950-line
//! literal table of diagnosis and procedure codes alongside the on-chain
//! lookup engine that consumed it. Because the table lived inside the
//! contract, `clinical_decision_support` and any off-chain tool had to
//! re-declare the same mapping, and the copies could drift (issue #1629).
//!
//! This crate splits the two concerns:
//!
//! * **this crate** owns the *mapping* — which code means what, how it is
//!   categorized, when it is effective, what it is related to, and which
//!   keywords identify it. It is `#![no_std]`, depends on nothing, and is
//!   therefore valid both as a Soroban contract dependency and as a plain
//!   Rust library for tests, fixtures and tooling.
//! * **the contract** owns the *storage shape* — a `MedicalCode` with
//!   `soroban_sdk::String` fields written into host storage, plus the keyword
//!   index built over it.
//!
//! The contract no longer carries a second copy of the data: it converts
//! [`CATALOG`] into host storage, and every other consumer reads the same
//! table this crate validates.
//!
//! # Example
//!
//! ```
//! let matches = icd_cpt::search("patient reports high blood pressure", None, 3);
//! assert_eq!(matches[0].code.code, "I10");
//! assert_eq!(matches[0].code.code_type, icd_cpt::CodeType::Icd10);
//!
//! let icd = icd_cpt::by_code("I10").expect("I10 is in the seed catalog");
//! assert!(icd.is_billable);
//! assert!(icd.related_codes.contains(&"I11.9"));
//!
//! assert_eq!(icd_cpt::validate(), Ok(()));
//! ```

extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Reverse;

pub mod catalog;

pub use catalog::CATALOG;

/// The coding system a code belongs to.
///
/// `Hcpcs`, `Snomed` and `Loinc` exist because the contract-side enum already
/// modelled them, so downstream code keeps compiling. The seed catalog
/// currently populates only [`CodeType::Icd10`] and [`CodeType::Cpt`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CodeType {
    Icd10,
    Cpt,
    Hcpcs,
    Snomed,
    Loinc,
}

impl CodeType {
    /// Every code type, in declaration order.
    pub const ALL: [CodeType; 5] = [
        CodeType::Icd10,
        CodeType::Cpt,
        CodeType::Hcpcs,
        CodeType::Snomed,
        CodeType::Loinc,
    ];

    /// The uppercase label used in event payloads and CLI output, e.g.
    /// `"ICD10"`. This is the spelling the contract used before the enum
    /// moved here, so a consumer that persisted the label as a string is
    /// unaffected.
    pub const fn label(self) -> &'static str {
        match self {
            CodeType::Icd10 => "ICD10",
            CodeType::Cpt => "CPT",
            CodeType::Hcpcs => "HCPCS",
            CodeType::Snomed => "SNOMED",
            CodeType::Loinc => "LOINC",
        }
    }

    /// Parses a [`CodeType::label`], case-insensitively.
    pub fn from_label(label: &str) -> Option<CodeType> {
        match label.to_ascii_lowercase().as_str() {
            "icd10" => Some(CodeType::Icd10),
            "cpt" => Some(CodeType::Cpt),
            "hcpcs" => Some(CodeType::Hcpcs),
            "snomed" => Some(CodeType::Snomed),
            "loinc" => Some(CodeType::Loinc),
            _ => None,
        }
    }

    /// Whether codes of this type describe a procedure rather than a
    /// diagnosis.
    pub const fn is_procedural(self) -> bool {
        matches!(self, CodeType::Cpt | CodeType::Hcpcs)
    }

    /// Every catalog entry of this type, in catalog order.
    pub fn entries(self) -> impl Iterator<Item = &'static CatalogCode> {
        CATALOG.iter().filter(move |entry| entry.code_type == self)
    }
}

/// One code in the canonical mapping.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CatalogCode {
    /// The code itself, e.g. `"I10"` or `"99213"`.
    pub code: &'static str,
    /// The coding system the code belongs to.
    pub code_type: CodeType,
    /// Human-readable meaning of the code.
    pub description: &'static str,
    /// Grouping the code is reported under, e.g. `"Diseases of the
    /// circulatory system"`.
    pub category: &'static str,
    /// Whether the code can be billed.
    pub is_billable: bool,
    /// Unix timestamp from which the code is valid; `0` means "always".
    pub effective_date: u64,
    /// Unix timestamp after which the code is retired, if it ever is.
    pub expiration_date: Option<u64>,
    /// Other codes in the same clinical family. References outside the seed
    /// catalog are legitimate — they point into the wider code set the catalog
    /// is a subset of — and are resolved by [`by_code`] when present.
    pub related_codes: &'static [&'static str],
    /// Lowercase keywords that identify this code in free text.
    ///
    /// A keyword may be claimed by more than one code: `"pacemaker"` is
    /// evidence for both `Z95.0` (a pacemaker in the patient's history) and
    /// `33208` (pacemaker implantation), and the contract's keyword index was
    /// always a keyword-to-many-codes map for exactly that reason. A keyword
    /// claimed twice by the *same* entry is a defect, and [`validate`]
    /// rejects it.
    pub keywords: &'static [&'static str],
}

impl CatalogCode {
    /// Whether the code is in force at `at`, taking both the effective date
    /// and any expiration into account.
    pub const fn is_effective_at(&self, at: u64) -> bool {
        if self.effective_date > at {
            return false;
        }
        match self.expiration_date {
            Some(expiry) => at < expiry,
            None => true,
        }
    }
}

/// A scored catalog hit produced by [`search`].
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Match<'a> {
    /// The matched entry.
    pub code: &'a CatalogCode,
    /// [`KEYWORD_WEIGHT`] times the number of distinct keywords that hit.
    pub score: u32,
    /// The distinct keywords from the input text that matched, in catalog
    /// keyword order.
    pub evidence: Vec<&'a str>,
}

impl<'a> Match<'a> {
    /// Confidence in basis points, using the same bands the contract
    /// reported before this crate existed: `>300 => 9500`, `>200 => 8500`,
    /// `>100 => 7500`, otherwise `6000`.
    pub const fn confidence_bps(&self) -> u32 {
        if self.score > 300 {
            9500
        } else if self.score > 200 {
            8500
        } else if self.score > 100 {
            7500
        } else {
            6000
        }
    }
}

/// Score contributed by a single keyword hit.
pub const KEYWORD_WEIGHT: u32 = 100;

/// Minimum token length, in bytes, for a word to be a keyword candidate.
pub const MIN_TOKEN_LEN: usize = 3;

/// A catalog that does not satisfy [`validate`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CatalogError {
    /// Two entries share a code.
    DuplicateCode,
    /// The code does not look like a code of its declared type.
    MalformedCode,
    /// `description` or `category` is empty.
    EmptyField,
    /// An entry has no keywords, so it can never be suggested.
    NoKeywords,
    /// A keyword is the empty string.
    EmptyKeyword,
    /// A keyword is not lowercase, so an index lookup would miss it.
    KeywordNotLowercase,
    /// One entry lists the same keyword twice, which would double-count it.
    DuplicateKeywordInEntry,
    /// `related_codes` points back at the entry itself.
    SelfRelated,
    /// `expiration_date` precedes `effective_date`.
    ExpiredBeforeEffective,
}

/// Every entry of the given type, in catalog order.
pub fn of_type(code_type: CodeType) -> impl Iterator<Item = &'static CatalogCode> {
    CATALOG.iter().filter(move |entry| entry.code_type == code_type)
}

/// Every ICD-10 entry.
pub fn icd10() -> impl Iterator<Item = &'static CatalogCode> {
    of_type(CodeType::Icd10)
}

/// Every CPT entry.
pub fn cpt() -> impl Iterator<Item = &'static CatalogCode> {
    of_type(CodeType::Cpt)
}

/// Looks a code up, case-insensitively.
pub fn by_code(code: &str) -> Option<&'static CatalogCode> {
    CATALOG.iter().find(|entry| entry.code.eq_ignore_ascii_case(code))
}

/// Looks a code up and additionally asserts its coding system.
pub fn by_code_typed(code: &str, code_type: CodeType) -> Option<&'static CatalogCode> {
    by_code(code).filter(|entry| entry.code_type == code_type)
}

/// Whether the catalog contains `code`.
pub fn contains(code: &str) -> bool {
    by_code(code).is_some()
}

/// The distinct categories in the catalog, sorted.
pub fn categories() -> Vec<&'static str> {
    let set: BTreeSet<&'static str> = CATALOG.iter().map(|entry| entry.category).collect();
    set.into_iter().collect()
}

/// The distinct keywords in the catalog, sorted.
pub fn keywords() -> Vec<&'static str> {
    let set: BTreeSet<&'static str> =
        CATALOG.iter().flat_map(|entry| entry.keywords.iter().copied()).collect();
    set.into_iter().collect()
}

/// Maps every keyword to the codes that claim it, in catalog order.
///
/// A keyword maps to more than one code when a diagnosis and the procedure
/// that addresses it share vocabulary. This mirrors the on-chain index the
/// contract builds, which was already a keyword-to-many-codes map.
///
/// Building the index is left to the caller rather than cached in a `static`,
/// because a lazily initialized static would add cost to every contract
/// invocation and shared mutable state is not available on-chain.
pub fn keyword_index() -> BTreeMap<&'static str, Vec<&'static str>> {
    let mut index: BTreeMap<&'static str, Vec<&'static str>> = BTreeMap::new();
    for entry in CATALOG {
        for &keyword in entry.keywords.iter() {
            index.entry(keyword).or_default().push(entry.code);
        }
    }
    index
}

/// The codes `entry` names directly in `related_codes`, skipping `entry`
/// itself so a caller never gets its input back as a "related" code.
pub fn related_of(entry: &CatalogCode) -> impl Iterator<Item = &'static str> {
    entry.related_codes.iter().copied().filter(move |reference| *reference != entry.code)
}

/// The codes named directly by `code`, or nothing when `code` is unknown.
///
/// The body is spelled out rather than written as `flat_map(related_of)`:
/// `related_of` returns `impl Iterator`, whose concrete type depends on the
/// lifetime of its argument, so it cannot be passed to `flat_map` as a
/// function item.
pub fn related(code: &str) -> impl Iterator<Item = &'static str> {
    by_code(code).into_iter().flat_map(|entry| {
        entry.related_codes.iter().copied().filter(move |reference| *reference != entry.code)
    })
}

/// The related-code family of `entry`: its direct references plus, for each
/// reference the catalog also carries, that entry's own references.
///
/// One extra hop is deliberate. The catalog references the wider code set, so
/// an unbounded walk could leave the catalog entirely and never terminate.
/// References the catalog does not carry are returned unresolved, which is
/// what keeps the result usable for a follow-up [`by_code`] lookup.
pub fn related_family_of(entry: &CatalogCode) -> BTreeSet<&'static str> {
    let mut family = BTreeSet::new();
    for reference in related_of(entry) {
        family.insert(reference);
        if let Some(linked) = by_code(reference) {
            family.extend(linked.related_codes.iter().copied());
        }
    }
    family.remove(entry.code);
    family
}

/// The related-code family of `code`, or an empty set when `code` is unknown.
pub fn related_family(code: &str) -> BTreeSet<&'static str> {
    by_code(code).map(related_family_of).unwrap_or_default()
}

/// Every catalog entry in force at `at`.
pub fn effective_at(at: u64) -> impl Iterator<Item = &'static CatalogCode> {
    CATALOG.iter().filter(move |entry| entry.is_effective_at(at))
}

/// ASCII-lowercases a keyword, mirroring the contract's byte-level
/// `to_lowercase` so an on-chain index key and an off-chain index key are the
/// same bytes.
pub fn normalize(keyword: &str) -> String {
    let mut out = String::with_capacity(keyword.len());
    for &byte in keyword.as_bytes() {
        if byte.is_ascii_uppercase() {
            out.push((byte + 32) as char);
        } else {
            out.push(byte as char);
        }
    }
    out
}

/// Whether `code` is shaped like a code of `code_type`:
///
/// * `Icd10` — three characters, a leading uppercase letter and two digits,
///   optionally followed by `.` and one or more digits or uppercase letters.
///   The decimal part is optional because a category code such as `I10` is
///   valid on its own.
/// * `Cpt` — five digits, then zero or more uppercase modifier letters.
/// * `Hcpcs` — one uppercase letter and four digits.
/// * `Loinc` — five digits, a hyphen, one digit.
/// * `Snomed` — six or more digits.
///
/// The implementation works on bytes, so it cannot panic on a multi-byte
/// character that happens to sit at a slice boundary.
pub fn is_well_formed(code: &str, code_type: CodeType) -> bool {
    let bytes = code.as_bytes();
    match code_type {
        CodeType::Icd10 => match bytes.iter().position(|byte| *byte == b'.') {
            Some(dot) => {
                let (head, rest) = bytes.split_at(dot);
                let tail = &rest[1..];
                head.len() == 3
                    && head[0].is_ascii_uppercase()
                    && head[1..].iter().all(u8::is_ascii_digit)
                    && !tail.is_empty()
                    && tail.iter().all(|byte| byte.is_ascii_digit() || byte.is_ascii_uppercase())
            }
            None => {
                bytes.len() == 3
                    && bytes[0].is_ascii_uppercase()
                    && bytes[1..].iter().all(u8::is_ascii_digit)
            }
        },
        CodeType::Cpt => {
            bytes.len() >= 5
                && bytes[..5].iter().all(u8::is_ascii_digit)
                && bytes[5..].iter().all(u8::is_ascii_uppercase)
        }
        CodeType::Hcpcs => {
            bytes.len() == 5
                && bytes[0].is_ascii_uppercase()
                && bytes[1..].iter().all(u8::is_ascii_digit)
        }
        CodeType::Loinc => {
            bytes.len() == 7
                && bytes[..5].iter().all(u8::is_ascii_digit)
                && bytes[5] == b'-'
                && bytes[6].is_ascii_digit()
        }
        CodeType::Snomed => bytes.len() >= 6 && bytes.iter().all(u8::is_ascii_digit),
    }
}

/// Checks every catalog invariant, returning the first violation.
///
/// The seed catalog satisfies this. The check exists so that a later catalog
/// addition which breaks one of the contracts above is caught by a test
/// rather than by a silently wrong suggestion on-chain.
///
/// A keyword shared by two codes is *not* a violation — see
/// [`CatalogCode::keywords`]. Everything else listed on [`CatalogError`] is.
pub fn validate() -> Result<(), CatalogError> {
    let mut seen_codes: BTreeSet<&str> = BTreeSet::new();

    for entry in CATALOG {
        if !seen_codes.insert(entry.code) {
            return Err(CatalogError::DuplicateCode);
        }
        if !is_well_formed(entry.code, entry.code_type) {
            return Err(CatalogError::MalformedCode);
        }
        if entry.description.is_empty() || entry.category.is_empty() {
            return Err(CatalogError::EmptyField);
        }
        if let Some(expiry) = entry.expiration_date {
            if expiry < entry.effective_date {
                return Err(CatalogError::ExpiredBeforeEffective);
            }
        }
        for &reference in entry.related_codes.iter() {
            if reference == entry.code {
                return Err(CatalogError::SelfRelated);
            }
        }
        if entry.keywords.is_empty() {
            return Err(CatalogError::NoKeywords);
        }
        let mut own_keywords: BTreeSet<&str> = BTreeSet::new();
        for &keyword in entry.keywords.iter() {
            if keyword.is_empty() {
                return Err(CatalogError::EmptyKeyword);
            }
            if keyword != normalize(keyword).as_str() {
                return Err(CatalogError::KeywordNotLowercase);
            }
            if !own_keywords.insert(keyword) {
                return Err(CatalogError::DuplicateKeywordInEntry);
            }
        }
    }

    Ok(())
}

/// Byte-exact port of the contract's former `extract_keywords`: runs of
/// ASCII `[0-9a-z]` at least [`MIN_TOKEN_LEN`] bytes long, with no case
/// folding.
///
/// The case sensitivity is inherited, not endorsed — an uppercase byte
/// terminates a token, so `"Chest pain"` yields `["hest", "pain"]`. Keeping
/// the quirk is precisely what makes this a pure extraction of the previous
/// behavior; [`tokenize_ci`] is the tokenizer new callers should use.
pub fn tokenize(text: &str) -> Vec<&str> {
    let bytes = text.as_bytes();
    let mut tokens = Vec::new();
    let mut start: Option<usize> = None;

    for (index, byte) in bytes.iter().enumerate() {
        if byte.is_ascii_digit() || byte.is_ascii_lowercase() {
            if start.is_none() {
                start = Some(index);
            }
        } else if let Some(begin) = start {
            if index - begin >= MIN_TOKEN_LEN {
                tokens.push(&text[begin..index]);
            }
            start = None;
        }
    }

    if let Some(begin) = start {
        if bytes.len() - begin >= MIN_TOKEN_LEN {
            tokens.push(&text[begin..]);
        }
    }

    tokens
}

/// [`tokenize`] with ASCII case folding, so `"Chest pain"` yields
/// `["chest", "pain"]`.
///
/// Returns owned tokens because the folded text is a temporary: a version
/// returning borrows would hand out references into a value already dropped.
pub fn tokenize_ci(text: &str) -> Vec<String> {
    let folded: String = text
        .bytes()
        .map(|byte| if byte.is_ascii_uppercase() { (byte + 32) as char } else { byte as char })
        .collect();
    let mut tokens = Vec::new();
    for token in tokenize(&folded) {
        tokens.push(String::from(token));
    }
    tokens
}

/// The entries whose keywords appear in `text`, best score first.
///
/// Scoring mirrors the contract's former `suggest_codes`: each distinct
/// keyword hit adds [`KEYWORD_WEIGHT`] and [`Match::confidence_bps`] maps the
/// total onto the same four bands, so a caller that persisted a confidence
/// keeps seeing the same number. Unlike the contract version this folds case,
/// because the catalog's keywords are lowercase and a capitalized clinical
/// note is the common case rather than the exception.
///
/// The scan is entry-driven rather than index-driven, so a keyword shared by
/// two codes scores for both, exactly as the contract's keyword-to-many-codes
/// index did. Ties break on the code itself, making the ordering total and
/// reproducible across runs and across on-chain and off-chain callers.
pub fn search(text: &str, filter: Option<CodeType>, max: usize) -> Vec<Match<'static>> {
    let tokens = tokenize_ci(text);
    let mut hits: BTreeMap<&'static str, Vec<&'static str>> = BTreeMap::new();

    for entry in CATALOG {
        if filter.is_some() && filter != Some(entry.code_type) {
            continue;
        }
        let mut evidence: Vec<&'static str> = Vec::new();
        for &keyword in entry.keywords.iter() {
            if tokens.iter().any(|token| token.as_str() == keyword)
                && !evidence.contains(&keyword)
            {
                evidence.push(keyword);
            }
        }
        if !evidence.is_empty() {
            hits.insert(entry.code, evidence);
        }
    }

    let mut matches: Vec<Match<'static>> = hits
        .into_iter()
        .filter_map(|(code, evidence)| {
            let entry = by_code(code)?;
            Some(Match {
                score: KEYWORD_WEIGHT * evidence.len() as u32,
                code: entry,
                evidence,
            })
        })
        .collect();

    matches.sort_by_key(|hit| (Reverse(hit.score), hit.code.code));
    matches.truncate(max);
    matches
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    /// A minimal, catalog-shaped entry for exercising the helpers that take a
    /// `&CatalogCode` without depending on catalog contents.
    fn fixture(code: &'static str) -> CatalogCode {
        CatalogCode {
            code,
            code_type: CodeType::Icd10,
            description: "fixture",
            category: "fixture",
            is_billable: false,
            effective_date: 0,
            expiration_date: None,
            related_codes: &[],
            keywords: &["fixturekeyword"],
        }
    }

    fn owned(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| String::from(*value)).collect()
    }

    #[test]
    fn seed_catalog_keeps_its_full_size() {
        assert_eq!(CATALOG.len(), 36);
        assert_eq!(icd10().count(), 20);
        assert_eq!(cpt().count(), 16);
    }

    #[test]
    fn catalog_satisfies_every_invariant() {
        assert_eq!(validate(), Ok(()));
    }

    #[test]
    fn codes_are_unique() {
        let unique: BTreeSet<&str> = CATALOG.iter().map(|entry| entry.code).collect();
        assert_eq!(unique.len(), CATALOG.len());
    }

    #[test]
    fn every_code_is_well_formed_for_its_type() {
        for entry in CATALOG {
            assert!(
                is_well_formed(entry.code, entry.code_type),
                "{} is not a well-formed {:?} code",
                entry.code,
                entry.code_type
            );
        }
    }

    #[test]
    fn is_well_formed_accepts_real_codes() {
        assert!(is_well_formed("I10", CodeType::Icd10));
        assert!(is_well_formed("E11.65", CodeType::Icd10));
        assert!(is_well_formed("J45.909", CodeType::Icd10));
        assert!(is_well_formed("99213", CodeType::Cpt));
        assert!(is_well_formed("94760TG", CodeType::Cpt));
        assert!(is_well_formed("A4253", CodeType::Hcpcs));
        assert!(is_well_formed("8867-4", CodeType::Loinc));
        assert!(is_well_formed("1234567", CodeType::Snomed));
    }

    #[test]
    fn is_well_formed_rejects_malformed_codes() {
        assert!(!is_well_formed("I1", CodeType::Icd10));
        assert!(!is_well_formed("I10.", CodeType::Icd10));
        assert!(!is_well_formed("I10.0.1", CodeType::Icd10));
        assert!(!is_well_formed("110.1", CodeType::Icd10));
        assert!(!is_well_formed("I1O.0", CodeType::Icd10));
        assert!(!is_well_formed("I10", CodeType::Cpt));
        assert!(!is_well_formed("9476", CodeType::Cpt));
        assert!(!is_well_formed("9921X", CodeType::Cpt));
        assert!(!is_well_formed("A425", CodeType::Hcpcs));
        assert!(!is_well_formed("4253", CodeType::Hcpcs));
        assert!(!is_well_formed("12345", CodeType::Loinc));
        assert!(!is_well_formed("12345", CodeType::Snomed));
    }

    #[test]
    fn is_well_formed_does_not_panic_on_multibyte_input() {
        for input in ["9€9135", "€€€€€1", "I1é", "9€", "9€€€5", "é", "€9", "9€9", ".5"] {
            let _ = is_well_formed(input, CodeType::Icd10);
            let _ = is_well_formed(input, CodeType::Cpt);
            let _ = is_well_formed(input, CodeType::Hcpcs);
            let _ = is_well_formed(input, CodeType::Loinc);
            let _ = is_well_formed(input, CodeType::Snomed);
        }
    }

    #[test]
    fn keywords_are_lowercase_non_empty_and_not_repeated_per_entry() {
        for entry in CATALOG {
            assert!(!entry.keywords.is_empty(), "{} has no keywords", entry.code);
            let mut seen: BTreeSet<&str> = BTreeSet::new();
            for &keyword in entry.keywords.iter() {
                assert!(!keyword.is_empty(), "{} has an empty keyword", entry.code);
                assert_eq!(keyword, normalize(keyword).as_str(), "{}", entry.code);
                assert!(seen.insert(keyword), "{} repeats {}", entry.code, keyword);
            }
        }
    }

    #[test]
    fn every_keyword_resolves_back_to_its_codes() {
        let index = keyword_index();
        assert_eq!(index.len(), keywords().len());
        for (keyword, codes) in index.iter() {
            assert!(!codes.is_empty(), "{} maps to nothing", keyword);
            for code in codes {
                let entry = by_code(code).expect("index points at a real code");
                assert!(
                    entry.keywords.contains(keyword),
                    "{} does not claim {}",
                    code,
                    keyword
                );
            }
        }
    }

    #[test]
    fn shared_keywords_map_to_more_than_one_code() {
        // "pacemaker" is evidence for both a pacemaker history (Z95.0) and
        // pacemaker implantation (33208). The contract's index was always a
        // keyword-to-many-codes map, so this must stay a multimap.
        let index = keyword_index();
        assert_eq!(index.get("pacemaker").map(Vec::len), Some(2));
        assert!(index["pacemaker"].contains(&"Z95.0"));
        assert!(index["pacemaker"].contains(&"33208"));
        assert_eq!(index.get("hypertension").map(Vec::len), Some(1));
    }

    #[test]
    fn search_lets_a_shared_keyword_reach_both_codes() {
        let hits: Vec<&str> = search("pacemaker", None, 5)
            .iter()
            .map(|hit| hit.code.code)
            .collect();
        assert_eq!(hits, vec!["33208", "Z95.0"]);

        let hits: Vec<&str> = search("ventilator", None, 5)
            .iter()
            .map(|hit| hit.code.code)
            .collect();
        assert_eq!(hits, vec!["94002", "Z99.11"]);
    }

    #[test]
    fn descriptions_and_categories_are_populated() {
        for entry in CATALOG {
            assert!(!entry.description.is_empty(), "{}", entry.code);
            assert!(!entry.category.is_empty(), "{}", entry.code);
        }
    }

    #[test]
    fn no_entry_relates_to_itself() {
        for entry in CATALOG {
            assert!(
                !entry.related_codes.contains(&entry.code),
                "{} relates to itself",
                entry.code
            );
            assert!(related_of(entry).all(|code| code != entry.code));
        }
    }

    #[test]
    fn related_of_and_related_agree() {
        for entry in CATALOG {
            let direct: BTreeSet<&str> = related_of(entry).collect();
            let via_code: BTreeSet<&str> = related(entry.code).collect();
            assert_eq!(direct, via_code, "{}", entry.code);
        }
    }

    #[test]
    fn related_family_never_includes_the_input() {
        for entry in CATALOG {
            assert!(!related_family_of(entry).contains(entry.code), "{}", entry.code);
        }
        let family = related_family("I10");
        assert!(family.contains("I11.9"));
        assert!(family.contains("I12.9"));
        assert!(!family.contains("I10"));
    }

    #[test]
    fn related_family_resolves_a_second_hop() {
        // "A99.9" -> "I10" (in the catalog) -> "I11.9"/"I12.9".
        let entry = CatalogCode { related_codes: &["I10"], ..fixture("A99.9") };
        let family = related_family_of(&entry);
        assert!(family.contains("I10"));
        assert!(family.contains("I11.9"));
        assert!(family.contains("I12.9"));
        assert!(!family.contains("A99.9"));
    }

    #[test]
    fn related_helpers_tolerate_an_unknown_code() {
        assert!(related_family("not-a-code").is_empty());
        assert!(related("not-a-code").next().is_none());
        assert!(related_of(&fixture("A99.9")).next().is_none());
    }

    #[test]
    fn by_code_is_case_insensitive_and_typed() {
        assert_eq!(by_code("i10").map(|entry| entry.code), Some("I10"));
        assert_eq!(by_code_typed("I10", CodeType::Icd10).map(|e| e.code), Some("I10"));
        assert!(by_code_typed("I10", CodeType::Cpt).is_none());
        assert!(by_code("not-a-code").is_none());
        assert!(contains("I10"));
        assert!(!contains("not-a-code"));
    }

    #[test]
    fn code_type_labels_round_trip() {
        for code_type in CodeType::ALL {
            assert_eq!(CodeType::from_label(code_type.label()), Some(code_type));
            let lowered = code_type.label().to_ascii_lowercase();
            assert_eq!(CodeType::from_label(&lowered), Some(code_type));
        }
        assert_eq!(CodeType::from_label("ICD-10"), None);
        assert_eq!(CodeType::from_label(""), None);
        assert!(CodeType::Cpt.is_procedural());
        assert!(CodeType::Hcpcs.is_procedural());
        assert!(!CodeType::Icd10.is_procedural());
    }

    #[test]
    fn type_filters_partition_the_catalog() {
        for entry in icd10() {
            assert_eq!(entry.code_type, CodeType::Icd10);
        }
        for entry in cpt() {
            assert_eq!(entry.code_type, CodeType::Cpt);
        }
        assert_eq!(icd10().count() + cpt().count(), CATALOG.len());
        assert_eq!(CodeType::Hcpcs.entries().count(), 0);
        assert_eq!(CodeType::Snomed.entries().count(), 0);
        assert_eq!(CodeType::Loinc.entries().count(), 0);
    }

    #[test]
    fn normalize_lowercases_ascii_only() {
        assert_eq!(normalize("Chest Pain"), "chest pain");
        assert_eq!(normalize("already"), "already");
        assert_eq!(normalize("I10"), "i10");
        assert_eq!(normalize(""), "");
    }

    #[test]
    fn tokenize_preserves_the_contracts_byte_rule() {
        assert_eq!(tokenize("hypertension"), vec!["hypertension"]);
        assert_eq!(tokenize("chest pain"), vec!["chest", "pain"]);
        // An uppercase byte terminates the token, exactly as it always has.
        assert_eq!(tokenize("Chest pain"), vec!["hest", "pain"]);
        assert!(tokenize("ab cd").is_empty());
        assert!(tokenize("").is_empty());
    }

    #[test]
    fn tokenize_ci_folds_case() {
        assert_eq!(tokenize_ci("Chest pain"), owned(&["chest", "pain"]));
        assert_eq!(tokenize_ci("CHEST PAIN"), owned(&["chest", "pain"]));
        assert!(tokenize_ci("").is_empty());
    }

    #[test]
    fn search_ranks_multi_keyword_hits_first() {
        let matches = search("elevated blood pressure and hypertension", None, 5);
        assert_eq!(matches[0].code.code, "I10");
        assert_eq!(matches[0].score, 2 * KEYWORD_WEIGHT);
        assert!(matches[0].evidence.contains(&"hypertension"));
        assert!(matches[0].evidence.contains(&"elevated blood pressure"));
    }

    #[test]
    fn search_is_case_insensitive() {
        let matches = search("Patient has Type 2 Diabetes", None, 5);
        assert!(matches.iter().any(|hit| hit.code.code == "E11.9"));
    }

    #[test]
    fn search_honours_the_type_filter() {
        let text = "blood count cbc";
        let diagnoses = search(text, Some(CodeType::Icd10), 5);
        assert!(diagnoses.iter().all(|hit| hit.code.code_type == CodeType::Icd10));
        let procedures = search(text, Some(CodeType::Cpt), 5);
        assert!(procedures.iter().any(|hit| hit.code.code == "85025"));
    }

    #[test]
    fn search_respects_the_result_cap() {
        let text = "hypertension diabetes asthma copd pneumonia stroke anemia \
                    seizure dizziness pacemaker ventilator";
        assert!(search(text, None, 3).len() <= 3);
        assert!(search(text, None, 100).len() > 3);
        assert!(search(text, None, 0).is_empty());
    }

    #[test]
    fn search_returns_nothing_for_unrelated_text() {
        assert!(search("the quick brown fox jumps", None, 5).is_empty());
        assert!(search("", None, 5).is_empty());
    }

    #[test]
    fn search_evidence_is_deduplicated() {
        let matches = search("hypertension hypertension htn", None, 5);
        let first = &matches[0];
        assert_eq!(first.code.code, "I10");
        assert_eq!(first.score, KEYWORD_WEIGHT * first.evidence.len() as u32);
        let mut evidence = first.evidence.clone();
        evidence.sort_unstable();
        let before = evidence.len();
        evidence.dedup();
        assert_eq!(evidence.len(), before);
    }

    #[test]
    fn search_ordering_is_total_and_stable() {
        let text = "hypertension diabetes asthma copd pneumonia stroke anemia \
                    seizure dizziness pacemaker ventilator";
        let first = search(text, None, 100);
        assert_eq!(first, search(text, None, 100));
        for pair in first.windows(2) {
            let ordered = pair[0].score > pair[1].score
                || (pair[0].score == pair[1].score && pair[0].code.code < pair[1].code.code);
            assert!(ordered, "{} then {}", pair[0].code.code, pair[1].code.code);
        }
    }

    #[test]
    fn confidence_bands_match_the_contract() {
        let make = |score: u32| -> Match<'static> {
            Match {
                code: &CATALOG[0],
                score,
                evidence: Vec::new(),
            }
        };
        assert_eq!(make(400).confidence_bps(), 9500);
        assert_eq!(make(301).confidence_bps(), 9500);
        assert_eq!(make(300).confidence_bps(), 8500);
        assert_eq!(make(201).confidence_bps(), 8500);
        assert_eq!(make(200).confidence_bps(), 7500);
        assert_eq!(make(101).confidence_bps(), 7500);
        assert_eq!(make(100).confidence_bps(), 6000);
    }

    #[test]
    fn is_effective_at_honours_both_dates() {
        let windowed = CatalogCode {
            effective_date: 100,
            expiration_date: Some(200),
            ..fixture("I99.9")
        };
        assert!(!windowed.is_effective_at(99));
        assert!(windowed.is_effective_at(100));
        assert!(windowed.is_effective_at(199));
        assert!(!windowed.is_effective_at(200));
        assert!(!windowed.is_effective_at(u64::MAX));
        for entry in CATALOG {
            assert!(entry.is_effective_at(0), "{}", entry.code);
            assert!(entry.is_effective_at(u64::MAX), "{}", entry.code);
        }
        assert_eq!(effective_at(0).count(), CATALOG.len());
    }

    #[test]
    fn categories_are_sorted_deduplicated_and_complete() {
        let listed = categories();
        let mut sorted = listed.clone();
        sorted.sort_unstable();
        assert_eq!(listed, sorted);
        sorted.dedup();
        assert_eq!(listed.len(), sorted.len());
        for entry in CATALOG {
            assert!(listed.contains(&entry.category), "{}", entry.code);
        }
    }

    #[test]
    fn keyword_list_and_index_agree() {
        assert_eq!(keyword_index().len(), keywords().len());
        for keyword in keywords() {
            assert!(keyword_index().contains_key(keyword));
        }
    }

    #[test]
    fn seed_catalog_carries_the_formerly_hardcoded_rows() {
        // Spot-checks for rows that used to be literals in the contract.
        let i10 = by_code("I10").expect("I10 is seeded");
        assert!(i10.is_billable);
        assert_eq!(i10.category, "Diseases of the circulatory system");
        assert_eq!(i10.related_codes, &["I11.9", "I12.9"][..]);

        assert_eq!(by_code("E11.9").map(|e| e.code_type), Some(CodeType::Icd10));
        assert!(by_code("99213").expect("99213 is seeded").is_billable);
        assert_eq!(by_code("85025").map(|e| e.category), Some("Pathology and Laboratory"));
        assert_eq!(
            by_code("R42").map(|e| e.related_codes),
            Some(&["H81.10"][..]),
            "R42 listed itself as related before the extraction"
        );
    }
}
