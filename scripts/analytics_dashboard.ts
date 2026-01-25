import { Contract, TransactionBuilder, Networks, rpc, Account, scValToNative, xdr } from '@stellar/stellar-sdk';

// Basic configuration - override via environment variables when needed
const RPC_URL = process.env.RPC_URL || 'https://soroban-testnet.stellar.org';
const NETWORK_PASSPHRASE = process.env.NETWORK_PASSPHRASE || Networks.TESTNET;
const server = new rpc.Server(RPC_URL);

// Contract IDs – set via env vars or by editing these defaults
const CONTRACT_IDS = {
  medicalRecords: process.env.MEDICAL_RECORDS_ID || 'REPLACE_WITH_MEDICAL_RECORDS_ID',
  anomalyDetection: process.env.ANOMALY_DETECTION_ID || '',
  predictiveAnalytics: process.env.PREDICTIVE_ANALYTICS_ID || '',
  federatedLearning: process.env.FEDERATED_LEARNING_ID || '',
  explainableAi: process.env.EXPLAINABLE_AI_ID || '',
};

// Optional model / round identifiers for deeper analytics
// ANALYTICS_MODEL_ID should be 32-byte (64 char) hex string matching on-chain BytesN<32>
const ANALYTICS_MODEL_ID_HEX = process.env.ANALYTICS_MODEL_ID || '';
// FEDERATED_ROUND_ID should be a small integer (fits in JS number range)
const FEDERATED_ROUND_ID = process.env.FEDERATED_ROUND_ID
  ? Number(process.env.FEDERATED_ROUND_ID)
  : undefined;

const DUMMY_ACCOUNT = new Account(
  'GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF',
  '0',
);

type Section = 'health' | 'anomaly' | 'predictive' | 'federated' | 'explainable';

function getFormat(): 'json' | 'table' {
  if (process.argv.includes('--format=json')) return 'json';
  if (process.argv.includes('--json')) return 'json';
  return 'table';
}

function getRequestedSections(): Set<Section> {
  const defaultSections: Section[] = [
    'health',
    'anomaly',
    'predictive',
    'federated',
    'explainable',
  ];

  const argPrefix = '--sections=';
  const arg = process.argv.find((a) => a.startsWith(argPrefix));
  if (!arg) {
    return new Set<Section>(defaultSections);
  }

  const raw = arg.slice(argPrefix.length);
  const parts = raw
    .split(',')
    .map((s: string) => s.trim().toLowerCase())
    .filter((s: string) => s.length > 0);

  const sections = new Set<Section>();
  for (const p of parts) {
    if (p === 'health' || p === 'medical' || p === 'records') {
      sections.add('health');
    } else if (p === 'anomaly' || p === 'anomalies') {
      sections.add('anomaly');
    } else if (p === 'predictive' || p === 'prediction' || p === 'model') {
      sections.add('predictive');
    } else if (p === 'federated' || p === 'fl' || p === 'round') {
      sections.add('federated');
    } else if (p === 'explainable' || p === 'xai' || p === 'bias') {
      sections.add('explainable');
    }
  }

  return sections.size ? sections : new Set<Section>(defaultSections);
}

function hexToBytes(hex: string): Uint8Array {
  const clean = hex.trim().toLowerCase();
  if (clean.length !== 64) {
    throw new Error('ANALYTICS_MODEL_ID must be 64 hex characters (32 bytes)');
  }
  const bytes = new Uint8Array(32);
  for (let i = 0; i < 32; i++) {
    const byte = clean.slice(i * 2, i * 2 + 2);
    bytes[i] = parseInt(byte, 16);
  }
  return bytes;
}

function modelIdToScVal(hex: string): xdr.ScVal {
  const bytes = hexToBytes(hex);
  return xdr.ScVal.scvBytes(Buffer.from(bytes));
}

async function simulateCall(
  contractId: string,
  method: string,
  args: xdr.ScVal[] = [],
): Promise<any | null> {
  const contract = new Contract(contractId);

  try {
    const operation = contract.call(method, ...args);

    const tx = new TransactionBuilder(DUMMY_ACCOUNT, {
      fee: '100',
      networkPassphrase: NETWORK_PASSPHRASE,
    })
      .addOperation(operation)
      .setTimeout(30)
      .build();

    const response = await server.simulateTransaction(tx);

    if (!rpc.Api.isSimulationSuccess(response) || !response.result?.retval) {
      return null;
    }

    return scValToNative(response.result.retval);
  } catch (error) {
    console.error(`Error simulating ${method} on ${contractId}:`, error);
    return null;
  }
}

function normalizeBigInt(value: any): any {
  if (typeof value === 'bigint') {
    return value.toString();
  }
  if (Array.isArray(value)) {
    return value.map(normalizeBigInt);
  }
  if (value && typeof value === 'object') {
    const out: any = {};
    for (const [k, v] of Object.entries(value)) {
      out[k] = normalizeBigInt(v);
    }
    return out;
  }
  return value;
}

