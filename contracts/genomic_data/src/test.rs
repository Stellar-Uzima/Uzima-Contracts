extern crate std;

use soroban_sdk::{testutils::Address as _, vec, Address, BytesN, Env, String, Vec};

use crate::{
    AnalysisType, BreachSeverity, BreachStatus, ConsentPurpose,
    GenomicDataContract, GenomicDataContractClient, GenomicDataFormat,
    MetabolizerStatus,
};

fn setup_env() -> (Env, GenomicDataContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(GenomicDataContract, ());
    let client = GenomicDataContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, client, admin)
}

fn dummy_hash(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[1u8; 32])
}

fn dummy_hash_2(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[2u8; 32])
}

// ==================== Initialization Tests ====================

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(GenomicDataContract, ());
    let client = GenomicDataContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
}

#[test]
#[should_panic]
fn test_double_initialize_fails() {
    let (env, client, _admin) = setup_env();
    let admin2 = Address::generate(&env);
    client.initialize(&admin2);
}

// ==================== Genomic Data Storage Tests ====================

#[test]
fn test_store_and_retrieve_genomic_data() {
    let (env, client, _admin) = setup_env();
    let owner = Address::generate(&env);

    let record_id = client.store_genomic_data(
        &owner,
        &GenomicDataFormat::Fasta,
        &dummy_hash(&env),
        &String::from_str(&env, "gzip"),
        &dummy_hash_2(&env),
        &1000u64,
        &95u32,
        &String::from_str(&env, "GRCh38"),
    );

    assert_eq!(record_id, 1);

    let record = client.get_genomic_data(&record_id);
    assert_eq!(record.owner, owner);
    assert_eq!(record.format, GenomicDataFormat::Fasta);
    assert_eq!(record.size_bytes, 1000);
    assert_eq!(record.quality_score, 95);
}

#[test]
fn test_store_multiple_formats() {
    let (env, client, _admin) = setup_env();
    let owner = Address::generate(&env);

    let formats = [
        GenomicDataFormat::Fasta,
        GenomicDataFormat::Vcf,
        GenomicDataFormat::Bam,
        GenomicDataFormat::Cram,
        GenomicDataFormat::Bed,
        GenomicDataFormat::Gff,
    ];

    for (i, format) in formats.iter().enumerate() {
        let id = client.store_genomic_data(
            &owner,
            format,
            &dummy_hash(&env),
            &String::from_str(&env, "zstd"),
            &dummy_hash_2(&env),
            &(500u64 * (i as u64 + 1)),
            &90u32,
            &String::from_str(&env, "GRCh38"),
        );
        let record = client.get_genomic_data(&id);
        assert_eq!(record.format, *format);
    }
}

#[test]
#[should_panic]
fn test_get_nonexistent_record_fails() {
    let (_env, client, _admin) = setup_env();
    client.get_genomic_data(&999);
}

// ==================== Consent Management Tests ====================

#[test]
fn test_grant_and_check_consent() {
    let (env, client, _admin) = setup_env();
    let patient = Address::generate(&env);
    let researcher = Address::generate(&env);

    let consent_id = client.grant_consent(
        &patient,
        &researcher,
        &ConsentPurpose::Research,
        &String::from_str(&env, "full_genome"),
        &0u64,    // no expiry
        &true,    // revocable
    );

    assert_eq!(consent_id, 1);

    let has_consent =
        client.check_consent(&patient, &researcher, &ConsentPurpose::Research);
    assert!(has_consent);

    // Different purpose should not match
    let no_consent =
        client.check_consent(&patient, &researcher, &ConsentPurpose::Marketplace);
    assert!(!no_consent);
}

#[test]
fn test_revoke_consent() {
    let (env, client, _admin) = setup_env();
    let patient = Address::generate(&env);
    let researcher = Address::generate(&env);

    let consent_id = client.grant_consent(
        &patient,
        &researcher,
        &ConsentPurpose::Research,
        &String::from_str(&env, "exome"),
        &0u64,
        &true,
    );

    // Consent is active
    assert!(client.check_consent(&patient, &researcher, &ConsentPurpose::Research));

    // Revoke
    client.revoke_consent(&patient, &consent_id);

    // After revocation, consent should not be valid
    assert!(!client.check_consent(&patient, &researcher, &ConsentPurpose::Research));
}

