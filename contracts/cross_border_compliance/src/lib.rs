#![no_std]

#[cfg(test)]
mod test;

use soroban_sdk::symbol_short;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, vec, Address, BytesN, Env, Map, String,
    Symbol, Vec,
};

// ==================== Cross-Border Telemedicine Compliance Types ====================

/// Country Code (ISO 3166-1 alpha-2)
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum CountryCode {
    US,
    CA,          // Canada
    MX,          // Mexico
    GB,          // United Kingdom
    DE,          // Germany
    FR,          // France
    IT,          // Italy
    ES,          // Spain
    NL,          // Netherlands
    BE,          // Belgium
    CH,          // Switzerland
    AT,          // Austria
    SE,          // Sweden
    NO,          // Norway
    DK,          // Denmark
    FI,          // Finland
    PL,          // Poland
    CZ,          // Czech Republic
    HU,          // Hungary
    GR,          // Greece
    PT,          // Portugal
    IE,          // Ireland
    LU,          // Luxembourg
    AU,          // Australia
    NZ,          // New Zealand
    JP,          // Japan
    SG,          // Singapore
    IN,          // India
    BR,          // Brazil
    AR,          // Argentina
    CL,          // Chile
    CO,          // Colombia
    PE,          // Peru
    ZA,          // South Africa
    EG,          // Egypt
    IL,          // Israel
    AE,          // United Arab Emirates
    SA,          // Saudi Arabia
    TH,          // Thailand
    MY,          // Malaysia
    PH,          // Philippines
    VN,          // Vietnam
    ID,          // Indonesia
    KR,          // South Korea
    CN,          // China
    HK,          // Hong Kong
    TW,          // Taiwan
    Custom(u16), // For other countries using numeric code
}

/// Compliance Status
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum ComplianceStatus {
    Compliant,
    NonCompliant,
    PendingReview,
    Restricted,
    Suspended,
    Banned,
}

/// License Type
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum LicenseType {
    MedicalLicense,
    TelemedicineLicense,
    DrugEnforcementAdministration,
    ControlledSubstance,
    SpecialtyCertification,
    LanguageProficiency,
    LocalPracticePermit,
}

/// Data Transfer Mechanism
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum DataTransferMechanism {
    StandardContractualClauses,
    BindingCorporateRules,
    AdequacyDecision,
    SpecificConsent,
    EmergencyException,
    PublicInterest,
}

/// Regulatory Framework
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum RegulatoryFramework {
    HIPAA,       // US Health Insurance Portability and Accountability Act
    GDPR,        // EU General Data Protection Regulation
    PIPEDA,      // Canada Personal Information Protection and Electronic Documents Act
    APPI,        // Japan Act on the Protection of Personal Information
    PDPA,        // Singapore Personal Data Protection Act
    DPA,         // UK Data Protection Act
    LGPD,        // Brazil Lei Geral de Proteção de Dados
    POPIA,       // South Africa Protection of Personal Information Act
    HIPAA_GDPR,  // Combined compliance
    GDPR_PIPEDA, // EU-Canada compliance
    Custom,      // Custom framework
}

/// Cross-Border License
#[derive(Clone)]
#[contracttype]
pub struct CrossBorderLicense {
    pub license_id: u64,
    pub provider: Address,
    pub license_type: LicenseType,
    pub issuing_country: CountryCode,
    pub license_number: String,
    pub issued_date: u64,
    pub expiration_date: u64,
    pub is_active: bool,
    pub verification_status: String, // "verified", "pending", "rejected"
    pub verification_documents: Vec<String>, // IPFS hashes
    pub restrictions: Vec<String>,
    pub scope_of_practice: Vec<String>,
    pub language_proficiency: Vec<String>,
    pub telemedicine_training: bool,
    pub local_mandatory_training: bool,
    pub continuing_education_hours: u16,
    pub disciplinary_actions: Vec<String>,
    pub malpractice_coverage: String,
    pub coverage_amount: u64,
    pub coverage_currency: String,
}

