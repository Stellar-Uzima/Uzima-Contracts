#![no_std]

// #[cfg(test)]
// mod test;

use soroban_sdk::symbol_short;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, vec, Address, BytesN, Env,
    Map, String, Symbol, Vec,
};

// ==================== Data Format Types ====================

/// Supported data formats
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum DataFormat {
    FHIRJSON = 0,
    FHIRXML = 1,
    HL7v2 = 2,
    CDA = 3,
    HL7v3 = 4,
    CCD = 5, // Continuity of Care Document
    C32 = 6, // Consolidated CDA
    PDF = 7,
    CSV = 8,
}

/// Field type information
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum FieldType {
    String,
    Integer,
    Decimal,
    DateTime,
    Boolean,
    Code,
    Array,
    Object,
}

/// Conversion rule for transforming data between formats
#[derive(Clone)]
#[contracttype]
pub struct ConversionRule {
    pub rule_id: String,
    pub source_format: DataFormat,
    pub target_format: DataFormat,
    pub source_path: String,         // JSON path or XPath
    pub target_path: String,         // JSON path or XPath
    pub transformation_type: String, // "direct", "mapped", "calculated", "lookup"
    pub field_type: FieldType,
    pub mapping_table_ref: String,     // Reference to mapping table
    pub validation_rules: Vec<String>, // Validation rules
    pub is_active: bool,
}

/// Healthcare coding mapping (e.g., ICD9 to ICD10)
#[derive(Clone)]
#[contracttype]
pub struct CodingMapping {
    pub mapping_id: String,
    pub source_code_system: String, // e.g., "ICD9", "ICD10"
    pub target_code_system: String, // e.g., "ICD10", "SNOMED-CT"
    pub source_code: String,        // e.g., "250.00"
    pub target_code: String,        // e.g., "E11.9"
    pub source_description: String,
    pub target_description: String,
    pub confidence_score: u32,            // 0-100 mapping confidence
    pub backward_mapping: Option<String>, // Reverse mapping code if applicable
    pub effective_date: String,
    pub end_date: String, // Empty if still active
}

/// Data format specification and metadata
#[derive(Clone)]
#[contracttype]
pub struct FormatSpecification {
    pub format: DataFormat,
    pub version: String, // e.g., "R4" for FHIR, "2.5.1" for HL7 v2
    pub mime_type: String,
    pub encoding: String, // UTF-8, UTF-16, etc.
    pub character_set: String,
    pub supported_resources: Vec<String>,
    pub description: String,
    pub standard_url: String,
}

/// Conversion request
#[derive(Clone)]
#[contracttype]
pub struct ConversionRequest {
    pub request_id: String,
    pub source_format: DataFormat,
    pub target_format: DataFormat,
    pub source_data_hash: BytesN<32>, // Hash of source data
    pub target_data_hash: BytesN<32>, // Hash of target data
    pub conversion_timestamp: u64,
    pub requester: Address,
    pub status: String, // pending, completed, failed
    pub error_details: String,
}

/// Validation result for data format conversion
#[derive(Clone)]
#[contracttype]
pub struct ValidationResult {
    pub validation_id: String,
    pub source_format: DataFormat,
    pub target_format: DataFormat,
    pub is_valid: bool,
    pub validation_errors: Vec<String>,
    pub validation_warnings: Vec<String>,
    pub validated_at: u64,
}

/// Lossy conversion warning (data loss during conversion)
#[derive(Clone)]
#[contracttype]
pub struct LossyConversionWarning {
    pub warning_id: String,
    pub conversion_request_id: String,
    pub lost_fields: Vec<String>,
    pub data_loss_percentage: u32, // 0-100
    pub mitigation_recommendation: String,
}

// Storage Keys
const ADMIN: Symbol = symbol_short!("ADMIN");
const CONVERSION_RULES: Symbol = symbol_short!("RULES");
const CODING_MAPPINGS: Symbol = symbol_short!("CODINGS");
const FORMAT_SPECS: Symbol = symbol_short!("FORMATS");
const CONVERSION_REQUESTS: Symbol = symbol_short!("REQUESTS");
const VALIDATION_RESULTS: Symbol = symbol_short!("VALIDATE");
const LOSSY_WARNINGS: Symbol = symbol_short!("WARNINGS");
const PAUSED: Symbol = symbol_short!("PAUSED");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    ContractPaused = 2,
    RuleNotFound = 3,
    CodingMappingNotFound = 4,
    FormatNotSupported = 5,
    ConversionFailed = 6,
    ValidationFailed = 7,
    InvalidConversionRequest = 8,
    SourceFormatNotSupported = 9,
    TargetFormatNotSupported = 10,
    MappingTableNotFound = 11,
    DuplicateRule = 12,
    IncompatibleFormats = 13,
    DataLossWarning = 14,
    InvalidMappingData = 15,
    OperationFailed = 16,
}

