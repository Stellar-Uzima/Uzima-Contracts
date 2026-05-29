#![no_std]
#![allow(clippy::too_many_arguments)]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
    IntoVal, String, Symbol, Vec,
};

const MAX_SETTLEMENT_WINDOW_SECS: u64 = 300;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum DataFormat {
    FhirJson,
    Hl7,
    Dicom,
    Csv,
    Parquet,
    Custom,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum AnonymizationLevel {
    KAnonymity,
    DifferentialPrivacy,
    Synthetic,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ListingStatus {
    Active,
    Reserved,
    Settled,
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct QualityMetrics {
    pub completeness_bps: u32,
    pub consistency_bps: u32,
    pub timeliness_bps: u32,
    pub validity_bps: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct RoyaltyPolicy {
    pub provider_bps: u32,
    pub curator_bps: u32,
    pub platform_bps: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Config {
    pub admin: Address,
    pub payment_router: Address,
    pub escrow_contract: Address,
    pub treasury: Address,
    pub settlement_window_secs: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ProviderProfile {
    pub provider: Address,
    pub active: bool,
    pub listings_count: u64,
    pub reputation_bps: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Listing {
    pub id: u64,
    pub provider: Address,
    pub data_ref: String,
    pub data_hash: BytesN<32>,
    pub format: DataFormat,
    pub anonymization: AnonymizationLevel,
    pub min_k: u32,
    pub dp_epsilon_milli: u32,
    pub quality: QualityMetrics,
    pub royalty: RoyaltyPolicy,
    pub price: i128,
    pub token: Address,
    pub created_at: u64,
    pub status: ListingStatus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ListingPayload {
    pub data_ref: String,
    pub data_hash: BytesN<32>,
    pub format: DataFormat,
    pub anonymization: AnonymizationLevel,
    pub min_k: u32,
    pub dp_epsilon_milli: u32,
    pub quality: QualityMetrics,
    pub royalty: RoyaltyPolicy,
    pub price: i128,
    pub token: Address,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct PurchaseIntent {
    pub id: u64,
    pub listing_id: u64,
    pub buyer: Address,
    pub amount: i128,
    pub created_at: u64,
    pub escrow_order_id: Option<u64>,
    pub settled: bool,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Config,
    ProviderCount,
    Provider(Address),
    NextListingId,
    Listing(u64),
    NextIntentId,
    Intent(u64),
    NextEscrowOrderId,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    ProviderNotActive = 4,
    ProviderExists = 5,
    ListingNotFound = 6,
    InvalidPricing = 7,
    InvalidQuality = 8,
    InvalidRoyalty = 9,
    InvalidAnonymization = 10,
    InvalidSettlementWindow = 11,
    InvalidStatus = 12,
    IntentNotFound = 13,
    EscrowNotLinked = 14,
    SettlementTimeout = 15,
}

#[contract]
pub struct HealthcareDataMarketplace;

#[contractimpl]
impl HealthcareDataMarketplace {
    pub fn initialize(
        env: Env,
        admin: Address,
        payment_router: Address,
        escrow_contract: Address,
        treasury: Address,
        settlement_window_secs: u64,
    ) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Config) {
            return Err(Error::AlreadyInitialized);
        }
        if settlement_window_secs == 0 || settlement_window_secs > MAX_SETTLEMENT_WINDOW_SECS {
            return Err(Error::InvalidSettlementWindow);
        }

        let cfg = Config {
            admin,
            payment_router,
            escrow_contract,
            treasury,
            settlement_window_secs,
        };
        env.storage().instance().set(&DataKey::Config, &cfg);
        env.storage().instance().set(&DataKey::ProviderCount, &0u64);
        env.storage().instance().set(&DataKey::NextListingId, &0u64);
        env.storage().instance().set(&DataKey::NextIntentId, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::NextEscrowOrderId, &0u64);
        Ok(())
    }

    pub fn register_provider(env: Env, provider: Address) -> Result<(), Error> {
        provider.require_auth();
        Self::require_initialized(&env)?;
        if env
            .storage()
            .persistent()
            .has(&DataKey::Provider(provider.clone()))
        {
            return Err(Error::ProviderExists);
        }

        let profile = ProviderProfile {
            provider: provider.clone(),
            active: true,
            listings_count: 0,
            reputation_bps: 5_000,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Provider(provider), &profile);

        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::ProviderCount)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::ProviderCount, &count.saturating_add(1));
        Ok(())
    }

    pub fn set_provider_status(
        env: Env,
        admin: Address,
        provider: Address,
        active: bool,
    ) -> Result<(), Error> {
        admin.require_auth();
        let cfg = Self::load_config(&env)?;
        if admin != cfg.admin {
            return Err(Error::Unauthorized);
        }

        let mut profile: ProviderProfile = env
            .storage()
            .persistent()
            .get(&DataKey::Provider(provider.clone()))
            .ok_or(Error::ProviderNotActive)?;
        profile.active = active;
        env.storage()
            .persistent()
            .set(&DataKey::Provider(provider), &profile);
        Ok(())
    }

    pub fn create_listing(
        env: Env,
        provider: Address,
        payload: ListingPayload,
    ) -> Result<u64, Error> {
        provider.require_auth();
        Self::require_initialized(&env)?;
        Self::require_provider_active(&env, &provider)?;
        Self::validate_anonymization(
            payload.anonymization,
            payload.min_k,
            payload.dp_epsilon_milli,
        )?;
        Self::validate_quality(&payload.quality)?;
        Self::validate_royalty(&payload.royalty)?;
        if payload.price <= 0 {
            return Err(Error::InvalidPricing);
        }

        let next_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextListingId)
            .unwrap_or(0u64)
            .saturating_add(1);
        let listing = Listing {
            id: next_id,
            provider: provider.clone(),
            data_ref: payload.data_ref,
            data_hash: payload.data_hash,
            format: payload.format,
            anonymization: payload.anonymization,
            min_k: payload.min_k,
            dp_epsilon_milli: payload.dp_epsilon_milli,
            quality: payload.quality,
            royalty: payload.royalty,
            price: payload.price,
            token: payload.token,
            created_at: env.ledger().timestamp(),
            status: ListingStatus::Active,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Listing(next_id), &listing);
        env.storage()
            .instance()
            .set(&DataKey::NextListingId, &next_id);

        let mut profile: ProviderProfile = env
            .storage()
            .persistent()
            .get(&DataKey::Provider(provider.clone()))
            .ok_or(Error::ProviderNotActive)?;
        profile.listings_count = profile.listings_count.saturating_add(1);
        env.storage()
            .persistent()
            .set(&DataKey::Provider(provider), &profile);

        env.events()
            .publish((symbol_short!("list_new"), next_id), listing.id);
        Ok(next_id)
    }

    pub fn reserve_purchase(env: Env, buyer: Address, listing_id: u64) -> Result<u64, Error> {
        buyer.require_auth();
        Self::require_initialized(&env)?;

        let mut listing: Listing = env
            .storage()
            .persistent()
            .get(&DataKey::Listing(listing_id))
            .ok_or(Error::ListingNotFound)?;
        if listing.status != ListingStatus::Active {
            return Err(Error::InvalidStatus);
        }

        listing.status = ListingStatus::Reserved;
        env.storage()
            .persistent()
            .set(&DataKey::Listing(listing_id), &listing.clone());

        let intent_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextIntentId)
            .unwrap_or(0u64)
            .saturating_add(1);
        let intent = PurchaseIntent {
            id: intent_id,
            listing_id,
            buyer,
            amount: listing.price,
            created_at: env.ledger().timestamp(),
            escrow_order_id: None,
            settled: false,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Intent(intent_id), &intent);
        env.storage()
            .instance()
            .set(&DataKey::NextIntentId, &intent_id);
        Ok(intent_id)
    }

    pub fn initiate_transaction(env: Env, buyer: Address, intent_id: u64) -> Result<u64, Error> {
        buyer.require_auth();
        let cfg = Self::load_config(&env)?;
        let mut intent: PurchaseIntent = env
            .storage()
            .persistent()
            .get(&DataKey::Intent(intent_id))
            .ok_or(Error::IntentNotFound)?;
        if intent.buyer != buyer || intent.settled {
            return Err(Error::InvalidStatus);
        }
        let listing: Listing = env
            .storage()
            .persistent()
            .get(&DataKey::Listing(intent.listing_id))
            .ok_or(Error::ListingNotFound)?;
        if listing.status != ListingStatus::Reserved {
            return Err(Error::InvalidStatus);
        }

        // Integrate with existing payment router split logic.
        let _: (i128, i128) = env.invoke_contract(
            &cfg.payment_router,
            &Symbol::new(&env, "compute_split"),
            Vec::from_array(&env, [listing.price.into_val(&env)]),
        );

        let escrow_order_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextEscrowOrderId)
            .unwrap_or(0u64)
            .saturating_add(1);
        env.storage()
            .instance()
            .set(&DataKey::NextEscrowOrderId, &escrow_order_id);

        // Integrate with existing escrow contract creation.
        let created: bool = env.invoke_contract(
            &cfg.escrow_contract,
            &Symbol::new(&env, "create_escrow"),
            Vec::from_array(
                &env,
                [
                    escrow_order_id.into_val(&env),
                    buyer.into_val(&env),
                    listing.provider.into_val(&env),
                    listing.price.into_val(&env),
                    listing.token.into_val(&env),
                ],
            ),
        );
        if !created {
            return Err(Error::InvalidStatus);
        }

        intent.escrow_order_id = Some(escrow_order_id);
        env.storage()
            .persistent()
            .set(&DataKey::Intent(intent_id), &intent);
        Ok(escrow_order_id)
    }

    pub fn finalize_settlement(
        env: Env,
        settler: Address,
        intent_id: u64,
    ) -> Result<(i128, i128, i128), Error> {
        settler.require_auth();
        let cfg = Self::load_config(&env)?;
        let mut intent: PurchaseIntent = env
            .storage()
            .persistent()
            .get(&DataKey::Intent(intent_id))
            .ok_or(Error::IntentNotFound)?;
        if intent.escrow_order_id.is_none() {
            return Err(Error::EscrowNotLinked);
        }
        if intent.settled {
            return Err(Error::InvalidStatus);
        }
        let now = env.ledger().timestamp();
        if now.saturating_sub(intent.created_at) > cfg.settlement_window_secs {
            return Err(Error::SettlementTimeout);
        }

        let mut listing: Listing = env
            .storage()
            .persistent()
            .get(&DataKey::Listing(intent.listing_id))
            .ok_or(Error::ListingNotFound)?;
        if listing.status != ListingStatus::Reserved {
            return Err(Error::InvalidStatus);
        }

        let provider_amount = intent
            .amount
            .saturating_mul(listing.royalty.provider_bps as i128)
            / 10_000;
        let curator_amount = intent
            .amount
            .saturating_mul(listing.royalty.curator_bps as i128)
            / 10_000;
        let platform_amount = intent
            .amount
            .saturating_sub(provider_amount)
            .saturating_sub(curator_amount);

        listing.status = ListingStatus::Settled;
        intent.settled = true;
        env.storage()
            .persistent()
            .set(&DataKey::Listing(intent.listing_id), &listing);
        env.storage()
            .persistent()
            .set(&DataKey::Intent(intent_id), &intent);
        env.events().publish(
            (symbol_short!("settled"), intent_id),
            (provider_amount, curator_amount, platform_amount),
        );

        Ok((provider_amount, curator_amount, platform_amount))
    }

    pub fn cancel_listing(env: Env, actor: Address, listing_id: u64) -> Result<(), Error> {
        actor.require_auth();
        let cfg = Self::load_config(&env)?;
        let mut listing: Listing = env
            .storage()
            .persistent()
            .get(&DataKey::Listing(listing_id))
            .ok_or(Error::ListingNotFound)?;

        if actor != listing.provider && actor != cfg.admin {
            return Err(Error::Unauthorized);
        }
        if listing.status == ListingStatus::Settled {
            return Err(Error::InvalidStatus);
        }
        listing.status = ListingStatus::Cancelled;
        env.storage()
            .persistent()
            .set(&DataKey::Listing(listing_id), &listing);
        Ok(())
    }

    pub fn get_provider_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::ProviderCount)
            .unwrap_or(0)
    }

    pub fn get_provider(env: Env, provider: Address) -> Option<ProviderProfile> {
        env.storage().persistent().get(&DataKey::Provider(provider))
    }

    pub fn get_listing(env: Env, listing_id: u64) -> Option<Listing> {
        env.storage()
            .persistent()
            .get(&DataKey::Listing(listing_id))
    }

    pub fn get_intent(env: Env, intent_id: u64) -> Option<PurchaseIntent> {
        env.storage().persistent().get(&DataKey::Intent(intent_id))
    }

    fn require_initialized(env: &Env) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Config) {
            Ok(())
        } else {
            Err(Error::NotInitialized)
        }
    }

    fn load_config(env: &Env) -> Result<Config, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)
    }

    fn require_provider_active(env: &Env, provider: &Address) -> Result<(), Error> {
        let profile: ProviderProfile = env
            .storage()
            .persistent()
            .get(&DataKey::Provider(provider.clone()))
            .ok_or(Error::ProviderNotActive)?;
        if profile.active {
            Ok(())
        } else {
            Err(Error::ProviderNotActive)
        }
    }

    fn validate_anonymization(
        anonymization: AnonymizationLevel,
        min_k: u32,
        dp_epsilon_milli: u32,
    ) -> Result<(), Error> {
        match anonymization {
            AnonymizationLevel::KAnonymity => {
                if min_k < 5 {
                    return Err(Error::InvalidAnonymization);
                }
            }
            AnonymizationLevel::DifferentialPrivacy => {
                if dp_epsilon_milli == 0 || dp_epsilon_milli > 10_000 {
                    return Err(Error::InvalidAnonymization);
                }
            }
            AnonymizationLevel::Synthetic => {}
        }
        Ok(())
    }

    fn validate_quality(quality: &QualityMetrics) -> Result<(), Error> {
        let fields = [
            quality.completeness_bps,
            quality.consistency_bps,
            quality.timeliness_bps,
            quality.validity_bps,
        ];
        for metric in fields {
            if !(7_000..=10_000).contains(&metric) {
                return Err(Error::InvalidQuality);
            }
        }
        Ok(())
    }

    fn validate_royalty(royalty: &RoyaltyPolicy) -> Result<(), Error> {
        let total = royalty
            .provider_bps
            .saturating_add(royalty.curator_bps)
            .saturating_add(royalty.platform_bps);
        if total != 10_000 || royalty.provider_bps == 0 {
            return Err(Error::InvalidRoyalty);
        }
        Ok(())
    }
}


