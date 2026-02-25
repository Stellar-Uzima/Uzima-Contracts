//! # Quality Assurance Module
//!
//! This module provides comprehensive data quality assessment, validation,
//! and cleansing capabilities for medical records.
//!
//! ## Features
//! - Data quality scoring (0-100 scale)
//! - Completeness checks for required fields
//! - Data format validation and normalization
//! - FHIR standard compliance validation
//! - Automated data cleansing and correction
//! - Validation error reporting with actionable feedback

use soroban_sdk::{contracttype, Env, Map, String, Vec};

use crate::errors::Error;
use crate::validation;
use crate::MedicalRecord;

// ==================== QUALITY SCORING CONSTANTS ====================

/// Maximum quality score (100%)
pub const MAX_QUALITY_SCORE: u32 = 100;

/// Weight for completeness in quality score (40%)
pub const COMPLETENESS_WEIGHT: u32 = 40;

/// Weight for format validity in quality score (30%)
pub const FORMAT_WEIGHT: u32 = 30;

/// Weight for FHIR compliance in quality score (20%)
pub const FHIR_WEIGHT: u32 = 20;

/// Weight for data consistency in quality score (10%)
pub const CONSISTENCY_WEIGHT: u32 = 10;

/// Minimum acceptable quality score for record acceptance
pub const MIN_ACCEPTABLE_QUALITY: u32 = 70;

// ==================== DATA STRUCTURES ====================

#[derive(Clone)]
#[contracttype]
pub struct QualityScore {
    pub overall_score: u32,
    pub completeness_score: u32,
    pub format_score: u32,
    pub fhir_compliance_score: u32,
    pub consistency_score: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct ValidationIssue {
    pub field_name: String,
    pub issue_type: IssueType,
    pub severity: IssueSeverity,
    pub message: String,
    pub suggested_fix: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum IssueType {
    Missing,
    Invalid,
    Incomplete,
    NonCompliant,
    Inconsistent,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Clone)]
#[contracttype]
pub struct ValidationReport {
    pub is_valid: bool,
    pub quality_score: QualityScore,
    pub issues: Vec<ValidationIssue>,
    pub warnings: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Clone)]
#[contracttype]
pub struct CleansingResult {
    pub was_modified: bool,
    pub changes_applied: Vec<String>,
    pub diagnosis: String,
    pub treatment: String,
    pub category: String,
    pub treatment_type: String,
    pub data_ref: String,
    pub tags: Vec<String>,
}

// ==================== QUALITY ASSESSMENT ====================

/// Performs comprehensive quality assessment of a medical record
///
/// # Arguments
/// * `env` - The environment
/// * `record` - The medical record to assess
///
/// # Returns
/// A detailed quality score breakdown
pub fn assess_quality(env: &Env, record: &MedicalRecord) -> QualityScore {
    let completeness = calculate_completeness_score(env, record);
    let format = calculate_format_score(env, record);
    let fhir = calculate_fhir_compliance_score(env, record);
    let consistency = calculate_consistency_score(env, record);

    // Calculate weighted overall score
    let overall = (completeness * COMPLETENESS_WEIGHT
        + format * FORMAT_WEIGHT
        + fhir * FHIR_WEIGHT
        + consistency * CONSISTENCY_WEIGHT)
        / 100;

    QualityScore {
        overall_score: overall,
        completeness_score: completeness,
        format_score: format,
        fhir_compliance_score: fhir,
        consistency_score: consistency,
    }
}

/// Calculates completeness score based on required and optional fields
fn calculate_completeness_score(env: &Env, record: &MedicalRecord) -> u32 {
    let _ = env;
    let mut score = 0u32;

    // Required fields (critical) - 20 points each
    if !record.diagnosis.is_empty() {
        score += 20;
    }
    if !record.treatment.is_empty() {
        score += 20;
    }
    if !record.category.is_empty() {
        score += 20;
    }
    if !record.treatment_type.is_empty() {
        score += 20;
    }
    if !record.data_ref.is_empty() {
        score += 20;
    }

    // Optional but valuable fields - 10 points each
    if !record.tags.is_empty() {
        score = score.saturating_add(10).min(100);
    }
    if record.doctor_did.is_some() {
        score = score.saturating_add(10).min(100);
    }

    score.min(100)
}

