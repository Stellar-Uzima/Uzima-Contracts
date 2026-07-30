# Python Integration Examples

Examples using the Stellar Python SDK (`stellar-sdk`) to interact with Uzima
Soroban contracts.

## Setup

```bash
pip install stellar-sdk
```

```python
from stellar_sdk import (
    Server, Keypair, Contract, Network,
    TransactionBuilder, InvokeHostFunctionOp,
    soroban_sdk,
)
```

## Contract Client Initialization

```python
server = Server("https://soroban-testnet.stellar.org")
contract = Contract("CD...CONTRACT_ID")
network_passphrase = Network.TESTNET_NETWORK_PASSPHRASE
```

## Medical Records — Create and Read

```python
from stellar_sdk import scval

doctor = Keypair.random()
patient = Keypair.random()

# Build add_record transaction
tx = (
    TransactionBuilder(
        source_account=server.load_account(doctor.public_key),
        network_passphrase=network_passphrase,
        base_fee=100,
    )
    .append_invoke_contract_function_op(
        contract_id="CD...CONTRACT_ID",
        function_name="add_record",
        parameters=[
            scval.to_address(doctor.public_key),
            scval.to_address(patient.public_key),
            scval.to_string("Hypertension diagnosed"),
            scval.to_string("Lisinopril 10mg daily"),
            scval.to_bool(False),
            scval.to_vec([scval.to_string("cardiology")]),
            scval.to_string("diagnosis"),
            scval.to_string("primary_care"),
            scval.to_string("ipfs://Qm..."),
        ],
    )
    .build()
)

response = server.submit_transaction(tx, network_passphrase)
record_id = scval.from_uint64(response.result_meta_xdr)
```

## Medical Records — Error Handling

```python
from stellar_sdk.exceptions import BaseRequestError

try:
    tx = (
        TransactionBuilder(
            source_account=server.load_account(patient.public_key),
            network_passphrase=network_passphrase,
            base_fee=100,
        )
        .append_invoke_contract_function_op(
            contract_id="CD...CONTRACT_ID",
            function_name="get_record",
            parameters=[
                scval.to_address(patient.public_key),
                scval.to_uint64(record_id),
            ],
        )
        .build()
    )

    response = server.simulate_transaction(tx, network_passphrase)
    if response.error:
        print(f"Contract error: {response.error}")
    else:
        record = scval.from hexatrigesimal(response.result_meta_xdr)
        print(f"Record: {record}")
except BaseRequestError as e:
    print(f"Request failed: {e}")
```

## Healthcare Payment — Submit a Claim

```python
payment_contract = Contract("CD...PAYMENT_ID")

tx = (
    TransactionBuilder(
        source_account=server.load_account(clinic.public_key),
        network_passphrase=network_passphrase,
        base_fee=200,
    )
    .append_invoke_contract_function_op(
        contract_id="CD...PAYMENT_ID",
        function_name="submit_claim",
        parameters=[
            scval.to_address(patient.public_key),
            scval.to_address(clinic.public_key),
            scval.to_string("X-22"),
            scval.to_i128(500),
            scval.to_string("BC001"),
            scval.to_option(None),
        ],
    )
    .build()
)

response = server.submit_transaction(tx, network_passphrase)
claim_id = scval.from_uint64(response.result_meta_xdr)
```

## Patient Consent — Grant and Check

```python
consent_contract = Contract("CD...CONSENT_ID")

# Grant consent
grant_tx = (
    TransactionBuilder(
        source_account=server.load_account(patient.public_key),
        network_passphrase=network_passphrase,
        base_fee=100,
    )
    .append_invoke_contract_function_op(
        contract_id="CD...CONSENT_ID",
        function_name="grant_consent",
        parameters=[
            scval.to_address(patient.public_key),
            scval.to_address(provider.public_key),
        ],
    )
    .build()
)

server.submit_transaction(grant_tx, network_passphrase)

# Check consent
check_tx = (
    TransactionBuilder(
        source_account=server.load_account(provider.public_key),
        network_passphrase=network_passphrase,
        base_fee=100,
    )
    .append_invoke_contract_function_op(
        contract_id="CD...CONSENT_ID",
        function_name="check_consent",
        parameters=[
            scval.to_address(patient.public_key),
            scval.to_address(provider.public_key),
        ],
    )
    .build()
)

response = server.simulate_transaction(check_tx, network_passphrase)
has_consent = scval.from_bool(response.result_meta_xdr)
print(f"Has consent: {has_consent}")
```

## Pagination Pattern

```python
offset = 0
limit = 20
has_more = True

while has_more:
    tx = (
        TransactionBuilder(
            source_account=server.load_account(patient.public_key),
            network_passphrase=network_passphrase,
            base_fee=100,
        )
        .append_invoke_contract_function_op(
            contract_id="CD...CONTRACT_ID",
            function_name="get_records_page",
            parameters=[
                scval.to_address(patient.public_key),
                scval.to_uint64(offset),
                scval.to_uint32(limit),
            ],
        )
        .build()
    )

    response = server.simulate_transaction(tx, network_passphrase)
    page = scval.from_dataframe(response.result_meta_xdr)

    offset += limit
    has_more = page["has_more"]
```

## Best Practices

1. **Use `simulate_transaction`** before submitting to estimate costs
   and catch errors without spending fees.
2. **Handle XDR decoding errors** — malformed responses indicate a
   contract schema mismatch.
3. **Retry with backoff** — RPC endpoints may transiently fail; use
   exponential backoff for reliability.
