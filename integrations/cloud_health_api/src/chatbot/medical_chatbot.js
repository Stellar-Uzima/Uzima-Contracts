const { MedicalKnowledgeBase } = require('./knowledge_base');

const LANGUAGE_PATTERNS = [
  { language: 'sw', patterns: ['habari', 'naumwa', 'homa', 'kikohozi', 'maumivu', 'dharura', 'tafadhali', 'mimba', 'kisukari'] },
  { language: 'fr', patterns: ['bonjour', 'douleur', 'fievre', 'urgence', 'toux', 'enceinte', 'diabete', 'poitrine'] }
];

const SYMPTOM_LEXICON = {
  fever: ['fever', 'temperature', 'pyrexia', 'homa', 'fievre'],
  cough: ['cough', 'kikohozi', 'toux'],
  headache: ['headache', 'migraine', 'kichwa', 'mal de tete'],
  chest_pain: ['chest pain', 'tight chest', 'maumivu ya kifua', 'douleur thoracique'],
  shortness_of_breath: ['shortness of breath', 'difficulty breathing', 'trouble breathing', 'breathless', 'shida ya kupumua', 'essoufflement'],
  dizziness: ['dizzy', 'dizziness', 'lightheaded', 'kizunguzungu', 'vertige'],
  vomiting: ['vomiting', 'vomit', 'throwing up', 'kutapika', 'vomissements'],
  diarrhea: ['diarrhea', 'loose stool', 'kuharisha', 'diarrhee'],
  fatigue: ['fatigue', 'tired', 'weak', 'uchovu', 'fatigue'],
  rash: ['rash', 'skin bumps', 'upele', 'eruption'],
  bleeding: ['bleeding', 'blood loss', 'kutokwa na damu', 'saignement'],
  confusion: ['confusion', 'confused', 'disoriented', 'kuchanganyikiwa', 'confusion'],
  seizure: ['seizure', 'convulsion', 'degedege', 'convulsion'],
  abdominal_pain: ['abdominal pain', 'stomach pain', 'tumbo', 'douleur abdominale'],
  pregnancy: ['pregnant', 'pregnancy', 'mjamzito', 'enceinte'],
  high_blood_sugar: ['high sugar', 'high blood sugar', 'glucose high', 'sukari juu', 'glycemie elevee']
};

const EDUCATION_LIBRARY = {
  fever: {
    en: 'Keep drinking fluids, rest, and monitor temperature. Seek care faster if fever lasts more than two days or comes with red-flag symptoms.',
    sw: 'Kunywa maji ya kutosha, pumzika, na fuatilia joto la mwili. Tafuta huduma mapema ikiwa homa inaendelea zaidi ya siku mbili au ina dalili za hatari.',
    fr: 'Buvez suffisamment, reposez-vous et surveillez la temperature. Consultez plus vite si la fievre dure plus de deux jours ou s accompagne de signes de danger.'
  },
  cough: {
    en: 'Warm fluids, rest, and avoiding smoke may help mild cough. Medical review is important if cough persists, is severe, or makes breathing hard.',
    sw: 'Vinywaji vya moto, mapumziko, na kuepuka moshi vinaweza kusaidia kikohozi kidogo. Tathmini ya kitabibu ni muhimu ikiwa kikohozi kinaendelea, ni kali, au kinafanya kupumua kuwa ngumu.',
    fr: 'Les boissons chaudes, le repos et l evitement de la fumee peuvent aider pour une toux legere. Une evaluation medicale est importante si la toux persiste, est severe ou gene la respiration.'
  },
  diabetes: {
    en: 'Stay consistent with prescribed medicines, meals, hydration, and glucose monitoring. Know your sick-day plan and when to seek urgent review for very high or low sugar.',
    sw: 'Fuata dawa ulizopewa, kula kwa mpangilio, kunywa maji, na pima sukari mara kwa mara. Jua mpango wako wa siku za ugonjwa na wakati wa kutafuta tathmini ya haraka kwa sukari ya juu au ya chini sana.',
    fr: 'Respectez les traitements prescrits, l alimentation, l hydratation et la surveillance de la glycemie. Connaissez votre plan en cas de maladie et le moment ou une evaluation urgente est necessaire pour une glycemie tres haute ou tres basse.'
  },
  hypertension: {
    en: 'Take blood pressure medicine as prescribed, limit salt, stay active, and attend follow-up visits. Severe neurologic or chest symptoms are emergencies.',
    sw: 'Tumia dawa za presha kama ulivyoelekezwa, punguza chumvi, fanya mazoezi, na hudhuria ufuatiliaji. Dalili kali za mfumo wa fahamu au kifua ni dharura.',
    fr: 'Prenez les medicaments antihypertenseurs comme prescrits, limitez le sel, restez actif et suivez vos rendez-vous. Les symptomes neurologiques severes ou thoraciques sont des urgences.'
  },
  wellness: {
    en: 'Healthy routines include hydration, balanced meals, sleep, movement, vaccinations, and regular checkups. A clinician can tailor these steps to your age and health conditions.',
    sw: 'Mazoea mazuri ya afya ni kunywa maji, kula mlo wenye usawa, kulala vizuri, kufanya mazoezi, chanjo, na uchunguzi wa mara kwa mara. Daktari anaweza kurekebisha ushauri huu kulingana na umri na hali yako.',
    fr: 'Les habitudes saines incluent l hydratation, une alimentation equilibree, le sommeil, l activite physique, la vaccination et des controles reguliers. Un clinicien peut adapter ces conseils a votre age et a votre etat de sante.'
  }
};

