//! On-chain ICD-10 / CPT suggestion engine.
//!
//! The *mapping* — which code means what, its category, its related codes and
//! its keywords — lives in the `icd_cpt` workspace crate and is no longer
//! duplicated here (issue #1629). This module keeps only what is genuinely
//! contract-shaped:
//!
//! * [`MedicalCode`], the storage representation with `soroban_sdk::String`
//!   fields, and [`MedicalCode::from_catalog`], which adapts a canonical
//!   `icd_cpt::CatalogCode` into it.
//! * [`CodingDatabase`], the `Map`-backed lookup plus the keyword index the
//!   contract writes into host storage.
//! * Byte-level string helpers that operate on `soroban_sdk::String` without
//!   a host round-trip. The `icd_cpt` crate documents the equivalent
//!   `&str`-based rules for off-chain callers.

use soroban_sdk::{Env, Map, String, Vec};

/// The coding system of a code. Re-exported from the shared `icd_cpt` crate
/// so there is exactly one definition; it is re-exported rather than wrapped
/// so existing `crate::icd_cpt_codes::CodeType` paths keep resolving.
pub use icd_cpt::CodeType;

pub struct MedicalCode {
    pub code: String,
    pub code_type: CodeType,
    pub description: String,
    pub category: String,
    pub is_billable: bool,
    pub effective_date: u64,
    pub expiration_date: Option<u64>,
    pub related_codes: Vec<String>,
    pub keywords: Vec<String>,
}

#[derive(Clone)]
pub struct CodingSuggestion {
    pub code: String,
    pub code_type: CodeType,
    pub description: String,
    pub confidence_bps: u32,
    pub supporting_evidence: Vec<String>,
    pub context: String,
}

pub struct CodingDatabase {
    pub env: Env,
    pub icd10_codes: Map<String, MedicalCode>,
    pub cpt_codes: Map<String, MedicalCode>,
    pub keyword_index: Map<String, Vec<String>>,
}

impl CodingDatabase {
    pub fn new() -> Self {
        let env = soroban_sdk::Env::default();
        Self {
            env,
            icd10_codes: Map::new(&env),
            cpt_codes: Map::new(&env),
            keyword_index: Map::new(&env),
        }
    }

    pub fn add_icd10_code(&mut self, code: MedicalCode) {
        let key = code.code.clone();

        for keyword in code.keywords.iter() {
            let keyword_lower = Self::to_lowercase(&keyword);
            let mut codes = self
                .keyword_index
                .get(keyword_lower.clone())
                .unwrap_or(Vec::new(&self.env));
            codes.push_back(key.clone());
            self.keyword_index.set(keyword_lower, codes);
        }

        self.icd10_codes.set(key, code);
    }

    pub fn add_cpt_code(&mut self, code: MedicalCode) {
        let key = code.code.clone();

        for keyword in code.keywords.iter() {
            let keyword_lower = Self::to_lowercase(&keyword);
            let mut codes = self
                .keyword_index
                .get(keyword_lower.clone())
                .unwrap_or(Vec::new(&self.env));
            codes.push_back(key.clone());
            self.keyword_index.set(keyword_lower, codes);
        }

        self.cpt_codes.set(key, code);
    }

    pub fn suggest_codes(
        &self,
        text: &String,
        code_type: Option<CodeType>,
        max_suggestions: u32,
    ) -> Vec<CodingSuggestion> {
        let mut suggestions = Vec::new(&self.env);

        let keywords = Self::extract_keywords(text);

        let mut scored_codes: Vec<(String, u32, Vec<String>)> = Vec::new(&self.env);

        for keyword in keywords.iter() {
            if let Some(codes) = self.keyword_index.get(keyword.clone()) {
                for code_key in codes.iter() {
                    let mut found = false;
                    for i in 0..scored_codes.len() {
                        let (ref existing_key, ref mut score, ref mut evidence) =
                            scored_codes.get(i).unwrap();
                        if existing_key == &code_key {
                            *score += 100;
                            evidence.push_back(keyword.clone());
                            found = true;
                            break;
                        }
                    }

                    if !found {
                        scored_codes.push_back((
                            code_key,
                            100,
                            Vec::from_array(&self.env, [keyword.clone()]),
                        ));
                    }
                }
            }
        }

        let mut count = 0;
        for (code_key, score, evidence) in scored_codes.iter() {
            if count >= max_suggestions {
                break;
            }

            if let Some(code_type_filter) = code_type {
                if code_type_filter != CodeType::Icd10 && code_type_filter != CodeType::Cpt {
                    continue;
                }
            }

            if let Some(icd10_code) = self.icd10_codes.get(code_key.clone()) {
                if code_type.is_none() || code_type == Some(CodeType::Icd10) {
                    let confidence = if score > 300 {
                        9500
                    } else if score > 200 {
                        8500
                    } else if score > 100 {
                        7500
                    } else {
                        6000
                    };

                    suggestions.push_back(CodingSuggestion {
                        code: icd10_code.code.clone(),
                        code_type: CodeType::Icd10,
                        description: icd10_code.description.clone(),
                        confidence_bps: confidence,
                        supporting_evidence: evidence.clone(),
                        context: Self::extract_context(text, &icd10_code.keywords),
                    });
                    count += 1;
                }
            }

            if let Some(cpt_code) = self.cpt_codes.get(code_key.clone()) {
                if code_type.is_none() || code_type == Some(CodeType::Cpt) {
                    let confidence = if score > 300 {
                        9500
                    } else if score > 200 {
                        8500
                    } else if score > 100 {
                        7500
                    } else {
                        6000
                    };

                    suggestions.push_back(CodingSuggestion {
                        code: cpt_code.code.clone(),
                        code_type: CodeType::Cpt,
                        description: cpt_code.description.clone(),
                        confidence_bps: confidence,
                        supporting_evidence: evidence.clone(),
                        context: Self::extract_context(text, &cpt_code.keywords),
                    });
                    count += 1;
                }
            }
        }

        suggestions
    }

