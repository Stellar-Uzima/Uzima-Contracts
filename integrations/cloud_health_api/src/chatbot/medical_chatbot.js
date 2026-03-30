const { MedicalKnowledgeBase } = require('./knowledge_base');

const SUPPORTED_LANGUAGES = ['en', 'sw', 'fr', 'es'];

const LANGUAGE_PATTERNS = [
  { language: 'sw', patterns: ['habari', 'naumwa', 'homa', 'kikohozi', 'maumivu', 'dharura', 'tafadhali', 'mimba', 'kisukari', 'kichwa', 'kizunguzungu'] },
  { language: 'fr', patterns: ['bonjour', 'douleur', 'fievre', 'urgence', 'toux', 'enceinte', 'diabete', 'poitrine', 'essoufflement', 'explique'] },
  { language: 'es', patterns: ['hola', 'dolor', 'fiebre', 'urgencia', 'tos', 'embarazada', 'diabetes', 'pecho', 'falta de aire', 'explica'] }
];

const SYMPTOM_LEXICON = {
  fever: ['fever', 'temperature', 'pyrexia', 'hot body', 'homa', 'fievre', 'fiebre'],
  cough: ['cough', 'dry cough', 'wet cough', 'kikohozi', 'toux', 'tos'],
  headache: ['headache', 'migraine', 'kichwa', 'mal de tete', 'dolor de cabeza'],
  chest_pain: ['chest pain', 'tight chest', 'maumivu ya kifua', 'douleur thoracique', 'dolor en el pecho'],
  shortness_of_breath: ['shortness of breath', 'difficulty breathing', 'trouble breathing', 'breathless', 'shida ya kupumua', 'essoufflement', 'falta de aire'],
  dizziness: ['dizzy', 'dizziness', 'lightheaded', 'kizunguzungu', 'vertige', 'mareado'],
  vomiting: ['vomiting', 'vomit', 'throwing up', 'kutapika', 'vomissements', 'vomitos'],
  diarrhea: ['diarrhea', 'loose stool', 'kuharisha', 'diarrhee', 'diarrea'],
  fatigue: ['fatigue', 'tired', 'weak', 'uchovu', 'fatigue', 'cansancio'],
  rash: ['rash', 'skin bumps', 'upele', 'eruption', 'erupcion'],
  bleeding: ['bleeding', 'blood loss', 'kutokwa na damu', 'saignement', 'sangrado'],
  confusion: ['confusion', 'confused', 'disoriented', 'kuchanganyikiwa', 'confusion', 'confusion mental'],
  seizure: ['seizure', 'convulsion', 'degedege', 'convulsion', 'convulsiones'],
  abdominal_pain: ['abdominal pain', 'stomach pain', 'tumbo', 'douleur abdominale', 'dolor abdominal'],
  pregnancy: ['pregnant', 'pregnancy', 'mjamzito', 'enceinte', 'embarazada', 'embarazo'],
  high_blood_sugar: ['high sugar', 'high blood sugar', 'glucose high', 'sukari juu', 'glycemie elevee', 'azucar alta'],
  dehydration: ['dehydrated', 'dehydration', 'dry mouth', 'thirsty', 'upungufu wa maji', 'deshydratation', 'deshidratacion']
};

const CONDITION_LEXICON = {
  diabetes: ['diabetes', 'kisukari', 'diabete'],
  hypertension: ['hypertension', 'high blood pressure', 'presha', 'tension', 'hipertension'],
  asthma: ['asthma', 'pumu', 'asthme', 'asma'],
  pregnancy: ['pregnant', 'pregnancy', 'mjamzito', 'enceinte', 'embarazada', 'embarazo']
};

const SEVERITY_PATTERNS = {
  severe: ['severe', 'very bad', 'worst', 'can t breathe', 'hard to breathe', 'serious', 'kali', 'grave', 'grave', 'intenso'],
  moderate: ['moderate', 'persistent', 'ongoing', 'increasing', 'continues', 'inaendelea', 'persistante', 'continua'],
  mild: ['mild', 'slight', 'little', 'kidogo', 'leger', 'leve']
};