/// Calculates format validity score
fn calculate_format_score(env: &Env, record: &MedicalRecord) -> u32 {
    let mut score = 100u32;

    // Check diagnosis format
    if validation::validate_diagnosis(&record.diagnosis).is_err() {
        score = score.saturating_sub(20);
    }

    // Check treatment format
    if validation::validate_treatment(&record.treatment).is_err() {
        score = score.saturating_sub(20);
    }

    // Check category format
    if validation::validate_category(&record.category, env).is_err() {
        score = score.saturating_sub(15);
    }

    // Check treatment type format
    if validation::validate_treatment_type(&record.treatment_type).is_err() {
        score = score.saturating_sub(15);
    }

    // Check data reference format
    if validation::validate_data_ref(env, &record.data_ref).is_err() {
        score = score.saturating_sub(15);
    }

    // Check tags format
    if validation::validate_tags(&record.tags).is_err() {
        score = score.saturating_sub(15);
    }

    score
}

/// Calculates FHIR compliance score
fn calculate_fhir_compliance_score(env: &Env, record: &MedicalRecord) -> u32 {
    let mut score = 100u32;

    // Check if diagnosis follows FHIR Condition resource patterns
    if !is_fhir_compliant_diagnosis(env, &record.diagnosis) {
        score = score.saturating_sub(25);
    }

    // Check if treatment follows FHIR Procedure/MedicationStatement patterns
    if !is_fhir_compliant_treatment(env, &record.treatment) {
        score = score.saturating_sub(25);
    }

    // Check if category maps to FHIR CodeableConcept
    if !is_fhir_compliant_category(env, &record.category) {
        score = score.saturating_sub(25);
    }

    // Check if data reference follows FHIR DocumentReference patterns
    if !is_fhir_compliant_reference(env, &record.data_ref) {
        score = score.saturating_sub(25);
    }

    score
}

/// Calculates data consistency score
fn calculate_consistency_score(env: &Env, record: &MedicalRecord) -> u32 {
    let _ = env;
    let mut score = 100u32;

    // Check patient and doctor are different
    if record.patient_id == record.doctor_id {
        score = score.saturating_sub(50);
    }

    // Check timestamp is reasonable
    if record.timestamp == 0 {
        score = score.saturating_sub(30);
    }

    // Check category and treatment type consistency
    if !is_consistent_category_treatment(&record.category, &record.treatment_type) {
        score = score.saturating_sub(20);
    }

    score
}

// ==================== FHIR COMPLIANCE CHECKS ====================

/// Validates diagnosis against FHIR Condition resource patterns
fn is_fhir_compliant_diagnosis(env: &Env, diagnosis: &String) -> bool {
    let _ = env;
    // FHIR Condition should have structured information
    // Check for minimum length and basic structure
    if diagnosis.len() < validation::MIN_DIAGNOSIS_LENGTH {
        return false;
    }

    // Check for common FHIR diagnosis patterns (simplified)
    // In production, this would validate against SNOMED CT, ICD-10, etc.
    true
}

/// Validates treatment against FHIR Procedure/MedicationStatement patterns
fn is_fhir_compliant_treatment(env: &Env, treatment: &String) -> bool {
    let _ = env;
    if treatment.len() < validation::MIN_TREATMENT_LENGTH {
        return false;
    }

    // FHIR treatments should have structured information
    // In production, validate against RxNorm, SNOMED CT, etc.
    true
}

/// Validates category against FHIR CodeableConcept patterns
fn is_fhir_compliant_category(env: &Env, category: &String) -> bool {
    // Categories should be from a controlled vocabulary
    validation::validate_category(category, env).is_ok()
}

/// Validates data reference against FHIR DocumentReference patterns
fn is_fhir_compliant_reference(env: &Env, data_ref: &String) -> bool {
    // FHIR DocumentReference should be a valid URI/URL
    validation::validate_data_ref(env, data_ref).is_ok()
}

/// Checks consistency between category and treatment type
fn is_consistent_category_treatment(category: &String, treatment_type: &String) -> bool {
    // Basic consistency check - both should be non-empty
    !category.is_empty() && !treatment_type.is_empty()
}

// ==================== COMPREHENSIVE VALIDATION ====================

