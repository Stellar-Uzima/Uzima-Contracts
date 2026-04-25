use soroban_sdk::{Env, Map, String, Vec};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SentimentLabel {
    VeryNegative,
    Negative,
    Neutral,
    Positive,
    VeryPositive,
}

#[derive(Clone)]
pub struct SentimentResult {
    pub score: i32,
    pub label: SentimentLabel,
    pub confidence_bps: u32,
    pub emotional_indicators: Vec<String>,
    pub patient_satisfaction_score: Option<u32>,
}

pub struct SentimentLexicon {
    pub env: Env,
    pub positive_words: Map<String, i32>,
    pub negative_words: Map<String, i32>,
    pub intensifiers: Map<String, i32>,
    pub negators: Vec<String>,
    pub medical_positive: Map<String, i32>,
    pub medical_negative: Map<String, i32>,
}

impl SentimentLexicon {
    pub fn new() -> Self {
        let env = soroban_sdk::Env::default();
        Self {
            env,
            positive_words: Map::new(&env),
            negative_words: Map::new(&env),
            intensifiers: Map::new(&env),
            negators: Vec::new(&env),
            medical_positive: Map::new(&env),
            medical_negative: Map::new(&env),
        }
    }

    pub fn analyze_sentiment(&self, text: &String) -> SentimentResult {
        let words = Self::tokenize(text);
        let mut total_score: i32 = 0;
        let mut word_count: u32 = 0;
        let mut emotional_indicators = Vec::new(&self.env);
        let mut negation_context = false;

        for i in 0..words.len() {
            let word = words.get(i).unwrap();
            let word_lower = Self::to_lowercase(&word);

            if self.is_negator(&word_lower) {
                negation_context = true;
                continue;
            }

            let mut intensity_multiplier: i32 = 100;
            if i > 0 {
                let prev_word = Self::to_lowercase(&words.get(i - 1).unwrap());
                if let Some(multiplier) = self.intensifiers.get(prev_word) {
                    intensity_multiplier = multiplier;
                }
            }

            let mut word_score: i32 = 0;

            if let Some(score) = self.positive_words.get(word_lower.clone()) {
                word_score = score;
                if !negation_context {
                    emotional_indicators
                        .push_back(String::from_str(&self.env, "positive_expression"));
                }
            } else if let Some(score) = self.negative_words.get(word_lower.clone()) {
                word_score = score;
                if !negation_context {
                    emotional_indicators
                        .push_back(String::from_str(&self.env, "negative_expression"));
                }
            } else if let Some(score) = self.medical_positive.get(word_lower.clone()) {
                word_score = score;
                emotional_indicators
                    .push_back(String::from_str(&self.env, "positive_medical_feedback"));
            } else if let Some(score) = self.medical_negative.get(word_lower.clone()) {
                word_score = score;
                emotional_indicators
                    .push_back(String::from_str(&self.env, "negative_medical_feedback"));
            }

            word_score = (word_score * intensity_multiplier) / 100;

            if negation_context {
                word_score = -word_score;
                negation_context = false;
            }

            total_score += word_score;
            if word_score != 0 {
                word_count += 1;
            }
        }

        let normalized_score = if word_count > 0 {
            let avg = total_score / word_count as i32;
            avg.max(-100).min(100)
        } else {
            0
        };

        let label = if normalized_score <= -60 {
            SentimentLabel::VeryNegative
        } else if normalized_score <= -20 {
            SentimentLabel::Negative
        } else if normalized_score < 20 {
            SentimentLabel::Neutral
        } else if normalized_score < 60 {
            SentimentLabel::Positive
        } else {
            SentimentLabel::VeryPositive
        };

        let confidence = if word_count > 5 {
            let base_confidence = 7000;
            let score_bonus = (normalized_score.abs() * 20) as u32;
            (base_confidence + score_bonus).min(9500)
        } else if word_count > 0 {
            5000
        } else {
            3000
        };

        let satisfaction_score = if normalized_score >= 0 {
            Some(((normalized_score + 100) / 2) as u32)
        } else {
            Some(0)
        };

        SentimentResult {
            score: normalized_score,
            label,
            confidence_bps: confidence,
            emotional_indicators,
            patient_satisfaction_score: satisfaction_score,
        }
    }