const EDUCATION_LIBRARY = {
  fever: {
    en: 'Keep drinking fluids, rest, and monitor temperature. Seek care faster if fever lasts more than two days or comes with red-flag symptoms.',
    sw: 'Kunywa maji ya kutosha, pumzika, na fuatilia joto la mwili. Tafuta huduma mapema ikiwa homa inaendelea zaidi ya siku mbili au ina dalili za hatari.',
    fr: 'Buvez suffisamment, reposez-vous et surveillez la temperature. Consultez plus vite si la fievre dure plus de deux jours ou s accompagne de signes de danger.',
    es: 'Beba suficientes liquidos, descansa y vigila la temperatura. Busca atencion antes si la fiebre dura mas de dos dias o aparece con signos de alarma.'
  },
  cough: {
    en: 'Warm fluids, rest, and avoiding smoke may help mild cough. Medical review is important if cough persists, is severe, or makes breathing hard.',
    sw: 'Vinywaji vya moto, mapumziko, na kuepuka moshi vinaweza kusaidia kikohozi kidogo. Tathmini ya kitabibu ni muhimu ikiwa kikohozi kinaendelea, ni kali, au kinafanya kupumua kuwa ngumu.',
    fr: 'Les boissons chaudes, le repos et l evitement de la fumee peuvent aider pour une toux legere. Une evaluation medicale est importante si la toux persiste, est severe ou gene la respiration.',
    es: 'Las bebidas tibias, el descanso y evitar el humo pueden ayudar con una tos leve. La evaluacion medica es importante si la tos persiste, es intensa o dificulta respirar.'
  },
  diabetes: {
    en: 'Stay consistent with prescribed medicines, meals, hydration, and glucose monitoring. Know your sick-day plan and when to seek urgent review for very high or low sugar.',
    sw: 'Fuata dawa ulizopewa, kula kwa mpangilio, kunywa maji, na pima sukari mara kwa mara. Jua mpango wako wa siku za ugonjwa na wakati wa kutafuta tathmini ya haraka kwa sukari ya juu au ya chini sana.',
    fr: 'Respectez les traitements prescrits, l alimentation, l hydratation et la surveillance de la glycemie. Connaissez votre plan en cas de maladie et le moment ou une evaluation urgente est necessaire pour une glycemie tres haute ou tres basse.',
    es: 'Mantente constante con los medicamentos indicados, la alimentacion, la hidratacion y el control de glucosa. Ten claro tu plan para dias de enfermedad y cuando buscar valoracion urgente.'
  },
  hypertension: {
    en: 'Take blood pressure medicine as prescribed, limit salt, stay active, and attend follow-up visits. Severe neurologic or chest symptoms are emergencies.',
    sw: 'Tumia dawa za presha kama ulivyoelekezwa, punguza chumvi, fanya mazoezi, na hudhuria ufuatiliaji. Dalili kali za mfumo wa fahamu au kifua ni dharura.',
    fr: 'Prenez les medicaments antihypertenseurs comme prescrits, limitez le sel, restez actif et suivez vos rendez-vous. Les symptomes neurologiques severes ou thoraciques sont des urgences.',
    es: 'Toma tus medicamentos para la presion como se indicaron, reduce la sal, mantente activo y acude a los controles. Los sintomas neurologicos intensos o toracicos son emergencias.'
  },
  asthma: {
    en: 'Use rescue and controller inhalers exactly as prescribed, avoid known triggers, and seek urgent care if breathing worsens or rescue medication is not helping.',
    sw: 'Tumia dawa za kuvuta za dharura na za kudhibiti kama ulivyoelekezwa, epuka vichochezi, na tafuta huduma ya haraka ikiwa kupumua kunazidi kuwa kugumu au dawa haisaidii.',
    fr: 'Utilisez les inhalateurs de secours et de fond comme prescrits, evitez les declencheurs connus et consultez rapidement si la respiration s aggrave ou si le traitement de secours n aide pas.',
    es: 'Usa los inhaladores de rescate y control exactamente como se indicaron, evita los desencadenantes conocidos y busca atencion urgente si la respiracion empeora o el inhalador de rescate no ayuda.'
  },
  wellness: {
    en: 'Healthy routines include hydration, balanced meals, sleep, movement, vaccinations, and regular checkups. A clinician can tailor these steps to your age and health conditions.',
    sw: 'Mazoea mazuri ya afya ni kunywa maji, kula mlo wenye usawa, kulala vizuri, kufanya mazoezi, chanjo, na uchunguzi wa mara kwa mara. Daktari anaweza kurekebisha ushauri huu kulingana na umri na hali yako.',
    fr: 'Les habitudes saines incluent l hydratation, une alimentation equilibree, le sommeil, l activite physique, la vaccination et des controles reguliers. Un clinicien peut adapter ces conseils a votre age et a votre etat de sante.',
    es: 'Los habitos saludables incluyen hidratacion, alimentacion equilibrada, sueno, movimiento, vacunas y controles regulares. Un profesional puede adaptar estos pasos a tu edad y condiciones de salud.'
  }
};

