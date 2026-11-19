#![no_std]
#![forbid(alloc)]
//! patient_risk_stratification - Healthcare smart contract on Stellar blockchain.
#![allow(clippy::too_many_arguments)]

use common_error::{get_suggestion as common_suggestion, CommonError};
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Map,
    String, Symbol, Vec,
};

#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub enum RiskModelType {
    Readmission,
    Mortality,
    Complications,
}

#[derive(Clone)]
#[contracttype]
pub struct RiskModel {
    pub model_id: BytesN<32>,
    pub model_type: RiskModelType,
    pub specialty: String, // e.g., "cardiology", "oncology"
    pub version: String,
    pub min_confidence_bps: u32,
    pub enabled: bool,
    pub description: String,
}

#[derive(Clone)]
#[contracttype]
pub struct RiskAssessment {
    pub assessment_id: u64,
    pub patient: Address,
    pub model_id: BytesN<32>,
    pub risk_score_bps: u32, // 0-10000 basis points
    pub confidence_bps: u32,
    pub assessment_date: u64,
    pub prediction_horizon_days: u32,
    pub risk_factors: Vec<RiskFactor>,
    pub interventions: Vec<InterventionRecommendation>,
    pub specialty: String,
    pub auc_score_bps: u32, // For tracking accuracy
}

#[derive(Clone)]
#[contracttype]
pub struct RiskFactor {
    pub factor_name: String,
    pub contribution_bps: i32, // Can be negative for protective factors
    pub importance_bps: u32,
    pub category: String, // e.g., "demographic", "clinical", "behavioral"
    pub explanation: String,
}

#[derive(Clone)]
#[contracttype]
pub struct InterventionRecommendation {
    pub intervention_type: String,
    pub priority: u32, // 1-5, 5 being highest
    pub description: String,
    pub expected_impact_bps: u32,
    pub timeframe_days: u32,
    pub resources_needed: Vec<String>,
}

#[derive(Clone)]
#[contracttype]
pub struct PatientRiskProfile {
    pub patient: Address,
    pub latest_assessment_id: u64,
    pub current_risk_level: String, // "low", "medium", "high", "critical"
    pub risk_trend: String,         // "improving", "stable", "worsening"
    pub last_updated: u64,
    pub total_assessments: u32,
    pub specialty_profiles: Map<String, SpecialtyRiskSummary>,
}

#[derive(Clone)]
#[contracttype]
pub struct SpecialtyRiskSummary {
    pub specialty: String,
    pub avg_risk_score_bps: u32,
    pub high_risk_count: u32,
    pub last_assessment_date: u64,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Config,
    RiskModel(BytesN<32>),
    Assessment(u64),
    PatientProfile(Address),
    AssessmentCounter,
    ModelRegistry(RiskModelType),
}

const ASSESSMENT_COUNTER: Symbol = symbol_short!("ASS_CT");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    ConfigNotSet = 2,
    ModelNotFound = 3,
    InvalidScore = 4,
    LowConfidence = 5,
    AssessmentNotFound = 6,
    InvalidModel = 7,
    DuplicateModel = 8,
    AlreadyInitialized = 9,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::NotAuthorized => write!(f, "not authorized"),
            Error::ConfigNotSet => write!(f, "config not set"),
            Error::ModelNotFound => write!(f, "model not found"),
            Error::InvalidScore => write!(f, "invalid score"),
            Error::LowConfidence => write!(f, "low confidence"),
            Error::AssessmentNotFound => write!(f, "assessment not found"),
            Error::InvalidModel => write!(f, "invalid model"),
            Error::DuplicateModel => write!(f, "duplicate model"),
            Error::AlreadyInitialized => write!(f, "already initialized"),
        }
    }
}

#[contract]
pub struct PatientRiskStratificationContract;

