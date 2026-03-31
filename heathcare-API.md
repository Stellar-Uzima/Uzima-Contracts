🌟 Stellar Uzima - Decentralized Medical Records on Stellar
Stellar Uzima is a decentralized smart contract system for secure, encrypted, and role-based management of medical records on the Stellar blockchain using Soroban and Rust. The project enables healthcare providers and patients to maintain control over sensitive medical data while ensuring privacy, immutability, and auditability. Built specifically for healthcare organizations transitioning to blockchain-based record keeping, the system also respects traditional healing practices by supporting metadata for indigenous medical records.

The platform provides a comprehensive solution for modern healthcare data management, combining the security benefits of blockchain technology with practical healthcare workflows. It's designed for hospitals, clinics, research institutions, and healthcare providers who need to maintain patient confidentiality while enabling secure data sharing between authorized parties.

� Table of Contents
Project Overview
Setup Instructions
Prerequisites
Quick Start
Environment Setup
Running Tests
Network Configuration
Features
Architecture
Project Structure
Usage Examples
Deployment
Helpful Links
Contribution Guidelines
License
🎯 Project Overview
Stellar Uzima transforms medical record management by leveraging Stellar's blockchain infrastructure to create an immutable, secure, and patient-centric healthcare data ecosystem. The system addresses critical healthcare challenges including data breaches, interoperability issues, and patient privacy concerns through cryptographic security and decentralized governance.

Key Benefits:

Enhanced Security: Military-grade encryption protects sensitive medical data
Patient Control: Patients grant and revoke access to their records
Interoperability: Standardized format enables seamless data exchange
Audit Trail: Complete, immutable history of all record access and modifications
Cultural Respect: Support for traditional healing practices and metadata
Target Users:

Healthcare providers and hospitals
Medical research institutions
Health insurance companies
Patients seeking control over their medical data
Traditional medicine practitioners
🚀 Setup Instructions
✅ Prerequisites
Before you begin, ensure you have the following installed:

Rust 1.78.0+ - Install Rust
Soroban CLI v23.1.4+ - Install Soroban
Git - For version control
Make - For using the provided Makefile (optional but recommended)
⚡ Quick Start
Get up and running in under 5 minutes:

# Clone the repository
git clone https://github.com/your-org/Uzima-Contracts.git
cd Uzima-Contracts

# Run the automated setup script
chmod +x setup.sh
./setup.sh

# Or use the Makefile for step-by-step setup
make setup
🔧 Environment Setup
Option 1: Automated Setup (Recommended)
The setup.sh script handles everything automatically:

./setup.sh
This script will:

Install Rust 1.78.0 and required targets
Install Soroban CLI v23.1.4
Set up project structure
Configure Soroban networks (local, testnet, futurenet)
Build the project and run tests
Generate default identity
Option 2: Manual Setup
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
🧪 Running Tests
Ensure everything is working correctly:

# Run all tests
make test

# Or use cargo directly
cargo test --all

# Run specific test types
make test-unit          # Unit tests only
make test-integration   # Integration tests only
🌐 Network Configuration
The project supports multiple Stellar networks:

# Start local development network
make start-local
# or
soroban network start local

# Deploy to local network
make deploy-local

# Stop local network
make stop-local
Available Networks:

Local: http://localhost:8000/soroban/rpc (Development)
Testnet: https://soroban-testnet.stellar.org:443 (Testing)
Futurenet: https://rpc-futurenet.stellar.org:443 (Staging)
✨ Features
📁 Encrypted on-chain medical records storage
🔐 Role-based access control (patients, doctors, admins)
⏱ Immutable timestamping and full history tracking
📜 Integration of traditional healing metadata
🔑 Public key-based identity verification
⚙️ Fully testable, modular, and CI-enabled
📦 Gas-efficient contract design
🗳️ Decentralized governance with Governor + Timelock (proposals, voting, queued execution)
🏗️ Project Structure
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
� Usage Examples
Basic Contract Interaction
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
Using the Makefile
# Complete development workflow
make dev-deploy

# Individual steps
make build           # Build contracts
make test            # Run tests
make start-local     # Start local network
make deploy-local    # Deploy contracts
🚀 Deployment
Local Development
# Quick deployment to local network
make dev-deploy

# Step-by-step deployment
make clean
make build-opt
make dist
make start-local
make deploy-local
Testnet Deployment
# Configure testnet (if not already configured)
soroban config network add testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"

