# Overview

Stellar Uzima transforms medical record management by leveraging Stellar's blockchain infrastructure to create an immutable, secure, and patient-centric healthcare data ecosystem.

## Key Benefits

- **Enhanced Security** — Military-grade encryption protects sensitive medical data
- **Patient Control** — Patients grant and revoke access to their records
- **Interoperability** — Standardized format enables seamless data exchange
- **Audit Trail** — Complete, immutable history of all record access and modifications
- **Cultural Respect** — Support for traditional healing practices and metadata

## Contract Ecosystem

The platform consists of 90+ Soroban smart contracts organized into functional domains:

### Core Contracts
| Contract | Purpose |
|----------|---------|
| `medical_records` | Primary medical record storage and access control |
| `identity_registry` | W3C DID-based identity and credential management |
| `rbac` | Role-based access control primitives |
| `audit` | Immutable audit trail for all data access |

### Payment Contracts
| Contract | Purpose |
|----------|---------|
| `healthcare_payment` | Insurance claim processing and payment routing |
| `appointment_booking_escrow` | Appointment scheduling with escrow-backed payments |
| `escrow` | General-purpose escrow with pull-payment pattern |
| `payment_router` | Payment splitting and routing logic |

### Compliance & Security
| Contract | Purpose |
|----------|---------|
| `healthcare_compliance` | HIPAA/regulatory compliance enforcement |
| `aml` | Anti-money laundering checks |
| `mfa` | Multi-factor authentication |
| `zk_verifier` | Zero-knowledge proof verification |

## Technology Stack

- **Blockchain**: Stellar (Soroban smart contracts)
- **Language**: Rust
- **SDK**: soroban-sdk v21.7.7 (exact pinned)
- **Standards**: W3C DID, FHIR R4, IHE profiles

## Next Steps

- [Prerequisites](prerequisites.md) — What you need before starting
- [Quick Start](quick-start.md) — Get running in 5 minutes
- [Environment Setup](environment-setup.md) — Detailed setup instructions
