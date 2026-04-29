# Contract Interactions

See [docs/CONTRACT_INTERACTIONS.md](../../CONTRACT_INTERACTIONS.md).

## Key Interaction Patterns

### Payment Router Pattern
`healthcare_payment` calls `payment_router.compute_split()` to determine fee splits before transferring tokens.

### Escrow Pattern
`appointment_booking_escrow` holds tokens in the contract address and transfers them on confirmation or refund.

### Pull-Payment Pattern
`escrow` credits balances internally; recipients call `withdraw()` to claim tokens.

### Cross-Contract Auth
When contract A calls contract B, B's `require_auth()` checks are satisfied by the calling contract's authorization context.
