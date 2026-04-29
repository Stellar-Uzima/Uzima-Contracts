# Integration Guide

See [docs/HEALTHCARE_INTEGRATION.md](../../HEALTHCARE_INTEGRATION.md) for the full integration guide.

## Quick Integration Steps

1. Deploy required contracts (see [Deployment Guide](deployment.md))
2. Initialize contracts with admin address
3. Register patient and provider identities via `identity_registry`
4. Use `medical_records` for record storage
5. Use `appointment_booking_escrow` for appointment payments
6. Use `healthcare_payment` for insurance claim processing

## SDK Clients

Use the Soroban TypeScript SDK for frontend integration:

```typescript
import { Contract, SorobanRpc } from '@stellar/stellar-sdk';

const server = new SorobanRpc.Server('https://soroban-testnet.stellar.org');
const contract = new Contract(CONTRACT_ID);
```

## See Also

- [docs/HEALTHCARE_INTEGRATION_INDEX.md](../../HEALTHCARE_INTEGRATION_INDEX.md)
- [docs/EMR_INTEGRATION.md](../../EMR_INTEGRATION.md)
