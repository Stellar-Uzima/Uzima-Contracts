# Contributing

## Architecture Decisions

Governance architecture decisions are tracked in
[`docs/adr/README.md`](docs/adr/README.md). Before changing governance
contracts, shared governance helpers, ZKP verification, or workspace membership
for excluded contracts, check the ADR index and add or update an ADR when the
change introduces a durable architectural decision.

## Event Topic Naming Convention

All contract event topics **must** use `snake_case` naming. This applies to both
the primary topic (first symbol) and any subtopics.

### Do
```rust
env.events().publish((symbol_short!("payment_processed"),), data);
env.events().publish((Symbol::new(&env, "record_created"),), data);
```

### Don't
```rust
env.events().publish((symbol_short!("PaymentProcessed"),), data);  // PascalCase
env.events().publish((symbol_short!("PAYMENT"),), data);           // UPPER_CASE
env.events().publish((Symbol::new(&env, "recordCreated"),), data); // camelCase
```

### Rationale
Consistent `snake_case` event naming ensures that off-chain indexers and
monitoring tools can reliably pattern-match event topics without case
sensitivity issues.