const TRANSLATIONS = {
  en: {
    dispositions: {
      emergency: 'This looks like an emergency situation.',
      urgent: 'This should be assessed urgently, ideally the same day.',
      routine: 'This deserves a routine clinical review.',
      self_care: 'This sounds suitable for home monitoring unless symptoms worsen.'
    },
    disclaimer: 'This guidance is informational and does not replace a licensed clinician evaluation.',
    emergencyLabel: 'Emergency support',
    nextStepLabel: 'Next steps',
    followUpLabel: 'Helpful follow-up',
    protocol: {
      immediate: 'Call local emergency services now or go to the nearest emergency department.',
      urgent: 'Arrange same-day medical assessment.',
      routine: 'Book a clinical review and monitor for changes.',
      self_care: 'Monitor symptoms, use self-care measures, and seek review if symptoms worsen.'
    }
  },
  sw: {
    dispositions: {
      emergency: 'Hii inaonekana kuwa hali ya dharura.',
      urgent: 'Hii inahitaji tathmini ya haraka, ikiwezekana siku hiyo hiyo.',
      routine: 'Hii inahitaji tathmini ya kawaida ya kitabibu.',
      self_care: 'Hii inaonekana kufaa kufuatiliwa nyumbani isipokuwa dalili zizidi.'
    },
    disclaimer: 'Huu ni mwongozo wa taarifa tu na hauchukui nafasi ya tathmini ya daktari.',
    emergencyLabel: 'Msaada wa dharura',
    nextStepLabel: 'Hatua zinazofuata',
    followUpLabel: 'Maswali ya kufuatilia',
    protocol: {
      immediate: 'Piga simu kwa huduma za dharura sasa au nenda kwenye idara ya dharura iliyo karibu.',
      urgent: 'Panga tathmini ya kitabibu siku hiyo hiyo.',
      routine: 'Panga tathmini ya kitabibu na ufuatilie mabadiliko ya dalili.',
      self_care: 'Fuatilia dalili, tumia hatua za kujitunza, na tafuta tathmini ikiwa dalili zizidi.'
    }
  },
  fr: {
    dispositions: {
      emergency: 'Cela ressemble a une situation d urgence.',
      urgent: 'Cela doit etre evalue rapidement, idealement le jour meme.',
      routine: 'Cela merite une evaluation clinique de routine.',
      self_care: 'Cela semble compatible avec une surveillance a domicile sauf aggravation.'
    },
    disclaimer: 'Ces informations servent de guide et ne remplacent pas une evaluation medicale.',
    emergencyLabel: 'Assistance urgente',
    nextStepLabel: 'Etapes suivantes',
    followUpLabel: 'Questions utiles',
    protocol: {
      immediate: 'Appelez les services d urgence locaux maintenant ou rendez-vous au service d urgence le plus proche.',
      urgent: 'Organisez une evaluation medicale le jour meme.',
      routine: 'Planifiez une consultation et surveillez toute aggravation.',
      self_care: 'Surveillez les symptomes, appliquez les mesures d auto-soins et consultez si la situation s aggrave.'
    }
  },
  es: {
    dispositions: {
      emergency: 'Esto parece una situacion de emergencia.',
      urgent: 'Esto debe evaluarse con urgencia, idealmente el mismo dia.',
      routine: 'Esto merece una revision clinica de rutina.',
      self_care: 'Esto parece compatible con control en casa salvo que empeore.'
    },
    disclaimer: 'Esta orientacion es informativa y no sustituye la evaluacion de un profesional sanitario.',
    emergencyLabel: 'Apoyo de emergencia',
    nextStepLabel: 'Siguientes pasos',
    followUpLabel: 'Preguntas utiles',
    protocol: {
      immediate: 'Llama ahora a los servicios de emergencia locales o acude al servicio de urgencias mas cercano.',
      urgent: 'Organiza una evaluacion medica el mismo dia.',
      routine: 'Agenda una revision clinica y vigila cambios.',
      self_care: 'Vigila los sintomas, usa medidas de autocuidado y busca revision si empeoran.'
    }
  }
};

