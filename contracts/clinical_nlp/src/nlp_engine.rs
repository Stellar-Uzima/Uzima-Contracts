use soroban_sdk::{Address, BytesN, Env, String, Vec};

use crate::errors::Error;
use crate::icd_cpt_codes::{CodingDatabase, CodingSuggestion};
use crate::medical_terms::{MedicalTerm, MedicalTermCategory, MedicalTermDatabase};
use crate::sentiment::{SentimentLexicon, SentimentResult};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Spanish,
    French,
    German,
    Portuguese,
}

#[derive(Clone)]
pub struct ExtractedEntity {
    pub entity_type: String,
    pub value: String,
    pub normalized_value: String,
    pub confidence_bps: u32,
    pub start_position: u32,
    pub end_position: u32,
}

#[derive(Clone)]
pub struct ClinicalConcept {
    pub concept_id: String,
    pub concept_name: String,
    pub category: String,
    pub confidence_bps: u32,
    pub related_entities: Vec<String>,
    pub icd10_codes: Vec<String>,
    pub cpt_codes: Vec<String>,
}

#[derive(Clone)]
pub struct NLPResult {
    pub note_id: BytesN<32>,
    pub language: u32,
    pub entities: Vec<ExtractedEntity>,
    pub concepts: Vec<ClinicalConcept>,
    pub sentiment: Option<SentimentResult>,
    pub coding_suggestions: Vec<CodingSuggestion>,
    pub processing_time_ms: u64,
    pub accuracy_score_bps: u32,
    pub word_count: u32,
    pub phi_detected: bool,
}

#[derive(Clone)]
pub struct NLPConfig {
    pub medical_records_contract: Option<Address>,
    pub enable_entity_extraction: bool,
    pub enable_concept_extraction: bool,
    pub enable_sentiment_analysis: bool,
    pub enable_coding_suggestions: bool,
    pub max_processing_time_ms: u64,
    pub supported_languages: Vec<u32>,
    pub phi_detection_enabled: bool,
    pub accuracy_threshold_bps: u32,
}

pub struct NLPEngine {
    pub config: NLPConfig,
    pub medical_terms: MedicalTermDatabase,
    pub coding_db: CodingDatabase,
    pub sentiment_lexicon: SentimentLexicon,
}

impl NLPEngine {
    pub fn new(_env: Env, config: NLPConfig) -> Self {
        Self {
            config,
            medical_terms: MedicalTermDatabase::new(),
            coding_db: CodingDatabase::new(),
            sentiment_lexicon: SentimentLexicon::new(),
        }
    }

    pub fn initialize_defaults(&mut self, env: &Env) {
        self.medical_terms = crate::medical_terms::load_default_medical_terms(env);
        self.coding_db = crate::icd_cpt_codes::load_default_coding_database(env);
        self.sentiment_lexicon = crate::sentiment::load_default_sentiment_lexicon(env);
    }

    pub fn process_clinical_note(
        &self,
        env: &Env,
        note_text: &String,
        note_id: &BytesN<32>,
        language: u32,
    ) -> Result<NLPResult, Error> {
        let start_time = env.ledger().timestamp();

        if note_text.is_empty() {
            return Err(Error::EmptyClinicalNote);
        }

        if note_text.len() > 100000 {
            return Err(Error::InputTooLarge);
        }

        let detected_language = language;

        if !self.is_language_supported(detected_language) {
            return Err(Error::InvalidLanguageCode);
        }

        let entities = if self.config.enable_entity_extraction {
            self.extract_entities(note_text)?
        } else {
            Vec::new(env)
        };

        let concepts = if self.config.enable_concept_extraction {
            self.extract_concepts(env, note_text, &entities)?
        } else {
            Vec::new(env)
        };

        let sentiment = if self.config.enable_sentiment_analysis {
            Some(self.analyze_sentiment(note_text))
        } else {
            None
        };

        let coding_suggestions = if self.config.enable_coding_suggestions {
            self.generate_coding_suggestions(note_text, 10)?
        } else {
            Vec::new(env)
        };

        let phi_detected = if self.config.phi_detection_enabled {
            self.detect_phi(note_text, &entities)
        } else {
            false
        };

        let end_time = env.ledger().timestamp();
        let processing_time_ms = (end_time - start_time) * 1000;

        if processing_time_ms > self.config.max_processing_time_ms {
            return Err(Error::ProcessingTimeExceeded);
        }

        let accuracy_score_bps = self.calculate_accuracy_score(&entities, &concepts);
        let word_count = self.count_words(note_text);

        Ok(NLPResult {
            note_id: note_id.clone(),
            language: detected_language,
            entities,
            concepts,
            sentiment,
            coding_suggestions,
            processing_time_ms,
            accuracy_score_bps,
            word_count,
            phi_detected,
        })
    }

