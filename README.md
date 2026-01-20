# 🌟 Stellar Uzima - Decentralized Medical Records on Stellar

Stellar Uzima is a decentralized smart contract system for secure, encrypted, and role-based management of medical records on the Stellar blockchain using Soroban and Rust. The project enables healthcare providers and patients to maintain control over sensitive medical data while ensuring privacy, immutability, and auditability. Built specifically for healthcare organizations transitioning to blockchain-based record keeping, the system also respects traditional healing practices by supporting metadata for indigenous medical records.

The platform provides a comprehensive solution for modern healthcare data management, combining the security benefits of blockchain technology with practical healthcare workflows. It's designed for hospitals, clinics, research institutions, and healthcare providers who need to maintain patient confidentiality while enabling secure data sharing between authorized parties.

---

## � Table of Contents

- [Project Overview](#project-overview)
- [Setup Instructions](#setup-instructions)
  - [Prerequisites](#prerequisites)
  - [Quick Start](#quick-start)
  - [Environment Setup](#environment-setup)
  - [Running Tests](#running-tests)
  - [Network Configuration](#network-configuration)
- [Features](#features)
- [Architecture](#architecture)
- [Project Structure](#project-structure)
- [Usage Examples](#usage-examples)
- [Deployment](#deployment)
- [Helpful Links](#helpful-links)
- [Contribution Guidelines](#contribution-guidelines)
- [License](#license)

---

## 🎯 Project Overview

Stellar Uzima transforms medical record management by leveraging Stellar's blockchain infrastructure to create an immutable, secure, and patient-centric healthcare data ecosystem. The system addresses critical healthcare challenges including data breaches, interoperability issues, and patient privacy concerns through cryptographic security and decentralized governance.

**Key Benefits:**
- **Enhanced Security**: Military-grade encryption protects sensitive medical data
- **Patient Control**: Patients grant and revoke access to their records
- **Interoperability**: Standardized format enables seamless data exchange
- **Audit Trail**: Complete, immutable history of all record access and modifications
- **Cultural Respect**: Support for traditional healing practices and metadata

**Target Users:**
- Healthcare providers and hospitals
- Medical research institutions
- Health insurance companies
- Patients seeking control over their medical data
- Traditional medicine practitioners

---

## 🚀 Setup Instructions

### ✅ Prerequisites

Before you begin, ensure you have the following installed:

- **Rust 1.78.0+** - [Install Rust](https://www.rust-lang.org/tools/install)
- **Soroban CLI v23.1.4+** - [Install Soroban](https://soroban.stellar.org/docs/getting-started/installation)
- **Git** - For version control
- **Make** - For using the provided Makefile (optional but recommended)

### ⚡ Quick Start

Get up and running in under 5 minutes:

```bash
# Clone the repository
git clone https://github.com/your-org/Uzima-Contracts.git
cd Uzima-Contracts

# Run the automated setup script
chmod +x setup.sh
./setup.sh

# Or use the Makefile for step-by-step setup
make setup
```

### 🔧 Environment Setup

#### Option 1: Automated Setup (Recommended)

The `setup.sh` script handles everything automatically:

```bash
./setup.sh
```

This script will:
- Install Rust 1.78.0 and required targets
- Install Soroban CLI v23.1.4
- Set up project structure
- Configure Soroban networks (local, testnet, futurenet)
- Build the project and run tests
- Generate default identity

#### Option 2: Manual Setup

```bash
# Install Rust targets and components
rustup target add wasm32-unknown-unknown
rustup component add rustfmt clippy rust-src

# Install Soroban CLI
cargo install --locked soroban-cli

# Configure Soroban
soroban config identity generate default
soroban config network add local \
  --rpc-url http://localhost:8000/soroban/rpc \
  --network-passphrase "Standalone Network ; February 2017"

# Build the project
cargo build --all-targets

# Run tests to verify setup
cargo test --all
```

### 🧪 Running Tests

Ensure everything is working correctly:

```bash
# Run all tests
make test

# Or use cargo directly
cargo test --all

# Run specific test types
make test-unit          # Unit tests only
make test-integration   # Integration tests only
```

### 🌐 Network Configuration

The project supports multiple Stellar networks:

```bash
# Start local development network
make start-local
# or
soroban network start local

# Deploy to local network
make deploy-local

# Stop local network
make stop-local
```

**Available Networks:**
- **Local**: `http://localhost:8000/soroban/rpc` (Development)
- **Testnet**: `https://soroban-testnet.stellar.org:443` (Testing)
- **Futurenet**: `https://rpc-futurenet.stellar.org:443` (Staging)

---

## ✨ Features

- 📁 Encrypted on-chain medical records storage
- 🔐 Role-based access control (patients, doctors, admins)
- ⏱ Immutable timestamping and full history tracking
- 📜 Integration of traditional healing metadata
- 🔑 Public key-based identity verification
- ⚙️ Fully testable, modular, and CI-enabled
- 📦 Gas-efficient contract design
- 🗳️ Decentralized governance with Governor + Timelock (proposals, voting, queued execution)

---

## 🏗️ Project Structure

```
Uzima-Contracts/
│
├── contracts/
│   └── medical_records/
│       ├── src/
│       │   └── lib.rs         # Main contract logic
│       └── Cargo.toml         # Contract dependencies
│
├── scripts/                   # Deployment and interaction scripts
│   ├── deploy.sh             # Contract deployment
│   ├── interact.sh           # Contract interaction
│   └── test_scripts/         # Test utilities
│
├── tests/
│   ├── integration/          # Integration tests
│   └── unit/                 # Unit tests
│
├── docs/                     # Documentation
│   ├── api.md               # API reference
│   └── architecture.md      # Architecture details
│
├── .github/
│   └── workflows/
│       └── ci.yml            # Continuous integration
│
├── setup.sh                  # Automated setup script
├── makefile                  # Build automation
├── dockerfile               # Docker support
├── Cargo.toml               # Workspace configuration
└── README.md                # This file
```

---

## � Usage Examples

### Basic Contract Interaction

```bash
# Deploy the medical records contract
./scripts/deploy.sh medical_records local

# Initialize the contract with admin
./scripts/interact.sh <CONTRACT_ID> local initialize

# Register a new patient
./scripts/interact.sh <CONTRACT_ID> local register_patient \
  --patient-id "P12345" \
  --public-key "GD5..."

# Add a medical record
./scripts/interact.sh <CONTRACT_ID> local write_record \
  --patient-id "P12345" \
  --doctor-id "D67890" \
  --encrypted-data "QmXxx..." \
  --metadata "traditional_healing"
```

### Using the Makefile

```bash
# Complete development workflow
make dev-deploy

# Individual steps
make build           # Build contracts
make test            # Run tests
make start-local     # Start local network
make deploy-local    # Deploy contracts
```

---

## 🚀 Deployment

### Local Development

```bash
# Quick deployment to local network
make dev-deploy

# Step-by-step deployment
make clean
make build-opt
make dist
make start-local
make deploy-local
```

### Testnet Deployment

```bash
# Configure testnet (if not already configured)
soroban config network add testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"

# Build for deployment
make build-opt

# Deploy to testnet
./scripts/deploy.sh medical_records testnet
```

### Production Deployment

For production deployment on Stellar Mainnet:

1. Ensure you have sufficient XLM for deployment
2. Configure mainnet network settings
3. Use optimized builds: `make build-opt`
4. Consider using the provided Dockerfile for consistent builds

```bash
# Build production Docker image
docker build -t uzima-contracts .

# Deploy using Docker
docker run -it --rm -v $(PWD):/workspace uzima-contracts \
  make build-opt deploy-mainnet
```

---

## 🔗 Helpful Links

### Documentation
- [API Reference](./docs/api.md) - Complete contract API documentation
- [Architecture Guide](./docs/architecture.md) - System design and patterns
- [Soroban Documentation](https://soroban.stellar.org/docs) - Official Soroban docs
- [Stellar Developer Portal](https://developers.stellar.org/) - Stellar ecosystem

### Repository Resources
- [Contracts](./contracts/) - Smart contract source code
- [Scripts](./scripts/) - Deployment and utility scripts
- [Tests](./tests/) - Test suites and examples
- [CI/CD](./.github/workflows/) - GitHub Actions workflows

### External Resources
- [Stellar Laboratory](https://laboratory.stellar.org/) - Transaction builder and explorer
- [Stellar Expert](https://stellar.expert/) - Blockchain explorer
- [Rust Documentation](https://doc.rust-lang.org/) - Rust language reference

---

## 🤝 Contribution Guidelines

We welcome contributions from the community! Please follow these guidelines to ensure smooth collaboration.

### Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/your-username/Uzima-Contracts.git
   cd Uzima-Contracts
   ```
3. **Add upstream remote**:
   ```bash
   git remote add upstream https://github.com/original-org/Uzima-Contracts.git
   ```

### Development Workflow

1. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** following our coding standards:
   - Use `cargo fmt` for formatting
   - Run `cargo clippy` for linting
   - Ensure all tests pass: `cargo test`

3. **Test thoroughly**:
   ```bash
   make test          # Run all tests
   make check         # Run formatting, linting, and tests
   ```

4. **Commit your changes**:
   ```bash
   git commit -m "feat: add your feature description"
   ```

5. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

6. **Create a Pull Request** with:
   - Clear description of changes
   - Links to relevant issues
   - Test results
   - Documentation updates (if applicable)

### Code Standards

- **Rust**: Follow official Rust style guidelines
- **Documentation**: Include doc comments for all public functions
- **Tests**: Maintain >80% code coverage
- **Commits**: Use [Conventional Commits](https://www.conventionalcommits.org/) format

### Review Process

All PRs undergo:
1. **Automated checks** (CI/CD pipeline)
2. **Code review** by maintainers
3. **Integration testing** on testnet
4. **Security audit** for significant changes

### Definition of Done

A contribution is complete when:
- ✅ All tests pass (`cargo test`)
- ✅ Code is formatted (`cargo fmt`)
- ✅ No linting warnings (`cargo clippy`)
- ✅ Documentation is updated
- ✅ CI/CD pipeline passes
- ✅ Security review completed (if applicable)

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

**Copyright © 2025 Stellar Uzima Contributors**

---

## 🆘 Support

- **Issues**: [GitHub Issues](https://github.com/your-org/Uzima-Contracts/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-org/Uzima-Contracts/discussions)
- **Documentation**: [Project Docs](./docs/)

---

*Built with ❤️ for the healthcare community*
