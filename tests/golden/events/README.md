# Golden File Tests — Contract Events

This directory contains **golden files** that lock in the exact JSON
serialization of each contract's events.  Tests in `tests/` compare the
actual event output against these files and **fail** if the format changes
unexpectedly.

## How to update golden files

When a contract change intentionally modifies an event format:

1. Run the golden tests to see which files fail.
2. Review the diff to confirm the change is intentional.
3. Update the golden file(s) by copying the new expected output.
4. Add a changelog entry explaining the breaking event change.
5. Commit the updated golden files alongside the code change.

## Covered contracts

| Contract              | Events                                      | Golden file               |
|-----------------------|---------------------------------------------|---------------------------|
| `healthcare_payment`  | CLAIM_SUB, CLAIM_PD, FRAUD, EOB            | `healthcare_payment.json` |
| `medical_records`     | REC_NEW, REC_ACC, META_UPD, EM_GRANT       | `medical_records.json`    |
| `identity_registry`   | DIDCreated, KeyRotated, CredentialIssued    | `identity_registry.json`  |
| `cross_chain_bridge`  | MessageSubmitted, MessageVerified           | `cross_chain_bridge.json` |
| `timelock`            | Queued, Executed, Cancelled                 | `timelock.json`           |
| `escrow`              | EscNew, EscRel, EscDisput, Refunded         | `escrow.json`             |
