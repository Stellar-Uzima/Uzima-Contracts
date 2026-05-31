const { MedicalChatbot } = require('../src/chatbot/medical_chatbot');
const { MedicalKnowledgeBase } = require('../src/chatbot/knowledge_base');

// ─── Shared fixtures ──────────────────────────────────────────────────────────

const EMERGENCY_SYMPTOMS = [
  'I have chest pain and trouble breathing, help now',
  'I have severe bleeding after an injury',
  'Pregnant with bleeding today',
  'I have chest pain and shortness of breath',
];

const URGENT_SYMPTOMS = [
  'I have fever and cough for two days, what should I do?',
  'I am vomiting and have diarrhea',
  'I have a headache and feel dizzy today',
];

const SELF_CARE_EDUCATION = [
  'Teach me about diabetes',
  'How can I reduce my high blood pressure?',
  'I feel healthy, teach me wellness basics',
  'Explain hypertension prevention for adults',
];

/** Validation set for accuracy benchmarking */
const ACCURACY_VALIDATION_SET = [
  {
    input: { message: 'I have fever and cough' },
    expected: { intent: 'symptom_check', triage: 'urgent', language: 'en', emergency: false },
  },
  {
    input: { message: 'Teach me about diabetes' },
    expected: { intent: 'health_education', triage: 'self_care', language: 'en', emergency: false },
  },
  {
    input: { message: 'Naumwa na homa' },
    expected: { intent: 'symptom_check', triage: 'routine', language: 'sw', emergency: false },
  },
  {
    input: { message: 'I have chest pain and shortness of breath' },
    expected: { intent: 'symptom_check', triage: 'emergency', language: 'en', emergency: true },
  },
  {
    input: { message: 'Bonjour, explique le diabete' },
    expected: { intent: 'health_education', triage: 'self_care', language: 'fr', emergency: false },
  },
  {
    input: { message: 'I am vomiting and have diarrhea' },
    expected: { intent: 'symptom_check', triage: 'urgent', language: 'en', emergency: false },
  },
  {
    input: { message: 'Pregnant with bleeding today' },
    expected: { intent: 'symptom_check', triage: 'emergency', language: 'en', emergency: true },
  },
  {
    input: { message: 'How can I reduce my high blood pressure?' },
    expected: { intent: 'health_education', triage: 'self_care', language: 'en', emergency: false },
  },
  {
    input: { message: 'J ai de la fievre et de la toux' },
    expected: { intent: 'symptom_check', triage: 'urgent', language: 'fr', emergency: false },
  },
  {
    input: { message: 'I feel healthy, teach me wellness basics' },
    expected: { intent: 'health_education', triage: 'self_care', language: 'en', emergency: false },
  },
  {
    input: { message: 'I have severe bleeding after an injury' },
    expected: { intent: 'symptom_check', triage: 'emergency', language: 'en', emergency: true },
  },
  {
    input: { message: 'Explain hypertension prevention for adults' },
    expected: { intent: 'health_education', triage: 'self_care', language: 'en', emergency: false },
  },
  {
    input: { message: 'Naumwa na kichwa na kizunguzungu' },
    expected: { intent: 'symptom_check', triage: 'urgent', language: 'sw', emergency: false },
  },
];

// ─── Helpers ──────────────────────────────────────────────────────────────────

/** Creates a fresh chatbot instance for each test to prevent state bleed */
function createChatbot(options = {}) {
  return new MedicalChatbot(options);
}

// ─── Suite ────────────────────────────────────────────────────────────────────

