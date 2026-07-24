# Artifact Signing and Verification

This document describes the standardized signing and verification process for Uzima WASM contract artifacts.

## Overview

All contract WASM artifacts are signed and checksummed to ensure integrity and provenance during deployment. The signing process creates per-artifact signatures that can be verified independently.

## Scripts

### `scripts/sign_artifacts.sh`

Signs WASM contract artifacts and creates per-artifact signatures alongside SHA256 checksums.

```bash
# Basic signing with ephemeral key
./scripts/sign_artifacts.sh --version 1.0.0

# Signing with a specific key for mainnet
./scripts/sign_artifacts.sh --version 1.0.0 --key /path/to/release-key.pem --network mainnet

# Preview without writing
./scripts/sign_artifacts.sh --version 1.0.0 --dry-run
```

**Options:**
- `--key <path>` - Path to signing key (GPG or ed25519 PEM)
- `--version <version>` - Release version tag (required)
- `--network <network>` - Target network (default: testnet)
- `--dry-run` - Show what would be signed without writing
- `--force` - Overwrite existing signatures

### `scripts/verify_artifacts.sh`

Verifies WASM artifact signatures and checksums against recorded values.

```bash
# Verify all artifacts
./scripts/verify_artifacts.sh

# Verify specific version
./scripts/verify_artifacts.sh --version 1.0.0

# Verify against specific network
./scripts/verify_artifacts.sh --version 1.0.0 --network mainnet --verbose
```

## Signing Directory Structure

```
deployments/
  signing/
    release-signing-key.pem      # Signing key (not committed)
    manifest.json                # Signing manifest for the release
    artifacts/
      <contract_name>/
        SHA256SUM                # Checksum file
        SHA256SUM.sig            # Signature of the checksum
        metadata.json            # Per-artifact metadata
```

## Signing Configuration

Configuration is stored in `deployments/signing-config.json` and controls:

- **Algorithm**: ed25519 for signatures, SHA256 for checksums
- **Key rotation**: Keys should be rotated every 90 days
- **Verification policy**: Which checks are required for deployment
- **CI integration**: Whether to run on PRs and releases

## Release Workflow

1. **Build** artifacts from the pinned toolchain:
   ```bash
   make dist
   ```

2. **Sign** the artifacts:
   ```bash
   ./scripts/sign_artifacts.sh --version 1.0.0
   ```

3. **Verify** locally before pushing:
   ```bash
   ./scripts/verify_artifacts.sh --version 1.0.0
   ```

4. **Commit** the signing records:
   ```bash
   git add deployments/signing/
   git commit -m "chore(signing): sign artifacts for v1.0.0"
   ```

5. **Deploy** and verify on-chain:
   ```bash
   ./scripts/deploy.sh <contract> <network>
   ./scripts/verify_artifacts.sh --version 1.0.0 --network <network>
   ```

## Integration with Existing Scripts

The signing process integrates with:

- **`sign_release_artifacts.sh`**: Creates the release-level checksums and manifests
- **`verify_release_artifacts.sh`**: Verifies release-level integrity
- **`deploy.sh`**: Records WASM hashes during deployment
- **`verify_deployment.sh`**: Validates deployed artifacts against records

## Security Considerations

- Never commit signing keys to the repository
- Use hardware security modules (HSMs) for production signing keys
- Rotate signing keys regularly (every 90 days recommended)
- Verify signatures before deploying to production networks
- The `deployments/signing/` directory should be in `.gitignore` for keys but signing records should be committed
