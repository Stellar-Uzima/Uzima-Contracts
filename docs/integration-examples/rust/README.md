# Rust Integration Examples

Examples using the Soroban Rust SDK (`soroban-sdk 21.7.7`) to interact with
Uzima contracts.

## Setup

Add to your `Cargo.toml`:

```toml
[dependencies]
soroban-sdk = "21.7.7"
```

## Contract Client Initialization

```rust
use soroban_sdk::{Address, Env, Symbol};

let env = Env::default();
let contract_id = Address::from_str(&env, "CD...CONTRACT_ID");
let client = medical_records::Client::new(&env, &contract_id);
```

## Medical Records — Create and Read

```rust
use soroban_sdk::{symbol_short, Address, Env, String, Vec};

let env = Env::default();
let doctor = Address::generate(&env);
let patient = Address::generate(&env);
let contract_id = Address::from_str(&env, "CD...CONTRACT_ID");
let client = medical_records::Client::new(&env, &contract_id);

let tags: Vec<String> = Vec::new(&env);
tags.push_back(String::from_str(&env, "cardiology"));

let record_id: u64 = client.add_record(
    &doctor,
    &patient,
    &String::from_str(&env, "Hypertension diagnosed"),
    &String::from_str(&env, "Lisinopril 10mg daily"),
    &false,            // is_encrypted
    &tags,
    &String::from_str(&env, "diagnosis"),
    &String::from_str(&env, "primary_care"),
    &String::from_str(&env, "ipfs://Qm..."),
);

let record = client.get_record(&patient, &record_id);
```

## Medical Records — Error Handling

```rust
use soroban_sdk::IntoVal;

match client.try_get_record(&patient, &record_id) {
    Ok(result) => match result {
        Ok(record) => println!("Record: {:?}", record),
        Err(e) => eprintln!("Contract error: {:?}", e),
    },
    Err(e) => eprintln!("Transaction failed: {:?}", e),
}
```

## Healthcare Payment — Submit a Claim

```rust
let payment_contract = Address::from_str(&env, "CD...PAYMENT_ID");
let pay_client = healthcare_payment::Client::new(&env, &payment_contract);

let claim_id: u64 = pay_client.submit_claim(
    &patient,
    &doctor,                  // provider
    &String::from_str(&env, "X-22"),   // claim code
    &500i128,                 // amount in stroops
    &String::from_str(&env, "BC001"),  // procedure code
    &None,                    // pre-auth id (optional)
);
```

## Healthcare Payment — Batch Processing

```rust
let claims: Vec<u64> = Vec::new(&env);
claims.push_back(1u64);
claims.push_back(2u64);
claims.push_back(3u64);

pay_client.batch_process_payments(&claims);
```

## Patient Consent — Grant and Revoke

```rust
let consent_contract = Address::from_str(&env, "CD...CONSENT_ID");
let consent_client = patient_consent_management::Client::new(&env, &consent_contract);

consent_client.grant_consent(&patient, &provider);

let has_consent: bool = consent_client.check_consent(&patient, &provider);
assert!(has_consent);

consent_client.revoke_consent(&patient, &provider);

let has_consent_after: bool = consent_client.check_consent(&patient, &provider);
assert!(!has_consent_after);
```

## Best Practices

1. **Always use `try_*` methods** for fallible calls to avoid panics in
   client code.
2. **Authorize explicitly** — `Address::require_auth()` is enforced on-chain,
   but client-side you must call `address.authorize()` before submitting.
3. **Keep WASM small** — avoid pulling in large dependencies; the contract
   WASM budget is 64 KiB.