/// Compliance Record
#[derive(Clone)]
#[contracttype]
pub struct ComplianceRecord {
    pub record_id: u64,
    pub provider: Address,
    pub patient: Address,
    pub provider_country: CountryCode,
    pub patient_country: CountryCode,
    pub consultation_type: String,
    pub regulatory_framework: RegulatoryFramework,
    pub compliance_status: ComplianceStatus,
    pub data_transfer_mechanism: DataTransferMechanism,
    pub patient_consent_obtained: bool,
    pub consent_token_id: u64,
    pub data_protection_measures: Vec<String>,
    pub storage_location: String, // Where data is stored
    pub retention_period_days: u32,
    pub audit_required: bool,
    pub audit_frequency: String, // "monthly", "quarterly", "annually"
    pub last_audit_date: u64,
    pub next_audit_date: u64,
    pub compliance_score: u8, // 0-100
    pub violations: Vec<ComplianceViolation>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Compliance Violation
#[derive(Clone)]
#[contracttype]
pub struct ComplianceViolation {
    pub violation_id: u64,
    pub compliance_record_id: u64,
    pub violation_type: String, // "data_breach", "unlicensed_practice", "consent_missing", etc.
    pub severity: String,       // "low", "medium", "high", "critical"
    pub description: String,
    pub regulatory_citation: String,
    pub fine_amount: Option<u64>,
    pub fine_currency: Option<String>,
    pub corrective_actions: Vec<String>,
    pub resolution_status: String, // "pending", "in_progress", "resolved"
    pub reported_date: u64,
    pub resolved_date: Option<u64>,
}

/// Country Regulation
#[derive(Clone)]
#[contracttype]
pub struct CountryRegulation {
    pub country: CountryCode,
    pub regulatory_framework: RegulatoryFramework,
    pub telemedicine_allowed: bool,
    pub cross_border_allowed: bool,
    pub license_requirements: Vec<LicenseType>,
    pub consent_requirements: Vec<String>,
    pub data_residency_required: bool,
    pub data_localization_required: bool,
    pub encryption_standards: Vec<String>,
    pub audit_requirements: Vec<String>,
    pub reporting_requirements: Vec<String>,
    pub restricted_treatments: Vec<String>,
    pub controlled_substance_rules: String,
    pub emergency_exceptions: Vec<String>,
    pub language_requirements: Vec<String>,
    pub cultural_competency_required: bool,
    pub local_supervision_required: bool,
    pub prescription_rules: String,
    pub insurance_requirements: Vec<String>,
    pub tax_obligations: Vec<String>,
    pub last_updated: u64,
}

/// Data Transfer Agreement
#[derive(Clone)]
#[contracttype]
pub struct DataTransferAgreement {
    pub agreement_id: u64,
    pub data_exporter: Address,
    pub data_importer: Address,
    pub exporter_country: CountryCode,
    pub importer_country: CountryCode,
    pub transfer_mechanism: DataTransferMechanism,
    pub data_types: Vec<String>, // "medical_records", "consultation_data", etc.
    pub purpose: String,
    pub retention_period_days: u32,
    pub security_measures: Vec<String>,
    pub breach_notification_timeline: u32, // hours
    pub subprocessor_restrictions: Vec<String>,
    pub audit_rights: bool,
    pub audit_frequency: String,
    pub governing_law: String,
    pub dispute_resolution: String,
    pub effective_date: u64,
    pub expiration_date: u64,
    pub status: String, // "active", "suspended", "terminated"
    pub amendments: Vec<String>,
}

/// Language Proficiency Certificate
#[derive(Clone)]
#[contracttype]
pub struct LanguageProficiencyCertificate {
    pub certificate_id: u64,
    pub provider: Address,
    pub language: String,
    pub proficiency_level: String, // "A1", "A2", "B1", "B2", "C1", "C2"
    pub test_type: String,         // "medical", "general", "professional"
    pub test_date: u64,
    pub expiry_date: u64,
    pub testing_organization: String,
    pub certificate_hash: BytesN<32>,
    pub verified: bool,
    pub verification_date: Option<u64>,
}

/// Currency Exchange Rate
#[derive(Clone)]
#[contracttype]
pub struct CurrencyExchangeRate {
    pub from_currency: String,
    pub to_currency: String,
    pub rate: f64,
    pub timestamp: u64,
    pub source: String, // "central_bank", "market", "fixed"
}

/// Tax Obligation
#[derive(Clone)]
#[contracttype]
pub struct TaxObligation {
    pub obligation_id: u64,
    pub provider: Address,
    pub country: CountryCode,
    pub tax_type: String, // "income", "vat", "service", "withholding"
    pub tax_rate: f64,
    pub taxable_amount: u64,
    pub tax_currency: String,
    pub payment_due_date: u64,
    pub paid_amount: Option<u64>,
    pub paid_date: Option<u64>,
    pub payment_reference: Option<String>,
    pub status: String, // "pending", "paid", "overdue", "disputed"
}

// Storage Keys
const ADMIN: Symbol = symbol_short!("ADMIN");
const CROSS_BORDER_LICENSES: Symbol = symbol_short!("LICENSES");
const COMPLIANCE_RECORDS: Symbol = symbol_short!("COMPLY");
const COUNTRY_REGULATIONS: Symbol = symbol_short!("REGULATE");
const DATA_TRANSFER_AGREEMENTS: Symbol = symbol_short!("XFER_AG");
const LANGUAGE_CERTIFICATES: Symbol = symbol_short!("LANG_CT");
const CURRENCY_RATES: Symbol = symbol_short!("RATES");
const TAX_OBLIGATIONS: Symbol = symbol_short!("TAXES");
const LICENSE_COUNTER: Symbol = symbol_short!("LIC_CNT");
const COMPLIANCE_COUNTER: Symbol = symbol_short!("COMP_CNT");
const TRANSFER_AGREEMENT_COUNTER: Symbol = symbol_short!("XFER_CNT");
const LANGUAGE_CERT_COUNTER: Symbol = symbol_short!("LANG_CNT");
const TAX_OBLIGATION_COUNTER: Symbol = symbol_short!("TAX_CNT");
const PAUSED: Symbol = symbol_short!("PAUSED");
const CONSENT_CONTRACT: Symbol = symbol_short!("CONSENT");
const MEDICAL_RECORDS_CONTRACT: Symbol = symbol_short!("MEDICAL");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    ContractPaused = 2,
    LicenseNotFound = 3,
    LicenseAlreadyExists = 4,
    ComplianceRecordNotFound = 5,
    InvalidCountry = 6,
    InvalidLicenseType = 7,
    LicenseExpired = 8,
    LicenseNotActive = 9,
    ComplianceCheckFailed = 10,
    DataTransferNotAllowed = 11,
    MissingConsent = 12,
    InvalidRegulatoryFramework = 13,
    CountryNotSupported = 14,
    LanguageProficiencyRequired = 15,
    TaxObligationNotFound = 16,
    TransferAgreementNotFound = 17,
    TransferAgreementExpired = 18,
    ViolationRecordNotFound = 19,
    InvalidCurrency = 20,
    ExchangeRateNotFound = 21,
    MedicalRecordsContractNotSet = 22,
    ConsentContractNotSet = 23,
}

