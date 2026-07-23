# Escrow

Contract: `escrow`

General-purpose escrow with pull-payment pattern, reentrancy guard, and platform fee support.

## Security

- Reentrancy guard via temporary storage lock (`REENTRANCY_LOCK`)
- Pull-payment pattern: funds are credited to balances, not pushed directly
- State updated before any external interaction (CEI pattern)

<!-- API_START -->

## Key Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address` | `Result<(), Error>` | — |
| `set_fee_config` | `env: Env, caller: Address, fee_receiver: Address, platform_fee_bps: u32` | `Result<(), Error>` | — |
| `get_fee_config` | `env: Env` | `Option<FeeConfig>` | — |
| `create_escrow` | `env: Env, order_id: u64, payer: Address, payee: Address, amount: i128, token: Address` | `Result<bool, Error>` | — |
| `mark_disputed` | `env: Env, caller: Address, order_id: u64` | `Result<(), Error>` | — |
| `approve_release` | `env: Env, order_id: u64, approver: Address` | `Result<(), Error>` | — |
| `release_escrow` | `env: Env, order_id: u64` | `Result<bool, Error>` | — |
| `refund_escrow` | `env: Env, order_id: u64, reason: String` | `Result<bool, Error>` | — |
| `get_escrow` | `env: Env, order_id: u64` | `Option<Escrow>` | — |
| `get_credit` | `env: Env, addr: Address` | `i128` | — |
| `withdraw` | `env: Env, caller: Address, token: Address, to: Address` | `Result<i128, Error>` | — |
| `get_total_volume` | `env: Env` | `i128` | — |
| `get_total_escrows` | `env: Env` | `u64` | — |
| `get_settled_rate` | `env: Env` | `u32` | — |
| `get_refund_rate` | `env: Env` | `u32` | — |
| `get_dispute_rate` | `env: Env` | `u32` | — |
| `get_active_escrows_count` | `env: Env` | `u64` | — |
| `get_stats_summary` | `env: Env` | `PlatformStats` | — |
| `get_platform_health_score` | `env: Env` | `u32` | — |
| `get_token_volume` | `env: Env, _token: Address` | `i128` | — |
| `get_donor_reputation` | `env: Env, _donor: Address` | `u32` | — |
| `get_daily_stats` | `env: Env, day_id: u64` | `Option<DailyStats>` | — |
| `export_summary` | `env: Env, format: String` | `ExportMetadata` | — |

## Types

### `enum EscrowStatus`

| Variant | Value | Description |
|---|---|---|
| `Pending` | 0 | — |
| `Active` | 1 | — |
| `Settled` | 2 | — |
| `Refunded` | 3 | — |
| `Disputed` | 4 | — |

### `struct Escrow`

| Field | Type | Description |
|---|---|---|
| `order_id` | `u64` | — |
| `payer` | `Address` | — |
| `payee` | `Address` | — |
| `amount` | `i128` | — |
| `token` | `Address` | — |
| `status` | `EscrowStatus` | — |
| `approvals` | `Vec<Address>` | — |
| `reason` | `String` | — |

### `struct PlatformStats`

| Field | Type | Description |
|---|---|---|
| `total_volume` | `i128` | — |
| `total_escrows` | `u64` | — |
| `settled_count` | `u64` | — |
| `refunded_count` | `u64` | — |
| `disputed_count` | `u64` | — |
| `active_count` | `u64` | — |

### `struct DailyStats`

| Field | Type | Description |
|---|---|---|
| `day_id` | `u64` | — |
| `volume` | `i128` | — |
| `count` | `u32` | — |

### `struct ExportMetadata`

| Field | Type | Description |
|---|---|---|
| `format` | `String` | — |
| `checksum` | `BytesN<32>` | — |
| `timestamp` | `u64` | — |

### `struct FeeConfig`

| Field | Type | Description |
|---|---|---|
| `platform_fee_bps` | `u32` | — |
| `fee_receiver` | `Address` | — |


## Error Codes

| Variant | Code | Description |
|---|---|---|
| `Unauthorized` | 100 | — |
| `NotAdmin` | 102 | — |
| `InsufficientApprovals` | 120 | — |
| `InvalidAmount` | 205 | — |
| `InvalidFeeBps` | 260 | — |
| `FeeNotSet` | 380 | — |
| `ReentrancyGuard` | 381 | — |
| `InvalidStateTransition` | 382 | — |
| `EscrowExists` | 480 | — |
| `EscrowNotFound` | 481 | — |
| `AlreadySettled` | 482 | — |
| `NoBasisToRefund` | 560 | — |
| `NoCredit` | 561 | — |
| `Overflow` | 562 | — |

<!-- API_END -->

## Escrow Status Flow

```
Pending → Active → Settled
       ↘ Disputed → Refunded
```
