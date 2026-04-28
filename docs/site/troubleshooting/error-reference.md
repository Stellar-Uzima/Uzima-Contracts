# Error Reference

See [Error Codes](../api-reference/error-codes.md) for the full error code table.

## Debugging with Diagnostic Events

When a transaction fails, check the `DIAG/VALFAIL`, `DIAG/AUTHFAIL`, and `DIAG/ERR` events for context:

```bash
soroban tx fetch <TX_HASH> --network testnet \
  | jq '.result.events[] | select(.topic[0] == "DIAG")'
```

## Common Error Patterns

| Error | Likely Cause | Fix |
|-------|-------------|-----|
| `Unauthorized (100)` | Wrong signer | Use correct key |
| `InvalidAmount (205)` | Amount ≤ 0 | Pass positive amount |
| `ClaimNotFound` | Wrong claim ID | Verify claim exists |
| `InvalidStatus` | Wrong state | Check current status |
| `DoubleWithdrawal` | Already released | Check `funds_released` |
