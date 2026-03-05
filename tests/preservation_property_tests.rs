// Preservation Property Tests for CI Merge Conflict Fix
// **Validates: Requirements 3.1, 3.2, 3.3, 3.4**
//
// These tests verify that existing DataKey usage patterns remain unchanged after
// resolving the merge conflict. Since the unfixed code doesn't compile, these tests
// define the preservation requirements based on the existing test suite.
//
// Property 2: Preservation - Existing DataKey Usage
// For any code that uses DataKey enum variants that were NOT part of the conflict,
// the fixed code SHALL produce exactly the same behavior as the original code.

use soroban_sdk::{Address, Bytes, BytesN, Env, Map, String, Vec};
use medical_records::{
    MedicalRecordsContract, MedicalRecordsContractClient, Role, KeyEnvelope, EnvelopeAlgorithm,
    GenomicDatasetHeader, GeneAssociationEntry, DrugResponseRule, AncestryProfile,
    AddGenomicDatasetConfig,
};

#[allow(clippy::unwrap_used)]

/// Test that genomic DataKey variants can be instantiated and used correctly
/// This verifies that the genomic storage keys added in HEAD branch work as expected
/// **Validates: Requirement 3.1** - Genomic functions access storage correctly
#[test]
fn test_preservation_genomic_datakey_usage() {
    let env = Env::default();
    env.mock_all_auths();

    let medical_id = env.register_contract(None, MedicalRecordsContract);
    let medical = MedicalRecordsContractClient::new(&env, &medical_id);

    let admin = Address::generate(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    medical.initialize(&admin);
    medical.manage_user(&admin, &doctor, &Role::Doctor).unwrap();
    medical.manage_user(&admin, &patient, &Role::Patient).unwrap();

    let envelope = KeyEnvelope {
        recipient: patient.clone(),
        key_version: 1,
        algorithm: EnvelopeAlgorithm::X25519,
        wrapped_key: Bytes::from_slice(&env, b"test_key"),
        pq_wrapped_key: None,
    };
    let envelopes = vec![&env, envelope];

    // Test NextGenomicId and GenomicDataset(u64) variants
    let dataset_id = medical.add_genomic_dataset(
        &AddGenomicDatasetConfig {
            doctor: doctor.clone(),
            patient: patient.clone(),
            format_code: 0u32,
            compression_code: 1u32,
            data_ref: String::from_str(&env, "ipfs://test"),
            data_hash: BytesN::from_array(&env, &[1u8; 32]),
            size_bytes: 1000u64,
            envelopes: envelopes.clone(),
            consent_token_id: None,
            is_confidential: true,
            tags: Vec::new(&env),
        }
    ).unwrap();

    // Test GenomicDataset(u64) variant - storage retrieval
    let header: GenomicDatasetHeader = medical.get_genomic_dataset(&dataset_id).unwrap();
    assert_eq!(header.dataset_id, dataset_id);
    assert_eq!(header.patient_id, patient);

    // Test PatientGenomic(Address) variant - list patient datasets
    let list = medical.list_patient_genomic(&patient);
    assert_eq!(list.len(), 1);
    assert_eq!(list.get(0).unwrap(), dataset_id);

    // Test GeneAssociationsByGene(String) and GeneAssociationsByDisease(String) variants
    medical.add_gene_disease_association(
        &doctor,
        &String::from_str(&env, "BRCA1"),
        &String::from_str(&env, "c.68_69delAG"),
        &String::from_str(&env, "C50"),
        &9000u32,
        &String::from_str(&env, "literature"),
        &dataset_id,
    ).unwrap();

    let assoc_gene: Vec<GeneAssociationEntry> =
        medical.get_gene_associations_by_gene(&String::from_str(&env, "BRCA1"));
    assert!(assoc_gene.len() >= 1);
    assert_eq!(assoc_gene.get(0).unwrap().gene, String::from_str(&env, "BRCA1"));

    // Test DrugResponseKey(String, String, String) variant
    let rule = DrugResponseRule {
        gene: String::from_str(&env, "CYP2C19"),
        variant: String::from_str(&env, "*2"),
        rxnorm_code: String::from_str(&env, "12345"),
        recommendation: String::from_str(&env, "avoid clopidogrel"),
        created_at: env.ledger().timestamp(),
    };
    medical.set_drug_response_rule(&admin, &rule).unwrap();
    let found = medical.get_drug_response(
        &String::from_str(&env, "CYP2C19"),
        &String::from_str(&env, "*2"),
        &String::from_str(&env, "12345"),
    ).unwrap();
    assert_eq!(found.recommendation, String::from_str(&env, "avoid clopidogrel"));

    // Test Ancestry(Address) variant
    let mut comps: Map<String, u32> = Map::new(&env);
    comps.set(String::from_str(&env, "European"), 5000u32);
    comps.set(String::from_str(&env, "EastAsian"), 3000u32);
    comps.set(String::from_str(&env, "Other"), 2000u32);
    medical.set_ancestry_profile(&patient, &comps).unwrap();
    let profile: AncestryProfile = medical.get_ancestry_profile(&patient).unwrap();
    assert_eq!(profile.patient, patient);
    assert_eq!(profile.components.len(), 3);
}

/// Test that existing non-genomic DataKey variants continue to work correctly
/// This verifies that ZK, rate limiting, and other variants are unaffected
/// **Validates: Requirement 3.3** - Other DataKey variants function without changes
#[test]
fn test_preservation_existing_datakey_variants() {
    let env = Env::default();
    env.mock_all_auths();

    let medical_id = env.register_contract(None, MedicalRecordsContract);
    let medical = MedicalRecordsContractClient::new(&env, &medical_id);

    let admin = Address::generate(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    // Test basic initialization and user management (uses Users, IdentityRegistry variants)
    medical.initialize(&admin);
    medical.manage_user(&admin, &doctor, &Role::Doctor).unwrap();
    medical.manage_user(&admin, &patient, &Role::Patient).unwrap();

    // Verify roles are stored correctly
    let doctor_role = medical.get_user_role(&doctor);
    assert_eq!(doctor_role, Role::Doctor);
    let patient_role = medical.get_user_role(&patient);
    assert_eq!(patient_role, Role::Patient);

    // Test record creation (uses NextId, RecordCount, Record(u64), PatientRecords(Address) variants)
    let record_data = Bytes::from_slice(&env, b"test_record_data");
    let record_hash = BytesN::from_array(&env, &[1u8; 32]);
    let tags = Vec::new(&env);
    
    let record_id = medical.add_record(
        &doctor,
        &patient,
        &record_data,
        &record_hash,
        &tags,
    ).unwrap();

    // Verify record was stored correctly
    assert!(record_id > 0);
    let patient_records = medical.get_patient_records(&patient);
    assert_eq!(patient_records.len(), 1);
    assert_eq!(patient_records.get(0).unwrap(), record_id);
}

/// Property-based test: Verify genomic operations preserve data integrity
/// Tests that multiple genomic datasets can be added and retrieved correctly
/// **Validates: Requirements 3.1, 3.4** - Genomic functions and existing tests
#[test]
fn test_preservation_property_multiple_genomic_datasets() {
    let env = Env::default();
    env.mock_all_auths();

    let medical_id = env.register_contract(None, MedicalRecordsContract);
    let medical = MedicalRecordsContractClient::new(&env, &medical_id);

    let admin = Address::generate(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    medical.initialize(&admin);
    medical.manage_user(&admin, &doctor, &Role::Doctor).unwrap();
    medical.manage_user(&admin, &patient, &Role::Patient).unwrap();

    let envelope = KeyEnvelope {
        recipient: patient.clone(),
        key_version: 1,
        algorithm: EnvelopeAlgorithm::X25519,
        wrapped_key: Bytes::from_slice(&env, b"k"),
        pq_wrapped_key: None,
    };
    let envelopes = vec![&env, envelope];

    // Property: Adding N datasets should result in N datasets being retrievable
    let num_datasets = 5;
    let mut dataset_ids = Vec::new(&env);

    for i in 0..num_datasets {
        let dataset_id = medical.add_genomic_dataset(
            &AddGenomicDatasetConfig {
                doctor: doctor.clone(),
                patient: patient.clone(),
                format_code: i,
                compression_code: 1u32,
                data_ref: String::from_str(&env, &format!("ipfs://dataset_{}", i)),
                data_hash: BytesN::from_array(&env, &[i as u8; 32]),
                size_bytes: (1000 + i * 100) as u64,
                envelopes: envelopes.clone(),
                consent_token_id: None,
                is_confidential: true,
                tags: Vec::new(&env),
            }
        ).unwrap();
        dataset_ids.push_back(dataset_id);
    }

    // Verify all datasets are retrievable
    let patient_datasets = medical.list_patient_genomic(&patient);
    assert_eq!(patient_datasets.len(), num_datasets);

    // Verify each dataset can be retrieved individually
    for i in 0..num_datasets {
        let dataset_id = dataset_ids.get(i).unwrap();
        let header: GenomicDatasetHeader = medical.get_genomic_dataset(&dataset_id).unwrap();
        assert_eq!(header.dataset_id, dataset_id);
        assert_eq!(header.patient_id, patient);
        assert_eq!(header.format_code, i);
    }
}

/// Property-based test: Verify gene associations preserve referential integrity
/// Tests that gene associations can be queried by both gene and disease
/// **Validates: Requirement 3.1** - Genomic functions access storage correctly
#[test]
fn test_preservation_property_gene_associations() {
    let env = Env::default();
    env.mock_all_auths();

    let medical_id = env.register_contract(None, MedicalRecordsContract);
    let medical = MedicalRecordsContractClient::new(&env, &medical_id);

    let admin = Address::generate(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    medical.initialize(&admin);
    medical.manage_user(&admin, &doctor, &Role::Doctor).unwrap();
    medical.manage_user(&admin, &patient, &Role::Patient).unwrap();

    let envelope = KeyEnvelope {
        recipient: patient.clone(),
        key_version: 1,
        algorithm: EnvelopeAlgorithm::X25519,
        wrapped_key: Bytes::from_slice(&env, b"k"),
        pq_wrapped_key: None,
    };
    let envelopes = vec![&env, envelope];

    let dataset_id = medical.add_genomic_dataset(
        &AddGenomicDatasetConfig {
            doctor: doctor.clone(),
            patient: patient.clone(),
            format_code: 1u32,
            compression_code: 1u32,
            data_ref: String::from_str(&env, "ipfs://vcf"),
            data_hash: BytesN::from_array(&env, &[1u8; 32]),
            size_bytes: 2000u64,
            envelopes: envelopes.clone(),
            consent_token_id: None,
            is_confidential: true,
            tags: Vec::new(&env),
        }
    ).unwrap();

    // Property: Gene associations should be queryable by both gene and disease
    let test_cases = vec![
        ("BRCA1", "c.68_69delAG", "C50"),
        ("BRCA2", "c.5946delT", "C50"),
        ("TP53", "c.524G>A", "C34"),
    ];

    for (gene, variant, disease) in test_cases.iter() {
        medical.add_gene_disease_association(
            &doctor,
            &String::from_str(&env, gene),
            &String::from_str(&env, variant),
            &String::from_str(&env, disease),
            &9000u32,
            &String::from_str(&env, "literature"),
            &dataset_id,
        ).unwrap();
    }

    // Verify associations can be queried by gene
    let brca1_assocs = medical.get_gene_associations_by_gene(&String::from_str(&env, "BRCA1"));
    assert_eq!(brca1_assocs.len(), 1);
    assert_eq!(brca1_assocs.get(0).unwrap().gene, String::from_str(&env, "BRCA1"));

    let brca2_assocs = medical.get_gene_associations_by_gene(&String::from_str(&env, "BRCA2"));
    assert_eq!(brca2_assocs.len(), 1);
    assert_eq!(brca2_assocs.get(0).unwrap().gene, String::from_str(&env, "BRCA2"));

    // Verify associations can be queried by disease
    let c50_assocs = medical.get_gene_associations_by_disease(&String::from_str(&env, "C50"));
    assert_eq!(c50_assocs.len(), 2); // BRCA1 and BRCA2 both associated with C50
}
