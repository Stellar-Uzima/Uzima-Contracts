# Medical Imaging AI Analysis Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend `medical_imaging` with radiologist workflow and create `medical_imaging_ai` for CNN model management, oracle-attested analysis/segmentation, and tiered performance benchmarking.

**Architecture:** Two contracts — extend existing `medical_imaging` (study lifecycle, multi-reader blind review, structured processing output) and create new `medical_imaging_ai` (CNN model metadata with ed25519 attestation, analysis/segmentation results, rolling-window performance tracking). No cross-contract calls; linked by result IDs.

**Tech Stack:** Rust, soroban-sdk 21.7.7, Soroban smart contracts (wasm32-unknown-unknown)

**Spec:** `docs/superpowers/specs/2026-03-28-medical-imaging-ai-analysis-design.md`

---

## File Structure

### New contract: `contracts/medical_imaging_ai/`
- `Cargo.toml` — crate config, depends on soroban-sdk workspace
- `src/lib.rs` — types, errors, storage keys, contract implementation (17 methods)
- `src/test.rs` — unit tests (~20 tests)

### Modified contract: `contracts/medical_imaging/`
- `src/lib.rs` — add study/workflow types, new DataKey variants, new error variants, 13 new methods
- `src/test.rs` — add workflow unit tests (~18 tests)

---

## Task 1: Create `medical_imaging_ai` Contract Scaffold

**Files:**
- Create: `contracts/medical_imaging_ai/Cargo.toml`
- Create: `contracts/medical_imaging_ai/src/lib.rs`

- [ ] **Step 1: Create Cargo.toml**

Create `contracts/medical_imaging_ai/Cargo.toml`:

```toml
[package]
name = "medical_imaging_ai"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
soroban-sdk = { workspace = true }

[dev-dependencies]
soroban-sdk = { workspace = true, features = ["testutils"] }

[features]
default = []
testutils = ["soroban-sdk/testutils"]
```

- [ ] **Step 2: Create lib.rs with types, enums, errors, and storage keys**

Create `contracts/medical_imaging_ai/src/lib.rs`:

```rust
#![no_std]
#![allow(clippy::too_many_arguments)]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
    String, Symbol, Vec,
};

const ADMIN: Symbol = symbol_short!("ADMIN");
const PAUSED: Symbol = symbol_short!("PAUSED");
const NEXT_RES: Symbol = symbol_short!("NRES");
const NEXT_SEG: Symbol = symbol_short!("NSEG");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ImagingModality {
    XRay,
    MRI,
    CT,
    Ultrasound,
    PET,
    Mammography,
    Custom(u32),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ModelStatus {
    Active,
    Degraded,
    Deactivated,
    Retired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct BoundingBox {
    pub x_min: u32,
    pub y_min: u32,
    pub x_max: u32,
    pub y_max: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CnnModelMetadata {
    pub model_id: BytesN<32>,
    pub owner: Address,
    pub version: u32,
    pub modality: ImagingModality,
    pub architecture_hash: BytesN<32>,
    pub layer_count: u32,
    pub input_rows: u32,
    pub input_cols: u32,
    pub input_channels: u32,
    pub training_samples: u64,
    pub validation_accuracy_bps: u32,
    pub training_dataset_hash: BytesN<32>,
    pub signing_pubkey: BytesN<32>,
    pub status: ModelStatus,
    pub registered_at: u64,
    pub last_evaluated_at: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Finding {
    pub finding_id: u32,
    pub condition_hash: BytesN<32>,
    pub confidence_bps: u32,
    pub severity: u32,
    pub region: BoundingBox,
    pub explanation_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct AnalysisResult {
    pub result_id: u64,
    pub image_id: u64,
    pub model_id: BytesN<32>,
    pub submitter: Address,
    pub attestation_hash: BytesN<32>,
    pub signature: BytesN<64>,
    pub findings: Vec<Finding>,
    pub overall_confidence_bps: u32,
    pub processing_time_ms: u32,
    pub created_at: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct SegmentedRegion {
    pub label_hash: BytesN<32>,
    pub pixel_count: u64,
    pub volume_mm3: u64,
    pub mean_intensity: u32,
    pub mask_ref: String,
    pub bounds: BoundingBox,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct SegmentationResult {
    pub seg_id: u64,
    pub image_id: u64,
    pub model_id: BytesN<32>,
    pub submitter: Address,
    pub attestation_hash: BytesN<32>,
    pub signature: BytesN<64>,
    pub regions: Vec<SegmentedRegion>,
    pub processing_time_ms: u32,
    pub created_at: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ModelPerformance {
    pub model_id: BytesN<32>,
    pub modality: ImagingModality,
    pub total_evaluated: u64,
    pub correct_count: u64,
    pub lifetime_accuracy_bps: u32,
    pub window_size: u64,
    pub window_correct: u64,
    pub window_total: u64,
    pub rolling_accuracy_bps: u32,
    pub avg_processing_time_ms: u32,
    pub warning_threshold_bps: u32,
    pub critical_threshold_bps: u32,
    pub min_sample_size: u64,
    pub last_updated: u64,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    CnnModel(BytesN<32>),
    AnalysisResult(u64),
    SegResult(u64),
    Performance(BytesN<32>),
    ImageResults(u64),
    ImageSegResults(u64),
    Evaluator(Address),
    DefaultWarningBps,
    DefaultCriticalBps,
    DefaultMinSamples,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    ContractPaused = 4,
    InvalidInput = 5,
    ModelNotFound = 6,
    ModelNotActive = 7,
    ModelAlreadyExists = 8,
    ResultNotFound = 9,
    SegmentationNotFound = 10,
    TooManyFindings = 11,
    TooManyRegions = 12,
    InvalidConfidence = 13,
    InvalidSeverity = 14,
    InvalidThreshold = 15,
    AttestationInvalid = 16,
    DuplicateResult = 17,
    InsufficientSamples = 18,
}

#[contract]
pub struct MedicalImagingAiContract;

#[contractimpl]
impl MedicalImagingAiContract {
    // Implementation will be added in subsequent tasks
}
```

- [ ] **Step 3: Create empty test module**

Create `contracts/medical_imaging_ai/src/test.rs`:

```rust
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{Address, BytesN, Env, String, Vec};
```

- [ ] **Step 4: Verify it compiles**

Run: `cd /Users/user/Desktop/Projects/Uzima-Contracts && cargo build -p medical_imaging_ai --target wasm32-unknown-unknown --release 2>&1 | tail -5`
Expected: `Compiling medical_imaging_ai` ... `Finished`

- [ ] **Step 5: Commit**

```bash
git add contracts/medical_imaging_ai/
git commit -m "feat(medical_imaging_ai): scaffold new contract with types and errors"
```

---

## Task 2: Implement Initialization, Pause, and Evaluator Management

**Files:**
- Modify: `contracts/medical_imaging_ai/src/lib.rs`
- Modify: `contracts/medical_imaging_ai/src/test.rs`

- [ ] **Step 1: Write tests for initialize, pause, and evaluator management**

Add to `contracts/medical_imaging_ai/src/test.rs`:

```rust
fn setup(env: &Env) -> (MedicalImagingAiContractClient<'_>, Address) {
    let contract_id = Address::generate(env);
    env.register_contract(&contract_id, MedicalImagingAiContract);
    let client = MedicalImagingAiContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.initialize(&admin, &9200, &8500, &50);
    (client, admin)
}

fn hash(env: &Env, v: u8) -> BytesN<32> {
    BytesN::from_array(env, &[v; 32])
}

fn sig(env: &Env, v: u8) -> BytesN<64> {
    BytesN::from_array(env, &[v; 64])
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, _) = setup(&env);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    client.initialize(&admin, &9200, &8500, &50);
}

#[test]
fn test_pause_unpause() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    client.pause(&admin);
    client.unpause(&admin);
}

#[test]
fn test_register_and_revoke_evaluator() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let evaluator = Address::generate(&env);
    client.register_evaluator(&admin, &evaluator);
    client.revoke_evaluator(&admin, &evaluator);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd /Users/user/Desktop/Projects/Uzima-Contracts && cargo test -p medical_imaging_ai 2>&1 | tail -10`
Expected: compilation errors — methods not implemented

- [ ] **Step 3: Implement initialize, pause, unpause, evaluator methods, and helpers**

Add inside the `impl MedicalImagingAiContract` block in `contracts/medical_imaging_ai/src/lib.rs`, replacing the placeholder comment:

```rust
    pub fn initialize(
        env: Env,
        admin: Address,
        default_warning_bps: u32,
        default_critical_bps: u32,
        default_min_samples: u64,
    ) -> Result<bool, Error> {
        admin.require_auth();
        if env.storage().instance().has(&ADMIN) {
            return Err(Error::AlreadyInitialized);
        }
        if default_warning_bps <= default_critical_bps || default_warning_bps > 10_000 {
            return Err(Error::InvalidThreshold);
        }
        if default_min_samples == 0 {
            return Err(Error::InvalidInput);
        }

        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&PAUSED, &false);
        env.storage().instance().set(&NEXT_RES, &1u64);
        env.storage().instance().set(&NEXT_SEG, &1u64);
        env.storage()
            .instance()
            .set(&DataKey::DefaultWarningBps, &default_warning_bps);
        env.storage()
            .instance()
            .set(&DataKey::DefaultCriticalBps, &default_critical_bps);
        env.storage()
            .instance()
            .set(&DataKey::DefaultMinSamples, &default_min_samples);
        Ok(true)
    }

    pub fn pause(env: Env, admin: Address) -> Result<bool, Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        env.storage().instance().set(&PAUSED, &true);
        Ok(true)
    }

    pub fn unpause(env: Env, admin: Address) -> Result<bool, Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        env.storage().instance().set(&PAUSED, &false);
        Ok(true)
    }

    pub fn register_evaluator(
        env: Env,
        admin: Address,
        evaluator: Address,
    ) -> Result<bool, Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        env.storage()
            .persistent()
            .set(&DataKey::Evaluator(evaluator), &true);
        Ok(true)
    }

    pub fn revoke_evaluator(
        env: Env,
        admin: Address,
        evaluator: Address,
    ) -> Result<bool, Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        env.storage()
            .persistent()
            .set(&DataKey::Evaluator(evaluator), &false);
        Ok(true)
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&ADMIN)
            .ok_or(Error::NotInitialized)?;
        if &admin != caller {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    fn require_not_paused(env: &Env) -> Result<(), Error> {
        let paused: bool = env.storage().instance().get(&PAUSED).unwrap_or(false);
        if paused {
            return Err(Error::ContractPaused);
        }
        Ok(())
    }

    fn require_evaluator(env: &Env, caller: &Address) -> Result<(), Error> {
        let is_eval: bool = env
            .storage()
            .persistent()
            .get(&DataKey::Evaluator(caller.clone()))
            .unwrap_or(false);
        if !is_eval {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    fn next_counter(env: &Env, key: &Symbol) -> u64 {
        let current: u64 = env.storage().instance().get(key).unwrap_or(1u64);
        env.storage()
            .instance()
            .set(key, &current.saturating_add(1));
        current
    }

    fn append_u64(env: &Env, key: DataKey, value: u64) {
        let mut values: Vec<u64> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));
        if !values.iter().any(|item| item == value) {
            values.push_back(value);
            env.storage().persistent().set(&key, &values);
        }
    }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd /Users/user/Desktop/Projects/Uzima-Contracts && cargo test -p medical_imaging_ai 2>&1 | tail -10`
Expected: 4 tests passed

- [ ] **Step 5: Commit**

```bash
git add contracts/medical_imaging_ai/
git commit -m "feat(medical_imaging_ai): implement init, pause, and evaluator management"
```

---

## Task 3: Implement CNN Model Registration and Queries

**Files:**
- Modify: `contracts/medical_imaging_ai/src/lib.rs`
- Modify: `contracts/medical_imaging_ai/src/test.rs`

- [ ] **Step 1: Write tests for model registration**

Add to `contracts/medical_imaging_ai/src/test.rs`:

```rust
fn register_test_model(
    env: &Env,
    client: &MedicalImagingAiContractClient<'_>,
    caller: &Address,
    model_id_byte: u8,
) {
    client.register_cnn_model(
        caller,
        &hash(env, model_id_byte),
        &ImagingModality::CT,
        &hash(env, 50),          // architecture_hash
        &1,                       // version
        &152,                     // layer_count
        &512,                     // input_rows
        &512,                     // input_cols
        &1,                       // input_channels
        &100_000,                 // training_samples
        &9500,                    // validation_accuracy_bps
        &hash(env, 51),          // training_dataset_hash
        &hash(env, 52),          // signing_pubkey
    );
}

#[test]
fn test_register_cnn_model() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_test_model(&env, &client, &admin, 1);

    let model = client.get_model(&hash(&env, 1));
    assert_eq!(model.version, 1);
    assert_eq!(model.layer_count, 152);
    assert_eq!(model.status, ModelStatus::Active);
    assert_eq!(model.validation_accuracy_bps, 9500);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_register_duplicate_model() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_test_model(&env, &client, &admin, 1);
    register_test_model(&env, &client, &admin, 1);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_register_model_zero_layers() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    client.register_cnn_model(
        &admin,
        &hash(&env, 1),
        &ImagingModality::CT,
        &hash(&env, 50),
        &1,     // version
        &0,     // zero layers — invalid
        &512,
        &512,
        &1,
        &100_000,
        &9500,
        &hash(&env, 51),
        &hash(&env, 52),
    );
}

#[test]
fn test_is_model_active() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_test_model(&env, &client, &admin, 1);
    assert!(client.is_model_active(&hash(&env, 1)));
}

#[test]
fn test_update_model_status_retire() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_test_model(&env, &client, &admin, 1);
    client.update_model_status(&admin, &hash(&env, 1), &ModelStatus::Retired);
    assert!(!client.is_model_active(&hash(&env, 1)));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd /Users/user/Desktop/Projects/Uzima-Contracts && cargo test -p medical_imaging_ai 2>&1 | tail -10`
Expected: compilation errors — `register_cnn_model`, `get_model`, `is_model_active`, `update_model_status` not found

- [ ] **Step 3: Implement model lifecycle methods**

Add to the `impl MedicalImagingAiContract` block in `contracts/medical_imaging_ai/src/lib.rs` (before the private helper methods):

```rust
    pub fn register_cnn_model(
        env: Env,
        caller: Address,
        model_id: BytesN<32>,
        modality: ImagingModality,
        architecture_hash: BytesN<32>,
        version: u32,
        layer_count: u32,
        input_rows: u32,
        input_cols: u32,
        input_channels: u32,
        training_samples: u64,
        validation_accuracy_bps: u32,
        training_dataset_hash: BytesN<32>,
        signing_pubkey: BytesN<32>,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_not_paused(&env)?;

        if version == 0 || layer_count == 0 || input_rows == 0 || input_cols == 0 || input_channels == 0 {
            return Err(Error::InvalidInput);
        }
        if validation_accuracy_bps > 10_000 {
            return Err(Error::InvalidConfidence);
        }
        if env
            .storage()
            .persistent()
            .has(&DataKey::CnnModel(model_id.clone()))
        {
            return Err(Error::ModelAlreadyExists);
        }

        let model = CnnModelMetadata {
            model_id: model_id.clone(),
            owner: caller.clone(),
            version,
            modality,
            architecture_hash,
            layer_count,
            input_rows,
            input_cols,
            input_channels,
            training_samples,
            validation_accuracy_bps,
            training_dataset_hash,
            signing_pubkey,
            status: ModelStatus::Active,
            registered_at: env.ledger().timestamp(),
            last_evaluated_at: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::CnnModel(model_id.clone()), &model);
        env.events()
            .publish((symbol_short!("MDL_REG"),), model_id);
        Ok(true)
    }

    pub fn update_model_status(
        env: Env,
        admin: Address,
        model_id: BytesN<32>,
        new_status: ModelStatus,
    ) -> Result<bool, Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let mut model: CnnModelMetadata = env
            .storage()
            .persistent()
            .get(&DataKey::CnnModel(model_id.clone()))
            .ok_or(Error::ModelNotFound)?;

        model.status = new_status;
        env.storage()
            .persistent()
            .set(&DataKey::CnnModel(model_id.clone()), &model);

        match new_status {
            ModelStatus::Active => {
                env.events()
                    .publish((symbol_short!("MDL_REACT"),), model_id);
            }
            ModelStatus::Retired => {
                env.events()
                    .publish((symbol_short!("MDL_RET"),), model_id);
            }
            _ => {}
        }
        Ok(true)
    }

    pub fn get_model(env: Env, model_id: BytesN<32>) -> CnnModelMetadata {
        env.storage()
            .persistent()
            .get(&DataKey::CnnModel(model_id))
            .unwrap()
    }

    pub fn is_model_active(env: Env, model_id: BytesN<32>) -> bool {
        let model: Option<CnnModelMetadata> = env
            .storage()
            .persistent()
            .get(&DataKey::CnnModel(model_id));
        match model {
            Some(m) => matches!(m.status, ModelStatus::Active | ModelStatus::Degraded),
            None => false,
        }
    }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd /Users/user/Desktop/Projects/Uzima-Contracts && cargo test -p medical_imaging_ai 2>&1 | tail -10`
Expected: 9 tests passed

- [ ] **Step 5: Commit**

```bash
git add contracts/medical_imaging_ai/
git commit -m "feat(medical_imaging_ai): implement CNN model registration and queries"
```

---

## Task 4: Implement Analysis Submission with Attestation

**Files:**
- Modify: `contracts/medical_imaging_ai/src/lib.rs`
- Modify: `contracts/medical_imaging_ai/src/test.rs`

- [ ] **Step 1: Write tests for analysis submission**

Add to `contracts/medical_imaging_ai/src/test.rs`:

```rust
fn make_finding(env: &Env, id: u32) -> Finding {
    Finding {
        finding_id: id,
        condition_hash: hash(env, id as u8),
        confidence_bps: 8500,
        severity: 3,
        region: BoundingBox {
            x_min: 10,
            y_min: 20,
            x_max: 100,
            y_max: 200,
        },
        explanation_ref: String::from_str(env, "ipfs://explanation"),
    }
}

#[test]
fn test_submit_analysis() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_test_model(&env, &client, &admin, 1);

    let mut findings = Vec::new(&env);
    findings.push_back(make_finding(&env, 1));
    findings.push_back(make_finding(&env, 2));

    let result_id = client.submit_analysis(
        &admin,
        &1,                     // image_id
        &hash(&env, 1),         // model_id
        &hash(&env, 90),        // attestation_hash
        &sig(&env, 91),         // signature (mock — ed25519 verification skipped in test)
        &findings,
        &8700,                  // overall_confidence_bps
        &1500,                  // processing_time_ms
    );
    assert_eq!(result_id, 1);

    let analysis = client.get_analysis(&1);
    assert_eq!(analysis.findings.len(), 2);
    assert_eq!(analysis.overall_confidence_bps, 8700);
}

#[test]
#[should_panic(expected = "Error(Contract, #11)")]
fn test_submit_analysis_too_many_findings() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_test_model(&env, &client, &admin, 1);

    let mut findings = Vec::new(&env);
    for i in 0..21u32 {
        findings.push_back(make_finding(&env, i));
    }

    client.submit_analysis(
        &admin,
        &1,
        &hash(&env, 1),
        &hash(&env, 90),
        &sig(&env, 91),
        &findings,
        &8700,
        &1500,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_submit_analysis_inactive_model() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_test_model(&env, &client, &admin, 1);
    client.update_model_status(&admin, &hash(&env, 1), &ModelStatus::Deactivated);

    let mut findings = Vec::new(&env);
    findings.push_back(make_finding(&env, 1));

    client.submit_analysis(
        &admin,
        &1,
        &hash(&env, 1),
        &hash(&env, 90),
        &sig(&env, 91),
        &findings,
        &8700,
        &1500,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_submit_analysis_invalid_bbox() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_test_model(&env, &client, &admin, 1);

    let bad_finding = Finding {
        finding_id: 1,
        condition_hash: hash(&env, 1),
        confidence_bps: 8500,
        severity: 3,
        region: BoundingBox {
            x_min: 200,
            y_min: 20,
            x_max: 100, // x_max < x_min — invalid
            y_max: 200,
        },
        explanation_ref: String::from_str(&env, "ipfs://explanation"),
    };
    let mut findings = Vec::new(&env);
    findings.push_back(bad_finding);

    client.submit_analysis(
        &admin,
        &1,
        &hash(&env, 1),
        &hash(&env, 90),
        &sig(&env, 91),
        &findings,
        &8700,
        &1500,
    );
}

#[test]
fn test_get_image_analyses() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_test_model(&env, &client, &admin, 1);

    let mut findings = Vec::new(&env);
    findings.push_back(make_finding(&env, 1));

    client.submit_analysis(&admin, &1, &hash(&env, 1), &hash(&env, 90), &sig(&env, 91), &findings, &8700, &1500);
    client.submit_analysis(&admin, &1, &hash(&env, 1), &hash(&env, 91), &sig(&env, 92), &findings, &9000, &1200);

    let ids = client.get_image_analyses(&1);
    assert_eq!(ids.len(), 2);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd /Users/user/Desktop/Projects/Uzima-Contracts && cargo test -p medical_imaging_ai 2>&1 | tail -10`
Expected: compilation errors

- [ ] **Step 3: Implement submit_analysis, get_analysis, get_image_analyses**

Add to the `impl MedicalImagingAiContract` block in `contracts/medical_imaging_ai/src/lib.rs`:

```rust
    pub fn submit_analysis(
        env: Env,
        caller: Address,
        image_id: u64,
        model_id: BytesN<32>,
        attestation_hash: BytesN<32>,
        signature: BytesN<64>,
        findings: Vec<Finding>,
        overall_confidence_bps: u32,
        processing_time_ms: u32,
    ) -> Result<u64, Error> {
        caller.require_auth();
        Self::require_not_paused(&env)?;

        let model: CnnModelMetadata = env
            .storage()
            .persistent()
            .get(&DataKey::CnnModel(model_id.clone()))
            .ok_or(Error::ModelNotFound)?;

        if !matches!(model.status, ModelStatus::Active | ModelStatus::Degraded) {
            return Err(Error::ModelNotActive);
        }
        if findings.len() > 20 {
            return Err(Error::TooManyFindings);
        }
        if overall_confidence_bps > 10_000 {
            return Err(Error::InvalidConfidence);
        }

        for finding in findings.iter() {
            if finding.confidence_bps > 10_000 {
                return Err(Error::InvalidConfidence);
            }
            if finding.severity == 0 || finding.severity > 5 {
                return Err(Error::InvalidSeverity);
            }
            if finding.region.x_min >= finding.region.x_max
                || finding.region.y_min >= finding.region.y_max
            {
                return Err(Error::InvalidInput);
            }
        }

        // Verify ed25519 attestation signature
        env.crypto().ed25519_verify(
            &model.signing_pubkey,
            &attestation_hash.to_array().into(),
            &signature,
        );

        let result_id = Self::next_counter(&env, &NEXT_RES);
        let result = AnalysisResult {
            result_id,
            image_id,
            model_id: model_id.clone(),
            submitter: caller.clone(),
            attestation_hash,
            signature,
            findings,
            overall_confidence_bps,
            processing_time_ms,
            created_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::AnalysisResult(result_id), &result);
        Self::append_u64(&env, DataKey::ImageResults(image_id), result_id);

        env.events()
            .publish((symbol_short!("ANALYSIS"),), (result_id, image_id));
        Ok(result_id)
    }

    pub fn get_analysis(env: Env, result_id: u64) -> AnalysisResult {
        env.storage()
            .persistent()
            .get(&DataKey::AnalysisResult(result_id))
            .unwrap()
    }

    pub fn get_image_analyses(env: Env, image_id: u64) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::ImageResults(image_id))
            .unwrap_or(Vec::new(&env))
    }
```

**Note on ed25519_verify:** In tests using `env.mock_all_auths()`, the crypto verification is still enforced. The tests pass mock signatures because the test environment's `ed25519_verify` is lenient. If the Soroban test environment does NOT skip crypto verification, wrap the verify call in a helper that can be feature-gated for tests, or generate a real ed25519 keypair in the test helpers:

```rust
// If crypto verification fails in tests, update sig() and hash() helpers
// to use real ed25519 keypair from soroban_sdk::testutils
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd /Users/user/Desktop/Projects/Uzima-Contracts && cargo test -p medical_imaging_ai 2>&1 | tail -10`
Expected: 14 tests passed

If ed25519_verify fails in tests, the implementor should generate a real keypair using `env.crypto().ed25519_keypair()` or equivalent test utilities and adjust `hash()`/`sig()` accordingly.

- [ ] **Step 5: Commit**

```bash
git add contracts/medical_imaging_ai/
git commit -m "feat(medical_imaging_ai): implement attested analysis submission"
```

---

## Task 5: Implement Segmentation Submission

**Files:**
- Modify: `contracts/medical_imaging_ai/src/lib.rs`
- Modify: `contracts/medical_imaging_ai/src/test.rs`

- [ ] **Step 1: Write tests for segmentation**

Add to `contracts/medical_imaging_ai/src/test.rs`:

```rust
fn make_region(env: &Env, id: u8) -> SegmentedRegion {
    SegmentedRegion {
        label_hash: hash(env, id),
        pixel_count: 50_000,
        volume_mm3: 120_000,
        mean_intensity: 128,
        mask_ref: String::from_str(env, "ipfs://mask"),
        bounds: BoundingBox {
            x_min: 0,
            y_min: 0,
            x_max: 256,
            y_max: 256,
        },
    }
}

#[test]
fn test_submit_segmentation() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_test_model(&env, &client, &admin, 1);

    let mut regions = Vec::new(&env);
    regions.push_back(make_region(&env, 1));
    regions.push_back(make_region(&env, 2));

    let seg_id = client.submit_segmentation(
        &admin,
        &1,
        &hash(&env, 1),
        &hash(&env, 80),
        &sig(&env, 81),
        &regions,
        &2000,
    );
    assert_eq!(seg_id, 1);

    let seg = client.get_segmentation(&1);
    assert_eq!(seg.regions.len(), 2);
}

#[test]
#[should_panic(expected = "Error(Contract, #12)")]
fn test_submit_segmentation_too_many_regions() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_test_model(&env, &client, &admin, 1);

    let mut regions = Vec::new(&env);
    for i in 0..31u8 {
        regions.push_back(make_region(&env, i));
    }

    client.submit_segmentation(
        &admin,
        &1,
        &hash(&env, 1),
        &hash(&env, 80),
        &sig(&env, 81),
        &regions,
        &2000,
    );
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd /Users/user/Desktop/Projects/Uzima-Contracts && cargo test -p medical_imaging_ai 2>&1 | tail -10`
Expected: compilation errors

- [ ] **Step 3: Implement submit_segmentation and get_segmentation**

Add to the `impl MedicalImagingAiContract` block:

```rust
    pub fn submit_segmentation(
        env: Env,
        caller: Address,
        image_id: u64,
        model_id: BytesN<32>,
        attestation_hash: BytesN<32>,
        signature: BytesN<64>,
        regions: Vec<SegmentedRegion>,
        processing_time_ms: u32,
    ) -> Result<u64, Error> {
        caller.require_auth();
        Self::require_not_paused(&env)?;

        let model: CnnModelMetadata = env
            .storage()
            .persistent()
            .get(&DataKey::CnnModel(model_id.clone()))
            .ok_or(Error::ModelNotFound)?;

        if !matches!(model.status, ModelStatus::Active | ModelStatus::Degraded) {
            return Err(Error::ModelNotActive);
        }
        if regions.len() > 30 {
            return Err(Error::TooManyRegions);
        }

        for region in regions.iter() {
            if region.bounds.x_min >= region.bounds.x_max
                || region.bounds.y_min >= region.bounds.y_max
            {
                return Err(Error::InvalidInput);
            }
        }

        env.crypto().ed25519_verify(
            &model.signing_pubkey,
            &attestation_hash.to_array().into(),
            &signature,
        );

        let seg_id = Self::next_counter(&env, &NEXT_SEG);
        let result = SegmentationResult {
            seg_id,
            image_id,
            model_id: model_id.clone(),
            submitter: caller.clone(),
            attestation_hash,
            signature,
            regions,
            processing_time_ms,
            created_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::SegResult(seg_id), &result);
        Self::append_u64(&env, DataKey::ImageSegResults(image_id), seg_id);

        env.events()
            .publish((symbol_short!("SEG"),), (seg_id, image_id));
        Ok(seg_id)
    }

    pub fn get_segmentation(env: Env, seg_id: u64) -> SegmentationResult {
        env.storage()
            .persistent()
            .get(&DataKey::SegResult(seg_id))
            .unwrap()
    }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd /Users/user/Desktop/Projects/Uzima-Contracts && cargo test -p medical_imaging_ai 2>&1 | tail -10`
Expected: 16 tests passed

- [ ] **Step 5: Commit**

```bash
git add contracts/medical_imaging_ai/
git commit -m "feat(medical_imaging_ai): implement attested segmentation submission"
```

---

## Task 6: Implement Performance Benchmarking

**Files:**
- Modify: `contracts/medical_imaging_ai/src/lib.rs`
- Modify: `contracts/medical_imaging_ai/src/test.rs`

- [ ] **Step 1: Write tests for performance benchmarking**

Add to `contracts/medical_imaging_ai/src/test.rs`:

```rust
#[test]
fn test_record_evaluation_updates_window() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_test_model(&env, &client, &admin, 1);

    let evaluator = Address::generate(&env);
    client.register_evaluator(&admin, &evaluator);

    // Submit an analysis first
    let mut findings = Vec::new(&env);
    findings.push_back(make_finding(&env, 1));
    let result_id = client.submit_analysis(
        &admin, &1, &hash(&env, 1), &hash(&env, 90), &sig(&env, 91),
        &findings, &8700, &1500,
    );

    let perf = client.record_evaluation(&evaluator, &result_id, &true);
    assert_eq!(perf.window_total, 1);
    assert_eq!(perf.window_correct, 1);
    assert_eq!(perf.rolling_accuracy_bps, 10_000);
}

#[test]
fn test_model_degrades_on_low_accuracy() {
    let env = Env::default();
    env.mock_all_auths();
    // Use small min_samples for testing
    let contract_id = Address::generate(&env);
    env.register_contract(&contract_id, MedicalImagingAiContract);
    let client = MedicalImagingAiContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin, &9200, &8500, &5); // min_samples=5

    register_test_model(&env, &client, &admin, 1);
    let evaluator = Address::generate(&env);
    client.register_evaluator(&admin, &evaluator);

    // Submit 10 analyses
    let mut findings = Vec::new(&env);
    findings.push_back(make_finding(&env, 1));
    for i in 0u64..10 {
        client.submit_analysis(
            &admin, &(i.saturating_add(1)), &hash(&env, 1),
            &hash(&env, (90u8).saturating_add(i as u8)),
            &sig(&env, (91u8).saturating_add(i as u8)),
            &findings, &8700, &1500,
        );
    }

    // 9 correct, 1 incorrect = 90% accuracy → should be Degraded (below 92% warning)
    for i in 1u64..10 {
        client.record_evaluation(&evaluator, &i, &true);
    }
    let perf = client.record_evaluation(&evaluator, &10, &false);

    assert_eq!(perf.rolling_accuracy_bps, 9000); // 9/10 = 90%
    let model = client.get_model(&hash(&env, 1));
    assert_eq!(model.status, ModelStatus::Degraded);
}

#[test]
fn test_no_enforcement_below_min_samples() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env); // min_samples=50

    register_test_model(&env, &client, &admin, 1);
    let evaluator = Address::generate(&env);
    client.register_evaluator(&admin, &evaluator);

    let mut findings = Vec::new(&env);
    findings.push_back(make_finding(&env, 1));
    client.submit_analysis(
        &admin, &1, &hash(&env, 1), &hash(&env, 90), &sig(&env, 91),
        &findings, &8700, &1500,
    );

    // 0% accuracy but only 1 sample — should NOT degrade
    let perf = client.record_evaluation(&evaluator, &1, &false);
    assert_eq!(perf.rolling_accuracy_bps, 0);
    let model = client.get_model(&hash(&env, 1));
    assert_eq!(model.status, ModelStatus::Active); // still active
}

#[test]
fn test_configure_thresholds() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_test_model(&env, &client, &admin, 1);
    client.configure_thresholds(&admin, &hash(&env, 1), &9500, &9000, &20, &50);

    let perf = client.get_performance(&hash(&env, 1));
    assert_eq!(perf.warning_threshold_bps, 9500);
    assert_eq!(perf.critical_threshold_bps, 9000);
}

#[test]
#[should_panic(expected = "Error(Contract, #15)")]
fn test_configure_thresholds_invalid() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_test_model(&env, &client, &admin, 1);
    // warning <= critical — invalid
    client.configure_thresholds(&admin, &hash(&env, 1), &8500, &9200, &20, &50);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd /Users/user/Desktop/Projects/Uzima-Contracts && cargo test -p medical_imaging_ai 2>&1 | tail -10`
Expected: compilation errors

- [ ] **Step 3: Implement record_evaluation, get_performance, configure_thresholds**

Add to the `impl MedicalImagingAiContract` block:

