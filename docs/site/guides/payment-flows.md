# Payment Flows

See [docs/PAYMENT_FLOW_DIAGRAMS.md](../../PAYMENT_FLOW_DIAGRAMS.md) for full diagrams.

## Appointment Payment Flow

```
Patient → book_appointment() → Funds locked in escrow
Provider → confirm_appointment() → Funds released to provider
Patient → refund_appointment() → Funds returned to patient
```

## Insurance Claim Flow

```
Provider → submit_claim() → Claim: Submitted
Admin → approve_claim() → Claim: Approved
System → process_payment() → Claim: Paid
         ↓
   payment_router.compute_split()
         ↓
   provider receives amount - fee
   treasury receives fee
```

## Escrow Pull-Payment Flow

```
create_escrow() → Escrow: Pending
approve_release() × 2 → Escrow: Active
release_escrow() → Credits added to balances
withdraw() → Actual token transfer to recipient
```

The pull-payment pattern in `escrow` means tokens are never pushed directly — recipients must call `withdraw()` to claim their balance.
