#![cfg(test)]
mod common;
use common::setup_uzima;
use medical_records::{Role, Error};
// FIX 1: Added `Ledger` to the testutils import below
use soroban_sdk::{testutils::{Address as _, Ledger}, Address, String, Vec}; 

// Helper function to create a record first
fn setup_record_for_access_test<'a>(t: &common::UzimaTest<'a>) -> u64 {
    t.client.manage_user(&t.admin1, &t.doctor, &Role::Doctor);
    t.client.manage_user(&t.admin1, &t.patient, &Role::Patient);
    
    let diagnosis = String::from_str(&t.env, "Test Diag");
    let treatment = String::from_str(&t.env, "Test Treat");
    let category = String::from_str(&t.env, "Modern");
    let data_ref = String::from_str(&t.env, "ipfs://recordcid00001");
    
    let mut tags = Vec::new(&t.env);
    tags.push_back(String::from_str(&t.env, "TestTag"));

    t.client.add_record(
        &t.doctor, 
        &t.patient, 
        &diagnosis, 
        &treatment, 
        &false, 
        &tags, 
        &category, 
        &String::from_str(&t.env, "TestType"),
        &data_ref
    )
}

#[test]
fn test_negative_rbac_and_pause() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    let intruder = Address::generate(&env);
    
    let _record_id = setup_record_for_access_test(&t);

    // 1. Test UNAUTHORIZED pause (Intruder != Admin)
    let pause_result = t.client.try_pause(&intruder);
    // FIX 2: Added .into() to handle type mismatch
    assert_eq!(pause_result, Err(Ok(Error::NotAuthorized.into())));

    // 2. Test UNAUTHORIZED user management (Doctor != Admin)
    t.client.manage_user(&t.admin1, &t.doctor, &Role::Doctor);
    let manage_result = t.client.try_manage_user(&t.doctor, &intruder, &Role::Doctor);
    // FIX 2: Added .into()
    assert_eq!(manage_result, Err(Ok(Error::NotAuthorized.into())));

    // 3. Test Unauthorized ACCESS to record (Non-patient/non-admin/non-doctor)
    let access_result = t.client.try_get_record(&intruder, &1);
    
    // This assertion is fine as is (doesn't use equality check)
    assert!(access_result.is_err() || access_result.unwrap().is_err());
}

#[test]
fn test_negative_recovery_timelock_and_approval() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    let safe_address = Address::generate(&env);
    let token_contract = Address::generate(&env);
    
    // 1. Propose recovery
    let proposal_id = t.client.propose_recovery(&t.admin1, &token_contract, &safe_address, &100);

    // 2. Test PREMATURE execution (Timelock not elapsed)
    let premature_result = t.client.try_execute_recovery(&t.admin1, &proposal_id);
    // FIX 2: Added .into()
    assert_eq!(premature_result, Err(Ok(Error::TimelockNotElasped.into())));

    // 3. Fast-forward time (Now works because Ledger is imported)
    env.ledger().set_timestamp(env.ledger().timestamp() + 86_400);

    // 4. Test NOT ENOUGH APPROVALS
    let insufficient_result = t.client.try_execute_recovery(&t.admin1, &proposal_id);
    // FIX 2: Added .into()
    assert_eq!(insufficient_result, Err(Ok(Error::NotEnoughApproval.into())));

    // 5. Approve with Admin2, then execute (This should pass)
    t.client.approve_recovery(&t.admin2, &proposal_id);
    assert_eq!(t.client.execute_recovery(&t.admin1, &proposal_id), true);
}