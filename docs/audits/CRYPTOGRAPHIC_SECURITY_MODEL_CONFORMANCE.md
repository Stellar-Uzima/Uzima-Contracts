# Cryptographic Security Model Conformance — Gap Analysis

Tracks: #1565 "Audit cryptographic security model conformance across
contracts"

## Policy (what should happen)

`docs/CRYPTOGRAPHIC_SECURITY_MODEL.md` defines the expected crypto posture:
SHA-256 for content hashing (`sha256(ciphertext) == ciphertext_hash`),
versioned key bundles with rotation support (`crypto_registry`), and
threshold + timelock governance for admin-level crypto config changes
(`propose_crypto_config_update` / `approve_crypto_config_update` /
`execute_crypto_config_update`).

## What was checked

1. **Weak hash algorithms.** Searched `Cargo.lock` and every contract's
   `Cargo.toml` for `md5`/`sha1` dependencies. **None found** — no contract
   depends on a deprecated hash crate. This is a genuine clean result, not
   an oversight.

2. **Hand-rolled crypto vs. host functions.** Contracts should route actual
   cryptographic operations (hashing, signature verification) through
   Soroban's `env.crypto()` host functions rather than hand-rolling them in
   pure Rust. Grepped for `encrypt`/`decrypt`/`hash` function names without
   a nearby `env.crypto()` call:

   - `credential_registry`
   - `homomorphic_registry`
   - `iot_device_management`
   - `medical_imaging`

   **These are not confirmed violations.** At least `homomorphic_registry`
   is expected to flag here — `docs/CRYPTOGRAPHIC_SECURITY_MODEL.md`'s
   "Privacy-preserving computations (HE + MPC)" section describes
   homomorphic encryption as a deliberately separate primitive class that
   isn't meant to go through `env.crypto()`. The other three need a human
   read of each contract to confirm whether they do real cryptography
   in-contract (a finding) or just store/reference externally-computed
   hashes (not a finding). That triage wasn't done here — flagged as a
   follow-up.

3. **Threshold/timelock governance for admin crypto config.** Grepped for
   the `propose_*_update`/`approve_*_update`/`timelock` pattern described
   for admin-level crypto config changes. It shows up in `governor`,
   `timelock`, `treasury_controller`, `upgrade_manager`, `identity_registry`,
   `medical_records`, `cross_chain_access`, `zkp_registry`, and
   `storage-snapshot` — but **not** in `crypto_registry` itself, the
   contract the doc uses as the canonical example. This needs a closer
   read to confirm whether `crypto_registry`'s own admin config surface
   (if any) is threshold-gated by a different mechanism, or whether the
   doc's example contract is actually the gap. Not resolved here — flagged
   as a follow-up requiring someone to read `crypto_registry/src/lib.rs`
   end-to-end.

## What landed instead of a full fix

Both open questions above need a careful per-contract read to turn into
confirmed findings rather than heuristic false-positive risk. What's here:

- This gap analysis with the concrete grep results and their caveats.
- `scripts/check_crypto_conformance.sh` — an informational check that
  reproduces check #1 and #2 above (weak-hash dependency scan +
  hand-rolled-crypto heuristic), wired into
  `.github/workflows/security-checks.yml`. Check #3 (governance pattern
  presence) isn't automated yet since "is this function actually
  admin-privileged" can't be determined by grep alone.

## Follow-ups (file as separate issues)

1. Manually triage `credential_registry`, `iot_device_management`,
   `medical_imaging` for hand-rolled crypto (confirm or dismiss).
2. Read `crypto_registry/src/lib.rs` to confirm whether its admin config
   surface (if any) actually has threshold/timelock protection, and
   reconcile with the doc's framing of it as the canonical example.
3. Once findings are confirmed, decide whether to expand
   `check_crypto_conformance.sh` into a `--strict` gate for new contracts.