    pub fn extract_entities(&self, text: &String) -> Result<Vec<ExtractedEntity>, Error> {
        let mut entities = Vec::new(&self.medical_terms.env);

        for term in self.medical_terms.terms.values() {
            if Self::contains_substring(text, &term.term) {
                let entity = ExtractedEntity {
                    entity_type: Self::category_to_string(&self.medical_terms.env, term.category),
                    value: term.term.clone(),
                    normalized_value: term.normalized_form.clone(),
                    confidence_bps: 9000,
                    start_position: 0,
                    end_position: term.term.len(),
                };
                entities.push_back(entity);
            }

            for syn in term.synonyms.iter() {
                if Self::contains_substring(text, &syn) {
                    let entity = ExtractedEntity {
                        entity_type: Self::category_to_string(
                            &self.medical_terms.env,
                            term.category,
                        ),
                        value: syn.clone(),
                        normalized_value: term.normalized_form.clone(),
                        confidence_bps: 8500,
                        start_position: 0,
                        end_position: syn.len(),
                    };
                    entities.push_back(entity);
                }
            }

            for abbr in term.abbreviations.iter() {
                if Self::contains_substring(text, &abbr) {
                    let entity = ExtractedEntity {
                        entity_type: Self::category_to_string(
                            &self.medical_terms.env,
                            term.category,
                        ),
                        value: abbr.clone(),
                        normalized_value: term.normalized_form.clone(),
                        confidence_bps: 8000,
                        start_position: 0,
                        end_position: abbr.len(),
                    };
                    entities.push_back(entity);
                }
            }
        }

        Ok(entities)
    }

    pub fn extract_concepts(
        &self,
        env: &Env,
        _text: &String,
        entities: &Vec<ExtractedEntity>,
    ) -> Result<Vec<ClinicalConcept>, Error> {
        let mut concepts = Vec::new(env);
        let mut processed_terms = Vec::new(env);

        for entity in entities.iter() {
            let mut already_processed = false;
            for processed in processed_terms.iter() {
                if processed == entity.normalized_value {
                    already_processed = true;
                    break;
                }
            }
            if already_processed {
                continue;
            }

            if let Some(term) = self.medical_terms.lookup_term(&entity.normalized_value) {
                let concept = ClinicalConcept {
                    concept_id: Self::generate_concept_id(env, &term.normalized_form),
                    concept_name: term.term.clone(),
                    category: Self::category_to_string(env, term.category),
                    confidence_bps: entity.confidence_bps,
                    related_entities: Vec::from_array(env, [entity.value.clone()]),
                    icd10_codes: term.icd10_codes.clone(),
                    cpt_codes: term.cpt_codes.clone(),
                };
                concepts.push_back(concept);
                processed_terms.push_back(entity.normalized_value.clone());
            }
        }

        Ok(concepts)
    }

    pub fn analyze_sentiment(&self, text: &String) -> SentimentResult {
        self.sentiment_lexicon.analyze_sentiment(text)
    }

    pub fn generate_coding_suggestions(
        &self,
        text: &String,
        max_suggestions: u32,
    ) -> Result<Vec<CodingSuggestion>, Error> {
        let suggestions = self.coding_db.suggest_codes(text, None, max_suggestions);
        Ok(suggestions)
    }

    pub fn detect_phi(&self, text: &String, entities: &Vec<ExtractedEntity>) -> bool {
        for entity in entities.iter() {
            if let Some(term) = self.medical_terms.lookup_term(&entity.normalized_value) {
                if term.is_phi {
                    return true;
                }
            }
        }

        let phi_patterns = [
            "ssn",
            "social security",
            "date of birth",
            "dob",
            "address",
            "phone",
            "email",
            "medical record",
            "mrn",
            "patient id",
        ];

        for pattern in phi_patterns {
            let pattern_string = String::from_str(&self.medical_terms.env, pattern);
            if Self::contains_substring(text, &pattern_string) {
                return true;
            }
        }

        false
    }

    fn is_language_supported(&self, language: u32) -> bool {
        for supported in self.config.supported_languages.iter() {
            if supported == language {
                return true;
            }
        }
        false
    }

