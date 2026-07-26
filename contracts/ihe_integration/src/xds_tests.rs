#[cfg(test)]
mod xds_tests {
    use soroban_sdk::{
        testutils::Address as _,
        Address, Bytes, Env, String as SorobanString,
    };
    use crate::{IheIntegrationContract, IheIntegrationContractClient};

    // IHE XDS document metadata helper
    fn mock_document_metadata(env: &Env) -> soroban_sdk::Map<SorobanString, SorobanString> {
        let mut meta = soroban_sdk::Map::new(env);
        meta.set(
            SorobanString::from_str(env, "patientId"),
            SorobanString::from_str(env, "PAT-001"),
        );
        meta.set(
            SorobanString::from_str(env, "documentClass"),
            SorobanString::from_str(env, "34133-9"), // LOINC code for Summary of episode note
        );
        meta.set(
            SorobanString::from_str(env, "mimeType"),
            SorobanString::from_str(env, "application/pdf"),
        );
        meta
    }

    fn mock_document_content(env: &Env) -> Bytes {
        Bytes::from_slice(env, b"mock_document_content_hash")
    }

    fn setup(env: &Env) -> (IheIntegrationContractClient, Address, Address) {
        let contract_id = env.register_contract(None, IheIntegrationContract);
        let client      = IheIntegrationContractClient::new(env, &contract_id);
        let admin       = Address::generate(env);
        let provider    = Address::generate(env);
        env.mock_all_auths();
        client.initialize(&admin);
        client.grant_provider(&admin, &provider);
        (client, admin, provider)
    }

    // ── ITI-42: Register Document Set ─────────────────────────────────────────
    // IHE ITI TF-2: Transaction ITI-42 — Document Registry registration

    #[test]
    fn test_iti_42_register_document_set() {
        let env = Env::default();
        let (client, _, provider) = setup(&env);

        let metadata = mock_document_metadata(&env);
        let content  = mock_document_content(&env);

        env.mock_all_auths();

        // ITI-42: ProvideAndRegisterDocumentSet
        let doc_id = client.register_document(&provider, &metadata, &content);

        assert!(
            !doc_id.is_empty(),
            "ITI-42: Document registration must return a valid document ID"
        );

        // Verify the document is now queryable in the registry
        let stored = client.get_document_metadata(&doc_id);
        assert!(stored.contains_key(SorobanString::from_str(&env, "patientId")),
            "ITI-42: Registered document metadata must be retrievable");
    }

    // ── ITI-43: Retrieve Document Set ─────────────────────────────────────────
    // IHE ITI TF-2: Transaction ITI-43 — Document Repository retrieval

    #[test]
    fn test_iti_43_retrieve_document_set() {
        let env = Env::default();
        let (client, _, provider) = setup(&env);

        let metadata = mock_document_metadata(&env);
        let content  = mock_document_content(&env);

        env.mock_all_auths();
        let doc_id = client.register_document(&provider, &metadata, &content);

        // ITI-43: RetrieveDocumentSet
        let retrieved = client.retrieve_document(&doc_id);

        assert_eq!(
            retrieved, content,
            "ITI-43: Retrieved document content must match the registered content"
        );
    }

    // ── Negative: Unauthorized registry access ─────────────────────────────────

    #[test]
    fn test_unauthorized_registry_access_rejected() {
        let env = Env::default();
        let (client, _, _) = setup(&env);

        let attacker = Address::generate(&env);
        let metadata = mock_document_metadata(&env);
        let content  = mock_document_content(&env);

        let result = std::panic::catch_unwind(|| {
            client.register_document(&attacker, &metadata, &content);
        });
        assert!(
            result.is_err(),
            "ITI-42: Unauthorized caller must not be able to register documents"
        );
    }

    // ── Negative: Retrieve non-existent document ──────────────────────────────

    #[test]
    fn test_retrieve_nonexistent_document_fails() {
        let env = Env::default();
        let (client, _, _) = setup(&env);

        let fake_id = SorobanString::from_str(&env, "nonexistent-doc-id");
        let result  = client.try_retrieve_document(&fake_id);

        assert!(
            result.is_err(),
            "ITI-43: Retrieving a non-existent document must return an error"
        );
    }
}