# Build for deployment
make build-opt

# Deploy to testnet
./scripts/deploy.sh medical_records testnet
Production Deployment
For production deployment on Stellar Mainnet:

Ensure you have sufficient XLM for deployment
Configure mainnet network settings
Use optimized builds: make build-opt
Consider using the provided Dockerfile for consistent builds
# Build production Docker image
docker build -t uzima-contracts .

# Deploy using Docker
docker run -it --rm -v $(PWD):/workspace uzima-contracts \
  make build-opt deploy-mainnet
🔄 CI/CD Pipeline
The project includes a comprehensive CI/CD pipeline for automated testing, building, security scanning, and deployment.

GitHub Actions Workflows
Continuous Integration (CI)
The CI workflow (.github/workflows/ci.yml) runs on every push and pull request:

Code Formatting: Checks code formatting with cargo fmt
Linting: Runs Clippy with strict warnings
Unit Tests: Executes all unit tests
Integration Tests: Runs integration tests with Soroban CLI
Build: Builds optimized WASM contracts
Shell Script Linting: Validates shell scripts with ShellCheck
# View CI status
# Check the "Actions" tab on GitHub or:
gh workflow view ci.yml
Security Scanning
The security workflow (.github/workflows/security.yml) performs automated security checks:

Dependency Audit: Scans for known vulnerabilities with cargo-audit
Security-focused Clippy: Checks for security anti-patterns
Secret Scanning: Detects hardcoded secrets with Gitleaks
Dependency Review: Reviews dependency changes in PRs
Runs on:

Every push to main/develop branches
Pull requests
Daily at 2 AM UTC (scheduled)
Automated Testnet Deployment
The deployment workflow (.github/workflows/deploy-testnet.yml) automatically deploys to testnet:

Triggers:

Pushes to develop branch
Version tags (e.g., v1.0.0)
Manual workflow dispatch
Process:

Runs pre-deployment tests (unless skipped)
Builds optimized contracts
Deploys to testnet
Verifies deployments
Creates deployment summary
Configuration:

Set up the following GitHub secrets:

TESTNET_DEPLOYER_SECRET_KEY: Secret key for testnet deployment account
# Manual deployment via GitHub Actions
gh workflow run deploy-testnet.yml \
  --field contract=medical_records \
  --field skip_tests=false
Deployment Scripts
Enhanced Deployment with Rollback
Deploy contracts with automatic backup and rollback support:

# Deploy with rollback enabled (default)
./scripts/deploy_with_rollback.sh medical_records testnet

# Deploy without rollback
./scripts/deploy_with_rollback.sh medical_records testnet default --no-rollback
Features:

Automatic backup of current deployment
Rollback on deployment failure
Contract verification after deployment
Deployment metadata tracking
Environment-based Deployment
Deploy all contracts to a specific environment:

# Deploy all contracts to testnet
./scripts/deploy_environment.sh testnet

# Deploy specific contracts
./scripts/deploy_environment.sh testnet --contracts medical_records,identity_registry

# Skip tests (not recommended)
./scripts/deploy_environment.sh testnet --skip-tests
Environments:

local: Local development network
testnet: Stellar testnet
futurenet: Stellar futurenet (staging)
mainnet: Production mainnet (requires confirmation)
Deployment Monitoring
Monitor deployed contracts and receive alerts:

# Monitor all deployments on testnet
./scripts/monitor_deployments.sh testnet

# Monitor with alerts on failure
./scripts/monitor_deployments.sh testnet --alert-on-failure
Features:

Health checks for all deployed contracts
Contract verification
Alert generation for unhealthy contracts
Deployment status reporting
Rollback Deployment
Rollback a contract to a previous version:

# Interactive rollback (selects latest backup)
./scripts/rollback_deployment.sh medical_records testnet

# Rollback to specific backup
./scripts/rollback_deployment.sh medical_records testnet \
  deployments/testnet_medical_records_backup_20240101_120000.json
Process:

Lists available backups
Verifies backup contract exists
Restores deployment configuration
Logs rollback action
Deployment Status
View deployment status across all networks:

# Show all deployments
./scripts/deployment_status.sh

# Show deployments for specific network
./scripts/deployment_status.sh testnet
Information displayed:

Contract names and IDs
Deployment timestamps
Rollback status
Backup availability
Rollback history
Deployment Workflow
Standard Deployment Process
Development: Make changes and test locally

make test
make build-opt
CI Validation: Push to branch triggers CI

