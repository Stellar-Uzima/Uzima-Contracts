use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{vec, Address, BytesN, Env, String};

#[test]
fn test_initialize_and_add_record() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    assert!(client.initialize(&admin));

    let patient = Address::generate(&env);
    let uploader = Address::generate(&env);
    let data_ref = String::from_str(&env, "ipfs://QmData");
    let data_hash = BytesN::from_array(&env, &[1u8; 32]);
    let cipher_hash = BytesN::from_array(&env, &[2u8; 32]);
    let tags = vec![
        &env,
        String::from_str(&env, "genomics"),
        String::from_str(&env, "vcf"),
    ];
    let envelopes = Vec::new(&env);

    let id = client.add_record(
        &patient,
        &uploader,
        &GenomicFormat::Vcf,
        &Compression::Zstd,
        &data_ref,
        &data_hash,
        &cipher_hash,
        &tags,
        &envelopes,
        &None,
    );
    assert!(id > 0);

    let header_for_patient = client.get_record_header(&patient, &id);
    assert!(header_for_patient.is_some());

    let stranger = Address::generate(&env);
    let header_for_stranger = client.get_record_header(&stranger, &id);
    assert!(header_for_stranger.is_none());
}

#[test]
fn test_consent_and_access() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let patient = Address::generate(&env);
    let uploader = Address::generate(&env);
    let rid = client.add_record(
        &patient,
        &uploader,
        &GenomicFormat::Fasta,
        &Compression::Gzip,
        &String::from_str(&env, "s3://bucket/file.fasta.gz"),
        &BytesN::from_array(&env, &[3u8; 32]),
        &BytesN::from_array(&env, &[4u8; 32]),
        &Vec::new(&env),
        &Vec::new(&env),
        &None,
    );
    let researcher = Address::generate(&env);
    let ok = client.grant_consent(
        &patient,
        &rid,
        &researcher,
        &String::from_str(&env, "read_header"),
        &0u64,
    );
    assert!(ok);
    let header = client.get_record_header(&researcher, &rid);
    assert!(header.is_some());
}

#[test]
fn test_marketplace_listing() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let patient = Address::generate(&env);
    let uploader = Address::generate(&env);
    let rid = client.add_record(
        &patient,
        &uploader,
        &GenomicFormat::Bam,
        &Compression::None,
        &String::from_str(&env, "ar://data"),
        &BytesN::from_array(&env, &[5u8; 32]),
        &BytesN::from_array(&env, &[6u8; 32]),
        &Vec::new(&env),
        &Vec::new(&env),
        &None,
    );
    let currency = Address::generate(&env);
    let lid = client.create_listing(&uploader, &rid, &1000i128, &currency, &None);
    assert!(lid > 0);
    let buyer = Address::generate(&env);
    let ok = client.purchase_listing(&buyer, &lid);
    assert!(ok);
}

#[test]
fn test_analysis_endpoints() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let patient = Address::generate(&env);
    let uploader = Address::generate(&env);
    let rid = client.add_record(
        &patient,
        &uploader,
        &GenomicFormat::Vcf,
        &Compression::Zstd,
        &String::from_str(&env, "ipfs://QmX"),
        &BytesN::from_array(&env, &[7u8; 32]),
        &BytesN::from_array(&env, &[8u8; 32]),
        &Vec::new(&env),
        &Vec::new(&env),
        &None,
    );
    let idx = client.add_gene_disease_assoc(
        &uploader,
        &rid,
        &String::from_str(&env, "BRCA1"),
        &String::from_str(&env, "C50"),
        &9500u32,
        &String::from_str(&env, "GWAS"),
    );
    assert_eq!(idx, 1);
    let comps = vec![
        &env,
        PopulationShare {
            label: String::from_str(&env, "Europe"),
            bps: 6000,
        },
        PopulationShare {
            label: String::from_str(&env, "Africa"),
            bps: 2000,
        },
        PopulationShare {
            label: String::from_str(&env, "Asia"),
            bps: 2000,
        },
    ];
    let ok = client.set_ancestry_profile(
        &uploader,
        &rid,
        &comps,
        &String::from_str(&env, "ADMIXTURE"),
    );
    assert!(ok);
    let didx = client.add_drug_response(
        &uploader,
        &rid,
        &String::from_str(&env, "Clopidogrel"),
        &String::from_str(&env, "CYP2C19*2"),
        &String::from_str(&env, "Reduced efficacy"),
        &String::from_str(&env, "Consider alternative"),
    );
    assert_eq!(didx, 1);
}

fn setup(env: &Env) -> (GenomicDataContractClient<'_>, Address) {
    let contract_id = env.register_contract(None, GenomicDataContract {});
    let client = GenomicDataContractClient::new(env, &contract_id);
    (client, contract_id)
}
