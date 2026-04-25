#![no_std]
#![allow(clippy::too_many_arguments)]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
    String, Symbol, Vec,
};

// =============================================================================
// Types
// =============================================================================

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum HEScheme {
    Paillier,
    BFV,
    BGV,
    CKKS,
    TFHE,
    Custom(u32),
}

#[derive(Clone)]
#[contracttype]
pub struct HEContext {
    pub context_id: BytesN<32>,
    pub scheme: HEScheme,
    pub params_ref: String,
    pub params_hash: BytesN<32>,
    pub created_at: u64,
    pub is_active: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct EncryptedComputation {
    pub computation_id: BytesN<32>,
    pub context_id: BytesN<32>,
    pub submitter: Address,
    pub ciphertext_ref: String,
    pub ciphertext_hash: BytesN<32>,
    /// Optional proof reference; empty string means "no proof".
    pub proof_ref: String,
    /// Optional proof hash; all-zero means "no proof".
    pub proof_hash: BytesN<32>,
    pub submitted_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct FHEKeyBundle {
    pub key_id: BytesN<32>,
    pub context_id: BytesN<32>,
    pub version: u32,
    pub public_key_ref: String,
    pub eval_key_ref: String,
    pub relin_key_ref: String,
    pub galois_key_ref: String,
    pub key_hash: BytesN<32>,
    pub created_at: u64,
    pub is_active: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct PerformanceProfile {
    pub context_id: BytesN<32>,
    pub batching_enabled: bool,
    pub max_batch_size: u32,
    pub lazy_relinearization: bool,
    pub auto_bootstrap: bool,
    pub bootstrap_threshold: u32,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct EncryptedVector {
    pub ciphertext_id: BytesN<32>,
    pub context_id: BytesN<32>,
    pub owner: Address,
    pub scheme: HEScheme,
    /// For CKKS we treat each value as fixed-point and keep this decimal precision.
    pub scale: u32,
    /// Remaining simulated noise budget in "levels".
    pub noise_budget: u32,
    /// Multiplicative depth reached by this ciphertext.
    pub multiplicative_depth: u32,
    pub slots: Vec<i128>,
    pub created_at: u64,
    pub last_bootstrapped_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct EncryptedStats {
    pub ciphertext_id: BytesN<32>,
    pub count: u32,
    pub sum: i128,
    pub mean_scaled: i128,
    pub variance_scaled: i128,
    pub min: i128,
    pub max: i128,
}

#[contracttype]
pub enum DataKey {
    Initialized,
    Admin,
    Context(BytesN<32>),
    Computation(BytesN<32>),
    KeyBundle(BytesN<32>),
    ActiveKey(BytesN<32>),
    Ciphertext(BytesN<32>),
    Profile(BytesN<32>),
}

const ADMIN: Symbol = symbol_short!("ADMIN");

// =============================================================================
// Errors
// =============================================================================

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    ContextNotFound = 4,
    ContextInactive = 5,
    InvalidInput = 6,
    ComputationAlreadyExists = 7,
    CiphertextNotFound = 8,
    CiphertextAlreadyExists = 9,
    SchemeMismatch = 10,
    IncompatibleDimensions = 11,
    NoiseBudgetExhausted = 12,
    ArithmeticOverflow = 13,
    KeyNotFound = 14,
}

// =============================================================================
// Contract
// =============================================================================

#[contract]
pub struct HomomorphicRegistry;

#[contractimpl]
impl HomomorphicRegistry {
    const DEFAULT_NOISE_BUDGET: u32 = 64;

    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&ADMIN, &admin);
        env.events()
            .publish((symbol_short!("he"), symbol_short!("init")), admin);
        Ok(())
    }

    pub fn register_key_bundle(
        env: Env,
        admin: Address,
        key_id: BytesN<32>,
        context_id: BytesN<32>,
        public_key_ref: String,
        eval_key_ref: String,
        relin_key_ref: String,
        galois_key_ref: String,
        key_hash: BytesN<32>,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_initialized(&env)?;
        Self::require_admin(&env, &admin)?;
        Self::require_active_context(&env, &context_id)?;

        if public_key_ref.is_empty() || eval_key_ref.is_empty() {
            return Err(Error::InvalidInput);
        }

        let next_version = if let Some(active_key_id) = env
            .storage()
            .persistent()
            .get::<_, BytesN<32>>(&DataKey::ActiveKey(context_id.clone()))
        {
            let active: FHEKeyBundle = env
                .storage()
                .persistent()
                .get(&DataKey::KeyBundle(active_key_id))
                .ok_or(Error::KeyNotFound)?;
            active
                .version
                .checked_add(1)
                .ok_or(Error::ArithmeticOverflow)?
        } else {
            1
        };

        let bundle = FHEKeyBundle {
            key_id: key_id.clone(),
            context_id: context_id.clone(),
            version: next_version,
            public_key_ref,
            eval_key_ref,
            relin_key_ref,
            galois_key_ref,
            key_hash,
            created_at: env.ledger().timestamp(),
            is_active: true,
        };

        env.storage()
            .persistent()
            .set(&DataKey::KeyBundle(key_id.clone()), &bundle);
        env.storage()
            .persistent()
            .set(&DataKey::ActiveKey(context_id.clone()), &key_id);
        env.events().publish(
            (symbol_short!("he"), symbol_short!("key")),
            (context_id, key_id),
        );
        Ok(())
    }

    pub fn get_active_key_bundle(
        env: Env,
        context_id: BytesN<32>,
    ) -> Result<Option<FHEKeyBundle>, Error> {
        Self::require_initialized(&env)?;
        if let Some(key_id) = env
            .storage()
            .persistent()
            .get::<_, BytesN<32>>(&DataKey::ActiveKey(context_id))
        {
            let bundle: FHEKeyBundle = env
                .storage()
                .persistent()
                .get(&DataKey::KeyBundle(key_id))
                .ok_or(Error::KeyNotFound)?;
            Ok(Some(bundle))
        } else {
            Ok(None)
        }
    }

    pub fn set_performance_profile(
        env: Env,
        admin: Address,
        context_id: BytesN<32>,
        batching_enabled: bool,
        max_batch_size: u32,
        lazy_relinearization: bool,
        auto_bootstrap: bool,
        bootstrap_threshold: u32,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_initialized(&env)?;
        Self::require_admin(&env, &admin)?;
        Self::require_active_context(&env, &context_id)?;

        if max_batch_size == 0 {
            return Err(Error::InvalidInput);
        }
        if auto_bootstrap && bootstrap_threshold == 0 {
            return Err(Error::InvalidInput);
        }

        let now = env.ledger().timestamp();
        let created_at = env
            .storage()
            .persistent()
            .get::<_, PerformanceProfile>(&DataKey::Profile(context_id.clone()))
            .map(|v| v.created_at)
            .unwrap_or(now);

        let profile = PerformanceProfile {
            context_id: context_id.clone(),
            batching_enabled,
            max_batch_size,
            lazy_relinearization,
            auto_bootstrap,
            bootstrap_threshold,
            created_at,
            updated_at: now,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Profile(context_id.clone()), &profile);
        env.events()
            .publish((symbol_short!("he"), symbol_short!("perf")), context_id);
        Ok(())
    }

    pub fn get_performance_profile(
        env: Env,
        context_id: BytesN<32>,
    ) -> Result<Option<PerformanceProfile>, Error> {
        Self::require_initialized(&env)?;
        Ok(env
            .storage()
            .persistent()
            .get(&DataKey::Profile(context_id)))
    }

    pub fn encrypt_ckks_vector(
        env: Env,
        submitter: Address,
        ciphertext_id: BytesN<32>,
        context_id: BytesN<32>,
        values: Vec<i128>,
        scale: u32,
    ) -> Result<(), Error> {
        submitter.require_auth();
        Self::require_initialized(&env)?;
        let ctx = Self::require_active_context(&env, &context_id)?;
        if ctx.scheme != HEScheme::CKKS {
            return Err(Error::SchemeMismatch);
        }
        Self::store_ciphertext(
            &env,
            submitter,
            ciphertext_id,
            context_id,
            HEScheme::CKKS,
            values,
            scale,
            Self::DEFAULT_NOISE_BUDGET,
            0,
        )
    }

    pub fn encrypt_bgv_vector(
        env: Env,
        submitter: Address,
        ciphertext_id: BytesN<32>,
        context_id: BytesN<32>,
        values: Vec<i128>,
    ) -> Result<(), Error> {
        submitter.require_auth();
        Self::require_initialized(&env)?;
        let ctx = Self::require_active_context(&env, &context_id)?;
        if ctx.scheme != HEScheme::BGV {
            return Err(Error::SchemeMismatch);
        }
        Self::store_ciphertext(
            &env,
            submitter,
            ciphertext_id,
            context_id,
            HEScheme::BGV,
            values,
            0,
            Self::DEFAULT_NOISE_BUDGET,
            0,
        )
    }

    pub fn fhe_add(
        env: Env,
        submitter: Address,
        output_id: BytesN<32>,
        left_id: BytesN<32>,
        right_id: BytesN<32>,
    ) -> Result<(), Error> {
        submitter.require_auth();
        Self::require_initialized(&env)?;
        let left = Self::load_ciphertext(&env, &left_id)?;
        let right = Self::load_ciphertext(&env, &right_id)?;
        Self::require_binary_compatible(&left, &right)?;

        let out_slots = Self::add_vectors(&env, &left.slots, &right.slots)?;
        let noise = left
            .noise_budget
            .min(right.noise_budget)
            .checked_sub(1)
            .ok_or(Error::NoiseBudgetExhausted)?;
        let depth = left.multiplicative_depth.max(right.multiplicative_depth);
        Self::store_ciphertext(
            &env,
            submitter,
            output_id,
            left.context_id,
            left.scheme,
            out_slots,
            left.scale.max(right.scale),
            noise,
            depth,
        )
    }

    pub fn fhe_multiply(
        env: Env,
        submitter: Address,
        output_id: BytesN<32>,
        left_id: BytesN<32>,
        right_id: BytesN<32>,
    ) -> Result<(), Error> {
        submitter.require_auth();
        Self::require_initialized(&env)?;
        let left = Self::load_ciphertext(&env, &left_id)?;
        let right = Self::load_ciphertext(&env, &right_id)?;
        Self::require_binary_compatible(&left, &right)?;

        let out_slots = Self::mul_vectors(&env, &left.slots, &right.slots)?;
        let base_noise = left.noise_budget.min(right.noise_budget);
        let mut noise = base_noise
            .checked_sub(8)
            .ok_or(Error::NoiseBudgetExhausted)?;
        let mut depth = left
            .multiplicative_depth
            .max(right.multiplicative_depth)
            .checked_add(1)
            .ok_or(Error::ArithmeticOverflow)?;

        if let Some(profile) = env
            .storage()
            .persistent()
            .get::<_, PerformanceProfile>(&DataKey::Profile(left.context_id.clone()))
        {
            if profile.lazy_relinearization {
                depth = depth.saturating_sub(1);
            }
            if profile.auto_bootstrap && noise <= profile.bootstrap_threshold {
                noise = Self::DEFAULT_NOISE_BUDGET;
            }
        }

        Self::store_ciphertext(
            &env,
            submitter,
            output_id,
            left.context_id,
            left.scheme,
            out_slots,
            left.scale.max(right.scale),
            noise,
            depth,
        )
    }

    pub fn bootstrap_ciphertext(
        env: Env,
        admin: Address,
        ciphertext_id: BytesN<32>,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_initialized(&env)?;
        Self::require_admin(&env, &admin)?;

        let mut ct = Self::load_ciphertext(&env, &ciphertext_id)?;
        ct.noise_budget = Self::DEFAULT_NOISE_BUDGET;
        ct.last_bootstrapped_at = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::Ciphertext(ciphertext_id.clone()), &ct);
        env.events()
            .publish((symbol_short!("he"), symbol_short!("boot")), ciphertext_id);
        Ok(())
    }

    pub fn get_ciphertext(
        env: Env,
        ciphertext_id: BytesN<32>,
    ) -> Result<Option<EncryptedVector>, Error> {
        Self::require_initialized(&env)?;
        Ok(env
            .storage()
            .persistent()
            .get(&DataKey::Ciphertext(ciphertext_id)))
    }

    pub fn encrypted_statistics(
        env: Env,
        submitter: Address,
        ciphertext_id: BytesN<32>,
    ) -> Result<EncryptedStats, Error> {
        submitter.require_auth();
        Self::require_initialized(&env)?;
        let ct = Self::load_ciphertext(&env, &ciphertext_id)?;
        if ct.slots.is_empty() {
            return Err(Error::InvalidInput);
        }

        let count = ct.slots.len();
        let mut sum = 0i128;
        let mut min = ct.slots.get(0).ok_or(Error::InvalidInput)?;
        let mut max = min;

        for i in 0..count {
            let v = ct.slots.get(i).ok_or(Error::InvalidInput)?;
            sum = sum.checked_add(v).ok_or(Error::ArithmeticOverflow)?;
            if v < min {
                min = v;
            }
            if v > max {
                max = v;
            }
        }

        let denom = i128::from(count);
        let mean = sum.checked_div(denom).ok_or(Error::InvalidInput)?;

        let mut var_acc = 0i128;
        for i in 0..count {
            let v = ct.slots.get(i).ok_or(Error::InvalidInput)?;
            let d = v.checked_sub(mean).ok_or(Error::ArithmeticOverflow)?;
            var_acc = var_acc
                .checked_add(d.checked_mul(d).ok_or(Error::ArithmeticOverflow)?)
                .ok_or(Error::ArithmeticOverflow)?;
        }
        let variance = var_acc.checked_div(denom).ok_or(Error::InvalidInput)?;

        Ok(EncryptedStats {
            ciphertext_id,
            count,
            sum,
            mean_scaled: mean,
            variance_scaled: variance,
            min,
            max,
        })
    }

    pub fn encrypted_linear_inference(
        env: Env,
        submitter: Address,
        output_id: BytesN<32>,
        features_id: BytesN<32>,
        model_weights: Vec<i128>,
        bias: i128,
    ) -> Result<(), Error> {
        submitter.require_auth();
        Self::require_initialized(&env)?;
        let ct = Self::load_ciphertext(&env, &features_id)?;

        if ct.slots.len() != model_weights.len() || ct.slots.is_empty() {
            return Err(Error::IncompatibleDimensions);
        }

        let mut acc = bias;
        for i in 0..ct.slots.len() {
            let x = ct.slots.get(i).ok_or(Error::InvalidInput)?;
            let w = model_weights.get(i).ok_or(Error::InvalidInput)?;
            let prod = x.checked_mul(w).ok_or(Error::ArithmeticOverflow)?;
            acc = acc.checked_add(prod).ok_or(Error::ArithmeticOverflow)?;
        }

        let mut out = Vec::new(&env);
        out.push_back(acc);
        let noise = ct
            .noise_budget
            .checked_sub(4)
            .ok_or(Error::NoiseBudgetExhausted)?;
        let depth = ct
            .multiplicative_depth
            .checked_add(1)
            .ok_or(Error::ArithmeticOverflow)?;

        Self::store_ciphertext(
            &env,
            submitter,
            output_id,
            ct.context_id,
            ct.scheme,
            out,
            ct.scale,
            noise,
            depth,
        )
    }

    pub fn estimate_operation_cost(
        env: Env,
        context_id: BytesN<32>,
        multiplicative_depth: u32,
        slot_count: u32,
    ) -> Result<u64, Error> {
        Self::require_initialized(&env)?;
        Self::require_active_context(&env, &context_id)?;
        if slot_count == 0 {
            return Err(Error::InvalidInput);
        }

        let depth_cost = u64::from(multiplicative_depth)
            .checked_mul(250)
            .ok_or(Error::ArithmeticOverflow)?;
        let slot_cost = u64::from(slot_count)
            .checked_mul(20)
            .ok_or(Error::ArithmeticOverflow)?;
        let mut cost = 1000u64
            .checked_add(depth_cost)
            .and_then(|v| v.checked_add(slot_cost))
            .ok_or(Error::ArithmeticOverflow)?;

        if let Some(profile) = env
            .storage()
            .persistent()
            .get::<_, PerformanceProfile>(&DataKey::Profile(context_id))
        {
            if profile.batching_enabled {
                cost = cost.saturating_mul(80) / 100;
            }
            if profile.lazy_relinearization {
                cost = cost.saturating_mul(90) / 100;
            }
        }
        Ok(cost)
    }

    pub fn register_context(
        env: Env,
        admin: Address,
        context_id: BytesN<32>,
        scheme: HEScheme,
        params_ref: String,
        params_hash: BytesN<32>,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_initialized(&env)?;
        Self::require_admin(&env, &admin)?;

        if params_ref.is_empty() {
            return Err(Error::InvalidInput);
        }

        let ctx = HEContext {
            context_id: context_id.clone(),
            scheme,
            params_ref,
            params_hash,
            created_at: env.ledger().timestamp(),
            is_active: true,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Context(context_id.clone()), &ctx);
        env.events().publish(
            (symbol_short!("he"), symbol_short!("ctx")),
            (context_id, ctx.created_at),
        );
        Ok(())
    }

    pub fn deactivate_context(
        env: Env,
        admin: Address,
        context_id: BytesN<32>,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_initialized(&env)?;
        Self::require_admin(&env, &admin)?;

        let mut ctx: HEContext = env
            .storage()
            .persistent()
            .get(&DataKey::Context(context_id.clone()))
            .ok_or(Error::ContextNotFound)?;
        ctx.is_active = false;
        env.storage()
            .persistent()
            .set(&DataKey::Context(context_id.clone()), &ctx);
        env.events()
            .publish((symbol_short!("he"), symbol_short!("ctx_off")), context_id);
        Ok(())
    }

    pub fn submit_encrypted_computation(
        env: Env,
        submitter: Address,
        computation_id: BytesN<32>,
        context_id: BytesN<32>,
        ciphertext_ref: String,
        ciphertext_hash: BytesN<32>,
        proof_ref: String,
        proof_hash: BytesN<32>,
    ) -> Result<(), Error> {
        submitter.require_auth();
        Self::require_initialized(&env)?;

        if env
            .storage()
            .persistent()
            .has(&DataKey::Computation(computation_id.clone()))
        {
            return Err(Error::ComputationAlreadyExists);
        }

        let ctx: HEContext = env
            .storage()
            .persistent()
            .get(&DataKey::Context(context_id.clone()))
            .ok_or(Error::ContextNotFound)?;
        if !ctx.is_active {
            return Err(Error::ContextInactive);
        }
        if ciphertext_ref.is_empty() {
            return Err(Error::InvalidInput);
        }

        let zero_hash = BytesN::from_array(&env, &[0u8; 32]);
        if proof_ref.is_empty() {
            // No proof: require the sentinel hash.
            if proof_hash != zero_hash {
                return Err(Error::InvalidInput);
            }
        } else if proof_hash == zero_hash {
            // Proof supplied: require a non-zero hash anchor.
            return Err(Error::InvalidInput);
        }

        let item = EncryptedComputation {
            computation_id: computation_id.clone(),
            context_id,
            submitter: submitter.clone(),
            ciphertext_ref,
            ciphertext_hash,
            proof_ref,
            proof_hash,
            submitted_at: env.ledger().timestamp(),
        };
        env.storage()
            .persistent()
            .set(&DataKey::Computation(computation_id.clone()), &item);
        env.events().publish(
            (symbol_short!("he"), symbol_short!("submit")),
            (submitter, computation_id),
        );
        Ok(())
    }

    pub fn get_context(env: Env, context_id: BytesN<32>) -> Result<Option<HEContext>, Error> {
        Self::require_initialized(&env)?;
        Ok(env
            .storage()
            .persistent()
            .get(&DataKey::Context(context_id)))
    }

    pub fn get_computation(
        env: Env,
        computation_id: BytesN<32>,
    ) -> Result<Option<EncryptedComputation>, Error> {
        Self::require_initialized(&env)?;
        Ok(env
            .storage()
            .persistent()
            .get(&DataKey::Computation(computation_id)))
    }

    // -------------------------------------------------------------------------
    // Internal helpers
    // -------------------------------------------------------------------------

    fn require_initialized(env: &Env) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Initialized) {
            Ok(())
        } else {
            Err(Error::NotInitialized)
        }
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if &admin != caller {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    fn require_active_context(env: &Env, context_id: &BytesN<32>) -> Result<HEContext, Error> {
        let ctx: HEContext = env
            .storage()
            .persistent()
            .get(&DataKey::Context(context_id.clone()))
            .ok_or(Error::ContextNotFound)?;
        if !ctx.is_active {
            return Err(Error::ContextInactive);
        }
        Ok(ctx)
    }

    fn store_ciphertext(
        env: &Env,
        owner: Address,
        ciphertext_id: BytesN<32>,
        context_id: BytesN<32>,
        scheme: HEScheme,
        slots: Vec<i128>,
        scale: u32,
        noise_budget: u32,
        multiplicative_depth: u32,
    ) -> Result<(), Error> {
        if slots.is_empty() {
            return Err(Error::InvalidInput);
        }
        if env
            .storage()
            .persistent()
            .has(&DataKey::Ciphertext(ciphertext_id.clone()))
        {
            return Err(Error::CiphertextAlreadyExists);
        }
        if noise_budget == 0 {
            return Err(Error::NoiseBudgetExhausted);
        }

        let ct = EncryptedVector {
            ciphertext_id: ciphertext_id.clone(),
            context_id,
            owner,
            scheme,
            scale,
            noise_budget,
            multiplicative_depth,
            slots,
            created_at: env.ledger().timestamp(),
            last_bootstrapped_at: 0,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Ciphertext(ciphertext_id.clone()), &ct);
        env.events()
            .publish((symbol_short!("he"), symbol_short!("ct")), ciphertext_id);
        Ok(())
    }

    fn load_ciphertext(env: &Env, ciphertext_id: &BytesN<32>) -> Result<EncryptedVector, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Ciphertext(ciphertext_id.clone()))
            .ok_or(Error::CiphertextNotFound)
    }

    fn require_binary_compatible(
        left: &EncryptedVector,
        right: &EncryptedVector,
    ) -> Result<(), Error> {
        if left.context_id != right.context_id || left.scheme != right.scheme {
            return Err(Error::SchemeMismatch);
        }
        if left.slots.len() != right.slots.len() {
            return Err(Error::IncompatibleDimensions);
        }
        Ok(())
    }

    fn add_vectors(env: &Env, left: &Vec<i128>, right: &Vec<i128>) -> Result<Vec<i128>, Error> {
        let mut out = Vec::new(env);
        for i in 0..left.len() {
            let l = left.get(i).ok_or(Error::InvalidInput)?;
            let r = right.get(i).ok_or(Error::InvalidInput)?;
            out.push_back(l.checked_add(r).ok_or(Error::ArithmeticOverflow)?);
        }
        Ok(out)
    }

    fn mul_vectors(env: &Env, left: &Vec<i128>, right: &Vec<i128>) -> Result<Vec<i128>, Error> {
        let mut out = Vec::new(env);
        for i in 0..left.len() {
            let l = left.get(i).ok_or(Error::InvalidInput)?;
            let r = right.get(i).ok_or(Error::InvalidInput)?;
            out.push_back(l.checked_mul(r).ok_or(Error::ArithmeticOverflow)?);
        }
        Ok(out)
    }
}
