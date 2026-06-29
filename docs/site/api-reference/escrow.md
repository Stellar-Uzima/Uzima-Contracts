# Escrow

Contract: `escrow`

General-purpose escrow with pull-payment pattern, reentrancy guard, and platform fee support.

## Security

- Reentrancy guard via temporary storage lock (`REENTRANCY_LOCK`)
- Pull-payment pattern: funds are credited to balances, not pushed directly
- State updated before any external interaction (CEI pattern)

## Key Functions

### `initialize(admin) → Result<(), Error>`

Initialize the contract.

### `set_fee_config(caller, fee_receiver, platform_fee_bps) → Result<(), Error>`

Configure platform fee (in basis points, max 10000).

### `create_escrow(order_id, payer, payee, amount, token) → Result<bool, Error>`

Create a new escrow. Transitions to `Pending`.

### `approve_release(order_id, approver) → Result<(), Error>`

Approve release of funds. Transitions `Pending → Active` on first approval.

### `release_escrow(order_id) → Result<bool, Error>`

Release funds to payee (requires ≥2 approvals). Credits balances via pull-payment.

**State transition**: `Active/Disputed → Settled`

### `refund_escrow(order_id, reason) → Result<bool, Error>`

Refund funds to payer.

**State transition**: `Active/Disputed → Refunded`

### `withdraw(caller, token, to) → Result<i128, Error>`

Withdraw credited balance. Caller must equal `to`.

### `get_credit(addr) → i128`

Get pending withdrawal balance for an address.

## Escrow Status Flow

```
Pending → Active → Settled
       ↘ Disputed → Refunded
```
