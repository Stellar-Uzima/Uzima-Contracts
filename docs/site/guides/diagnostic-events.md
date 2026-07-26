# Diagnostic Events

Diagnostic events provide structured observability for debugging contract execution without requiring a debugger.

## Event Categories

| Category | Symbol | Level | When Emitted |
|----------|--------|-------|-------------|
| Function Enter | `DIAG/ENTER` | DEBUG | Start of key functions |
| Function Exit | `DIAG/EXIT` | DEBUG | Successful function completion |
| State Change | `DIAG/STATE` | INFO | Status transitions |
| Validation Failure | `DIAG/VALFAIL` | WARN | Input validation fails |
| Auth Failure | `DIAG/AUTHFAIL` | WARN | Authorization check fails |
| Error | `DIAG/ERR` | ERROR | Unrecoverable error |

## Supported Contracts

Diagnostic events are emitted by:
- `appointment_booking_escrow` — `book_appointment`, `confirm_appointment`, `refund_appointment`
- `healthcare_payment` — `process_payment`

## Reading Diagnostic Events

```bash
# Invoke and capture events
soroban contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- confirm_appointment \
  --provider $PROVIDER \
  --appointment_id 42

# Parse DIAG events from transaction result
soroban tx fetch <TX_HASH> --network testnet \
  | jq '.result.events[] | select(.topic[0] == "DIAG")'
```

## Example Output

```json
{"topic": ["DIAG", "ENTER"], "data": "confirm_appointment"}
{"topic": ["DIAG", "STATE"], "data": [42, 0, 3]}
{"topic": ["DIAG", "EXIT"],  "data": "confirm_appointment"}
```

## Production Considerations

Diagnostic events add a small amount of ledger cost. In production, you may want to filter them out at the indexer level rather than removing them from contracts, as they are invaluable for incident investigation.
