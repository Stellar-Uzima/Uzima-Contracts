import { Contract, TransactionBuilder, Networks, rpc, Account, scValToNative, xdr } from '@stellar/stellar-sdk';

// Basic configuration - override via environment variables when needed
const RPC_URL = process.env.RPC_URL || 'https://soroban-testnet.stellar.org';
const NETWORK_PASSPHRASE = process.env.NETWORK_PASSPHRASE || Networks.TESTNET;

// Contract IDs – set via env vars or by editing these defaults
const CONTRACT_IDS = {
  fhirIntegration: process.env.FHIR_INTEGRATION_ID || 'REPLACE_WITH_FHIR_INTEGRATION_ID',
  emrIntegration: process.env.EMR_INTEGRATION_ID || 'REPLACE_WITH_EMR_INTEGRATION_ID',
  anomalyDetection: process.env.ANOMALY_DETECTION_ID || '',
  predictiveAnalytics: process.env.PREDICTIVE_ANALYTICS_ID || '',
};

// ANALYTICS_MODEL_ID should be 32-byte (64 char) hex string matching on-chain BytesN<32>
const ANALYTICS_MODEL_ID_HEX = process.env.ANALYTICS_MODEL_ID || '';

// Provider and network node identifiers (off-chain configuration)
// These are logical institution identifiers used by FHIR/EMR contracts
const PROVIDER_IDS = parseListEnv('PROVIDER_IDS');
const NETWORK_NODE_IDS = parseListEnv('NETWORK_NODE_IDS');

const DUMMY_ACCOUNT = new Account(
  'GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF',
  '0',
);

function getFormat(): 'json' | 'table' {
  if (process.argv.includes('--format=json')) return 'json';
  if (process.argv.includes('--json')) return 'json';
  return 'table';
}

