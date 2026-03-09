#![no_std]
#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, BytesN, Env,
    String, Symbol, Vec,
};

// ==================== Genomic Data Formats ====================

/// Supported genomic data formats (FASTA, VCF, BAM, CRAM, BED, GFF)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum GenomicDataFormat {
    Fasta,
    Vcf,
    Bam,
    Cram,
    Bed,
    Gff,
}

// ==================== Consent Types ====================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum ConsentStatus {
    Active,
    Revoked,
    Expired,
    Suspended,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum ConsentPurpose {
    Research,
    ClinicalCare,
    Marketplace,
    AncestryAnalysis,
    DrugResponse,
}

// ==================== Breach Types ====================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum BreachSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum BreachStatus {
    Detected,
    Investigating,
    Contained,
    Resolved,
}

// ==================== Marketplace Types ====================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum MarketplaceListingStatus {
    Active,
    Sold,
    Expired,
    Cancelled,
}

// ==================== Analysis Types ====================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum AnalysisType {
    VariantCalling,
    RiskAssessment,
    AncestryPrediction,
    DrugResponse,
    PatternRecognition,
}

// ==================== Metabolizer Status ====================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum MetabolizerStatus {
    PoorMetabolizer,
    IntermediateMetabolizer,
    NormalMetabolizer,
    RapidMetabolizer,
    UltraRapidMetabolizer,
}

// ==================== Data Structs ====================

/// Core genomic data record — stores metadata about sequenced data on-chain
#[derive(Clone)]
#[contracttype]
pub struct GenomicDataRecord {
    pub id: u64,
    pub owner: Address,
    pub format: GenomicDataFormat,
    pub data_hash: BytesN<32>,
    pub compression_algo: String,
    pub encryption_key_ref: BytesN<32>,
    pub size_bytes: u64,
    pub quality_score: u32,
    pub reference_genome: String,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Genomic data consent record
#[derive(Clone)]
#[contracttype]
pub struct GenomicConsent {
    pub id: u64,
    pub patient: Address,
    pub grantee: Address,
    pub purpose: ConsentPurpose,
    pub scope: String,
    pub status: ConsentStatus,
    pub expiry: u64,
    pub revocable: bool,
    pub created_at: u64,
    pub revoked_at: u64,
}

/// Gene-disease association
#[derive(Clone)]
#[contracttype]
pub struct GeneDiseaseAssociation {
    pub id: u64,
    pub gene_id: String,
    pub disease_id: String,
    pub association_score: u32,
    pub evidence_level: u32,
    pub variant_info: String,
    pub reporter: Address,
    pub created_at: u64,
}

/// Pharmacogenomic profile for personalized medicine
#[derive(Clone)]
#[contracttype]
pub struct PharmacogenomicProfile {
    pub patient: Address,
    pub gene_id: String,
    pub drug_name: String,
    pub metabolizer_status: MetabolizerStatus,
    pub dosage_recommendation: String,
    pub risk_level: u32,
    pub updated_at: u64,
}

/// Ancestry and heritage record
#[derive(Clone)]
#[contracttype]
pub struct AncestryRecord {
    pub patient: Address,
    pub population_labels: Vec<String>,
    pub population_percentages: Vec<u32>,
    pub maternal_haplogroup: String,
    pub paternal_haplogroup: String,
    pub migration_markers: String,
    pub updated_at: u64,
}

/// Marketplace listing for genomic data
#[derive(Clone)]
#[contracttype]
pub struct MarketplaceListing {
    pub id: u64,
    pub data_record_id: u64,
    pub seller: Address,
    pub price: u64,
    pub anonymization_level: u32,
    pub licensing_terms: String,
    pub status: MarketplaceListingStatus,
    pub buyer: Address,
    pub created_at: u64,
    pub sold_at: u64,
}

/// Genomic analysis result
#[derive(Clone)]
#[contracttype]
pub struct GenomicAnalysisResult {
    pub id: u64,
    pub record_id: u64,
    pub analysis_type: AnalysisType,
    pub patterns_found: Vec<String>,
    pub confidence_score: u32,
    pub risk_factors: Vec<String>,
    pub analyst: Address,
    pub created_at: u64,
}

/// Privacy-preserving research data share
#[derive(Clone)]
#[contracttype]
pub struct ResearchDataShare {
    pub id: u64,
    pub record_id: u64,
    pub researcher: Address,
    pub institution: String,
    pub anonymization_proof: BytesN<32>,
    pub consent_id: u64,
    pub shared_at: u64,
}

/// Breach incident record
#[derive(Clone)]
#[contracttype]
pub struct BreachIncident {
    pub id: u64,
    pub affected_record_ids: Vec<u64>,
    pub severity: BreachSeverity,
    pub status: BreachStatus,
    pub description: String,
    pub response_actions: String,
    pub reporter: Address,
    pub detected_at: u64,
    pub resolved_at: u64,
}

// ==================== Storage Keys ====================

#[contracttype]
pub enum DataKey {
    Admin,
    Initialized,
    // Genomic records
    GenomicRecord(u64),
    GenomicRecordNextId,
    // Consents
    Consent(u64),
    ConsentNextId,
    // Gene-disease associations
    Association(u64),
    AssociationNextId,
    AssociationByGene(String),
    // Analyses
    Analysis(u64),
    AnalysisNextId,
    // Research shares
    ResearchShare(u64),
    ResearchShareNextId,
    // Marketplace
    Listing(u64),
    ListingNextId,
    // Pharmacogenomics
    PharmaProfile(Address),
    PharmaProfileCount,
    // Ancestry
    Ancestry(Address),
    // Breaches
    Breach(u64),
    BreachNextId,
}

// ==================== Errors ====================

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    RecordNotFound = 4,
    ConsentNotFound = 5,
    ConsentExpired = 6,
    ConsentRevoked = 7,
    NoValidConsent = 8,
    AssociationNotFound = 9,
    ListingNotFound = 10,
    ListingNotActive = 11,
    AnalysisNotFound = 12,
    BreachNotFound = 13,
    InvalidInput = 14,
    ProfileNotFound = 15,
    AncestryNotFound = 16,
    CannotPurchaseOwnListing = 17,
}

