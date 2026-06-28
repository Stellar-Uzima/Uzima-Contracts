use soroban_sdk::{Env, Map, String, Vec};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MedicalTermCategory {
    Diagnosis,
    Symptom,
    Procedure,
    Medication,
    Anatomy,
    LabTest,
    VitalSign,
    MedicalDevice,
    Allergy,
    Condition,
}

#[derive(Clone)]
pub struct MedicalTerm {
    pub term: String,
    pub normalized_form: String,
    pub category: MedicalTermCategory,
    pub synonyms: Vec<String>,
    pub abbreviations: Vec<String>,
    pub icd10_codes: Vec<String>,
    pub cpt_codes: Vec<String>,
    pub snomed_code: Option<String>,
    pub loinc_code: Option<String>,
    pub description: String,
    pub is_phi: bool,
}

pub struct MedicalTermDatabase {
    pub env: Env,
    pub terms: Map<String, MedicalTerm>,
    pub abbreviations: Map<String, String>,
    pub synonyms: Map<String, String>,
}

impl MedicalTermDatabase {
    pub fn new() -> Self {
        let env = soroban_sdk::Env::default();
        Self {
            env,
            terms: Map::new(&env),
            abbreviations: Map::new(&env),
            synonyms: Map::new(&env),
        }
    }

    pub fn add_term(&mut self, term: MedicalTerm) {
        let key = term.normalized_form.clone();
        self.terms.set(key.clone(), term.clone());

        for abbr in term.abbreviations.iter() {
            self.abbreviations.set(abbr, key.clone());
        }

        for syn in term.synonyms.iter() {
            self.synonyms.set(syn, key.clone());
        }
    }

    pub fn lookup_term(&self, term: &String) -> Option<MedicalTerm> {
        if let Some(found) = self.terms.get(term.clone()) {
            return Some(found);
        }

        if let Some(abbr_key) = self.abbreviations.get(term.clone()) {
            return self.terms.get(abbr_key);
        }

        if let Some(syn_key) = self.synonyms.get(term.clone()) {
            return self.terms.get(syn_key);
        }

        None
    }