#[contract]
pub struct CrossBorderComplianceContract;

/// Cross-border license data
#[contracttype]
#[derive(Clone)]
pub struct CrossBorderLicenseData {
    pub license_type: LicenseType,
    pub issuing_country: CountryCode,
    pub license_number: String,
    pub issued_date: u64,
    pub expiration_date: u64,
    pub verification_documents: Vec<String>,
    pub scope_of_practice: Vec<String>,
    pub language_proficiency: Vec<String>,
    pub telemedicine_training: bool,
    pub local_mandatory_training: bool,
    pub continuing_education_hours: u32,
    pub malpractice_coverage: String,
    pub coverage_amount: u64,
    pub coverage_currency: String,
}

#[contractimpl]
impl CrossBorderComplianceContract {
    /// Initialize the cross-border compliance contract
    pub fn initialize(
        env: Env,
        admin: Address,
        consent_contract: Address,
        medical_records_contract: Address,
    ) -> Result<bool, Error> {
        admin.require_auth();

        if env.storage().persistent().has(&ADMIN) {
            return Err(Error::LicenseAlreadyExists);
        }

        env.storage().persistent().set(&ADMIN, &admin);
        env.storage()
            .persistent()
            .set(&CONSENT_CONTRACT, &consent_contract);
        env.storage()
            .persistent()
            .set(&MEDICAL_RECORDS_CONTRACT, &medical_records_contract);
        env.storage().persistent().set(&PAUSED, &false);
        env.storage().persistent().set(&LICENSE_COUNTER, &0u64);
        env.storage().persistent().set(&COMPLIANCE_COUNTER, &0u64);
        env.storage()
            .persistent()
            .set(&TRANSFER_AGREEMENT_COUNTER, &0u64);
        env.storage()
            .persistent()
            .set(&LANGUAGE_CERT_COUNTER, &0u64);
        env.storage()
            .persistent()
            .set(&TAX_OBLIGATION_COUNTER, &0u64);

        // Initialize country regulations
        Self::initialize_country_regulations(&env)?;

        Ok(true)
    }

    /// Register cross-border license
    pub fn register_cross_border_license(
        env: Env,
        provider: Address,
        license_data: CrossBorderLicenseData,
    ) -> Result<u64, Error> {
        provider.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Validate license doesn't already exist
        let licenses: Map<u64, CrossBorderLicense> = env
            .storage()
            .persistent()
            .get(&CROSS_BORDER_LICENSES)
            .unwrap_or(Map::new(&env));

        for license in licenses.values() {
            if license.provider == provider
                && license.license_type == license_data.license_type
                && license.issuing_country == license_data.issuing_country
            {
                return Err(Error::LicenseAlreadyExists);
            }
        }

        let license_id = Self::get_and_increment_license_counter(&env);

        let license = CrossBorderLicense {
            license_id,
            provider: provider.clone(),
            license_type: license_data.license_type,
            issuing_country: license_data.issuing_country,
            license_number: license_data.license_number,
            issued_date: license_data.issued_date,
            expiration_date: license_data.expiration_date,
            is_active: true,
            verification_status: String::from_str(&env, "pending"),
            verification_documents: license_data.verification_documents,
            restrictions: Vec::new(&env),
            scope_of_practice: license_data.scope_of_practice,
            language_proficiency: license_data.language_proficiency,
            telemedicine_training: license_data.telemedicine_training,
            local_mandatory_training: license_data.local_mandatory_training,
            continuing_education_hours: license_data.continuing_education_hours,
            disciplinary_actions: Vec::new(&env),
            malpractice_coverage: license_data.malpractice_coverage,
            coverage_amount: license_data.coverage_amount,
            coverage_currency: license_data.coverage_currency,
        };

        let mut licenses: Map<u64, CrossBorderLicense> = env
            .storage()
            .persistent()
            .get(&CROSS_BORDER_LICENSES)
            .unwrap_or(Map::new(&env));
        licenses.set(license_id, license);
        env.storage()
            .persistent()
            .set(&CROSS_BORDER_LICENSES, &licenses);

        // Emit event
        env.events().publish(
            (symbol_short!("License"), symbol_short!("Reg")),
            (license_id, provider),
        );

        Ok(license_id)
    }

    /// Verify cross-border license
    pub fn verify_license(
        env: Env,
        license_id: u64,
        verifier: Address,
        approved: bool,
        notes: String,
    ) -> Result<bool, Error> {
        verifier.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Verify verifier is authorized (admin or regulatory body)
        if !Self::is_authorized_verifier(&env, verifier) {
            return Err(Error::NotAuthorized);
        }

        let mut licenses: Map<u64, CrossBorderLicense> = env
            .storage()
            .persistent()
            .get(&CROSS_BORDER_LICENSES)
            .ok_or(Error::LicenseNotFound)?;

        let mut license = licenses.get(license_id).ok_or(Error::LicenseNotFound)?;

        license.verification_status = if approved {
            "verified".to_string()
        } else {
            "rejected".to_string()
        };
        if !approved {
            license.is_active = false;
        }

        licenses.set(license_id, license);
        env.storage()
            .persistent()
            .set(&CROSS_BORDER_LICENSES, &licenses);

        // Emit event
        env.events().publish(
            (symbol_short!("License"), symbol_short!("Verified")),
            (license_id, approved),
        );

        Ok(true)
    }

