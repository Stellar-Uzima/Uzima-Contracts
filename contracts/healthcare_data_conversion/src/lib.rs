#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, vec, Address, BytesN, Env,
    Map, String, Symbol, Vec,
};

// ==================== Data Format Types ====================

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum DataFormat {
    FHIRJSON = 0,
    FHIRXML = 1,
    HL7v2 = 2,
    CDA = 3,
    HL7v3 = 4,
    CCD = 5,
    C32 = 6,
    PDF = 7,
    CSV = 8,
}

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

#[derive(Clone)]
#[contracttype]
pub struct ConversionRule {
    pub rule_id: String,
    pub source_format: DataFormat,
    pub target_format: DataFormat,
    pub source_path: String,
    pub target_path: String,
    pub transformation_type: String,
    pub field_type: FieldType,
    pub mapping_table_ref: String,
    pub validation_rules: Vec<String>,
    pub is_active: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct CodingMapping {
    pub mapping_id: String,
    pub source_code_system: String,
    pub target_code_system: String,
    pub source_code: String,
    pub target_code: String,
    pub source_description: String,
    pub target_description: String,
    pub confidence_score: u32,
    pub backward_mapping: Option<String>,
    pub effective_date: String,
    pub end_date: String,
}

#[derive(Clone)]
#[contracttype]
pub struct FormatSpecification {
    pub format: DataFormat,
    pub version: String,
    pub mime_type: String,
    pub encoding: String,
    pub character_set: String,
    pub supported_resources: Vec<String>,
    pub description: String,
    pub standard_url: String,
}

#[derive(Clone)]
#[contracttype]
pub struct ConversionRequest {
    pub request_id: u64,
    pub source_format: DataFormat,
    pub target_format: DataFormat,
    pub source_data_hash: BytesN<32>,
    pub target_data_hash: BytesN<32>,
    pub conversion_timestamp: u64,
    pub requester: Address,
    pub status: String,
    pub error_details: String,
}

#[derive(Clone)]
#[contracttype]
pub struct ValidationResult {
    pub validation_id: u64,
    pub source_format: DataFormat,
    pub target_format: DataFormat,
    pub is_valid: bool,
    pub validation_errors: Vec<String>,
    pub validation_warnings: Vec<String>,
    pub validated_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct LossyConversionWarning {
    pub warning_id: String,
    pub conversion_request_id: u64,
    pub lost_fields: Vec<String>,
    pub data_loss_percentage: u32,
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

const NEXT_CONVERSION_ID: Symbol = symbol_short!("REQ_NXT");
const NEXT_VALIDATION_ID: Symbol = symbol_short!("VAL_NXT");

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
    pub fn initialize(env: Env, admin: Address) -> Result<bool, Error> {
        admin.require_auth();
        if env.storage().persistent().has(&ADMIN) {
            return Err(Error::OperationFailed);
        }
        env.storage().persistent().set(&ADMIN, &admin);
        env.storage().persistent().set(&PAUSED, &false);
        env.storage().persistent().set(&NEXT_CONVERSION_ID, &0u64);
        env.storage().persistent().set(&NEXT_VALIDATION_ID, &0u64);

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
            ],
            description: String::from_str(&env, "HL7 FHIR R4"),
            standard_url: String::from_str(&env, "https://hl7.org/fhir/"),
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

    pub fn register_conversion_rule(
        env: Env,
        admin: Address,
        rule: ConversionRule,
    ) -> Result<bool, Error> {
        admin.require_auth();
        let stored_admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN)
            .ok_or(Error::NotAuthorized)?;
        if admin != stored_admin {
            return Err(Error::NotAuthorized);
        }
        let mut rules: Map<String, ConversionRule> = env
            .storage()
            .persistent()
            .get(&CONVERSION_RULES)
            .unwrap_or(Map::new(&env));
        rules.set(rule.rule_id.clone(), rule);
        env.storage().persistent().set(&CONVERSION_RULES, &rules);
        Ok(true)
    }

    pub fn get_conversion_rule(env: Env, rule_id: String) -> Result<ConversionRule, Error> {
        let rules: Map<String, ConversionRule> = env
            .storage()
            .persistent()
            .get(&CONVERSION_RULES)
            .ok_or(Error::RuleNotFound)?;
        rules.get(rule_id).ok_or(Error::RuleNotFound)
    }

    pub fn register_coding_mapping(
        env: Env,
        admin: Address,
        mapping: CodingMapping,
    ) -> Result<bool, Error> {
        admin.require_auth();
        let stored_admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN)
            .ok_or(Error::NotAuthorized)?;
        if admin != stored_admin {
            return Err(Error::NotAuthorized);
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

    pub fn find_coding_mapping(
        _env: Env,
        _src_sys: String,
        _tgt_sys: String,
        _src_code: String,
    ) -> Result<CodingMapping, Error> {
        // Mock search logic to satisfy interface
        Err(Error::CodingMappingNotFound)
    }

    pub fn validate_conversion(
        env: Env,
        validator: Address,
        _val_id: String,
        source_format: DataFormat,
        target_format: DataFormat,
        _hash: BytesN<32>,
    ) -> Result<ValidationResult, Error> {
        validator.require_auth();
        let result = ValidationResult {
            validation_id,
            source_format,
            target_format,
            is_valid: true,
            validation_errors: vec![&env],
            validation_warnings: vec![&env],
            validated_at: env.ledger().timestamp(),
        };
        Ok(result)
    }

    pub fn record_conversion(
        env: Env,
        requester: Address,
        _req_id: String,
        source_format: DataFormat,
        target_format: DataFormat,
        source_data_hash: BytesN<32>,
        target_data_hash: BytesN<32>,
    ) -> Result<u64, Error> {
        requester.require_auth();
        let request = ConversionRequest {
            request_id,
            source_format,
            target_format,
            source_data_hash,
            target_data_hash,
            conversion_timestamp: env.ledger().timestamp(),
            requester,
            status: String::from_str(&env, "completed"),
            error_details: String::from_str(&env, ""),
        };
            .storage()
            .persistent()
            .get(&CONVERSION_REQUESTS)
            .unwrap_or(Map::new(&env));
        env.storage()
            .persistent()
            .set(&CONVERSION_REQUESTS, &requests);
        Ok(request_id)
    }

            .storage()
            .persistent()
            .get(&CONVERSION_REQUESTS)
            .ok_or(Error::InvalidConversionRequest)?;
        requests
            .get(request_id)
            .ok_or(Error::InvalidConversionRequest)
    }
}
