# API Reference Overview

All contracts are written in Rust using the Soroban SDK. Functions are invoked via the Soroban CLI or SDK clients.

## Calling Conventions

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --network <NETWORK> \
  --source <IDENTITY> \
  -- <FUNCTION_NAME> \
  --arg1 value1
```

## Common Types

| Type | Description |
|------|-------------|
| `Address` | Stellar account or contract address |
| `i128` | Token amounts (stroops) |
| `u64` | IDs, timestamps (Unix seconds) |
| `String` | UTF-8 string (Soroban `String`) |
| `BytesN<32>` | Fixed-length byte array (hashes) |

## Error Handling

All fallible functions return `Result<T, Error>`. Error codes are stable integers — see [Error Codes](error-codes.md).

## Authentication

Functions that modify state require `address.require_auth()`. The caller must sign the transaction with the corresponding key.

## Contract Index

- [Medical Records](medical-records.md)
- [Healthcare Payment](healthcare-payment.md)
- [Appointment Booking Escrow](appointment-booking-escrow.md)
- [Identity Registry](identity-registry.md)
- [Escrow](escrow.md)
- [Error Codes](error-codes.md)
- [Events Reference](events.md)
