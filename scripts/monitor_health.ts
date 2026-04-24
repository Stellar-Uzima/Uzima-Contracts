import { Contract, TransactionBuilder, Networks, rpc, Account, scValToNative } from '@stellar/stellar-sdk';

// CONFIGURATION
// Replace with your actual Contract ID from the deployment step
const CONTRACT_ID = "CD7O5HYNHXGWLKWAUS7NVTCDQJFOTHFFHJRTHNVS6VH2UIO4NKLUW3H7"; 
const RPC_URL = "https://soroban-testnet.stellar.org";
const NETWORK_PASSPHRASE = Networks.TESTNET;

async function checkHealth() {
  const server = new rpc.Server(RPC_URL);
  const contract = new Contract(CONTRACT_ID);

  console.log(`🏥 Connecting to contract: ${CONTRACT_ID}...`);

  try {
    const operation = contract.call("health_check");

    const dummyAccount = new Account(
      "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF", 
      "0"
    );
    
    const tx = new TransactionBuilder(dummyAccount, { fee: "100", networkPassphrase: NETWORK_PASSPHRASE })
      .addOperation(operation)
      .setTimeout(30)
      .build();

    const response = await server.simulateTransaction(tx);

    // Check if simulation was successful
    if (rpc.Api.isSimulationSuccess(response)) {
      // 1. Safe parsing of the result using scValToNative
      // This converts the obscure XDR format into a standard JS Array: ["OK", 1, 123456n]
      const rawRetval = response.result?.retval;
      
      if (!rawRetval) {
        console.error("❌ No return value found in response.");
        return;
      }

      const result = scValToNative(rawRetval);

      console.log("✅ Health Check Successful!");
      console.log("--------------------------------");
      
      if (Array.isArray(result)) {
        console.log(`Status:    ${result[0]}`); // "OK" or "PAUSED"
        console.log(`Version:   ${result[1]}`); // 1
        console.log(`Timestamp: ${result[2]}`); // Ledger timestamp
      } else {
        console.log("Raw Result:", result);
      }

      // 2. Metrics (Gas Cost)
      // We use @ts-ignore here because sometimes the type definitions lag behind the actual API response
      // @ts-ignore
      if (response.cost) {
        // @ts-ignore
        console.log(`CPU Cost:  ${response.cost.cpuInsns}`);
        // @ts-ignore
        console.log(`Mem Cost:  ${response.cost.memBytes}`);
      }
      
    } else {
      console.error("❌ Simulation Failed:", response);
    }

  } catch (error) {
    console.error("❌ Connection Error:", error);
  }
}

checkHealth();