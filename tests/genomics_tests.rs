use soroban_sdk::{Address, Bytes, BytesN, Env, Map, String, Vec};
use medical_records::{
    MedicalRecordsContract, MedicalRecordsContractClient, Role, KeyEnvelope, EnvelopeAlgorithm,
    GenomicDatasetHeader, GeneAssociationEntry, DrugResponseRule, AncestryProfile,
};
use zk_verifier::{ZkVerifierContract, ZkVerifierContractClient};

#[allow(clippy::unwrap_used)]

pub mod genomics_tests {
    use super::*;

    #[test]
    fn test_genomic_dataset_storage_and_queries() {
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

        let dataset_id_fasta = medical.add_genomic_dataset(
            &doctor,
            &patient,
            &0u32,
            &1u32,
            &String::from_str(&env, "ipfs://fasta"),
            &BytesN::from_array(&env, &[1u8; 32]),
            &1234u64,
            &envelopes,
            &None,
            &true,
            &Vec::new(&env),
        ).unwrap();
        let dataset_id_vcf = medical.add_genomic_dataset(
            &doctor,
            &patient,
            &1u32,
            &2u32,
            &String::from_str(&env, "ipfs://vcf"),
            &BytesN::from_array(&env, &[2u8; 32]),
            &2345u64,
            &envelopes,
            &None,
            &true,
            &Vec::new(&env),
        ).unwrap();
        let header_fasta: GenomicDatasetHeader = medical.get_genomic_dataset(&dataset_id_fasta).unwrap();
        assert_eq!(header_fasta.dataset_id, dataset_id_fasta);
        assert_eq!(header_fasta.patient_id, patient);
        let list = medical.list_patient_genomic(&patient);
        assert_eq!(list.len(), 2);

        medical.add_gene_disease_association(
            &doctor,
            &String::from_str(&env, "BRCA1"),
            &String::from_str(&env, "c.68_69delAG"),
            &String::from_str(&env, "C50"),
            &9000u32,
            &String::from_str(&env, "literature"),
            &dataset_id_vcf,
        ).unwrap();
        let assoc_gene: Vec<GeneAssociationEntry> =
            medical.get_gene_associations_by_gene(&String::from_str(&env, "BRCA1"));
        assert!(assoc_gene.len() >= 1);

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

        let mut comps: Map<String, u32> = Map::new(&env);
        comps.set(String::from_str(&env, "WestAfrican"), 4000u32);
        comps.set(String::from_str(&env, "European"), 3000u32);
        comps.set(String::from_str(&env, "EastAsian"), 2000u32);
        comps.set(String::from_str(&env, "Other"), 1000u32);
        medical.set_ancestry_profile(&patient, &comps).unwrap();
        let profile: AncestryProfile = medical.get_ancestry_profile(&patient).unwrap();
        assert_eq!(profile.patient, patient);
    }

    #[test]
    fn test_privacy_preserving_research_access() {
        let env = Env::default();
        env.mock_all_auths();

        let medical_id = env.register_contract(None, MedicalRecordsContract);
        let medical = MedicalRecordsContractClient::new(&env, &medical_id);
        let zk_id = env.register_contract(None, ZkVerifierContract);
        let zk = ZkVerifierContractClient::new(&env, &zk_id);

        let admin = Address::generate(&env);
        let doctor = Address::generate(&env);
        let patient = Address::generate(&env);
        let attestor = Address::generate(&env);
        let requester = Address::generate(&env);

        medical.initialize(&admin);
        medical.manage_user(&admin, &doctor, &Role::Doctor).unwrap();
        medical.manage_user(&admin, &patient, &Role::Patient).unwrap();
        medical.set_zk_verifier_contract(&admin, &zk_id).unwrap();

        zk.initialize(&admin, &600).unwrap();
        let vk_hash = BytesN::from_array(&env, &[1u8; 32]);
        let circuit_id = BytesN::from_array(&env, &[2u8; 32]);
        let metadata_hash = BytesN::from_array(&env, &[3u8; 32]);
        let version = zk.register_verifying_key(&admin, &vk_hash, &circuit_id, &attestor, &metadata_hash);
        assert_eq!(version, 1);

        let envelope = KeyEnvelope {
            recipient: patient.clone(),
            key_version: 1,
            algorithm: EnvelopeAlgorithm::X25519,
            wrapped_key: Bytes::from_slice(&env, b"k"),
            pq_wrapped_key: None,
        };
        let envelopes = vec![&env, envelope];
        let dataset_id = medical.add_genomic_dataset(
            &doctor,
            &patient,
            &1u32,
            &1u32,
            &String::from_str(&env, "ipfs://vcf"),
            &BytesN::from_array(&env, &[9u8; 32]),
            &3456u64,
            &envelopes,
            &None,
            &true,
            &Vec::new(&env),
        ).unwrap();

        let public_inputs_hash = BytesN::from_array(&env, &[7u8; 32]);
        let proof = Bytes::from_slice(&env, b"proof-vcf");
        let proof_hash: BytesN<32> = env.crypto().sha256(&proof).into();
        zk.submit_attestation(&attestor, &1, &public_inputs_hash, &proof_hash, &true, &300).unwrap();

        medical.grant_privacy_preserving_genomic_access(
            &admin,
            &requester,
            &dataset_id,
            &1u32,
            &public_inputs_hash,
            &proof,
            &Some(120u64),
        ).unwrap();
        let ok = medical.has_valid_zk_access_grant(&requester, &dataset_id);
        assert!(ok);
    }
}
