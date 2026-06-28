//! Storage-read regression benchmarks for medical_records.
#![allow(clippy::unwrap_used)]
extern crate std;

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Bytes, BytesN, Env, String, Vec};

fn measure_cpu<F: FnOnce()>(env: &Env, f: F) -> u64 {
    env.budget().reset_unlimited();
    f();
    env.budget().cpu_instruction_cost()
}

fn print_delta(name: &str, before: u64, after: u64) {
    let saved = before.saturating_sub(after);
    let reduction_pct = if before == 0 {
        0.0
    } else {
        (saved as f64 * 100.0) / before as f64
    };
    std::println!(
        "[STORAGE-BENCH] {} before={} after={} saved={} reduction_pct={:.2}",
        name,
        before,
        after,
        saved,
        reduction_pct
    );
}

fn setup_contract(env: &Env) -> (MedicalRecordsContractClient<'_>, Address, Address, Address) {
    env.mock_all_auths();

    let admin = Address::generate(env);
    let rbac_id = env.register_contract(None, MockRbac);
    let rbac_client = MockRbacClient::new(env, &rbac_id);
    rbac_client.assign_role(&admin, &RbacRole::Admin);

    let contract_id = Address::generate(env);
    env.register_contract(&contract_id, MedicalRecordsContract);
    let client = MedicalRecordsContractClient::new(env, &contract_id);
    client.initialize(&admin, &rbac_id);

    let doctor = Address::generate(env);
    let patient = Address::generate(env);
    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    (client, admin, doctor, patient)
}

fn old_manage_user_rbac_flow(
    env: &Env,
    caller: &Address,
    user: &Address,
    previous_role: Option<Role>,
    new_role: Role,
) -> Result<(), Error> {
    MedicalRecordsContract::require_admin(env, caller)?;
    MedicalRecordsContract::sync_rbac_role(env, user, previous_role, new_role)
}

fn new_manage_user_rbac_flow(
    env: &Env,
    caller: &Address,
    user: &Address,
    previous_role: Option<Role>,
    new_role: Role,
) -> Result<(), Error> {
    let users = MedicalRecordsContract::read_users(env);
    let rbac_addr = MedicalRecordsContract::load_rbac_contract(env).ok_or(Error::Unauthorized)?;
    if !MedicalRecordsContract::is_active_role_with_context(
        env,
        &users,
        &rbac_addr,
        caller,
        RbacRole::Admin,
    ) {
        return Err(Error::Unauthorized);
    }
    MedicalRecordsContract::sync_rbac_role_with_contract(
        env,
        &rbac_addr,
        user,
        previous_role,
        new_role,
    )
}

#[must_use]
fn old_history_gate(env: &Env, caller: &Address, patient: &Address) -> Result<(), Error> {
    if caller != patient
        && !MedicalRecordsContract::is_admin(env, caller)
        && !MedicalRecordsContract::is_active_doctor(env, caller)
    {
        return Err(Error::Unauthorized);
    }
    Ok(())
}

#[must_use]
fn new_history_gate(env: &Env, caller: &Address, patient: &Address) -> Result<(), Error> {
    let access_ctx = if caller == patient {
        None
    } else {
        Some((
            MedicalRecordsContract::read_users(env),
            MedicalRecordsContract::load_rbac_contract(env),
        ))
    };
    let is_admin = access_ctx
        .as_ref()
        .and_then(|(users, rbac_addr)| {
            rbac_addr.as_ref().map(|addr| {
                MedicalRecordsContract::is_active_role_with_context(
                    env,
                    users,
                    addr,
                    caller,
                    RbacRole::Admin,
                )
            })
        })
        .unwrap_or(false);
    let is_active_doctor = access_ctx
        .as_ref()
        .and_then(|(users, rbac_addr)| {
            rbac_addr.as_ref().map(|addr| {
                MedicalRecordsContract::is_active_role_with_context(
                    env,
                    users,
                    addr,
                    caller,
                    RbacRole::Doctor,
                )
            })
        })
        .unwrap_or(false);
    if caller != patient && !is_admin && !is_active_doctor {
        return Err(Error::Unauthorized);
    }
    Ok(())
}

fn sample_encrypted_record(
    env: &Env,
    patient: &Address,
    doctor: &Address,
    is_confidential: bool,
) -> EncryptedRecord {
    let envelope = KeyEnvelope {
        recipient: doctor.clone(),
        key_version: 1,
        algorithm: EnvelopeAlgorithm::X25519,
        wrapped_key: Bytes::new(env),
        pq_wrapped_key: None,
    };
    let mut envelopes = Vec::new(env);
    envelopes.push_back(envelope);

    EncryptedRecord {
        patient_id: patient.clone(),
        doctor_id: doctor.clone(),
        timestamp: env.ledger().timestamp(),
        is_confidential,
        tags: Vec::new(env),
        category: String::from_str(env, "General"),
        treatment_type: String::from_str(env, "Medication"),
        ciphertext_ref: String::from_str(env, "cipher://bench"),
        ciphertext_hash: BytesN::from_array(env, &[7u8; 32]),
        envelopes,
        doctor_did: None,
    }
}