/// Performs comprehensive validation and generates detailed report
///
/// # Arguments
/// * `env` - The environment
/// * `record` - The medical record to validate
///
/// # Returns
/// A detailed validation report with issues and recommendations
pub fn validate_comprehensive(env: &Env, record: &MedicalRecord) -> ValidationReport {
    let mut issues = Vec::new(env);
    let mut warnings = Vec::new(env);
    let mut recommendations = Vec::new(env);

    // Validate required fields
    validate_required_fields(env, record, &mut issues);

    // Validate field formats
    validate_field_formats(env, record, &mut issues);

    // Validate FHIR compliance
    validate_fhir_compliance(env, record, &mut issues, &mut warnings);

    // Validate data consistency
    validate_data_consistency(env, record, &mut issues);

    // Generate recommendations
    generate_recommendations(env, record, &mut recommendations);

    // Calculate quality score
    let quality_score = assess_quality(env, record);

    // Determine if record is valid (no critical issues)
    let is_valid =
        !has_critical_issues(&issues) && quality_score.overall_score >= MIN_ACCEPTABLE_QUALITY;

    ValidationReport {
        is_valid,
        quality_score,
        issues,
        warnings,
        recommendations,
    }
}

/// Validates all required fields are present and non-empty
fn validate_required_fields(env: &Env, record: &MedicalRecord, issues: &mut Vec<ValidationIssue>) {
    if record.diagnosis.is_empty() {
        issues.push_back(ValidationIssue {
            field_name: String::from_str(env, "diagnosis"),
            issue_type: IssueType::Missing,
            severity: IssueSeverity::Critical,
            message: String::from_str(env, "Diagnosis is required"),
            suggested_fix: Some(String::from_str(env, "Provide a detailed diagnosis")),
        });
    }

    if record.treatment.is_empty() {
        issues.push_back(ValidationIssue {
            field_name: String::from_str(env, "treatment"),
            issue_type: IssueType::Missing,
            severity: IssueSeverity::Critical,
            message: String::from_str(env, "Treatment is required"),
            suggested_fix: Some(String::from_str(env, "Provide treatment details")),
        });
    }

    if record.category.is_empty() {
        issues.push_back(ValidationIssue {
            field_name: String::from_str(env, "category"),
            issue_type: IssueType::Missing,
            severity: IssueSeverity::High,
            message: String::from_str(env, "Category is required"),
            suggested_fix: Some(String::from_str(
                env,
                "Select: Modern, Traditional, Herbal, or Spiritual",
            )),
        });
    }
}

/// Validates field formats and lengths
fn validate_field_formats(env: &Env, record: &MedicalRecord, issues: &mut Vec<ValidationIssue>) {
    if validation::validate_diagnosis(&record.diagnosis).is_err() {
        issues.push_back(ValidationIssue {
            field_name: String::from_str(env, "diagnosis"),
            issue_type: IssueType::Invalid,
            severity: IssueSeverity::High,
            message: String::from_str(env, "Diagnosis format is invalid"),
            suggested_fix: Some(String::from_str(
                env,
                "Ensure diagnosis is 1-512 characters",
            )),
        });
    }

    if validation::validate_category(&record.category, env).is_err() {
        issues.push_back(ValidationIssue {
            field_name: String::from_str(env, "category"),
            issue_type: IssueType::Invalid,
            severity: IssueSeverity::High,
            message: String::from_str(env, "Invalid category"),
            suggested_fix: Some(String::from_str(
                env,
                "Use: Modern, Traditional, Herbal, or Spiritual",
            )),
        });
    }

    if validation::validate_tags(&record.tags).is_err() {
        issues.push_back(ValidationIssue {
            field_name: String::from_str(env, "tags"),
            issue_type: IssueType::Invalid,
            severity: IssueSeverity::Medium,
            message: String::from_str(env, "Tags format is invalid"),
            suggested_fix: Some(String::from_str(env, "Check tag length and count limits")),
        });
    }
}