function parseListEnv(name: string): string[] {
  const raw = process.env[name];
  if (!raw) return [];
  return raw
    .split(',')
    .map((s: string) => s.trim())
    .filter((s: string) => s.length > 0);
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
  const server = new rpc.Server(RPC_URL);
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

interface InstitutionAnalytics {
  id: string;
  name?: string;
  type?: string;
  region?: string;
  emr_system?: string;
  fhir_endpoint?: string;
  anomalyStats: any | null;
  predictiveMetrics: any | null;
}

async function collectCrossInstitutionAnalytics() {
  const result: any = {
    network: {
      rpcUrl: RPC_URL,
      networkPassphrase: NETWORK_PASSPHRASE,
    },
    contracts: { ...CONTRACT_IDS },
    configuration: {
      providerIds: PROVIDER_IDS,
      networkNodeIds: NETWORK_NODE_IDS,
    },
    providers: [] as InstitutionAnalytics[],
    networkNodes: [] as InstitutionAnalytics[],
    aggregates: {
      providersByType: {} as Record<string, number>,
      providersByRegion: {} as Record<string, number>,
      networkNodesByType: {} as Record<string, number>,
      networkNodesByRegion: {} as Record<string, number>,
    },
  };

  // Global stats that can be associated with each institution for now.
  // Future versions can replace with true per-institution breakdowns.
  let globalAnomalyStats: any | null = null;
  let globalPredictiveMetrics: any | null = null;

  if (CONTRACT_IDS.anomalyDetection) {
    const stats = await simulateCall(CONTRACT_IDS.anomalyDetection, 'get_stats');
    globalAnomalyStats = normalizeBigInt(stats);
  }

  if (CONTRACT_IDS.predictiveAnalytics && ANALYTICS_MODEL_ID_HEX) {
    try {
      const modelIdVal = modelIdToScVal(ANALYTICS_MODEL_ID_HEX);
      const metrics = await simulateCall(
        CONTRACT_IDS.predictiveAnalytics,
        'get_model_metrics',
        [modelIdVal],
      );
      globalPredictiveMetrics = normalizeBigInt(metrics);
    } catch (e) {
      console.error('Failed to query predictive analytics metrics:', e);
    }
  }

  // FHIR providers (institution-level view)
  if (!CONTRACT_IDS.fhirIntegration || CONTRACT_IDS.fhirIntegration.startsWith('REPLACE_WITH')) {
    if (PROVIDER_IDS.length > 0) {
      console.warn(
        'FHIR_INTEGRATION_ID is not configured; set FHIR_INTEGRATION_ID env var to resolve provider lookups.',
      );
    }
  } else {
    for (const providerId of PROVIDER_IDS) {
      try {
        const provider = await simulateCall(
          CONTRACT_IDS.fhirIntegration,
          'get_provider',
          [xdr.ScVal.scvString(providerId)],
        );

        if (!provider) {
          console.warn(`Provider ${providerId} not found or get_provider failed.`);
          continue;
        }

        const p: any = normalizeBigInt(provider);
        const entry: InstitutionAnalytics = {
          id: providerId,
          name: p.name,
          type: p.facility_type,
          region: p.address,
          emr_system: p.emr_system,
          fhir_endpoint: p.fhir_endpoint,
          anomalyStats: globalAnomalyStats,
          predictiveMetrics: globalPredictiveMetrics,
        };

        result.providers.push(entry);
        // Update provider aggregates (in-memory only)
        if (entry.type) {
          const key = entry.type;
          result.aggregates.providersByType[key] =
            (result.aggregates.providersByType[key] || 0) + 1;
        }
        if (entry.region) {
          const key = entry.region;
          result.aggregates.providersByRegion[key] =
            (result.aggregates.providersByRegion[key] || 0) + 1;
        }
      } catch (e) {
        console.error(`Error fetching provider ${providerId}:`, e);
      }
    }
  }

  // EMR network nodes (institution or site-level view)
  if (!CONTRACT_IDS.emrIntegration || CONTRACT_IDS.emrIntegration.startsWith('REPLACE_WITH')) {
    if (NETWORK_NODE_IDS.length > 0) {
      console.warn(
        'EMR_INTEGRATION_ID is not configured; set EMR_INTEGRATION_ID env var to resolve network node lookups.',
      );
    }
  } else {
    for (const nodeId of NETWORK_NODE_IDS) {
      try {
        const node = await simulateCall(
          CONTRACT_IDS.emrIntegration,
          'get_network_node',
          [xdr.ScVal.scvString(nodeId)],
        );

        if (!node) {
          console.warn(`Network node ${nodeId} not found or get_network_node failed.`);
          continue;
        }

        const n: any = normalizeBigInt(node);
        const entry: InstitutionAnalytics = {
          id: nodeId,
          name: n.network_name,
          type: n.node_type,
          region: n.geographic_region,
          emr_system: undefined,
          fhir_endpoint: undefined,
          anomalyStats: globalAnomalyStats,
          predictiveMetrics: globalPredictiveMetrics,
        };

        result.networkNodes.push(entry);
        // Update network node aggregates (in-memory only)
        if (entry.type) {
          const key = entry.type;
          result.aggregates.networkNodesByType[key] =
            (result.aggregates.networkNodesByType[key] || 0) + 1;
        }
        if (entry.region) {
          const key = entry.region;
          result.aggregates.networkNodesByRegion[key] =
            (result.aggregates.networkNodesByRegion[key] || 0) + 1;
        }
      } catch (e) {
        console.error(`Error fetching network node ${nodeId}:`, e);
      }
    }
  }

  return result;
}

function printTable(analytics: any) {
  console.log('UZIMA CROSS-INSTITUTION ANALYTICS');
  console.log('=================================');

  console.log('\n[Network]');
  console.log(`  RPC URL : ${analytics.network.rpcUrl}`);
  console.log(`  Network : ${analytics.network.networkPassphrase}`);

  if (analytics.providers && analytics.providers.length > 0) {
    console.log('\n[Providers (FHIR Integration)]');
    for (const p of analytics.providers as InstitutionAnalytics[]) {
      console.log(`- Provider: ${p.id}`);
      if (p.name) console.log(`    Name       : ${p.name}`);
      if (p.type) console.log(`    Type       : ${p.type}`);
      if (p.region) console.log(`    Region     : ${p.region}`);
      if (p.emr_system) console.log(`    EMR System : ${p.emr_system}`);
      if (p.fhir_endpoint) console.log(`    FHIR URL   : ${p.fhir_endpoint}`);
      if (p.anomalyStats) {
        console.log(
          `    Anomalies : total=${p.anomalyStats.total_anomalies}, high_severity=${p.anomalyStats.high_severity_count}`,
        );
      }
      if (p.predictiveMetrics) {
        console.log(
          `    Predictive: acc=${p.predictiveMetrics.accuracy_bps}bps, f1=${p.predictiveMetrics.f1_score_bps}bps`,
        );
      }
    }
  } else {
    console.log('\n[Providers]');
    console.log('  No providers configured (set PROVIDER_IDS env var).');
  }

  if (analytics.networkNodes && analytics.networkNodes.length > 0) {
    console.log('\n[Network Nodes (EMR Integration)]');
    for (const n of analytics.networkNodes as InstitutionAnalytics[]) {
      console.log(`- Node: ${n.id}`);
      if (n.name) console.log(`    Network  : ${n.name}`);
      if (n.type) console.log(`    Type     : ${n.type}`);
      if (n.region) console.log(`    Region   : ${n.region}`);
      if (n.anomalyStats) {
        console.log(
          `    Anomalies: total=${n.anomalyStats.total_anomalies}, high_severity=${n.anomalyStats.high_severity_count}`,
        );
      }
      if (n.predictiveMetrics) {
        console.log(
          `    Predictive: acc=${n.predictiveMetrics.accuracy_bps}bps, f1=${n.predictiveMetrics.f1_score_bps}bps`,
        );
      }
    }
  } else {
    console.log('\n[Network Nodes]');
    console.log('  No network nodes configured (set NETWORK_NODE_IDS env var).');
  }

  // Aggregated view (good for dashboards and quick comparisons)
  if (analytics.aggregates) {
    console.log('\n[Aggregates]');

    const agg = analytics.aggregates as any;

    const providerTypes = Object.entries(agg.providersByType || {});
    if (providerTypes.length > 0) {
      console.log('  Providers by type:');
      for (const [t, count] of providerTypes) {
        console.log(`    ${t}: ${count}`);
      }
    }

    const nodeTypes = Object.entries(agg.networkNodesByType || {});
    if (nodeTypes.length > 0) {
      console.log('  Network nodes by type:');
      for (const [t, count] of nodeTypes) {
        console.log(`    ${t}: ${count}`);
      }
    }
  }
}

async function main() {
  const format = getFormat();
  const analytics = await collectCrossInstitutionAnalytics();

  if (format === 'json') {
    console.log(JSON.stringify(normalizeBigInt(analytics), null, 2));
  } else {
    printTable(analytics);
  }
}

main().catch((err) => {
  console.error('Cross-institution analytics failed:', err);
  process.exit(1);
});