    fn calculate_accuracy_score(
        &self,
        entities: &Vec<ExtractedEntity>,
        concepts: &Vec<ClinicalConcept>,
    ) -> u32 {
        if entities.is_empty() && concepts.is_empty() {
            return 0;
        }

        let mut total_confidence: u64 = 0;
        let mut count: u64 = 0;

        for entity in entities.iter() {
            total_confidence += entity.confidence_bps as u64;
            count += 1;
        }

        for concept in concepts.iter() {
            total_confidence += concept.confidence_bps as u64;
            count += 1;
        }

        if count > 0 {
            (total_confidence / count) as u32
        } else {
            0
        }
    }

    fn count_words(&self, text: &String) -> u32 {
        let mut word_count = 0;
        let mut in_word = false;
        let len = text.len();

        for i in 0..len {
            let ch = text.get(i).unwrap_or(0);
            let is_whitespace = ch == 32 || ch == 9 || ch == 10 || ch == 13;

            if !is_whitespace && !in_word {
                word_count += 1;
                in_word = true;
            } else if is_whitespace {
                in_word = false;
            }
        }

        word_count
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

    fn category_to_string(env: &Env, category: MedicalTermCategory) -> String {
        match category {
            MedicalTermCategory::Diagnosis => String::from_str(env, "Diagnosis"),
            MedicalTermCategory::Symptom => String::from_str(env, "Symptom"),
            MedicalTermCategory::Procedure => String::from_str(env, "Procedure"),
            MedicalTermCategory::Medication => String::from_str(env, "Medication"),
            MedicalTermCategory::Anatomy => String::from_str(env, "Anatomy"),
            MedicalTermCategory::LabTest => String::from_str(env, "LabTest"),
            MedicalTermCategory::VitalSign => String::from_str(env, "VitalSign"),
            MedicalTermCategory::MedicalDevice => String::from_str(env, "MedicalDevice"),
            MedicalTermCategory::Allergy => String::from_str(env, "Allergy"),
            MedicalTermCategory::Condition => String::from_str(env, "Condition"),
        }
    }

    fn generate_concept_id(env: &Env, normalized_form: &String) -> String {
        let len = normalized_form.len();
        let mut hash: u64 = 5381;

        for i in 0..len {
            let ch = normalized_form.get(i).unwrap_or(0) as u64;
            hash = ((hash << 5).wrapping_add(hash)).wrapping_add(ch);
        }

        let mut hex_string = String::from_str(env, "");
        let hex_chars = "0123456789abcdef";

        for i in 0..16 {
            let nibble = ((hash >> (i * 4)) & 0xf) as usize;
            let hex_char = hex_chars.as_bytes()[nibble];
            hex_string.push(hex_char);
        }

        hex_string
    }

    pub fn detect_language(&self, text: &String) -> u32 {
        // Simple language detection based on common words
        let text_lower = Self::to_lowercase(text);

        // Spanish indicators
        let spanish_words = [
            "el", "la", "los", "las", "un", "una", "es", "son", "está", "están", "paciente",
            "doctor", "hospital",
        ];
        for word in spanish_words {
            let word_string = String::from_str(&self.medical_terms.env, word);
            if Self::contains_substring(&text_lower, &word_string) {
                return 1; // Spanish
            }
        }

        // French indicators
        let french_words = [
            "le", "la", "les", "un", "une", "est", "sont", "patient", "médecin", "hôpital",
        ];
        for word in french_words {
            let word_string = String::from_str(&self.medical_terms.env, word);
            if Self::contains_substring(&text_lower, &word_string) {
                return 2; // French
            }
        }

        // German indicators
        let german_words = [
            "der",
            "die",
            "das",
            "ein",
            "eine",
            "ist",
            "sind",
            "patient",
            "arzt",
            "krankenhaus",
        ];
        for word in german_words {
            let word_string = String::from_str(&self.medical_terms.env, word);
            if Self::contains_substring(&text_lower, &word_string) {
                return 3; // German
            }
        }

        // Portuguese indicators
        let portuguese_words = [
            "o", "a", "os", "as", "um", "uma", "é", "são", "paciente", "médico", "hospital",
        ];
        for word in portuguese_words {
            let word_string = String::from_str(&self.medical_terms.env, word);
            if Self::contains_substring(&text_lower, &word_string) {
                return 4; // Portuguese
            }
        }

        // Default to English
        0
    }

    pub fn get_language_name(&self, language_code: u32) -> String {
        match language_code {
            0 => String::from_str(&self.medical_terms.env, "English"),
            1 => String::from_str(&self.medical_terms.env, "Spanish"),
            2 => String::from_str(&self.medical_terms.env, "French"),
            3 => String::from_str(&self.medical_terms.env, "German"),
            4 => String::from_str(&self.medical_terms.env, "Portuguese"),
            _ => String::from_str(&self.medical_terms.env, "Unknown"),
        }
    }
}
