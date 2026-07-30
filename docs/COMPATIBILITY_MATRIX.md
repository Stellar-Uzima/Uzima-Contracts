# Stellar Network & Toolchain Compatibility Matrix

This document defines the explicitly supported combinations of Stellar network versions, Soroban SDK versions, and Rust toolchains for the Uzima contract portfolio.

## Supported Combinations

| Soroban SDK | Rust Toolchain | Stellar Network | Status | Notes |
|-------------|----------------|-----------------|--------|-------|
| 21.7.7 | 1.92.0 | Testnet | **Active** | Primary development network |
| 21.7.7 | 1.92.0 | Futurenet | **Active** | Staging and pre-production |
| 21.7.7 | 1.92.0 | Mainnet | **Active** | Production deployments |
| 21.7.7 | 1.92.0 | Local | **Active** | Local development (Standalone) |
| 21.6.x | 1.85.0+ | Testnet | Deprecated | Migrate to 21.7.7 |
| 20.x.x | 1.79.0+ | Testnet | End-of-Life | No longer supported |

## Version Requirements

### Rust Toolchain

| Component | Version | Source |
|-----------|---------|--------|
| Rust | 1.92.0 | `rust-toolchain.toml` |
| rustfmt | 1.92.0 | Component |
| clippy | 1.92.0 | Component |
| rust-src | 1.92.0 | Component |
| WASM target | wasm32-unknown-unknown | Target |

### Soroban SDK

| Component | Version | Source |
|-----------|---------|--------|
| soroban-sdk | 21.7.7 | `Cargo.toml` workspace dependency |
| soroban-cli | 21.7.7 | `Cargo.toml` workspace dependency |
| soroban-env-host | 21.2.1 | Transitive dependency |

### Stellar Network Parameters

| Network | Passphrase | RPC URL | Safety Level |
|---------|-----------|---------|--------------|
| Local | `Standalone Network ; February 2017` | `http://localhost:8000/soroban/rpc` | Low |
| Testnet | `Test SDF Network ; September 2015` | `https://soroban-testnet.stellar.org` | Medium |
| Futurenet | `Test SDF Future Network ; October 2022` | `https://rpc-futurenet.stellar.org` | Medium |
| Mainnet | `Public Global Stellar Network ; September 2015` | `https://soroban-mainnet.stellar.org` | High |

## Resource Limits

| Resource | Limit | Source |
|----------|-------|--------|
| Max WASM size | 640 KB | Soroban protocol |
| Max read entries | 2,000 | Soroban protocol |
| Max write entries | 1,000 | Soroban protocol |
| Max instructions | 100,000,000 | Soroban protocol |
| Max memory | 50 MiB | Soroban protocol |

## Breaking Change Policy

- **Major SDK version change** (e.g. 21.x → 22.x): Requires migration guide, full regression testing, and governance vote
- **Minor SDK version change** (e.g. 21.7 → 21.8): Requires build verification and smoke tests
- **Patch SDK version change** (e.g. 21.7.7 → 21.7.8): Build verification only
- **Rust toolchain change**: Requires full workspace rebuild and CI verification

## Deprecation Timeline

| Version | Deprecation Date | Removal Date | Migration Guide |
|---------|-----------------|--------------|-----------------|
| soroban-sdk 21.6.x | 2026-01-01 | 2026-04-01 | Upgrade to 21.7.7 |
| soroban-sdk 20.x.x | 2025-06-01 | 2025-09-01 | N/A (end-of-life) |

## Verification Commands

```bash
# Check current toolchain
rustc --version
rustup show active-toolchain

# Check Soroban SDK version
cargo tree -p soroban-sdk | head -1

# Verify WASM build
cargo build --release --target wasm32-unknown-unknown

# Run compatibility check
./scripts/validate_multi_tenant_config.sh
```
