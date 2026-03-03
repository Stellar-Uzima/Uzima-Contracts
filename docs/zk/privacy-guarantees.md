# ZK Privacy Guarantees

## Hidden Data
The system is designed so the following are not published in events or state during access verification:
- Medical record plaintext fields used in off-chain witness construction.
- Full credential claim payloads.
- Credential holder identity in clear (replaced by commitments/pseudonym).

## Revealed Data
The following is intentionally visible on-chain:
- `record_id` (or equivalent record commitment reference).
- Verification result (`true`/`false`).
- Timestamp.
- Nullifier hash (optional event field).
- Pseudonym hash.
- Credential root commitment (not raw claims).

## Guarantee Set
1. Access confidentiality:
   Access events do not include diagnosis/treatment or credential claim values.
2. Selective disclosure:
   Predicate checks are represented via commitments and public-input constraints rather than claim plaintext.
3. Replay mitigation:
   Nullifiers are one-time; reused nullifiers are rejected.
4. Verifier isolation:
   On-chain path only checks attested proof/public-input hashes and key version; witness stays off-chain.

## Limitations
1. Metadata leakage:
   Record IDs and access timestamps remain visible.
2. Trusted attestor assumption:
   This architecture trusts attestor-authenticated proof attestations when full in-contract SNARK verification is not feasible.
3. Linkability risk:
   Reuse of the same pseudonym derivation strategy across contexts may allow correlation. Domain separation and per-record nullifier use reduce this.

