const EventEmitter = require('events');

class Orchestrator extends EventEmitter {
  constructor({ google, azure, aws, transformer }){
    super();
    this.google = google;
    this.azure = azure;
    this.aws = aws;
    this.transformer = transformer;
  }

  async init(){
    await Promise.all([
      this.google?.init?.(),
      this.azure?.init?.(),
      this.aws?.init?.()
    ]);
  }

  // Cross-cloud sync: basic flow - transform into FHIR, then push to target
  async crossCloudSync(sourceName, targetName, resource){
    const source = this[sourceName];
    const target = this[targetName];
    if(!source || !target) throw new Error('Invalid connector name');

    // Step 1: convert to FHIR (standardization)
    const fhir = this.transformer.transformToFhir(resource);
    const validation = this.transformer.validateFhirResource(fhir);
    if(!validation.valid) throw new Error('FHIR validation failed: ' + (validation.reason||'unknown'));

    // Step 2: push to target
    const result = await target.pushResource(fhir.resourceType, fhir);
    this.emit('sync', { from: sourceName, to: targetName, id: result.id, resourceType: fhir.resourceType });
    return result;
  }

  // Simple simulation of realtime events - emits a mock patient create event every 5 seconds
  simulateRealtimeEvents(intervalMs = 5000){
    let i = 0;
    setInterval(async ()=>{
      i++;
      const sample = { id: `sim-${i}`, name: `Sim Patient ${i}`, dob: '1980-01-01' };
      this.emit('realtime:event', { event: 'patient.created', payload: sample });
      // By default cross-sync the simulated patient across all providers (google -> azure -> aws)
      try{
        await this.crossCloudSync('google','azure', sample);
        await this.crossCloudSync('azure','aws', sample);
      }catch(err){
        this.emit('error', err);
      }
    }, intervalMs);
  }
}

module.exports = { Orchestrator };
