const { Orchestrator } = require('./orchestrator');
const { GoogleConnector } = require('./connectors/google');
const { AzureConnector } = require('./connectors/azure');
const { AwsConnector } = require('./connectors/aws');
const { FhirTransformer } = require('./transform/fhir');

async function main(){
  // basic in-memory config - replace with environment/config integration in production
  const config = {
    google: { projectId: process.env.GCP_PROJECT || 'mock-project' },
    azure: { subscriptionId: process.env.AZURE_SUBSCRIPTION || 'mock-sub' },
    aws: { region: process.env.AWS_REGION || 'us-east-1' }
  };

  const google = new GoogleConnector(config.google);
  const azure = new AzureConnector(config.azure);
  const aws = new AwsConnector(config.aws);

  const transformer = new FhirTransformer();
  const orch = new Orchestrator({ google, azure, aws, transformer });

  console.log('Starting Uzima Cloud Health API orchestrator (mock)');

  // start a mock realtime simulation (in production this would be webhooks / pubsub listeners)
  orch.simulateRealtimeEvents();

  // demonstrate a cross-sync between Google -> Azure using mock data
  const sample = { id: 'patient-1', name: 'Jane Doe', dob: '1990-01-01' };
  await orch.crossCloudSync('google', 'azure', sample);
  console.log('Initial cross-cloud sync completed (mock)');
}

if (require.main === module){
  main().catch(err => {
    console.error('Fatal error in orchestrator', err);
    process.exit(1);
  });
}

module.exports = { main };