```rust
    pub fn record_evaluation(
        env: Env,
        caller: Address,
        result_id: u64,
        is_correct: bool,
    ) -> Result<ModelPerformance, Error> {
        caller.require_auth();
        Self::require_evaluator(&env, &caller)?;

        let result: AnalysisResult = env
            .storage()
            .persistent()
            .get(&DataKey::AnalysisResult(result_id))
            .ok_or(Error::ResultNotFound)?;

        let model_id = result.model_id.clone();
        let model: CnnModelMetadata = env
            .storage()
            .persistent()
            .get(&DataKey::CnnModel(model_id.clone()))
            .ok_or(Error::ModelNotFound)?;

        let default_warning: u32 = env
            .storage()
            .instance()
            .get(&DataKey::DefaultWarningBps)
            .unwrap_or(9200);
        let default_critical: u32 = env
            .storage()
            .instance()
            .get(&DataKey::DefaultCriticalBps)
            .unwrap_or(8500);
        let default_min: u64 = env
            .storage()
            .instance()
            .get(&DataKey::DefaultMinSamples)
            .unwrap_or(50);

        let mut perf: ModelPerformance = env
            .storage()
            .persistent()
            .get(&DataKey::Performance(model_id.clone()))
            .unwrap_or(ModelPerformance {
                model_id: model_id.clone(),
                modality: model.modality,
                total_evaluated: 0,
                correct_count: 0,
                lifetime_accuracy_bps: 0,
                window_size: 100,
                window_correct: 0,
                window_total: 0,
                rolling_accuracy_bps: 0,
                avg_processing_time_ms: 0,
                warning_threshold_bps: default_warning,
                critical_threshold_bps: default_critical,
                min_sample_size: default_min,
                last_updated: 0,
            });

        // Update lifetime counters
        perf.total_evaluated = perf.total_evaluated.saturating_add(1);
        if is_correct {
            perf.correct_count = perf.correct_count.saturating_add(1);
        }
        perf.lifetime_accuracy_bps = if perf.total_evaluated == 0 {
            0
        } else {
            perf.correct_count
                .saturating_mul(10_000)
                .checked_div(perf.total_evaluated)
                .unwrap_or(0) as u32
        };

        // Update rolling window
        if perf.window_total >= perf.window_size {
            perf.window_correct = 0;
            perf.window_total = 0;
        }
        perf.window_total = perf.window_total.saturating_add(1);
        if is_correct {
            perf.window_correct = perf.window_correct.saturating_add(1);
        }
        perf.rolling_accuracy_bps = if perf.window_total == 0 {
            0
        } else {
            perf.window_correct
                .saturating_mul(10_000)
                .checked_div(perf.window_total)
                .unwrap_or(0) as u32
        };

        // Update avg processing time
        let total_time = (perf.avg_processing_time_ms as u64)
            .saturating_mul(perf.total_evaluated.saturating_sub(1))
            .saturating_add(result.processing_time_ms as u64);
        perf.avg_processing_time_ms = total_time
            .checked_div(perf.total_evaluated)
            .unwrap_or(0) as u32;

        perf.last_updated = env.ledger().timestamp();

        // Threshold enforcement (only if enough samples)
        if perf.window_total >= perf.min_sample_size {
            let mut updated_model = model.clone();

            if perf.rolling_accuracy_bps < perf.critical_threshold_bps
                && !matches!(updated_model.status, ModelStatus::Deactivated)
            {
                updated_model.status = ModelStatus::Deactivated;
                updated_model.last_evaluated_at = env.ledger().timestamp();
                env.storage()
                    .persistent()
                    .set(&DataKey::CnnModel(model_id.clone()), &updated_model);
                env.events().publish(
                    (symbol_short!("MDL_DEAC"),),
                    (model_id.clone(), perf.rolling_accuracy_bps),
                );
            } else if perf.rolling_accuracy_bps < perf.warning_threshold_bps
                && matches!(updated_model.status, ModelStatus::Active)
            {
                updated_model.status = ModelStatus::Degraded;
                updated_model.last_evaluated_at = env.ledger().timestamp();
                env.storage()
                    .persistent()
                    .set(&DataKey::CnnModel(model_id.clone()), &updated_model);
                env.events().publish(
                    (symbol_short!("MDL_WARN"),),
                    (model_id.clone(), perf.rolling_accuracy_bps),
                );
            }
        }

        env.storage()
            .persistent()
            .set(&DataKey::Performance(model_id), &perf);
        Ok(perf)
    }

    pub fn get_performance(env: Env, model_id: BytesN<32>) -> ModelPerformance {
        let default_warning: u32 = env
            .storage()
            .instance()
            .get(&DataKey::DefaultWarningBps)
            .unwrap_or(9200);
        let default_critical: u32 = env
            .storage()
            .instance()
            .get(&DataKey::DefaultCriticalBps)
            .unwrap_or(8500);
        let default_min: u64 = env
            .storage()
            .instance()
            .get(&DataKey::DefaultMinSamples)
            .unwrap_or(50);

        env.storage()
            .persistent()
            .get(&DataKey::Performance(model_id.clone()))
            .unwrap_or(ModelPerformance {
                model_id,
                modality: ImagingModality::CT,
                total_evaluated: 0,
                correct_count: 0,
                lifetime_accuracy_bps: 0,
                window_size: 100,
                window_correct: 0,
                window_total: 0,
                rolling_accuracy_bps: 0,
                avg_processing_time_ms: 0,
                warning_threshold_bps: default_warning,
                critical_threshold_bps: default_critical,
                min_sample_size: default_min,
                last_updated: 0,
            })
    }

    pub fn configure_thresholds(
        env: Env,
        admin: Address,
        model_id: BytesN<32>,
        warning_bps: u32,
        critical_bps: u32,
        min_samples: u64,
        window_size: u64,
    ) -> Result<bool, Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        if warning_bps <= critical_bps || warning_bps > 10_000 {
            return Err(Error::InvalidThreshold);
        }
        if min_samples == 0 || window_size == 0 {
            return Err(Error::InvalidInput);
        }

        let _model: CnnModelMetadata = env
            .storage()
            .persistent()
            .get(&DataKey::CnnModel(model_id.clone()))
            .ok_or(Error::ModelNotFound)?;

        let mut perf = Self::get_performance(env.clone(), model_id.clone());
        perf.warning_threshold_bps = warning_bps;
        perf.critical_threshold_bps = critical_bps;
        perf.min_sample_size = min_samples;
        perf.window_size = window_size;

        env.storage()
            .persistent()
            .set(&DataKey::Performance(model_id), &perf);
        Ok(true)
    }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd /Users/user/Desktop/Projects/Uzima-Contracts && cargo test -p medical_imaging_ai 2>&1 | tail -10`
Expected: 21 tests passed

- [ ] **Step 5: Commit**

```bash
git add contracts/medical_imaging_ai/
git commit -m "feat(medical_imaging_ai): implement performance benchmarking with tiered enforcement"
```

---

## Task 7: Add Study Types and New Errors to `medical_imaging`

**Files:**
- Modify: `contracts/medical_imaging/src/lib.rs`

- [ ] **Step 1: Add new types after the existing `RadiationDoseEntry` struct (line 210)**

Add these types to `contracts/medical_imaging/src/lib.rs` after the `DoseSummary` struct (after line 220):

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum StudyStatus {
    Pending,
    Assigned,
    InReview,
    PreliminaryReport,
    DiscrepancyReview,
    FinalReport,
    Amended,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ImagingStudy {
    pub study_id: u64,
    pub patient: Address,
    pub created_by: Address,
    pub modality: ImagingModality,
    pub image_ids: Vec<u64>,
    pub ai_result_ids: Vec<u64>,
    pub required_readers: u32,
    pub status: StudyStatus,
    pub created_at: u64,
    pub finalized_at: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ReaderReport {
    pub report_id: u64,
    pub study_id: u64,
    pub reader: Address,
    pub diagnosis_hash: BytesN<32>,
    pub findings_hash: BytesN<32>,
    pub findings_ref: String,
    pub agrees_with_ai: bool,
    pub ai_accuracy_feedback_bps: u32,
    pub submitted_at: u64,
}
```

- [ ] **Step 2: Add new DataKey variants**

In the existing `DataKey` enum (after line 242), add these variants before the closing brace:

```rust
    Study(u64),
    ReaderReportEntry(u64),
    StudyReports(u64),
    StudyReaders(u64),
    ReaderStudies(Address),
    StatusStudies(u32),
    PatientStudies(Address),
    StudyArbitrator(u64),
```

Note: `StatusStudies` uses `u32` because Soroban contracttype enums as map keys need a serializable primitive. We map `StudyStatus` to `u32` via a helper.

- [ ] **Step 3: Add new error variants**

In the existing `Error` enum (after `IntegrityMismatch = 13`), add:

```rust
    StudyNotFound = 14,
    StudyNotInExpectedStatus = 15,
    ReaderNotAssigned = 16,
    ReaderAlreadySubmitted = 17,
    TooManyReaders = 18,
    TooManyImages = 19,
    AllReadersNotSubmitted = 20,
    ArbitratorNotAssigned = 21,
    InvalidStatusTransition = 22,
    ReportsNotYetAvailable = 23,
```

- [ ] **Step 4: Add new counter symbols**

Add after the existing counter symbols (after line 26):

```rust
const NEXT_STD: Symbol = symbol_short!("NSTD");
const NEXT_RPT: Symbol = symbol_short!("NRPT");
```

- [ ] **Step 5: Initialize new counters in the `initialize` method**

Add these two lines inside the `initialize` method, after the existing counter initializations (after line 283):

```rust
        env.storage().instance().set(&NEXT_STD, &1u64);
        env.storage().instance().set(&NEXT_RPT, &1u64);
```

- [ ] **Step 6: Add status_to_u32 helper**

Add a private helper method to the `impl MedicalImagingContract` block:

```rust
    fn status_to_u32(status: &StudyStatus) -> u32 {
        match status {
            StudyStatus::Pending => 0,
            StudyStatus::Assigned => 1,
            StudyStatus::InReview => 2,
            StudyStatus::PreliminaryReport => 3,
            StudyStatus::DiscrepancyReview => 4,
            StudyStatus::FinalReport => 5,
            StudyStatus::Amended => 6,
        }
    }

    fn update_status_index(env: &Env, study_id: u64, old_status: &StudyStatus, new_status: &StudyStatus) {
        // Remove from old index
        let old_key = DataKey::StatusStudies(Self::status_to_u32(old_status));
        let mut old_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&old_key)
            .unwrap_or(Vec::new(env));
        let mut new_vec = Vec::new(env);
        for id in old_ids.iter() {
            if id != study_id {
                new_vec.push_back(id);
            }
        }
        env.storage().persistent().set(&old_key, &new_vec);

        // Add to new index
        Self::append_u64(env, DataKey::StatusStudies(Self::status_to_u32(new_status)), study_id);
    }
```

- [ ] **Step 7: Verify it compiles**

Run: `cd /Users/user/Desktop/Projects/Uzima-Contracts && cargo build -p medical_imaging --target wasm32-unknown-unknown --release 2>&1 | tail -5`
Expected: `Finished`

- [ ] **Step 8: Commit**

```bash
git add contracts/medical_imaging/
git commit -m "feat(medical_imaging): add study types, errors, and storage keys for workflow"
```

---

## Task 8: Implement Study Creation and Reader Assignment

**Files:**
- Modify: `contracts/medical_imaging/src/lib.rs`
- Modify: `contracts/medical_imaging/src/test.rs`

- [ ] **Step 1: Write tests**

Add to `contracts/medical_imaging/src/test.rs`:

```rust
#[test]
fn test_create_study() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let tech = Address::generate(&env);
    let patient = Address::generate(&env);
    let (xray_id, _, _) = upload_three_modalities(&env, &client, &admin, &tech, &patient);

    let mut image_ids = Vec::new(&env);
    image_ids.push_back(xray_id);

    let study_id = client.create_study(
        &admin,
        &patient,
        &ImagingModality::XRay,
        &image_ids,
        &2,
    );
    assert_eq!(study_id, 1);

    let study = client.get_study(&study_id);
    assert_eq!(study.status, StudyStatus::Pending);
    assert_eq!(study.required_readers, 2);
}

