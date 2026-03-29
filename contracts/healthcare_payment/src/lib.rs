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
    InsuranceProviderNotFound = 12,
    CoveragePolicyNotFound = 13,
    EligibilityCheckNotFound = 14,
    ClaimSubmissionNotFound = 15,
    EobNotFound = 16,
    InvalidCoverage = 17,
    UnsupportedTransaction = 18,
    PolicyMismatch = 19,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ClaimSubmissionStatus {
    Submitted = 0,
    Acknowledged = 1,
    Adjudicated = 2,
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
pub struct InsuranceProvider {
    pub id: u64,
    pub name: String,
    pub payer_code: String,
    pub supports_edi_837: bool,
    pub supports_edi_834: bool,
    pub active: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct CoveragePolicy {
    pub id: u64,
    pub patient: Address,
    pub insurance_provider_id: u64,
    pub policy_external_id: String,
    pub member_id: String,
    pub group_number: String,
    pub deductible_total: i128,
    pub deductible_met: i128,
    pub copay_amount: i128,
    pub coinsurance_bps: u32,
    pub coverage_active: bool,
    pub last_verified_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct EligibilityCheck {
    pub id: u64,
    pub policy_id: u64,
    pub service_id: String,
    pub eligible: bool,
    pub coverage_bps: u32,
    pub copay_amount: i128,
    pub deductible_remaining: i128,
    pub checked_at: u64,
    pub provider_ref: String,
}

#[derive(Clone)]
#[contracttype]
pub struct ClaimSubmission {
    pub claim_id: u64,
    pub policy_id: u64,
    pub submission_format: String,
    pub transaction_code: String,
    pub payer_ref: String,
    pub submitted_at: u64,
    pub status: ClaimSubmissionStatus,
}

#[derive(Clone)]
#[contracttype]
pub struct CoverageEnrollment {
    pub id: u64,
    pub policy_id: u64,
    pub transaction_code: String,
    pub enrollment_ref: String,
    pub synced_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct ExplanationOfBenefits {
    pub claim_id: u64,
    pub policy_id: u64,
    pub allowed_amount: i128,
    pub insurer_paid: i128,
    pub patient_responsibility: i128,
    pub deductible_applied: i128,
    pub copay_amount: i128,
    pub adjudication_notes: String,
    pub processed_at: u64,
    pub edi_transaction: String,
}

#[derive(Clone)]
#[contracttype]
pub struct PatientResponsibility {
    pub patient: Address,
    pub total_copay_tracked: i128,
    pub total_deductible_tracked: i128,
    pub total_patient_responsibility: i128,
    pub last_updated: u64,
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
    InsuranceProviderCount,
    InsuranceProvider(u64),
    CoveragePolicyCount,
    CoveragePolicy(u64),
    PolicyByExternalId(String),
    EligibilityCount,
    Eligibility(u64),
    LatestEligibilityByPolicy(u64),
    ClaimSubmission(u64),
    CoverageEnrollmentCount,
    CoverageEnrollment(u64),
    Eob(u64),
    PatientResponsibility(Address),
}

#[contract]
pub struct HealthcarePayment;

#[allow(clippy::too_many_arguments)]
#[contractimpl]
impl HealthcarePayment {
    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let config: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;
        if config.admin != *caller {
            return Err(Error::Unauthorized);
        }
        Ok(())
    }

    fn read_counter(env: &Env, key: &DataKey) -> u64 {
        env.storage().instance().get(key).unwrap_or(0u64)
    }

    fn next_counter(env: &Env, key: &DataKey) -> u64 {
        let next = Self::read_counter(env, key).saturating_add(1);
        env.storage().instance().set(key, &next);
        next
    }

    fn get_policy(env: &Env, policy_id: u64) -> Result<CoveragePolicy, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::CoveragePolicy(policy_id))
            .ok_or(Error::CoveragePolicyNotFound)
    }

    fn get_provider(env: &Env, provider_id: u64) -> Result<InsuranceProvider, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::InsuranceProvider(provider_id))
            .ok_or(Error::InsuranceProviderNotFound)
    }

    fn validate_positive_amount(amount: i128) -> Result<(), Error> {
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        Ok(())
    }

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
        env.storage()
            .instance()
            .set(&DataKey::InsuranceProviderCount, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::CoveragePolicyCount, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::EligibilityCount, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::CoverageEnrollmentCount, &0u64);

        Ok(())
    }

    pub fn register_insurance_provider(
        env: Env,
        caller: Address,
        name: String,
        payer_code: String,
        supports_edi_837: bool,
        supports_edi_834: bool,
    ) -> Result<u64, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        if name.is_empty() || payer_code.is_empty() {
            return Err(Error::InvalidCoverage);
        }

        let provider_id = Self::next_counter(&env, &DataKey::InsuranceProviderCount);
        let provider = InsuranceProvider {
            id: provider_id,
            name,
            payer_code,
            supports_edi_837,
            supports_edi_834,
            active: true,
        };

        env.storage()
            .persistent()
            .set(&DataKey::InsuranceProvider(provider_id), &provider);
        env.events()
            .publish((symbol_short!("INS_PROV"),), (provider_id, provider.active));

        Ok(provider_id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn register_coverage_policy(
        env: Env,
        caller: Address,
        patient: Address,
        insurance_provider_id: u64,
        policy_external_id: String,
        member_id: String,
        group_number: String,
        deductible_total: i128,
        copay_amount: i128,
        coinsurance_bps: u32,
    ) -> Result<u64, Error> {
        caller.require_auth();
        if caller != patient {
            Self::require_admin(&env, &caller)?;
        }

        let provider = Self::get_provider(&env, insurance_provider_id)?;
        if !provider.active
            || policy_external_id.is_empty()
            || member_id.is_empty()
            || deductible_total < 0
            || copay_amount < 0
            || coinsurance_bps > 10_000
        {
            return Err(Error::InvalidCoverage);
        }

        let policy_id = Self::next_counter(&env, &DataKey::CoveragePolicyCount);
        let policy = CoveragePolicy {
            id: policy_id,
            patient: patient.clone(),
            insurance_provider_id,
            policy_external_id: policy_external_id.clone(),
            member_id,
            group_number,
            deductible_total,
            deductible_met: 0,
            copay_amount,
            coinsurance_bps,
            coverage_active: true,
            last_verified_at: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::CoveragePolicy(policy_id), &policy);
        env.storage()
            .persistent()
            .set(&DataKey::PolicyByExternalId(policy_external_id), &policy_id);

        Ok(policy_id)
    }

    pub fn verify_insurance_eligibility(
        env: Env,
        caller: Address,
        policy_id: u64,
        service_id: String,
        coverage_bps: u32,
        provider_ref: String,
    ) -> Result<u64, Error> {
        caller.require_auth();

        let mut policy = Self::get_policy(&env, policy_id)?;
        if service_id.is_empty() || provider_ref.is_empty() || coverage_bps > 10_000 {
            return Err(Error::InvalidCoverage);
        }

        let check_id = Self::next_counter(&env, &DataKey::EligibilityCount);
        let deductible_remaining = policy
            .deductible_total
            .saturating_sub(policy.deductible_met)
            .max(0);
        let eligibility = EligibilityCheck {
            id: check_id,
            policy_id,
            service_id,
            eligible: policy.coverage_active && coverage_bps > 0,
            coverage_bps,
            copay_amount: policy.copay_amount,
            deductible_remaining,
            checked_at: env.ledger().timestamp(),
            provider_ref,
        };

        policy.last_verified_at = eligibility.checked_at;
        env.storage()
            .persistent()
            .set(&DataKey::CoveragePolicy(policy_id), &policy);
        env.storage()
            .persistent()
            .set(&DataKey::Eligibility(check_id), &eligibility);
        env.storage()
            .persistent()
            .set(&DataKey::LatestEligibilityByPolicy(policy_id), &check_id);

        env.events().publish(
            (symbol_short!("ELIG"),),
            (policy_id, eligibility.eligible, eligibility.coverage_bps),
        );

        Ok(check_id)
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
        Self::validate_positive_amount(amount)?;

        let claim_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::ClaimCount)
            .unwrap_or(0u64)
            .saturating_add(1);

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

    pub fn submit_insurance_claim(
        env: Env,
        caller: Address,
        claim_id: u64,
        coverage_policy_id: u64,
        payer_ref: String,
        transaction_code: String,
    ) -> Result<bool, Error> {
        caller.require_auth();
        if transaction_code != String::from_str(&env, "837") {
            return Err(Error::UnsupportedTransaction);
        }
        if payer_ref.is_empty() {
            return Err(Error::InvalidCoverage);
        }

        let claim: Claim = env
            .storage()
            .persistent()
            .get(&DataKey::Claim(claim_id))
            .ok_or(Error::ClaimNotFound)?;
        if claim.provider != caller {
            return Err(Error::Unauthorized);
        }

        let policy = Self::get_policy(&env, coverage_policy_id)?;
        let provider = Self::get_provider(&env, policy.insurance_provider_id)?;
        if !provider.supports_edi_837 {
            return Err(Error::UnsupportedTransaction);
        }
        if claim.policy_id != policy.policy_external_id {
            return Err(Error::PolicyMismatch);
        }

        let latest_eligibility_id: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::LatestEligibilityByPolicy(coverage_policy_id))
            .ok_or(Error::EligibilityCheckNotFound)?;
        let eligibility: EligibilityCheck = env
            .storage()
            .persistent()
            .get(&DataKey::Eligibility(latest_eligibility_id))
            .ok_or(Error::EligibilityCheckNotFound)?;
        if !eligibility.eligible {
            return Err(Error::InvalidCoverage);
        }

        let submission = ClaimSubmission {
            claim_id,
            policy_id: coverage_policy_id,
            submission_format: String::from_str(&env, "HIPAA"),
            transaction_code,
            payer_ref,
            submitted_at: env.ledger().timestamp(),
            status: ClaimSubmissionStatus::Submitted,
        };
        env.storage()
            .persistent()
            .set(&DataKey::ClaimSubmission(claim_id), &submission);

        env.events().publish(
            (symbol_short!("CLAIM_EDI"),),
            (claim_id, coverage_policy_id, submission.status as u32),
        );

        Ok(true)
    }

    pub fn sync_coverage_enrollment(
        env: Env,
        caller: Address,
        coverage_policy_id: u64,
        enrollment_ref: String,
        transaction_code: String,
    ) -> Result<u64, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        if transaction_code != String::from_str(&env, "834") || enrollment_ref.is_empty() {
            return Err(Error::UnsupportedTransaction);
        }

        let policy = Self::get_policy(&env, coverage_policy_id)?;
        let provider = Self::get_provider(&env, policy.insurance_provider_id)?;
        if !provider.supports_edi_834 {
            return Err(Error::UnsupportedTransaction);
        }

        let enrollment_id = Self::next_counter(&env, &DataKey::CoverageEnrollmentCount);
        let enrollment = CoverageEnrollment {
            id: enrollment_id,
            policy_id: coverage_policy_id,
            transaction_code,
            enrollment_ref,
            synced_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::CoverageEnrollment(enrollment_id), &enrollment);
        env.events().publish(
            (symbol_short!("COV_834"),),
            (coverage_policy_id, enrollment_id),
        );

        Ok(enrollment_id)
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

    #[allow(clippy::too_many_arguments)]
    pub fn process_eob(
        env: Env,
        caller: Address,
        claim_id: u64,
        coverage_policy_id: u64,
        allowed_amount: i128,
        insurer_paid: i128,
        deductible_applied: i128,
        adjudication_notes: String,
        edi_transaction: String,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        if edi_transaction != String::from_str(&env, "835")
            || allowed_amount < 0
            || insurer_paid < 0
            || deductible_applied < 0
        {
            return Err(Error::InvalidCoverage);
        }

        let claim: Claim = env
            .storage()
            .persistent()
            .get(&DataKey::Claim(claim_id))
            .ok_or(Error::ClaimNotFound)?;
        let mut policy = Self::get_policy(&env, coverage_policy_id)?;
        if claim.policy_id != policy.policy_external_id {
            return Err(Error::PolicyMismatch);
        }

        let copay_amount = policy.copay_amount;
        let patient_responsibility = allowed_amount
            .saturating_sub(insurer_paid)
            .saturating_add(copay_amount);

        let eob = ExplanationOfBenefits {
            claim_id,
            policy_id: coverage_policy_id,
            allowed_amount,
            insurer_paid,
            patient_responsibility,
            deductible_applied,
            copay_amount,
            adjudication_notes,
            processed_at: env.ledger().timestamp(),
            edi_transaction,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Eob(claim_id), &eob);

        policy.deductible_met = policy
            .deductible_met
            .saturating_add(deductible_applied)
            .min(policy.deductible_total);
        env.storage()
            .persistent()
            .set(&DataKey::CoveragePolicy(coverage_policy_id), &policy);

        let mut responsibility: PatientResponsibility = env
            .storage()
            .persistent()
            .get(&DataKey::PatientResponsibility(claim.patient.clone()))
            .unwrap_or(PatientResponsibility {
                patient: claim.patient.clone(),
                total_copay_tracked: 0,
                total_deductible_tracked: 0,
                total_patient_responsibility: 0,
                last_updated: 0,
            });
        responsibility.total_copay_tracked = responsibility
            .total_copay_tracked
            .saturating_add(copay_amount);
        responsibility.total_deductible_tracked = responsibility
            .total_deductible_tracked
            .saturating_add(deductible_applied);
        responsibility.total_patient_responsibility = responsibility
            .total_patient_responsibility
            .saturating_add(patient_responsibility);
        responsibility.last_updated = eob.processed_at;
        env.storage().persistent().set(
            &DataKey::PatientResponsibility(claim.patient),
            &responsibility,
        );

        if let Some(mut submission) = env
            .storage()
            .persistent()
            .get::<DataKey, ClaimSubmission>(&DataKey::ClaimSubmission(claim_id))
        {
            submission.status = ClaimSubmissionStatus::Adjudicated;
            env.storage()
                .persistent()
                .set(&DataKey::ClaimSubmission(claim_id), &submission);
        }

        env.events().publish(
            (symbol_short!("EOB"),),
            (claim_id, insurer_paid, patient_responsibility),
        );

        Ok(true)
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
            &env.ledger().sequence().saturating_add(1000),
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
        Self::validate_positive_amount(estimated_cost)?;

        let preauth_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PreAuthCount)
            .unwrap_or(0u64)
            .saturating_add(1);

        let preauth = PreAuth {
            id: preauth_id,
            patient,
            provider,
            service_id,
            estimated_cost,
            status: PreAuthStatus::Pending,
            expiry: env.ledger().timestamp().saturating_add(604800),
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
        Self::validate_positive_amount(total_amount)?;
        Self::validate_positive_amount(installment_amount)?;

        let plan_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PaymentPlanCount)
            .unwrap_or(0u64)
            .saturating_add(1);

        let plan = PaymentPlan {
            id: plan_id,
            patient,
            provider,
            total_amount,
            remaining_amount: total_amount,
            installment_amount,
            frequency,
            next_due: env.ledger().timestamp().saturating_add(frequency),
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

        plan.remaining_amount = plan.remaining_amount.saturating_sub(amount_to_pay);
        plan.next_due = plan.next_due.saturating_add(plan.frequency);

        if plan.remaining_amount == 0 {
            plan.status = PaymentPlanStatus::Completed;
        }

        env.storage()
            .persistent()
            .set(&DataKey::PaymentPlan(plan_id), &plan);

        Ok(())
    }

    pub fn get_coverage_policy(env: Env, coverage_policy_id: u64) -> Result<CoveragePolicy, Error> {
        Self::get_policy(&env, coverage_policy_id)
    }

    pub fn get_eligibility_check(env: Env, eligibility_id: u64) -> Result<EligibilityCheck, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Eligibility(eligibility_id))
            .ok_or(Error::EligibilityCheckNotFound)
    }

    pub fn get_claim_submission(env: Env, claim_id: u64) -> Result<ClaimSubmission, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::ClaimSubmission(claim_id))
            .ok_or(Error::ClaimSubmissionNotFound)
    }

    pub fn get_coverage_enrollment(
        env: Env,
        enrollment_id: u64,
    ) -> Result<CoverageEnrollment, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::CoverageEnrollment(enrollment_id))
            .ok_or(Error::UnsupportedTransaction)
    }

    pub fn get_explanation_of_benefits(
        env: Env,
        claim_id: u64,
    ) -> Result<ExplanationOfBenefits, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Eob(claim_id))
            .ok_or(Error::EobNotFound)
    }

    pub fn get_patient_responsibility(env: Env, patient: Address) -> Option<PatientResponsibility> {
        env.storage()
            .persistent()
            .get(&DataKey::PatientResponsibility(patient))
    }
}

#[cfg(test)]
mod test;
