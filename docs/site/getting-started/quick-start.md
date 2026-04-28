# Quick Start

Get up and running in under 5 minutes.

## 1. Clone the Repository

```bash
git clone https://github.com/Stellar-Uzima/Uzima-Contracts.git
cd Uzima-Contracts
```

## 2. Run Automated Setup

```bash
chmod +x setup.sh
./setup.sh
```

This script installs Rust, Soroban CLI, configures networks, builds contracts, and runs tests.

## 3. Build Contracts

```bash
cargo build --all-targets
# or
make build
```

## 4. Run Tests

```bash
cargo test --all
# or
make test
```

## 5. Deploy to Local Network

```bash
# Start local Stellar node
make start-local

# Deploy all contracts
make deploy-local
```

## What's Next?

- [Environment Setup](environment-setup.md) — Detailed configuration options
- [API Reference](../api-reference/overview.md) — Explore contract functions
- [Deployment Guide](../guides/deployment.md) — Deploy to testnet/mainnet