describe('MedicalChatbot', () => {
  let chatbot;

  beforeEach(() => {
    chatbot = createChatbot();
  });

  afterEach(async () => {
    if (typeof chatbot.destroy === 'function') {
      await chatbot.destroy();
    }
  });

  // ─── Symptom triage ─────────────────────────────────────────────────────────

  describe('Symptom Triage', () => {
    test('identifies symptom_check intent and urgent triage for fever + cough', async () => {
      const result = await chatbot.respond({
        message: 'I have fever and cough for two days, what should I do?',
      });

      expect(result.intent).toBe('symptom_check');
      expect(result.triage).toBe('urgent');
      expect(result.symptoms).toEqual(expect.arrayContaining(['fever', 'cough']));
      expect(result.response).toMatch(/same day|rapid/i);
    });

    test.each(URGENT_SYMPTOMS)(
      'classifies "%s" as urgent or higher',
      async (message) => {
        const result = await chatbot.respond({ message });
        expect(['urgent', 'emergency']).toContain(result.triage);
        expect(result.intent).toBe('symptom_check');
      }
    );

    test('classifies routine low-risk symptoms correctly', async () => {
      const result = await chatbot.respond({
        message: 'I have a mild runny nose for one day with no fever',
      });

      expect(result.intent).toBe('symptom_check');
      expect(['routine', 'self_care']).toContain(result.triage);
      expect(result.escalation?.emergencyDetected).toBeFalsy();
    });

    test('extracts multiple symptoms from a single message', async () => {
      const result = await chatbot.respond({
        message: 'I have fever, headache, joint pain, and loss of appetite',
      });

      expect(result.symptoms.length).toBeGreaterThanOrEqual(3);
      expect(result.symptoms).toEqual(
        expect.arrayContaining(['fever', 'headache'])
      );
    });

    test('returns a non-empty symptom list for every triage response', async () => {
      const result = await chatbot.respond({
        message: 'I have been feeling nauseous and tired since yesterday',
      });

      expect(Array.isArray(result.symptoms)).toBe(true);
      expect(result.symptoms.length).toBeGreaterThan(0);
    });
  });

  // ─── Emergency detection ────────────────────────────────────────────────────

  describe('Emergency Detection & Escalation', () => {
    test('detects emergency and provides full escalation protocol for chest pain', async () => {
      const result = await chatbot.respond({
        message: 'I have chest pain and trouble breathing, help now',
      });

      expect(result.triage).toBe('emergency');
      expect(result.escalation.emergencyDetected).toBe(true);
      expect(result.escalation.protocol).toMatch(/emergency/i);
      expect(result.escalation.hotline).toBeTruthy();
      expect(result.escalation.nextActions.length).toBeGreaterThan(0);
    });

    test.each(EMERGENCY_SYMPTOMS)(
      'flags "%s" as emergency',
      async (message) => {
        const result = await chatbot.respond({ message });
        expect(result.triage).toBe('emergency');
        expect(result.escalation.emergencyDetected).toBe(true);
      }
    );

    test('emergency response always includes a non-empty hotline number', async () => {
      for (const message of EMERGENCY_SYMPTOMS) {
        const result = await chatbot.respond({ message });
        expect(typeof result.escalation.hotline).toBe('string');
        expect(result.escalation.hotline.trim().length).toBeGreaterThan(0);
      }
    });

    test('emergency response always lists at least 2 immediate next actions', async () => {
      const result = await chatbot.respond({
        message: 'I have severe chest pain radiating to my arm',
      });
      expect(result.escalation.nextActions.length).toBeGreaterThanOrEqual(2);
    });

    test('does not flag routine symptoms as emergency', async () => {
      const result = await chatbot.respond({
        message: 'I have a mild cold and slight throat irritation',
      });
      expect(result.triage).not.toBe('emergency');
      expect(result.escalation?.emergencyDetected).not.toBe(true);
    });
  });

  // ─── Multilingual support ───────────────────────────────────────────────────

  describe('Multilingual Interactions', () => {
    test('detects Swahili and responds in Swahili for fever + cough', async () => {
      const result = await chatbot.respond({
        message: 'Naumwa na homa na kikohozi, nifanye nini?',
      });

      expect(result.language).toBe('sw');
      expect(result.response).toMatch(/dharura|haraka|kitabibu/i);
    });

    test('detects French and responds in French for chest pain + cough', async () => {
      const result = await chatbot.respond({
        message: 'J ai une douleur thoracique et de la toux',
      });

      expect(result.language).toBe('fr');
      expect(result.response).toMatch(/urgence|evaluation/i);
    });

    test('respects explicit preferredLanguage for Spanish health education', async () => {
      const result = await chatbot.respond({
        message: 'Necesito educacion sobre diabetes',
        preferredLanguage: 'es',
      });

      expect(result.language).toBe('es');
      expect(result.intent).toBe('health_education');
      expect(result.response).toMatch(/diabetes|glucosa|orientacion/i);
    });

    test('preferredLanguage overrides auto-detected language', async () => {
      // Message is in English but preferred language is French
      const result = await chatbot.respond({
        message: 'Tell me about diabetes',
        preferredLanguage: 'fr',
      });

      expect(result.language).toBe('fr');
    });

    test('falls back to English for unrecognised language codes', async () => {
      const result = await chatbot.respond({
        message: 'Tell me about fever',
        preferredLanguage: 'xx', // unsupported code
      });

      expect(result.language).toBe('en');
      expect(result.response).toBeTruthy();
    });

    test('returns a valid response for each core supported language', async () => {
      const cases = [
        { message: 'I have a fever', preferredLanguage: 'en', expected: 'en' },
        { message: 'J ai de la fievre', preferredLanguage: 'fr', expected: 'fr' },
        { message: 'Naumwa na homa', expected: 'sw' },
        { message: 'Tengo fiebre', preferredLanguage: 'es', expected: 'es' },
      ];

      for (const { message, preferredLanguage, expected } of cases) {
        const result = await chatbot.respond({ message, preferredLanguage });
        expect(result.language).toBe(expected);
        expect(result.response.trim().length).toBeGreaterThan(0);
      }
    });
  });

  // ─── Personalisation ────────────────────────────────────────────────────────

  describe('Personalised Responses', () => {
    test('incorporates pregnancy condition into response and education', async () => {
      const result = await chatbot.respond({
        message: 'I am pregnant and have mild fever. Explain what I should watch for.',
        patientProfile: { chronicConditions: ['hypertension'] },
      });

      expect(result.conditions).toContain('pregnancy');
      expect(result.response).toMatch(/pregnant|midwife/i);
      expect(result.education.personalized).toMatch(/hypertension|care plan/i);
    });

    test('adjusts triage threshold for high-risk profiles', async () => {
      const standardResult = await chatbot.respond({
        message: 'I have a mild headache',
      });
      const highRiskResult = await chatbot.respond({
        message: 'I have a mild headache',
        patientProfile: { chronicConditions: ['hypertension', 'diabetes'] },
      });

      // High-risk patient should receive same or higher triage urgency
      const triageOrder = ['self_care', 'routine', 'urgent', 'emergency'];
      const standardIdx = triageOrder.indexOf(standardResult.triage);
      const highRiskIdx = triageOrder.indexOf(highRiskResult.triage);
      expect(highRiskIdx).toBeGreaterThanOrEqual(standardIdx);
    });

    test('includes chronic conditions in education personalisation', async () => {
      const result = await chatbot.respond({
        message: 'Teach me about staying healthy',
        patientProfile: { chronicConditions: ['diabetes', 'hypertension'] },
      });

      expect(result.education.personalized).toMatch(/diabetes|hypertension/i);
    });

    test('education response for patient with no profile is still coherent', async () => {
      const result = await chatbot.respond({
        message: 'Teach me about nutrition',
      });

      expect(result.intent).toBe('health_education');
      expect(result.response.trim().length).toBeGreaterThan(0);
    });
  });

  // ─── Knowledge base integration ─────────────────────────────────────────────

  describe('Knowledge Base Integration', () => {
    test('surfaces references from an external search hook', async () => {
      const knowledgeBase = new MedicalKnowledgeBase({
        externalSearch: async () => [
          {
            id: 'external-1',
            topic: 'triage',
            score: 10,
            summary: 'External clinical summary.',
          },
        ],
      });
      const bot = createChatbot({ knowledgeBase });

      const result = await bot.respond({ message: 'Teach me about diabetes care' });

      expect(result.metadata.references[0]).toHaveProperty('id', 'external-1');
      expect(result.response).toMatch(/External clinical summary|glucose|diabetes/i);
    });

    test('degrades gracefully when external search throws', async () => {
      const knowledgeBase = new MedicalKnowledgeBase({
        externalSearch: async () => {
          throw new Error('External KB unavailable');
        },
      });
      const bot = createChatbot({ knowledgeBase });

      // Should not throw — should fall back to local knowledge
      const result = await bot.respond({ message: 'Teach me about diabetes care' });
      expect(result.response.trim().length).toBeGreaterThan(0);
      expect(result.metadata.references).toBeDefined();
    });

    test('degrades gracefully when external search returns empty results', async () => {
      const knowledgeBase = new MedicalKnowledgeBase({
        externalSearch: async () => [],
      });
      const bot = createChatbot({ knowledgeBase });

      const result = await bot.respond({ message: 'Teach me about malaria prevention' });
      expect(result.response.trim().length).toBeGreaterThan(0);
    });

    test('references array is always defined on metadata', async () => {
      const result = await chatbot.respond({ message: 'I have a sore throat' });
      expect(Array.isArray(result.metadata.references)).toBe(true);
    });
  });

  // ─── Conversation context ───────────────────────────────────────────────────

  describe('Conversation Context & Follow-ups', () => {
    test('maintains context across a multi-turn conversation', async () => {
      const sessionId = 'session-context-01';

      const turn1 = await chatbot.respond({
        message: 'I have been having chest pain since morning',
        sessionId,
      });
      expect(turn1.triage).toBe('emergency');

      const turn2 = await chatbot.respond({
        message: 'It also spreads to my left arm',
        sessionId,
      });
      // Follow-up should retain emergency context
      expect(turn2.triage).toBe('emergency');
      expect(turn2.escalation?.emergencyDetected).toBe(true);
    });

    test('correctly resolves ambiguous follow-up using prior context', async () => {
      const sessionId = 'session-context-02';

      await chatbot.respond({
        message: 'I have been diagnosed with diabetes',
        sessionId,
      });
      const followUp = await chatbot.respond({
        message: 'What foods should I avoid?',
        sessionId,
      });

      expect(followUp.intent).toBe('health_education');
      expect(followUp.response).toMatch(/diabetes|sugar|glucose/i);
    });

    test('separate sessions do not share context', async () => {
      await chatbot.respond({
        message: 'I have severe chest pain',
        sessionId: 'session-A',
      });

      const sessionB = await chatbot.respond({
        message: 'I feel fine, just checking in',
        sessionId: 'session-B',
      });

      expect(sessionB.triage).not.toBe('emergency');
      expect(sessionB.escalation?.emergencyDetected).not.toBe(true);
    });
  });

  // ─── Audit trail ───────────────────────────────────────────────────────────

  describe('Audit Trail & Compliance Metadata', () => {
    test('response includes a unique interaction ID', async () => {
      const r1 = await chatbot.respond({ message: 'I have fever' });
      const r2 = await chatbot.respond({ message: 'I have fever' });

      expect(r1.metadata.interactionId).toBeTruthy();
      expect(r2.metadata.interactionId).toBeTruthy();
      expect(r1.metadata.interactionId).not.toBe(r2.metadata.interactionId);
    });

    test('response includes a timestamp within the last 5 seconds', async () => {
      const before = Date.now();
      const result = await chatbot.respond({ message: 'I have fever' });
      const after = Date.now();

      const ts = new Date(result.metadata.timestamp).getTime();
      expect(ts).toBeGreaterThanOrEqual(before);
      expect(ts).toBeLessThanOrEqual(after + 100); // small tolerance
    });

    test('response includes model version or source identifier', async () => {
      const result = await chatbot.respond({ message: 'I have fever' });
      expect(result.metadata.modelVersion ?? result.metadata.source).toBeTruthy();
    });

    test('emergency responses log escalation reason in audit metadata', async () => {
      const result = await chatbot.respond({
        message: 'I have severe chest pain and cannot breathe',
      });

      expect(result.triage).toBe('emergency');
      expect(result.metadata.auditFlags ?? result.metadata.escalationReason).toBeTruthy();
    });
  });

  // ─── Input validation & edge cases ─────────────────────────────────────────

  describe('Input Validation & Edge Cases', () => {
    test('returns a safe error response for an empty message', async () => {
      const result = await chatbot.respond({ message: '' });
      expect(result.response.trim().length).toBeGreaterThan(0);
      expect(result.intent).toBeDefined();
    });

    test('handles very long input without throwing', async () => {
      const longMessage = 'I have fever and cough. '.repeat(200);
      const result = await chatbot.respond({ message: longMessage });
      expect(result.response.trim().length).toBeGreaterThan(0);
    });

    test('handles messages with special characters gracefully', async () => {
      const result = await chatbot.respond({
        message: '<<I have fever & cough!! [urgent?] 🤒>>',
      });
      expect(result.intent).toBeDefined();
      expect(result.response.trim().length).toBeGreaterThan(0);
    });

    test('handles null patientProfile without throwing', async () => {
      const result = await chatbot.respond({
        message: 'I have a headache',
        patientProfile: null,
      });
      expect(result.intent).toBeDefined();
    });

    test('response confidence is between 0 and 1 inclusive', async () => {
      const result = await chatbot.respond({ message: 'I have a headache' });
      expect(result.metadata.confidence).toBeGreaterThanOrEqual(0);
      expect(result.metadata.confidence).toBeLessThanOrEqual(1);
    });
  });

  // ─── Performance ────────────────────────────────────────────────────────────

  describe('Performance', () => {
    test('responds within 2000ms with confidence > 0.5 for a common symptom', async () => {
      const start = Date.now();
      const result = await chatbot.respond({
        message: 'I have a headache and feel dizzy today',
      });
      const elapsed = Date.now() - start;

      expect(result.metadata.responseTimeMs).toBeLessThan(2000);
      expect(elapsed).toBeLessThan(2000);
      expect(result.metadata.confidence).toBeGreaterThan(0.5);
    });

    test('handles 10 concurrent requests without throwing', async () => {
      const requests = Array.from({ length: 10 }, (_, i) =>
        chatbot.respond({ message: `I have symptom number ${i}`, sessionId: `sess-${i}` })
      );

      const results = await Promise.allSettled(requests);
      const failures = results.filter((r) => r.status === 'rejected');
      expect(failures.length).toBe(0);
    });

    test('p95 response time stays under 3000ms across 20 sequential requests', async () => {
      const messages = [
        ...URGENT_SYMPTOMS,
        ...SELF_CARE_EDUCATION,
        'I have a sore throat',
        'My child has a rash',
        'I feel faint when standing',
        'I have been losing weight unexpectedly',
        'I have lower back pain for a week',
        'I get shortness of breath when climbing stairs',
        'I have been feeling anxious and cannot sleep',
        'Teach me about malaria prevention',
      ].slice(0, 20);

      const times = [];
      for (const message of messages) {
        const start = Date.now();
        await chatbot.respond({ message });
        times.push(Date.now() - start);
      }

      times.sort((a, b) => a - b);
      const p95 = times[Math.ceil(times.length * 0.95) - 1];
      expect(p95).toBeLessThan(3000);
    });
  });

  // ─── Accuracy benchmark ─────────────────────────────────────────────────────

  describe('Accuracy Benchmark', () => {
    test('maintains >90% accuracy on the curated validation set', async () => {
      const evaluation = await chatbot.evaluateAccuracy(ACCURACY_VALIDATION_SET);
      expect(evaluation.accuracy).toBeGreaterThan(0.9);
    });

    test('accuracy report includes per-case pass/fail breakdown', async () => {
      const evaluation = await chatbot.evaluateAccuracy(ACCURACY_VALIDATION_SET);

      expect(Array.isArray(evaluation.results)).toBe(true);
      expect(evaluation.results.length).toBe(ACCURACY_VALIDATION_SET.length);

      for (const caseResult of evaluation.results) {
        expect(caseResult).toHaveProperty('passed');
        expect(typeof caseResult.passed).toBe('boolean');
      }
    });

    test('achieves >95% emergency detection recall on the validation set', async () => {
      const emergencyCases = ACCURACY_VALIDATION_SET.filter((c) => c.expected.emergency);
      const evaluation = await chatbot.evaluateAccuracy(emergencyCases);

      // Recall = correctly detected emergencies / total emergencies
      const detected = evaluation.results.filter(
        (r) => r.actual?.triage === 'emergency' || r.actual?.escalation?.emergencyDetected
      ).length;

      const recall = detected / emergencyCases.length;
      expect(recall).toBeGreaterThan(0.95);
    });

    test('achieves >85% intent classification accuracy on the validation set', async () => {
      const evaluation = await chatbot.evaluateAccuracy(ACCURACY_VALIDATION_SET);
      const intentMatches = evaluation.results.filter(
        (r) => r.actual?.intent === ACCURACY_VALIDATION_SET[evaluation.results.indexOf(r)]?.expected?.intent
      ).length;

      const intentAccuracy = intentMatches / ACCURACY_VALIDATION_SET.length;
      expect(intentAccuracy).toBeGreaterThan(0.85);
    });

    test('achieves >85% language detection accuracy on the validation set', async () => {
      const evaluation = await chatbot.evaluateAccuracy(ACCURACY_VALIDATION_SET);
      const langMatches = evaluation.results.filter(
        (r) => r.actual?.language === ACCURACY_VALIDATION_SET[evaluation.results.indexOf(r)]?.expected?.language
      ).length;

      const langAccuracy = langMatches / ACCURACY_VALIDATION_SET.length;
      expect(langAccuracy).toBeGreaterThan(0.85);
    });
  });

  // ─── Rate limiting / abuse prevention ──────────────────────────────────────

  describe('Rate Limiting & Abuse Prevention', () => {
    test('responds to rapid-fire requests without crashing', async () => {
      const burst = Array.from({ length: 50 }, () =>
        chatbot.respond({ message: 'I have fever', sessionId: 'burst-session' })
      );
      const results = await Promise.allSettled(burst);
      // All should either resolve or be gracefully rate-limited (not throw)
      for (const r of results) {
        if (r.status === 'rejected') {
          // Rate-limit rejections must include a readable message
          expect(r.reason?.message).toMatch(/rate|limit|too many/i);
        }
      }
    });

    test('does not expose internal stack traces in error responses', async () => {
      // Simulate an invalid input that might trigger an internal error
      const result = await chatbot.respond({ message: null });
      expect(JSON.stringify(result)).not.toMatch(/at Object\.|at Module\.|\.js:\d+/);
    });
  });
});