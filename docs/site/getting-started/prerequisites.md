# Prerequisites

Before you begin, ensure you have the following installed:

## Required Tools

| Tool | Version | Install |
|------|---------|---------|
| Rust | 1.78.0+ | [rustup.rs](https://rustup.rs) |
| Soroban CLI | 21.7.7 | `cargo install --locked --version 21.7.7 soroban-cli` |
| Git | Any | [git-scm.com](https://git-scm.com) |

## Rust Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Add WASM target (required for Soroban)
rustup target add wasm32-unknown-unknown

# Add required components
rustup component add rustfmt clippy rust-src
```

## Soroban CLI

```bash
cargo install --locked --version 21.7.7 soroban-cli
```

## Optional Tools

- **Make** — For using the provided Makefile shortcuts
- **Docker** — For containerized builds and deployments
- **jq** — For parsing JSON output from CLI commands

## Verify Installation

```bash
rustc --version        # rustc 1.78.0 or higher
soroban --version      # 21.7.7
cargo --version        # cargo 1.78.0 or higher
```
