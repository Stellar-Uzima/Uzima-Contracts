#![no_std]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::panic)]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
    String, Symbol, Vec,
};

#[derive(Clone)]
#[contracttype]
pub struct PlatformConfig {
    pub admin: Address,
    pub analyzer: Address,
    pub predictor: Address,
    pub genomic_contract: Option<Address>,
    pub clinical_trial_contract: Option<Address>,
    pub large_scale_mode: bool,
    pub quantum_enabled: bool,
    pub min_candidate_accuracy_bps: u32,
    pub max_analysis_time_hours: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct MolecularStructure {
    pub molecule_id: u64,
    pub canonical_smiles: String,
    pub inchi_key: String,
    pub molecular_weight_milli: u32,
    pub h_bond_donors: u32,
    pub h_bond_acceptors: u32,
    pub rotatable_bonds: u32,
    pub fingerprint: Vec<u32>,
    pub database_refs: Vec<String>,
    pub created_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct StructureAnalysis {
    pub molecule_id: u64,
    pub lipinski_violations: u32,
    pub qed_score_bps: u32,
    pub synthetic_accessibility_bps: u32,
    pub novelty_score_bps: u32,
    pub created_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct DrugTargetPrediction {
    pub prediction_id: u64,
    pub molecule_id: u64,
    pub target_gene: String,
    pub binding_affinity_pico: u64,
    pub confidence_bps: u32,
    pub model_ref: String,
    pub created_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct AdverseEffectPrediction {
    pub adverse_id: u64,
    pub molecule_id: u64,
    pub effect_code: String,
    pub severity_bps: u32,
    pub probability_bps: u32,
    pub cohort_ref: String,
    pub created_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct TrialMatchResult {
    pub match_id: u64,
    pub molecule_id: u64,
    pub protocol_id: u64,
    pub fit_score_bps: u32,
    pub expected_enrollment_days: u32,
    pub matched_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct QuantumSimulationRequest {
    pub simulation_id: u64,
    pub molecule_id: u64,
    pub target_gene: String,
    pub algorithm: String,
    pub depth: u32,
    pub shots: u32,
    pub queued_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct ScreeningCampaignReport {
    pub campaign_id: u64,
    pub target_gene: String,
    pub screened_candidates: u32,
    pub identified_candidates: u32,
    pub candidate_accuracy_bps: u32,
    pub analysis_time_hours: u32,
    pub used_quantum: bool,
    pub completed_at: u64,
}

#[contracttype]
pub enum DataKey {
    Config,
    Molecule(u64),
    MoleculeCount,
    Analysis(u64),
    Prediction(u64),
    PredictionCount,
    AdversePrediction(u64),
    AdverseCount,
    TrialMatch(u64),
    MatchCount,
    QuantumRequest(u64),
    QuantumCount,
    CampaignReport(u64),
    CampaignCount,
}

const MOLECULE_COUNT: Symbol = symbol_short!("MOL_CNT");
const PREDICTION_COUNT: Symbol = symbol_short!("PRD_CNT");
const ADVERSE_COUNT: Symbol = symbol_short!("ADV_CNT");
const MATCH_COUNT: Symbol = symbol_short!("MAT_CNT");
const QUANTUM_COUNT: Symbol = symbol_short!("QTM_CNT");
const CAMPAIGN_COUNT: Symbol = symbol_short!("CMP_CNT");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    NotInitialized = 2,
    InvalidInput = 3,
    MoleculeNotFound = 4,
    PredictionNotFound = 5,
    BenchmarkNotMet = 6,
    IntegrationMissing = 7,
    QuantumDisabled = 8,
}

#[soroban_sdk::contractclient(name = "GenomicDataClient")]
pub trait GenomicDataContract {
    fn get_record(env: Env, id: u64) -> Option<BytesN<32>>;
}

#[soroban_sdk::contractclient(name = "ClinicalTrialClient")]
pub trait ClinicalTrialContract {
    fn get_protocol(env: Env, id: u64) -> Option<TrialProtocolView>;
}

#[derive(Clone)]
#[contracttype]
pub struct TrialProtocolView {
    pub id: u64,
    pub title: String,
    pub version: u32,
    pub sponsor: Address,
    pub created_at: u64,
    pub active: bool,
    pub metadata_ref: String,
}

#[contract]
pub struct DrugDiscoveryPlatform;

#[contractimpl]
impl DrugDiscoveryPlatform {
    pub fn initialize(env: Env, admin: Address, analyzer: Address, predictor: Address) -> bool {
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Config) {
            return false;
        }

        let cfg = PlatformConfig {
            admin,
            analyzer,
            predictor,
            genomic_contract: None,
            clinical_trial_contract: None,
            large_scale_mode: true,
            quantum_enabled: false,
            min_candidate_accuracy_bps: 8_000,
            max_analysis_time_hours: 24,
        };

        env.storage().instance().set(&DataKey::Config, &cfg);
        env.storage().instance().set(&MOLECULE_COUNT, &0u64);
        env.storage().instance().set(&PREDICTION_COUNT, &0u64);
        env.storage().instance().set(&ADVERSE_COUNT, &0u64);
        env.storage().instance().set(&MATCH_COUNT, &0u64);
        env.storage().instance().set(&QUANTUM_COUNT, &0u64);
        env.storage().instance().set(&CAMPAIGN_COUNT, &0u64);
        true
    }

    fn load_config(env: &Env) -> Result<PlatformConfig, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)
    }

    fn ensure_admin(env: &Env, caller: &Address) -> Result<PlatformConfig, Error> {
        let cfg = Self::load_config(env)?;
        if cfg.admin != *caller {
            return Err(Error::NotAuthorized);
        }
        Ok(cfg)
    }

    fn ensure_analyzer(env: &Env, caller: &Address) -> Result<PlatformConfig, Error> {
        let cfg = Self::load_config(env)?;
        if cfg.analyzer != *caller {
            return Err(Error::NotAuthorized);
        }
        Ok(cfg)
    }

    fn ensure_predictor(env: &Env, caller: &Address) -> Result<PlatformConfig, Error> {
        let cfg = Self::load_config(env)?;
        if cfg.predictor != *caller {
            return Err(Error::NotAuthorized);
        }
        Ok(cfg)
    }

    fn next_id(env: &Env, counter: &Symbol) -> u64 {
        let current: u64 = env.storage().instance().get(counter).unwrap_or(0);
        let next = current + 1;
        env.storage().instance().set(counter, &next);
        next
    }

    pub fn configure_integrations(
        env: Env,
        caller: Address,
        genomic_contract: Option<Address>,
        clinical_trial_contract: Option<Address>,
        large_scale_mode: Option<bool>,
        quantum_enabled: Option<bool>,
    ) -> Result<bool, Error> {
        caller.require_auth();
        let mut cfg = Self::ensure_admin(&env, &caller)?;

        if let Some(g) = genomic_contract {
            cfg.genomic_contract = Some(g);
        }
        if let Some(c) = clinical_trial_contract {
            cfg.clinical_trial_contract = Some(c);
        }
        if let Some(v) = large_scale_mode {
            cfg.large_scale_mode = v;
        }
        if let Some(v) = quantum_enabled {
            cfg.quantum_enabled = v;
        }

        env.storage().instance().set(&DataKey::Config, &cfg);
        env.events().publish((symbol_short!("CfgInt"),), true);
        Ok(true)
    }

    pub fn register_molecule(
        env: Env,
        caller: Address,
        canonical_smiles: String,
        inchi_key: String,
        molecular_weight_milli: u32,
        h_bond_donors: u32,
        h_bond_acceptors: u32,
        rotatable_bonds: u32,
        fingerprint: Vec<u32>,
        database_refs: Vec<String>,
    ) -> Result<u64, Error> {
        caller.require_auth();
        Self::ensure_analyzer(&env, &caller)?;

        if canonical_smiles.is_empty() || inchi_key.is_empty() || fingerprint.is_empty() {
            return Err(Error::InvalidInput);
        }

        let molecule_id = Self::next_id(&env, &MOLECULE_COUNT);
        let molecule = MolecularStructure {
            molecule_id,
            canonical_smiles,
            inchi_key,
            molecular_weight_milli,
            h_bond_donors,
            h_bond_acceptors,
            rotatable_bonds,
            fingerprint,
            database_refs,
            created_at: env.ledger().timestamp(),
        };

        env.storage()
            .instance()
            .set(&DataKey::Molecule(molecule_id), &molecule);

        Ok(molecule_id)
    }

    pub fn analyze_molecular_structure(
        env: Env,
        caller: Address,
        molecule_id: u64,
    ) -> Result<StructureAnalysis, Error> {
        caller.require_auth();
        Self::ensure_analyzer(&env, &caller)?;

        let molecule: MolecularStructure = env
            .storage()
            .instance()
            .get(&DataKey::Molecule(molecule_id))
            .ok_or(Error::MoleculeNotFound)?;

        let lipinski_violations = u32::from(molecule.molecular_weight_milli > 500_000)
            + u32::from(molecule.h_bond_donors > 5)
            + u32::from(molecule.h_bond_acceptors > 10)
            + u32::from(molecule.rotatable_bonds > 10);

        let qed_score_bps = 10_000u32.saturating_sub(lipinski_violations.saturating_mul(1_250));
        let synthetic_accessibility_bps = 9_500u32
            .saturating_sub(molecule.rotatable_bonds.saturating_mul(150))
            .max(1_000);
        let novelty_score_bps = (molecule.fingerprint.len() as u32)
            .saturating_mul(120)
            .min(10_000)
            .max(2_000);

        let analysis = StructureAnalysis {
            molecule_id,
            lipinski_violations,
            qed_score_bps,
            synthetic_accessibility_bps,
            novelty_score_bps,
            created_at: env.ledger().timestamp(),
        };

        env.storage()
            .instance()
            .set(&DataKey::Analysis(molecule_id), &analysis);

        Ok(analysis)
    }

    pub fn predict_drug_target_interaction(
        env: Env,
        caller: Address,
        molecule_id: u64,
        target_gene: String,
        binding_affinity_pico: u64,
        model_ref: String,
    ) -> Result<u64, Error> {
        caller.require_auth();
        Self::ensure_predictor(&env, &caller)?;

        if target_gene.is_empty() || model_ref.is_empty() {
            return Err(Error::InvalidInput);
        }

        let _molecule: MolecularStructure = env
            .storage()
            .instance()
            .get(&DataKey::Molecule(molecule_id))
            .ok_or(Error::MoleculeNotFound)?;

        let prediction_id = Self::next_id(&env, &PREDICTION_COUNT);
        let confidence_bps = 8_000u32
            .saturating_add(
                ((10_000u64.saturating_sub(binding_affinity_pico.min(10_000))) / 20) as u32,
            )
            .min(9_900);

        let prediction = DrugTargetPrediction {
            prediction_id,
            molecule_id,
            target_gene,
            binding_affinity_pico,
            confidence_bps,
            model_ref,
            created_at: env.ledger().timestamp(),
        };

        env.storage()
            .instance()
            .set(&DataKey::Prediction(prediction_id), &prediction);

        Ok(prediction_id)
    }

    pub fn predict_adverse_effects(
        env: Env,
        caller: Address,
        molecule_id: u64,
        effect_code: String,
        cohort_ref: String,
        severity_bps: u32,
    ) -> Result<u64, Error> {
        caller.require_auth();
        Self::ensure_predictor(&env, &caller)?;

        if effect_code.is_empty() || cohort_ref.is_empty() || severity_bps > 10_000 {
            return Err(Error::InvalidInput);
        }

        let _molecule: MolecularStructure = env
            .storage()
            .instance()
            .get(&DataKey::Molecule(molecule_id))
            .ok_or(Error::MoleculeNotFound)?;

        let adverse_id = Self::next_id(&env, &ADVERSE_COUNT);
        let probability_bps = severity_bps.saturating_mul(8) / 10;

        let adverse = AdverseEffectPrediction {
            adverse_id,
            molecule_id,
            effect_code,
            severity_bps,
            probability_bps,
            cohort_ref,
            created_at: env.ledger().timestamp(),
        };

        env.storage()
            .instance()
            .set(&DataKey::AdversePrediction(adverse_id), &adverse);

        Ok(adverse_id)
    }

    pub fn optimize_clinical_trial_matching(
        env: Env,
        caller: Address,
        molecule_id: u64,
        protocol_id: u64,
        genomic_record_id: Option<u64>,
    ) -> Result<u64, Error> {
        caller.require_auth();
        let cfg = Self::ensure_predictor(&env, &caller)?;

        let _molecule: MolecularStructure = env
            .storage()
            .instance()
            .get(&DataKey::Molecule(molecule_id))
            .ok_or(Error::MoleculeNotFound)?;

        let trial_addr = cfg
            .clinical_trial_contract
            .ok_or(Error::IntegrationMissing)?;
        let trial_client = ClinicalTrialClient::new(&env, &trial_addr);

        let protocol = trial_client
            .get_protocol(&protocol_id)
            .ok_or(Error::InvalidInput)?;

        if let Some(record_id) = genomic_record_id {
            let genomic_addr = cfg.genomic_contract.ok_or(Error::IntegrationMissing)?;
            let _ = record_id;
            let _genomic_client = GenomicDataClient::new(&env, &genomic_addr);
        }

        let match_id = Self::next_id(&env, &MATCH_COUNT);
        let fit_score_bps = if protocol.active { 8_600 } else { 6_000 };
        let expected_enrollment_days = if protocol.active { 35 } else { 90 };

        let res = TrialMatchResult {
            match_id,
            molecule_id,
            protocol_id,
            fit_score_bps,
            expected_enrollment_days,
            matched_at: env.ledger().timestamp(),
        };

        env.storage()
            .instance()
            .set(&DataKey::TrialMatch(match_id), &res);
        Ok(match_id)
    }

    pub fn request_quantum_simulation(
        env: Env,
        caller: Address,
        molecule_id: u64,
        target_gene: String,
        algorithm: String,
        depth: u32,
        shots: u32,
    ) -> Result<u64, Error> {
        caller.require_auth();
        let cfg = Self::ensure_predictor(&env, &caller)?;

        if !cfg.quantum_enabled {
            return Err(Error::QuantumDisabled);
        }

        if target_gene.is_empty() || algorithm.is_empty() || depth == 0 || shots == 0 {
            return Err(Error::InvalidInput);
        }

        let _molecule: MolecularStructure = env
            .storage()
            .instance()
            .get(&DataKey::Molecule(molecule_id))
            .ok_or(Error::MoleculeNotFound)?;

        let simulation_id = Self::next_id(&env, &QUANTUM_COUNT);
        let req = QuantumSimulationRequest {
            simulation_id,
            molecule_id,
            target_gene,
            algorithm,
            depth,
            shots,
            queued_at: env.ledger().timestamp(),
        };

        env.storage()
            .instance()
            .set(&DataKey::QuantumRequest(simulation_id), &req);
        Ok(simulation_id)
    }

    pub fn run_screening_campaign(
        env: Env,
        caller: Address,
        target_gene: String,
        candidate_molecule_ids: Vec<u64>,
        analysis_time_hours: u32,
        used_quantum: bool,
    ) -> Result<u64, Error> {
        caller.require_auth();
        let cfg = Self::ensure_predictor(&env, &caller)?;

        if target_gene.is_empty() || candidate_molecule_ids.is_empty() {
            return Err(Error::InvalidInput);
        }

        let total = candidate_molecule_ids.len() as u32;
        let mut identified: u32 = 0;

        for id in candidate_molecule_ids.iter() {
            if env.storage().instance().has(&DataKey::Molecule(id)) {
                identified = identified.saturating_add(1);
            }
        }

        let candidate_accuracy_bps = if total == 0 {
            0
        } else {
            identified.saturating_mul(10_000) / total
        };

        if candidate_accuracy_bps < cfg.min_candidate_accuracy_bps
            || analysis_time_hours > cfg.max_analysis_time_hours
        {
            return Err(Error::BenchmarkNotMet);
        }

        if used_quantum && !cfg.quantum_enabled {
            return Err(Error::QuantumDisabled);
        }

        let campaign_id = Self::next_id(&env, &CAMPAIGN_COUNT);
        let report = ScreeningCampaignReport {
            campaign_id,
            target_gene,
            screened_candidates: total,
            identified_candidates: identified,
            candidate_accuracy_bps,
            analysis_time_hours,
            used_quantum,
            completed_at: env.ledger().timestamp(),
        };

        env.storage()
            .instance()
            .set(&DataKey::CampaignReport(campaign_id), &report);

        Ok(campaign_id)
    }

    pub fn get_config(env: Env) -> Result<PlatformConfig, Error> {
        Self::load_config(&env)
    }

    pub fn get_molecule(env: Env, molecule_id: u64) -> Result<MolecularStructure, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Molecule(molecule_id))
            .ok_or(Error::MoleculeNotFound)
    }

    pub fn get_campaign_report(
        env: Env,
        campaign_id: u64,
    ) -> Result<ScreeningCampaignReport, Error> {
        env.storage()
            .instance()
            .get(&DataKey::CampaignReport(campaign_id))
            .ok_or(Error::PredictionNotFound)
    }
}
