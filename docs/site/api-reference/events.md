# Events Reference

All contracts emit structured events. See [docs/EVENTS.md](../../EVENTS.md) for the full registry.

## Event Format

Events have a topic tuple and a data payload:

```rust
env.events().publish(
    (symbol_short!("CATEGORY"), symbol_short!("ACTION")),
    (field1, field2, ...),
);
```

## Appointment Booking Escrow Events

| Topic | Data | Description |
|-------|------|-------------|
| `APPT/BOOK` | `(id, patient, provider, amount, timestamp)` | Appointment booked |
| `APPT/CONF` | `(id, provider, timestamp)` | Appointment confirmed |
| `APPT/REFUND` | `(id, patient, amount, timestamp)` | Appointment refunded |
| `APPT/RELEASE` | `(id, provider, amount, timestamp)` | Funds released |
| `APPT/INIT` | `admin` | Contract initialized |

## Healthcare Payment Events

| Topic | Data | Description |
|-------|------|-------------|
| `CLAIM_PD` | `(claim_id, provider, amount)` | Claim paid |

## Escrow Events

| Topic | Data | Description |
|-------|------|-------------|
| `EscNew` | `(payer, payee, amount, token)` | Escrow created |
| `EscRel` | `(payee, amount, fee_receiver, fee, token)` | Escrow released |
| `Refunded` | `(payer, amount, token, reason)` | Escrow refunded |
| `EscDisput` | `()` | Escrow disputed |
| `Withdrawn` | `(to, amount, token)` | Balance withdrawn |

## Diagnostic Events

All key contracts emit diagnostic events for debugging:

| Topic | Data | Level | Description |
|-------|------|-------|-------------|
| `DIAG/ENTER` | `fn_name` | DEBUG | Function entered |
| `DIAG/EXIT` | `fn_name` | DEBUG | Function exited successfully |
| `DIAG/STATE` | `(id, old_status, new_status)` | INFO | State changed |
| `DIAG/VALFAIL` | `(fn_name, reason)` | WARN | Validation failed |
| `DIAG/AUTHFAIL` | `fn_name` | WARN | Authorization failed |
| `DIAG/ERR` | `(fn_name, error_code)` | ERROR | Error occurred |

## Filtering Events

```bash
# Filter diagnostic events from a transaction
soroban contract invoke ... | jq '.events[] | select(.topic[0] == "DIAG")'
```