    fn extract_keywords(text: &String) -> Vec<String> {
        let mut keywords = Vec::new(&soroban_sdk::Env::default());
        let len = text.len();
        let mut current_word = Vec::new(&soroban_sdk::Env::default());

        for i in 0..len {
            let ch = text.get(i).unwrap_or(0);

            if (ch >= 48 && ch <= 57) || (ch >= 97 && ch <= 122) {
                current_word.push_back(ch);
            } else if !current_word.is_empty() {
                if current_word.len() >= 3 {
                    let word = String::from_bytes(&soroban_sdk::Env::default(), &current_word);
                    keywords.push_back(word);
                }
                current_word = Vec::new(&soroban_sdk::Env::default());
            }
        }

        if !current_word.is_empty() && current_word.len() >= 3 {
            let word = String::from_bytes(&soroban_sdk::Env::default(), &current_word);
            keywords.push_back(word);
        }

        keywords
    }

    fn extract_context(text: &String, keywords: &Vec<String>) -> String {
        for keyword in keywords.iter() {
            if Self::contains_substring(text, keyword) {
                let mut context_bytes = Vec::new(&soroban_sdk::Env::default());
                let end = if text.len() > 100 { 100 } else { text.len() };
                for i in 0..end {
                    context_bytes.push_back(text.get(i).unwrap_or(0));
                }
                return String::from_bytes(&soroban_sdk::Env::default(), &context_bytes);
            }
        }

        let mut context_bytes = Vec::new(&soroban_sdk::Env::default());
        let end = if text.len() > 100 { 100 } else { text.len() };
        for i in 0..end {
            context_bytes.push_back(text.get(i).unwrap_or(0));
        }
        String::from_bytes(&soroban_sdk::Env::default(), &context_bytes)
    }

    fn contains_substring(text: &String, pattern: &String) -> bool {
        let text_len = text.len();
        let pattern_len = pattern.len();

        if pattern_len > text_len {
            return false;
        }

        for i in 0..=(text_len - pattern_len) {
            let mut found = true;
            for j in 0..pattern_len {
                if text.get(i + j).unwrap_or(0) != pattern.get(j).unwrap_or(0) {
                    found = false;
                    break;
                }
            }
            if found {
                return true;
            }
        }

        false
    }

    fn to_lowercase(s: &String) -> String {
        let env = soroban_sdk::Env::default();
        let len = s.len();
        let mut lower_bytes = Vec::new(&env);

        for i in 0..len {
            let ch = s.get(i).unwrap_or(0);
            if ch >= 65 && ch <= 90 {
                lower_bytes.push_back(ch + 32);
            } else {
                lower_bytes.push_back(ch);
            }
        }

        String::from_bytes(&env, &lower_bytes)
    }
}

impl MedicalCode {
    /// Adapts a canonical `icd_cpt::CatalogCode` into the on-chain shape.
    ///
    /// This is the only place the shared mapping is materialized into host
    /// storage, which is what keeps a second, drifting copy of the data from
    /// appearing in this crate.
    pub fn from_catalog(env: &Env, entry: &icd_cpt::CatalogCode) -> Self {
        let mut related_codes = Vec::new(env);
        for &related in entry.related_codes.iter() {
            related_codes.push_back(String::from_str(env, related));
        }

        let mut keywords = Vec::new(env);
        for &keyword in entry.keywords.iter() {
            keywords.push_back(String::from_str(env, keyword));
        }

        Self {
            code: String::from_str(env, entry.code),
            code_type: entry.code_type,
            description: String::from_str(env, entry.description),
            category: String::from_str(env, entry.category),
            is_billable: entry.is_billable,
            effective_date: entry.effective_date,
            expiration_date: entry.expiration_date,
            related_codes,
            keywords,
        }
    }
}

/// Seeds a database from the canonical `icd_cpt` seed catalog.
pub fn load_default_coding_database(env: &Env) -> CodingDatabase {
    load_coding_database_from(env, icd_cpt::CATALOG)
}

/// Seeds a database from an explicit slice of the canonical catalog, e.g. a
/// diagnosis-only subset.
///
/// Rows whose coding system has no on-chain map are skipped rather than filed
/// under a neighbouring map: the seed catalog carries only ICD-10 and CPT, so
/// this keeps a future HCPCS or SNOMED row from silently masquerading as a
/// procedure code.
pub fn load_coding_database_from(env: &Env, entries: &[icd_cpt::CatalogCode]) -> CodingDatabase {
    let mut db = CodingDatabase {
        env: env.clone(),
        icd10_codes: Map::new(env),
        cpt_codes: Map::new(env),
        keyword_index: Map::new(env),
    };

    for entry in entries {
        let code = MedicalCode::from_catalog(env, entry);
        match entry.code_type {
            CodeType::Icd10 => db.add_icd10_code(code),
            CodeType::Cpt => db.add_cpt_code(code),
            _ => {}
        }
    }

    db
}
