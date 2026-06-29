#[cfg(test)]
mod rate_limit_tests {
    use soroban_sdk::{
        testutils::Address as _,
        Address, Env,
    };
    use crate::{
        OracleContract, OracleContractClient,
        OracleError, MAX_REQUESTS_PER_LEDGER,
    };

    fn setup(env: &Env) -> (OracleContractClient, Address) {
        let contract_id = env.register_contract(None, OracleContract);
        let client      = OracleContractClient::new(env, &contract_id);
        let admin       = Address::generate(env);
        env.mock_all_auths();
        client.initialize(&admin);
        (client, admin)
    }

    // ── Test 1: Requests succeed up to quota ─────────────────────────────────

    #[test]
    fn test_requests_succeed_within_quota() {
        let env = Env::default();
        let (client, _) = setup(&env);
        let caller = Address::generate(&env);

        env.mock_all_auths();
        for _ in 0..MAX_REQUESTS_PER_LEDGER {
            // Should not panic for any of these
            let result = client.try_query_rate(&caller);
            assert!(result.is_ok(), "Requests within quota must succeed");
        }
    }

    // ── Test 2: Quota exhaustion returns typed error ──────────────────────────

    #[test]
    fn test_quota_exhaustion_returns_typed_error() {
        let env = Env::default();
        let (client, _) = setup(&env);
        let caller = Address::generate(&env);

        env.mock_all_auths();

        // Exhaust the quota
        for _ in 0..MAX_REQUESTS_PER_LEDGER {
            let _ = client.try_query_rate(&caller);
        }

        // The next request must return QuotaExceeded
        let result = client.try_query_rate(&caller);
        assert!(
            matches!(result, Err(Ok(OracleError::QuotaExceeded))),
            "Request after quota exhaustion must return OracleError::QuotaExceeded"
        );
    }

    // ── Test 3: Quota is per-caller, not global ───────────────────────────────

    #[test]
    fn test_quota_is_per_caller() {
        let env = Env::default();
        let (client, _) = setup(&env);

        let caller_a = Address::generate(&env);
        let caller_b = Address::generate(&env);

        env.mock_all_auths();

        // Exhaust caller_a's quota
        for _ in 0..MAX_REQUESTS_PER_LEDGER {
            let _ = client.try_query_rate(&caller_a);
        }

        // caller_b should still be allowed
        let result = client.try_query_rate(&caller_b);
        assert!(
            result.is_ok(),
            "Quota exhaustion for one caller must not affect other callers"
        );
    }

    // ── Test 4: Quota resets on new ledger ───────────────────────────────────

    #[test]
    fn test_quota_resets_on_new_ledger() {
        let env = Env::default();
        let (client, _) = setup(&env);
        let caller = Address::generate(&env);

        env.mock_all_auths();

        // Exhaust quota on current ledger
        for _ in 0..MAX_REQUESTS_PER_LEDGER {
            let _ = client.try_query_rate(&caller);
        }
        assert!(client.try_query_rate(&caller).is_err(), "Should be exhausted");

        // Advance to next ledger
        env.ledger().with_mut(|li| {
            li.sequence_number += 1;
        });

        // Quota must reset — request should now succeed
        let result = client.try_query_rate(&caller);
        assert!(
            result.is_ok(),
            "Quota must reset at the start of a new ledger"
        );
    }

    // ── Test 5: Different callers exhaust independently ───────────────────────

    #[test]
    fn test_multiple_callers_independent_quotas() {
        let env     = Env::default();
        let (client, _) = setup(&env);
        let callers: Vec<Address> = (0..5).map(|_| Address::generate(&env)).collect();

        env.mock_all_auths();

        for caller in &callers {
            for _ in 0..MAX_REQUESTS_PER_LEDGER {
                assert!(client.try_query_rate(caller).is_ok());
            }
            // Each caller's quota is now exhausted
            assert!(
                client.try_query_rate(caller).is_err(),
                "Each caller's quota must be tracked independently"
            );
        }
    }
}