// ==================== Gene-Disease Association Tests ====================

#[test]
fn test_register_and_query_gene_disease_association() {
    let (env, client, _admin) = setup_env();
    let reporter = Address::generate(&env);

    let gene = String::from_str(&env, "BRCA1");
    let disease = String::from_str(&env, "breast_cancer");

    let assoc_id = client.register_gene_disease_assoc(
        &reporter,
        &gene,
        &disease,
        &85u32,
        &3u32,
        &String::from_str(&env, "rs80357906"),
    );

    assert_eq!(assoc_id, 1);

    // Register another association for the same gene
    client.register_gene_disease_assoc(
        &reporter,
        &gene,
        &String::from_str(&env, "ovarian_cancer"),
        &72u32,
        &2u32,
        &String::from_str(&env, "rs80358981"),
    );

    let results = client.query_associations_by_gene(&gene);
    assert_eq!(results.len(), 2);
}

#[test]
fn test_query_empty_gene_returns_empty() {
    let (env, client, _admin) = setup_env();
    let results =
        client.query_associations_by_gene(&String::from_str(&env, "UNKNOWN_GENE"));
    assert_eq!(results.len(), 0);
}

// ==================== Genomic Analysis Tests ====================

#[test]
fn test_run_and_get_analysis() {
    let (env, client, _admin) = setup_env();
    let owner = Address::generate(&env);
    let analyst = Address::generate(&env);

    // First store a genomic record
    let record_id = client.store_genomic_data(
        &owner,
        &GenomicDataFormat::Vcf,
        &dummy_hash(&env),
        &String::from_str(&env, "bgzip"),
        &dummy_hash_2(&env),
        &2000u64,
        &92u32,
        &String::from_str(&env, "GRCh38"),
    );

    let patterns: Vec<String> = vec![
        &env,
        String::from_str(&env, "SNP_rs12345"),
        String::from_str(&env, "INDEL_chr1_100"),
    ];
    let risks: Vec<String> = vec![
        &env,
        String::from_str(&env, "elevated_cardiac"),
    ];

    let analysis_id = client.run_analysis(
        &analyst,
        &record_id,
        &AnalysisType::VariantCalling,
        &patterns,
        &88u32,
        &risks,
    );

    let result = client.get_analysis(&analysis_id);
    assert_eq!(result.record_id, record_id);
    assert_eq!(result.analysis_type, AnalysisType::VariantCalling);
    assert_eq!(result.confidence_score, 88);
    assert_eq!(result.patterns_found.len(), 2);
    assert_eq!(result.risk_factors.len(), 1);
}

#[test]
#[should_panic]
fn test_analysis_on_nonexistent_record_fails() {
    let (env, client, _admin) = setup_env();
    let analyst = Address::generate(&env);

    client.run_analysis(
        &analyst,
        &999u64,
        &AnalysisType::RiskAssessment,
        &Vec::new(&env),
        &50u32,
        &Vec::new(&env),
    );
}

// ==================== Research Sharing Tests ====================

#[test]
fn test_share_for_research_with_valid_consent() {
    let (env, client, _admin) = setup_env();
    let patient = Address::generate(&env);
    let researcher = Address::generate(&env);

    // Store data
    let record_id = client.store_genomic_data(
        &patient,
        &GenomicDataFormat::Bam,
        &dummy_hash(&env),
        &String::from_str(&env, "lz4"),
        &dummy_hash_2(&env),
        &5000u64,
        &97u32,
        &String::from_str(&env, "GRCh38"),
    );

    // Grant research consent
    let consent_id = client.grant_consent(
        &patient,
        &researcher,
        &ConsentPurpose::Research,
        &String::from_str(&env, "variant_analysis"),
        &0u64,
        &true,
    );

    // Share for research
    let share_id = client.share_for_research(
        &patient,
        &record_id,
        &researcher,
        &String::from_str(&env, "MIT_Genomics_Lab"),
        &dummy_hash(&env),
        &consent_id,
    );

    assert_eq!(share_id, 1);
}

