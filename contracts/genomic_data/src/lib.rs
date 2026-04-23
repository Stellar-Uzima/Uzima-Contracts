#![no_std]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::arithmetic_side_effects)]

#[cfg(test)]
mod test;

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
    ErrorLevel,
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
    Initialized,
    NextId,
    Record(u64),
    RecordHeader(u64),
    PatientRecords(Address),
    ZkVerifierContract,
    Consent(u64, Address),
    AssocCount(u64),
    Assoc(u64, u64),
    DrugRespCount(u64),
    DrugResp(u64, u64),
    Ancestry(u64),
    ListingNextId,
    Listing(u64),
    RecordListings(u64),
    BreachCount,
    Breach(u64),
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
            LogLevel::ErrorLevel => symbol_short!("LOG_ERROR"),
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

    pub fn initialize(env: Env, admin: Address) -> bool {
        admin.require_auth();
        if env.storage().instance().has(&UPGRADE_ADMIN) {
            return false;
        }
        env.storage().instance().set(&UPGRADE_ADMIN, &admin);
        env.storage().instance().set(&VERSION, &1u32);
        env.storage().persistent().set(&DataKey::Initialized, &true);
        env.storage().persistent().set(&DataKey::NextId, &0u64);
        env.storage()
            .persistent()
            .set(&DataKey::ListingNextId, &0u64);
        Self::emit_log(
            &env,
            LogLevel::Info,
            "initialize",
            Some(&admin),
            None,
            "initialized",
        );
        true
    }

    pub fn set_zk_verifier(env: Env, admin: Address, contract_id: Address) -> bool {
        admin.require_auth();
        if !Self::require_admin(&env, &admin) {
            return false;
        }
        env.storage()
            .persistent()
            .set(&DataKey::ZkVerifierContract, &contract_id);
        Self::emit_log(
            &env,
            LogLevel::Info,
            "set_zk_verifier",
            Some(&admin),
            None,
            "zk verifier set",
        );
        true
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
            .persistent()
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
        env.storage().persistent().set(&DataKey::NextId, &new_id);
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

    pub fn revoke_consent(env: Env, patient: Address, record_id: u64, grantee: Address) -> bool {
        patient.require_auth();
        let ce_opt = env
            .storage()
            .persistent()
            .get::<DataKey, ConsentEntry>(&DataKey::Consent(record_id, grantee.clone()));
        let mut entry = if let Some(entry) = ce_opt {
            if entry.patient != patient {
                return false;
            }
            entry
        } else {
            return false;
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
        true
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
            .persistent()
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
            .persistent()
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
        env.storage()
            .persistent()
            .set(&DataKey::ListingNextId, &lid);
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

    pub fn purchase_listing(env: Env, buyer: Address, listing_id: u64) -> bool {
        buyer.require_auth();
        let l_opt = env
            .storage()
            .persistent()
            .get::<DataKey, Listing>(&DataKey::Listing(listing_id));
        let mut listing = if let Some(listing) = l_opt {
            listing
        } else {
            return false;
        };
        if listing.status != ListingStatus::Active {
            return false;
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
        true
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
