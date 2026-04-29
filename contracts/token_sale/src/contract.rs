// Token sale contract
use crate::errors::Error;
use crate::storage::*;
use crate::types::*;
use soroban_sdk::{contract, contractimpl, contractmeta, token, Address, Env};
extern crate fp_math;

// Metadata that is added on to every WASM custom section
contractmeta!(
    key = "Description",
    val = "Capped Token Sale Contract with Vesting"
);

#[contract]
pub struct TokenSaleContract;

#[contractimpl]
impl TokenSaleContract {
    /// Initialize the token sale contract
    pub fn initialize(
        env: Env,
        owner: Address,
        token_address: Address,
        treasury: Address,
        soft_cap: u128,
        hard_cap: u128,
        token_decimals: u32,
    ) {
        if env.storage().instance().has(&DataKey::Config) {
            return; // Already initialized - early return instead of panic
        }

        owner.require_auth();

        assert!(soft_cap <= hard_cap, "Soft cap must be <= hard cap");
        assert!(token_decimals <= 18, "Decimals must be <= 18");

        let config = SaleConfig {
            token_address: token_address.clone(),
            treasury: treasury.clone(),
            soft_cap,
            hard_cap,
            token_decimals,
            is_finalized: false,
            refunds_enabled: false,
        };

        set_config(&env, &config);
        set_owner(&env, &owner);
        set_paused(&env, false);
        set_total_raised(&env, 0);
        set_phase_count(&env, 0);

        env.events().publish(
            ("sale_initialized",),
            (token_address, treasury, soft_cap, hard_cap),
        );
    }

    /// Add a new sale phase
    pub fn add_sale_phase(
        env: Env,
        start_time: u64,
        end_time: u64,
        price_per_token: u128,
        max_tokens: u128,
        per_address_cap: u128,
    ) {
        let owner = get_owner(&env);
        owner.require_auth();

        assert!(start_time < end_time, "Invalid time range");
        assert!(price_per_token > 0, "Price must be > 0");
        assert!(max_tokens > 0, "Max tokens must be > 0");

        let phase_id = get_phase_count(&env);
        let new_phase = SalePhase {
            start_time,
            end_time,
            price_per_token,
            max_tokens,
            sold_tokens: 0,
            per_address_cap,
            is_active: true,
        };

        set_sale_phase(&env, phase_id, &new_phase);
        set_phase_count(&env, phase_id + 1);

        env.events().publish(
            ("phase_added",),
            (phase_id, start_time, end_time, price_per_token, max_tokens),
        );
    }

    /// Add supported payment token
    pub fn add_supported_token(env: Env, token: Address) {
        let owner = get_owner(&env);
        owner.require_auth();

        set_supported_token(&env, &token, true);

        env.events().publish(("token_added",), (token,));
    }

    /// Pause the sale
    pub fn pause_sale(env: Env) {
        let owner = get_owner(&env);
        owner.require_auth();

        set_paused(&env, true);
        env.events().publish(("sale_paused",), ());
    }

    /// Unpause the sale
    pub fn unpause_sale(env: Env) {
        let owner = get_owner(&env);
        owner.require_auth();

        set_paused(&env, false);
        env.events().publish(("sale_unpaused",), ());
    }

    /// Emergency withdraw tokens
    pub fn emergency_withdraw(env: Env, token: Address, amount: u128) {
        let owner = get_owner(&env);
        owner.require_auth();

        let config = get_config(&env);
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(
            &env.current_contract_address(),
            &config.treasury,
            &(amount as i128),
        );

        env.events()
            .publish(("emergency_withdraw",), (token, amount));
    }

    /// Contribute to the sale using ERC20 tokens
    pub fn contribute(
        env: Env,
        contributor: Address,
        phase_id: u32,
        token: Address,
        amount: u128,
    ) -> Result<(), Error> {
        contributor.require_auth();

        if is_paused(&env) {
            return Err(Error::Paused);
        }
        let config = get_config(&env);
        if config.is_finalized {
            return Err(Error::PhaseClosed);
        }
        if !is_supported_token(&env, &token) {
            return Err(Error::InvalidArgument);
        }

        let current_time = get_ledger_timestamp(&env);
        let Some(mut phase) = get_sale_phase(&env, phase_id) else {
            return Err(Error::PhaseNotFound);
        };

        if !phase.is_active || current_time < phase.start_time || current_time > phase.end_time {
            return Err(Error::PhaseClosed);
        }

        let tokens_to_allocate =
            fp_math::tokens_for_payment(amount, phase.price_per_token, config.token_decimals)
                .expect("token allocation overflow");

        let new_sold = phase
            .sold_tokens
            .checked_add(tokens_to_allocate)
            .ok_or(Error::Overflow)?;
        if new_sold > phase.max_tokens {
            return Err(Error::CapExceeded);
        }

        let user_phase_contribution = get_phase_contribution(&env, &contributor, phase_id);
        let new_contribution = user_phase_contribution
            .checked_add(amount)
            .ok_or(Error::Overflow)?;
        if new_contribution > phase.per_address_cap {
            return Err(Error::CapExceeded);
        }

        let token_client = token::Client::new(&env, &token);
        token_client.transfer(
            &contributor,
            &env.current_contract_address(),
            &(amount as i128),
        );

        phase.sold_tokens = new_sold;
        set_sale_phase(&env, phase_id, &phase);

        set_phase_contribution(&env, &contributor, phase_id, new_contribution);

        let new_total = get_total_raised(&env)
            .checked_add(amount)
            .ok_or(Error::Overflow)?;
        set_total_raised(&env, new_total);

        let mut contribution = get_contribution(&env, &contributor).unwrap_or(Contribution {
            amount: 0,
            tokens_allocated: 0,
            phase_id,
            timestamp: current_time,
            claimed: false,
        });

        contribution.amount = contribution
            .amount
            .checked_add(amount)
            .ok_or(Error::Overflow)?;
        contribution.tokens_allocated = contribution
            .tokens_allocated
            .checked_add(tokens_to_allocate)
            .ok_or(Error::Overflow)?;
        contribution.timestamp = current_time;
        set_contribution(&env, &contributor, &contribution);

        env.events().publish(
            ("contribution",),
            (contributor, phase_id, amount, tokens_to_allocate),
        );

        Ok(())
    }

