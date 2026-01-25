# Smart Contract Data Migration Guide

## Overview
This document outlines the procedures for upgrading the `MedicalRecords` contract and migrating its data storage. The system uses an **atomic upgrade pattern**, ensuring that code updates and data transformations happen in a single transaction. If any part of the migration fails, the entire upgrade rolls back, preventing data corruption.

## Versioning System
The contract tracks a `ProtocolVersion` in its persistent storage.
- **Current Code Version:** Defined by `const CONTRACT_VERSION` in `lib.rs`.
- **Storage Version:** Stored in `DataKey::ProtocolVersion`.

When an upgrade is triggered, the contract checks:
1. Is the caller the Admin?
2. Is the stored version less than the new code version?

## How to Perform an Upgrade

### 1. Preparation
1.  **Modify Data Structures:** If you change a struct (e.g., `MedicalRecord`), update `CONTRACT_VERSION` in `lib.rs` (increment by 1).
2.  **Write Migration Logic:** Implement a specific function (e.g., `migrate_v1_to_v2`) inside `migrate_data()` to handle the data transformation.
3.  **Compile WASM:** Build the optimized WASM file.
    ```bash
    soroban contract build --release
    ```
4.  **Calculate WASM Hash:**
    ```bash
    soroban contract install --wasm target/wasm32-unknown-unknown/release/medical_records.wasm
    # Save the output hash (e.g., 7a0b...)
    ```

### 2. Execution (Admin Only)
Call the `upgrade` function using the Soroban CLI or a script.

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source <ADMIN_SECRET_KEY> \
  --network testnet \
  -- \
  upgrade \
  --new_wasm_hash <NEW_WASM_HASH>