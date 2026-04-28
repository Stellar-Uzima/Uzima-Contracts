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