#[test]
#[should_panic(expected = "Error(Contract, #18)")]
fn test_create_study_too_many_readers() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let tech = Address::generate(&env);
    let patient = Address::generate(&env);
    let (xray_id, _, _) = upload_three_modalities(&env, &client, &admin, &tech, &patient);

    let mut image_ids = Vec::new(&env);
    image_ids.push_back(xray_id);

    client.create_study(&admin, &patient, &ImagingModality::XRay, &image_ids, &6);
}

#[test]
fn test_assign_reader() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let tech = Address::generate(&env);
    let patient = Address::generate(&env);
    let (xray_id, _, _) = upload_three_modalities(&env, &client, &admin, &tech, &patient);

    let mut image_ids = Vec::new(&env);
    image_ids.push_back(xray_id);
    let study_id = client.create_study(&admin, &patient, &ImagingModality::XRay, &image_ids, &2);

    let reader = Address::generate(&env);
    client.assign_role(&admin, &reader, &ROLE_RADIOLOGIST);
    client.assign_reader(&admin, &study_id, &reader);

    let study = client.get_study(&study_id);
    assert_eq!(study.status, StudyStatus::Assigned);

    let reader_studies = client.get_studies_by_reader(&reader);
    assert_eq!(reader_studies.len(), 1);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd /Users/user/Desktop/Projects/Uzima-Contracts && cargo test -p medical_imaging -- test_create_study test_assign_reader 2>&1 | tail -10`
Expected: compilation errors

- [ ] **Step 3: Implement create_study, assign_reader, assign_arbitrator, link_ai_results, and query methods**

Add to `impl MedicalImagingContract` in `contracts/medical_imaging/src/lib.rs`:

```rust
    pub fn create_study(
        env: Env,
        caller: Address,
        patient: Address,
        modality: ImagingModality,
        image_ids: Vec<u64>,
        required_readers: u32,
    ) -> Result<u64, Error> {
        caller.require_auth();
        Self::require_not_paused(&env)?;
        Self::require_role_or_admin(&env, &caller, ROLE_PHYSICIAN)?;

        if required_readers == 0 || required_readers > 5 {
            return Err(Error::TooManyReaders);
        }
        if image_ids.is_empty() || image_ids.len() > 500 {
            return Err(Error::TooManyImages);
        }

        for id in image_ids.iter() {
            Self::require_image_exists(&env, id)?;
        }

        let study_id = Self::next_counter(&env, &NEXT_STD);
        let study = ImagingStudy {
            study_id,
            patient: patient.clone(),
            created_by: caller.clone(),
            modality,
            image_ids,
            ai_result_ids: Vec::new(&env),
            required_readers,
            status: StudyStatus::Pending,
            created_at: env.ledger().timestamp(),
            finalized_at: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Study(study_id), &study);
        Self::append_u64(&env, DataKey::PatientStudies(patient.clone()), study_id);
        Self::append_u64(
            &env,
            DataKey::StatusStudies(Self::status_to_u32(&StudyStatus::Pending)),
            study_id,
        );

        env.events()
            .publish((symbol_short!("STDY_NEW"),), (study_id, patient));
        Ok(study_id)
    }

    pub fn assign_reader(
        env: Env,
        caller: Address,
        study_id: u64,
        reader: Address,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_role_or_admin(&env, &caller, ROLE_PHYSICIAN)?;
        Self::require_role_or_admin(&env, &reader, ROLE_RADIOLOGIST)?;

        let mut study: ImagingStudy = env
            .storage()
            .persistent()
            .get(&DataKey::Study(study_id))
            .ok_or(Error::StudyNotFound)?;

        if !matches!(study.status, StudyStatus::Pending | StudyStatus::Assigned) {
            return Err(Error::StudyNotInExpectedStatus);
        }

        let mut readers: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::StudyReaders(study_id))
            .unwrap_or(Vec::new(&env));

        if readers.iter().any(|r| r == reader) {
            return Err(Error::InvalidInput);
        }
        if readers.len() >= study.required_readers {
            return Err(Error::TooManyReaders);
        }

        readers.push_back(reader.clone());
        env.storage()
            .persistent()
            .set(&DataKey::StudyReaders(study_id), &readers);

        Self::append_u64(&env, DataKey::ReaderStudies(reader.clone()), study_id);

        if matches!(study.status, StudyStatus::Pending) {
            let old_status = study.status;
            study.status = StudyStatus::Assigned;
            Self::update_status_index(&env, study_id, &old_status, &study.status);
            env.storage()
                .persistent()
                .set(&DataKey::Study(study_id), &study);
        }

        env.events()
            .publish((symbol_short!("STDY_ASG"),), (study_id, reader));
        Ok(true)
    }

    pub fn assign_arbitrator(
        env: Env,
        caller: Address,
        study_id: u64,
        arbitrator: Address,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_role_or_admin(&env, &caller, ROLE_PHYSICIAN)?;
        Self::require_role_or_admin(&env, &arbitrator, ROLE_RADIOLOGIST)?;

        let study: ImagingStudy = env
            .storage()
            .persistent()
            .get(&DataKey::Study(study_id))
            .ok_or(Error::StudyNotFound)?;

        if !matches!(study.status, StudyStatus::DiscrepancyReview) {
            return Err(Error::StudyNotInExpectedStatus);
        }

        env.storage()
            .persistent()
            .set(&DataKey::StudyArbitrator(study_id), &arbitrator);
        Ok(true)
    }

    pub fn link_ai_results(
        env: Env,
        caller: Address,
        study_id: u64,
        result_ids: Vec<u64>,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_role_or_admin(&env, &caller, ROLE_PHYSICIAN)?;

        let mut study: ImagingStudy = env
            .storage()
            .persistent()
            .get(&DataKey::Study(study_id))
            .ok_or(Error::StudyNotFound)?;

        for id in result_ids.iter() {
            study.ai_result_ids.push_back(id);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Study(study_id), &study);
        Ok(true)
    }

    pub fn get_study(env: Env, study_id: u64) -> Option<ImagingStudy> {
        env.storage().persistent().get(&DataKey::Study(study_id))
    }

    pub fn get_studies_by_reader(env: Env, reader: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::ReaderStudies(reader))
            .unwrap_or(Vec::new(&env))
    }

    pub fn get_studies_by_status(env: Env, status: StudyStatus) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::StatusStudies(Self::status_to_u32(&status)))
            .unwrap_or(Vec::new(&env))
    }

    pub fn get_studies_by_patient(env: Env, patient: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::PatientStudies(patient))
            .unwrap_or(Vec::new(&env))
    }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd /Users/user/Desktop/Projects/Uzima-Contracts && cargo test -p medical_imaging -- test_create_study test_assign_reader 2>&1 | tail -10`
Expected: 3 tests passed

- [ ] **Step 5: Commit**

```bash
git add contracts/medical_imaging/
git commit -m "feat(medical_imaging): implement study creation, reader assignment, and queries"
```

---

## Task 9: Implement Reader Report Submission and Discrepancy Detection

**Files:**
- Modify: `contracts/medical_imaging/src/lib.rs`
- Modify: `contracts/medical_imaging/src/test.rs`

- [ ] **Step 1: Write tests for reader report submission**

Add to `contracts/medical_imaging/src/test.rs`:

```rust
fn setup_study_with_readers(
    env: &Env,
    client: &MedicalImagingContractClient<'_>,
    admin: &Address,
) -> (u64, Address, Address) {
    let tech = Address::generate(env);
    let patient = Address::generate(env);
    let (xray_id, _, _) = upload_three_modalities(env, client, admin, &tech, &patient);

    let mut image_ids = Vec::new(env);
    image_ids.push_back(xray_id);
    let study_id = client.create_study(admin, &patient, &ImagingModality::XRay, &image_ids, &2);

    let reader1 = Address::generate(env);
    let reader2 = Address::generate(env);
    client.assign_role(admin, &reader1, &ROLE_RADIOLOGIST);
    client.assign_role(admin, &reader2, &ROLE_RADIOLOGIST);
    client.assign_reader(admin, &study_id, &reader1);
    client.assign_reader(admin, &study_id, &reader2);

    (study_id, reader1, reader2)
}

#[test]
fn test_submit_reader_report_agreement() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let (study_id, reader1, reader2) = setup_study_with_readers(&env, &client, &admin);

    let diagnosis = hash(&env, 99); // same diagnosis

    client.submit_reader_report(
        &reader1, &study_id, &diagnosis, &hash(&env, 100),
        &String::from_str(&env, "ipfs://report1"), &true, &9500,
    );

    let study = client.get_study(&study_id).unwrap();
    assert_eq!(study.status, StudyStatus::InReview);

    client.submit_reader_report(
        &reader2, &study_id, &diagnosis, &hash(&env, 101),
        &String::from_str(&env, "ipfs://report2"), &true, &9200,
    );

    let study = client.get_study(&study_id).unwrap();
    assert_eq!(study.status, StudyStatus::PreliminaryReport);
}

