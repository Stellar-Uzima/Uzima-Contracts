# Identity & DID

See [docs/DID_INTEGRATION.md](../../DID_INTEGRATION.md) and [docs/IDENTITY_VERIFICATION_FLOW.md](../../IDENTITY_VERIFICATION_FLOW.md).

## DID Method

`did:stellar:<stellar_address>`

## Credential Types

- `HealthcareProvider` — Licensed medical provider
- `Patient` — Registered patient
- `InsuranceVerification` — Active insurance coverage
- `MedicalLicense` — Professional license

## Verification Flow

1. Provider registers DID via `identity_registry.register_did()`
2. Issuer issues credential via `issue_credential()`
3. Verifier calls `verify_credential()` before granting access