    /// Finalize the sale
    pub fn finalize_sale(env: Env) {
        let owner = get_owner(&env);
        owner.require_auth();

        let mut config = get_config(&env);
        assert!(!config.is_finalized, "Sale already finalized");

        let total_raised = get_total_raised(&env);
        let success = total_raised >= config.soft_cap;

        config.is_finalized = true;
        if !success {
            config.refunds_enabled = true;
        }

        set_config(&env, &config);

        env.events()
            .publish(("sale_finalized",), (total_raised, success));
    }

    /// Claim allocated tokens
    pub fn claim_tokens(env: Env, claimer: Address) {
        claimer.require_auth();

        let config = get_config(&env);
        assert!(config.is_finalized, "Sale not finalized");
        assert!(
            !config.refunds_enabled,
            "Refunds enabled, cannot claim tokens"
        );

        let Some(mut contribution) = get_contribution(&env, &claimer) else {
            return; // No contribution found
        };
        assert!(!contribution.claimed, "Tokens already claimed");
        assert!(contribution.tokens_allocated > 0, "No tokens to claim");

        contribution.claimed = true;
        set_contribution(&env, &claimer, &contribution);

        let token_client = token::Client::new(&env, &config.token_address);
        token_client.transfer(
            &env.current_contract_address(),
            &claimer,
            &(contribution.tokens_allocated as i128),
        );

        env.events().publish(
            ("tokens_claimed",),
            (claimer, contribution.tokens_allocated),
        );
    }

    /// Claim refund if sale failed
    pub fn claim_refund(env: Env, claimer: Address, payment_token: Address) {
        claimer.require_auth();

        let config = get_config(&env);
        assert!(config.refunds_enabled, "Refunds not enabled");

        let Some(mut contribution) = get_contribution(&env, &claimer) else {
            return; // No contribution found
        };
        assert!(!contribution.claimed, "Already claimed");
        assert!(contribution.amount > 0, "No contribution to refund");

        contribution.claimed = true;
        set_contribution(&env, &claimer, &contribution);

        let token_client = token::Client::new(&env, &payment_token);
        token_client.transfer(
            &env.current_contract_address(),
            &claimer,
            &(contribution.amount as i128),
        );

        env.events()
            .publish(("refund_claimed",), (claimer, contribution.amount));
    }

    // View functions
    pub fn get_sale_phase(env: Env, phase_id: u32) -> Option<SalePhase> {
        get_sale_phase(&env, phase_id)
    }

    pub fn get_contribution(env: Env, user: Address) -> Option<Contribution> {
        get_contribution(&env, &user)
    }

    pub fn get_config(env: Env) -> SaleConfig {
        get_config(&env)
    }

    pub fn get_total_raised(env: Env) -> u128 {
        get_total_raised(&env)
    }

    pub fn is_sale_finalized(env: Env) -> bool {
        get_config(&env).is_finalized
    }

    pub fn get_current_phase(env: Env) -> Option<u32> {
        let current_time = get_ledger_timestamp(&env);
        let phase_count = get_phase_count(&env);

        for i in 0..phase_count {
            if let Some(phase) = get_sale_phase(&env, i) {
                if phase.is_active
                    && current_time >= phase.start_time
                    && current_time <= phase.end_time
                {
                    return Some(i);
                }
            }
        }

        None
    }

    pub fn get_claimable_tokens(env: Env, user: Address) -> u128 {
        let config = get_config(&env);
        if !config.is_finalized || config.refunds_enabled {
            return 0;
        }

        if let Some(contribution) = get_contribution(&env, &user) {
            if !contribution.claimed {
                return contribution.tokens_allocated;
            }
        }

        0
    }
}
