# Advanced Access Control with CP-ABE

## Architecture

This implementation uses a hybrid model:

- `contracts/medical_records` remains the on-chain source of truth for roles, delegated permissions, encrypted-record ownership, custom access attributes, and ABE policy metadata.
- `abe_sdk` performs CP-ABE key generation, policy compilation, DEK wrapping, and authorized-user decryption off-chain.
- Record payload encryption stays off-chain and unchanged. Revocation rotates only the DEK access package and on-chain policy epoch metadata, rather than re-encrypting the medical payload itself.

This fits Soroban well because pairing-heavy CP-ABE operations are not pushed into the contract runtime.

## Policy Model

The off-chain SDK exposes a structured policy tree:

- `Attribute(namespace, value)`
- `And(children)`
- `Or(children)`
- `Threshold { required, children }`

Examples:

- `AND(role:doctor, department:oncology)`
- `OR(role:doctor, role:emergency_responder)`
- `2-of(project:a, project:b, project:c)`
- `AND(region:KE, facility:uzima-main, valid_until:2026-12-31)`

The contract stores:

- `policy_ref`: canonical off-chain policy document reference
- `policy_hash`: immutable hash anchor
- `access_ciphertext_ref`: CP-ABE-wrapped DEK package reference
- `access_ciphertext_hash`: integrity anchor
- `required_permission`: mapped legacy permission anchor
- `attribute_count`
- `valid_until`
- `revocation_epoch`

## Permission Integration

The existing permission system remains authoritative:

- standard ACL checks still gate record visibility
- ABE policy metadata is attached to encrypted records, not used to replace the current ACL
- role and permission changes rotate the matching attribute-class epochs
- custom attributes such as `region`, `facility`, and `location` are issued through dedicated contract entrypoints

This means the old permission flow still works, while the ABE layer adds fine-grained cryptographic enforcement to encrypted records.

## Time and Location Attributes

- Time-based attributes are represented as issued attributes plus expiry windows.
- Location-based access is modeled as trusted, signed attributes like `location:nairobi`, `region:KE`, or `facility:uzima-main`.
- The SDK only treats location-style attributes as active when they are marked `is_verified=true`.
- No real-time GPS attestation is assumed.

## Revocation Strategy

Revocation uses a practical hybrid strategy:

- custom attribute revocation marks the attribute inactive on-chain
- the attribute-class epoch is incremented on-chain
- new CP-ABE access packages are compiled against the latest epoch
- existing record payloads stay encrypted as-is
- only the DEK access package needs rewrapping

Tradeoff:

- epoch rotation is coarse at the attribute-class level, so revoking `region:KE` invalidates all keys carrying that class until active users receive refreshed keys
- this avoids full medical-payload re-encryption and keeps review scope small

## Performance Notes

`abe_sdk` includes a benchmark harness for a `60-of-120` threshold policy. It prints the authorized decryption duration and the attribute count so reviewers can validate the `<200ms` target on their own hardware.

Because CP-ABE performance depends on CPU, pairing backend, and toolchain, treat the benchmark output as the source of truth.

## Security Assumptions and Limitations

- The contract does not execute CP-ABE pairings.
- Policy and ABE ciphertext bytes are anchored by hash on-chain but stored off-chain.
- Secret keys and decrypted DEKs must remain outside logs and telemetry.
- Location attributes must come from a trusted issuer.
- Revocation of shared attribute classes requires access-package refresh for still-authorized users.
- The included SDK uses real AC17 CP-ABE primitives for the leaf encryption step and a threshold-share tree for AND/OR/k-of-n composition.
