# Running Tests

## All Tests

```bash
cargo test --all
# or
make test
```

## Specific Contract Tests

```bash
cargo test -p appointment_booking_escrow
cargo test -p healthcare_payment
cargo test -p medical_records
```

## Test Categories

```bash
make test-unit          # Unit tests only
make test-integration   # Integration tests only
```

## Test with Logs

```bash
RUST_LOG=debug cargo test --all -- --nocapture
```

## Coverage Report

```bash
./scripts/coverage_report.sh
```

## CI Checks (run locally before pushing)

```bash
cargo fmt --all --check   # Formatting
cargo clippy --all        # Linting
cargo test --all          # Tests
```