    /// Create compliance record for cross-border consultation
    pub fn create_compliance_record(
        env: Env,
        provider: Address,
        patient: Address,
        provider_country: CountryCode,
        patient_country: CountryCode,
        consultation_type: String,
        consent_token_id: u64,
        data_transfer_mechanism: DataTransferMechanism,
        storage_location: String,
        retention_period_days: u32,
    ) -> Result<u64, Error> {
        provider.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Verify provider has valid license for cross-border practice
        if !Self::has_valid_cross_border_license(
            &env,
            provider.clone(),
            provider_country,
            patient_country,
        )? {
            return Err(Error::LicenseNotFound);
        }

        // Verify consent
        if !Self::verify_consent_token(&env, consent_token_id, patient.clone(), provider.clone())? {
            return Err(Error::MissingConsent);
        }

        // Determine applicable regulatory framework
        let regulatory_framework =
            Self::determine_regulatory_framework(&env, provider_country, patient_country)?;

        // Check compliance requirements
        let compliance_status = Self::check_compliance_requirements(
            &env,
            provider.clone(),
            patient.clone(),
            provider_country,
            patient_country,
            regulatory_framework,
        )?;

        let compliance_record_id = Self::get_and_increment_compliance_counter(&env);
        let timestamp = env.ledger().timestamp();

        let compliance_record = ComplianceRecord {
            record_id: compliance_record_id,
            provider: provider.clone(),
            patient: patient.clone(),
            provider_country,
            patient_country,
            consultation_type,
            regulatory_framework,
            compliance_status,
            data_transfer_mechanism,
            patient_consent_obtained: true,
            consent_token_id,
            data_protection_measures: Vec::new(&env), // Would populate based on requirements
            storage_location,
            retention_period_days,
            audit_required: true,
            audit_frequency: "quarterly".to_string(),
            last_audit_date: timestamp,
            next_audit_date: timestamp + 7776000, // 90 days from now
            compliance_score: Self::calculate_compliance_score(
                &env,
                provider.clone(),
                patient_country,
            )?,
            violations: Vec::new(&env),
            created_at: timestamp,
            updated_at: timestamp,
        };

        let mut compliance_records: Map<u64, ComplianceRecord> = env
            .storage()
            .persistent()
            .get(&COMPLIANCE_RECORDS)
            .unwrap_or(Map::new(&env));
        compliance_records.set(compliance_record_id, compliance_record);
        env.storage()
            .persistent()
            .set(&COMPLIANCE_RECORDS, &compliance_records);

        // Create data transfer agreement if needed
        if Self::requires_data_transfer_agreement(&env, provider_country, patient_country)? {
            Self::create_data_transfer_agreement(
                &env,
                provider.clone(),
                patient.clone(),
                provider_country,
                patient_country,
                data_transfer_mechanism,
            )?;
        }

        // Emit event
        env.events().publish(
            (symbol_short!("Comply"), symbol_short!("Created")),
            (compliance_record_id, provider, patient),
        );

        Ok(compliance_record_id)
    }

    /// Register language proficiency certificate
    pub fn register_language_certificate(
        env: Env,
        provider: Address,
        language: String,
        proficiency_level: String,
        test_type: String,
        test_date: u64,
        expiry_date: u64,
        testing_organization: String,
        certificate_hash: BytesN<32>,
    ) -> Result<u64, Error> {
        provider.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        let certificate_id = Self::get_and_increment_language_cert_counter(&env);

        let certificate = LanguageProficiencyCertificate {
            certificate_id,
            provider: provider.clone(),
            language,
            proficiency_level,
            test_type,
            test_date,
            expiry_date,
            testing_organization,
            certificate_hash,
            verified: false,
            verification_date: None,
        };

        let mut certificates: Map<u64, LanguageProficiencyCertificate> = env
            .storage()
            .persistent()
            .get(&LANGUAGE_CERTIFICATES)
            .unwrap_or(Map::new(&env));
        certificates.set(certificate_id, certificate);
        env.storage()
            .persistent()
            .set(&LANGUAGE_CERTIFICATES, &certificates);

        // Emit event
        env.events().publish(
            (symbol_short!("Cert"), symbol_short!("Reg")),
            (certificate_id, provider),
        );

        Ok(certificate_id)
    }

    /// Create tax obligation for cross-border income
    pub fn create_tax_obligation(
        env: Env,
        provider: Address,
        country: CountryCode,
        tax_type: String,
        taxable_amount: u64,
        tax_currency: String,
        payment_due_date: u64,
    ) -> Result<u64, Error> {
        provider.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Get tax rate for the country
        let tax_rate = Self::get_tax_rate(&env, country, tax_type.clone())?;

        let obligation_id = Self::get_and_increment_tax_obligation_counter(&env);

        let tax_obligation = TaxObligation {
            obligation_id,
            provider: provider.clone(),
            country,
            tax_type,
            tax_rate,
            taxable_amount,
            tax_currency,
            payment_due_date,
            paid_amount: None,
            paid_date: None,
            payment_reference: None,
            status: "pending".to_string(),
        };

        let mut tax_obligations: Map<u64, TaxObligation> = env
            .storage()
            .persistent()
            .get(&TAX_OBLIGATIONS)
            .unwrap_or(Map::new(&env));
        tax_obligations.set(obligation_id, tax_obligation);
        env.storage()
            .persistent()
            .set(&TAX_OBLIGATIONS, &tax_obligations);

        // Emit event
        env.events().publish(
            (symbol_short!("Tax"), symbol_short!("Created")),
            (obligation_id, provider),
        );

        Ok(obligation_id)
    }