#[test]
#[should_panic]
fn test_share_for_research_with_revoked_consent_fails() {
    let (env, client, _admin) = setup_env();
    let patient = Address::generate(&env);
    let researcher = Address::generate(&env);

    let record_id = client.store_genomic_data(
        &patient,
        &GenomicDataFormat::Vcf,
        &dummy_hash(&env),
        &String::from_str(&env, "gzip"),
        &dummy_hash_2(&env),
        &1000u64,
        &90u32,
        &String::from_str(&env, "GRCh38"),
    );

    let consent_id = client.grant_consent(
        &patient,
        &researcher,
        &ConsentPurpose::Research,
        &String::from_str(&env, "full"),
        &0u64,
        &true,
    );

    // Revoke consent
    client.revoke_consent(&patient, &consent_id);

    // Sharing should fail
    client.share_for_research(
        &patient,
        &record_id,
        &researcher,
        &String::from_str(&env, "Lab"),
        &dummy_hash(&env),
        &consent_id,
    );
}

// ==================== Marketplace Tests ====================

#[test]
fn test_create_and_purchase_listing() {
    let (env, client, _admin) = setup_env();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    let record_id = client.store_genomic_data(
        &seller,
        &GenomicDataFormat::Vcf,
        &dummy_hash(&env),
        &String::from_str(&env, "gzip"),
        &dummy_hash_2(&env),
        &3000u64,
        &94u32,
        &String::from_str(&env, "GRCh38"),
    );

    let listing_id = client.create_marketplace_listing(
        &seller,
        &record_id,
        &100u64,
        &3u32,
        &String::from_str(&env, "research_only"),
    );

    assert_eq!(listing_id, 1);

    // Purchase
    client.purchase_listing(&buyer, &listing_id);
}

#[test]
#[should_panic]
fn test_cannot_purchase_own_listing() {
    let (env, client, _admin) = setup_env();
    let seller = Address::generate(&env);

    let record_id = client.store_genomic_data(
        &seller,
        &GenomicDataFormat::Fasta,
        &dummy_hash(&env),
        &String::from_str(&env, "gzip"),
        &dummy_hash_2(&env),
        &1000u64,
        &90u32,
        &String::from_str(&env, "GRCh38"),
    );

    let listing_id = client.create_marketplace_listing(
        &seller,
        &record_id,
        &50u64,
        &2u32,
        &String::from_str(&env, "open"),
    );

    // Seller cannot buy their own listing
    client.purchase_listing(&seller, &listing_id);
}

#[test]
#[should_panic]
fn test_non_owner_cannot_list() {
    let (env, client, _admin) = setup_env();
    let owner = Address::generate(&env);
    let other = Address::generate(&env);

    let record_id = client.store_genomic_data(
        &owner,
        &GenomicDataFormat::Fasta,
        &dummy_hash(&env),
        &String::from_str(&env, "gzip"),
        &dummy_hash_2(&env),
        &1000u64,
        &90u32,
        &String::from_str(&env, "GRCh38"),
    );

    // Non-owner tries to list
    client.create_marketplace_listing(
        &other,
        &record_id,
        &50u64,
        &2u32,
        &String::from_str(&env, "open"),
    );
}

// ==================== Pharmacogenomics Tests ====================

#[test]
fn test_store_and_get_pharmacogenomic_profile() {
    let (env, client, _admin) = setup_env();
    let patient = Address::generate(&env);

    client.store_pharmacogenomic_profile(
        &patient,
        &String::from_str(&env, "CYP2D6"),
        &String::from_str(&env, "codeine"),
        &MetabolizerStatus::PoorMetabolizer,
        &String::from_str(&env, "avoid_codeine_use_alternative"),
        &4u32,
    );

    let profile = client.get_pharmacogenomic_profile(&patient);
    assert_eq!(profile.patient, patient);
    assert_eq!(profile.metabolizer_status, MetabolizerStatus::PoorMetabolizer);
    assert_eq!(profile.risk_level, 4);
}

#[test]
#[should_panic]
fn test_get_nonexistent_pharma_profile_fails() {
    let (env, client, _admin) = setup_env();
    let unknown = Address::generate(&env);
    client.get_pharmacogenomic_profile(&unknown);
}

