# Deployment WASM Hash Records

This directory stores the SHA-256 hashes of the WASM artifacts that were built
for each tagged release and deployed to a given network. It is the source of
truth for **deterministic build verification** (Issue #854): auditors sign a
specific binary, and these records let anyone confirm the deployed/rebuilt
bytecode matches that audited artifact.

## Layout

```
deployments/
  <network>/            # testnet | mainnet
    <release>/          # e.g. v1.0.0 or a git tag
      hashes.txt        # recorded sha256 of each contract .wasm + Signed-by
```

## `hashes.txt` format

```
# Uzima-Contracts deployment WASM hashes
# network:   mainnet
# release:   v1.0.0
# generated: 2026-06-21T12:00:00Z
# toolchain: rustc 1.92.0 (...)
Signed-by: <auditor ed25519 pubkey>
#
# <sha256>  <artifact>
<64-hex>  access_control.wasm
<64-hex>  audit.wasm
...
```

## Workflow

1. **Build** the release artifacts from the pinned toolchain
   (`rust-toolchain.toml`): `make dist`.
2. **Sign** release artifacts and record checksums:

   ```sh
   ./scripts/sign_release_artifacts.sh v1.0.0
   ```

   This generates SHA256SUMS.txt, creates a GPG signature, and builds a
   release manifest at `artifacts/release-v1.0.0/`.

3. **Record** the hashes for the network/release and attach the auditor's
   signing pubkey:

   ```sh
   ./scripts/verify_deployment.sh record mainnet v1.0.0 <auditor_pubkey>
   git add deployments/mainnet/v1.0.0/hashes.txt && git commit
   ```

4. **Verify** at any later point (and in CI) that a fresh build still matches
   the audited record:

   ```sh
   make dist
   ./scripts/verify_release_artifacts.sh v1.0.0 mainnet
   ```

   `verify_release_artifacts.sh` checks WASM checksums, deployment hashes,
   release manifest integrity, and build provenance. It exits non-zero on
   any mismatch, failing CI.

See [`docs/SECURITY_BEST_PRACTICES.md`](../docs/SECURITY_BEST_PRACTICES.md)
("Deterministic Build Verification") and
[`docs/SOROBAN_CLI_DEPLOYMENT.md`](../docs/SOROBAN_CLI_DEPLOYMENT.md) for the
full process and threat model.
