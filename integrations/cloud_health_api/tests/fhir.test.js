const { FhirTransformer } = require('../src/transform/fhir');

describe('FHIR Transformer', ()=>{
  const t = new FhirTransformer();

  test('transforms a patient-like object to FHIR Patient', ()=>{
    const input = { id: 'p1', name: 'Alice', dob: '1970-01-01' };
    const out = t.transformToFhir(input);
    expect(out.resourceType).toBe('Patient');
    expect(out.id).toBeDefined();
    expect(out.name[0].text).toBe('Alice');
  });

  test('validates FHIR Patient', ()=>{
    const input = { id: 'p1', resourceType: 'Patient' };
    const v = t.validateFhirResource(input);
    expect(v.valid).toBe(true);
  });
});