    fn tokenize(text: &String) -> Vec<String> {
        let env = soroban_sdk::Env::default();
        let mut words = Vec::new(&env);
        let len = text.len();
        let mut current_word = Vec::new(&env);

        for i in 0..len {
            let ch = text.get(i).unwrap_or(0);

            if (48u32..=57).contains(&ch)
                || (65u32..=90).contains(&ch)
                || (97u32..=122).contains(&ch)
                || ch == 39
            {
                current_word.push_back(ch);
            } else if !current_word.is_empty() {
                let word = String::from_bytes(&env, &current_word);
                words.push_back(word);
                current_word = Vec::new(&env);
            }
        }

        if !current_word.is_empty() {
            let word = String::from_bytes(&env, &current_word);
            words.push_back(word);
        }

        words
    }

    fn is_negator(&self, word: &String) -> bool {
        for negator in self.negators.iter() {
            if &negator == word {
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

pub fn load_default_sentiment_lexicon(env: &Env) -> SentimentLexicon {
    let mut lexicon = SentimentLexicon {
        env: env.clone(),
        positive_words: Map::new(env),
        negative_words: Map::new(env),
        intensifiers: Map::new(env),
        negators: Vec::new(env),
        medical_positive: Map::new(env),
        medical_negative: Map::new(env),
    };

    lexicon
        .positive_words
        .set(String::from_str(env, "good"), 30);
    lexicon
        .positive_words
        .set(String::from_str(env, "great"), 40);
    lexicon
        .positive_words
        .set(String::from_str(env, "excellent"), 50);
    lexicon
        .positive_words
        .set(String::from_str(env, "wonderful"), 50);
    lexicon
        .positive_words
        .set(String::from_str(env, "amazing"), 50);
    lexicon
        .positive_words
        .set(String::from_str(env, "fantastic"), 50);
    lexicon
        .positive_words
        .set(String::from_str(env, "helpful"), 35);
    lexicon
        .positive_words
        .set(String::from_str(env, "caring"), 40);
    lexicon
        .positive_words
        .set(String::from_str(env, "professional"), 35);
    lexicon
        .positive_words
        .set(String::from_str(env, "kind"), 35);
    lexicon
        .positive_words
        .set(String::from_str(env, "attentive"), 40);
    lexicon
        .positive_words
        .set(String::from_str(env, "thorough"), 40);
    lexicon
        .positive_words
        .set(String::from_str(env, "efficient"), 35);
    lexicon
        .positive_words
        .set(String::from_str(env, "satisfied"), 40);
    lexicon
        .positive_words
        .set(String::from_str(env, "happy"), 40);
    lexicon
        .positive_words
        .set(String::from_str(env, "pleased"), 35);
    lexicon
        .positive_words
        .set(String::from_str(env, "comfortable"), 30);
    lexicon
        .positive_words
        .set(String::from_str(env, "relieved"), 35);
    lexicon
        .positive_words
        .set(String::from_str(env, "grateful"), 45);
    lexicon
        .positive_words
        .set(String::from_str(env, "thankful"), 45);

    lexicon
        .negative_words
        .set(String::from_str(env, "bad"), -30);
    lexicon
        .negative_words
        .set(String::from_str(env, "terrible"), -50);
    lexicon
        .negative_words
        .set(String::from_str(env, "horrible"), -50);
    lexicon
        .negative_words
        .set(String::from_str(env, "awful"), -50);
    lexicon
        .negative_words
        .set(String::from_str(env, "poor"), -35);
    lexicon
        .negative_words
        .set(String::from_str(env, "disappointing"), -40);
    lexicon
        .negative_words
        .set(String::from_str(env, "frustrating"), -40);
    lexicon
        .negative_words
        .set(String::from_str(env, "rude"), -45);
    lexicon
        .negative_words
        .set(String::from_str(env, "unprofessional"), -45);
    lexicon
        .negative_words
        .set(String::from_str(env, "negligent"), -50);
    lexicon
        .negative_words
        .set(String::from_str(env, "careless"), -45);
    lexicon
        .negative_words
        .set(String::from_str(env, "painful"), -35);
    lexicon
        .negative_words
        .set(String::from_str(env, "uncomfortable"), -30);
    lexicon
        .negative_words
        .set(String::from_str(env, "dissatisfied"), -40);
    lexicon
        .negative_words
        .set(String::from_str(env, "unhappy"), -35);
    lexicon
        .negative_words
        .set(String::from_str(env, "angry"), -40);
    lexicon
        .negative_words
        .set(String::from_str(env, "upset"), -35);
    lexicon
        .negative_words
        .set(String::from_str(env, "worried"), -25);
    lexicon
        .negative_words
        .set(String::from_str(env, "anxious"), -25);
    lexicon
        .negative_words
        .set(String::from_str(env, "scared"), -30);

    lexicon
        .medical_positive
        .set(String::from_str(env, "improved"), 40);
    lexicon
        .medical_positive
        .set(String::from_str(env, "recovered"), 45);
    lexicon
        .medical_positive
        .set(String::from_str(env, "healed"), 45);
    lexicon
        .medical_positive
        .set(String::from_str(env, "stable"), 30);
    lexicon
        .medical_positive
        .set(String::from_str(env, "healthy"), 40);
    lexicon
        .medical_positive
        .set(String::from_str(env, "effective"), 35);
    lexicon
        .medical_positive
        .set(String::from_str(env, "successful"), 45);
    lexicon
        .medical_positive
        .set(String::from_str(env, "responsive"), 35);
    lexicon
        .medical_positive
        .set(String::from_str(env, "tolerated"), 30);
    lexicon
        .medical_positive
        .set(String::from_str(env, "well-controlled"), 40);

    lexicon
        .medical_negative
        .set(String::from_str(env, "worsened"), -40);
    lexicon
        .medical_negative
        .set(String::from_str(env, "deteriorated"), -45);
    lexicon
        .medical_negative
        .set(String::from_str(env, "complications"), -35);
    lexicon
        .medical_negative
        .set(String::from_str(env, "adverse"), -40);
    lexicon
        .medical_negative
        .set(String::from_str(env, "side effects"), -30);
    lexicon
        .medical_negative
        .set(String::from_str(env, "infection"), -35);
    lexicon
        .medical_negative
        .set(String::from_str(env, "pain"), -25);
    lexicon
        .medical_negative
        .set(String::from_str(env, "discomfort"), -20);
    lexicon
        .medical_negative
        .set(String::from_str(env, "nausea"), -25);
    lexicon
        .medical_negative
        .set(String::from_str(env, "fever"), -25);

    lexicon.intensifiers.set(String::from_str(env, "very"), 150);
    lexicon
        .intensifiers
        .set(String::from_str(env, "extremely"), 200);
    lexicon
        .intensifiers
        .set(String::from_str(env, "really"), 150);
    lexicon
        .intensifiers
        .set(String::from_str(env, "absolutely"), 200);
    lexicon
        .intensifiers
        .set(String::from_str(env, "completely"), 180);
    lexicon
        .intensifiers
        .set(String::from_str(env, "totally"), 180);
    lexicon
        .intensifiers
        .set(String::from_str(env, "somewhat"), 50);
    lexicon
        .intensifiers
        .set(String::from_str(env, "slightly"), 30);
    lexicon.intensifiers.set(String::from_str(env, "a bit"), 40);

    lexicon.negators.push_back(String::from_str(env, "not"));
    lexicon.negators.push_back(String::from_str(env, "no"));
    lexicon.negators.push_back(String::from_str(env, "never"));
    lexicon.negators.push_back(String::from_str(env, "neither"));
    lexicon.negators.push_back(String::from_str(env, "nor"));
    lexicon.negators.push_back(String::from_str(env, "none"));
    lexicon.negators.push_back(String::from_str(env, "nothing"));
    lexicon.negators.push_back(String::from_str(env, "nowhere"));
    lexicon.negators.push_back(String::from_str(env, "hardly"));
    lexicon.negators.push_back(String::from_str(env, "barely"));
    lexicon
        .negators
        .push_back(String::from_str(env, "scarcely"));
    lexicon.negators.push_back(String::from_str(env, "seldom"));
    lexicon.negators.push_back(String::from_str(env, "rarely"));

    // Additional positive words
    lexicon
        .positive_words
        .set(String::from_str(env, "outstanding"), 55);
    lexicon
        .positive_words
        .set(String::from_str(env, "exceptional"), 55);
    lexicon
        .positive_words
        .set(String::from_str(env, "superb"), 50);
    lexicon
        .positive_words
        .set(String::from_str(env, "brilliant"), 50);
    lexicon
        .positive_words
        .set(String::from_str(env, "perfect"), 60);
    lexicon
        .positive_words
        .set(String::from_str(env, "best"), 50);
    lexicon
        .positive_words
        .set(String::from_str(env, "love"), 45);
    lexicon
        .positive_words
        .set(String::from_str(env, "appreciate"), 40);
    lexicon
        .positive_words
        .set(String::from_str(env, "commend"), 45);
    lexicon
        .positive_words
        .set(String::from_str(env, "praise"), 45);
    lexicon
        .positive_words
        .set(String::from_str(env, "recommend"), 40);
    lexicon
        .positive_words
        .set(String::from_str(env, "trust"), 35);
    lexicon
        .positive_words
        .set(String::from_str(env, "confident"), 35);
    lexicon
        .positive_words
        .set(String::from_str(env, "reassured"), 40);
    lexicon
        .positive_words
        .set(String::from_str(env, "comforted"), 40);
    lexicon
        .positive_words
        .set(String::from_str(env, "supported"), 35);
    lexicon
        .positive_words
        .set(String::from_str(env, "valued"), 35);
    lexicon
        .positive_words
        .set(String::from_str(env, "respected"), 35);
    lexicon
        .positive_words
        .set(String::from_str(env, "listened"), 30);
    lexicon
        .positive_words
        .set(String::from_str(env, "understood"), 30);

    // Additional negative words
    lexicon
        .negative_words
        .set(String::from_str(env, "worst"), -60);
    lexicon
        .negative_words
        .set(String::from_str(env, "dreadful"), -55);
    lexicon
        .negative_words
        .set(String::from_str(env, "miserable"), -50);
    lexicon
        .negative_words
        .set(String::from_str(env, "unbearable"), -55);
    lexicon
        .negative_words
        .set(String::from_str(env, "intolerable"), -55);
    lexicon
        .negative_words
        .set(String::from_str(env, "disgusting"), -55);
    lexicon
        .negative_words
        .set(String::from_str(env, "appalling"), -55);
    lexicon
        .negative_words
        .set(String::from_str(env, "shocking"), -50);
    lexicon
        .negative_words
        .set(String::from_str(env, "unacceptable"), -50);
    lexicon
        .negative_words
        .set(String::from_str(env, "inadequate"), -40);
    lexicon
        .negative_words
        .set(String::from_str(env, "insufficient"), -35);
    lexicon
        .negative_words
        .set(String::from_str(env, "lacking"), -30);
    lexicon
        .negative_words
        .set(String::from_str(env, "deficient"), -40);
    lexicon
        .negative_words
        .set(String::from_str(env, "subpar"), -40);
    lexicon
        .negative_words
        .set(String::from_str(env, "inferior"), -40);
    lexicon
        .negative_words
        .set(String::from_str(env, "dismissive"), -45);
    lexicon
        .negative_words
        .set(String::from_str(env, "indifferent"), -35);
    lexicon
        .negative_words
        .set(String::from_str(env, "cold"), -30);
    lexicon
        .negative_words
        .set(String::from_str(env, "impersonal"), -30);
    lexicon
        .negative_words
        .set(String::from_str(env, "rushed"), -35);
    lexicon
        .negative_words
        .set(String::from_str(env, "hurried"), -30);
    lexicon
        .negative_words
        .set(String::from_str(env, "ignored"), -45);
    lexicon
        .negative_words
        .set(String::from_str(env, "dismissed"), -45);
    lexicon
        .negative_words
        .set(String::from_str(env, "belittled"), -50);
    lexicon
        .negative_words
        .set(String::from_str(env, "patronizing"), -45);
    lexicon
        .negative_words
        .set(String::from_str(env, "condescending"), -45);

    // Additional medical positive terms
    lexicon
        .medical_positive
        .set(String::from_str(env, "improving"), 40);
    lexicon
        .medical_positive
        .set(String::from_str(env, "progressing"), 35);
    lexicon
        .medical_positive
        .set(String::from_str(env, "responding"), 35);
    lexicon
        .medical_positive
        .set(String::from_str(env, "healing"), 40);
    lexicon
        .medical_positive
        .set(String::from_str(env, "remission"), 50);
    lexicon
        .medical_positive
        .set(String::from_str(env, "cured"), 55);
    lexicon
        .medical_positive
        .set(String::from_str(env, "resolved"), 45);
    lexicon
        .medical_positive
        .set(String::from_str(env, "managed"), 30);
    lexicon
        .medical_positive
        .set(String::from_str(env, "controlled"), 30);
    lexicon
        .medical_positive
        .set(String::from_str(env, "stable"), 30);
    lexicon
        .medical_positive
        .set(String::from_str(env, "normal"), 25);
    lexicon
        .medical_positive
        .set(String::from_str(env, "within normal limits"), 30);
    lexicon
        .medical_positive
        .set(String::from_str(env, "unremarkable"), 25);
    lexicon
        .medical_positive
        .set(String::from_str(env, "clear"), 25);
    lexicon
        .medical_positive
        .set(String::from_str(env, "negative"), 20);

    // Additional medical negative terms
    lexicon
        .medical_negative
        .set(String::from_str(env, "worsening"), -40);
    lexicon
        .medical_negative
        .set(String::from_str(env, "deteriorating"), -45);
    lexicon
        .medical_negative
        .set(String::from_str(env, "declining"), -40);
    lexicon
        .medical_negative
        .set(String::from_str(env, "progressive"), -35);
    lexicon
        .medical_negative
        .set(String::from_str(env, "chronic"), -25);
    lexicon
        .medical_negative
        .set(String::from_str(env, "acute"), -20);
    lexicon
        .medical_negative
        .set(String::from_str(env, "severe"), -40);
    lexicon
        .medical_negative
        .set(String::from_str(env, "critical"), -45);
    lexicon
        .medical_negative
        .set(String::from_str(env, "life-threatening"), -55);
    lexicon
        .medical_negative
        .set(String::from_str(env, "terminal"), -60);
    lexicon
        .medical_negative
        .set(String::from_str(env, "malignant"), -50);
    lexicon
        .medical_negative
        .set(String::from_str(env, "metastatic"), -55);
    lexicon
        .medical_negative
        .set(String::from_str(env, "recurrence"), -40);
    lexicon
        .medical_negative
        .set(String::from_str(env, "relapse"), -40);
    lexicon
        .medical_negative
        .set(String::from_str(env, "complication"), -35);
    lexicon
        .medical_negative
        .set(String::from_str(env, "reaction"), -30);
    lexicon
        .medical_negative
        .set(String::from_str(env, "allergic"), -35);
    lexicon
        .medical_negative
        .set(String::from_str(env, "toxic"), -45);
    lexicon
        .medical_negative
        .set(String::from_str(env, "hemorrhage"), -50);
    lexicon
        .medical_negative
        .set(String::from_str(env, "bleeding"), -35);
    lexicon
        .medical_negative
        .set(String::from_str(env, "swelling"), -25);
    lexicon
        .medical_negative
        .set(String::from_str(env, "inflammation"), -30);

    // Additional intensifiers
    lexicon
        .intensifiers
        .set(String::from_str(env, "highly"), 180);
    lexicon
        .intensifiers
        .set(String::from_str(env, "incredibly"), 200);
    lexicon
        .intensifiers
        .set(String::from_str(env, "remarkably"), 180);
    lexicon
        .intensifiers
        .set(String::from_str(env, "exceptionally"), 200);
    lexicon
        .intensifiers
        .set(String::from_str(env, "particularly"), 150);
    lexicon
        .intensifiers
        .set(String::from_str(env, "especially"), 150);
    lexicon
        .intensifiers
        .set(String::from_str(env, "notably"), 140);
    lexicon
        .intensifiers
        .set(String::from_str(env, "significantly"), 160);
    lexicon
        .intensifiers
        .set(String::from_str(env, "substantially"), 160);
    lexicon
        .intensifiers
        .set(String::from_str(env, "considerably"), 150);
    lexicon
        .intensifiers
        .set(String::from_str(env, "mildly"), 40);
    lexicon
        .intensifiers
        .set(String::from_str(env, "moderately"), 60);
    lexicon
        .intensifiers
        .set(String::from_str(env, "somewhat"), 50);
    lexicon
        .intensifiers
        .set(String::from_str(env, "fairly"), 60);
    lexicon
        .intensifiers
        .set(String::from_str(env, "rather"), 70);
    lexicon.intensifiers.set(String::from_str(env, "quite"), 80);
    lexicon
        .intensifiers
        .set(String::from_str(env, "pretty"), 70);

    // Additional negators
    lexicon.negators.push_back(String::from_str(env, "cannot"));
    lexicon.negators.push_back(String::from_str(env, "can't"));
    lexicon.negators.push_back(String::from_str(env, "don't"));
    lexicon.negators.push_back(String::from_str(env, "doesn't"));
    lexicon.negators.push_back(String::from_str(env, "didn't"));
    lexicon.negators.push_back(String::from_str(env, "won't"));
    lexicon
        .negators
        .push_back(String::from_str(env, "wouldn't"));
    lexicon
        .negators
        .push_back(String::from_str(env, "shouldn't"));
    lexicon
        .negators
        .push_back(String::from_str(env, "couldn't"));
    lexicon.negators.push_back(String::from_str(env, "isn't"));
    lexicon.negators.push_back(String::from_str(env, "aren't"));
    lexicon.negators.push_back(String::from_str(env, "wasn't"));
    lexicon.negators.push_back(String::from_str(env, "weren't"));
    lexicon.negators.push_back(String::from_str(env, "hasn't"));
    lexicon.negators.push_back(String::from_str(env, "haven't"));
    lexicon.negators.push_back(String::from_str(env, "hadn't"));

    lexicon
}
