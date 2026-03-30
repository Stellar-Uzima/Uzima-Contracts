const { validate } = require('./fhir_validator');

class FhirTransformer {
  constructor(options = {}){
    this.options = options;
  }

  // Convert a simple patient-like object into a minimal FHIR Patient resource
  transformToFhir(resource){
    if(!resource) throw new Error('No resource provided');
    const fhir = {
      resourceType: 'Patient',
      id: resource.id || `p-${Date.now()}`,
      name: [{ text: resource.name || resource.fullName }],
      birthDate: resource.dob || resource.birthDate || undefined,
      meta: { source: 'uzima-mock-transform' }
    };
    return fhir;
  }

  validateFhirResource(res){
    const result = validate(res);
    if(!result.valid){
      return { valid: false, reason: Array.isArray(result.errors) ? JSON.stringify(result.errors) : result.errors };
    }
    return { valid: true };
  }
}

module.exports = { FhirTransformer };