// ============================================================
// Issue #658: Token-Gated Access Tiers
// ============================================================

const BASIC_COST: i128 = 0;
const STANDARD_COST: i128 = 100;   // 100 SUT
const PREMIUM_COST: i128 = 1000;   // 1000 SUT
const TIER_DURATION_LEDGERS: u32 = 30 * 24 * 60 * 12; // ~30 days at 5s/ledger

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum AccessTier {
    Basic,
    Standard,
    Premium,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct TierGrant {
    pub tier: AccessTier,
    pub granted_at: u32, // ledger sequence
    pub expires_at: u32,
}

#[contracttype]
pub enum TierKey {
    Grant(Address), // keyed by buyer/researcher address
}

/// Purchase an access tier by transferring SUT tokens to treasury.
/// Basic is free; Standard costs 100 SUT; Premium costs 1000 SUT.
pub fn purchase_access_tier(
    env: Env,
    buyer: Address,
    tier: AccessTier,
    sut_token: Address,
    treasury: Address,
) {
    buyer.require_auth();
    let cost: i128 = match &tier {
        AccessTier::Basic => BASIC_COST,
        AccessTier::Standard => STANDARD_COST,
        AccessTier::Premium => PREMIUM_COST,
    };
    if cost > 0 {
        let token_client = token::Client::new(&env, &sut_token);
        token_client.transfer(&buyer, &treasury, &cost);
    }
    let current = env.ledger().sequence();
    let grant = TierGrant {
        tier: tier.clone(),
        granted_at: current,
        expires_at: current + TIER_DURATION_LEDGERS,
    };
    env.storage()
        .persistent()
        .set(&TierKey::Grant(buyer.clone()), &grant);
    env.events().publish(
        (Symbol::new(&env, "TierPurchased"),),
        (buyer, tier, cost),
    );
}

/// Get a caller's current tier. Returns Basic if grant is missing or expired.
pub fn get_effective_tier(env: Env, caller: Address) -> AccessTier {
    match env
        .storage()
        .persistent()
        .get::<TierKey, TierGrant>(&TierKey::Grant(caller))
    {
        Some(grant) if env.ledger().sequence() <= grant.expires_at => grant.tier,
        _ => AccessTier::Basic,
    }
}

/// Query data — returns granularity matching the caller's tier.
/// Basic: aggregated only. Standard: anonymized records. Premium: full de-identified.
pub fn query_data(env: Env, caller: Address) -> Symbol {
    match get_effective_tier(env.clone(), caller) {
        AccessTier::Basic => Symbol::new(&env, "aggregated_only"),
        AccessTier::Standard => Symbol::new(&env, "anonymized_records"),
        AccessTier::Premium => Symbol::new(&env, "full_deidentified"),
    }
}

#[cfg(test)]
mod tier_tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::Env;

    #[test]
    fn test_basic_tier_default() {
        let env = Env::default();
        let caller = Address::generate(&env);
        let tier = get_effective_tier(env.clone(), caller);
        assert_eq!(tier, AccessTier::Basic);
    }

    #[test]
    fn test_query_returns_correct_granularity_by_tier() {
        // Basic returns aggregated_only
        let env = Env::default();
        let caller = Address::generate(&env);
        let result = query_data(env.clone(), caller);
        assert_eq!(result, Symbol::new(&env, "aggregated_only"));
    }

    #[test]
    fn test_expired_tier_reverts_to_basic() {
        let env = Env::default();
        env.mock_all_auths();
        let caller = Address::generate(&env);
        // Manually write an already-expired grant
        let grant = TierGrant {
            tier: AccessTier::Premium,
            granted_at: 0,
            expires_at: 0,
        };
        env.storage()
            .persistent()
            .set(&TierKey::Grant(caller.clone()), &grant);
        let tier = get_effective_tier(env.clone(), caller);
        assert_eq!(tier, AccessTier::Basic);
    }
}