// ==================== Contract ====================

#[contract]
pub struct GenomicDataContract;

#[contractimpl]
impl GenomicDataContract {
    // ==================== Initialization ====================

    /// Initialize the genomic data contract
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::GenomicRecordNextId, &1u64);
        env.storage()
            .instance()
            .set(&DataKey::ConsentNextId, &1u64);
        env.storage()
            .instance()
            .set(&DataKey::AssociationNextId, &1u64);
        env.storage()
            .instance()
            .set(&DataKey::AnalysisNextId, &1u64);
        env.storage()
            .instance()
            .set(&DataKey::ResearchShareNextId, &1u64);
        env.storage()
            .instance()
            .set(&DataKey::ListingNextId, &1u64);
        env.storage()
            .instance()
            .set(&DataKey::BreachNextId, &1u64);
        env.storage()
            .instance()
            .set(&DataKey::PharmaProfileCount, &0u64);
        env.events()
            .publish((Symbol::new(&env, "Initialized"),), (admin,));
        Ok(())
    }

    // ==================== Genomic Data Storage (Criterion #1, #5) ====================

    /// Store genomic data with format, compression metadata, and encryption reference
    pub fn store_genomic_data(
        env: Env,
        owner: Address,
        format: GenomicDataFormat,
        data_hash: BytesN<32>,
        compression_algo: String,
        encryption_key_ref: BytesN<32>,
        size_bytes: u64,
        quality_score: u32,
        reference_genome: String,
    ) -> Result<u64, Error> {
        owner.require_auth();
        Self::require_initialized(&env)?;

        let id = Self::next_id(&env, &DataKey::GenomicRecordNextId);
        let now = env.ledger().timestamp();

        let record = GenomicDataRecord {
            id,
            owner: owner.clone(),
            format,
            data_hash,
            compression_algo,
            encryption_key_ref,
            size_bytes,
            quality_score,
            reference_genome,
            created_at: now,
            updated_at: now,
        };

        env.storage()
            .persistent()
            .set(&DataKey::GenomicRecord(id), &record);

        env.events().publish(
            (Symbol::new(&env, "GenomicDataStored"),),
            (id, owner, format),
        );
        Ok(id)
    }

    /// Retrieve a stored genomic data record
    pub fn get_genomic_data(env: Env, record_id: u64) -> Result<GenomicDataRecord, Error> {
        Self::require_initialized(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::GenomicRecord(record_id))
            .ok_or(Error::RecordNotFound)
    }

    // ==================== Consent Management (Criterion #6) ====================

    /// Grant consent for genomic data usage
    pub fn grant_consent(
        env: Env,
        patient: Address,
        grantee: Address,
        purpose: ConsentPurpose,
        scope: String,
        expiry: u64,
        revocable: bool,
    ) -> Result<u64, Error> {
        patient.require_auth();
        Self::require_initialized(&env)?;

        let id = Self::next_id(&env, &DataKey::ConsentNextId);
        let now = env.ledger().timestamp();

        let consent = GenomicConsent {
            id,
            patient: patient.clone(),
            grantee: grantee.clone(),
            purpose,
            scope,
            status: ConsentStatus::Active,
            expiry,
            revocable,
            created_at: now,
            revoked_at: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Consent(id), &consent);

        env.events().publish(
            (Symbol::new(&env, "ConsentGranted"),),
            (id, patient, grantee, purpose),
        );
        Ok(id)
    }

    /// Revoke a previously granted consent
    pub fn revoke_consent(env: Env, patient: Address, consent_id: u64) -> Result<(), Error> {
        patient.require_auth();
        Self::require_initialized(&env)?;

        let mut consent: GenomicConsent = env
            .storage()
            .persistent()
            .get(&DataKey::Consent(consent_id))
            .ok_or(Error::ConsentNotFound)?;

        if consent.patient != patient {
            return Err(Error::Unauthorized);
        }
        if consent.status == ConsentStatus::Revoked {
            return Err(Error::ConsentRevoked);
        }

        consent.status = ConsentStatus::Revoked;
        consent.revoked_at = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::Consent(consent_id), &consent);

        env.events()
            .publish((Symbol::new(&env, "ConsentRevoked"),), (consent_id, patient));
        Ok(())
    }

    /// Check whether a valid consent exists for a specific patient, grantee, and purpose
    pub fn check_consent(
        env: Env,
        patient: Address,
        grantee: Address,
        purpose: ConsentPurpose,
    ) -> Result<bool, Error> {
        Self::require_initialized(&env)?;

        let next_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::ConsentNextId)
            .unwrap_or(1);

        let now = env.ledger().timestamp();

        let mut i: u64 = 1;
        while i < next_id {
            if let Some(consent) = env
                .storage()
                .persistent()
                .get::<DataKey, GenomicConsent>(&DataKey::Consent(i))
            {
                if consent.patient == patient
                    && consent.grantee == grantee
                    && consent.purpose == purpose
                    && consent.status == ConsentStatus::Active
                    && (consent.expiry == 0 || consent.expiry > now)
                {
                    return Ok(true);
                }
            }
            i = i.saturating_add(1);
        }
        Ok(false)
    }

    // ==================== Gene-Disease Association (Criterion #4) ====================

    /// Register a gene-disease association with evidence
    pub fn register_gene_disease_assoc(
        env: Env,
        reporter: Address,
        gene_id: String,
        disease_id: String,
        association_score: u32,
        evidence_level: u32,
        variant_info: String,
    ) -> Result<u64, Error> {
        reporter.require_auth();
        Self::require_initialized(&env)?;

        let id = Self::next_id(&env, &DataKey::AssociationNextId);
        let now = env.ledger().timestamp();

        let assoc = GeneDiseaseAssociation {
            id,
            gene_id: gene_id.clone(),
            disease_id,
            association_score,
            evidence_level,
            variant_info,
            reporter: reporter.clone(),
            created_at: now,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Association(id), &assoc);

        // Store association IDs by gene for lookups
        let gene_key = DataKey::AssociationByGene(gene_id.clone());
        let mut ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&gene_key)
            .unwrap_or(Vec::new(&env));
        ids.push_back(id);
        env.storage().persistent().set(&gene_key, &ids);

        env.events().publish(
            (Symbol::new(&env, "AssociationRegistered"),),
            (id, gene_id, reporter),
        );
        Ok(id)
    }

    /// Query all disease associations for a given gene
    pub fn query_associations_by_gene(
        env: Env,
        gene_id: String,
    ) -> Result<Vec<GeneDiseaseAssociation>, Error> {
        Self::require_initialized(&env)?;

        let gene_key = DataKey::AssociationByGene(gene_id);
        let ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&gene_key)
            .unwrap_or(Vec::new(&env));

        let mut results: Vec<GeneDiseaseAssociation> = Vec::new(&env);
        for i in 0..ids.len() {
            let assoc_id = ids.get(i).unwrap();
            if let Some(assoc) = env
                .storage()
                .persistent()
                .get::<DataKey, GeneDiseaseAssociation>(&DataKey::Association(assoc_id))
            {
                results.push_back(assoc);
            }
        }
        Ok(results)
    }

    // ==================== Genomic Analysis & Pattern Recognition (Criterion #2) ====================

    /// Run / record a genomic analysis with pattern recognition results
    pub fn run_analysis(
        env: Env,
        analyst: Address,
        record_id: u64,
        analysis_type: AnalysisType,
        patterns_found: Vec<String>,
        confidence_score: u32,
        risk_factors: Vec<String>,
    ) -> Result<u64, Error> {
        analyst.require_auth();
        Self::require_initialized(&env)?;

        // Verify the genomic record exists
        if !env
            .storage()
            .persistent()
            .has(&DataKey::GenomicRecord(record_id))
        {
            return Err(Error::RecordNotFound);
        }

        let id = Self::next_id(&env, &DataKey::AnalysisNextId);
        let now = env.ledger().timestamp();

        let result = GenomicAnalysisResult {
            id,
            record_id,
            analysis_type,
            patterns_found,
            confidence_score,
            risk_factors,
            analyst: analyst.clone(),
            created_at: now,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Analysis(id), &result);

        env.events().publish(
            (Symbol::new(&env, "AnalysisCompleted"),),
            (id, record_id, analysis_type, analyst),
        );
        Ok(id)
    }

    /// Retrieve an analysis result
    pub fn get_analysis(env: Env, analysis_id: u64) -> Result<GenomicAnalysisResult, Error> {
        Self::require_initialized(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::Analysis(analysis_id))
            .ok_or(Error::AnalysisNotFound)
    }

    // ==================== Privacy-Preserving Research Sharing (Criterion #3) ====================

    /// Share genomic data for research with consent verification and anonymization proof
    pub fn share_for_research(
        env: Env,
        patient: Address,
        record_id: u64,
        researcher: Address,
        institution: String,
        anonymization_proof: BytesN<32>,
        consent_id: u64,
    ) -> Result<u64, Error> {
        patient.require_auth();
        Self::require_initialized(&env)?;

        // Verify the record exists
        if !env
            .storage()
            .persistent()
            .has(&DataKey::GenomicRecord(record_id))
        {
            return Err(Error::RecordNotFound);
        }

        // Verify consent is valid
        let consent: GenomicConsent = env
            .storage()
            .persistent()
            .get(&DataKey::Consent(consent_id))
            .ok_or(Error::ConsentNotFound)?;

        if consent.status != ConsentStatus::Active {
            return Err(Error::ConsentRevoked);
        }

        let now = env.ledger().timestamp();
        if consent.expiry != 0 && consent.expiry <= now {
            return Err(Error::ConsentExpired);
        }

        if consent.purpose != ConsentPurpose::Research {
            return Err(Error::NoValidConsent);
        }

        let id = Self::next_id(&env, &DataKey::ResearchShareNextId);

        let share = ResearchDataShare {
            id,
            record_id,
            researcher: researcher.clone(),
            institution,
            anonymization_proof,
            consent_id,
            shared_at: now,
        };

        env.storage()
            .persistent()
            .set(&DataKey::ResearchShare(id), &share);

        env.events().publish(
            (Symbol::new(&env, "DataSharedForResearch"),),
            (id, record_id, researcher),
        );
        Ok(id)
    }

    // ==================== Genomic Data Marketplace (Criterion #7) ====================

    /// Create a marketplace listing for genomic data
    pub fn create_marketplace_listing(
        env: Env,
        seller: Address,
        data_record_id: u64,
        price: u64,
        anonymization_level: u32,
        licensing_terms: String,
    ) -> Result<u64, Error> {
        seller.require_auth();
        Self::require_initialized(&env)?;

        // Verify record exists and seller owns it
        let record: GenomicDataRecord = env
            .storage()
            .persistent()
            .get(&DataKey::GenomicRecord(data_record_id))
            .ok_or(Error::RecordNotFound)?;

        if record.owner != seller {
            return Err(Error::Unauthorized);
        }

        let id = Self::next_id(&env, &DataKey::ListingNextId);
        let now = env.ledger().timestamp();

        let listing = MarketplaceListing {
            id,
            data_record_id,
            seller: seller.clone(),
            price,
            anonymization_level,
            licensing_terms,
            status: MarketplaceListingStatus::Active,
            buyer: seller.clone(), // placeholder, updated on purchase
            created_at: now,
            sold_at: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Listing(id), &listing);

        env.events().publish(
            (Symbol::new(&env, "ListingCreated"),),
            (id, data_record_id, seller, price),
        );
        Ok(id)
    }

    /// Purchase a marketplace listing (payment handled off-chain or via token contract)
    pub fn purchase_listing(
        env: Env,
        buyer: Address,
        listing_id: u64,
    ) -> Result<(), Error> {
        buyer.require_auth();
        Self::require_initialized(&env)?;

        let mut listing: MarketplaceListing = env
            .storage()
            .persistent()
            .get(&DataKey::Listing(listing_id))
            .ok_or(Error::ListingNotFound)?;

        if listing.status != MarketplaceListingStatus::Active {
            return Err(Error::ListingNotActive);
        }

        if listing.seller == buyer {
            return Err(Error::CannotPurchaseOwnListing);
        }

        listing.buyer = buyer.clone();
        listing.status = MarketplaceListingStatus::Sold;
        listing.sold_at = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::Listing(listing_id), &listing);

        env.events().publish(
            (Symbol::new(&env, "ListingPurchased"),),
            (listing_id, buyer, listing.price),
        );
        Ok(())
    }

    // ==================== Pharmacogenomics (Criterion #8) ====================

    /// Store a pharmacogenomic profile for personalized medicine
    pub fn store_pharmacogenomic_profile(
        env: Env,
        patient: Address,
        gene_id: String,
        drug_name: String,
        metabolizer_status: MetabolizerStatus,
        dosage_recommendation: String,
        risk_level: u32,
    ) -> Result<(), Error> {
        patient.require_auth();
        Self::require_initialized(&env)?;

        let profile = PharmacogenomicProfile {
            patient: patient.clone(),
            gene_id,
            drug_name,
            metabolizer_status,
            dosage_recommendation,
            risk_level,
            updated_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::PharmaProfile(patient.clone()), &profile);

        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PharmaProfileCount)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::PharmaProfileCount, &count.saturating_add(1));

        env.events()
            .publish((Symbol::new(&env, "PharmaProfileStored"),), (patient,));
        Ok(())
    }

    /// Retrieve a pharmacogenomic profile
    pub fn get_pharmacogenomic_profile(
        env: Env,
        patient: Address,
    ) -> Result<PharmacogenomicProfile, Error> {
        Self::require_initialized(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::PharmaProfile(patient))
            .ok_or(Error::ProfileNotFound)
    }

    // ==================== Ancestry & Heritage Tracking (Criterion #9) ====================

    /// Store ancestry and heritage data
    pub fn store_ancestry_record(
        env: Env,
        patient: Address,
        population_labels: Vec<String>,
        population_percentages: Vec<u32>,
        maternal_haplogroup: String,
        paternal_haplogroup: String,
        migration_markers: String,
    ) -> Result<(), Error> {
        patient.require_auth();
        Self::require_initialized(&env)?;

        if population_labels.len() != population_percentages.len() {
            return Err(Error::InvalidInput);
        }

        let record = AncestryRecord {
            patient: patient.clone(),
            population_labels,
            population_percentages,
            maternal_haplogroup,
            paternal_haplogroup,
            migration_markers,
            updated_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::Ancestry(patient.clone()), &record);

        env.events()
            .publish((Symbol::new(&env, "AncestryStored"),), (patient,));
        Ok(())
    }

    /// Retrieve ancestry data for a patient
    pub fn get_ancestry_record(env: Env, patient: Address) -> Result<AncestryRecord, Error> {
        Self::require_initialized(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::Ancestry(patient))
            .ok_or(Error::AncestryNotFound)
    }

    // ==================== Breach Detection & Response (Criterion #10) ====================

    /// Report a genomic data breach
    pub fn report_breach(
        env: Env,
        reporter: Address,
        affected_record_ids: Vec<u64>,
        severity: BreachSeverity,
        description: String,
        response_actions: String,
    ) -> Result<u64, Error> {
        reporter.require_auth();
        Self::require_initialized(&env)?;

        let id = Self::next_id(&env, &DataKey::BreachNextId);
        let now = env.ledger().timestamp();

        let incident = BreachIncident {
            id,
            affected_record_ids,
            severity,
            status: BreachStatus::Detected,
            description,
            response_actions,
            reporter: reporter.clone(),
            detected_at: now,
            resolved_at: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Breach(id), &incident);

        env.events()
            .publish((Symbol::new(&env, "BreachReported"),), (id, severity, reporter));
        Ok(id)
    }

    /// Update the status of a breach investigation (admin only)
    pub fn update_breach_status(
        env: Env,
        admin: Address,
        breach_id: u64,
        new_status: BreachStatus,
        updated_response: String,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_initialized(&env)?;
        Self::require_admin(&env, &admin)?;

        let mut incident: BreachIncident = env
            .storage()
            .persistent()
            .get(&DataKey::Breach(breach_id))
            .ok_or(Error::BreachNotFound)?;

        incident.status = new_status;
        incident.response_actions = updated_response;

        if new_status == BreachStatus::Resolved {
            incident.resolved_at = env.ledger().timestamp();
        }

        env.storage()
            .persistent()
            .set(&DataKey::Breach(breach_id), &incident);

        env.events().publish(
            (Symbol::new(&env, "BreachStatusUpdated"),),
            (breach_id, new_status),
        );
        Ok(())
    }

    /// Retrieve breach details
    pub fn get_breach(env: Env, breach_id: u64) -> Result<BreachIncident, Error> {
        Self::require_initialized(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::Breach(breach_id))
            .ok_or(Error::BreachNotFound)
    }

    // ==================== Internal Helpers ====================

    fn require_initialized(env: &Env) -> Result<(), Error> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::NotInitialized);
        }
        Ok(())
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if admin != *caller {
            return Err(Error::Unauthorized);
        }
        Ok(())
    }

    fn next_id(env: &Env, key: &DataKey) -> u64 {
        let current: u64 = env.storage().instance().get(key).unwrap_or(1);
        env.storage()
            .instance()
            .set(key, &current.saturating_add(1));
        current
    }
}
