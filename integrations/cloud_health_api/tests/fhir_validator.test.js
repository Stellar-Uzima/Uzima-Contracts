const { validate } = require('../src/transform/fhir_validator');

describe('FHIR validator', ()=>{
  test('valid patient passes validation', ()=>{
    const p = { resourceType: 'Patient', id: 'p1', name: [{ text: 'Zoe' }] };
    const res = validate(p);
    expect(res.valid).toBe(true);
  });

  test('invalid patient without id fails', ()=>{
    const p = { resourceType: 'Patient' };
    const res = validate(p);
    expect(res.valid).toBe(false);
  });
});
