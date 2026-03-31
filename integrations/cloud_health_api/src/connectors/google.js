const { google } = require('googleapis');

class GoogleConnector {
  constructor(cfg = {}){
    this.cfg = cfg;
    this.store = new Map(); // in-memory mock store
    // cfg.useReal === true will enable real Google Healthcare API calls
    this.useReal = !!cfg.useReal || !!process.env.UZIMA_USE_GOOGLE_REAL;
    // expected cfg fields when using real: projectId, location, datasetId, fhirStoreId
  }

  async init(){
    if(this.useReal){
      if(!this.cfg.projectId || !this.cfg.location || !this.cfg.datasetId || !this.cfg.fhirStoreId){
        throw new Error('GoogleConnector real mode requires projectId, location, datasetId and fhirStoreId in config');
      }
      // Acquire authenticated client using Application Default Credentials
      this.authClient = await google.auth.getClient({ scopes: ['https://www.googleapis.com/auth/cloud-platform'] });
      this.basePath = `projects/${this.cfg.projectId}/locations/${this.cfg.location}/datasets/${this.cfg.datasetId}/fhirStores/${this.cfg.fhirStoreId}`;
      this.baseUrl = `https://healthcare.googleapis.com/v1/${this.basePath}`;
      this.ready = true;
    }else{
      // mock mode
      this.ready = true;
    }
  }

  async pushResource(resourceType, resource){
    if(this.useReal){
      // POST to FHIR endpoint: POST {baseUrl}/fhir/{resourceType}
      const url = `${this.baseUrl}/fhir/${resourceType}`;
      const res = await this.authClient.request({ url, method: 'POST', data: resource });
      // The FHIR server returns the created resource (or OperationOutcome). Return the id and raw response status.
      const returned = res.data || {};
      const id = returned.id || (returned.resource && returned.resource.id) || `${resourceType}-${Date.now()}`;
      return { id, status: res.status === 200 || res.status === 201 ? 'ok' : 'error', provider: 'google', raw: returned };
    }

    // Mock behaviour
    const id = resource.id || `${resourceType}-${Date.now()}`;
    this.store.set(id, { resourceType, resource });
    return { id, status: 'ok', provider: 'google' };
  }

  async fetchResource(idOrType, maybeId){
    // If called with (resourceType, id) in real mode, construct GET accordingly. If called with single id in mock, fetch from store.
    if(this.useReal){
      const resourceType = idOrType;
      const id = maybeId;
      if(!resourceType || !id) throw new Error('fetchResource(resourceType, id) required in real mode');
      const url = `${this.baseUrl}/fhir/${resourceType}/${id}`;
      const res = await this.authClient.request({ url, method: 'GET' });
      return res.data;
    }

    // mock: idOrType is id
    return this.store.get(idOrType) || null;
  }

  async listResources(){
    if(this.useReal){
      // Real listing would require FHIR search; for simplicity, perform a search for all patients: GET {baseUrl}/fhir/Patient
      const url = `${this.baseUrl}/fhir/Patient`;
      const res = await this.authClient.request({ url, method: 'GET' });
      return res.data;
    }
    return Array.from(this.store.values()).map(v => v.resource);
  }
}

module.exports = { GoogleConnector };
