# Uzima Contract Ecosystem — Canonical Glossary

This document defines the standard terminology used across the Uzima healthcare contract portfolio. All contributors should use these terms consistently in code, documentation, and communication.

## Core Concepts

### Blockchain & Soroban

| Term | Definition |
|------|-----------|
| **Soroban** | The smart contract platform built into the Stellar blockchain, used by all Uzima contracts. |
| **Stellar** | The Layer 1 blockchain network that hosts Soroban smart contracts. |
| **Contract** | A deployed Soroban smart contract containing business logic and storage. |
| **Entry Point** | A public function on a Soroban contract that can be invoked by external callers. |
| **WASM** | WebAssembly — the compilation target for Soroban contracts (`wasm32-unknown-unknown`). |
| **Soroban SDK** | The Rust library (`soroban-sdk`) used to write Soroban contracts. |
| **Contract ID** | A unique 56-character address identifying a deployed Soroban contract. |
| **Transaction** | An atomic unit of work on Stellar containing one or more contract invocations. |
| **Footprint** | The set of storage keys a Soroban transaction declares it will read or write. |
| **Soroban RPC** | The JSON-RPC interface for interacting with Soroban contracts on Stellar. |

### Uzima Platform

| Term | Definition |
|------|-----------|
| **Uzima** | The healthcare data management platform built on Stellar/Soroban. |
| **Uzima Token (SUT)** | The utility token used for access control, payments, and governance within Uzima. |
| **Tenant** | An organization (hospital, clinic, research institution) using the Uzima platform. |
| **Multi-tenancy** | The architecture where multiple organizations share the same contracts with isolated data. |
| **Network** | The Stellar network instance (testnet, futurenet, mainnet, local Standalone). |

## Healthcare Domain

| Term | Definition |
|------|-----------|
| **Medical Record** | A digital record of a patient's health information, stored on-chain as encrypted data. |
| **PHI** | Protected Health Information — any individually identifiable health information (HIPAA term). |
| **EHR** | Electronic Health Record — a comprehensive digital record of a patient's health history. |
| **FHIR** | Fast Healthcare Interoperability Resources — a standard for exchanging healthcare data. |
| **DICOM** | Digital Imaging and Communications in Medicine — standard for medical imaging data. |
| **Patient Consent** | Explicit permission from a patient for specific uses of their health data. |
| **Consent NFT** | A non-fungible token representing a specific patient consent grant. |
| **Provider** | A healthcare professional or institution authorized to access patient records. |
| **Provider Directory** | A registry of verified healthcare providers within the Uzima ecosystem. |

## Access Control & Identity

| Term | Definition |
|------|-----------|
| **RBAC** | Role-Based Access Control — permissions assigned based on user roles. |
| **ABE** | Attribute-Based Encryption — encryption tied to user attributes for fine-grained access. |
| **DID** | Decentralized Identifier — a self-sovereign identity standard used for user identity. |
| **ZKP** | Zero-Knowledge Proof — cryptographic proof that reveals no information beyond validity. |
| **MFA** | Multi-Factor Authentication — requiring multiple forms of identity verification. |
| **FIDO2** | Fast Identity Online — a passwordless authentication standard. |
| **Emergency Access** | A break-glass mechanism allowing access without normal authorization during emergencies. |
| **Access Grant** | A specific permission allowing a provider to access a patient's record. |
| **Session Token** | A time-limited credential authorizing a series of contract invocations. |
| **Custodial Wallet** | A wallet managed by the platform on behalf of the user (for key recovery). |

## Payments & Financial

| Term | Definition |
|------|-----------|
| **Escrow** | Funds held in a contract until specified conditions are met. |
| **Healthcare Payment** | A payment for medical services processed through Uzima contracts. |
| **Payment Router** | A contract that routes payments to the correct recipient contract. |
| **Treasury Controller** | The contract managing platform-level fund flows and reserves. |
| **Fee** | A percentage deducted from transactions for platform operations. |
| **Settlement** | The final confirmation that a payment has been processed and recorded. |

## Governance & Upgrades