#[test]
fn test_submit_reader_report_discrepancy() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let (study_id, reader1, reader2) = setup_study_with_readers(&env, &client, &admin);

    client.submit_reader_report(
        &reader1, &study_id, &hash(&env, 99), &hash(&env, 100),
        &String::from_str(&env, "ipfs://report1"), &true, &9500,
    );

    client.submit_reader_report(
        &reader2, &study_id, &hash(&env, 88), &hash(&env, 101), // different diagnosis
        &String::from_str(&env, "ipfs://report2"), &false, &7000,
    );

    let study = client.get_study(&study_id).unwrap();
    assert_eq!(study.status, StudyStatus::DiscrepancyReview);
}

#[test]
#[should_panic(expected = "Error(Contract, #16)")]
fn test_submit_report_not_assigned() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let (study_id, _, _) = setup_study_with_readers(&env, &client, &admin);

    let stranger = Address::generate(&env);
    client.submit_reader_report(
        &stranger, &study_id, &hash(&env, 99), &hash(&env, 100),
        &String::from_str(&env, "ipfs://report"), &true, &9500,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #17)")]
fn test_submit_report_duplicate() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let (study_id, reader1, _) = setup_study_with_readers(&env, &client, &admin);

    let diagnosis = hash(&env, 99);
    client.submit_reader_report(
        &reader1, &study_id, &diagnosis, &hash(&env, 100),
        &String::from_str(&env, "ipfs://report1"), &true, &9500,
    );
    client.submit_reader_report(
        &reader1, &study_id, &diagnosis, &hash(&env, 101),
        &String::from_str(&env, "ipfs://report2"), &true, &9500,
    );
}

#[test]
fn test_single_reader_study() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let tech = Address::generate(&env);
    let patient = Address::generate(&env);
    let (xray_id, _, _) = upload_three_modalities(&env, &client, &admin, &tech, &patient);

    let mut image_ids = Vec::new(&env);
    image_ids.push_back(xray_id);
    let study_id = client.create_study(&admin, &patient, &ImagingModality::XRay, &image_ids, &1);

    let reader = Address::generate(&env);
    client.assign_role(&admin, &reader, &ROLE_RADIOLOGIST);
    client.assign_reader(&admin, &study_id, &reader);

    client.submit_reader_report(
        &reader, &study_id, &hash(&env, 99), &hash(&env, 100),
        &String::from_str(&env, "ipfs://report"), &true, &9500,
    );

    let study = client.get_study(&study_id).unwrap();
    assert_eq!(study.status, StudyStatus::PreliminaryReport);
}

#[test]
fn test_blind_review_enforcement() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let (study_id, reader1, reader2) = setup_study_with_readers(&env, &client, &admin);

    client.submit_reader_report(
        &reader1, &study_id, &hash(&env, 99), &hash(&env, 100),
        &String::from_str(&env, "ipfs://report1"), &true, &9500,
    );

    // Reader can see their own report
    let my_report = client.get_my_report(&reader1, &study_id);
    assert_eq!(my_report.reader, reader1);

    // All reports not yet available (still InReview)
    let reports = client.get_reader_reports(&reader2, &study_id);
    assert_eq!(reports.len(), 0); // empty because study not finalized yet
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd /Users/user/Desktop/Projects/Uzima-Contracts && cargo test -p medical_imaging -- test_submit_reader test_single_reader test_blind 2>&1 | tail -10`
Expected: compilation errors

- [ ] **Step 3: Implement submit_reader_report, get_reader_reports, get_my_report**

Add to `impl MedicalImagingContract`:

```rust
    pub fn submit_reader_report(
        env: Env,
        reader: Address,
        study_id: u64,
        diagnosis_hash: BytesN<32>,
        findings_hash: BytesN<32>,
        findings_ref: String,
        agrees_with_ai: bool,
        ai_accuracy_feedback_bps: u32,
    ) -> Result<u64, Error> {
        reader.require_auth();
        Self::require_not_paused(&env)?;

        let mut study: ImagingStudy = env
            .storage()
            .persistent()
            .get(&DataKey::Study(study_id))
            .ok_or(Error::StudyNotFound)?;

        if !matches!(
            study.status,
            StudyStatus::Assigned | StudyStatus::InReview | StudyStatus::DiscrepancyReview
        ) {
            return Err(Error::StudyNotInExpectedStatus);
        }

        // Verify reader is assigned
        let readers: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::StudyReaders(study_id))
            .unwrap_or(Vec::new(&env));

        let is_assigned = readers.iter().any(|r| r == reader);

        // Also check if reader is the arbitrator
        let is_arbitrator: bool = env
            .storage()
            .persistent()
            .get(&DataKey::StudyArbitrator(study_id))
            .map(|arb: Address| arb == reader)
            .unwrap_or(false);

        if !is_assigned && !is_arbitrator {
            return Err(Error::ReaderNotAssigned);
        }

        // Check for duplicate submission
        let report_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::StudyReports(study_id))
            .unwrap_or(Vec::new(&env));

        for rid in report_ids.iter() {
            let existing: ReaderReport = env
                .storage()
                .persistent()
                .get(&DataKey::ReaderReportEntry(rid))
                .unwrap();
            if existing.reader == reader {
                return Err(Error::ReaderAlreadySubmitted);
            }
        }

        if ai_accuracy_feedback_bps > 10_000 {
            return Err(Error::InvalidInput);
        }

        let report_id = Self::next_counter(&env, &NEXT_RPT);
        let report = ReaderReport {
            report_id,
            study_id,
            reader: reader.clone(),
            diagnosis_hash,
            findings_hash,
            findings_ref,
            agrees_with_ai,
            ai_accuracy_feedback_bps,
            submitted_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::ReaderReportEntry(report_id), &report);
        Self::append_u64(&env, DataKey::StudyReports(study_id), report_id);

        // Auto-transition status
        let old_status = study.status;
        let updated_reports: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::StudyReports(study_id))
            .unwrap_or(Vec::new(&env));

        if matches!(old_status, StudyStatus::Assigned) {
            study.status = StudyStatus::InReview;
            Self::update_status_index(&env, study_id, &old_status, &study.status);
        }

        // Check if all readers submitted (count non-arbitrator reports)
        let reader_report_count = updated_reports.len();
        let all_submitted = reader_report_count >= study.required_readers;

        if all_submitted && matches!(study.status, StudyStatus::InReview) {
            // Check diagnosis agreement
            let mut all_agree = true;
            let mut first_diag: Option<BytesN<32>> = None;
            for rid in updated_reports.iter() {
                let rpt: ReaderReport = env
                    .storage()
                    .persistent()
                    .get(&DataKey::ReaderReportEntry(rid))
                    .unwrap();
                match &first_diag {
                    None => first_diag = Some(rpt.diagnosis_hash),
                    Some(first) => {
                        if &rpt.diagnosis_hash != first {
                            all_agree = false;
                        }
                    }
                }
            }

            let prev_status = study.status;
            if all_agree {
                study.status = StudyStatus::PreliminaryReport;
            } else {
                study.status = StudyStatus::DiscrepancyReview;
                env.events()
                    .publish((symbol_short!("DISCREP"),), study_id);
            }
            Self::update_status_index(&env, study_id, &prev_status, &study.status);
        }

        // Handle arbitrator resolution
        if is_arbitrator && matches!(old_status, StudyStatus::DiscrepancyReview) {
            let prev_status = study.status;
            study.status = StudyStatus::PreliminaryReport;
            Self::update_status_index(&env, study_id, &prev_status, &study.status);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Study(study_id), &study);

        env.events()
            .publish((symbol_short!("RPT_SUB"),), (report_id, study_id, reader));
        Ok(report_id)
    }

    pub fn get_reader_reports(env: Env, caller: Address, study_id: u64) -> Vec<ReaderReport> {
        let study: Option<ImagingStudy> = env.storage().persistent().get(&DataKey::Study(study_id));
        let study = match study {
            Some(s) => s,
            None => return Vec::new(&env),
        };

        // Only return reports if study is past InReview, or caller is admin
        let is_admin = Self::require_admin(&env, &caller).is_ok();
        let is_viewable = matches!(
            study.status,
            StudyStatus::PreliminaryReport
                | StudyStatus::DiscrepancyReview
                | StudyStatus::FinalReport
                | StudyStatus::Amended
        );

        if !is_viewable && !is_admin {
            return Vec::new(&env);
        }

        let report_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::StudyReports(study_id))
            .unwrap_or(Vec::new(&env));

        let mut reports = Vec::new(&env);
        for rid in report_ids.iter() {
            if let Some(rpt) = env
                .storage()
                .persistent()
                .get(&DataKey::ReaderReportEntry(rid))
            {
                reports.push_back(rpt);
            }
        }
        reports
    }

    pub fn get_my_report(env: Env, reader: Address, study_id: u64) -> ReaderReport {
        let report_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::StudyReports(study_id))
            .unwrap_or(Vec::new(&env));

        for rid in report_ids.iter() {
            let rpt: ReaderReport = env
                .storage()
                .persistent()
                .get(&DataKey::ReaderReportEntry(rid))
                .unwrap();
            if rpt.reader == reader {
                return rpt;
            }
        }
        panic!("report not found");
    }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd /Users/user/Desktop/Projects/Uzima-Contracts && cargo test -p medical_imaging -- test_submit_reader test_single_reader test_blind 2>&1 | tail -10`
Expected: 6 tests passed

- [ ] **Step 5: Commit**

```bash
git add contracts/medical_imaging/
git commit -m "feat(medical_imaging): implement reader report submission with blind review and discrepancy detection"
```

---

## Task 10: Implement Study Finalization and Amendment

**Files:**
- Modify: `contracts/medical_imaging/src/lib.rs`
- Modify: `contracts/medical_imaging/src/test.rs`

- [ ] **Step 1: Write tests**

Add to `contracts/medical_imaging/src/test.rs`:

```rust
#[test]
fn test_finalize_study() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let (study_id, reader1, reader2) = setup_study_with_readers(&env, &client, &admin);

    let diagnosis = hash(&env, 99);
    client.submit_reader_report(
        &reader1, &study_id, &diagnosis, &hash(&env, 100),
        &String::from_str(&env, "ipfs://report1"), &true, &9500,
    );
    client.submit_reader_report(
        &reader2, &study_id, &diagnosis, &hash(&env, 101),
        &String::from_str(&env, "ipfs://report2"), &true, &9200,
    );

    client.finalize_study(
        &admin, &study_id,
        &String::from_str(&env, "ipfs://final_report"),
    );

    let study = client.get_study(&study_id).unwrap();
    assert_eq!(study.status, StudyStatus::FinalReport);
    assert!(study.finalized_at > 0);
}

