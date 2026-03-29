# Data Migration System

## Overview
The Medical Records contract includes a built-in data migration system. This ensures that when the contract logic is upgraded (e.g., adding new fields to a struct), the existing data in storage is automatically updated to match the new format.

## How it Works
1.  **Versioning:** The contract tracks a `ContractVersion` in storage (default is 0).
2.  **Atomicity:** The `upgrade` function performs two actions in a single transaction:
    * Updates the WASM code.
    * Runs the `migrate_data` logic.
3.  **Safety:** If the migration logic fails (panics), the entire transaction reverts. The contract code remains on the old version, preventing data corruption.

## Operational Guide (For Admins)

To upgrade the contract, the Admin must perform the following:

1.  **Deploy** the new WASM file to the network to get its `wasm_hash`.
2.  **Call** the `upgrade` function on the *existing* contract:
    ```bash
    soroban contract invoke \
      --id <CONTRACT_ID> \
      --source <ADMIN_SECRET> \
      --network <NETWORK> \
      -- \
      upgrade \
      --new_wasm_hash <NEW_WASM_HASH>
    ```

## Developer Guide (Adding a New Migration)

When you make a "breaking change" to the data structure (e.g., V1 to V2):

1.  **Increment Version:**
    In `lib.rs`, update the constant:
    ```rust
    const CURRENT_CONTRACT_VERSION: u32 = 2; // Was 1
    ```

2.  **Add Migration Logic:**
    Inside `migrate_data`, add a specific handler for the new version gap:
    ```rust
    if current_version < 2 {
        // specific logic to transform data from V1 format to V2
        migrate_v1_to_v2(env);
    }
    ```

3.  **Test:**
    Add a test case in `tests/test_migration.rs` that explicitly sets up "Old Data" and verifies it transforms correctly into "New Data".