| Term | Definition |
|------|-----------|
| **Governor** | The top-level governance contract controlling protocol parameters. |
| **Timelock** | A delay mechanism requiring waiting period before executing governance actions. |
| **Proposal** | A formal suggestion to change protocol parameters or upgrade contracts. |
| **Quorum** | The minimum number of votes required for a governance proposal to pass. |
| **Upgrade Manager** | A contract coordinating safe upgrades of other contracts. |
| **Storage Migration** | The process of moving data between contract versions during upgrades. |
| **Versioning (SemVer)** | Semantic versioning — MAJOR.MINOR.PATCH format for contract releases. |

## Cross-Chain

| Term | Definition |
|------|-----------|
| **Cross-Chain Bridge** | Infrastructure connecting Uzima to other blockchain networks. |
| **Bridge Validator** | A node that verifies and relays cross-chain transactions. |
| **Reorg Protection** | Mechanisms to handle blockchain reorganizations affecting cross-chain state. |
| **Sync Manager** | A contract coordinating cross-chain state synchronization. |
| **Chain ID** | A unique identifier for each supported blockchain network. |

## Security & Compliance

| Term | Definition |
|------|-----------|
| **Audit Trail** | An immutable log of all data access and modification events. |
| **Threat Model** | A systematic analysis of potential security threats to the system. |
| **Compliance Framework** | A set of regulations and standards the platform must adhere to (HIPAA, GDPR, SOC 2). |
| **Data Sovereignty** | The principle that data is subject to the laws of the nation where it is stored. |
| **Encryption at Rest** | Encrypting stored data to protect it from unauthorized access. |
| **Key Rotation** | The practice of periodically changing cryptographic keys to limit exposure. |
| **Incident Response** | The organized process for handling security incidents. |
| **Penetration Testing** | Authorized simulated attacks to identify security vulnerabilities. |

## Development & Operations

| Term | Definition |
|------|-----------|
| **Workspace** | The Rust workspace containing all Uzima contracts and libraries. |
| **Library (lib)** | Shared Rust code used across multiple contracts (not deployed independently). |
| **Schema** | A JSON definition describing the structure of data or events. |
| **Interface Registry** | A registry mapping contract capabilities to their interface definitions. |
| **CI/CD** | Continuous Integration / Continuous Deployment — automated build and deploy pipelines. |
| **Smoke Test** | A quick test verifying basic functionality after deployment. |
| **Performance Budget** | A predefined limit on resource usage (instructions, memory, storage). |
| **WASM Size** | The compiled size of a contract, limited to 640 KB by Soroban protocol. |
| **Fuzzing** | Automated testing with random inputs to find edge cases and bugs. |
| **Property Test** | A test that verifies a property holds for all valid inputs. |

## Event System

| Term | Definition |
|------|-----------|
| **Event** | An immutable log entry emitted by a contract during execution. |
| **Event Schema** | A JSON Schema defining the structure of a specific event type. |
| **Event Envelope** | The standard wrapper around all Uzima events with metadata (timestamp, contract, tx hash). |
| **Diagnostic Event** | A Soroban-specific event emitted for debugging purposes. |
| **Event Registry** | The central catalog of all event schemas used across contracts. |

## Acronyms Quick Reference

| Acronym | Full Form |
|---------|-----------|
| PHI | Protected Health Information |
| EHR | Electronic Health Record |
| FHIR | Fast Healthcare Interoperability Resources |
| DICOM | Digital Imaging and Communications in Medicine |
| RBAC | Role-Based Access Control |
| ABE | Attribute-Based Encryption |
| DID | Decentralized Identifier |
| ZKP | Zero-Knowledge Proof |
| MFA | Multi-Factor Authentication |
| HIPAA | Health Insurance Portability and Accountability Act |
| GDPR | General Data Protection Regulation |
| SOC 2 | Service Organization Control 2 |
| SemVer | Semantic Versioning |
| WASM | WebAssembly |
| RPC | Remote Procedure Call |
| CI/CD | Continuous Integration / Continuous Deployment |
