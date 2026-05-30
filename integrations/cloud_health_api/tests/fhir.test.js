const { FhirTransformer } = require('../src/transform/fhir');

// ─── Fixtures ─────────────────────────────────────────────────────────────────

const VALID_PATIENT_INPUT = {
  id: 'p1',
  name: 'Alice Njeri',
  dob: '1970-01-01',
  gender: 'female',
  phone: '+254700000001',
  email: 'alice@example.com',
  address: { city: 'Nairobi', country: 'KE' },
};

const VALID_OBSERVATION_INPUT = {
  id: 'obs-1',
  patientId: 'p1',
  code: '8310-5',
  display: 'Body temperature',
  value: 38.5,
  unit: 'Cel',
  status: 'final',
  effectiveDate: '2024-06-01T10:00:00Z',
};

const VALID_ENCOUNTER_INPUT = {
  id: 'enc-1',
  patientId: 'p1',
  status: 'finished',
  class: 'AMB',
  startDate: '2024-06-01T09:00:00Z',
  endDate: '2024-06-01T10:00:00Z',
  reasonCode: 'fever',
};

const VALID_CONDITION_INPUT = {
  id: 'cond-1',
  patientId: 'p1',
  code: '386661006',
  display: 'Fever',
  clinicalStatus: 'active',
  onsetDate: '2024-05-30',
};

// ─── Suite ────────────────────────────────────────────────────────────────────

