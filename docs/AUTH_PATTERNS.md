# Soroban Auth Patterns

Standard authorization patterns for all Uzima contracts. Every public function that mutates state **must** follow one of these patterns.

---

## The Golden Rule

```
require_auth()  →  role/permission check  →  input validation  →  state change  →  emit event
```

Never skip `require_auth()`. Never put it after a state change.

---

## Pattern 1 — Admin-only function

```rust
pub fn admin_action(env: Env, admin: Address, value: u64) -> Result<(), Error> {
    // 1. Authenticate
    admin.require_auth();

    // 2. Verify role
    let stored_admin: Address = env.storage().instance()
        .get(&KEY_ADMIN)
        .ok_or(Error::NotInitialized)?;
    if admin != stored_admin {
        return Err(Error::Unauthorized);
    }

    // 3. Validate input
    if value == 0 {
        return Err(Error::InvalidInput);
    }

    // 4. Mutate state
    env.storage().instance().set(&KEY_VALUE, &value);

    // 5. Emit event
    env.events().publish((symbol_short!("set_val"),), (admin, value));
    Ok(())
}
```

## Pattern 2 — Owner-only function (per-resource ownership)

```rust
pub fn update_record(env: Env, owner: Address, record_id: u64, data: String) -> Result<(), Error> {
    // 1. Authenticate the claimed owner
    owner.require_auth();

    // 2. Load the record and verify ownership
    let record: Record = env.storage().persistent()
        .get(&record_id)
        .ok_or(Error::NotFound)?;
    if record.owner != owner {
        return Err(Error::Unauthorized);
    }

    // 3. Validate input
    if data.len() > MAX_DATA_LEN {
        return Err(Error::InputTooLong);
    }

    // 4. Mutate
    env.storage().persistent().set(&record_id, &Record { owner, data: data.clone() });

    // 5. Emit
    env.events().publish((symbol_short!("upd_rec"),), (owner, record_id));
    Ok(())
}
```

## Pattern 3 — Role-based access (RBAC)

```rust
pub fn write_record(env: Env, caller: Address, patient_id: Address, data: String) -> Result<(), Error> {
    // 1. Authenticate
    caller.require_auth();

    // 2. Check role — caller must be a registered doctor
    let role: Role = env.storage().persistent()
        .get(&(KEY_ROLE, caller.clone()))
        .unwrap_or(Role::None);
    if role != Role::Doctor {
        return Err(Error::Unauthorized);
    }

    // 3. Validate
    if data.len() > MAX_RECORD_LEN {
        return Err(Error::InputTooLong);
    }

    // 4. Mutate
    // ...

    // 5. Emit
    env.events().publish((symbol_short!("wr_rec"),), (caller, patient_id));
    Ok(())
}
```

## Pattern 4 — Patient self-service (caller == subject)

```rust
pub fn grant_access(env: Env, patient: Address, grantee: Address) -> Result<(), Error> {
    // 1. Authenticate — the patient must sign this transaction
    patient.require_auth();

    // 2. No separate role check needed; the patient IS the subject.
    //    Verify the patient record exists.
    if !env.storage().persistent().has(&(KEY_PATIENT, patient.clone())) {
        return Err(Error::NotFound);
    }

    // 3. Validate
    if patient == grantee {
        return Err(Error::InvalidInput); // can't grant access to yourself
    }

    // 4. Mutate
    env.storage().persistent().set(&(KEY_ACCESS, patient.clone(), grantee.clone()), &true);

    // 5. Emit
    env.events().publish((symbol_short!("grant"),), (patient, grantee));
    Ok(())
}
```

---

## Anti-patterns to avoid

| ❌ Wrong | ✅ Correct |
|---|---|
| Check role before `require_auth()` | Always call `require_auth()` first |
| Use `env.invoker()` for auth | Pass the address as a parameter and call `require_auth()` on it |
| Skip `require_auth()` for "read-only" functions that have side effects | Every state-mutating function needs auth |
| Rely on `Address` equality alone without `require_auth()` | `require_auth()` is the cryptographic proof; equality is just a label |

---

## Reference implementation

See `contracts/contract_template/src/lib.rs` for a complete working example of patterns 1 and 2.