/// Validates FHIR standard compliance
fn validate_fhir_compliance(
    env: &Env,
    record: &MedicalRecord,
    issues: &mut Vec<ValidationIssue>,
    warnings: &mut Vec<String>,
) {
    if !is_fhir_compliant_diagnosis(env, &record.diagnosis) {
        warnings.push_back(String::from_str(env, "Diagnosis may not be FHIR compliant"));
    }

    if !is_fhir_compliant_treatment(env, &record.treatment) {
        warnings.push_back(String::from_str(env, "Treatment may not be FHIR compliant"));
    }

    if !is_fhir_compliant_reference(env, &record.data_ref) {
        issues.push_back(ValidationIssue {
            field_name: String::from_str(env, "data_ref"),
            issue_type: IssueType::NonCompliant,
            severity: IssueSeverity::Medium,
            message: String::from_str(env, "Data reference not FHIR compliant"),
            suggested_fix: Some(String::from_str(
                env,
                "Use valid URI format (e.g., IPFS CID)",
            )),
        });
    }
}

/// Validates data consistency across fields
fn validate_data_consistency(env: &Env, record: &MedicalRecord, issues: &mut Vec<ValidationIssue>) {
    if record.patient_id == record.doctor_id {
        issues.push_back(ValidationIssue {
            field_name: String::from_str(env, "patient_id"),
            issue_type: IssueType::Inconsistent,
            severity: IssueSeverity::Critical,
            message: String::from_str(env, "Patient and doctor cannot be the same"),
            suggested_fix: Some(String::from_str(env, "Use different addresses")),
        });
    }

    if record.timestamp == 0 {
        issues.push_back(ValidationIssue {
            field_name: String::from_str(env, "timestamp"),
            issue_type: IssueType::Invalid,
            severity: IssueSeverity::High,
            message: String::from_str(env, "Invalid timestamp"),
            suggested_fix: Some(String::from_str(env, "Use current ledger timestamp")),
        });
    }
}

/// Generates actionable recommendations for improving data quality
fn generate_recommendations(env: &Env, record: &MedicalRecord, recommendations: &mut Vec<String>) {
    if record.tags.is_empty() {
        recommendations.push_back(String::from_str(env, "Add tags to improve searchability"));
    }

    if record.doctor_did.is_none() {
        recommendations.push_back(String::from_str(
            env,
            "Link doctor DID for enhanced verification",
        ));
    }

    if record.diagnosis.len() < 20 {
        recommendations.push_back(String::from_str(
            env,
            "Provide more detailed diagnosis information",
        ));
    }

    if record.treatment.len() < 20 {
        recommendations.push_back(String::from_str(
            env,
            "Provide more detailed treatment information",
        ));
    }
}

/// Checks if there are any critical issues in the validation report
fn has_critical_issues(issues: &Vec<ValidationIssue>) -> bool {
    for issue in issues.iter() {
        if issue.severity == IssueSeverity::Critical {
            return true;
        }
    }
    false
}

// ==================== DATA CLEANSING ====================

/// Performs automated data cleansing and normalization
///
/// # Arguments
/// * `env` - The environment
/// * `diagnosis` - Diagnosis text to cleanse
/// * `treatment` - Treatment text to cleanse
/// * `category` - Category to normalize
/// * `treatment_type` - Treatment type to cleanse
/// * `data_ref` - Data reference to normalize
/// * `tags` - Tags to cleanse
///
/// # Returns
/// Cleansed data with list of changes applied
pub fn cleanse_data(
    env: &Env,
    diagnosis: String,
    treatment: String,
    category: String,
    treatment_type: String,
    data_ref: String,
    tags: Vec<String>,
) -> CleansingResult {
    let mut changes = Vec::new(env);
    let mut was_modified = false;

    // Cleanse diagnosis
    let (clean_diagnosis, diag_modified) = cleanse_text(env, diagnosis, "diagnosis");
    if diag_modified {
        was_modified = true;
        changes.push_back(String::from_str(env, "Normalized diagnosis text"));
    }

    // Cleanse treatment
    let (clean_treatment, treat_modified) = cleanse_text(env, treatment, "treatment");
    if treat_modified {
        was_modified = true;
        changes.push_back(String::from_str(env, "Normalized treatment text"));
    }

    // Normalize category
    let (clean_category, cat_modified) = normalize_category(env, category);
    if cat_modified {
        was_modified = true;
        changes.push_back(String::from_str(env, "Normalized category"));
    }

    // Cleanse treatment type
    let (clean_treatment_type, tt_modified) = cleanse_text(env, treatment_type, "treatment_type");
    if tt_modified {
        was_modified = true;
        changes.push_back(String::from_str(env, "Normalized treatment type"));
    }

    // Normalize data reference
    let (clean_data_ref, ref_modified) = normalize_data_ref(env, data_ref);
    if ref_modified {
        was_modified = true;
        changes.push_back(String::from_str(env, "Normalized data reference"));
    }

    // Cleanse tags
    let (clean_tags, tags_modified) = cleanse_tags(env, tags);
    if tags_modified {
        was_modified = true;
        changes.push_back(String::from_str(env, "Cleaned and deduplicated tags"));
    }

    CleansingResult {
        was_modified,
        changes_applied: changes,
        diagnosis: clean_diagnosis,
        treatment: clean_treatment,
        category: clean_category,
        treatment_type: clean_treatment_type,
        data_ref: clean_data_ref,
        tags: clean_tags,
    }
}