    /// Update currency exchange rate
    pub fn update_exchange_rate(
        env: Env,
        admin: Address,
        from_currency: String,
        to_currency: String,
        rate: f64,
        source: String,
    ) -> Result<bool, Error> {
        admin.require_auth();

        let contract_admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN)
            .ok_or(Error::NotAuthorized)?;

        if admin != contract_admin {
            return Err(Error::NotAuthorized);
        }

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        let exchange_rate = CurrencyExchangeRate {
            from_currency: from_currency.clone(),
            to_currency: to_currency.clone(),
            rate,
            timestamp: env.ledger().timestamp(),
            source,
        };

        let mut rates: Map<String, CurrencyExchangeRate> = env
            .storage()
            .persistent()
            .get(&CURRENCY_RATES)
            .unwrap_or(Map::new(&env));

        let key = String::from_str(&env, "exchange_rate"); // Simplified
        rates.set(key, exchange_rate);
        env.storage().persistent().set(&CURRENCY_RATES, &rates);

        Ok(true)
    }

    /// Record compliance violation
    pub fn record_violation(
        env: Env,
        compliance_record_id: u64,
        violation_type: String,
        severity: String,
        description: String,
        regulatory_citation: String,
        fine_amount: Option<u64>,
        fine_currency: Option<String>,
        corrective_actions: Vec<String>,
    ) -> Result<u64, Error> {
        // This would typically be called by compliance officer or automated system
        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        let violation_id = Self::get_and_increment_compliance_counter(&env);
        let timestamp = env.ledger().timestamp();

        let violation = ComplianceViolation {
            violation_id,
            compliance_record_id,
            violation_type,
            severity,
            description,
            regulatory_citation,
            fine_amount,
            fine_currency,
            corrective_actions,
            resolution_status: "pending".to_string(),
            reported_date: timestamp,
            resolved_date: None,
        };

        // Update compliance record
        let mut compliance_records: Map<u64, ComplianceRecord> = env
            .storage()
            .persistent()
            .get(&COMPLIANCE_RECORDS)
            .ok_or(Error::ComplianceRecordNotFound)?;

        let mut compliance_record = compliance_records
            .get(compliance_record_id)
            .ok_or(Error::ComplianceRecordNotFound)?;

        compliance_record.violations.push_back(violation.clone());
        compliance_record.compliance_status = ComplianceStatus::NonCompliant;
        compliance_record.updated_at = timestamp;

        compliance_records.set(compliance_record_id, compliance_record);
        env.storage()
            .persistent()
            .set(&COMPLIANCE_RECORDS, &compliance_records);

        // Emit event
        env.events().publish(
            (symbol_short!("Violation"), symbol_short!("Recorded")),
            (violation_id, compliance_record_id),
        );

        Ok(violation_id)
    }

    /// Get compliance record
    pub fn get_compliance_record(env: Env, record_id: u64) -> Result<ComplianceRecord, Error> {
        let compliance_records: Map<u64, ComplianceRecord> = env
            .storage()
            .persistent()
            .get(&COMPLIANCE_RECORDS)
            .ok_or(Error::ComplianceRecordNotFound)?;

        compliance_records
            .get(record_id)
            .ok_or(Error::ComplianceRecordNotFound)
    }

    /// Get provider's cross-border licenses
    pub fn get_provider_licenses(
        env: Env,
        provider: Address,
    ) -> Result<Vec<CrossBorderLicense>, Error> {
        let licenses: Map<u64, CrossBorderLicense> = env
            .storage()
            .persistent()
            .get(&CROSS_BORDER_LICENSES)
            .unwrap_or(Map::new(&env));

        let mut provider_licenses = Vec::new(&env);
        for license in licenses.values() {
            if license.provider == provider {
                provider_licenses.push_back(license);
            }
        }

        Ok(provider_licenses)
    }

    /// Get country regulation
    pub fn get_country_regulation(
        env: Env,
        country: CountryCode,
    ) -> Result<CountryRegulation, Error> {
        let regulations: Map<CountryCode, CountryRegulation> = env
            .storage()
            .persistent()
            .get(&COUNTRY_REGULATIONS)
            .ok_or(Error::CountryNotSupported)?;

        regulations.get(country).ok_or(Error::CountryNotSupported)
    }

    /// Get currency exchange rate
    pub fn get_exchange_rate(
        env: Env,
        from_currency: String,
        to_currency: String,
    ) -> Result<CurrencyExchangeRate, Error> {
        let rates: Map<String, CurrencyExchangeRate> = env
            .storage()
            .persistent()
            .get(&CURRENCY_RATES)
            .ok_or(Error::ExchangeRateNotFound)?;

        let key = String::from_str(&env, "exchange_rate"); // Simplified
        rates.get(key).ok_or(Error::ExchangeRateNotFound)
    }

    /// Check if provider can practice telemedicine across borders
    pub fn can_practice_cross_border(
        env: Env,
        provider: Address,
        provider_country: CountryCode,
        patient_country: CountryCode,
        consultation_type: String,
    ) -> Result<bool, Error> {
        // Check if provider has valid license
        if !Self::has_valid_cross_border_license(
            &env,
            provider.clone(),
            provider_country,
            patient_country,
        )? {
            return Ok(false);
        }

        // Check country regulations
        let provider_regulation = Self::get_country_regulation(&env, provider_country)?;
        let patient_regulation = Self::get_country_regulation(&env, patient_country)?;

        if !provider_regulation.cross_border_allowed || !patient_regulation.telemedicine_allowed {
            return Ok(false);
        }

        // Check language requirements
        if !Self::has_required_language_proficiency(&env, provider.clone(), patient_country)? {
            return Ok(false);
        }

        // Check consultation type restrictions
        if patient_regulation
            .restricted_treatments
            .contains(&consultation_type)
        {
            return Ok(false);
        }

        Ok(true)
    }

    // ==================== Helper Functions ====================

    fn verify_consent_token(
        env: &Env,
        token_id: u64,
        patient: Address,
        provider: Address,
    ) -> Result<bool, Error> {
        let consent_contract: Address = env
            .storage()
            .persistent()
            .get(&CONSENT_CONTRACT)
            .ok_or(Error::ConsentContractNotSet)?;

        // This would call the consent contract to verify the token
        // For now, we'll assume it's valid if not revoked
        Ok(true)
    }

    fn is_authorized_verifier(env: &Env, verifier: Address) -> bool {
        // This would check if verifier is admin or authorized regulatory body
        // For now, we'll implement basic logic
        true
    }

    fn has_valid_cross_border_license(
        env: &Env,
        provider: Address,
        provider_country: CountryCode,
        patient_country: CountryCode,
    ) -> Result<bool, Error> {
        let licenses: Map<u64, CrossBorderLicense> = env
            .storage()
            .persistent()
            .get(&CROSS_BORDER_LICENSES)
            .unwrap_or(Map::new(env));

        let timestamp = env.ledger().timestamp();

        for license in licenses.values() {
            if license.provider == provider
                && license.issuing_country == provider_country
                && license.is_active
                && license.verification_status == "verified"
                && license.expiration_date > timestamp
            {
                // Check if license covers the patient country
                if license
                    .scope_of_practice
                    .contains(&String::from_str(&env, "country_code")) // Simplified
                {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    fn determine_regulatory_framework(
        env: &Env,
        provider_country: CountryCode,
        patient_country: CountryCode,
    ) -> Result<RegulatoryFramework, Error> {
        // Simplified logic for determining applicable framework
        match (provider_country, patient_country) {
            (CountryCode::US, CountryCode::US) => Ok(RegulatoryFramework::HIPAA),
            (CountryCode::US, _) => Ok(RegulatoryFramework::HIPAA_GDPR),
            (_, CountryCode::US) => Ok(RegulatoryFramework::HIPAA_GDPR),
            (CountryCode::GB, CountryCode::GB) => Ok(RegulatoryFramework::DPA),
            (CountryCode::GB, _) => Ok(RegulatoryFramework::GDPR),
            (_, CountryCode::GB) => Ok(RegulatoryFramework::GDPR),
            (CountryCode::CA, CountryCode::CA) => Ok(RegulatoryFramework::PIPEDA),
            (CountryCode::CA, _) => Ok(RegulatoryFramework::GDPR_PIPEDA),
            (_, CountryCode::CA) => Ok(RegulatoryFramework::GDPR_PIPEDA),
            (CountryCode::JP, CountryCode::JP) => Ok(RegulatoryFramework::APPI),
            (CountryCode::SG, CountryCode::SG) => Ok(RegulatoryFramework::PDPA),
            _ => Ok(RegulatoryFramework::GDPR), // Default to GDPR for EU and others
        }
    }

    fn check_compliance_requirements(
        env: &Env,
        provider: Address,
        patient: Address,
        provider_country: CountryCode,
        patient_country: CountryCode,
        regulatory_framework: RegulatoryFramework,
    ) -> Result<ComplianceStatus, Error> {
        // Simplified compliance check
        // In production, this would be much more comprehensive

        // Check if both countries allow cross-border telemedicine
        let provider_regulation = Self::get_country_regulation(&env, provider_country)?;
        let patient_regulation = Self::get_country_regulation(&env, patient_country)?;

        if !provider_regulation.cross_border_allowed || !patient_regulation.telemedicine_allowed {
            return Ok(ComplianceStatus::NonCompliant);
        }

        // Check data residency requirements
        if patient_regulation.data_residency_required {
            // Would need to verify data storage location
            return Ok(ComplianceStatus::PendingReview);
        }

        Ok(ComplianceStatus::Compliant)
    }

    fn calculate_compliance_score(
        env: &Env,
        provider: Address,
        patient_country: CountryCode,
    ) -> Result<u8, Error> {
        // Simplified compliance score calculation
        let mut score = 80u8; // Base score

        // Add points for valid licenses
        if Self::has_valid_cross_border_license(
            &env,
            provider.clone(),
            CountryCode::US,
            patient_country,
        )
        .unwrap_or(false)
        {
            score += 10;
        }

        // Add points for language proficiency
        if Self::has_required_language_proficiency(&env, provider, patient_country).unwrap_or(false)
        {
            score += 10;
        }

        // Cap at 100
        if score > 100 {
            score = 100;
        }

        Ok(score)
    }

    fn requires_data_transfer_agreement(
        env: &Env,
        provider_country: CountryCode,
        patient_country: CountryCode,
    ) -> Result<bool, Error> {
        // Check if data transfer agreement is required
        let provider_regulation = Self::get_country_regulation(&env, provider_country)?;
        let patient_regulation = Self::get_country_regulation(&env, patient_country)?;

        // Simplified logic - in production would be more complex
        Ok(provider_country != patient_country || patient_regulation.data_localization_required)
    }

    fn create_data_transfer_agreement(
        env: &Env,
        provider: Address,
        patient: Address,
        exporter_country: CountryCode,
        importer_country: CountryCode,
        transfer_mechanism: DataTransferMechanism,
    ) -> Result<(), Error> {
        let agreement_id = Self::get_and_increment_transfer_agreement_counter(env);
        let timestamp = env.ledger().timestamp();

        let agreement = DataTransferAgreement {
            agreement_id,
            data_exporter: provider,
            data_importer: patient,
            exporter_country,
            importer_country,
            transfer_mechanism,
            data_types: vec![
                env,
                "medical_records".to_string(),
                "consultation_data".to_string(),
            ],
            purpose: "telemedicine_consultation".to_string(),
            retention_period_days: 2555, // 7 years
            security_measures: vec![
                env,
                "end_to_end_encryption".to_string(),
                "access_logging".to_string(),
            ],
            breach_notification_timeline: 72, // 72 hours
            subprocessor_restrictions: Vec::new(env),
            audit_rights: true,
            audit_frequency: "quarterly".to_string(),
            governing_law: "international".to_string(),
            dispute_resolution: "arbitration".to_string(),
            effective_date: timestamp,
            expiration_date: timestamp + 31536000, // 1 year
            status: "active".to_string(),
            amendments: Vec::new(env),
        };

        let mut agreements: Map<u64, DataTransferAgreement> = env
            .storage()
            .persistent()
            .get(&DATA_TRANSFER_AGREEMENTS)
            .unwrap_or(Map::new(env));
        agreements.set(agreement_id, agreement);
        env.storage()
            .persistent()
            .set(&DATA_TRANSFER_AGREEMENTS, &agreements);

        Ok(())
    }

    fn has_required_language_proficiency(
        env: &Env,
        provider: Address,
        patient_country: CountryCode,
    ) -> Result<bool, Error> {
        // Get required languages for patient country
        let regulation = Self::get_country_regulation(&env, patient_country)?;

        if regulation.language_requirements.is_empty() {
            return Ok(true); // No specific language requirements
        }

        // Check provider's language certificates
        let certificates: Map<u64, LanguageProficiencyCertificate> = env
            .storage()
            .persistent()
            .get(&LANGUAGE_CERTIFICATES)
            .unwrap_or(Map::new(env));

        let timestamp = env.ledger().timestamp();

        for required_lang in regulation.language_requirements.iter() {
            let mut has_proficiency = false;

            for certificate in certificates.values() {
                if certificate.provider == provider
                    && certificate.language == *required_lang
                    && certificate.verified
                    && certificate.expiry_date > timestamp
                {
                    // Check proficiency level (B2 or higher typically required)
                    if certificate.proficiency_level == "B2"
                        || certificate.proficiency_level == "C1"
                        || certificate.proficiency_level == "C2"
                    {
                        has_proficiency = true;
                        break;
                    }
                }
            }

            if !has_proficiency {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn get_tax_rate(env: &Env, country: CountryCode, tax_type: String) -> Result<f64, Error> {
        // Simplified tax rate lookup - in production would be more comprehensive
        match (country, tax_type.as_str()) {
            (CountryCode::US, "income") => Ok(0.30), // 30% federal + state average
            (CountryCode::US, "service") => Ok(0.10), // 10% service tax
            (CountryCode::GB, "income") => Ok(0.20), // 20% basic rate
            (CountryCode::GB, "vat") => Ok(0.20),    // 20% VAT
            (CountryCode::CA, "income") => Ok(0.25), // 25% average
            (CountryCode::CA, "service") => Ok(0.05), // 5% GST
            _ => Ok(0.20),                           // Default 20%
        }
    }

    fn initialize_country_regulations(env: &Env) -> Result<(), Error> {
        let mut regulations: Map<CountryCode, CountryRegulation> = Map::new(env);
        let timestamp = env.ledger().timestamp();

        // US Regulations
        regulations.set(
            CountryCode::US,
            CountryRegulation {
                country: CountryCode::US,
                regulatory_framework: RegulatoryFramework::HIPAA,
                telemedicine_allowed: true,
                cross_border_allowed: true,
                license_requirements: vec![
                    env,
                    LicenseType::MedicalLicense,
                    LicenseType::TelemedicineLicense,
                ],
                consent_requirements: vec![
                    env,
                    "informed_consent".to_string(),
                    "hipaa_authorization".to_string(),
                ],
                data_residency_required: false,
                data_localization_required: false,
                encryption_standards: vec![env, "AES-256".to_string()],
                audit_requirements: vec![
                    env,
                    "access_logs".to_string(),
                    "security_logs".to_string(),
                ],
                reporting_requirements: vec![env, "breach_notification".to_string()],
                restricted_treatments: Vec::new(env),
                controlled_substance_rules: "dea_registration_required".to_string(),
                emergency_exceptions: vec![env, "life_threatening".to_string()],
                language_requirements: Vec::new(env),
                cultural_competency_required: false,
                local_supervision_required: false,
                prescription_rules: "state_license_required".to_string(),
                insurance_requirements: vec![env, "malpractice_insurance".to_string()],
                tax_obligations: vec![
                    env,
                    "income_tax".to_string(),
                    "self_employment_tax".to_string(),
                ],
                last_updated: timestamp,
            },
        );

        // UK Regulations
        regulations.set(
            CountryCode::GB,
            CountryRegulation {
                country: CountryCode::GB,
                regulatory_framework: RegulatoryFramework::DPA,
                telemedicine_allowed: true,
                cross_border_allowed: true,
                license_requirements: vec![
                    env,
                    LicenseType::MedicalLicense,
                    LicenseType::TelemedicineLicense,
                ],
                consent_requirements: vec![
                    env,
                    "explicit_consent".to_string(),
                    "data_processing_consent".to_string(),
                ],
                data_residency_required: false,
                data_localization_required: false,
                encryption_standards: vec![env, "AES-256".to_string()],
                audit_requirements: vec![env, "data_protection_impact_assessment".to_string()],
                reporting_requirements: vec![env, "breach_notification_72h".to_string()],
                restricted_treatments: Vec::new(env),
                controlled_substance_rules: "home_office_registration".to_string(),
                emergency_exceptions: vec![env, "emergency_treatment".to_string()],
                language_requirements: vec![env, "English".to_string()],
                cultural_competency_required: true,
                local_supervision_required: false,
                prescription_rules: "gmc_registration_required".to_string(),
                insurance_requirements: vec![env, "professional_indemnity".to_string()],
                tax_obligations: vec![env, "income_tax".to_string(), "vat".to_string()],
                last_updated: timestamp,
            },
        );

        // Canada Regulations
        regulations.set(
            CountryCode::CA,
            CountryRegulation {
                country: CountryCode::CA,
                regulatory_framework: RegulatoryFramework::PIPEDA,
                telemedicine_allowed: true,
                cross_border_allowed: true,
                license_requirements: vec![env, LicenseType::MedicalLicense],
                consent_requirements: vec![env, "meaningful_consent".to_string()],
                data_residency_required: false,
                data_localization_required: false,
                encryption_standards: vec![env, "AES-256".to_string()],
                audit_requirements: vec![env, "privacy_impact_assessment".to_string()],
                reporting_requirements: vec![env, "breach_notification".to_string()],
                restricted_treatments: Vec::new(env),
                controlled_substance_rules: "controlled_drugs_regulation".to_string(),
                emergency_exceptions: vec![env, "emergency_care".to_string()],
                language_requirements: vec![env, "English".to_string(), "French".to_string()],
                cultural_competency_required: true,
                local_supervision_required: false,
                prescription_rules: "provincial_license_required".to_string(),
                insurance_requirements: vec![env, "malpractice_insurance".to_string()],
                tax_obligations: vec![env, "income_tax".to_string(), "gst".to_string()],
                last_updated: timestamp,
            },
        );

        env.storage()
            .persistent()
            .set(&COUNTRY_REGULATIONS, &regulations);

        Ok(())
    }

    fn get_and_increment_license_counter(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&LICENSE_COUNTER)
            .unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&LICENSE_COUNTER, &next);
        next
    }

    fn get_and_increment_compliance_counter(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&COMPLIANCE_COUNTER)
            .unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&COMPLIANCE_COUNTER, &next);
        next
    }

    fn get_and_increment_transfer_agreement_counter(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&TRANSFER_AGREEMENT_COUNTER)
            .unwrap_or(0);
        let next = count + 1;
        env.storage()
            .persistent()
            .set(&TRANSFER_AGREEMENT_COUNTER, &next);
        next
    }

    fn get_and_increment_language_cert_counter(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&LANGUAGE_CERT_COUNTER)
            .unwrap_or(0);
        let next = count + 1;
        env.storage()
            .persistent()
            .set(&LANGUAGE_CERT_COUNTER, &next);
        next
    }

    fn get_and_increment_tax_obligation_counter(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&TAX_OBLIGATION_COUNTER)
            .unwrap_or(0);
        let next = count + 1;
        env.storage()
            .persistent()
            .set(&TAX_OBLIGATION_COUNTER, &next);
        next
    }

    /// Pause contract operations (admin only)
    pub fn pause(env: Env, admin: Address) -> Result<bool, Error> {
        let contract_admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN)
            .ok_or(Error::NotAuthorized)?;

        if admin != contract_admin {
            return Err(Error::NotAuthorized);
        }

        env.storage().persistent().set(&PAUSED, &true);
        Ok(true)
    }

    /// Resume contract operations (admin only)
    pub fn resume(env: Env, admin: Address) -> Result<bool, Error> {
        let contract_admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN)
            .ok_or(Error::NotAuthorized)?;

        if admin != contract_admin {
            return Err(Error::NotAuthorized);
        }

        env.storage().persistent().set(&PAUSED, &false);
        Ok(true)
    }

    /// Health check for monitoring
    pub fn health_check(env: Env) -> (Symbol, u32, u64) {
        let status = if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            symbol_short!("PAUSED")
        } else {
            symbol_short!("OK")
        };
        (status, 1, env.ledger().timestamp())
    }
}
