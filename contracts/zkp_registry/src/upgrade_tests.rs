#[cfg(test)]
mod upgrade_tests {
    use soroban_sdk::{
        testutils::Address as _,
        Address, Bytes, Env,
    };
    use crate::{ZkpRegistryContract, ZkpRegistryContractClient};

    fn mock_verifier_key(env: &Env, seed: u8) -> Bytes {
        Bytes::from_slice(env, &[seed; 32])
    }

    fn mock_proof(env: &Env, seed: u8) -> Bytes {
        Bytes::from_slice(env, &[seed; 64])
    }

    fn setup(env: &Env) -> (ZkpRegistryContractClient, Address) {
        let contract_id = env.register_contract(None, ZkpRegistryContract);
        let client      = ZkpRegistryContractClient::new(env, &contract_id);
        let admin       = Address::generate(env);
        env.mock_all_auths();
        client.initialize(&admin, &mock_verifier_key(env, 0xAA));
        (client, admin)
    }

    // ── Test 1: Admin can update the verifier key ─────────────────────────────

    #[test]
    fn test_admin_can_update_verifier_key() {
        let env = Env::default();
        let (client, admin) = setup(&env);

        let new_key = mock_verifier_key(&env, 0xBB);
        env.mock_all_auths();
        client.update_verifier_key(&admin, &new_key);

        let stored = client.get_verifier_key();
        assert_eq!(stored, new_key, "Verifier key should be updated to the new key");
    }

    // ── Test 2: Old proofs are rejected after key rotation ────────────────────

    #[test]
    fn test_old_proof_rejected_after_key_rotation() {
        let env = Env::default();
        let (client, admin) = setup(&env);

        // A proof valid under the original key
        let old_proof = mock_proof(&env, 0x01);

        // Rotate the key
        let new_key = mock_verifier_key(&env, 0xBB);
        env.mock_all_auths();
        client.update_verifier_key(&admin, &new_key);

        // Attempt to verify the old proof — must now fail
        let result = client.try_verify_proof(&old_proof);
        assert!(
            result.is_err(),
            "Proofs generated under the old key must be rejected after key rotation"
        );
    }

    // ── Test 3: Unauthorized key update is rejected ───────────────────────────

    #[test]
    fn test_unauthorized_key_update_rejected() {
        let env = Env::default();
        let (client, _) = setup(&env);

        let attacker = Address::generate(&env);
        let new_key  = mock_verifier_key(&env, 0xCC);

        let result = std::panic::catch_unwind(|| {
            client.update_verifier_key(&attacker, &new_key);
        });
        assert!(
            result.is_err(),
            "Non-admin must not be able to update the verifier key"
        );
    }

    // ── Test 4: Key update emits an auditable event ───────────────────────────

    #[test]
    fn test_key_update_emits_event() {
        let env = Env::default();
        let (client, admin) = setup(&env);

        let new_key = mock_verifier_key(&env, 0xDD);
        env.mock_all_auths();
        client.update_verifier_key(&admin, &new_key);

        // Verify at least one event was emitted
        let events = env.events().all();
        assert!(
            !events.is_empty(),
            "Key rotation must emit a contract event for audit trail"
        );
    }

    // ── Test 5: New proof valid under new key ────────────────────────────────

    #[test]
    fn test_new_proof_valid_after_key_rotation() {
        let env = Env::default();
        let (client, admin) = setup(&env);

        let new_key   = mock_verifier_key(&env, 0xBB);
        let new_proof = mock_proof(&env, 0x02); // proof for new key

        env.mock_all_auths();
        client.update_verifier_key(&admin, &new_key);

        // Should succeed with the new key
        let result = client.try_verify_proof(&new_proof);
        assert!(
            result.is_ok(),
            "Valid proof under the new key should be accepted"
        );
    }
}