/// Cleanses and normalizes text fields
fn cleanse_text(env: &Env, text: String, _field_name: &str) -> (String, bool) {
    // In a real implementation, this would:
    // - Trim whitespace
    // - Remove control characters
    // - Normalize unicode
    // - Fix common typos
    // For now, we return as-is
    let _ = env;
    (text, false)
}

/// Normalizes category to standard values
fn normalize_category(env: &Env, category: String) -> (String, bool) {
    // Check for standard categories first
    let modern = String::from_str(env, "Modern");
    let traditional = String::from_str(env, "Traditional");
    let herbal = String::from_str(env, "Herbal");
    let spiritual = String::from_str(env, "Spiritual");

    // Direct match
    if category == modern || category == traditional || category == herbal || category == spiritual
    {
        return (category, false);
    }

    // For now, return as-is since we can't do case-insensitive comparison easily in no_std
    // In production, this would use proper string matching
    (category, false)
}

/// Normalizes data reference format
fn normalize_data_ref(env: &Env, data_ref: String) -> (String, bool) {
    // In production, this would:
    // - Validate IPFS CID format
    // - Normalize URL schemes
    // - Remove trailing slashes
    let _ = env;
    (data_ref, false)
}

/// Cleanses tags by removing duplicates and invalid entries
fn cleanse_tags(env: &Env, tags: Vec<String>) -> (Vec<String>, bool) {
    let mut clean_tags = Vec::new(env);
    let mut seen = Map::new(env);
    let mut modified = false;

    for tag in tags.iter() {
        // Skip empty tags
        if tag.is_empty() {
            modified = true;
            continue;
        }

        // Skip duplicates
        if seen.contains_key(tag.clone()) {
            modified = true;
            continue;
        }

        // Skip tags that are too long
        if tag.len() > validation::MAX_TAG_LENGTH {
            modified = true;
            continue;
        }

        seen.set(tag.clone(), true);
        clean_tags.push_back(tag);
    }

    (clean_tags, modified)
}

// ==================== COMPLETENESS CHECKS ====================

/// Checks data completeness and identifies gaps
///
/// # Arguments
/// * `env` - The environment
/// * `record` - The medical record to check
///
/// # Returns
/// List of missing or incomplete fields
pub fn check_completeness(env: &Env, record: &MedicalRecord) -> Vec<String> {
    let mut gaps = Vec::new(env);

    if record.diagnosis.is_empty() {
        gaps.push_back(String::from_str(env, "diagnosis"));
    }

    if record.treatment.is_empty() {
        gaps.push_back(String::from_str(env, "treatment"));
    }

    if record.category.is_empty() {
        gaps.push_back(String::from_str(env, "category"));
    }

    if record.treatment_type.is_empty() {
        gaps.push_back(String::from_str(env, "treatment_type"));
    }

    if record.data_ref.is_empty() {
        gaps.push_back(String::from_str(env, "data_ref"));
    }

    if record.tags.is_empty() {
        gaps.push_back(String::from_str(env, "tags"));
    }

    if record.doctor_did.is_none() {
        gaps.push_back(String::from_str(env, "doctor_did"));
    }

    gaps
}

