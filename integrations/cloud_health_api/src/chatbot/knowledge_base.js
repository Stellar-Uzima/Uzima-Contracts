const DEFAULT_ARTICLES = [
  {
    id: 'fever-home-care',
    topic: 'fever',
    keywords: ['fever', 'temperature', 'pyrexia', 'hot body', 'homa'],
    education: {
      en: 'Fever can happen during many infections. Rest, hydration, and temperature monitoring are important. Seek urgent care if fever is severe, persistent, or paired with breathing trouble, confusion, seizures, or chest pain.',
      sw: 'Homa inaweza kutokea wakati wa maambukizi mengi. Pumzika, kunywa maji ya kutosha, na fuatilia joto la mwili. Tafuta huduma ya haraka ikiwa homa ni kali, inaendelea, au inaambatana na shida ya kupumua, kuchanganyikiwa, degedege, au maumivu ya kifua.',
      fr: 'La fievre peut apparaitre lors de nombreuses infections. Repos, hydratation et surveillance de la temperature sont importants. Consultez en urgence si la fievre est severe, persistante ou accompagnee de difficultes respiratoires, confusion, convulsions ou douleur thoracique.',
      es: 'La fiebre puede aparecer durante muchas infecciones. El descanso, la hidratacion y vigilar la temperatura son importantes. Busca atencion urgente si la fiebre es intensa, persistente o se acompana de dificultad respiratoria, confusion, convulsiones o dolor toracico.'
    }
  },
  {
    id: 'cough-respiratory',
    topic: 'cough',
    keywords: ['cough', 'dry cough', 'wet cough', 'kikohozi', 'toux'],
    education: {
      en: 'A cough may come from viral illness, asthma, allergy, or pneumonia. Persistent cough, shortness of breath, bluish lips, or chest pain need fast evaluation.',
      sw: 'Kikohozi kinaweza kusababishwa na virusi, pumu, mzio, au nimonia. Kikohozi cha muda mrefu, shida ya kupumua, midomo kuwa ya bluu, au maumivu ya kifua vinahitaji tathmini ya haraka.',
      fr: 'La toux peut etre liee a une infection virale, a l asthme, a une allergie ou a une pneumonie. Une toux persistante, un essoufflement, des levres bleutees ou une douleur thoracique necessitent une evaluation rapide.',
      es: 'La tos puede relacionarse con una infeccion viral, asma, alergia o neumonia. Una tos persistente, falta de aire, labios azulados o dolor en el pecho requieren evaluacion rapida.'
    }
  },
  {
    id: 'diabetes-lifestyle',
    topic: 'diabetes',
    keywords: ['diabetes', 'blood sugar', 'glucose', 'kisukari', 'diabete'],
    education: {
      en: 'Diabetes care usually includes medication adherence, balanced meals, exercise, hydration, and regular blood sugar checks. Very high sugar with vomiting, confusion, or deep breathing needs urgent medical care.',
      sw: 'Huduma ya kisukari mara nyingi inajumuisha kutumia dawa vizuri, kula mlo wenye usawa, kufanya mazoezi, kunywa maji, na kupima sukari mara kwa mara. Sukari ikiwa juu sana pamoja na kutapika, kuchanganyikiwa, au kupumua kwa kina inahitaji huduma ya haraka.',
      fr: 'La prise en charge du diabete comprend generalement le respect des traitements, une alimentation equilibree, l exercice, l hydratation et le controle regulier de la glycemie. Une glycemie tres elevee avec vomissements, confusion ou respiration profonde exige des soins urgents.',
      es: 'El cuidado de la diabetes suele incluir adherencia al tratamiento, alimentacion equilibrada, ejercicio, hidratacion y controles regulares de glucosa. Una glucosa muy alta con vomitos, confusion o respiracion profunda requiere atencion urgente.'
    }
  },
  {
    id: 'hypertension-education',
    topic: 'hypertension',
    keywords: ['hypertension', 'high blood pressure', 'bp', 'presha', 'tension'],
    education: {
      en: 'High blood pressure may have no symptoms, so regular checks matter. Medicines, reduced salt, activity, and follow-up care help lower risk. Severe headache, weakness on one side, chest pain, or vision loss need emergency attention.',
      sw: 'Shinikizo la damu linaweza kutokuwa na dalili, hivyo vipimo vya mara kwa mara ni muhimu. Dawa, kupunguza chumvi, mazoezi, na ufuatiliaji husaidia kupunguza hatari. Kichwa kuuma sana, upande mmoja kuwa dhaifu, maumivu ya kifua, au kupoteza kuona vinahitaji huduma ya dharura.',
      fr: 'L hypertension peut ne provoquer aucun symptome, d ou l importance des controles reguliers. Les medicaments, la reduction du sel, l activite physique et le suivi reduisent le risque. Des maux de tete intenses, une faiblesse d un cote, une douleur thoracique ou une perte de vision necessitent une prise en charge urgente.',
      es: 'La presion arterial alta puede no dar sintomas, por lo que los controles regulares importan. Los medicamentos, reducir la sal, la actividad fisica y el seguimiento ayudan a reducir riesgo. Dolor de cabeza intenso, debilidad de un lado, dolor toracico o perdida de vision requieren atencion urgente.'
    }
  },
  {
    id: 'asthma-action-plan',
    topic: 'asthma',
    keywords: ['asthma', 'wheeze', 'pumu', 'asthme', 'asma'],
    education: {
      en: 'Asthma care works best with a written action plan, avoidance of triggers, and correct inhaler use. Shortness of breath that is worsening or not responding to a rescue inhaler needs urgent evaluation.',
      sw: 'Huduma ya pumu huwa bora zaidi ukiwa na mpango wa maandishi, ukiepuka vichochezi, na ukitumia inhaler kwa usahihi. Shida ya kupumua inayozidi au isiyopungua baada ya dawa ya dharura inahitaji tathmini ya haraka.',
      fr: 'La prise en charge de l asthme est meilleure avec un plan d action ecrit, l evitement des declencheurs et une bonne technique d inhalation. Un essoufflement qui s aggrave ou ne repond pas au traitement de secours exige une evaluation rapide.',
      es: 'El manejo del asma funciona mejor con un plan de accion escrito, evitar desencadenantes y usar bien los inhaladores. La falta de aire que empeora o no responde al inhalador de rescate requiere valoracion urgente.'
    }
  }
];

function normalizeText(text) {
  return String(text || '')
    .toLowerCase()
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .replace(/[^\w\s]/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();
}

class MedicalKnowledgeBase {
  constructor(options = {}) {
    this.articles = options.articles || DEFAULT_ARTICLES;
    this.externalSearch = options.externalSearch || null;
  }

  async search(query, language = 'en') {
    const normalized = normalizeText(query);
    const localMatches = this.articles
      .map((article) => {
        const score = article.keywords.reduce((total, keyword) => (
          normalized.includes(normalizeText(keyword)) ? total + 1 : total
        ), 0);

        return score > 0 ? {
          id: article.id,
          topic: article.topic,
          score,
          summary: article.education[language] || article.education.en
        } : null;
      })
      .filter(Boolean)
      .sort((a, b) => b.score - a.score);

    if (!this.externalSearch) {
      return localMatches;
    }

    const externalMatches = await this.externalSearch(query, language);
    return [...localMatches, ...(externalMatches || [])]
      .sort((a, b) => (b.score || 0) - (a.score || 0));
  }
}

module.exports = {
  MedicalKnowledgeBase,
  DEFAULT_ARTICLES
};
