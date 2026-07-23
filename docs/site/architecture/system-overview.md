# System Overview

See [docs/SYSTEM_ARCHITECTURE.md](../../SYSTEM_ARCHITECTURE.md) for the full architecture diagram.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────┐
│                   Client Applications                │
│         (Web Portal, Mobile SDK, CLI)                │
└──────────────────────┬──────────────────────────────┘
                       │ Soroban RPC
┌──────────────────────▼──────────────────────────────┐
│                  Core Contracts                      │
│  medical_records │ identity_registry │ rbac │ audit  │
└──────────────────────┬──────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────┐
│               Payment Contracts                      │
│  healthcare_payment │ appointment_booking_escrow     │
│  escrow │ payment_router │ treasury_controller       │
└──────────────────────┬──────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────┐
│            Compliance & Security                     │
│  healthcare_compliance │ aml │ mfa │ zk_verifier     │
└─────────────────────────────────────────────────────┘
```

## Storage Strategy

| Storage Type | TTL | Use Case |
|-------------|-----|---------|
| `instance` | Contract lifetime | Config, counters |
| `persistent` | Extended (10000 ledgers) | Records, claims, escrows |
| `temporary` | Short (500 ledgers) | Reentrancy locks, sessions |

## Core Components Reference

Below are the key smart contracts in the Uzima ecosystem, along with direct links to their source code, test suites, and deployment scripts.

### 1. Medical Records
- **Contract Source**: [medical_records/src/lib.rs](../../contracts/medical_records/src/lib.rs)
- **Test Suite**: [medical_records/src/test.rs](../../contracts/medical_records/src/test.rs)
- **Deployment Flow**: [deploy_healthcare_integration.sh](../../scripts/deploy_healthcare_integration.sh)

### 2. Identity Registry
- **Contract Source**: [identity_registry/src/lib.rs](../../contracts/identity_registry/src/lib.rs)
- **Test Suite**: [identity_registry/src/comprehensive_tests.rs](../../contracts/identity_registry/src/comprehensive_tests.rs)
- **Deployment Flow**: [deploy_identity_registry.sh](../../scripts/deploy_identity_registry.sh)

### 3. Healthcare Payment
- **Contract Source**: [healthcare_payment/src/lib.rs](../../contracts/healthcare_payment/src/lib.rs)
- **Test Suite**: [healthcare_payment/src/test.rs](../../contracts/healthcare_payment/src/test.rs)
- **Deployment Flow**: [deploy_healthcare_integration.sh](../../scripts/deploy_healthcare_integration.sh)

### 4. Appointment Booking Escrow
- **Contract Source**: [appointment_booking_escrow/src/lib.rs](../../contracts/appointment_booking_escrow/src/lib.rs)
- **Test Suite**: [appointment_booking_escrow/src/test.rs](../../contracts/appointment_booking_escrow/src/test.rs)
- **Deployment Flow**: [deploy_all.sh](../../scripts/deploy_all.sh)

### 5. Escrow
- **Contract Source**: [escrow/src/lib.rs](../../contracts/escrow/src/lib.rs)
- **Test Suite**: Embedded in [escrow/src/lib.rs](../../contracts/escrow/src/lib.rs#L627)
- **Deployment Flow**: [deploy.sh](../../scripts/deploy.sh)