const EMERGENCY_PATTERNS = [
  ['chest_pain'],
  ['shortness_of_breath'],
  ['bleeding'],
  ['confusion'],
  ['seizure'],
  ['pregnancy', 'bleeding'],
  ['high_blood_sugar', 'vomiting']
];

function hasPattern(entities, pattern) {
  return pattern.every((entity) => entities.includes(entity));
}

function normalizeText(text) {
  return String(text || '')
    .toLowerCase()
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .replace(/[^\w\s]/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();
}

function extractEntities(normalized, lexicon) {
  return Object.entries(lexicon)
    .filter(([, aliases]) => aliases.some((alias) => normalized.includes(normalizeText(alias))))
    .map(([entity]) => entity);
}

function unique(values) {
  return [...new Set(values)];
}

class MedicalChatbot {
  constructor(options = {}) {
    this.knowledgeBase = options.knowledgeBase || new MedicalKnowledgeBase();
    this.emergencyContacts = options.emergencyContacts || {
      en: {
        hotline: '911',
        instruction: 'Call local emergency services now or go to the nearest emergency department.',
        escalationTarget: 'Emergency department'
      },
      sw: {
        hotline: '112',
        instruction: 'Piga simu kwa huduma za dharura sasa au nenda kwenye idara ya dharura iliyo karibu.',
        escalationTarget: 'Idara ya dharura'
      },
      fr: {
        hotline: '112',
        instruction: 'Appelez les services d urgence locaux maintenant ou rendez-vous au service d urgence le plus proche.',
        escalationTarget: 'Service d urgence'
      },
      es: {
        hotline: '112',
        instruction: 'Llama ahora a los servicios de emergencia locales o acude al servicio de urgencias mas cercano.',
        escalationTarget: 'Servicio de urgencias'
      }
    };
  }

  getLocale(language) {
    return TRANSLATIONS[SUPPORTED_LANGUAGES.includes(language) ? language : 'en'];
  }

  detectLanguage(message, preferredLanguage) {
    if (preferredLanguage && SUPPORTED_LANGUAGES.includes(preferredLanguage)) {
      return preferredLanguage;
    }

    const normalized = normalizeText(message);
    const matched = LANGUAGE_PATTERNS.find(({ patterns }) => patterns.some((pattern) => normalized.includes(normalizeText(pattern))));
    return matched ? matched.language : 'en';
  }

  extractDurationDays(normalized) {
    const durationMatch = normalized.match(/\b(\d{1,2})\s*(day|days|siku|jour|jours|dia|dias)\b/);
    return durationMatch ? Number(durationMatch[1]) : null;
  }

  extractSeverity(normalized) {
    if (SEVERITY_PATTERNS.severe.some((pattern) => normalized.includes(pattern))) {
      return 'severe';
    }
    if (SEVERITY_PATTERNS.moderate.some((pattern) => normalized.includes(pattern))) {
      return 'moderate';
    }
    if (SEVERITY_PATTERNS.mild.some((pattern) => normalized.includes(pattern))) {
      return 'mild';
    }
    return 'unspecified';
  }

  extractProfileHints(normalized) {
    const profile = {};
    const ageMatch = normalized.match(/\b(\d{1,3})\s*(years old|yo|ans|yr|anos)\b/);

    if (ageMatch) {
      profile.age = Number(ageMatch[1]);
    }
    if (/\b(child|baby|infant|mtoto|enfant|nino|nina)\b/.test(normalized)) {
      profile.population = 'child';
    }
    if (/\b(elderly|older adult|senior|personne agee|adulto mayor)\b/.test(normalized)) {
      profile.population = 'older_adult';
    }
    if (/\b(pregnant|pregnancy|mjamzito|enceinte|embarazada|embarazo)\b/.test(normalized)) {
      profile.pregnant = true;
    }

    return profile;
  }

  parseMedicalQuery(message) {
    const normalized = normalizeText(message);
    const symptoms = extractEntities(normalized, SYMPTOM_LEXICON);
    const conditions = extractEntities(normalized, CONDITION_LEXICON);
    const severity = this.extractSeverity(normalized);
    const durationDays = this.extractDurationDays(normalized);

    const asksSymptomCheck = /\b(symptom|triage|what should i do|what do i do|naumwa|que faire|check|what should we do|should i worry|nifanye nini|que hago|debo preocuparme)\b/.test(normalized) || symptoms.length > 0;
    const asksEducation = /\b(educate|learn|prevent|manage|control|reduce|explain|about|health education|teach me|wellness|nifundishe|explique|explica|prevenir)\b/.test(normalized);
    const asksEmergency = /\b(emergency|urgent|help now|dharura|urgence|911|112|urgencia|auxilio)\b/.test(normalized);

    let intent = 'general_inquiry';
    if (asksEmergency && symptoms.length === 0) {
      intent = 'emergency_help';
    } else if (asksSymptomCheck) {
      intent = 'symptom_check';
    } else if (asksEducation || conditions.length > 0) {
      intent = 'health_education';
    }

    const confidence = Math.min(
      0.99,
      0.4
        + (symptoms.length * 0.15)
        + (conditions.length * 0.12)
        + (intent !== 'general_inquiry' ? 0.18 : 0)
        + (severity !== 'unspecified' ? 0.05 : 0)
    );

    return {
      normalized,
      intent,
      symptoms,
      conditions,
      severity,
      durationDays,
      profileHints: this.extractProfileHints(normalized),
      confidence
    };
  }

  assessEmergency(symptoms, parsed, profile) {
    const reasons = EMERGENCY_PATTERNS
      .filter((pattern) => hasPattern(symptoms, pattern))
      .map((pattern) => pattern.join('+'));

    if (parsed.severity === 'severe' && (symptoms.includes('shortness_of_breath') || symptoms.includes('chest_pain'))) {
      reasons.push('severe_respiratory_or_chest_distress');
    }
    if (profile.pregnant && symptoms.includes('abdominal_pain')) {
      reasons.push('pregnancy_with_abdominal_pain');
    }
    if (parsed.intent === 'emergency_help' && reasons.length === 0) {
      reasons.push('explicit_emergency_request');
    }

    return {
      detected: reasons.length > 0,
      reasons: unique(reasons)
    };
  }

  determineTriage(symptoms, parsed, emergencyDetected, profile) {
    if (emergencyDetected) {
      return 'emergency';
    }

    if (
      hasPattern(symptoms, ['fever', 'cough']) ||
      hasPattern(symptoms, ['vomiting', 'diarrhea']) ||
      hasPattern(symptoms, ['dizziness', 'headache']) ||
      hasPattern(symptoms, ['pregnancy', 'abdominal_pain']) ||
      (parsed.severity === 'severe' && symptoms.length > 0) ||
      (parsed.durationDays !== null && parsed.durationDays >= 3 && symptoms.length > 0)
    ) {
      return 'urgent';
    }

    if (symptoms.length > 0 || profile.pregnant || profile.population === 'child' || profile.population === 'older_adult') {
      return 'routine';
    }

    return 'self_care';
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
      knowledgeSummary: kbSummary,
      topic: educationKey
    };
  }

  personalizeEducation(baseText, profile, parsed, language) {
    const additions = [];

    if (profile.population === 'child') {
      additions.push(
        language === 'sw'
          ? 'Kwa mtoto, dalili zinazozidi au kukataa kunywa zinahitaji tathmini mapema.'
          : language === 'fr'
            ? 'Chez un enfant, l aggravation des symptomes ou le refus de boire justifient une evaluation precoce.'
            : language === 'es'
              ? 'En un nino, el empeoramiento de sintomas o no poder beber requiere valoracion temprana.'
              : 'For a child, worsening symptoms or poor fluid intake should be reviewed early.'
      );
    }

    if (profile.population === 'older_adult') {
      additions.push(
        language === 'sw'
          ? 'Kwa mtu mzima mwenye umri mkubwa, tathmini mapema ni muhimu zaidi kwa sababu ya hatari kubwa ya madhara.'
          : language === 'fr'
            ? 'Chez la personne agee, une evaluation precoce est encore plus importante en raison d un risque plus eleve de complications.'
            : language === 'es'
              ? 'En una persona mayor, una valoracion temprana es aun mas importante por el mayor riesgo de complicaciones.'
              : 'For an older adult, earlier clinical review is wise because complications can escalate faster.'
      );
    }

    if (profile.pregnant) {
      additions.push(
        language === 'sw'
          ? 'Ikiwa una ujauzito, mjulishe daktari au mkunga mapema kuhusu dalili mpya.'
          : language === 'fr'
            ? 'Si vous etes enceinte, informez rapidement votre medecin ou votre sage-femme de tout nouveau symptome.'
            : language === 'es'
              ? 'Si estas embarazada, informa pronto a tu clinico o matrona sobre cualquier sintoma nuevo.'
              : 'If you are pregnant, let a clinician or midwife know promptly about new symptoms.'
      );
    }

    if (Array.isArray(profile.chronicConditions) && profile.chronicConditions.length > 0) {
      const joined = profile.chronicConditions.join(', ');
      additions.push(
        language === 'sw'
          ? `Kwa kuwa una hali sugu kama ${joined}, fuata mpango wako wa matibabu na tafuta tathmini mapema ikiwa dalili zinaathiri udhibiti wa hali hiyo.`
          : language === 'fr'
            ? `Comme vous vivez avec des maladies chroniques telles que ${joined}, suivez votre plan de traitement et consultez plus tot si les symptomes perturbent leur controle.`
            : language === 'es'
              ? `Como vives con enfermedades cronicas como ${joined}, sigue tu plan de tratamiento y busca revision antes si los sintomas alteran su control.`
              : `Because you live with chronic conditions such as ${joined}, follow your care plan closely and seek review earlier if symptoms disrupt control.`
      );
    }

    if (parsed.durationDays !== null && parsed.durationDays >= 3) {
      additions.push(
        language === 'sw'
          ? 'Kwa kuwa dalili zimechukua siku kadhaa, tathmini ya kitabibu inafaa zaidi.'
          : language === 'fr'
            ? 'Comme les symptomes durent depuis plusieurs jours, une evaluation clinique devient plus importante.'
            : language === 'es'
              ? 'Como los sintomas llevan varios dias, una valoracion clinica cobra mayor importancia.'
              : 'Because symptoms have lasted several days, clinical review becomes more important.'
      );
    }

    return [baseText, ...additions].filter(Boolean).join(' ');
  }

  buildEmergencyProtocol(language, triage, emergency) {
    const locale = this.getLocale(language);
    const contact = this.emergencyContacts[language] || this.emergencyContacts.en;

    return {
      emergencyDetected: emergency.detected,
      reasons: emergency.reasons,
      hotline: emergency.detected ? contact.hotline : null,
      escalationTarget: emergency.detected ? contact.escalationTarget : null,
      protocol: emergency.detected ? contact.instruction : locale.protocol[triage],
      nextActions: emergency.detected
        ? [
            contact.instruction,
            locale.protocol.urgent
          ]
        : [locale.protocol[triage]]
    };
  }

  buildFollowUpQuestions(language, triage, parsed) {
    if (triage === 'emergency') {
      return [];
    }

    if (parsed.intent === 'symptom_check') {
      const localized = {
        en: [
          'When did the symptoms start?',
          'Are the symptoms getting worse, staying the same, or improving?',
          'Do you have any long-term conditions or medications that matter here?'
        ],
        sw: [
          'Dalili zilianza lini?',
          'Dalili zinazidi, hazibadiliki, au zinapungua?',
          'Una magonjwa ya muda mrefu au dawa zozote muhimu hapa?'
        ],
        fr: [
          'Quand les symptomes ont-ils commence ?',
          'Les symptomes s aggravent-ils, restent-ils stables ou s ameliorent-ils ?',
          'Avez-vous des maladies chroniques ou des traitements importants ici ?'
        ],
        es: [
          'Cuando empezaron los sintomas?',
          'Los sintomas estan empeorando, iguales o mejorando?',
          'Tienes enfermedades cronicas o medicamentos importantes para esto?'
        ]
      };

      return localized[language] || localized.en;
    }

    return [];
  }

  buildResponseText(language, triage, personalizedEducation, emergencyProtocol) {
    const locale = this.getLocale(language);

    return [
      locale.dispositions[triage],
      personalizedEducation,
      emergencyProtocol.protocol,
      locale.disclaimer
    ].filter(Boolean).join(' ');
  }

  async respond({ message, patientProfile = {}, preferredLanguage } = {}) {
    const startedAt = Date.now();
    const language = this.detectLanguage(message, preferredLanguage);
    const parsed = this.parseMedicalQuery(message);
    const mergedProfile = { ...parsed.profileHints, ...patientProfile };
    const emergency = this.assessEmergency(parsed.symptoms, parsed, mergedProfile);
    const triage = this.determineTriage(parsed.symptoms, parsed, emergency.detected, mergedProfile);
    const education = await this.generateEducation(parsed, language);
    const personalizedEducation = this.personalizeEducation(
      education.knowledgeSummary || education.guidance,
      mergedProfile,
      parsed,
      language
    );
    const emergencyProtocol = this.buildEmergencyProtocol(language, triage, emergency);
    const followUpQuestions = this.buildFollowUpQuestions(language, triage, parsed);
    const response = this.buildResponseText(language, triage, personalizedEducation, emergencyProtocol);

    return {
      language,
      intent: parsed.intent,
      triage,
      symptoms: parsed.symptoms,
      conditions: parsed.conditions,
      education: {
        topic: education.topic,
        personalized: personalizedEducation
      },
      escalation: emergencyProtocol,
      response,
      metadata: {
        responseTimeMs: Date.now() - startedAt,
        references: education.references,
        confidence: parsed.confidence,
        followUpQuestions
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
