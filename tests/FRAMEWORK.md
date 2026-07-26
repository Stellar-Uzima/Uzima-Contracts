# Integration Test Framework

`tests/integration_framework` is the shared harness for all cross-contract
integration tests in Uzima Contracts.  It provides a `TestWorld` struct with
named contract registration, ledger helpers, and assertion utilities so that
every test file follows the same conventions.

---

## Table of Contents

1. [Why a shared framework?](#why-a-shared-framework)
2. [Quick start](#quick-start)
3. [API reference](#api-reference)
4. [Writing a new integration test](#writing-a-new-integration-test)
5. [Migrating an existing test](#migrating-an-existing-test)
6. [Running the tests](#running-the-tests)
7. [Design decisions](#design-decisions)
8. [Backward compatibility](#backward-compatibility)

---

## Why a shared framework?

Before this framework, each integration test file:

* Created its own `Env` differently (some called `Env::default()`, others used
  `Env::default().mock_all_auths_allowing_non_root_auth()`).
* Used raw `Address` variables with no human-readable names, making failures
  hard to diagnose.
* Duplicated ledger-advance helpers copy-pasted across files.
* Gave no standard way to assert that a cross-contract call failed with a
  *specific* Soroban error.

With 100+ interdependent contracts the drift compounds.  `TestWorld` solves
this in ~250 lines of library code.

---

## Quick start

Add the crate to your test file's imports (it is a workspace member, no
`Cargo.toml` changes needed for tests inside `tests/`):

```rust
use integration_framework::prelude::*;
```

Import the contracts you want to test with `soroban_sdk::contractimport!`:

```rust
mod my_contract {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/my_contract.wasm"
    );
}

mod dependency_contract {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/dependency_contract.wasm"
    );
}
```

Build a `TestWorld`, register contracts, create clients, write tests:

```rust
#[test]
fn test_cross_contract_flow() {
    let mut world = TestWorld::new();

    let my_addr  = world.register_contract("my_contract",       my_contract::WASM);
    let dep_addr = world.register_contract("dependency_contract", dependency_contract::WASM);

    let env = world.env();
    let my_client  = my_contract::Client::new(env, &my_addr);
    let dep_client = dependency_contract::Client::new(env, &dep_addr);

    my_client.initialize(&dep_addr);

    let user = world.new_account();
    let result = my_client.do_something(&user);
    assert_eq!(result, expected_value);
}
```

---

## API reference

### `TestWorld`

| Method | Description |
|--------|-------------|
| `TestWorld::new()` | Creates a fresh `Env` with all auths mocked. |
| `TestWorld::with_env(env)` | Uses a custom `Env` (e.g. with a ledger snapshot). |
| `world.env() -> &Env` | Borrow the underlying Soroban `Env`. |
| `world.register_contract(name, wasm) -> Address` | Register a contract from its WASM bytes; panics if `name` is already taken. |
| `world.register_contract_at(name, addr, wasm) -> Address` | Register at a specific address (for contracts that embed their own address). |
| `world.address_of(name) -> Address` | Look up a contract address by name; panics with a helpful message listing all known names. |
| `world.new_account() -> Address` | Generate a unique test account address. |
| `world.new_accounts(n) -> Vec<Address>` | Generate `n` unique accounts. |
| `world.advance_time(seconds)` | Advance the ledger `timestamp` by `seconds`. |
| `world.set_time(timestamp)` | Set the ledger `timestamp` to an absolute value. |
| `world.advance_sequence(n)` | Advance the ledger `sequence_number` by `n`. |

### `UnwrapTestResult` trait

Extends `Result<T, soroban_sdk::Error>` with:

| Method | Description |
|--------|-------------|
| `.expect_ok("contract", "fn")` | Unwrap `Ok` or panic with contract + function name in the message. |
| `.expect_err("contract", "fn")` | Assert `Err` or panic if the call unexpectedly succeeded. |

### `prelude`

`use integration_framework::prelude::*` imports: `TestWorld`,
`UnwrapTestResult`, and the most common `soroban_sdk` types
(`Address`, `BytesN`, `Env`, `Symbol`).

---

## Writing a new integration test

1. Create `tests/my_feature_integration_test.rs`.
2. Add `use integration_framework::prelude::*;` at the top.
3. Import each WASM with `contractimport!`.
4. Write a `setup()` helper that builds a `TestWorld`, registers all
   contracts, and returns typed clients.  Keep `setup()` in the same file;
   extract a shared `setup` to a `tests/common/` module only when two or more
   files need the exact same topology.
5. Write `#[test]` functions that call `setup()` and exercise the contracts.

### Conventions

* Name contracts by their crate name in snake_case (`"did_registry"`, not
  `"DID Registry"`).
* Keep each `#[test]` short and focused on one behaviour.
* Use `world.advance_time()` rather than raw `ledger.set()` for TTL/expiry
  tests.
* Prefer `client.try_foo(...)` + `assert!(result.is_err())` over catching
  panics.

---

## Migrating an existing test

Typical migration steps for an ad-hoc test file:

```diff
- use soroban_sdk::{Env, Address, testutils::Address as _};
+ use integration_framework::prelude::*;

  #[test]
  fn test_something() {
-     let env = Env::default();
-     env.mock_all_auths();
-     let contract_id = env.register_contract_wasm(None, my_contract::WASM);
-     let other_id    = env.register_contract_wasm(None, other::WASM);
-     let client      = my_contract::Client::new(&env, &contract_id);
-     let user        = Address::generate(&env);
+     let mut world  = TestWorld::new();
+     let my_addr    = world.register_contract("my_contract", my_contract::WASM);
+     let _other     = world.register_contract("other",       other::WASM);
+     let client     = my_contract::Client::new(world.env(), &my_addr);
+     let user       = world.new_account();
      // ... rest of test unchanged ...
  }
```

---

## Running the tests

```bash
# All integration tests in the workspace
cargo test --workspace -- integration

# A specific file
cargo test --test did_integration_test

# All tests (unit + integration)
cargo test --all
```

> **Note**: WASM artifacts must be built first:
> ```bash
> cargo build --release --target wasm32-unknown-unknown --workspace
> ```

---

## Design decisions

**Why not a macro?**  Macros are hard to document, debug, and autocomplete.
`TestWorld` is a plain struct — `rust-analyzer` shows all methods inline.

**Why name-indexed lookups?**  Raw `Address` values in test failure messages
look like `Address(GABC...123)`.  Named lookups mean a panic says
`"no contract named 'my_contract'. Registered: [did_registry, auth_verifier]"`,
which is immediately actionable.

**Why `mock_all_auths()` by default?**  Integration tests focus on cross-
contract *logic*, not auth mechanics (there are dedicated auth tests for that).
You can always re-enable strict auth per test:

```rust
// Disable the mock for a specific test block
world.env().set_auths(&[]);
```

**Why no `invoke(name, fn, args)` helper?**  The Soroban SDK's `contractimport!`
macro generates a fully typed client struct (`did_registry::Client`).  A
dynamic `invoke(name, "register", ...)` would throw away that type safety and
lose compile-time argument checking.  Named address lookups via
`world.address_of("did_registry")` give the convenience of name-based
addressing while keeping typed clients.

---

## Backward compatibility

All pre-existing integration tests that do not use this framework continue to
compile and pass — the framework adds no mandatory dependency.  For files that
*have* been migrated, the diff is mechanical (see [Migrating an existing
test](#migrating-an-existing-test)) and the test names, assertions, and
coverage are unchanged.

The `integration_framework` crate re-exports `soroban_sdk` so that callers
can write `use integration_framework::soroban_sdk::...` and avoid a direct
`soroban-sdk` dep in test binaries that only interact via the framework.