describe('FhirTransformer', () => {
  let t;

  beforeEach(() => {
    t = new FhirTransformer();
  });

  // ─── Patient ──────────────────────────────────────────────────────────────

  describe('Patient Resource', () => {
    test('transforms a minimal patient input to a valid FHIR Patient', () => {
      const input = { id: 'p1', name: 'Alice', dob: '1970-01-01' };
      const out = t.transformToFhir(input, 'Patient');

      expect(out.resourceType).toBe('Patient');
      expect(out.id).toBeDefined();
      expect(out.name[0].text).toBe('Alice');
    });

    test('maps full patient input fields to correct FHIR paths', () => {
      const out = t.transformToFhir(VALID_PATIENT_INPUT, 'Patient');

      expect(out.resourceType).toBe('Patient');
      expect(out.id).toBe('p1');
      expect(out.name[0].text).toBe('Alice Njeri');
      expect(out.birthDate).toBe('1970-01-01');
      expect(out.gender).toBe('female');
      expect(out.telecom).toEqual(
        expect.arrayContaining([
          expect.objectContaining({ system: 'phone', value: '+254700000001' }),
          expect.objectContaining({ system: 'email', value: 'alice@example.com' }),
        ])
      );
      expect(out.address[0].city).toBe('Nairobi');
      expect(out.address[0].country).toBe('KE');
    });

    test('preserves the source id in the FHIR resource id', () => {
      const out = t.transformToFhir({ id: 'xyz-99', name: 'Bob' }, 'Patient');
      expect(out.id).toBe('xyz-99');
    });

    test('splits a full name into family and given name parts', () => {
      const out = t.transformToFhir({ id: 'p2', name: 'John Doe' }, 'Patient');
      const name = out.name[0];

      expect(name.family).toBe('Doe');
      expect(name.given).toEqual(expect.arrayContaining(['John']));
    });

    test('generates a unique id when source id is missing', () => {
      const out = t.transformToFhir({ name: 'No ID Patient' }, 'Patient');
      expect(typeof out.id).toBe('string');
      expect(out.id.trim().length).toBeGreaterThan(0);
    });

    test('includes meta.lastUpdated timestamp on transformed resource', () => {
      const before = new Date().toISOString();
      const out = t.transformToFhir(VALID_PATIENT_INPUT, 'Patient');
      const after = new Date().toISOString();

      expect(out.meta?.lastUpdated).toBeDefined();
      expect(out.meta.lastUpdated >= before).toBe(true);
      expect(out.meta.lastUpdated <= after).toBe(true);
    });
  });

  // ─── Observation ──────────────────────────────────────────────────────────

  describe('Observation Resource', () => {
    test('transforms observation input to a valid FHIR Observation', () => {
      const out = t.transformToFhir(VALID_OBSERVATION_INPUT, 'Observation');

      expect(out.resourceType).toBe('Observation');
      expect(out.id).toBe('obs-1');
      expect(out.status).toBe('final');
      expect(out.subject.reference).toBe('Patient/p1');
      expect(out.code.coding[0].code).toBe('8310-5');
      expect(out.code.coding[0].display).toBe('Body temperature');
      expect(out.valueQuantity.value).toBe(38.5);
      expect(out.valueQuantity.unit).toBe('Cel');
      expect(out.effectiveDateTime).toBe('2024-06-01T10:00:00Z');
    });

    test('maps observation status correctly for all standard values', () => {
      const statuses = ['registered', 'preliminary', 'final', 'amended', 'cancelled'];

      for (const status of statuses) {
        const out = t.transformToFhir(
          { ...VALID_OBSERVATION_INPUT, status },
          'Observation'
        );
        expect(out.status).toBe(status);
      }
    });
  });

  // ─── Encounter ────────────────────────────────────────────────────────────

  describe('Encounter Resource', () => {
    test('transforms encounter input to a valid FHIR Encounter', () => {
      const out = t.transformToFhir(VALID_ENCOUNTER_INPUT, 'Encounter');

      expect(out.resourceType).toBe('Encounter');
      expect(out.id).toBe('enc-1');
      expect(out.status).toBe('finished');
      expect(out.subject.reference).toBe('Patient/p1');
      expect(out.period.start).toBe('2024-06-01T09:00:00Z');
      expect(out.period.end).toBe('2024-06-01T10:00:00Z');
      expect(out.class.code).toBe('AMB');
    });
  });

  // ─── Condition ────────────────────────────────────────────────────────────

  describe('Condition Resource', () => {
    test('transforms condition input to a valid FHIR Condition', () => {
      const out = t.transformToFhir(VALID_CONDITION_INPUT, 'Condition');

      expect(out.resourceType).toBe('Condition');
      expect(out.id).toBe('cond-1');
      expect(out.subject.reference).toBe('Patient/p1');
      expect(out.code.coding[0].code).toBe('386661006');
      expect(out.code.coding[0].display).toBe('Fever');
      expect(out.clinicalStatus.coding[0].code).toBe('active');
      expect(out.onsetDateTime).toBe('2024-05-30');
    });
  });

  // ─── Validation ───────────────────────────────────────────────────────────

  describe('FHIR Resource Validation', () => {
    test('validates a well-formed FHIR Patient as valid', () => {
      const resource = { id: 'p1', resourceType: 'Patient' };
      const v = t.validateFhirResource(resource);
      expect(v.valid).toBe(true);
      expect(v.errors).toHaveLength(0);
    });

    test('validates a fully-transformed Patient as valid', () => {
      const out = t.transformToFhir(VALID_PATIENT_INPUT, 'Patient');
      const v = t.validateFhirResource(out);
      expect(v.valid).toBe(true);
    });

    test('fails validation when resourceType is missing', () => {
      const v = t.validateFhirResource({ id: 'p1' });
      expect(v.valid).toBe(false);
      expect(v.errors).toEqual(
        expect.arrayContaining([expect.stringMatching(/resourceType/i)])
      );
    });

    test('fails validation when id is missing', () => {
      const v = t.validateFhirResource({ resourceType: 'Patient' });
      expect(v.valid).toBe(false);
      expect(v.errors).toEqual(
        expect.arrayContaining([expect.stringMatching(/id/i)])
      );
    });

    test('fails validation for an unrecognised resourceType', () => {
      const v = t.validateFhirResource({ id: 'x1', resourceType: 'FakeResource' });
      expect(v.valid).toBe(false);
      expect(v.errors.length).toBeGreaterThan(0);
    });

    test('fails validation for Observation missing subject reference', () => {
      const out = t.transformToFhir(
        { ...VALID_OBSERVATION_INPUT, patientId: undefined },
        'Observation'
      );
      const v = t.validateFhirResource(out);
      expect(v.valid).toBe(false);
      expect(v.errors).toEqual(
        expect.arrayContaining([expect.stringMatching(/subject/i)])
      );
    });

    test('returns errors as an array even when only one error exists', () => {
      const v = t.validateFhirResource({ id: 'x' });
      expect(Array.isArray(v.errors)).toBe(true);
    });

    test('validation result always includes a valid boolean and errors array', () => {
      const cases = [
        { id: 'p1', resourceType: 'Patient' },
        { id: 'x' },
        {},
        null,
      ];
      for (const input of cases) {
        const v = t.validateFhirResource(input);
        expect(typeof v.valid).toBe('boolean');
        expect(Array.isArray(v.errors)).toBe(true);
      }
    });
  });

  // ─── Round-trip integrity ─────────────────────────────────────────────────

  describe('Round-trip Integrity', () => {
    test('transformed Patient passes validation', () => {
      const out = t.transformToFhir(VALID_PATIENT_INPUT, 'Patient');
      const v = t.validateFhirResource(out);
      expect(v.valid).toBe(true);
      expect(v.errors).toHaveLength(0);
    });

    test('transformed Observation passes validation', () => {
      const out = t.transformToFhir(VALID_OBSERVATION_INPUT, 'Observation');
      const v = t.validateFhirResource(out);
      expect(v.valid).toBe(true);
    });

    test('transformed Encounter passes validation', () => {
      const out = t.transformToFhir(VALID_ENCOUNTER_INPUT, 'Encounter');
      const v = t.validateFhirResource(out);
      expect(v.valid).toBe(true);
    });

    test('transformed Condition passes validation', () => {
      const out = t.transformToFhir(VALID_CONDITION_INPUT, 'Condition');
      const v = t.validateFhirResource(out);
      expect(v.valid).toBe(true);
    });

    test('two transforms of the same input produce structurally identical output', () => {
      const out1 = t.transformToFhir(VALID_PATIENT_INPUT, 'Patient');
      const out2 = t.transformToFhir(VALID_PATIENT_INPUT, 'Patient');

      // Exclude meta.lastUpdated which is timestamp-dependent
      const strip = (o) => { const c = { ...o }; delete c.meta; return c; };
      expect(strip(out1)).toEqual(strip(out2));
    });
  });

  // ─── Batch transforms ─────────────────────────────────────────────────────

  describe('Batch Transforms', () => {
    test('transforms an array of patient inputs into a FHIR Bundle', () => {
      const inputs = [
        { id: 'p1', name: 'Alice', dob: '1990-01-01' },
        { id: 'p2', name: 'Bob', dob: '1985-05-15' },
        { id: 'p3', name: 'Carol', dob: '2000-11-30' },
      ];

      const bundle = t.transformBatch(inputs, 'Patient');

      expect(bundle.resourceType).toBe('Bundle');
      expect(bundle.type).toBe('collection');
      expect(bundle.entry).toHaveLength(3);
      expect(bundle.entry[0].resource.resourceType).toBe('Patient');
      expect(bundle.entry[1].resource.name[0].text).toBe('Bob');
    });

    test('batch transform produces a bundle with a unique id', () => {
      const bundle = t.transformBatch(
        [{ id: 'p1', name: 'Alice' }],
        'Patient'
      );
      expect(typeof bundle.id).toBe('string');
      expect(bundle.id.trim().length).toBeGreaterThan(0);
    });

    test('batch transform on empty array returns an empty Bundle', () => {
      const bundle = t.transformBatch([], 'Patient');
      expect(bundle.resourceType).toBe('Bundle');
      expect(bundle.entry).toHaveLength(0);
    });

    test('all entries in a batch bundle pass individual validation', () => {
      const inputs = [VALID_PATIENT_INPUT, { id: 'p2', name: 'Bob' }];
      const bundle = t.transformBatch(inputs, 'Patient');

      for (const entry of bundle.entry) {
        const v = t.validateFhirResource(entry.resource);
        expect(v.valid).toBe(true);
      }
    });
  });

  // ─── Error handling ───────────────────────────────────────────────────────

  describe('Error Handling', () => {
    test('throws or returns an error result for an unsupported resource type', () => {
      expect(() => {
        t.transformToFhir({ id: 'x1', name: 'Test' }, 'UnsupportedResource');
      }).toThrow();
    });

    test('handles null input without an unhandled exception', () => {
      expect(() => t.transformToFhir(null, 'Patient')).toThrow();
    });

    test('handles undefined input without an unhandled exception', () => {
      expect(() => t.transformToFhir(undefined, 'Patient')).toThrow();
    });

    test('handles empty object input and still returns a resourceType', () => {
      const out = t.transformToFhir({}, 'Patient');
      expect(out.resourceType).toBe('Patient');
    });

    test('does not mutate the original input object', () => {
      const input = { id: 'p1', name: 'Alice', dob: '1970-01-01' };
      const original = JSON.parse(JSON.stringify(input));
      t.transformToFhir(input, 'Patient');
      expect(input).toEqual(original);
    });
  });

  // ─── Output structure ────────────────────────────────────────────────────

  describe('Output Structure Compliance', () => {
    test('every transformed resource includes resourceType, id, and meta', () => {
      const resources = [
        t.transformToFhir(VALID_PATIENT_INPUT, 'Patient'),
        t.transformToFhir(VALID_OBSERVATION_INPUT, 'Observation'),
        t.transformToFhir(VALID_ENCOUNTER_INPUT, 'Encounter'),
        t.transformToFhir(VALID_CONDITION_INPUT, 'Condition'),
      ];

      for (const resource of resources) {
        expect(resource.resourceType).toBeDefined();
        expect(resource.id).toBeDefined();
        expect(resource.meta).toBeDefined();
      }
    });

    test('transformed resources do not contain non-FHIR snake_case keys', () => {
      const out = t.transformToFhir(VALID_PATIENT_INPUT, 'Patient');
      const keys = Object.keys(out);
      const snakeCaseKeys = keys.filter((k) => k.includes('_'));
      expect(snakeCaseKeys).toHaveLength(0);
    });

    test('patient telecom entries each have system and value fields', () => {
      const out = t.transformToFhir(VALID_PATIENT_INPUT, 'Patient');
      for (const contact of out.telecom ?? []) {
        expect(contact.system).toBeDefined();
        expect(contact.value).toBeDefined();
      }
    });
  });
});