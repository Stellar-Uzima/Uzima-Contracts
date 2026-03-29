const Ajv = require('ajv');

// Minimal FHIR Patient schema (R4 subset) for lightweight validation in this scaffold.
const patientSchema = {
  $id: 'https://example.org/fhir/patient.schema.json',
  type: 'object',
  required: ['resourceType', 'id'],
  properties: {
    resourceType: { const: 'Patient' },
    id: { type: 'string' },
    name: { type: 'array' },
    birthDate: { type: 'string' }
  },
  additionalProperties: true
};

const ajv = new Ajv({ allErrors: true, strict: false });
const validatePatient = ajv.compile(patientSchema);

function validate(resource){
  if(!resource) return { valid: false, errors: ['no resource'] };
  if(resource.resourceType === 'Patient'){
    const ok = validatePatient(resource);
    if(!ok) return { valid: false, errors: validatePatient.errors };
    return { valid: true };
  }
  // For non-Patient resources, do a minimal check
  if(!resource.resourceType) return { valid: false, errors: ['missing resourceType'] };
  return { valid: true };
}

module.exports = { validate };
