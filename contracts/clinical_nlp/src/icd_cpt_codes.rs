use soroban_sdk::{Env, Map, String, Vec};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CodeType {
    ICD10,
    CPT,
    HCPCS,
    SNOMED,
    LOINC,
}

#[derive(Clone)]
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
                if code_type_filter != CodeType::ICD10 && code_type_filter != CodeType::CPT {
                    continue;
                }
            }

            if let Some(icd10_code) = self.icd10_codes.get(code_key.clone()) {
                if code_type.is_none() || code_type == Some(CodeType::ICD10) {
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
                        code_type: CodeType::ICD10,
                        description: icd10_code.description.clone(),
                        confidence_bps: confidence,
                        supporting_evidence: evidence.clone(),
                        context: Self::extract_context(text, &icd10_code.keywords),
                    });
                    count += 1;
                }
            }

            if let Some(cpt_code) = self.cpt_codes.get(code_key.clone()) {
                if code_type.is_none() || code_type == Some(CodeType::CPT) {
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
                        code_type: CodeType::CPT,
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

            if (48u32..=57).contains(&ch) || (97u32..=122).contains(&ch) {
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
            if (65u32..=90).contains(&ch) {
                lower_bytes.push_back(ch + 32);
            } else {
                lower_bytes.push_back(ch);
            }
        }

        String::from_bytes(&env, &lower_bytes)
    }
}