// ==================== Ancestry Tests ====================

#[test]
fn test_store_and_get_ancestry_record() {
    let (env, client, _admin) = setup_env();
    let patient = Address::generate(&env);

    let labels: Vec<String> = vec![
        &env,
        String::from_str(&env, "European"),
        String::from_str(&env, "West_African"),
        String::from_str(&env, "East_Asian"),
    ];
    let percentages: Vec<u32> = vec![&env, 55, 30, 15];

    client.store_ancestry_record(
        &patient,
        &labels,
        &percentages,
        &String::from_str(&env, "H1a"),
        &String::from_str(&env, "R1b"),
        &String::from_str(&env, "M168_M89_M9"),
    );

    let record = client.get_ancestry_record(&patient);
    assert_eq!(record.patient, patient);
    assert_eq!(record.population_labels.len(), 3);
    assert_eq!(record.population_percentages.len(), 3);
}

#[test]
#[should_panic]
fn test_ancestry_mismatched_lengths_fails() {
    let (env, client, _admin) = setup_env();
    let patient = Address::generate(&env);

    let labels: Vec<String> = vec![
        &env,
        String::from_str(&env, "European"),
        String::from_str(&env, "African"),
    ];
    let percentages: Vec<u32> = vec![&env, 60]; // mismatch

    client.store_ancestry_record(
        &patient,
        &labels,
        &percentages,
        &String::from_str(&env, "H"),
        &String::from_str(&env, "R"),
        &String::from_str(&env, "M"),
    );
}

// ==================== Breach Detection Tests ====================

#[test]
fn test_report_and_get_breach() {
    let (env, client, _admin) = setup_env();
    let reporter = Address::generate(&env);

    let affected: Vec<u64> = vec![&env, 1, 2, 3];

    let breach_id = client.report_breach(
        &reporter,
        &affected,
        &BreachSeverity::High,
        &String::from_str(&env, "unauthorized_api_access"),
        &String::from_str(&env, "revoke_all_tokens"),
    );

    assert_eq!(breach_id, 1);

    let incident = client.get_breach(&breach_id);
    assert_eq!(incident.severity, BreachSeverity::High);
    assert_eq!(incident.status, BreachStatus::Detected);
    assert_eq!(incident.affected_record_ids.len(), 3);
}

#[test]
fn test_update_breach_status() {
    let (env, client, admin) = setup_env();
    let reporter = Address::generate(&env);

    let affected: Vec<u64> = vec![&env, 5];
    let breach_id = client.report_breach(
        &reporter,
        &affected,
        &BreachSeverity::Critical,
        &String::from_str(&env, "data_exfiltration"),
        &String::from_str(&env, "isolate_systems"),
    );

    // Admin updates status to investigating
    client.update_breach_status(
        &admin,
        &breach_id,
        &BreachStatus::Investigating,
        &String::from_str(&env, "forensic_analysis_underway"),
    );

    let incident = client.get_breach(&breach_id);
    assert_eq!(incident.status, BreachStatus::Investigating);

    // Resolve the breach
    client.update_breach_status(
        &admin,
        &breach_id,
        &BreachStatus::Resolved,
        &String::from_str(&env, "patched_and_monitoring"),
    );

    let resolved = client.get_breach(&breach_id);
    assert_eq!(resolved.status, BreachStatus::Resolved);
    assert_eq!(resolved.resolved_at, env.ledger().timestamp());
}

#[test]
#[should_panic]
fn test_non_admin_cannot_update_breach() {
    let (env, client, _admin) = setup_env();
    let reporter = Address::generate(&env);
    let non_admin = Address::generate(&env);

    let affected: Vec<u64> = vec![&env, 1];
    let breach_id = client.report_breach(
        &reporter,
        &affected,
        &BreachSeverity::Low,
        &String::from_str(&env, "minor"),
        &String::from_str(&env, "monitor"),
    );

    // Non-admin tries to update — should fail
    client.update_breach_status(
        &non_admin,
        &breach_id,
        &BreachStatus::Resolved,
        &String::from_str(&env, "done"),
    );
}