Tests run automatically
Security scans execute
Build artifacts created
Testnet Deployment: Merge to develop triggers testnet deployment

Contracts deployed automatically
Verification runs
Status reported
Production Deployment: Manual deployment to mainnet

./scripts/deploy_environment.sh mainnet
Rollback Process
If a deployment fails or issues are detected:

Identify Issue: Check deployment logs and monitoring

./scripts/monitor_deployments.sh testnet
Review Backups: List available backups

./scripts/rollback_deployment.sh medical_records testnet
Execute Rollback: Restore previous version

./scripts/rollback_deployment.sh medical_records testnet <backup_file>
Verify: Confirm rollback success

./scripts/monitor_deployments.sh testnet
Deployment Artifacts
Deployment information is stored in the deployments/ directory:

deployments/
├── testnet_medical_records.json          # Current deployment
├── testnet_medical_records_backup_*.json # Backup files
├── rollback_log.json                     # Rollback history
└── alerts.log                            # Alert log
Deployment File Format:

{
  "contract_name": "medical_records",
  "contract_id": "C...",
  "network": "testnet",
  "deployer": "deployer-testnet",
  "deployed_at": "2025-01-15T10:30:00Z",
  "wasm_hash": "...",
  "commit_sha": "..."
}
CI/CD Best Practices
Always run tests locally before pushing
Review CI results before merging PRs
Monitor deployments after each release
Keep backups for critical deployments
Use rollback when issues are detected
Document any manual deployment steps
Troubleshooting
CI Failures
Formatting errors: Run cargo fmt --all
Clippy warnings: Fix warnings or add #[allow(...)] with justification
Test failures: Review test output and fix issues
Build failures: Check Rust version and dependencies
Deployment Failures
Network issues: Verify network connectivity and RPC endpoints
Insufficient funds: Fund deployment account
Contract errors: Check contract logs and verify WASM file
Identity issues: Ensure identity is properly configured
Rollback Issues
No backups: Previous deployments weren't backed up
Invalid backup: Backup file may be corrupted
Contract not found: Backup contract may have been removed
For more help, check the GitHub Issues or Discussions.

🔗 Helpful Links
Documentation
API Reference - Complete contract API documentation
Architecture Guide - System design and patterns
Soroban Documentation - Official Soroban docs
Stellar Developer Portal - Stellar ecosystem
Repository Resources
Contracts - Smart contract source code
Scripts - Deployment and utility scripts
Tests - Test suites and examples
CI/CD - GitHub Actions workflows
External Resources
Stellar Laboratory - Transaction builder and explorer
Stellar Expert - Blockchain explorer
Rust Documentation - Rust language reference
🤝 Contribution Guidelines
We welcome contributions from the community! Please follow these guidelines to ensure smooth collaboration.

Getting Started
Fork the repository on GitHub
Clone your fork locally:
git clone https://github.com/your-username/Uzima-Contracts.git
cd Uzima-Contracts
Add upstream remote:
git remote add upstream https://github.com/original-org/Uzima-Contracts.git
Development Workflow
Create a feature branch:

git checkout -b feature/your-feature-name
Make your changes following our coding standards:

Use cargo fmt for formatting
Run cargo clippy for linting
Ensure all tests pass: cargo test
Test thoroughly:

make test          # Run all tests
make check         # Run formatting, linting, and tests
Commit your changes:

git commit -m "feat: add your feature description"
Push to your fork:

git push origin feature/your-feature-name
Create a Pull Request with:

Clear description of changes
Links to relevant issues
Test results
Documentation updates (if applicable)
Code Standards
Rust: Follow official Rust style guidelines
Documentation: Include doc comments for all public functions
Tests: Maintain >80% code coverage
Commits: Use Conventional Commits format
Review Process
All PRs undergo:

Automated checks (CI/CD pipeline)
Code review by maintainers
Integration testing on testnet
Security audit for significant changes
Definition of Done
A contribution is complete when:

✅ All tests pass (cargo test)
✅ Code is formatted (cargo fmt)
✅ No linting warnings (cargo clippy)
✅ Documentation is updated
✅ CI/CD pipeline passes
✅ Security review completed (if applicable)
📄 License
This project is licensed under the MIT License - see the LICENSE file for details.

Copyright © 2025 Stellar Uzima Contributors

🆘 Support
Issues: GitHub Issues
Discussions: GitHub Discussions
Documentation: Project Docs
Built with ❤️ for the healthcare community