const { MedicalChatbot } = require('../src/chatbot/medical_chatbot');
const { MedicalKnowledgeBase } = require('../src/chatbot/knowledge_base');

describe('MedicalChatbot', () => {
  test('understands medical symptom queries and triages them', async () => {
    const chatbot = new MedicalChatbot();
    const result = await chatbot.respond({
      message: 'I have fever and cough for two days, what should I do?'
    });

    expect(result.intent).toBe('symptom_check');
    expect(result.triage).toBe('urgent');
    expect(result.symptoms).toEqual(expect.arrayContaining(['fever', 'cough']));
    expect(result.response).toMatch(/same day|rapid/i);
  });

  test('detects emergencies and provides escalation protocol', async () => {
    const chatbot = new MedicalChatbot();
    const result = await chatbot.respond({
      message: 'I have chest pain and trouble breathing, help now'
    });

    expect(result.triage).toBe('emergency');
    expect(result.escalation.emergencyDetected).toBe(true);
    expect(result.escalation.protocol).toMatch(/emergency/i);
  });

  test('supports multilingual interactions', async () => {
    const chatbot = new MedicalChatbot();
    const swahiliResult = await chatbot.respond({
      message: 'Naumwa na homa na kikohozi, nifanye nini?'
    });
    const frenchResult = await chatbot.respond({
      message: 'J ai une douleur thoracique et de la toux'
    });

    expect(swahiliResult.language).toBe('sw');
    expect(swahiliResult.response).toMatch(/dharura|haraka|kitabibu/i);
    expect(frenchResult.language).toBe('fr');
    expect(frenchResult.response).toMatch(/urgence|evaluation/i);
  });

  test('creates personalized education using profile hints', async () => {
    const chatbot = new MedicalChatbot();
    const result = await chatbot.respond({
      message: 'I am pregnant and have mild fever. Explain what I should watch for.',
      patientProfile: { chronicConditions: ['hypertension'] }
    });

    expect(result.conditions).toContain('pregnancy');
    expect(result.response).toMatch(/pregnant|midwife/i);
  });

  test('integrates with an external knowledge base search hook', async () => {
    const knowledgeBase = new MedicalKnowledgeBase({
      externalSearch: async () => ([
        { id: 'external-1', topic: 'triage', score: 10, summary: 'External clinical summary.' }
      ])
    });
    const chatbot = new MedicalChatbot({ knowledgeBase });
    const result = await chatbot.respond({
      message: 'Teach me about diabetes care'
    });

    expect(result.metadata.references[0]).toHaveProperty('id', 'external-1');
    expect(result.response).toMatch(/External clinical summary|glucose|diabetes/i);
  });

  test('meets latency target for deterministic local responses', async () => {
    const chatbot = new MedicalChatbot();
    const startedAt = Date.now();
    const result = await chatbot.respond({
      message: 'I have a headache and feel dizzy today'
    });
    const elapsed = Date.now() - startedAt;

    expect(result.metadata.responseTimeMs).toBeLessThan(2000);
    expect(elapsed).toBeLessThan(2000);
  });

  test('maintains conversation accuracy above 90 percent on a curated validation set', async () => {
    const chatbot = new MedicalChatbot();
    const evaluation = await chatbot.evaluateAccuracy([
      {
        input: { message: 'I have fever and cough' },
        expected: { intent: 'symptom_check', triage: 'urgent', language: 'en', emergency: false }
      },
      {
        input: { message: 'Teach me about diabetes' },
        expected: { intent: 'health_education', triage: 'self_care', language: 'en', emergency: false }
      },
      {
        input: { message: 'Naumwa na homa' },
        expected: { intent: 'symptom_check', triage: 'routine', language: 'sw', emergency: false }
      },
      {
        input: { message: 'I have chest pain and shortness of breath' },
        expected: { intent: 'symptom_check', triage: 'emergency', language: 'en', emergency: true }
      },
      {
        input: { message: 'Bonjour, explique le diabete' },
        expected: { intent: 'health_education', triage: 'self_care', language: 'fr', emergency: false }
      },
      {
        input: { message: 'I am vomiting and have diarrhea' },
        expected: { intent: 'symptom_check', triage: 'urgent', language: 'en', emergency: false }
      },
      {
        input: { message: 'Pregnant with bleeding today' },
        expected: { intent: 'symptom_check', triage: 'emergency', language: 'en', emergency: true }
      },
      {
        input: { message: 'How can I reduce my high blood pressure?' },
        expected: { intent: 'health_education', triage: 'self_care', language: 'en', emergency: false }
      },
      {
        input: { message: 'J ai de la fievre et de la toux' },
        expected: { intent: 'symptom_check', triage: 'urgent', language: 'fr', emergency: false }
      },
      {
        input: { message: 'I feel healthy, teach me wellness basics' },
        expected: { intent: 'health_education', triage: 'self_care', language: 'en', emergency: false }
      },
      {
        input: { message: 'I have severe bleeding after an injury' },
        expected: { intent: 'symptom_check', triage: 'emergency', language: 'en', emergency: true }
      },
      {
        input: { message: 'Explain hypertension prevention for adults' },
        expected: { intent: 'health_education', triage: 'self_care', language: 'en', emergency: false }
      },
      {
        input: { message: 'Naumwa na kichwa na kizunguzungu' },
        expected: { intent: 'symptom_check', triage: 'urgent', language: 'sw', emergency: false }
      }
    ]);

    expect(evaluation.accuracy).toBeGreaterThan(0.9);
  });
});