async function collectAnalytics(sections: Set<Section>) {
  const result: any = {
    network: {
      rpcUrl: RPC_URL,
      networkPassphrase: NETWORK_PASSPHRASE,
    },
    contracts: { ...CONTRACT_IDS },
    health: null as any,
    anomalyStats: null as any,
    predictiveMetrics: null as any,
    federatedRound: null as any,
    biasAudit: null as any,
  };

  if (
    sections.has('health') &&
    CONTRACT_IDS.medicalRecords &&
    !CONTRACT_IDS.medicalRecords.startsWith('REPLACE_WITH')
  ) {
    const health = await simulateCall(CONTRACT_IDS.medicalRecords, 'health_check');
    result.health = normalizeBigInt(health);
  } else if (sections.has('health')) {
    console.warn('Medical Records contract ID is not configured; set MEDICAL_RECORDS_ID env var.');
  }

  if (sections.has('anomaly') && CONTRACT_IDS.anomalyDetection) {
    const stats = await simulateCall(CONTRACT_IDS.anomalyDetection, 'get_stats');
    result.anomalyStats = normalizeBigInt(stats);
  }

  if (
    sections.has('predictive') &&
    CONTRACT_IDS.predictiveAnalytics &&
    ANALYTICS_MODEL_ID_HEX
  ) {
    try {
      const modelIdVal = modelIdToScVal(ANALYTICS_MODEL_ID_HEX);
      const metrics = await simulateCall(
        CONTRACT_IDS.predictiveAnalytics,
        'get_model_metrics',
        [modelIdVal],
      );
      result.predictiveMetrics = normalizeBigInt(metrics);
    } catch (e) {
      console.error('Failed to query predictive analytics metrics:', e);
    }
  }

  if (
    sections.has('federated') &&
    CONTRACT_IDS.federatedLearning &&
    typeof FEDERATED_ROUND_ID === 'number'
  ) {
    const roundIdVal = xdr.ScVal.scvU64(BigInt(FEDERATED_ROUND_ID));
    const round = await simulateCall(
      CONTRACT_IDS.federatedLearning,
      'get_round',
      [roundIdVal],
    );
    result.federatedRound = normalizeBigInt(round);
  }

  if (
    sections.has('explainable') &&
    CONTRACT_IDS.explainableAi &&
    ANALYTICS_MODEL_ID_HEX
  ) {
    try {
      const modelIdVal = modelIdToScVal(ANALYTICS_MODEL_ID_HEX);
      const audit = await simulateCall(
        CONTRACT_IDS.explainableAi,
        'get_bias_audit',
        [modelIdVal],
      );
      result.biasAudit = normalizeBigInt(audit);
    } catch (e) {
      console.error('Failed to query explainable AI bias audit:', e);
    }
  }

  return result;
}

function printTable(analytics: any) {
  console.log('UZIMA ANALYTICS DASHBOARD');
  console.log('=========================');

  if (analytics.health) {
    const [status, version, timestamp] = analytics.health as any;
    console.log('\n[Health]');
    console.log(`  Status   : ${status}`);
    console.log(`  Version  : ${version}`);
    console.log(`  Time     : ${timestamp}`);
  } else {
    console.log('\n[Health]');
    console.log('  Health check not available');
  }

  if (analytics.anomalyStats) {
    console.log('\n[Anomaly Detection]');
    console.log(`  Total anomalies     : ${analytics.anomalyStats.total_anomalies}`);
    console.log(`  High severity count : ${analytics.anomalyStats.high_severity_count}`);
    console.log(`  Last detection at   : ${analytics.anomalyStats.last_detection_at}`);
  }

  if (analytics.predictiveMetrics) {
    console.log('\n[Predictive Analytics Model Metrics]');
    console.log(`  Accuracy (bps)  : ${analytics.predictiveMetrics.accuracy_bps}`);
    console.log(`  Precision (bps) : ${analytics.predictiveMetrics.precision_bps}`);
    console.log(`  Recall (bps)    : ${analytics.predictiveMetrics.recall_bps}`);
    console.log(`  F1 score (bps)  : ${analytics.predictiveMetrics.f1_score_bps}`);
    console.log(`  Last updated    : ${analytics.predictiveMetrics.last_updated}`);
  }

  if (analytics.federatedRound) {
    console.log('\n[Federated Learning Round]');
    console.log(`  Round ID         : ${analytics.federatedRound.id}`);
    console.log(`  Min participants : ${analytics.federatedRound.min_participants}`);
    console.log(`  DP epsilon       : ${analytics.federatedRound.dp_epsilon}`);
    console.log(`  Total updates    : ${analytics.federatedRound.total_updates}`);
    console.log(`  Finalized        : ${analytics.federatedRound.is_finalized}`);
  }

  if (analytics.biasAudit) {
    console.log('\n[Explainable AI Bias Audit]');
    console.log(`  Audit date : ${analytics.biasAudit.audit_date}`);
    console.log(`  Summary    : ${analytics.biasAudit.audit_summary}`);
  }
}

async function main() {
  const format = getFormat();
  const sections = getRequestedSections();
  const analytics = await collectAnalytics(sections);

  if (format === 'json') {
    console.log(JSON.stringify(normalizeBigInt(analytics), null, 2));
  } else {
    printTable(analytics);
  }
}

main().catch((err) => {
  console.error('Analytics dashboard failed:', err);
  process.exit(1);
});
