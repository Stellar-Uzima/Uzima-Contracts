const { Orchestrator } = require('../src/orchestrator');
const { GoogleConnector } = require('../src/connectors/google');
const { AzureConnector } = require('../src/connectors/azure');
const { FhirTransformer } = require('../src/transform/fhir');

describe('Orchestrator cross-cloud sync', ()=>{
  test('syncs google -> azure with FHIR transformation', async ()=>{
    const google = new GoogleConnector();
    const azure = new AzureConnector();
    const transformer = new FhirTransformer();
    const orch = new Orchestrator({ google, azure, transformer });

    const sample = { id: 's1', name: 'Bob', dob: '1965-05-05' };
    const res = await orch.crossCloudSync('google','azure', sample);
    expect(res).toHaveProperty('id');
    expect(res).toHaveProperty('status','ok');
  });
  
  test('emits sync event on crossCloudSync', async ()=>{
    const google = new GoogleConnector();
    const azure = new AzureConnector();
    const transformer = new FhirTransformer();
    const orch = new Orchestrator({ google, azure, transformer });

    const events = [];
    orch.on('sync', e => events.push(e));
    await orch.crossCloudSync('google','azure', { id: 's2', name: 'Carol' });
    expect(events.length).toBeGreaterThan(0);
    expect(events[0]).toHaveProperty('from','google');
  });
});