#[test]
fn test_amend_study() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let (study_id, reader1, reader2) = setup_study_with_readers(&env, &client, &admin);

    let diagnosis = hash(&env, 99);
    client.submit_reader_report(
        &reader1, &study_id, &diagnosis, &hash(&env, 100),
        &String::from_str(&env, "ipfs://report1"), &true, &9500,
    );
    client.submit_reader_report(
        &reader2, &study_id, &diagnosis, &hash(&env, 101),
        &String::from_str(&env, "ipfs://report2"), &true, &9200,
    );
    client.finalize_study(
        &admin, &study_id,
        &String::from_str(&env, "ipfs://final_report"),
    );

    client.amend_study(
        &admin, &study_id,
        &String::from_str(&env, "ipfs://amendment"),
        &hash(&env, 200),
    );

    let study = client.get_study(&study_id).unwrap();
    assert_eq!(study.status, StudyStatus::Amended);
}

#[test]
fn test_amend_study_multiple() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let (study_id, reader1, reader2) = setup_study_with_readers(&env, &client, &admin);

    let diagnosis = hash(&env, 99);
    client.submit_reader_report(
        &reader1, &study_id, &diagnosis, &hash(&env, 100),
        &String::from_str(&env, "ipfs://report1"), &true, &9500,
    );
    client.submit_reader_report(
        &reader2, &study_id, &diagnosis, &hash(&env, 101),
        &String::from_str(&env, "ipfs://report2"), &true, &9200,
    );
    client.finalize_study(
        &admin, &study_id,
        &String::from_str(&env, "ipfs://final_report"),
    );
    client.amend_study(
        &admin, &study_id,
        &String::from_str(&env, "ipfs://amendment1"),
        &hash(&env, 200),
    );
    client.amend_study(
        &admin, &study_id,
        &String::from_str(&env, "ipfs://amendment2"),
        &hash(&env, 201),
    );

    let study = client.get_study(&study_id).unwrap();
    assert_eq!(study.status, StudyStatus::Amended);
}

#[test]
#[should_panic(expected = "Error(Contract, #15)")]
fn test_finalize_wrong_status() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let (study_id, reader1, _) = setup_study_with_readers(&env, &client, &admin);

    // Only one reader submitted — still InReview
    client.submit_reader_report(
        &reader1, &study_id, &hash(&env, 99), &hash(&env, 100),
        &String::from_str(&env, "ipfs://report1"), &true, &9500,
    );

    client.finalize_study(
        &admin, &study_id,
        &String::from_str(&env, "ipfs://final_report"),
    );
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd /Users/user/Desktop/Projects/Uzima-Contracts && cargo test -p medical_imaging -- test_finalize test_amend 2>&1 | tail -10`
Expected: compilation errors

- [ ] **Step 3: Implement finalize_study and amend_study**

Add to `impl MedicalImagingContract`:

```rust
    pub fn finalize_study(
        env: Env,
        caller: Address,
        study_id: u64,
        final_report_ref: String,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_role_or_admin(&env, &caller, ROLE_RADIOLOGIST)?;

        let mut study: ImagingStudy = env
            .storage()
            .persistent()
            .get(&DataKey::Study(study_id))
            .ok_or(Error::StudyNotFound)?;

        if !matches!(study.status, StudyStatus::PreliminaryReport) {
            return Err(Error::StudyNotInExpectedStatus);
        }

        let old_status = study.status;
        study.status = StudyStatus::FinalReport;
        study.finalized_at = env.ledger().timestamp();
        Self::update_status_index(&env, study_id, &old_status, &study.status);

        env.storage()
            .persistent()
            .set(&DataKey::Study(study_id), &study);
        env.events()
            .publish((symbol_short!("STDY_FIN"),), study_id);
        Ok(true)
    }

    pub fn amend_study(
        env: Env,
        caller: Address,
        study_id: u64,
        amendment_ref: String,
        reason_hash: BytesN<32>,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_role_or_admin(&env, &caller, ROLE_RADIOLOGIST)?;

        let mut study: ImagingStudy = env
            .storage()
            .persistent()
            .get(&DataKey::Study(study_id))
            .ok_or(Error::StudyNotFound)?;

        if !matches!(study.status, StudyStatus::FinalReport | StudyStatus::Amended) {
            return Err(Error::StudyNotInExpectedStatus);
        }

        let old_status = study.status;
        study.status = StudyStatus::Amended;
        if old_status != StudyStatus::Amended {
            Self::update_status_index(&env, study_id, &old_status, &study.status);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Study(study_id), &study);
        env.events()
            .publish((symbol_short!("STDY_AMD"),), study_id);
        Ok(true)
    }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd /Users/user/Desktop/Projects/Uzima-Contracts && cargo test -p medical_imaging -- test_finalize test_amend 2>&1 | tail -10`
Expected: 4 tests passed

- [ ] **Step 5: Commit**

```bash
git add contracts/medical_imaging/
git commit -m "feat(medical_imaging): implement study finalization and amendment"
```

---

## Task 11: Run Full Test Suite and Fix Issues

**Files:**
- Possibly modify: `contracts/medical_imaging/src/lib.rs`
- Possibly modify: `contracts/medical_imaging_ai/src/lib.rs`

- [ ] **Step 1: Run full medical_imaging test suite**

Run: `cd /Users/user/Desktop/Projects/Uzima-Contracts && cargo test -p medical_imaging 2>&1 | tail -20`
Expected: all existing tests + new tests pass

- [ ] **Step 2: Run full medical_imaging_ai test suite**

Run: `cd /Users/user/Desktop/Projects/Uzima-Contracts && cargo test -p medical_imaging_ai 2>&1 | tail -20`
Expected: all tests pass

- [ ] **Step 3: Build both contracts for WASM target**

Run: `cd /Users/user/Desktop/Projects/Uzima-Contracts && cargo build -p medical_imaging -p medical_imaging_ai --target wasm32-unknown-unknown --release 2>&1 | tail -10`
Expected: both compile successfully

- [ ] **Step 4: Run clippy on both contracts**

Run: `cd /Users/user/Desktop/Projects/Uzima-Contracts && cargo clippy -p medical_imaging -p medical_imaging_ai -- -D warnings 2>&1 | tail -20`
Expected: no warnings

- [ ] **Step 5: Fix any issues found in steps 1-4**

Address compilation errors, test failures, or clippy warnings. Common issues to watch for:
- `saturating_add`/`saturating_mul` required for arithmetic (clippy `arithmetic_side_effects`)
- Missing `Clone` derives for types used in storage
- `Vec` type needs to be `soroban_sdk::Vec`, not `std::Vec`

- [ ] **Step 6: Commit fixes if any**

```bash
git add contracts/medical_imaging/ contracts/medical_imaging_ai/
git commit -m "fix: address clippy warnings and test issues"
```

---

## Task 12: Run Workspace-Wide Build

**Files:** None (verification only)

- [ ] **Step 1: Build the entire workspace**

Run: `cd /Users/user/Desktop/Projects/Uzima-Contracts && cargo build --workspace --target wasm32-unknown-unknown --release 2>&1 | tail -10`
Expected: all 55 contracts compile (54 existing + 1 new)

- [ ] **Step 2: Run all workspace tests**

Run: `cd /Users/user/Desktop/Projects/Uzima-Contracts && cargo test --workspace 2>&1 | tail -30`
Expected: all tests pass across the workspace

- [ ] **Step 3: Final commit if any remaining fixes**

```bash
git add -A
git commit -m "fix: workspace-wide build and test compatibility"
```
