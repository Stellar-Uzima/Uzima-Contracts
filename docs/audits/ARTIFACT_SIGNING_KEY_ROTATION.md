# Artifact Signing — Key Rotation Verification Gap

Tracks: #1570 "Verify artifact-signing key management and rotation is
documented and enforced"

## Policy (what should happen)

`docs/ARTIFACT_SIGNING.md` documents that release signing keys "should be
rotated every 90 days," and `scripts/manage_secrets.sh rotate` exists to
rotate a stored key/identity.

## Finding

`scripts/verify_artifacts.sh` (`verify_signatures()`) always verifies
against a single fixed path, `deployments/signing/release-signing-key.pem`
— whatever key currently lives there. Per-artifact `metadata.json` records
`signer_key` (the key **filename**, not a key ID/fingerprint), and
`manifest.json` records `signing_key` the same way. Neither records enough
to identify *which specific key material* signed a given artifact once the
file at that path has been replaced.

Practical effect: once a key is rotated (the `.pem` at that fixed path is
replaced), there is no supported way to re-verify artifacts that were
signed under the previous key — `verify_artifacts.sh` has no notion of "try
the archived key for this key ID." Rotating the key silently makes prior
releases unverifiable through the documented tooling, unless someone
manually swaps the old key file back in (which defeats the point of
rotating it).

## What landed instead of a full fix

Designing real key-ID-tagged metadata plus multi-key/archived-key lookup in
`verify_artifacts.sh` is a small protocol change to the signing metadata
schema, not a one-file fix, and touches release tooling directly — worth
its own reviewed PR rather than bundling into this one.

What's here:

- `scripts/test_key_rotation_invalidation.sh` — an informational,
  non-blocking script that reproduces the gap end-to-end (sign a fixture
  artifact, verify, rotate the key, verify again) against throwaway
  fixture files, so nothing needs to be built. It reports what it observes
  rather than hard-asserting, since the pre-rotation baseline hadn't
  previously been exercised in CI.
- Wired into `.github/workflows/security-checks.yml` as a non-blocking
  step.

## Recommendation (follow-up)

1. Add a `key_id` (e.g. a truncated public-key fingerprint) to
   `metadata.json` and `manifest.json` at sign time.
2. Store historical public keys (not private keys) in an archive directory
   keyed by `key_id`.
3. Update `verify_artifacts.sh` to look up the archived public key by the
   artifact's recorded `key_id` instead of always reading the current
   `release-signing-key.pem`.
4. Once that lands, flip `scripts/test_key_rotation_invalidation.sh`'s
   expectation: a rotated key should NOT invalidate verification of prior
   artifacts (only signing new ones requires the new key), and the check
   can become blocking.
