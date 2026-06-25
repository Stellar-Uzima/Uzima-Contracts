#![no_std]
//! genomic_data - Healthcare smart contract on Stellar blockchain.
#![allow(clippy::too_many_arguments)]
#![allow(clippy::arithmetic_side_effects)]

#[cfg(test)]
mod test;

use common_error::CommonError;
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Bytes, BytesN, Env, String,
    Symbol, Vec,
};
use upgradeability::storage::{ADMIN as UPGRADE_ADMIN, VERSION};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum GenomicFormat {
    Fasta,
    Vcf,
    Bam,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum Compression {
    None,
    Gzip,
    Zstd,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum EnvelopeAlgorithm {
    X25519,
    Kyber768,
    HybridX25519Kyber768,
}

#[derive(Clone)]
#[contracttype]
pub struct KeyEnvelope {
    pub recipient: Address,
    pub key_version: u32,
    pub algorithm: EnvelopeAlgorithm,
    pub wrapped_key: Bytes,
    pub pq_wrapped_key: Option<Bytes>,
}

#[derive(Clone)]
#[contracttype]
pub struct GenomicRecordHeader {
    pub id: u64,
    pub patient: Address,
    pub uploader: Address,
    pub format: GenomicFormat,
    pub compression: Compression,
    pub created_at: u64,
    pub data_ref: String,
    pub data_hash: BytesN<32>,
    pub ciphertext_hash: BytesN<32>,
}

#[derive(Clone)]
#[contracttype]
pub struct GenomicRecord {
    pub header: GenomicRecordHeader,
    pub tags: Vec<String>,
    pub envelopes: Vec<KeyEnvelope>,
    pub consent_id: Option<Bytes>,
}

#[derive(Clone)]
#[contracttype]
pub struct PrivacyGrant {
    pub record_id: u64,
    pub requester: Address,
    pub expires_at: u64,
    pub pseudonym: BytesN<32>,
    pub vk_version: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct GeneDiseaseAssoc {
    pub record_id: u64,
    pub gene: String,
    pub disease_code: String,
    pub score_bps: u32,
    pub method: String,
    pub created_at: u64,
    pub curator: Address,
}

#[derive(Clone)]
#[contracttype]
pub struct DrugResponse {
    pub record_id: u64,
    pub drug: String,
    pub genotype_marker: String,
    pub effect: String,
    pub recommendation: String,
    pub created_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct PopulationShare {
    pub label: String,
    pub bps: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct AncestryProfile {
    pub record_id: u64,
    pub components: Vec<PopulationShare>,
    pub method: String,
    pub created_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct ConsentEntry {
    pub record_id: u64,
    pub patient: Address,
    pub grantee: Address,
    pub scope: String,
    pub expires_at: u64,
    pub active: bool,
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub enum GenomicConsentCategory {
    GeneralResearch,
    DiseaseSpecific(String),
    CommercialResearch,
    InternationalTransfer,
}

#[derive(Clone)]
#[contracttype]
pub struct ResearchConsentEntry {
    pub record_id: u64,
    pub patient: Address,
    pub grantee: Address,
    pub category: GenomicConsentCategory,
    pub expires_at: u64,
    pub active: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct GenomicConsentAudit {
    pub record_id: u64,
    pub requester: Address,
    pub category: GenomicConsentCategory,
    pub granted: bool,
    pub timestamp: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct ResearchWithdrawalNotification {
    pub record_id: u64,
    pub category: GenomicConsentCategory,
    pub notified_project: Address,
    pub revoked_grantee: Address,
    pub timestamp: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum ListingStatus {
    Active,
    Purchased,
    Cancelled,
}

#[derive(Clone)]
#[contracttype]
pub struct Listing {
    pub listing_id: u64,
    pub record_id: u64,
    pub seller: Address,
    pub price: i128,
    pub currency: Address,
    pub escrow: Option<Address>,
    pub buyer: Option<Address>,
    pub status: ListingStatus,
    pub created_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct BreachEvent {
    pub id: u64,
    pub reporter: Address,
    pub record_id: Option<u64>,
    pub severity_bps: u32,
    pub message: String,
    pub created_at: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum LogLevel {
    Info,
    Warning,
    Err,
}

#[derive(Clone)]
#[contracttype]
pub struct StructuredLog {
    pub timestamp: u64,
    pub level: LogLevel,
    pub operation: String,
    pub actor: Option<Address>,
    pub record_id: Option<u64>,
    pub message: String,
}

#[derive(Clone)]
#[contracttype]
pub struct RateLimitConfig {
    pub max_calls: u32,
    pub window_secs: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct RateLimitEntry {
    pub count: u32,
    pub window_start: u64,
}

#[contracttype]
pub enum DataKey {
    // Instance storage keys (contract config/metadata)
    Initialized,
    NextId,
    ListingNextId,
    ZkVerifierContract,
    // Persistent storage keys (critical long-lived data)
    Record(u64),
    RecordHeader(u64),
    PatientRecords(Address),
    Consent(u64, Address),
    ResearchConsent(u64, Address, GenomicConsentCategory),
    ActiveResearchProjects(u64, GenomicConsentCategory),
    AssocCount(u64),
    Assoc(u64, u64),
    DrugRespCount(u64),
    DrugResp(u64, u64),
    Ancestry(u64),
    Listing(u64),
    RecordListings(u64),
    BreachCount,
    Breach(u64),
    // Temporary storage keys (short-lived data)
    RateLimitCfg(u32),
    RateLimit(Address, u32),
}

#[soroban_sdk::contractclient(name = "ZkVerifierClient")]
pub trait ZkVerifier {
    fn verify_proof(
        env: Env,
        vk_version: u32,
        public_inputs_hash: BytesN<32>,
        proof: Bytes,
    ) -> bool;
}

#[contract]
pub struct GenomicDataContract;

#[contractimpl]
impl GenomicDataContract {
    fn emit_log(
        env: &Env,
        level: LogLevel,
        operation: &str,
        actor: Option<&Address>,
        record_id: Option<u64>,
        message: &str,
    ) {
        let topic = match level {
            LogLevel::Info => symbol_short!("LOG_INFO"),
            LogLevel::Warning => symbol_short!("LOG_WARN"),
            LogLevel::Err => symbol_short!("LOG_ERROR"),
        };
        let entry = StructuredLog {
            timestamp: env.ledger().timestamp(),
            level,
            operation: String::from_str(env, operation),
            actor: actor.cloned(),
            record_id,
            message: String::from_str(env, message),
        };
        env.events().publish(("LOG", topic), entry);
    }

    pub fn initialize(env: Env, admin: Address) -> Result<(), CommonError> {
        admin.require_auth();
        if env.storage().instance().has(&UPGRADE_ADMIN) {
            return Err(CommonError::AlreadyInitialized);
        }
        env.storage().instance().set(&UPGRADE_ADMIN, &admin);
        env.storage().instance().set(&VERSION, &1u32);
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::NextId, &0u64);
        env.storage().instance().set(&DataKey::ListingNextId, &0u64);
        Self::emit_log(
            &env,
            LogLevel::Info,
            "initialize",
            Some(&admin),
            None,
            "initialized",
        );
        Ok(())
    }

    pub fn set_zk_verifier(
        env: Env,
        admin: Address,
        contract_id: Address,
    ) -> Result<(), CommonError> {
        admin.require_auth();
        if !Self::require_admin(&env, &admin) {
            return Err(CommonError::Unauthorized);
        }
        env.storage()
            .instance()
            .set(&DataKey::ZkVerifierContract, &contract_id);
        Self::emit_log(
            &env,
            LogLevel::Info,
            "set_zk_verifier",
            Some(&admin),
            None,
            "zk verifier set",
        );
        Ok(())
    }

    pub fn add_record(
        env: Env,
        patient: Address,
        uploader: Address,
        format: GenomicFormat,
        compression: Compression,
        data_ref: String,
        data_hash: BytesN<32>,
        ciphertext_hash: BytesN<32>,
        tags: Vec<String>,
        envelopes: Vec<KeyEnvelope>,
        consent_id: Option<Bytes>,
    ) -> u64 {
        uploader.require_auth();
        let id = env
            .storage()
            .instance()
            .get::<DataKey, u64>(&DataKey::NextId)
            .unwrap_or(0);
        let new_id = id + 1;
        let header = GenomicRecordHeader {
            id: new_id,
            patient: patient.clone(),
            uploader: uploader.clone(),
            format,
            compression,
            created_at: env.ledger().timestamp(),
            data_ref,
            data_hash,
            ciphertext_hash,
        };
        let record = GenomicRecord {
            header: header.clone(),
            tags,
            envelopes,
            consent_id,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Record(new_id), &record);
        env.storage()
            .persistent()
            .set(&DataKey::RecordHeader(new_id), &header);
        let mut list: Vec<u64> = env
            .storage()
            .persistent()
            .get::<DataKey, Vec<u64>>(&DataKey::PatientRecords(patient.clone()))
            .unwrap_or(Vec::new(&env));
        list.push_back(new_id);
        env.storage()
            .persistent()
            .set(&DataKey::PatientRecords(patient), &list);
        env.storage().instance().set(&DataKey::NextId, &new_id);
        Self::emit_log(
            &env,
            LogLevel::Info,
            "add_record",
            Some(&uploader),
            Some(new_id),
            "record added",
        );
        new_id
    }

    pub fn get_record_header(env: Env, caller: Address, id: u64) -> Option<GenomicRecordHeader> {
        caller.require_auth();
        let header = env
            .storage()
            .persistent()
            .get::<DataKey, GenomicRecordHeader>(&DataKey::RecordHeader(id));
        let h = header.as_ref()?;
        if caller == h.patient
            || caller == h.uploader
            || Self::is_consent_granted(&env, id, &caller)
        {
            header
        } else {
            None
        }
    }

    pub fn grant_consent(
        env: Env,
        patient: Address,
        record_id: u64,
        grantee: Address,
        scope: String,
        expires_at: u64,
    ) -> bool {
        patient.require_auth();
        let header_opt = env
            .storage()
            .persistent()
            .get::<DataKey, GenomicRecordHeader>(&DataKey::RecordHeader(record_id));
        if let Some(header) = header_opt.as_ref() {
            if header.patient != patient {
                return false;
            }
        } else {
            return false;
        }
        env.storage().persistent().set(
            &DataKey::Consent(record_id, grantee.clone()),
            &ConsentEntry {
                record_id,
                patient: patient.clone(),
                grantee: grantee.clone(),
                scope,
                expires_at,
                active: true,
            },
        );
        Self::emit_log(
            &env,
            LogLevel::Info,
            "grant_consent",
            Some(&patient),
            Some(record_id),
            "consent granted",
        );
        true
    }

    pub fn revoke_consent(
        env: Env,
        patient: Address,
        record_id: u64,
        grantee: Address,
    ) -> Result<(), CommonError> {
        patient.require_auth();
        let ce_opt = env
            .storage()
            .persistent()
            .get::<DataKey, ConsentEntry>(&DataKey::Consent(record_id, grantee.clone()));
        let mut entry = if let Some(entry) = ce_opt {
            if entry.patient != patient {
                return Err(CommonError::Unauthorized);
            }
            entry
        } else {
            return Err(CommonError::NotFound);
        };
        entry.active = false;
        env.storage()
            .persistent()
            .set(&DataKey::Consent(record_id, grantee), &entry);
        Self::emit_log(
            &env,
            LogLevel::Info,
            "revoke_consent",
            Some(&patient),
            Some(record_id),
            "consent revoked",
        );
        Ok(())
    }

    pub fn grant_research_consent(
        env: Env,
        patient: Address,
        record_id: u64,
        grantee: Address,
        category: GenomicConsentCategory,
        expires_at: u64,
    ) -> bool {
        patient.require_auth();
        let header_opt = env
            .storage()
            .persistent()
            .get::<DataKey, GenomicRecordHeader>(&DataKey::RecordHeader(record_id));
        if let Some(header) = header_opt.as_ref() {
            if header.patient != patient {
                return false;
            }
        } else {
            return false;
        }
        env.storage().persistent().set(
            &DataKey::ResearchConsent(record_id, grantee.clone(), category.clone()),
            &ResearchConsentEntry {
                record_id,
                patient: patient.clone(),
                grantee: grantee.clone(),
                category: category.clone(),
                expires_at,
                active: true,
            },
        );
        Self::emit_log(
            &env,
            LogLevel::Info,
            "grant_research_consent",
            Some(&patient),
            Some(record_id),
            "research consent granted",
        );
        true
    }

    pub fn revoke_research_consent(
        env: Env,
        patient: Address,
        record_id: u64,
        grantee: Address,
        category: GenomicConsentCategory,
    ) -> bool {
        patient.require_auth();
        let ce_opt = env
            .storage()
            .persistent()
            .get::<DataKey, ResearchConsentEntry>(&DataKey::ResearchConsent(
                record_id,
                grantee.clone(),
                category.clone(),
            ));
        let mut entry = if let Some(entry) = ce_opt {
            if entry.patient != patient {
                return false;
            }
            entry
        } else {
            return false;
        };
        entry.active = false;
        env.storage().persistent().set(
            &DataKey::ResearchConsent(record_id, grantee.clone(), category.clone()),
            &entry,
        );
        Self::notify_research_withdrawal(&env, record_id, &category, &grantee);
        Self::emit_log(
            &env,
            LogLevel::Info,
            "revoke_research_consent",
            Some(&patient),
            Some(record_id),
            "research consent revoked",
        );
        true
    }

    pub fn get_record_header_for_research(
        env: Env,
        requester: Address,
        record_id: u64,
        category: GenomicConsentCategory,
    ) -> Option<GenomicRecordHeader> {
        requester.require_auth();
        let header = env
            .storage()
            .persistent()
            .get::<DataKey, GenomicRecordHeader>(&DataKey::RecordHeader(record_id));
        let header_ref = header.as_ref()?;
        if requester == header_ref.patient
            || requester == header_ref.uploader
            || Self::is_research_consent_granted(&env, record_id, &requester, &category)
        {
            Self::emit_research_audit(&env, record_id, &requester, &category, true);
            Self::track_active_research_project(&env, record_id, requester.clone(), &category);
            header
        } else {
            Self::emit_research_audit(&env, record_id, &requester, &category, false);
            None
        }
    }

    fn emit_research_audit(
        env: &Env,
        record_id: u64,
        requester: &Address,
        category: &GenomicConsentCategory,
        granted: bool,
    ) {
        let audit = GenomicConsentAudit {
            record_id,
            requester: requester.clone(),
            category: category.clone(),
            granted,
            timestamp: env.ledger().timestamp(),
        };
        env.events()
            .publish(("GENOMIC_CONSENT", symbol_short!("AUDIT")), audit);
    }

    fn track_active_research_project(
        env: &Env,
        record_id: u64,
        project: Address,
        category: &GenomicConsentCategory,
    ) {
        let mut projects: Vec<Address> = env
            .storage()
            .persistent()
            .get::<DataKey, Vec<Address>>(&DataKey::ActiveResearchProjects(
                record_id,
                category.clone(),
            ))
            .unwrap_or(Vec::new(env));
        let mut already_present = false;
        let len = projects.len();
        let mut i = 0;
        while i < len {
            if projects.get(i) == Some(project.clone()) {
                already_present = true;
                break;
            }
            i += 1;
        }
        if !already_present {
            projects.push_back(project.clone());
            env.storage().persistent().set(
                &DataKey::ActiveResearchProjects(record_id, category.clone()),
                &projects,
            );
        }
    }

    fn notify_research_withdrawal(
        env: &Env,
        record_id: u64,
        category: &GenomicConsentCategory,
        revoked_grantee: &Address,
    ) {
        let projects: Vec<Address> = env
            .storage()
            .persistent()
            .get::<DataKey, Vec<Address>>(&DataKey::ActiveResearchProjects(
                record_id,
                category.clone(),
            ))
            .unwrap_or(Vec::new(env));
        let len = projects.len();
        let mut i = 0;
        while i < len {
            if let Some(project) = projects.get(i) {
                let notification = ResearchWithdrawalNotification {
                    record_id,
                    category: category.clone(),
                    notified_project: project.clone(),
                    revoked_grantee: revoked_grantee.clone(),
                    timestamp: env.ledger().timestamp(),
                };
                env.events().publish(
                    ("GENOMIC_CONSENT", Symbol::new(env, "WITHDRAWAL")),
                    notification,
                );
            }
            i += 1;
        }
    }

    fn is_research_consent_granted(
        env: &Env,
        record_id: u64,
        grantee: &Address,
        category: &GenomicConsentCategory,
    ) -> bool {
        let ce = env
            .storage()
            .persistent()
            .get::<DataKey, ResearchConsentEntry>(&DataKey::ResearchConsent(
                record_id,
                grantee.clone(),
                category.clone(),
            ));
        if let Some(c) = ce {
            if c.active && (c.expires_at == 0 || env.ledger().timestamp() <= c.expires_at) {
                return true;
            }
        }
        false
    }

    pub fn verify_and_grant_access(
        env: Env,
        patient: Address,
        record_id: u64,
        requester: Address,
        vk_version: u32,
        public_inputs_hash: BytesN<32>,
        proof: Bytes,
        pseudonym: BytesN<32>,
        expires_at: u64,
    ) -> bool {
        patient.require_auth();
        let header_opt = env
            .storage()
            .persistent()
            .get::<DataKey, GenomicRecordHeader>(&DataKey::RecordHeader(record_id));
        if let Some(h) = header_opt.as_ref() {
            if h.patient != patient {
                return false;
            }
        } else {
            return false;
        }
        let verifier = env
            .storage()
            .instance()
            .get::<DataKey, Address>(&DataKey::ZkVerifierContract);
        if let Some(addr) = verifier {
            let client = ZkVerifierClient::new(&env, &addr);
            let ok = client.verify_proof(&vk_version, &public_inputs_hash, &proof);
            if !ok {
                return false;
            }
        }
        let grant = PrivacyGrant {
            record_id,
            requester: requester.clone(),
            expires_at,
            pseudonym,
            vk_version,
        };
        env.storage().temporary().set(
            &(Symbol::new(&env, "pg"), record_id, requester.clone()),
            &grant,
        );
        // Set TTL for privacy grant temporary storage (~4 hours)
        env.storage().temporary().extend_ttl(
            &(Symbol::new(&env, "pg"), record_id, requester.clone()),
            0,
            1000,
        );
        Self::emit_log(
            &env,
            LogLevel::Info,
            "verify_and_grant_access",
            Some(&patient),
            Some(record_id),
            "zk access granted",
        );
        true
    }

    pub fn add_gene_disease_assoc(
        env: Env,
        curator: Address,
        record_id: u64,
        gene: String,
        disease_code: String,
        score_bps: u32,
        method: String,
    ) -> u64 {
        curator.require_auth();
        if env
            .storage()
            .persistent()
            .get::<DataKey, GenomicRecordHeader>(&DataKey::RecordHeader(record_id))
            .is_none()
        {
            return 0;
        }
        let count = env
            .storage()
            .persistent()
            .get::<DataKey, u64>(&DataKey::AssocCount(record_id))
            .unwrap_or(0);
        let new_idx = count + 1;
        let assoc = GeneDiseaseAssoc {
            record_id,
            gene,
            disease_code,
            score_bps,
            method,
            created_at: env.ledger().timestamp(),
            curator,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Assoc(record_id, new_idx), &assoc);
        env.storage()
            .persistent()
            .set(&DataKey::AssocCount(record_id), &new_idx);
        Self::emit_log(
            &env,
            LogLevel::Info,
            "add_gene_disease_assoc",
            None,
            Some(record_id),
            "assoc added",
        );
        new_idx
    }

    pub fn add_drug_response(
        env: Env,
        caller: Address,
        record_id: u64,
        drug: String,
        genotype_marker: String,
        effect: String,
        recommendation: String,
    ) -> u64 {
        caller.require_auth();
        if env
            .storage()
            .persistent()
            .get::<DataKey, GenomicRecordHeader>(&DataKey::RecordHeader(record_id))
            .is_none()
        {
            return 0;
        }
        let count = env
            .storage()
            .persistent()
            .get::<DataKey, u64>(&DataKey::DrugRespCount(record_id))
            .unwrap_or(0);
        let new_idx = count + 1;
        let resp = DrugResponse {
            record_id,
            drug,
            genotype_marker,
            effect,
            recommendation,
            created_at: env.ledger().timestamp(),
        };
        env.storage()
            .persistent()
            .set(&DataKey::DrugResp(record_id, new_idx), &resp);
        env.storage()
            .persistent()
            .set(&DataKey::DrugRespCount(record_id), &new_idx);
        Self::emit_log(
            &env,
            LogLevel::Info,
            "add_drug_response",
            None,
            Some(record_id),
            "pharmacogenomic added",
        );
        new_idx
    }

    pub fn set_ancestry_profile(
        env: Env,
        caller: Address,
        record_id: u64,
        components: Vec<PopulationShare>,
        method: String,
    ) -> bool {
        caller.require_auth();
        if env
            .storage()
            .persistent()
            .get::<DataKey, GenomicRecordHeader>(&DataKey::RecordHeader(record_id))
            .is_none()
        {
            return false;
        }
        let profile = AncestryProfile {
            record_id,
            components,
            method,
            created_at: env.ledger().timestamp(),
        };
        env.storage()
            .persistent()
            .set(&DataKey::Ancestry(record_id), &profile);
        Self::emit_log(
            &env,
            LogLevel::Info,
            "set_ancestry_profile",
            Some(&caller),
            Some(record_id),
            "ancestry set",
        );
        true
    }

    pub fn create_listing(
        env: Env,
        seller: Address,
        record_id: u64,
        price: i128,
        currency: Address,
        escrow: Option<Address>,
    ) -> u64 {
        seller.require_auth();
        let header_opt = env
            .storage()
            .persistent()
            .get::<DataKey, GenomicRecordHeader>(&DataKey::RecordHeader(record_id));
        if let Some(h) = header_opt.as_ref() {
            if h.uploader != seller {
                return 0;
            }
        } else {
            return 0;
        }
        let lid = env
            .storage()
            .instance()
            .get::<DataKey, u64>(&DataKey::ListingNextId)
            .unwrap_or(0)
            + 1;
        let listing = Listing {
            listing_id: lid,
            record_id,
            seller: seller.clone(),
            price,
            currency,
            escrow,
            buyer: None,
            status: ListingStatus::Active,
            created_at: env.ledger().timestamp(),
        };
        env.storage()
            .persistent()
            .set(&DataKey::Listing(lid), &listing);
        let mut ids: Vec<u64> = env
            .storage()
            .persistent()
            .get::<DataKey, Vec<u64>>(&DataKey::RecordListings(record_id))
            .unwrap_or(Vec::new(&env));
        ids.push_back(lid);
        env.storage()
            .persistent()
            .set(&DataKey::RecordListings(record_id), &ids);
        env.storage().instance().set(&DataKey::ListingNextId, &lid);
        Self::emit_log(
            &env,
            LogLevel::Info,
            "create_listing",
            Some(&seller),
            Some(record_id),
            "listing created",
        );
        lid
    }

    pub fn purchase_listing(env: Env, buyer: Address, listing_id: u64) -> Result<(), CommonError> {
        buyer.require_auth();
        let l_opt = env
            .storage()
            .persistent()
            .get::<DataKey, Listing>(&DataKey::Listing(listing_id));
        let mut listing = if let Some(listing) = l_opt {
            listing
        } else {
            return Err(CommonError::NotFound);
        };
        if listing.status != ListingStatus::Active {
            return Err(CommonError::InvalidState);
        }
        listing.buyer = Some(buyer.clone());
        listing.status = ListingStatus::Purchased;
        env.storage()
            .persistent()
            .set(&DataKey::Listing(listing_id), &listing);
        Self::emit_log(
            &env,
            LogLevel::Info,
            "purchase_listing",
            Some(&buyer),
            Some(listing.record_id),
            "listing purchased",
        );
        Ok(())
    }

    pub fn report_breach(
        env: Env,
        reporter: Address,
        record_id: Option<u64>,
        severity_bps: u32,
        message: String,
    ) -> u64 {
        reporter.require_auth();
        let id = env
            .storage()
            .persistent()
            .get::<DataKey, u64>(&DataKey::BreachCount)
            .unwrap_or(0)
            + 1;
        let ev = BreachEvent {
            id,
            reporter: reporter.clone(),
            record_id,
            severity_bps,
            message,
            created_at: env.ledger().timestamp(),
        };
        env.storage().persistent().set(&DataKey::Breach(id), &ev);
        env.storage().persistent().set(&DataKey::BreachCount, &id);
        Self::emit_log(
            &env,
            LogLevel::Warning,
            "report_breach",
            Some(&reporter),
            record_id,
            "breach reported",
        );
        id
    }

    fn require_admin(env: &Env, who: &Address) -> bool {
        if let Some(admin) = upgradeability::storage::get_admin(env) {
            &admin == who
        } else {
            false
        }
    }

    fn is_consent_granted(env: &Env, record_id: u64, grantee: &Address) -> bool {
        let ce = env
            .storage()
            .persistent()
            .get::<DataKey, ConsentEntry>(&DataKey::Consent(record_id, grantee.clone()));
        if let Some(c) = ce {
            if c.active && (c.expires_at == 0 || env.ledger().timestamp() <= c.expires_at) {
                return true;
            }
        }
        false
    }

    pub fn upgrade(
        env: Env,
        caller: Address,
        new_wasm_hash: BytesN<32>,
        new_version: u32,
    ) -> Result<(), upgradeability::UpgradeError> {
        caller.require_auth();
        if !Self::require_admin(&env, &caller) {
            return Err(upgradeability::UpgradeError::NotAuthorized);
        }

        upgradeability::execute_upgrade::<Self>(
            &env,
            new_wasm_hash,
            new_version,
            symbol_short!("Upgrade"),
        )
    }

    pub fn validate_upgrade(
        env: Env,
        new_wasm_hash: BytesN<32>,
    ) -> Result<upgradeability::UpgradeValidation, upgradeability::UpgradeError> {
        upgradeability::validate_upgrade::<Self>(&env, new_wasm_hash)
    }
}

impl upgradeability::migration::Migratable for GenomicDataContract {
    fn migrate(_env: &Env, _from_version: u32) -> Result<(), upgradeability::UpgradeError> {
        Ok(())
    }

    fn verify_integrity(env: &Env) -> Result<BytesN<32>, upgradeability::UpgradeError> {
        let next_id = env
            .storage()
            .persistent()
            .get::<DataKey, u64>(&DataKey::NextId)
            .unwrap_or(0);
        let mut data = Vec::new(env);
        data.push_back(next_id);
        let hash_bytes = env.crypto().sha256(&data.to_xdr(env));
        Ok(BytesN::from_array(env, &hash_bytes.to_array()))
    }
}
