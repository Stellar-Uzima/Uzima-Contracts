class AwsConnector {
  constructor(cfg = {}){
    this.cfg = cfg;
    this.store = new Map();
  }

  async init(){
    // In production: initialize AWS HealthLake client
    this.ready = true;
  }

  async pushResource(resourceType, resource){
    const id = resource.id || `${resourceType}-${Date.now()}`;
    this.store.set(id, { resourceType, resource });
    return { id, status: 'ok', provider: 'aws' };
  }

  async fetchResource(id){
    return this.store.get(id) || null;
  }

  async listResources(){
    return Array.from(this.store.values()).map(v => v.resource);
  }
}

module.exports = { AwsConnector };