/// Validates that a record meets minimum quality standards
///
/// # Arguments
/// * `env` - The environment
/// * `record` - The medical record to validate
///
/// # Returns
/// Ok if quality is acceptable, Error otherwise
pub fn validate_minimum_quality(env: &Env, record: &MedicalRecord) -> Result<(), Error> {
    let quality = assess_quality(env, record);

    if quality.overall_score < MIN_ACCEPTABLE_QUALITY {
        return Err(Error::InvalidInput);
    }

    // Check for critical completeness issues
    if record.diagnosis.is_empty() || record.treatment.is_empty() {
        return Err(Error::InvalidInput);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    fn create_test_record(env: &Env) -> MedicalRecord {
        MedicalRecord {
            patient_id: Address::generate(env),
            doctor_id: Address::generate(env),
            timestamp: 1000,
            diagnosis: String::from_str(env, "Patient presents with acute bronchitis"),
            treatment: String::from_str(env, "Prescribed antibiotics and rest"),
            is_confidential: false,
            tags: soroban_sdk::vec![env, String::from_str(env, "respiratory")],
            category: String::from_str(env, "Modern"),
            treatment_type: String::from_str(env, "Medication"),
            data_ref: String::from_str(env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx"),
            doctor_did: Some(String::from_str(env, "did:example:123")),
        }
    }

    #[test]
    fn test_assess_quality_high_quality_record() {
        let env = Env::default();
        let record = create_test_record(&env);

        let score = assess_quality(&env, &record);

        assert!(score.overall_score >= MIN_ACCEPTABLE_QUALITY);
        assert!(score.completeness_score >= 80);
        assert!(score.format_score >= 80);
    }

    #[test]
    fn test_assess_quality_incomplete_record() {
        let env = Env::default();
        let mut record = create_test_record(&env);
        record.diagnosis = String::from_str(&env, "");
        record.treatment = String::from_str(&env, "");
        record.tags = soroban_sdk::vec![&env];
        record.doctor_did = None;

        let score = assess_quality(&env, &record);

        // With diagnosis and treatment empty (40 points lost), no tags, no DID
        // we have category (20) + treatment_type (20) + data_ref (20) = 60
        assert_eq!(score.completeness_score, 60);
        // Overall score should be lower due to format issues
        assert!(score.overall_score < MIN_ACCEPTABLE_QUALITY);
    }

    #[test]
    fn test_validate_comprehensive() {
        let env = Env::default();
        let record = create_test_record(&env);

        let report = validate_comprehensive(&env, &record);

        assert!(report.is_valid);
        assert!(report.quality_score.overall_score >= MIN_ACCEPTABLE_QUALITY);
    }

    #[test]
    fn test_validate_comprehensive_with_issues() {
        let env = Env::default();
        let mut record = create_test_record(&env);
        record.diagnosis = String::from_str(&env, "");

        let report = validate_comprehensive(&env, &record);

        assert!(!report.is_valid);
        assert!(!report.issues.is_empty());
    }

    #[test]
    fn test_cleanse_tags_removes_duplicates() {
        let env = Env::default();
        let tags = soroban_sdk::vec![
            &env,
            String::from_str(&env, "tag1"),
            String::from_str(&env, "tag2"),
            String::from_str(&env, "tag1"),
        ];

        let (clean_tags, modified) = cleanse_tags(&env, tags);

        assert!(modified);
        assert_eq!(clean_tags.len(), 2);
    }

    #[test]
    fn test_check_completeness() {
        let env = Env::default();
        let mut record = create_test_record(&env);
        record.diagnosis = String::from_str(&env, "");
        record.treatment = String::from_str(&env, "");

        let gaps = check_completeness(&env, &record);

        assert!(gaps.len() >= 2);
    }

    #[test]
    fn test_validate_minimum_quality_pass() {
        let env = Env::default();
        let record = create_test_record(&env);

        let result = validate_minimum_quality(&env, &record);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_minimum_quality_fail() {
        let env = Env::default();
        let mut record = create_test_record(&env);
        record.diagnosis = String::from_str(&env, "");

        let result = validate_minimum_quality(&env, &record);

        assert!(result.is_err());
    }

    #[test]
    fn test_normalize_category() {
        let env = Env::default();

        // Test that standard categories are not modified
        let (normalized, modified) = normalize_category(&env, String::from_str(&env, "Modern"));
        assert!(!modified);
        assert_eq!(normalized, String::from_str(&env, "Modern"));

        let (normalized, modified) =
            normalize_category(&env, String::from_str(&env, "Traditional"));
        assert!(!modified);
        assert_eq!(normalized, String::from_str(&env, "Traditional"));
    }
}