#[contract]
pub struct HealthcareDataConversionContract;

#[contractimpl]
impl HealthcareDataConversionContract {
    /// Initialize the healthcare data conversion contract
    pub fn initialize(env: Env, admin: Address) -> Result<bool, Error> {
        admin.require_auth();

        if env.storage().persistent().has(&ADMIN) {
            return Err(Error::OperationFailed);
        }

        env.storage().persistent().set(&ADMIN, &admin);
        env.storage().persistent().set(&PAUSED, &false);

        // Initialize default FHIR format specification
        let fhir_spec = FormatSpecification {
            format: DataFormat::FHIRJSON,
            version: String::from_str(&env, "R4"),
            mime_type: String::from_str(&env, "application/fhir+json"),
            encoding: String::from_str(&env, "UTF-8"),
            character_set: String::from_str(&env, "UTF-8"),
            supported_resources: vec![
                &env,
                String::from_str(&env, "Patient"),
                String::from_str(&env, "Observation"),
                String::from_str(&env, "Condition"),
                String::from_str(&env, "Medication"),
            ],
            description: String::from_str(&env, "HL7 FHIR Release 4 JSON Format"),
            standard_url: String::from_str(&env, "https://www.hl7.org/fhir/"),
        };

        let mut specs: Map<u32, FormatSpecification> = env
            .storage()
            .persistent()
            .get(&FORMAT_SPECS)
            .unwrap_or(Map::new(&env));

        specs.set(0, fhir_spec);
        env.storage().persistent().set(&FORMAT_SPECS, &specs);

        Ok(true)
    }

    /// Register a conversion rule
    pub fn register_conversion_rule(
        env: Env,
        admin: Address,
        rule: ConversionRule,
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

        let mut rules: Map<String, ConversionRule> = env
            .storage()
            .persistent()
            .get(&CONVERSION_RULES)
            .unwrap_or(Map::new(&env));

        if rules.contains_key(rule.rule_id.clone()) {
            return Err(Error::DuplicateRule);
        }

        rules.set(rule.rule_id.clone(), rule);
        env.storage().persistent().set(&CONVERSION_RULES, &rules);

        Ok(true)
    }

    /// Get conversion rule
    pub fn get_conversion_rule(env: Env, rule_id: String) -> Result<ConversionRule, Error> {
        let rules: Map<String, ConversionRule> = env
            .storage()
            .persistent()
            .get(&CONVERSION_RULES)
            .ok_or(Error::RuleNotFound)?;

        rules.get(rule_id).ok_or(Error::RuleNotFound)
    }

    /// Register healthcare coding mapping (e.g., ICD9 to ICD10)
    pub fn register_coding_mapping(
        env: Env,
        admin: Address,
        mapping: CodingMapping,
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

        // Validate mapping data
        if mapping.source_code.is_empty() || mapping.target_code.is_empty() {
            return Err(Error::InvalidMappingData);
        }

        if mapping.confidence_score > 100 {
            return Err(Error::InvalidMappingData);
        }

        let mut mappings: Map<String, CodingMapping> = env
            .storage()
            .persistent()
            .get(&CODING_MAPPINGS)
            .unwrap_or(Map::new(&env));

        mappings.set(mapping.mapping_id.clone(), mapping);
        env.storage().persistent().set(&CODING_MAPPINGS, &mappings);

        Ok(true)
    }

    /// Get coding mapping
    pub fn get_coding_mapping(env: Env, mapping_id: String) -> Result<CodingMapping, Error> {
        let mappings: Map<String, CodingMapping> = env
            .storage()
            .persistent()
            .get(&CODING_MAPPINGS)
            .ok_or(Error::CodingMappingNotFound)?;

        mappings.get(mapping_id).ok_or(Error::CodingMappingNotFound)
    }

    /// Get coding mapping by source and target codes
    #[allow(unused_variables)]
    pub fn find_coding_mapping(
        env: Env,
        _source_system: String,
        _target_system: String,
        _source_code: String,
    ) -> Result<CodingMapping, Error> {
        let _mappings: Map<String, CodingMapping> = env
            .storage()
            .persistent()
            .get(&CODING_MAPPINGS)
            .ok_or(Error::CodingMappingNotFound)?;

        // In a real implementation, this would search through the mappings
        // For now, we return error (would need proper indexing)
        Err(Error::CodingMappingNotFound)
    }

    /// Register format specification
    pub fn register_format_specification(
        env: Env,
        admin: Address,
        spec: FormatSpecification,
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

        let mut specs: Map<u32, FormatSpecification> = env
            .storage()
            .persistent()
            .get(&FORMAT_SPECS)
            .unwrap_or(Map::new(&env));

        let format_key = spec.format as u32;
        specs.set(format_key, spec);
        env.storage().persistent().set(&FORMAT_SPECS, &specs);

        Ok(true)
    }

