#![no_std]
#![allow(clippy::too_many_arguments)]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
    String, Symbol,
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

#[contracttype]
pub enum DataKey {
    Initialized,
    Admin,
    Context(BytesN<32>),
    Computation(BytesN<32>),
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
}

// =============================================================================
// Contract
// =============================================================================

#[contract]
pub struct HomomorphicRegistry;

#[contractimpl]
impl HomomorphicRegistry {
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
}
