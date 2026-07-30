# TypeScript Integration Examples

Examples using the Stellar TypeScript SDK (`@stellar/stellar-sdk`) to interact
with Uzima Soroban contracts.

## Setup

```bash
npm install @stellar/stellar-sdk
```

```typescript
import * as StellarSdk from "@stellar/stellar-sdk";
```

## Contract Client Initialization

```typescript
const server = new StellarSdk.SorobanRpc.Server(
  process.env.STELLAR_RPC_URL || "https://soroban-testnet.stellar.org"
);

const contractId = "CD...CONTRACT_ID";
const contract = new StellarSdk.Contract(contractId);
```

## Medical Records — Create and Read

```typescript
const doctor = StellarSdk.Keypair.random();
const patient = StellarSdk.Keypair.random();
const networkPassphrase =
  process.env.STELLAR_NETWORK_PASSPHRASE ||
  "Test SDF Network ; September 1995";

// Build the add_record transaction
const addRecordTx = new StellarSdk.TransactionBuilder(
  { accountId: doctor.publicKey() },
  { fee: "100", networkPassphrase }
)
  .addOperation(
    contract.call(
      "add_record",
      StellarSdk.Address.fromString(doctor.publicKey()),
      StellarSdk.Address.fromString(patient.publicKey()),
      StellarSdk.nativeToScVal("Hypertension diagnosed", { type: "string" }),
      StellarSdk.nativeToScVal("Lisinopril 10mg daily", { type: "string" }),
      StellarSdk.nativeToScVal(false, { type: "bool" }),
      StellarSdk.nativeToScVal(["cardiology"], {
        type: "vec",
        inner: { type: "string" },
      }),
      StellarSdk.nativeToScVal("diagnosis", { type: "string" }),
      StellarSdk.nativeToScVal("primary_care", { type: "string" }),
      StellarSdk.nativeToScVal("ipfs://Qm...", { type: "string" })
    )
  )
  .setTimeout(StellarSdk.TimeoutInfinite)
  .build();

const preparedTx = await server.prepareTransaction(addRecordTx);
preparedTx.sign(doctor);
const result = await server.sendTransaction(preparedTx);
const recordId = StellarSdk.scValToNative(result.result.meta);
```

## Medical Records — Error Handling

```typescript
try {
  const getRecordTx = new StellarSdk.TransactionBuilder(
    { accountId: patient.publicKey() },
    { fee: "100", networkPassphrase }
  )
    .addOperation(
      contract.call(
        "get_record",
        StellarSdk.Address.fromString(patient.publicKey()),
        StellarSdk.nativeToScVal(recordId, { type: "u64" })
      )
    )
    .setTimeout(StellarSdk.TimeoutInfinite)
    .build();

  const prepared = await server.prepareTransaction(getRecordTx);
  prepared.sign(patient);
  const result = await server.sendTransaction(prepared);
  console.log("Record:", StellarSdk.scValToNative(result.result.meta));
} catch (err) {
  if (err instanceof StellarSdk.rpc.HttpError) {
    console.error("RPC error:", err.data);
  } else {
    console.error("Transaction error:", err);
  }
}
```

## Healthcare Payment — Submit a Claim

```typescript
const paymentContract = new StellarSdk.Contract("CD...PAYMENT_ID");

const claimTx = new StellarSdk.TransactionBuilder(
  { accountId: clinic.publicKey() },
  { fee: "200", networkPassphrase }
)
  .addOperation(
    paymentContract.call(
      "submit_claim",
      StellarSdk.Address.fromString(patient.publicKey()),
      StellarSdk.Address.fromString(clinic.publicKey()),
      StellarSdk.nativeToScVal("X-22", { type: "string" }),
      StellarSdk.nativeToScVal(500, { type: "i128" }),
      StellarSdk.nativeToScVal("BC001", { type: "string" }),
      StellarSdk.nativeToScVal(null, { type: "option" })
    )
  )
  .setTimeout(StellarSdk.TimeoutInfinite)
  .build();

const preparedClaim = await server.prepareTransaction(claimTx);
preparedClaim.sign(clinic);
const claimResult = await server.sendTransaction(preparedClaim);
```

## Patient Consent — Grant and Check

```typescript
const consentContract = new StellarSdk.Contract("CD...CONSENT_ID");

// Grant consent
const grantTx = new StellarSdk.TransactionBuilder(
  { accountId: patient.publicKey() },
  { fee: "100", networkPassphrase }
)
  .addOperation(
    consentContract.call(
      "grant_consent",
      StellarSdk.Address.fromString(patient.publicKey()),
      StellarSdk.Address.fromString(provider.publicKey())
    )
  )
  .setTimeout(StellarSdk.TimeoutInfinite)
  .build();

const preparedGrant = await server.prepareTransaction(grantTx);
preparedGrant.sign(patient);
await server.sendTransaction(preparedGrant);

// Check consent
const checkTx = new StellarSdk.TransactionBuilder(
  { accountId: provider.publicKey() },
  { fee: "100", networkPassphrase }
)
  .addOperation(
    consentContract.call(
      "check_consent",
      StellarSdk.Address.fromString(patient.publicKey()),
      StellarSdk.Address.fromString(provider.publicKey())
    )
  )
  .setTimeout(StellarSdk.TimeoutInfinite)
  .build();

const preparedCheck = await server.prepareTransaction(checkTx);
preparedCheck.sign(provider);
const checkResult = await server.sendTransaction(preparedCheck);
const hasConsent = StellarSdk.scValToNative(checkResult.result.meta);
console.log("Has consent:", hasConsent);
```

## Pagination Pattern

```typescript
let offset = 0;
const limit = 20;
let hasMore = true;

while (hasMore) {
  const tx = new StellarSdk.TransactionBuilder(
    { accountId: patient.publicKey() },
    { fee: "100", networkPassphrase }
  )
    .addOperation(
      contract.call(
        "get_records_page",
        StellarSdk.Address.fromString(patient.publicKey()),
        StellarSdk.nativeToScVal(offset, { type: "u64" }),
        StellarSdk.nativeToScVal(limit, { type: "u32" })
      )
    )
    .setTimeout(StellarSdk.TimeoutInfinite)
    .build();

  const prepared = await server.prepareTransaction(tx);
  prepared.sign(patient);
  const result = await server.sendTransaction(prepared);
  const page = StellarSdk.scValToNative(result.result.meta);

  offset += limit;
  hasMore = page.has_more;
}
```

## Best Practices

1. **Always `prepareTransaction`** before signing to get the correct
   `sequence number` and `sorobanData`.
2. **Use `StellarSdk.TimeoutInfinite`** for Soroban transactions —
   timeouts are enforced at the network level.
3. **Handle `HttpError`** from RPC calls — network failures are expected
   and should be retried with backoff.