pub fn load_default_coding_database(env: &Env) -> CodingDatabase {
    let mut db = CodingDatabase {
        env: env.clone(),
        icd10_codes: Map::new(env),
        cpt_codes: Map::new(env),
        keyword_index: Map::new(env),
    };

    let i10 = MedicalCode {
        code: String::from_str(env, "I10"),
        code_type: CodeType::ICD10,
        description: String::from_str(env, "Essential (primary) hypertension"),
        category: String::from_str(env, "Diseases of the circulatory system"),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "I11.9"),
                String::from_str(env, "I12.9"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "hypertension"),
                String::from_str(env, "high blood pressure"),
                String::from_str(env, "htn"),
                String::from_str(env, "elevated blood pressure"),
            ],
        ),
    };
    db.add_icd10_code(i10);

    let e11_9 = MedicalCode {
        code: String::from_str(env, "E11.9"),
        code_type: CodeType::ICD10,
        description: String::from_str(env, "Type 2 diabetes mellitus without complications"),
        category: String::from_str(env, "Endocrine, nutritional and metabolic diseases"),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "E11.65"),
                String::from_str(env, "E11.8"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "diabetes"),
                String::from_str(env, "type 2"),
                String::from_str(env, "t2dm"),
                String::from_str(env, "non-insulin dependent"),
            ],
        ),
    };
    db.add_icd10_code(e11_9);

    let r07_9 = MedicalCode {
        code: String::from_str(env, "R07.9"),
        code_type: CodeType::ICD10,
        description: String::from_str(env, "Chest pain, unspecified"),
        category: String::from_str(
            env,
            "Symptoms, signs and abnormal clinical and laboratory findings",
        ),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "R07.89"),
                String::from_str(env, "R07.0"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "chest pain"),
                String::from_str(env, "thoracic pain"),
                String::from_str(env, "precordial pain"),
            ],
        ),
    };
    db.add_icd10_code(r07_9);

    let r06_00 = MedicalCode {
        code: String::from_str(env, "R06.00"),
        code_type: CodeType::ICD10,
        description: String::from_str(env, "Dyspnea, unspecified"),
        category: String::from_str(
            env,
            "Symptoms, signs and abnormal clinical and laboratory findings",
        ),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "R06.02"),
                String::from_str(env, "R06.09"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "dyspnea"),
                String::from_str(env, "shortness of breath"),
                String::from_str(env, "breathlessness"),
                String::from_str(env, "difficulty breathing"),
                String::from_str(env, "sob"),
            ],
        ),
    };
    db.add_icd10_code(r06_00);

    let cpt_99213 = MedicalCode {
        code: String::from_str(env, "99213"),
        code_type: CodeType::CPT,
        description: String::from_str(env, "Office or other outpatient visit for the evaluation and management of an established patient"),
        category: String::from_str(env, "Evaluation and Management"),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(env, [
            String::from_str(env, "99212"),
            String::from_str(env, "99214"),
        ]),
        keywords: Vec::from_array(env, [
            String::from_str(env, "office visit"),
            String::from_str(env, "outpatient"),
            String::from_str(env, "established patient"),
            String::from_str(env, "evaluation"),
            String::from_str(env, "management"),
        ]),
    };
    db.add_cpt_code(cpt_99213);

    let cpt_93000 = MedicalCode {
        code: String::from_str(env, "93000"),
        code_type: CodeType::CPT,
        description: String::from_str(
            env,
            "Electrocardiogram, routine ECG with at least 12 leads; with interpretation and report",
        ),
        category: String::from_str(env, "Medicine"),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "93005"),
                String::from_str(env, "93010"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "ecg"),
                String::from_str(env, "ekg"),
                String::from_str(env, "electrocardiogram"),
                String::from_str(env, "heart rhythm"),
                String::from_str(env, "cardiac"),
            ],
        ),
    };
    db.add_cpt_code(cpt_93000);

    let cpt_36415 = MedicalCode {
        code: String::from_str(env, "36415"),
        code_type: CodeType::CPT,
        description: String::from_str(env, "Collection of venous blood by venipuncture"),
        category: String::from_str(env, "Surgery"),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(env, [String::from_str(env, "36416")]),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "blood draw"),
                String::from_str(env, "venipuncture"),
                String::from_str(env, "phlebotomy"),
                String::from_str(env, "blood collection"),
            ],
        ),
    };
    db.add_cpt_code(cpt_36415);

    // Additional ICD-10 Codes
    let j45_909 = MedicalCode {
        code: String::from_str(env, "J45.909"),
        code_type: CodeType::ICD10,
        description: String::from_str(env, "Unspecified asthma, uncomplicated"),
        category: String::from_str(env, "Diseases of the respiratory system"),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "J45.901"),
                String::from_str(env, "J45.20"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "asthma"),
                String::from_str(env, "bronchial asthma"),
                String::from_str(env, "reactive airway"),
            ],
        ),
    };
    db.add_icd10_code(j45_909);

    let j18_9 = MedicalCode {
        code: String::from_str(env, "J18.9"),
        code_type: CodeType::ICD10,
        description: String::from_str(env, "Pneumonia, unspecified organism"),
        category: String::from_str(env, "Diseases of the respiratory system"),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "J15.9"),
                String::from_str(env, "J18.1"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "pneumonia"),
                String::from_str(env, "lung infection"),
                String::from_str(env, "pulmonary infection"),
            ],
        ),
    };
    db.add_icd10_code(j18_9);

    let j44_1 = MedicalCode {
        code: String::from_str(env, "J44.1"),
        code_type: CodeType::ICD10,
        description: String::from_str(
            env,
            "Chronic obstructive pulmonary disease with acute exacerbation",
        ),
        category: String::from_str(env, "Diseases of the respiratory system"),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "J44.0"),
                String::from_str(env, "J44.9"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "copd"),
                String::from_str(env, "chronic obstructive pulmonary disease"),
                String::from_str(env, "emphysema"),
                String::from_str(env, "chronic bronchitis"),
            ],
        ),
    };
    db.add_icd10_code(j44_1);

    let i50_9 = MedicalCode {
        code: String::from_str(env, "I50.9"),
        code_type: CodeType::ICD10,
        description: String::from_str(env, "Heart failure, unspecified"),
        category: String::from_str(env, "Diseases of the circulatory system"),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "I50.20"),
                String::from_str(env, "I50.1"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "heart failure"),
                String::from_str(env, "congestive heart failure"),
                String::from_str(env, "chf"),
                String::from_str(env, "cardiac failure"),
            ],
        ),
    };
    db.add_icd10_code(i50_9);

    let i63_9 = MedicalCode {
        code: String::from_str(env, "I63.9"),
        code_type: CodeType::ICD10,
        description: String::from_str(env, "Cerebral infarction, unspecified"),
        category: String::from_str(env, "Diseases of the circulatory system"),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [String::from_str(env, "I64"), String::from_str(env, "I63.5")],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "stroke"),
                String::from_str(env, "cerebrovascular accident"),
                String::from_str(env, "cva"),
                String::from_str(env, "brain attack"),
            ],
        ),
    };
    db.add_icd10_code(i63_9);

    let r50_9 = MedicalCode {
        code: String::from_str(env, "R50.9"),
        code_type: CodeType::ICD10,
        description: String::from_str(env, "Fever, unspecified"),
        category: String::from_str(
            env,
            "Symptoms, signs and abnormal clinical and laboratory findings",
        ),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "R50.81"),
                String::from_str(env, "R50.83"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "fever"),
                String::from_str(env, "pyrexia"),
                String::from_str(env, "febrile"),
                String::from_str(env, "elevated temperature"),
            ],
        ),
    };
    db.add_icd10_code(r50_9);

    let r51_9 = MedicalCode {
        code: String::from_str(env, "R51.9"),
        code_type: CodeType::ICD10,
        description: String::from_str(env, "Headache, unspecified"),
        category: String::from_str(
            env,
            "Symptoms, signs and abnormal clinical and laboratory findings",
        ),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "R51.0"),
                String::from_str(env, "G43.909"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "headache"),
                String::from_str(env, "cephalgia"),
                String::from_str(env, "head pain"),
            ],
        ),
    };
    db.add_icd10_code(r51_9);

    let r11_0 = MedicalCode {
        code: String::from_str(env, "R11.0"),
        code_type: CodeType::ICD10,
        description: String::from_str(env, "Nausea"),
        category: String::from_str(
            env,
            "Symptoms, signs and abnormal clinical and laboratory findings",
        ),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "R11.10"),
                String::from_str(env, "R11.2"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "nausea"),
                String::from_str(env, "queasiness"),
                String::from_str(env, "sickness"),
            ],
        ),
    };
    db.add_icd10_code(r11_0);

    let r53_83 = MedicalCode {
        code: String::from_str(env, "R53.83"),
        code_type: CodeType::ICD10,
        description: String::from_str(env, "Other fatigue"),
        category: String::from_str(
            env,
            "Symptoms, signs and abnormal clinical and laboratory findings",
        ),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "R53.1"),
                String::from_str(env, "R53.81"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "fatigue"),
                String::from_str(env, "tiredness"),
                String::from_str(env, "exhaustion"),
                String::from_str(env, "weakness"),
            ],
        ),
    };
    db.add_icd10_code(r53_83);

    let r42 = MedicalCode {
        code: String::from_str(env, "R42"),
        code_type: CodeType::ICD10,
        description: String::from_str(env, "Dizziness and giddiness"),
        category: String::from_str(
            env,
            "Symptoms, signs and abnormal clinical and laboratory findings",
        ),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "R42"),
                String::from_str(env, "H81.10"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "dizziness"),
                String::from_str(env, "lightheadedness"),
                String::from_str(env, "vertigo"),
            ],
        ),
    };
    db.add_icd10_code(r42);

    let e66_9 = MedicalCode {
        code: String::from_str(env, "E66.9"),
        code_type: CodeType::ICD10,
        description: String::from_str(env, "Obesity, unspecified"),
        category: String::from_str(env, "Endocrine, nutritional and metabolic diseases"),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "E66.01"),
                String::from_str(env, "E66.8"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "obesity"),
                String::from_str(env, "overweight"),
            ],
        ),
    };
    db.add_icd10_code(e66_9);

    let d64_9 = MedicalCode {
        code: String::from_str(env, "D64.9"),
        code_type: CodeType::ICD10,
        description: String::from_str(env, "Anemia, unspecified"),
        category: String::from_str(env, "Diseases of the blood and blood-forming organs"),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "D50.9"),
                String::from_str(env, "D64.81"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "anemia"),
                String::from_str(env, "low hemoglobin"),
            ],
        ),
    };
    db.add_icd10_code(d64_9);

    let b99_9 = MedicalCode {
        code: String::from_str(env, "B99.9"),
        code_type: CodeType::ICD10,
        description: String::from_str(env, "Unspecified infectious disease"),
        category: String::from_str(env, "Certain infectious and parasitic diseases"),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [String::from_str(env, "B99"), String::from_str(env, "A41.9")],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "infection"),
                String::from_str(env, "infectious disease"),
            ],
        ),
    };
    db.add_icd10_code(b99_9);

    let z88_0 = MedicalCode {
        code: String::from_str(env, "Z88.0"),
        code_type: CodeType::ICD10,
        description: String::from_str(env, "Allergy status to penicillin"),
        category: String::from_str(
            env,
            "Factors influencing health status and contact with health services",
        ),
        is_billable: false,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "Z88.1"),
                String::from_str(env, "Z88.8"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "penicillin allergy"),
                String::from_str(env, "pcn allergy"),
            ],
        ),
    };
    db.add_icd10_code(z88_0);

    let z95_0 = MedicalCode {
        code: String::from_str(env, "Z95.0"),
        code_type: CodeType::ICD10,
        description: String::from_str(env, "Presence of cardiac pacemaker"),
        category: String::from_str(
            env,
            "Factors influencing health status and contact with health services",
        ),
        is_billable: false,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "Z95.1"),
                String::from_str(env, "Z95.8"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "pacemaker"),
                String::from_str(env, "cardiac pacemaker"),
            ],
        ),
    };
    db.add_icd10_code(z95_0);

    let z99_11 = MedicalCode {
        code: String::from_str(env, "Z99.11"),
        code_type: CodeType::ICD10,
        description: String::from_str(env, "Dependence on respirator"),
        category: String::from_str(
            env,
            "Factors influencing health status and contact with health services",
        ),
        is_billable: false,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "Z99.12"),
                String::from_str(env, "Z99.81"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "ventilator"),
                String::from_str(env, "mechanical ventilator"),
                String::from_str(env, "respirator"),
            ],
        ),
    };
    db.add_icd10_code(z99_11);

    // Additional CPT Codes
    let cpt_71046 = MedicalCode {
        code: String::from_str(env, "71046"),
        code_type: CodeType::CPT,
        description: String::from_str(env, "Radiologic examination, chest; 2 views"),
        category: String::from_str(env, "Radiology"),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "71045"),
                String::from_str(env, "71047"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "chest x-ray"),
                String::from_str(env, "cxr"),
                String::from_str(env, "radiograph"),
            ],
        ),
    };
    db.add_cpt_code(cpt_71046);

    let cpt_73030 = MedicalCode {
        code: String::from_str(env, "73030"),
        code_type: CodeType::CPT,
        description: String::from_str(env, "Radiologic examination, shoulder; 1 view"),
        category: String::from_str(env, "Radiology"),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "73020"),
                String::from_str(env, "73040"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "shoulder x-ray"),
                String::from_str(env, "shoulder radiograph"),
            ],
        ),
    };
    db.add_cpt_code(cpt_73030);

    let cpt_70553 = MedicalCode {
        code: String::from_str(env, "70553"),
        code_type: CodeType::CPT,
        description: String::from_str(
            env,
            "Magnetic resonance imaging, brain; without contrast, followed by with contrast",
        ),
        category: String::from_str(env, "Radiology"),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "70551"),
                String::from_str(env, "70552"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "mri brain"),
                String::from_str(env, "magnetic resonance imaging"),
            ],
        ),
    };
    db.add_cpt_code(cpt_70553);

    let cpt_72148 = MedicalCode {
        code: String::from_str(env, "72148"),
        code_type: CodeType::CPT,
        description: String::from_str(
            env,
            "Magnetic resonance imaging, lumbar spine; without contrast",
        ),
        category: String::from_str(env, "Radiology"),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "72149"),
                String::from_str(env, "72158"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "mri spine"),
                String::from_str(env, "lumbar mri"),
            ],
        ),
    };
    db.add_cpt_code(cpt_72148);

    let cpt_71260 = MedicalCode {
        code: String::from_str(env, "71260"),
        code_type: CodeType::CPT,
        description: String::from_str(env, "Computed tomography, thorax; with contrast"),
        category: String::from_str(env, "Radiology"),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "71250"),
                String::from_str(env, "71270"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "ct chest"),
                String::from_str(env, "computed tomography"),
                String::from_str(env, "cat scan"),
            ],
        ),
    };
    db.add_cpt_code(cpt_71260);

    let cpt_74178 = MedicalCode {
        code: String::from_str(env, "74178"),
        code_type: CodeType::CPT,
        description: String::from_str(
            env,
            "Computed tomography, abdomen and pelvis; with contrast",
        ),
        category: String::from_str(env, "Radiology"),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "74176"),
                String::from_str(env, "74177"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "ct abdomen"),
                String::from_str(env, "ct pelvis"),
                String::from_str(env, "abdominal ct"),
            ],
        ),
    };
    db.add_cpt_code(cpt_74178);

    let cpt_82947 = MedicalCode {
        code: String::from_str(env, "82947"),
        code_type: CodeType::CPT,
        description: String::from_str(env, "Glucose; quantitative, blood (except reagent strip)"),
        category: String::from_str(env, "Pathology and Laboratory"),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "82948"),
                String::from_str(env, "82950"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "glucose test"),
                String::from_str(env, "blood sugar"),
                String::from_str(env, "fasting glucose"),
            ],
        ),
    };
    db.add_cpt_code(cpt_82947);

    let cpt_80061 = MedicalCode {
        code: String::from_str(env, "80061"),
        code_type: CodeType::CPT,
        description: String::from_str(env, "Lipid panel"),
        category: String::from_str(env, "Pathology and Laboratory"),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "80076"),
                String::from_str(env, "83718"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "cholesterol test"),
                String::from_str(env, "lipid panel"),
                String::from_str(env, "lipid profile"),
            ],
        ),
    };
    db.add_cpt_code(cpt_80061);

    let cpt_83036 = MedicalCode {
        code: String::from_str(env, "83036"),
        code_type: CodeType::CPT,
        description: String::from_str(env, "Hemoglobin; glycosylated (A1C)"),
        category: String::from_str(env, "Pathology and Laboratory"),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "83037"),
                String::from_str(env, "83038"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "hemoglobin a1c"),
                String::from_str(env, "hba1c"),
                String::from_str(env, "a1c"),
                String::from_str(env, "glycated hemoglobin"),
            ],
        ),
    };
    db.add_cpt_code(cpt_83036);

    let cpt_94760 = MedicalCode {
        code: String::from_str(env, "94760"),
        code_type: CodeType::CPT,
        description: String::from_str(
            env,
            "Noninvasive ear or pulse oximetry for oxygen saturation",
        ),
        category: String::from_str(env, "Medicine"),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "94761"),
                String::from_str(env, "94762"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "oxygen saturation"),
                String::from_str(env, "spo2"),
                String::from_str(env, "pulse oximetry"),
            ],
        ),
    };
    db.add_cpt_code(cpt_94760);

    let cpt_33208 = MedicalCode {
        code: String::from_str(env, "33208"),
        code_type: CodeType::CPT,
        description: String::from_str(env, "Insertion of pacemaker, single chamber"),
        category: String::from_str(env, "Surgery"),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "33206"),
                String::from_str(env, "33210"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "pacemaker"),
                String::from_str(env, "cardiac pacemaker"),
            ],
        ),
    };
    db.add_cpt_code(cpt_33208);

    let cpt_94002 = MedicalCode {
        code: String::from_str(env, "94002"),
        code_type: CodeType::CPT,
        description: String::from_str(env, "Ventilation assist and management, initiation of pressure or volume preset ventilator assistance"),
        category: String::from_str(env, "Medicine"),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(env, [
            String::from_str(env, "94003"),
            String::from_str(env, "94004"),
        ]),
        keywords: Vec::from_array(env, [
            String::from_str(env, "ventilator"),
            String::from_str(env, "mechanical ventilator"),
            String::from_str(env, "respirator"),
        ]),
    };
    db.add_cpt_code(cpt_94002);

    let cpt_85025 = MedicalCode {
        code: String::from_str(env, "85025"),
        code_type: CodeType::CPT,
        description: String::from_str(env, "Blood count; complete (CBC), automated"),
        category: String::from_str(env, "Pathology and Laboratory"),
        is_billable: true,
        effective_date: 0,
        expiration_date: None,
        related_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "85027"),
                String::from_str(env, "85048"),
            ],
        ),
        keywords: Vec::from_array(
            env,
            [
                String::from_str(env, "blood count"),
                String::from_str(env, "cbc"),
                String::from_str(env, "complete blood count"),
            ],
        ),
    };
    db.add_cpt_code(cpt_85025);

    db
}
