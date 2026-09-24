# Ed25519 Signature-Malleability Check Conformance — Gap Analysis

Tracks: #1566 "Enforce canonical Ed25519 signature-malleability checks
repo-wide"

## Finding

`contracts/common_error/src/sig_malleability.rs` implements
`check_s_value()`, a canonical low-S / non-malleable-signature check for
raw Ed25519 signatures. It was **not wired into its own crate** — the file
existed but `contracts/common_error/src/lib.rs` never declared `pub mod
sig_malleability;`, so it wasn't even part of the compiled `common_error`
crate's public API. Nothing could have imported it even if it tried.

Separately, and consistent with that: of the 4 contracts that call Soroban's
`env.crypto().ed25519_verify()` directly —

- `cross_chain_bridge`
- `fido2_authenticator`
- `medical_imaging_ai`
- `meta_tx_forwarder`

**none** of them depend on `common_error` at all (checked each contract's
`Cargo.toml`), let alone call `check_s_value`. So every raw-signature
verification path in the repo is currently exposed to Ed25519 signature
malleability (an attacker can produce a second valid signature `(R, S')`
for the same message by using `S' = order - S`, which breaks
signature-based replay/uniqueness assumptions unless the caller separately
enforces canonical-S or tracks used-signature nonces some other way).

## What landed

1. **Fixed the dead-module bug**: added `pub mod sig_malleability;` to
   `contracts/common_error/src/lib.rs` so the canonical check is actually
   part of the crate's compiled surface and importable. This is a
   zero-behavior-change fix for existing code (nothing depended on it
   before), so it's safe on its own.
2. This gap analysis, naming the exact 4 unprotected call sites.
3. `scripts/check_ed25519_malleability_conformance.sh` — an informational
   check that flags any `ed25519_verify` call site with no nearby
   `check_s_value`/`sig_malleability` reference in the same contract,
   wired into `.github/workflows/security-checks.yml`.

## What didn't land here

Actually calling `common_error::sig_malleability::check_s_value()` before
each `ed25519_verify()` in those 4 contracts is explicitly the kind of
authentication-primitive rewrite this issue's "Out of scope" section
excludes, and it's genuinely risky to guess at without compiling and
testing — each call site needs to extract the raw 64-byte signature in the
right form, decide what to do on rejection (return an error vs. panic),
and the change touches live signature-verification logic for
cross-chain-bridge and meta-tx-forwarder in particular, where a mistake is
a real security risk, not just a build break.

## Follow-ups (file as separate issues, one per contract)

1. `cross_chain_bridge` — add `check_s_value` before `ed25519_verify`.
2. `fido2_authenticator` — add `check_s_value` before
   `verify_ed25519_assertion`'s internal `ed25519_verify` call (see
   `docs/threat_models/fido2_authenticator.md`, which independently
   flagged this contract's assertion-verification path as needing review).
3. `medical_imaging_ai` — add `check_s_value` before `ed25519_verify`.
4. `meta_tx_forwarder` — add `check_s_value` before `ed25519_verify`
   (meta-transaction relaying is exactly the kind of context where
   signature malleability matters for replay protection).
5. Once all 4 are fixed, flip
   `scripts/check_ed25519_malleability_conformance.sh` to `--strict` in CI.