    pub fn search_terms(
        &self,
        query: &String,
        category: Option<MedicalTermCategory>,
    ) -> Vec<MedicalTerm> {
        let mut results = Vec::new(&self.env);

        for term in self.terms.values() {
            if let Some(cat) = category {
                if term.category != cat {
                    continue;
                }
            }

            if Self::contains_substring(&term.term, query)
                || Self::contains_substring(&term.normalized_form, query)
            {
                results.push_back(term.clone());
                continue;
            }

            for syn in term.synonyms.iter() {
                if Self::contains_substring(&syn, query) {
                    results.push_back(term.clone());
                    break;
                }
            }
        }

        results
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
}

pub fn load_default_medical_terms(env: &Env) -> MedicalTermDatabase {
    let mut db = MedicalTermDatabase {
        env: env.clone(),
        terms: Map::new(env),
        abbreviations: Map::new(env),
        synonyms: Map::new(env),
    };

    let hypertension = MedicalTerm {
        term: String::from_str(env, "Hypertension"),
        normalized_form: String::from_str(env, "hypertension"),
        category: MedicalTermCategory::Diagnosis,
        synonyms: Vec::from_array(
            env,
            [
                String::from_str(env, "High Blood Pressure"),
                String::from_str(env, "HTN"),
            ],
        ),
        abbreviations: Vec::from_array(
            env,
            [String::from_str(env, "HTN"), String::from_str(env, "HBP")],
        ),
        icd10_codes: Vec::from_array(
            env,
            [String::from_str(env, "I10"), String::from_str(env, "I11.9")],
        ),
        cpt_codes: Vec::new(env),
        snomed_code: Some(String::from_str(env, "38341003")),
        loinc_code: Some(String::from_str(env, "85354-9")),
        description: String::from_str(env, "Persistently high arterial blood pressure"),
        is_phi: false,
    };
    db.add_term(hypertension);

    let diabetes = MedicalTerm {
        term: String::from_str(env, "Diabetes Mellitus"),
        normalized_form: String::from_str(env, "diabetes mellitus"),
        category: MedicalTermCategory::Diagnosis,
        synonyms: Vec::from_array(
            env,
            [
                String::from_str(env, "Diabetes"),
                String::from_str(env, "DM"),
            ],
        ),
        abbreviations: Vec::from_array(
            env,
            [
                String::from_str(env, "DM"),
                String::from_str(env, "T2DM"),
                String::from_str(env, "T1DM"),
            ],
        ),
        icd10_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "E11.9"),
                String::from_str(env, "E10.9"),
            ],
        ),
        cpt_codes: Vec::new(env),
        snomed_code: Some(String::from_str(env, "73211009")),
        loinc_code: Some(String::from_str(env, "4548-4")),
        description: String::from_str(
            env,
            "Group of metabolic disorders characterized by high blood sugar",
        ),
        is_phi: false,
    };
    db.add_term(diabetes);

    let chest_pain = MedicalTerm {
        term: String::from_str(env, "Chest Pain"),
        normalized_form: String::from_str(env, "chest pain"),
        category: MedicalTermCategory::Symptom,
        synonyms: Vec::from_array(
            env,
            [
                String::from_str(env, "Thoracic Pain"),
                String::from_str(env, "Precordial Pain"),
            ],
        ),
        abbreviations: Vec::from_array(env, [String::from_str(env, "CP")]),
        icd10_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "R07.9"),
                String::from_str(env, "R07.89"),
            ],
        ),
        cpt_codes: Vec::new(env),
        snomed_code: Some(String::from_str(env, "29857009")),
        loinc_code: None,
        description: String::from_str(env, "Pain or discomfort in the chest area"),
        is_phi: false,
    };
    db.add_term(chest_pain);

    let dyspnea = MedicalTerm {
        term: String::from_str(env, "Dyspnea"),
        normalized_form: String::from_str(env, "dyspnea"),
        category: MedicalTermCategory::Symptom,
        synonyms: Vec::from_array(
            env,
            [
                String::from_str(env, "Shortness of Breath"),
                String::from_str(env, "Breathlessness"),
                String::from_str(env, "Difficulty Breathing"),
            ],
        ),
        abbreviations: Vec::from_array(env, [String::from_str(env, "SOB")]),
        icd10_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "R06.00"),
                String::from_str(env, "R06.02"),
            ],
        ),
        cpt_codes: Vec::new(env),
        snomed_code: Some(String::from_str(env, "267036007")),
        loinc_code: None,
        description: String::from_str(env, "Subjective sensation of difficulty breathing"),
        is_phi: false,
    };
    db.add_term(dyspnea);

    let appendectomy = MedicalTerm {
        term: String::from_str(env, "Appendectomy"),
        normalized_form: String::from_str(env, "appendectomy"),
        category: MedicalTermCategory::Procedure,
        synonyms: Vec::from_array(env, [String::from_str(env, "Appendix Removal")]),
        abbreviations: Vec::new(env),
        icd10_codes: Vec::from_array(env, [String::from_str(env, "0DTJ4ZZ")]),
        cpt_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "44970"),
                String::from_str(env, "44950"),
            ],
        ),
        snomed_code: Some(String::from_str(env, "80146002")),
        loinc_code: None,
        description: String::from_str(env, "Surgical removal of the appendix"),
        is_phi: false,
    };
    db.add_term(appendectomy);

    let aspirin = MedicalTerm {
        term: String::from_str(env, "Aspirin"),
        normalized_form: String::from_str(env, "aspirin"),
        category: MedicalTermCategory::Medication,
        synonyms: Vec::from_array(
            env,
            [
                String::from_str(env, "Acetylsalicylic Acid"),
                String::from_str(env, "ASA"),
            ],
        ),
        abbreviations: Vec::from_array(env, [String::from_str(env, "ASA")]),
        icd10_codes: Vec::new(env),
        cpt_codes: Vec::new(env),
        snomed_code: Some(String::from_str(env, "387458008")),
        loinc_code: None,
        description: String::from_str(env, "Nonsteroidal anti-inflammatory drug (NSAID)"),
        is_phi: false,
    };
    db.add_term(aspirin);

    let blood_pressure = MedicalTerm {
        term: String::from_str(env, "Blood Pressure"),
        normalized_form: String::from_str(env, "blood pressure"),
        category: MedicalTermCategory::VitalSign,
        synonyms: Vec::from_array(
            env,
            [
                String::from_str(env, "BP"),
                String::from_str(env, "Arterial Pressure"),
            ],
        ),
        abbreviations: Vec::from_array(env, [String::from_str(env, "BP")]),
        icd10_codes: Vec::new(env),
        cpt_codes: Vec::from_array(env, [String::from_str(env, "93000")]),
        snomed_code: Some(String::from_str(env, "75367002")),
        loinc_code: Some(String::from_str(env, "85354-9")),
        description: String::from_str(env, "Force of blood against artery walls"),
        is_phi: false,
    };
    db.add_term(blood_pressure);

    let heart_rate = MedicalTerm {
        term: String::from_str(env, "Heart Rate"),
        normalized_form: String::from_str(env, "heart rate"),
        category: MedicalTermCategory::VitalSign,
        synonyms: Vec::from_array(
            env,
            [String::from_str(env, "Pulse"), String::from_str(env, "HR")],
        ),
        abbreviations: Vec::from_array(
            env,
            [String::from_str(env, "HR"), String::from_str(env, "PR")],
        ),
        icd10_codes: Vec::new(env),
        cpt_codes: Vec::from_array(env, [String::from_str(env, "93000")]),
        snomed_code: Some(String::from_str(env, "364075005")),
        loinc_code: Some(String::from_str(env, "8867-4")),
        description: String::from_str(env, "Number of heartbeats per minute"),
        is_phi: false,
    };
    db.add_term(heart_rate);

    // Additional Diagnosis Terms
    let asthma = MedicalTerm {
        term: String::from_str(env, "Asthma"),
        normalized_form: String::from_str(env, "asthma"),
        category: MedicalTermCategory::Diagnosis,
        synonyms: Vec::from_array(
            env,
            [
                String::from_str(env, "Bronchial Asthma"),
                String::from_str(env, "Reactive Airway Disease"),
            ],
        ),
        abbreviations: Vec::from_array(env, [String::from_str(env, "RAD")]),
        icd10_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "J45.909"),
                String::from_str(env, "J45.901"),
            ],
        ),
        cpt_codes: Vec::new(env),
        snomed_code: Some(String::from_str(env, "195967001")),
        loinc_code: None,
        description: String::from_str(env, "Chronic inflammatory disease of the airways"),
        is_phi: false,
    };
    db.add_term(asthma);

    let pneumonia = MedicalTerm {
        term: String::from_str(env, "Pneumonia"),
        normalized_form: String::from_str(env, "pneumonia"),
        category: MedicalTermCategory::Diagnosis,
        synonyms: Vec::from_array(
            env,
            [
                String::from_str(env, "Lung Infection"),
                String::from_str(env, "Pulmonary Infection"),
            ],
        ),
        abbreviations: Vec::from_array(env, [String::from_str(env, "CAP")]),
        icd10_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "J18.9"),
                String::from_str(env, "J15.9"),
            ],
        ),
        cpt_codes: Vec::new(env),
        snomed_code: Some(String::from_str(env, "233604007")),
        loinc_code: None,
        description: String::from_str(env, "Infection that inflames air sacs in one or both lungs"),
        is_phi: false,
    };
    db.add_term(pneumonia);

    let copd = MedicalTerm {
        term: String::from_str(env, "Chronic Obstructive Pulmonary Disease"),
        normalized_form: String::from_str(env, "chronic obstructive pulmonary disease"),
        category: MedicalTermCategory::Diagnosis,
        synonyms: Vec::from_array(
            env,
            [
                String::from_str(env, "COPD"),
                String::from_str(env, "Emphysema"),
                String::from_str(env, "Chronic Bronchitis"),
            ],
        ),
        abbreviations: Vec::from_array(env, [String::from_str(env, "COPD")]),
        icd10_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "J44.1"),
                String::from_str(env, "J44.0"),
            ],
        ),
        cpt_codes: Vec::new(env),
        snomed_code: Some(String::from_str(env, "13645005")),
        loinc_code: None,
        description: String::from_str(
            env,
            "Chic inflammatory lung disease causing obstructed airflow",
        ),
        is_phi: false,
    };
    db.add_term(copd);

    let heart_failure = MedicalTerm {
        term: String::from_str(env, "Heart Failure"),
        normalized_form: String::from_str(env, "heart failure"),
        category: MedicalTermCategory::Diagnosis,
        synonyms: Vec::from_array(
            env,
            [
                String::from_str(env, "Congestive Heart Failure"),
                String::from_str(env, "CHF"),
                String::from_str(env, "Cardiac Failure"),
            ],
        ),
        abbreviations: Vec::from_array(
            env,
            [String::from_str(env, "CHF"), String::from_str(env, "HF")],
        ),
        icd10_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "I50.9"),
                String::from_str(env, "I50.20"),
            ],
        ),
        cpt_codes: Vec::new(env),
        snomed_code: Some(String::from_str(env, "84114007")),
        loinc_code: None,
        description: String::from_str(
            env,
            "Condition in which the heart can't pump blood effectively",
        ),
        is_phi: false,
    };
    db.add_term(heart_failure);

    let stroke = MedicalTerm {
        term: String::from_str(env, "Stroke"),
        normalized_form: String::from_str(env, "stroke"),
        category: MedicalTermCategory::Diagnosis,
        synonyms: Vec::from_array(
            env,
            [
                String::from_str(env, "Cerebrovascular Accident"),
                String::from_str(env, "CVA"),
                String::from_str(env, "Brain Attack"),
            ],
        ),
        abbreviations: Vec::from_array(env, [String::from_str(env, "CVA")]),
        icd10_codes: Vec::from_array(
            env,
            [String::from_str(env, "I63.9"), String::from_str(env, "I64")],
        ),
        cpt_codes: Vec::new(env),
        snomed_code: Some(String::from_str(env, "230690007")),
        loinc_code: None,
        description: String::from_str(env, "Interruption of blood supply to the brain"),
        is_phi: false,
    };
    db.add_term(stroke);

    // Additional Symptom Terms
    let fever = MedicalTerm {
        term: String::from_str(env, "Fever"),
        normalized_form: String::from_str(env, "fever"),
        category: MedicalTermCategory::Symptom,
        synonyms: Vec::from_array(
            env,
            [
                String::from_str(env, "Pyrexia"),
                String::from_str(env, "Febrile"),
                String::from_str(env, "Elevated Temperature"),
            ],
        ),
        abbreviations: Vec::from_array(env, [String::from_str(env, "Temp")]),
        icd10_codes: Vec::from_array(env, [String::from_str(env, "R50.9")]),
        cpt_codes: Vec::new(env),
        snomed_code: Some(String::from_str(env, "386661006")),
        loinc_code: None,
        description: String::from_str(env, "Body temperature above normal range"),
        is_phi: false,
    };
    db.add_term(fever);

    let headache = MedicalTerm {
        term: String::from_str(env, "Headache"),
        normalized_form: String::from_str(env, "headache"),
        category: MedicalTermCategory::Symptom,
        synonyms: Vec::from_array(
            env,
            [
                String::from_str(env, "Cephalgia"),
                String::from_str(env, "Head Pain"),
            ],
        ),
        abbreviations: Vec::from_array(env, [String::from_str(env, "HA")]),
        icd10_codes: Vec::from_array(env, [String::from_str(env, "R51.9")]),
        cpt_codes: Vec::new(env),
        snomed_code: Some(String::from_str(env, "25064002")),
        loinc_code: None,
        description: String::from_str(env, "Pain in any region of the head"),
        is_phi: false,
    };
    db.add_term(headache);

    let nausea = MedicalTerm {
        term: String::from_str(env, "Nausea"),
        normalized_form: String::from_str(env, "nausea"),
        category: MedicalTermCategory::Symptom,
        synonyms: Vec::from_array(
            env,
            [
                String::from_str(env, "Queasiness"),
                String::from_str(env, "Sickness"),
            ],
        ),
        abbreviations: Vec::new(env),
        icd10_codes: Vec::from_array(env, [String::from_str(env, "R11.0")]),
        cpt_codes: Vec::new(env),
        snomed_code: Some(String::from_str(env, "422587007")),
        loinc_code: None,
        description: String::from_str(
            env,
            "Sensation of unease and discomfort in the stomach with an urge to vomit",
        ),
        is_phi: false,
    };
    db.add_term(nausea);

    let fatigue = MedicalTerm {
        term: String::from_str(env, "Fatigue"),
        normalized_form: String::from_str(env, "fatigue"),
        category: MedicalTermCategory::Symptom,
        synonyms: Vec::from_array(
            env,
            [
                String::from_str(env, "Tiredness"),
                String::from_str(env, "Exhaustion"),
                String::from_str(env, "Weakness"),
            ],
        ),
        abbreviations: Vec::new(env),
        icd10_codes: Vec::from_array(env, [String::from_str(env, "R53.83")]),
        cpt_codes: Vec::new(env),
        snomed_code: Some(String::from_str(env, "84229001")),
        loinc_code: None,
        description: String::from_str(env, "State of extreme tiredness or exhaustion"),
        is_phi: false,
    };
    db.add_term(fatigue);

    let dizziness = MedicalTerm {
        term: String::from_str(env, "Dizziness"),
        normalized_form: String::from_str(env, "dizziness"),
        category: MedicalTermCategory::Symptom,
        synonyms: Vec::from_array(
            env,
            [
                String::from_str(env, "Lightheadedness"),
                String::from_str(env, "Vertigo"),
            ],
        ),
        abbreviations: Vec::new(env),
        icd10_codes: Vec::from_array(env, [String::from_str(env, "R42")]),
        cpt_codes: Vec::new(env),
        snomed_code: Some(String::from_str(env, "404640003")),
        loinc_code: None,
        description: String::from_str(env, "Feeling of being unbalanced or lightheaded"),
        is_phi: false,
    };
    db.add_term(dizziness);

    // Additional Procedure Terms
    let blood_test = MedicalTerm {
        term: String::from_str(env, "Blood Test"),
        normalized_form: String::from_str(env, "blood test"),
        category: MedicalTermCategory::Procedure,
        synonyms: Vec::from_array(
            env,
            [
                String::from_str(env, "Blood Work"),
                String::from_str(env, "Laboratory Test"),
            ],
        ),
        abbreviations: Vec::from_array(env, [String::from_str(env, "Lab")]),
        icd10_codes: Vec::new(env),
        cpt_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "36415"),
                String::from_str(env, "85025"),
            ],
        ),
        snomed_code: Some(String::from_str(env, "396667008")),
        loinc_code: None,
        description: String::from_str(env, "Laboratory analysis of a blood sample"),
        is_phi: false,
    };
    db.add_term(blood_test);

    let xray = MedicalTerm {
        term: String::from_str(env, "X-Ray"),
        normalized_form: String::from_str(env, "x-ray"),
        category: MedicalTermCategory::Procedure,
        synonyms: Vec::from_array(
            env,
            [
                String::from_str(env, "Radiograph"),
                String::from_str(env, "Radiography"),
            ],
        ),
        abbreviations: Vec::from_array(env, [String::from_str(env, "XR")]),
        icd10_codes: Vec::new(env),
        cpt_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "71046"),
                String::from_str(env, "73030"),
            ],
        ),
        snomed_code: Some(String::from_str(env, "363680008")),
        loinc_code: None,
        description: String::from_str(env, "Imaging test using electromagnetic radiation"),
        is_phi: false,
    };
    db.add_term(xray);

    let mri = MedicalTerm {
        term: String::from_str(env, "MRI"),
        normalized_form: String::from_str(env, "mri"),
        category: MedicalTermCategory::Procedure,
        synonyms: Vec::from_array(env, [String::from_str(env, "Magnetic Resonance Imaging")]),
        abbreviations: Vec::from_array(env, [String::from_str(env, "MRI")]),
        icd10_codes: Vec::new(env),
        cpt_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "70553"),
                String::from_str(env, "72148"),
            ],
        ),
        snomed_code: Some(String::from_str(env, "113091000")),
        loinc_code: None,
        description: String::from_str(
            env,
            "Imaging technique using magnetic fields and radio waves",
        ),
        is_phi: false,
    };
    db.add_term(mri);

    let ct_scan = MedicalTerm {
        term: String::from_str(env, "CT Scan"),
        normalized_form: String::from_str(env, "ct scan"),
        category: MedicalTermCategory::Procedure,
        synonyms: Vec::from_array(
            env,
            [
                String::from_str(env, "Computed Tomography"),
                String::from_str(env, "CAT Scan"),
            ],
        ),
        abbreviations: Vec::from_array(env, [String::from_str(env, "CT")]),
        icd10_codes: Vec::new(env),
        cpt_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "71260"),
                String::from_str(env, "74178"),
            ],
        ),
        snomed_code: Some(String::from_str(env, "77477000")),
        loinc_code: None,
        description: String::from_str(env, "Imaging test using X-rays and computer processing"),
        is_phi: false,
    };
    db.add_term(ct_scan);

    // Additional Medication Terms
    let ibuprofen = MedicalTerm {
        term: String::from_str(env, "Ibuprofen"),
        normalized_form: String::from_str(env, "ibuprofen"),
        category: MedicalTermCategory::Medication,
        synonyms: Vec::from_array(
            env,
            [
                String::from_str(env, "Advil"),
                String::from_str(env, "Motrin"),
            ],
        ),
        abbreviations: Vec::from_array(env, [String::from_str(env, "IBU")]),
        icd10_codes: Vec::new(env),
        cpt_codes: Vec::new(env),
        snomed_code: Some(String::from_str(env, "387207008")),
        loinc_code: None,
        description: String::from_str(env, "Nonsteroidal anti-inflammatory drug (NSAID)"),
        is_phi: false,
    };
    db.add_term(ibuprofen);

    let acetaminophen = MedicalTerm {
        term: String::from_str(env, "Acetaminophen"),
        normalized_form: String::from_str(env, "acetaminophen"),
        category: MedicalTermCategory::Medication,
        synonyms: Vec::from_array(
            env,
            [
                String::from_str(env, "Tylenol"),
                String::from_str(env, "Paracetamol"),
            ],
        ),
        abbreviations: Vec::from_array(env, [String::from_str(env, "APAP")]),
        icd10_codes: Vec::new(env),
        cpt_codes: Vec::new(env),
        snomed_code: Some(String::from_str(env, "387518008")),
        loinc_code: None,
        description: String::from_str(env, "Pain reliever and fever reducer"),
        is_phi: false,
    };
    db.add_term(acetaminophen);

    let metformin = MedicalTerm {
        term: String::from_str(env, "Metformin"),
        normalized_form: String::from_str(env, "metformin"),
        category: MedicalTermCategory::Medication,
        synonyms: Vec::from_array(env, [String::from_str(env, "Glucophage")]),
        abbreviations: Vec::new(env),
        icd10_codes: Vec::new(env),
        cpt_codes: Vec::new(env),
        snomed_code: Some(String::from_str(env, "387326008")),
        loinc_code: None,
        description: String::from_str(env, "Oral diabetes medication"),
        is_phi: false,
    };
    db.add_term(metformin);

    let lisinopril = MedicalTerm {
        term: String::from_str(env, "Lisinopril"),
        normalized_form: String::from_str(env, "lisinopril"),
        category: MedicalTermCategory::Medication,
        synonyms: Vec::from_array(
            env,
            [
                String::from_str(env, "Prinivil"),
                String::from_str(env, "Zestril"),
            ],
        ),
        abbreviations: Vec::new(env),
        icd10_codes: Vec::new(env),
        cpt_codes: Vec::new(env),
        snomed_code: Some(String::from_str(env, "386862002")),
        loinc_code: None,
        description: String::from_str(env, "ACE inhibitor for blood pressure"),
        is_phi: false,
    };
    db.add_term(lisinopril);

    // Additional Anatomy Terms
    let heart = MedicalTerm {
        term: String::from_str(env, "Heart"),
        normalized_form: String::from_str(env, "heart"),
        category: MedicalTermCategory::Anatomy,
        synonyms: Vec::from_array(env, [String::from_str(env, "Cardiac")]),
        abbreviations: Vec::new(env),
        icd10_codes: Vec::new(env),
        cpt_codes: Vec::new(env),
        snomed_code: Some(String::from_str(env, "80891009")),
        loinc_code: None,
        description: String::from_str(env, "Organ that pumps blood through the body"),
        is_phi: false,
    };
    db.add_term(heart);

    let lung = MedicalTerm {
        term: String::from_str(env, "Lung"),
        normalized_form: String::from_str(env, "lung"),
        category: MedicalTermCategory::Anatomy,
        synonyms: Vec::from_array(env, [String::from_str(env, "Pulmonary")]),
        abbreviations: Vec::new(env),
        icd10_codes: Vec::new(env),
        cpt_codes: Vec::new(env),
        snomed_code: Some(String::from_str(env, "39607008")),
        loinc_code: None,
        description: String::from_str(env, "Organ for gas exchange in the respiratory system"),
        is_phi: false,
    };
    db.add_term(lung);

    let brain = MedicalTerm {
        term: String::from_str(env, "Brain"),
        normalized_form: String::from_str(env, "brain"),
        category: MedicalTermCategory::Anatomy,
        synonyms: Vec::from_array(env, [String::from_str(env, "Cerebral")]),
        abbreviations: Vec::new(env),
        icd10_codes: Vec::new(env),
        cpt_codes: Vec::new(env),
        snomed_code: Some(String::from_str(env, "12738006")),
        loinc_code: None,
        description: String::from_str(env, "Organ of the central nervous system"),
        is_phi: false,
    };
    db.add_term(brain);

    let kidney = MedicalTerm {
        term: String::from_str(env, "Kidney"),
        normalized_form: String::from_str(env, "kidney"),
        category: MedicalTermCategory::Anatomy,
        synonyms: Vec::from_array(env, [String::from_str(env, "Renal")]),
        abbreviations: Vec::new(env),
        icd10_codes: Vec::new(env),
        cpt_codes: Vec::new(env),
        snomed_code: Some(String::from_str(env, "64033007")),
        loinc_code: None,
        description: String::from_str(env, "Organ that filters blood and produces urine"),
        is_phi: false,
    };
    db.add_term(kidney);

    // Additional Lab Test Terms
    let glucose_test = MedicalTerm {
        term: String::from_str(env, "Glucose Test"),
        normalized_form: String::from_str(env, "glucose test"),
        category: MedicalTermCategory::LabTest,
        synonyms: Vec::from_array(
            env,
            [
                String::from_str(env, "Blood Sugar Test"),
                String::from_str(env, "Fasting Glucose"),
            ],
        ),
        abbreviations: Vec::from_array(env, [String::from_str(env, "FBS")]),
        icd10_codes: Vec::new(env),
        cpt_codes: Vec::from_array(env, [String::from_str(env, "82947")]),
        snomed_code: Some(String::from_str(env, "36048009")),
        loinc_code: Some(String::from_str(env, "2345-7")),
        description: String::from_str(env, "Test to measure blood glucose levels"),
        is_phi: false,
    };
    db.add_term(glucose_test);

    let cholesterol_test = MedicalTerm {
        term: String::from_str(env, "Cholesterol Test"),
        normalized_form: String::from_str(env, "cholesterol test"),
        category: MedicalTermCategory::LabTest,
        synonyms: Vec::from_array(
            env,
            [
                String::from_str(env, "Lipid Panel"),
                String::from_str(env, "Lipid Profile"),
            ],
        ),
        abbreviations: Vec::from_array(env, [String::from_str(env, "LIPID")]),
        icd10_codes: Vec::new(env),
        cpt_codes: Vec::from_array(env, [String::from_str(env, "80061")]),
        snomed_code: Some(String::from_str(env, "104785007")),
        loinc_code: Some(String::from_str(env, "24331-1")),
        description: String::from_str(env, "Test to measure cholesterol levels in blood"),
        is_phi: false,
    };
    db.add_term(cholesterol_test);

    let hemoglobin_a1c = MedicalTerm {
        term: String::from_str(env, "Hemoglobin A1C"),
        normalized_form: String::from_str(env, "hemoglobin a1c"),
        category: MedicalTermCategory::LabTest,
        synonyms: Vec::from_array(
            env,
            [
                String::from_str(env, "HbA1c"),
                String::from_str(env, "Glycated Hemoglobin"),
            ],
        ),
        abbreviations: Vec::from_array(env, [String::from_str(env, "A1C")]),
        icd10_codes: Vec::new(env),
        cpt_codes: Vec::from_array(env, [String::from_str(env, "83036")]),
        snomed_code: Some(String::from_str(env, "104866001")),
        loinc_code: Some(String::from_str(env, "4548-4")),
        description: String::from_str(
            env,
            "Test to measure average blood sugar levels over 2-3 months",
        ),
        is_phi: false,
    };
    db.add_term(hemoglobin_a1c);

    // Additional Vital Sign Terms
    let temperature = MedicalTerm {
        term: String::from_str(env, "Temperature"),
        normalized_form: String::from_str(env, "temperature"),
        category: MedicalTermCategory::VitalSign,
        synonyms: Vec::from_array(env, [String::from_str(env, "Body Temperature")]),
        abbreviations: Vec::from_array(env, [String::from_str(env, "Temp")]),
        icd10_codes: Vec::new(env),
        cpt_codes: Vec::from_array(env, [String::from_str(env, "99213")]),
        snomed_code: Some(String::from_str(env, "276885007")),
        loinc_code: Some(String::from_str(env, "8310-5")),
        description: String::from_str(env, "Measurement of body heat"),
        is_phi: false,
    };
    db.add_term(temperature);

    let respiratory_rate = MedicalTerm {
        term: String::from_str(env, "Respiratory Rate"),
        normalized_form: String::from_str(env, "respiratory rate"),
        category: MedicalTermCategory::VitalSign,
        synonyms: Vec::from_array(
            env,
            [
                String::from_str(env, "Breathing Rate"),
                String::from_str(env, "RR"),
            ],
        ),
        abbreviations: Vec::from_array(env, [String::from_str(env, "RR")]),
        icd10_codes: Vec::new(env),
        cpt_codes: Vec::from_array(env, [String::from_str(env, "99213")]),
        snomed_code: Some(String::from_str(env, "86290005")),
        loinc_code: Some(String::from_str(env, "9279-1")),
        description: String::from_str(env, "Number of breaths per minute"),
        is_phi: false,
    };
    db.add_term(respiratory_rate);

    let oxygen_saturation = MedicalTerm {
        term: String::from_str(env, "Oxygen Saturation"),
        normalized_form: String::from_str(env, "oxygen saturation"),
        category: MedicalTermCategory::VitalSign,
        synonyms: Vec::from_array(
            env,
            [
                String::from_str(env, "SpO2"),
                String::from_str(env, "O2 Sat"),
            ],
        ),
        abbreviations: Vec::from_array(env, [String::from_str(env, "SpO2")]),
        icd10_codes: Vec::new(env),
        cpt_codes: Vec::from_array(env, [String::from_str(env, "94760")]),
        snomed_code: Some(String::from_str(env, "431314004")),
        loinc_code: Some(String::from_str(env, "2708-6")),
        description: String::from_str(env, "Percentage of oxygen-saturated hemoglobin in blood"),
        is_phi: false,
    };
    db.add_term(oxygen_saturation);

    // Additional Allergy Terms
    let penicillin_allergy = MedicalTerm {
        term: String::from_str(env, "Penicillin Allergy"),
        normalized_form: String::from_str(env, "penicillin allergy"),
        category: MedicalTermCategory::Allergy,
        synonyms: Vec::from_array(env, [String::from_str(env, "PCN Allergy")]),
        abbreviations: Vec::from_array(env, [String::from_str(env, "PCN")]),
        icd10_codes: Vec::from_array(env, [String::from_str(env, "Z88.0")]),
        cpt_codes: Vec::new(env),
        snomed_code: Some(String::from_str(env, "91936005")),
        loinc_code: None,
        description: String::from_str(env, "Allergic reaction to penicillin antibiotics"),
        is_phi: false,
    };
    db.add_term(penicillin_allergy);

    let latex_allergy = MedicalTerm {
        term: String::from_str(env, "Latex Allergy"),
        normalized_form: String::from_str(env, "latex allergy"),
        category: MedicalTermCategory::Allergy,
        synonyms: Vec::new(env),
        abbreviations: Vec::new(env),
        icd10_codes: Vec::from_array(env, [String::from_str(env, "Z91.040")]),
        cpt_codes: Vec::new(env),
        snomed_code: Some(String::from_str(env, "300916003")),
        loinc_code: None,
        description: String::from_str(env, "Allergic reaction to latex products"),
        is_phi: false,
    };
    db.add_term(latex_allergy);

    // Additional Condition Terms
    let obesity = MedicalTerm {
        term: String::from_str(env, "Obesity"),
        normalized_form: String::from_str(env, "obesity"),
        category: MedicalTermCategory::Condition,
        synonyms: Vec::from_array(env, [String::from_str(env, "Overweight")]),
        abbreviations: Vec::new(env),
        icd10_codes: Vec::from_array(
            env,
            [
                String::from_str(env, "E66.9"),
                String::from_str(env, "E66.01"),
            ],
        ),
        cpt_codes: Vec::new(env),
        snomed_code: Some(String::from_str(env, "414916001")),
        loinc_code: None,
        description: String::from_str(env, "Condition of having excess body fat"),
        is_phi: false,
    };
    db.add_term(obesity);

    let anemia = MedicalTerm {
        term: String::from_str(env, "Anemia"),
        normalized_form: String::from_str(env, "anemia"),
        category: MedicalTermCategory::Condition,
        synonyms: Vec::from_array(env, [String::from_str(env, "Low Hemoglobin")]),
        abbreviations: Vec::new(env),
        icd10_codes: Vec::from_array(env, [String::from_str(env, "D64.9")]),
        cpt_codes: Vec::new(env),
        snomed_code: Some(String::from_str(env, "271737000")),
        loinc_code: None,
        description: String::from_str(
            env,
            "Condition with insufficient red blood cells or hemoglobin",
        ),
        is_phi: false,
    };
    db.add_term(anemia);

    let infection = MedicalTerm {
        term: String::from_str(env, "Infection"),
        normalized_form: String::from_str(env, "infection"),
        category: MedicalTermCategory::Condition,
        synonyms: Vec::from_array(env, [String::from_str(env, "Infectious Disease")]),
        abbreviations: Vec::new(env),
        icd10_codes: Vec::from_array(env, [String::from_str(env, "B99.9")]),
        cpt_codes: Vec::new(env),
        snomed_code: Some(String::from_str(env, "40733004")),
        loinc_code: None,
        description: String::from_str(
            env,
            "Invasion and growth of harmful microorganisms in the body",
        ),
        is_phi: false,
    };
    db.add_term(infection);

    // Additional Medical Device Terms
    let pacemaker = MedicalTerm {
        term: String::from_str(env, "Pacemaker"),
        normalized_form: String::from_str(env, "pacemaker"),
        category: MedicalTermCategory::MedicalDevice,
        synonyms: Vec::from_array(env, [String::from_str(env, "Cardiac Pacemaker")]),
        abbreviations: Vec::new(env),
        icd10_codes: Vec::from_array(env, [String::from_str(env, "Z95.0")]),
        cpt_codes: Vec::from_array(env, [String::from_str(env, "33208")]),
        snomed_code: Some(String::from_str(env, "14106009")),
        loinc_code: None,
        description: String::from_str(env, "Device that helps control abnormal heart rhythms"),
        is_phi: false,
    };
    db.add_term(pacemaker);

    let ventilator = MedicalTerm {
        term: String::from_str(env, "Ventilator"),
        normalized_form: String::from_str(env, "ventilator"),
        category: MedicalTermCategory::MedicalDevice,
        synonyms: Vec::from_array(
            env,
            [
                String::from_str(env, "Mechanical Ventilator"),
                String::from_str(env, "Respirator"),
            ],
        ),
        abbreviations: Vec::new(env),
        icd10_codes: Vec::from_array(env, [String::from_str(env, "Z99.11")]),
        cpt_codes: Vec::from_array(env, [String::from_str(env, "94002")]),
        snomed_code: Some(String::from_str(env, "40617009")),
        loinc_code: None,
        description: String::from_str(env, "Machine that helps patients breathe"),
        is_phi: false,
    };
    db.add_term(ventilator);

    db
}
