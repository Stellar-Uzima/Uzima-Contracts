# Common Issues

## Build Issues

**WASM target not found**
```bash
rustup target add wasm32-unknown-unknown
```

**Contract compilation fails**
```bash
make clean && make build
```

**Version mismatch**
Ensure you're using exactly `soroban-cli 21.7.7`:
```bash
cargo install --locked --version 21.7.7 soroban-cli
```

## Network Issues

**Local network won't start**
```bash
netstat -tulpn | grep :8000
# Kill existing process if needed
make start-local
```

**Testnet deployment fails — insufficient funds**
```bash
# Fund account via friendbot
curl "https://friendbot.stellar.org?addr=$(soroban config identity address default)"
```

## Contract Issues

**`NotInitialized` error**
Call `initialize()` before any other function.

**`Unauthorized` error**
Ensure the transaction is signed by the correct key. Check `require_auth()` requirements.

**`AlreadySettled` / `AppointmentAlreadyConfirmed`**
The operation has already been completed. Check the current status with `get_escrow()` or `get_appointment()`.

**`CircuitOpen` in healthcare_payment**
The circuit breaker is open. An admin must call `reset_circuit_breaker()`.

## See Also

- [docs/TROUBLESHOOTING section in README](../../../README.md)
- [Error Reference](error-reference.md)