#[contractimpl]
impl PatientRiskStratificationContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Config) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Config, &admin);
        env.storage().instance().set(&ASSESSMENT_COUNTER, &0u64);
        Ok(())
    }

    #[must_use]
    fn load_admin(env: &Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::ConfigNotSet)
    }

    #[must_use]
    fn ensure_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin = Self::load_admin(env)?;
        if admin != *caller {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    fn next_assessment_id(env: &Env) -> u64 {
        let current: u64 = env
            .storage()
            .instance()
            .get(&ASSESSMENT_COUNTER)
            .unwrap_or(0);
        let next = current + 1;
        env.storage().instance().set(&ASSESSMENT_COUNTER, &next);
        next
    }

    pub fn register_risk_model(
        env: Env,
        caller: Address,
        model_id: BytesN<32>,
        model_type: RiskModelType,
        specialty: String,
        version: String,
        min_confidence_bps: u32,
        description: String,
    ) -> Result<(), Error> {
        Self::ensure_admin(&env, &caller)?;

        if env
            .storage()
            .instance()
            .has(&DataKey::RiskModel(model_id.clone()))
        {
            return Err(Error::DuplicateModel);
        }

        if min_confidence_bps > 10_000 {
            return Err(Error::InvalidScore);
        }

        let model = RiskModel {
            model_id: model_id.clone(),
            model_type,
            specialty,
            version,
            min_confidence_bps,
            enabled: true,
            description,
        };

        env.storage()
            .instance()
            .set(&DataKey::RiskModel(model_id.clone()), &model);

        env.events().publish((symbol_short!("ModelReg"),), model_id);

        Ok(())
    }

    pub fn perform_risk_assessment(
        env: Env,
        caller: Address,
        patient: Address,
        model_id: BytesN<32>,
        risk_score_bps: u32,
        confidence_bps: u32,
        prediction_horizon_days: u32,
        risk_factors: Vec<RiskFactor>,
        interventions: Vec<InterventionRecommendation>,
        auc_score_bps: u32,
    ) -> Result<u64, Error> {
        caller.require_auth();

        let model: RiskModel = env
            .storage()
            .instance()
            .get(&DataKey::RiskModel(model_id.clone()))
            .ok_or(Error::ModelNotFound)?;

        if !model.enabled {
            return Err(Error::InvalidModel);
        }

        if risk_score_bps > 10_000 || confidence_bps > 10_000 || auc_score_bps > 10_000 {
            return Err(Error::InvalidScore);
        }

        if confidence_bps < model.min_confidence_bps {
            return Err(Error::LowConfidence);
        }

        let assessment_id = Self::next_assessment_id(&env);
        let timestamp = env.ledger().timestamp();

        let assessment = RiskAssessment {
            assessment_id,
            patient: patient.clone(),
            model_id: model_id.clone(),
            risk_score_bps,
            confidence_bps,
            assessment_date: timestamp,
            prediction_horizon_days,
            risk_factors,
            interventions,
            specialty: model.specialty.clone(),
            auc_score_bps,
        };

        env.storage()
            .instance()
            .set(&DataKey::Assessment(assessment_id), &assessment);

        // Update patient risk profile
        Self::update_patient_profile(
            &env,
            &patient,
            assessment_id,
            risk_score_bps,
            &model.specialty,
            timestamp,
        );

        env.events().publish(
            (symbol_short!("RiskAsses"),),
            (assessment_id, patient, risk_score_bps, confidence_bps),
        );

        Ok(assessment_id)
    }

    fn update_patient_profile(
        env: &Env,
        patient: &Address,
        assessment_id: u64,
        risk_score_bps: u32,
        specialty: &String,
        timestamp: u64,
    ) {
        let mut profile: PatientRiskProfile = env
            .storage()
            .instance()
            .get(&DataKey::PatientProfile(patient.clone()))
            .unwrap_or_else(|| PatientRiskProfile {
                patient: patient.clone(),
                latest_assessment_id: 0,
                current_risk_level: String::from_str(env, "unknown"),
                risk_trend: String::from_str(env, "unknown"),
                last_updated: 0,
                total_assessments: 0,
                specialty_profiles: Map::new(env),
            });

        profile.latest_assessment_id = assessment_id;
        profile.last_updated = timestamp;
        profile.total_assessments += 1;

        // Determine risk level
        profile.current_risk_level = if risk_score_bps >= 7500 {
            String::from_str(env, "critical")
        } else if risk_score_bps >= 5000 {
            String::from_str(env, "high")
        } else if risk_score_bps >= 2500 {
            String::from_str(env, "medium")
        } else {
            String::from_str(env, "low")
        };

        // Update specialty summary
        let specialty_key = specialty.clone();
        let mut specialty_summary: SpecialtyRiskSummary = profile
            .specialty_profiles
            .get(specialty_key.clone())
            .unwrap_or(SpecialtyRiskSummary {
                specialty: specialty.clone(),
                avg_risk_score_bps: 0,
                high_risk_count: 0,
                last_assessment_date: 0,
            });

        // Update average risk score
        let total = specialty_summary.avg_risk_score_bps as u64
            * (profile.total_assessments as u64 - 1)
            + risk_score_bps as u64;
        specialty_summary.avg_risk_score_bps = (total / profile.total_assessments as u64) as u32;

        if risk_score_bps >= 5000 {
            specialty_summary.high_risk_count += 1;
        }
        specialty_summary.last_assessment_date = timestamp;

        profile
            .specialty_profiles
            .set(specialty_key, specialty_summary);

        env.storage()
            .instance()
            .set(&DataKey::PatientProfile(patient.clone()), &profile);
    }

    pub fn get_risk_assessment(env: Env, assessment_id: u64) -> Option<RiskAssessment> {
        env.storage()
            .instance()
            .get(&DataKey::Assessment(assessment_id))
    }

    pub fn get_patient_risk_profile(env: Env, patient: Address) -> Option<PatientRiskProfile> {
        env.storage()
            .instance()
            .get(&DataKey::PatientProfile(patient))
    }

    pub fn get_risk_model(env: Env, model_id: BytesN<32>) -> Option<RiskModel> {
        env.storage().instance().get(&DataKey::RiskModel(model_id))
    }

    pub fn get_patient_risk_factors(
        env: Env,
        patient: Address,
        specialty: String,
    ) -> Vec<RiskFactor> {
        let profile: Option<PatientRiskProfile> = env
            .storage()
            .instance()
            .get(&DataKey::PatientProfile(patient));
        let profile = match profile {
            Some(p) => p,
            None => return Vec::new(&env),
        };

        let assessment: Option<RiskAssessment> = env
            .storage()
            .instance()
            .get(&DataKey::Assessment(profile.latest_assessment_id));
        let assessment = match assessment {
            Some(a) => a,
            None => return Vec::new(&env),
        };

        if assessment.specialty != specialty {
            return Vec::new(&env);
        }

        assessment.risk_factors
    }

    pub fn get_intervention_recommendations(
        env: Env,
        patient: Address,
    ) -> Vec<InterventionRecommendation> {
        let profile: Option<PatientRiskProfile> = env
            .storage()
            .instance()
            .get(&DataKey::PatientProfile(patient));
        let profile = match profile {
            Some(p) => p,
            None => return Vec::new(&env),
        };

        let assessment: Option<RiskAssessment> = env
            .storage()
            .instance()
            .get(&DataKey::Assessment(profile.latest_assessment_id));
        let assessment = match assessment {
            Some(a) => a,
            None => return Vec::new(&env),
        };

        assessment.interventions
    }

    pub fn update_model_status(
        env: Env,
        caller: Address,
        model_id: BytesN<32>,
        enabled: bool,
    ) -> Result<bool, Error> {
        Self::ensure_admin(&env, &caller)?;

        let mut model: RiskModel = env
            .storage()
            .instance()
            .get(&DataKey::RiskModel(model_id.clone()))
            .ok_or(Error::ModelNotFound)?;

        model.enabled = enabled;
        env.storage()
            .instance()
            .set(&DataKey::RiskModel(model_id.clone()), &model);

        env.events()
            .publish((symbol_short!("ModelUpd"),), (model_id, enabled));

        Ok(true)
    }
}

pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::NotAuthorized => common_suggestion(CommonError::Unauthorized),
        Error::ConfigNotSet => common_suggestion(CommonError::NotInitialized),
        Error::AlreadyInitialized => common_suggestion(CommonError::AlreadyInitialized),
        Error::ModelNotFound | Error::AssessmentNotFound => {
            common_suggestion(CommonError::NotFound)
        },
        Error::InvalidScore | Error::InvalidModel => common_suggestion(CommonError::InvalidInput),
        Error::LowConfidence => common_suggestion(CommonError::InvalidState),
        Error::DuplicateModel => common_suggestion(CommonError::InvalidState),
    }
}

#[cfg(test)]
mod test;