fn old_can_view_encrypted_record(
    env: &Env,
    caller: &Address,
    record: &EncryptedRecord,
    record_id: u64,
) -> bool {
    if MedicalRecordsContract::is_admin(env, caller) {
        return true;
    }
    if *caller == record.patient_id {
        return true;
    }
    if *caller == record.doctor_id {
        return true;
    }
    if MedicalRecordsContract::is_active_doctor(env, caller) && !record.is_confidential {
        return true;
    }
    if MedicalRecordsContract::has_emergency_access_internal(
        env,
        caller,
        &record.patient_id,
        record_id,
    ) {
        return true;
    }
    false
}

fn new_can_view_encrypted_record(
    env: &Env,
    caller: &Address,
    record: &EncryptedRecord,
    record_id: u64,
) -> bool {
    MedicalRecordsContract::can_view_encrypted_record(env, caller, record, record_id)
}

#[test]
fn bench_storage_manage_user_rbac_flow() {
    let env_before = Env::default();
    let (_client_before, admin_before, doctor_before, _patient_before) =
        setup_contract(&env_before);
    let before = measure_cpu(&env_before, || {
        old_manage_user_rbac_flow(
            &env_before,
            &admin_before,
            &doctor_before,
            Some(Role::Doctor),
            Role::Patient,
        )
        .unwrap();
    });

    let env_after = Env::default();
    let (_client_after, admin_after, doctor_after, _patient_after) = setup_contract(&env_after);
    let after = measure_cpu(&env_after, || {
        new_manage_user_rbac_flow(
            &env_after,
            &admin_after,
            &doctor_after,
            Some(Role::Doctor),
            Role::Patient,
        )
        .unwrap();
    });

    print_delta("medical_records::manage_user_rbac_flow", before, after);
    assert!(after <= before);
}

#[test]
fn bench_storage_history_gate() {
    let env_before = Env::default();
    let (_client_before, _admin_before, doctor_before, patient_before) =
        setup_contract(&env_before);
    let before = measure_cpu(&env_before, || {
        old_history_gate(&env_before, &doctor_before, &patient_before).unwrap();
    });

    let env_after = Env::default();
    let (_client_after, _admin_after, doctor_after, patient_after) = setup_contract(&env_after);
    let after = measure_cpu(&env_after, || {
        new_history_gate(&env_after, &doctor_after, &patient_after).unwrap();
    });

    print_delta("medical_records::history_gate", before, after);
    assert!(after <= before);
}

#[test]
fn bench_storage_encrypted_record_view_gate() {
    let env_before = Env::default();
    let (_client_before, admin_before, doctor_before, patient_before) = setup_contract(&env_before);
    let viewer_before = Address::generate(&env_before);
    new_manage_user_rbac_flow(
        &env_before,
        &admin_before,
        &viewer_before,
        None,
        Role::Doctor,
    )
    .unwrap();
    let record_before =
        sample_encrypted_record(&env_before, &patient_before, &doctor_before, false);
    let before = measure_cpu(&env_before, || {
        let allowed =
            old_can_view_encrypted_record(&env_before, &viewer_before, &record_before, 77);
        assert!(allowed);
    });

    let env_after = Env::default();
    let (_client_after, admin_after, doctor_after, patient_after) = setup_contract(&env_after);
    let viewer_after = Address::generate(&env_after);
    new_manage_user_rbac_flow(&env_after, &admin_after, &viewer_after, None, Role::Doctor).unwrap();
    let record_after = sample_encrypted_record(&env_after, &patient_after, &doctor_after, false);
    let after = measure_cpu(&env_after, || {
        let allowed = new_can_view_encrypted_record(&env_after, &viewer_after, &record_after, 77);
        assert!(allowed);
    });

    print_delta("medical_records::encrypted_record_view_gate", before, after);
    assert!(after <= before);
}

fn populate_records(
    env: &Env,
    client: &MedicalRecordsContractClient<'_>,
    doctor: &Address,
    patient: &Address,
    count: u64,
) {
    for i in 0..count {
        client.add_record(
            doctor,
            patient,
            &String::from_str(env, &format!("Diagnosis {}", i)),
            &String::from_str(env, &format!("Treatment {}", i)),
            &false,
            &Vec::new(env),
            &String::from_str(env, "General"),
            &String::from_str(env, "Medication"),
            &String::from_str(env, &format!("ipfs://record{}", i)),
        );
    }
}

#[test]
fn bench_write_record_1_record() { bench_write_record_with_count(1); }

#[test]
fn bench_write_record_100_records() { bench_write_record_with_count(100); }

#[test]
fn bench_write_record_1000_records() { bench_write_record_with_count(1000); }

fn bench_write_record_with_count(existing: u64) {
    let env = Env::default();
    let (client, _admin, doctor, patient) = setup_contract(&env);

    populate_records(&env, &client, &doctor, &patient, existing);

    let cost = measure_cpu(&env, || {
        client.add_record(
            &doctor,
            &patient,
            &String::from_str(&env, "Benchmark Diagnosis"),
            &String::from_str(&env, "Benchmark Treatment"),
            &false,
            &Vec::new(&env),
            &String::from_str(&env, "General"),
            &String::from_str(&env, "Medication"),
            &String::from_str(&env, "ipfs://benchmark-record"),
        );
    });

    std::println!(
        "[STORAGE-BENCH] write_record existing={} cpu_cost={}",
        existing,
        cost
    );
}
