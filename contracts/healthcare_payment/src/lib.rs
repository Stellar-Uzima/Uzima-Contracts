#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short,
    token::Client as TokenClient, Address, Env, IntoVal, String, Symbol, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    ClaimNotFound = 4,
    InvalidStatus = 5,
    PreAuthNotFound = 6,
    PaymentPlanNotFound = 7,
    InsufficientFunds = 8,
    FraudDetected = 9,
    EscrowFailed = 10,
    InvalidAmount = 11,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ClaimStatus {
    Submitted = 0,
    Verified = 1,
    Approved = 2,
    Rejected = 3,
    Paid = 4,
    Disputed = 5,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum PreAuthStatus {
    Pending = 0,
    Approved = 1,
    Denied = 2,
    Expired = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum PaymentPlanStatus {
    Active = 0,
    Completed = 1,
    Defaulted = 2,
    Cancelled = 3,
}

#[derive(Clone)]
#[contracttype]
pub struct Claim {
    pub id: u64,
    pub patient: Address,
    pub provider: Address,
    pub service_id: String,
    pub amount: i128,
    pub status: ClaimStatus,
    pub policy_id: String,
    pub preauth_id: Option<u64>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct PreAuth {
    pub id: u64,
    pub patient: Address,
    pub provider: Address,
    pub service_id: String,
    pub estimated_cost: i128,
    pub status: PreAuthStatus,
    pub expiry: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct PaymentPlan {
    pub id: u64,
    pub patient: Address,
    pub provider: Address,
    pub total_amount: i128,
    pub remaining_amount: i128,
    pub installment_amount: i128,
    pub frequency: u64,
    pub next_due: u64,
    pub status: PaymentPlanStatus,
}

#[derive(Clone)]
#[contracttype]
pub struct FraudReport {
    pub claim_id: u64,
    pub reporter: Address,
    pub reason: String,
    pub timestamp: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct Config {
    pub admin: Address,
    pub payment_router: Address,
    pub escrow_contract: Address,
    pub treasury: Address,
    pub token: Address,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Config,
    ClaimCount,
    Claim(u64),
    PreAuthCount,
    PreAuth(u64),
    PaymentPlanCount,
    PaymentPlan(u64),
    FraudReport(u64),
}

#[contract]
pub struct HealthcarePayment;

#[contractimpl]
impl HealthcarePayment {
    pub fn initialize(
        env: Env,
        admin: Address,
        payment_router: Address,
        escrow_contract: Address,
        treasury: Address,
        token: Address,
    ) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Config) {
            return Err(Error::AlreadyInitialized);
        }

        let config = Config {
            admin,
            payment_router,
            escrow_contract,
            treasury,
            token,
        };

        env.storage().instance().set(&DataKey::Config, &config);
        env.storage().instance().set(&DataKey::ClaimCount, &0u64);
        env.storage().instance().set(&DataKey::PreAuthCount, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::PaymentPlanCount, &0u64);

        Ok(())
    }

    pub fn submit_claim(
        env: Env,
        patient: Address,
        provider: Address,
        service_id: String,
        amount: i128,
        policy_id: String,
        preauth_id: Option<u64>,
    ) -> Result<u64, Error> {
        provider.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let mut claim_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::ClaimCount)
            .unwrap_or(0);
        claim_id += 1;

        let current_time = env.ledger().timestamp();

        let claim = Claim {
            id: claim_id,
            patient,
            provider: provider.clone(),
            service_id,
            amount,
            status: ClaimStatus::Submitted,
            policy_id,
            preauth_id,
            created_at: current_time,
            updated_at: current_time,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Claim(claim_id), &claim);
        env.storage()
            .instance()
            .set(&DataKey::ClaimCount, &claim_id);

        env.events()
            .publish((symbol_short!("CLAIM_SUB"),), (claim_id, provider, amount));

        Ok(claim_id)
    }

    pub fn verify_claim(env: Env, claim_id: u64, verifier: Address) -> Result<(), Error> {
        verifier.require_auth();

        let mut claim: Claim = env
            .storage()
            .persistent()
            .get(&DataKey::Claim(claim_id))
            .ok_or(Error::ClaimNotFound)?;

        if claim.status != ClaimStatus::Submitted {
            return Err(Error::InvalidStatus);
        }

        claim.status = ClaimStatus::Verified;
        claim.updated_at = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::Claim(claim_id), &claim);
        env.events()
            .publish((symbol_short!("CLAIM_VER"),), (claim_id, verifier));

        Ok(())
    }

    pub fn approve_claim(env: Env, claim_id: u64, approver: Address) -> Result<(), Error> {
        approver.require_auth();

        let mut claim: Claim = env
            .storage()
            .persistent()
            .get(&DataKey::Claim(claim_id))
            .ok_or(Error::ClaimNotFound)?;

        if env
            .storage()
            .persistent()
            .has(&DataKey::FraudReport(claim_id))
        {
            return Err(Error::FraudDetected);
        }

        if claim.status != ClaimStatus::Verified {
            return Err(Error::InvalidStatus);
        }

        claim.status = ClaimStatus::Approved;
        claim.updated_at = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::Claim(claim_id), &claim);
        env.events()
            .publish((symbol_short!("CLAIM_APP"),), (claim_id, approver));

        Ok(())
    }

    pub fn reject_claim(
        env: Env,
        claim_id: u64,
        rejector: Address,
        reason: String,
    ) -> Result<(), Error> {
        rejector.require_auth();

        let mut claim: Claim = env
            .storage()
            .persistent()
            .get(&DataKey::Claim(claim_id))
            .ok_or(Error::ClaimNotFound)?;

        claim.status = ClaimStatus::Rejected;
        claim.updated_at = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::Claim(claim_id), &claim);
        env.events()
            .publish((symbol_short!("CLAIM_REJ"),), (claim_id, rejector, reason));

        Ok(())
    }

    pub fn process_payment(env: Env, claim_id: u64) -> Result<(), Error> {
        let config: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;
        let mut claim: Claim = env
            .storage()
            .persistent()
            .get(&DataKey::Claim(claim_id))
            .ok_or(Error::ClaimNotFound)?;

        if claim.status != ClaimStatus::Approved {
            return Err(Error::InvalidStatus);
        }

        let (provider_amount, fee_amount): (i128, i128) = env.invoke_contract(
            &config.payment_router,
            &Symbol::new(&env, "compute_split"),
            Vec::from_array(&env, [claim.amount.into_val(&env)]),
        );

        let token_client = TokenClient::new(&env, &config.token);

        token_client.transfer(
            &env.current_contract_address(),
            &claim.provider,
            &provider_amount,
        );

        if fee_amount > 0 {
            token_client.transfer(
                &env.current_contract_address(),
                &config.treasury,
                &fee_amount,
            );
        }

        claim.status = ClaimStatus::Paid;
        claim.updated_at = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::Claim(claim_id), &claim);
        env.events().publish(
            (symbol_short!("CLAIM_PD"),),
            (claim_id, claim.provider, provider_amount),
        );

        Ok(())
    }

    pub fn escrow_claim(env: Env, claim_id: u64) -> Result<(), Error> {
        let config: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;
        let mut claim: Claim = env
            .storage()
            .persistent()
            .get(&DataKey::Claim(claim_id))
            .ok_or(Error::ClaimNotFound)?;

        if claim.status != ClaimStatus::Approved && claim.status != ClaimStatus::Disputed {
            return Err(Error::InvalidStatus);
        }

        let token_client = TokenClient::new(&env, &config.token);

        token_client.approve(
            &env.current_contract_address(),
            &config.escrow_contract,
            &claim.amount,
            &(env.ledger().sequence() + 1000),
        );

        let escrow_args = Vec::from_array(
            &env,
            [
                claim_id.into_val(&env),
                env.current_contract_address().into_val(&env),
                claim.provider.clone().into_val(&env),
                claim.amount.into_val(&env),
                config.token.into_val(&env),
            ],
        );

        let escrow_created: bool = env.invoke_contract(
            &config.escrow_contract,
            &Symbol::new(&env, "create_escrow"),
            escrow_args,
        );

        if !escrow_created {
            return Err(Error::EscrowFailed);
        }

        claim.status = ClaimStatus::Paid;
        claim.updated_at = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::Claim(claim_id), &claim);

        Ok(())
    }

    pub fn request_preauth(
        env: Env,
        patient: Address,
        provider: Address,
        service_id: String,
        estimated_cost: i128,
    ) -> Result<u64, Error> {
        provider.require_auth();

        let mut preauth_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PreAuthCount)
            .unwrap_or(0);
        preauth_id += 1;

        let preauth = PreAuth {
            id: preauth_id,
            patient,
            provider,
            service_id,
            estimated_cost,
            status: PreAuthStatus::Pending,
            expiry: env.ledger().timestamp() + 604800,
        };

        env.storage()
            .persistent()
            .set(&DataKey::PreAuth(preauth_id), &preauth);
        env.storage()
            .instance()
            .set(&DataKey::PreAuthCount, &preauth_id);

        Ok(preauth_id)
    }

    pub fn approve_preauth(env: Env, preauth_id: u64, approver: Address) -> Result<(), Error> {
        approver.require_auth();

        let mut preauth: PreAuth = env
            .storage()
            .persistent()
            .get(&DataKey::PreAuth(preauth_id))
            .ok_or(Error::PreAuthNotFound)?;

        preauth.status = PreAuthStatus::Approved;
        env.storage()
            .persistent()
            .set(&DataKey::PreAuth(preauth_id), &preauth);

        Ok(())
    }

    pub fn report_fraud(
        env: Env,
        claim_id: u64,
        reporter: Address,
        reason: String,
    ) -> Result<(), Error> {
        reporter.require_auth();

        let report = FraudReport {
            claim_id,
            reporter: reporter.clone(),
            reason: reason.clone(),
            timestamp: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::FraudReport(claim_id), &report);

        if let Some(mut claim) = env
            .storage()
            .persistent()
            .get::<_, Claim>(&DataKey::Claim(claim_id))
        {
            if claim.status != ClaimStatus::Paid {
                claim.status = ClaimStatus::Disputed;
                env.storage()
                    .persistent()
                    .set(&DataKey::Claim(claim_id), &claim);
            }
        }

        env.events()
            .publish((symbol_short!("FRAUD"),), (claim_id, reporter, reason));

        Ok(())
    }

    pub fn create_payment_plan(
        env: Env,
        patient: Address,
        provider: Address,
        total_amount: i128,
        installment_amount: i128,
        frequency: u64,
    ) -> Result<u64, Error> {
        patient.require_auth();

        let mut plan_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PaymentPlanCount)
            .unwrap_or(0);
        plan_id += 1;

        let plan = PaymentPlan {
            id: plan_id,
            patient,
            provider,
            total_amount,
            remaining_amount: total_amount,
            installment_amount,
            frequency,
            next_due: env.ledger().timestamp() + frequency,
            status: PaymentPlanStatus::Active,
        };

        env.storage()
            .persistent()
            .set(&DataKey::PaymentPlan(plan_id), &plan);
        env.storage()
            .instance()
            .set(&DataKey::PaymentPlanCount, &plan_id);

        Ok(plan_id)
    }

    pub fn pay_installment(env: Env, plan_id: u64) -> Result<(), Error> {
        let mut plan: PaymentPlan = env
            .storage()
            .persistent()
            .get(&DataKey::PaymentPlan(plan_id))
            .ok_or(Error::PaymentPlanNotFound)?;

        if plan.status != PaymentPlanStatus::Active {
            return Err(Error::InvalidStatus);
        }

        let amount_to_pay = if plan.remaining_amount < plan.installment_amount {
            plan.remaining_amount
        } else {
            plan.installment_amount
        };

        let config: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;
        let token_client = TokenClient::new(&env, &config.token);

        token_client.transfer_from(
            &env.current_contract_address(),
            &plan.patient,
            &plan.provider,
            &amount_to_pay,
        );

        plan.remaining_amount -= amount_to_pay;
        plan.next_due += plan.frequency;

        if plan.remaining_amount == 0 {
            plan.status = PaymentPlanStatus::Completed;
        }

        env.storage()
            .persistent()
            .set(&DataKey::PaymentPlan(plan_id), &plan);

        Ok(())
    }
}

#[cfg(test)]
mod test;
