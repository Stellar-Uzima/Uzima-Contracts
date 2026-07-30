#![no_std]
use soroban_sdk::{contracterror, contracttype, Address, Env, Symbol, Vec, Map};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u32)]
pub enum PolicyError {
    UnauthorizedRegion = 1,
    DataExportRestricted = 2,
    PolicyExpired = 3,
    InvalidJurisdiction = 4,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegionalRule {
    pub region_code: Symbol,          // e.g., Symbol::new(&env, "EU"), Symbol::new(&env, "KE")
    pub export_allowed: bool,         // Permissibility of cross-border data transfer
    pub retention_period_sec: u64,    // Mandated data retention window
    pub requires_patient_consent: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferRequest {
    pub source_region: Symbol,
    pub target_region: Symbol,
    pub data_type: Symbol,            // e.g., Symbol::new(&env, "EHR")
    pub patient_consent_given: bool,
}

pub struct PolicyEngine;

impl PolicyEngine {
    /// Evaluates cross-border transfer requests against regional compliance rules
    pub fn evaluate_transfer(
        env: &Env,
        rules: &Map<Symbol, RegionalRule>,
        request: &TransferRequest,
    ) -> Result<bool, PolicyError> {
        // Fetch source region rule
        let source_rule = rules
            .get(request.source_region.clone())
            .ok_or(PolicyError::InvalidJurisdiction)?;

        // Verify intra-region transfers (always valid if consent requirements are met)
        if request.source_region == request.target_region {
            if source_rule.requires_patient_consent && !request.patient_consent_given {
                return Err(PolicyError::UnauthorizedRegion);
            }
            return Ok(true);
        }

        // Cross-border evaluation
        if !source_rule.export_allowed {
            return Err(PolicyError::DataExportRestricted);
        }

        // Verify destination region existence
        if !rules.contains_key(request.target_region.clone()) {
            return Err(PolicyError::InvalidJurisdiction);
        }

        if source_rule.requires_patient_consent && !request.patient_consent_given {
            return Err(PolicyError::UnauthorizedRegion);
        }

        Ok(true)
    }
}