    /// Get format specification
    pub fn get_format_specification(
        env: Env,
        format: DataFormat,
    ) -> Result<FormatSpecification, Error> {
        let specs: Map<u32, FormatSpecification> = env
            .storage()
            .persistent()
            .get(&FORMAT_SPECS)
            .ok_or(Error::FormatNotSupported)?;

        specs.get(format as u32).ok_or(Error::FormatNotSupported)
    }

    /// Validate data format conversion compatibility
    #[allow(unused_variables)]
    pub fn validate_conversion(
        env: Env,
        validator: Address,
        validation_id: String,
        source_format: DataFormat,
        target_format: DataFormat,
        _source_data_hash: BytesN<32>,
    ) -> Result<ValidationResult, Error> {
        validator.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Validate source and target formats are supported
        let specs: Map<u32, FormatSpecification> = env
            .storage()
            .persistent()
            .get(&FORMAT_SPECS)
            .ok_or(Error::FormatNotSupported)?;

        if !specs.contains_key(source_format as u32) {
            return Err(Error::SourceFormatNotSupported);
        }

        if !specs.contains_key(target_format as u32) {
            return Err(Error::TargetFormatNotSupported);
        }

        let validation_id = String::from_str(&env, "validation");

        let result = ValidationResult {
            validation_id: validation_id.clone(),
            source_format,
            target_format,
            is_valid: true,
            validation_errors: vec![&env],
            validation_warnings: vec![&env],
            validated_at: env.ledger().timestamp(),
        };

        let mut results: Map<String, ValidationResult> = env
            .storage()
            .persistent()
            .get(&VALIDATION_RESULTS)
            .unwrap_or(Map::new(&env));

        results.set(validation_id, result.clone());
        env.storage()
            .persistent()
            .set(&VALIDATION_RESULTS, &results);

        Ok(result)
    }

    /// Record a data conversion request
    pub fn record_conversion(
        env: Env,
        requester: Address,
        request_id: String,
        source_format: DataFormat,
        target_format: DataFormat,
        source_data_hash: BytesN<32>,
        target_data_hash: BytesN<32>,
    ) -> Result<String, Error> {
        requester.require_auth();

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        let request_id = String::from_str(&env, "conversion");

        let request = ConversionRequest {
            request_id: request_id.clone(),
            source_format,
            target_format,
            source_data_hash,
            target_data_hash,
            conversion_timestamp: env.ledger().timestamp(),
            requester,
            status: String::from_str(&env, "completed"),
            error_details: String::from_str(&env, ""),
        };

        let mut requests: Map<String, ConversionRequest> = env
            .storage()
            .persistent()
            .get(&CONVERSION_REQUESTS)
            .unwrap_or(Map::new(&env));

        requests.set(request_id.clone(), request);
        env.storage()
            .persistent()
            .set(&CONVERSION_REQUESTS, &requests);

        Ok(request_id)
    }

    /// Get conversion request details
    pub fn get_conversion_request(
        env: Env,
        request_id: String,
    ) -> Result<ConversionRequest, Error> {
        let requests: Map<String, ConversionRequest> = env
            .storage()
            .persistent()
            .get(&CONVERSION_REQUESTS)
            .ok_or(Error::InvalidConversionRequest)?;

        requests
            .get(request_id)
            .ok_or(Error::InvalidConversionRequest)
    }

    /// Record lossy conversion warning
    pub fn record_lossy_conversion_warning(
        env: Env,
        admin: Address,
        warning: LossyConversionWarning,
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

        // Validate data loss percentage
        if warning.data_loss_percentage > 100 {
            return Err(Error::InvalidConversionRequest);
        }

        let mut warnings: Map<String, LossyConversionWarning> = env
            .storage()
            .persistent()
            .get(&LOSSY_WARNINGS)
            .unwrap_or(Map::new(&env));

        warnings.set(warning.warning_id.clone(), warning);
        env.storage().persistent().set(&LOSSY_WARNINGS, &warnings);

        Ok(true)
    }

    /// Get lossy conversion warning
    pub fn get_lossy_conversion_warning(
        env: Env,
        warning_id: String,
    ) -> Result<LossyConversionWarning, Error> {
        let warnings: Map<String, LossyConversionWarning> = env
            .storage()
            .persistent()
            .get(&LOSSY_WARNINGS)
            .ok_or(Error::DataLossWarning)?;

        warnings.get(warning_id).ok_or(Error::DataLossWarning)
    }

    /// Pause contract operations
    pub fn pause(env: Env, admin: Address) -> Result<bool, Error> {
        admin.require_auth();

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

    /// Resume contract operations
    pub fn resume(env: Env, admin: Address) -> Result<bool, Error> {
        admin.require_auth();

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
}