const EMERGENCY_PATTERNS = [
  ['chest_pain', 'shortness_of_breath'],
  ['bleeding'],
  ['confusion'],
  ['seizure'],
  ['pregnancy', 'bleeding'],
  ['high_blood_sugar', 'vomiting'],
  ['shortness_of_breath'],
  ['chest_pain']
];

const TRIAGE_RULES = [
  {
    level: 'emergency',
    when: (symptoms) => hasPattern(symptoms, ['chest_pain']) || hasPattern(symptoms, ['shortness_of_breath']) || hasPattern(symptoms, ['bleeding']) || hasPattern(symptoms, ['confusion']) || hasPattern(symptoms, ['seizure'])
  },
  {
    level: 'urgent',
    when: (symptoms) => hasPattern(symptoms, ['fever', 'cough']) || hasPattern(symptoms, ['vomiting', 'diarrhea']) || hasPattern(symptoms, ['dizziness', 'headache']) || hasPattern(symptoms, ['pregnancy', 'abdominal_pain'])
  },
  {
    level: 'routine',
    when: (symptoms) => symptoms.length > 0
  }
];

function hasPattern(symptoms, pattern) {
  return pattern.every((symptom) => symptoms.includes(symptom));
}

function normalizeText(text) {
  return String(text || '')
    .toLowerCase()
    .replace(/[^\w\s]/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();
}

class MedicalChatbot {
  constructor(options = {}) {
    this.knowledgeBase = options.knowledgeBase || new MedicalKnowledgeBase();
    this.emergencyContacts = options.emergencyContacts || {
      en: 'Call local emergency services or go to the nearest emergency department now.',
      sw: 'Piga simu kwa huduma za dharura za eneo lako au nenda kwenye idara ya dharura iliyo karibu sasa.',
      fr: 'Appelez les services d urgence locaux ou rendez-vous immediatement au service d urgence le plus proche.'
    };
  }

  detectLanguage(message, preferredLanguage) {
    if (preferredLanguage) {
      return preferredLanguage;
    }

    const normalized = normalizeText(message);
    const matched = LANGUAGE_PATTERNS.find(({ patterns }) => patterns.some((pattern) => normalized.includes(pattern)));
    return matched ? matched.language : 'en';
  }

  parseMedicalQuery(message) {
    const normalized = normalizeText(message);
    const symptoms = Object.entries(SYMPTOM_LEXICON)
      .filter(([, aliases]) => aliases.some((alias) => normalized.includes(alias)))
      .map(([symptom]) => symptom);

    const asksSymptomCheck = /\b(symptom|triage|what should i do|naumwa|que faire|check)\b/.test(normalized) || symptoms.length > 0;
    const asksEducation = /\b(educate|learn|prevent|manage|control|reduce|explain|about|health education|teach me|wellness|nifundishe|explique)\b/.test(normalized);
    const asksEmergency = /\b(emergency|urgent|help now|dharura|urgence|911)\b/.test(normalized);

    let intent = 'general_inquiry';
    if (asksEmergency && symptoms.length === 0) {
      intent = 'emergency_help';
    } else if (asksEducation && symptoms.length === 0) {
      intent = 'health_education';
    } else if (asksSymptomCheck) {
      intent = 'symptom_check';
    } else if (asksEducation || this.extractConditions(normalized).length > 0) {
      intent = 'health_education';
    }
    
    const conditions = this.extractConditions(normalized);
    return {
      normalized,
      intent,
      symptoms,
      conditions,
      profileHints: this.extractProfileHints(normalized)
    };
  }

  extractConditions(normalized) {
    const conditions = [];
    if (/\b(diabetes|kisukari|diabete)\b/.test(normalized)) conditions.push('diabetes');
    if (/\b(hypertension|high blood pressure|presha|tension)\b/.test(normalized)) conditions.push('hypertension');
    if (/\b(pregnant|pregnancy|mjamzito|enceinte)\b/.test(normalized)) conditions.push('pregnancy');
    return conditions;
  }

  extractProfileHints(normalized) {
    const profile = {};
    const ageMatch = normalized.match(/\b(\d{1,3})\s*(years old|yo|ans|yr)\b/);
    if (ageMatch) {
      profile.age = Number(ageMatch[1]);
    }
    if (/\b(child|baby|infant|mtoto|enfant)\b/.test(normalized)) {
      profile.population = 'child';
    }
    if (/\b(elderly|older adult|senior)\b/.test(normalized)) {
      profile.population = 'older_adult';
    }
    if (/\b(pregnant|pregnancy|mjamzito|enceinte)\b/.test(normalized)) {
      profile.pregnant = true;
    }
    return profile;
  }

  assessEmergency(symptoms, parsed) {
    const reasons = EMERGENCY_PATTERNS
      .filter((pattern) => hasPattern(symptoms, pattern))
      .map((pattern) => pattern.join('+'));

    if (parsed.intent === 'emergency_help' && reasons.length === 0) {
      reasons.push('explicit_emergency_request');
    }

    return {
      detected: reasons.length > 0,
      reasons
    };
  }

  determineTriage(symptoms, emergencyDetected) {
    if (emergencyDetected) {
      return 'emergency';
    }

    const match = TRIAGE_RULES.find((rule) => rule.when(symptoms));
    return match ? match.level : 'self_care';
  }

  async generateEducation(parsed, language) {
    const kbMatches = await this.knowledgeBase.search(parsed.normalized, language);
    const primaryCondition = parsed.conditions[0];
    const primarySymptom = parsed.symptoms[0];
    const educationKey = primaryCondition || primarySymptom || 'wellness';
    const localEducation = EDUCATION_LIBRARY[educationKey] || EDUCATION_LIBRARY.wellness;

    const guidance = localEducation[language] || localEducation.en;
    const kbSummary = kbMatches[0] ? kbMatches[0].summary : null;

    return {
      guidance,
      references: kbMatches.slice(0, 3),
      knowledgeSummary: kbSummary
    };
  }

  personalizeEducation(baseText, profile, language) {
    if (profile.population === 'child') {
      return `${baseText} ${(language === 'sw') ? 'Kwa mtoto, dalili zinazozidi au kukataa kunywa zinahitaji tathmini mapema.' : (language === 'fr') ? 'Chez un enfant, l aggravation des symptomes ou le refus de boire justifient une evaluation precoce.' : 'For a child, worsening symptoms or poor fluid intake should be reviewed early.'}`;
    }
    if (profile.population === 'older_adult') {
      return `${baseText} ${(language === 'sw') ? 'Kwa mtu mzima mwenye umri mkubwa, tathmini mapema ni muhimu zaidi kwa sababu ya hatari kubwa ya madhara.' : (language === 'fr') ? 'Chez la personne agee, une evaluation precoce est encore plus importante en raison d un risque plus eleve de complications.' : 'For an older adult, earlier clinical review is wise because complications can escalate faster.'}`;
    }
    if (profile.pregnant) {
      return `${baseText} ${(language === 'sw') ? 'Ikiwa una ujauzito, mjulishe daktari au mkunga mapema kuhusu dalili mpya.' : (language === 'fr') ? 'Si vous etes enceinte, informez rapidement votre medecin ou votre sage-femme de tout nouveau symptome.' : 'If you are pregnant, let a clinician or midwife know promptly about new symptoms.'}`;
    }
    return baseText;
  }

  buildDisposition(triage, language) {
    const dispositions = {
      en: {
        emergency: 'This looks like an emergency situation.',
        urgent: 'This should be assessed urgently, ideally the same day.',
        routine: 'This deserves a routine clinical review.',
        self_care: 'This sounds suitable for home monitoring unless symptoms worsen.'
      },
      sw: {
        emergency: 'Hii inaonekana kuwa hali ya dharura.',
        urgent: 'Hii inahitaji tathmini ya haraka, ikiwezekana siku hiyo hiyo.',
        routine: 'Hii inahitaji tathmini ya kawaida ya kitabibu.',
        self_care: 'Hii inaonekana kufaa kufuatiliwa nyumbani isipokuwa dalili zizidi.'
      },
      fr: {
        emergency: 'Cela ressemble a une situation d urgence.',
        urgent: 'Cela doit etre evalue rapidement, idealement le jour meme.',
        routine: 'Cela merite une evaluation clinique de routine.',
        self_care: 'Cela semble compatible avec une surveillance a domicile sauf aggravation.'
      }
    };

    return (dispositions[language] || dispositions.en)[triage];
  }

  async respond({ message, patientProfile = {}, preferredLanguage } = {}) {
    const startedAt = Date.now();
    const language = this.detectLanguage(message, preferredLanguage);
    const parsed = this.parseMedicalQuery(message);
    const mergedProfile = { ...parsed.profileHints, ...patientProfile };
    const emergency = this.assessEmergency(parsed.symptoms, parsed);
    const triage = this.determineTriage(parsed.symptoms, emergency.detected);
    const education = await this.generateEducation(parsed, language);
    const personalizedEducation = this.personalizeEducation(
      education.knowledgeSummary || education.guidance,
      mergedProfile,
      language
    );

    const disclaimer = (language === 'sw')
      ? 'Huu ni mwongozo wa taarifa tu na hauchukui nafasi ya tathmini ya daktari.'
      : (language === 'fr')
        ? 'Ces informations servent de guide et ne remplacent pas une evaluation medicale.'
        : 'This guidance is informational and does not replace a licensed clinician evaluation.';

    const escalation = emergency.detected ? {
      emergencyDetected: true,
      reasons: emergency.reasons,
      protocol: this.emergencyContacts[language] || this.emergencyContacts.en
    } : {
      emergencyDetected: false,
      reasons: [],
      protocol: null
    };

    return {
      language,
      intent: parsed.intent,
      triage,
      symptoms: parsed.symptoms,
      conditions: parsed.conditions,
      escalation,
      response: [
        this.buildDisposition(triage, language),
        personalizedEducation,
        escalation.protocol,
        disclaimer
      ].filter(Boolean).join(' '),
      metadata: {
        responseTimeMs: Date.now() - startedAt,
        references: education.references
      }
    };
  }

  async evaluateAccuracy(dataset = []) {
    if (!Array.isArray(dataset) || dataset.length === 0) {
      return { accuracy: 1, passed: 0, total: 0 };
    }

    let passed = 0;
    for (const scenario of dataset) {
      const result = await this.respond(scenario.input);
      const intentOk = !scenario.expected.intent || result.intent === scenario.expected.intent;
      const triageOk = !scenario.expected.triage || result.triage === scenario.expected.triage;
      const languageOk = !scenario.expected.language || result.language === scenario.expected.language;
      const emergencyOk = typeof scenario.expected.emergency !== 'boolean'
        || result.escalation.emergencyDetected === scenario.expected.emergency;

      if (intentOk && triageOk && languageOk && emergencyOk) {
        passed += 1;
      }
    }

    return {
      accuracy: passed / dataset.length,
      passed,
      total: dataset.length
    };
  }
}

module.exports = {
  MedicalChatbot
};
