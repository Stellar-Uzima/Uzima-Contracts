# Medical Records

Contract: `medical_records`

Encrypted on-chain medical record storage with role-based access control, patient consent management, and full audit trail.

<!-- API_START -->

## Key Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `env: Env, admin: Address, rbac_contract: Address` | `bool` | Initialize the contract, setting the admin and default storage values. |
| `health_check` | `env: Env` | `(Symbol, u32, u64)` | Return contract status, current version, and ledger timestamp. |
| `set_audit_forensics` | `env: Env, admin: Address, contract_id: Address` | `Result<bool, Error>` | Set the audit/forensics contract address; only callable by admin. |
| `get_audit_forensics` | `env: Env` | `Option<Address>` | Return the registered audit/forensics contract address, if set. |
| `manage_user` | `env: Env, caller: Address, user: Address, role: Role` | `Result<bool, Error>` | Register or update a user's role; only callable by admin. |
| `set_user_qkd_status` | `env: Env, admin: Address, user: Address, capable: bool` | `Result<(), Error>` | — |
| `is_user_qkd_capable` | `env: Env, user: Address` | `bool` | — |
| `deactivate_user` | `env: Env, caller: Address, user: Address` | `Result<bool, Error>` | — |
| `get_user_role` | `env: Env, user: Address` | `Result<Role, Error>` | — |
| `pause` | `env: Env, caller: Address` | `Result<bool, Error>` | — |
| `unpause` | `env: Env, caller: Address` | `Result<bool, Error>` | — |
| `grant_permission` | `env: Env, granter: Address, grantee: Address, permission: Permission, expiration: u64, // 0 = permanent is_delegatable: bool` | `Result<bool, Error>` | — |
| `revoke_permission` | `env: Env, revoker: Address, grantee: Address, permission: Permission` | `Result<bool, Error>` | — |
| `issue_access_attribute` | `env: Env, issuer: Address, user: Address, namespace: String, value: String, expires_at: u64, is_verified: bool` | `Result<bool, Error>` | — |
| `revoke_access_attribute` | `env: Env, revoker: Address, user: Address, namespace: String, value: String` | `Result<bool, Error>` | — |
| `get_user_access_attributes` | `env: Env, user: Address` | `Result<Vec<UserAccessAttribute>, Error>` | — |
| `get_access_attribute_epoch` | `env: Env, namespace: String, value: String` | `Result<u32, Error>` | — |
| `add_record` | `env: Env, caller: Address, patient: Address, diagnosis: String, treatment: String, is_confidential: bool, tags: Vec<String>, category: String, treatment_type: String, data_ref: String` | `Result<u64, Error>` | Store a new medical record; enforces RBAC and consent checks. |
| `write_record` | `env: Env, caller: Address, patient: Address, diagnosis: String, treatment: String, is_confidential: bool, tags: Vec<String>, category: String, treatment_type: String, data_ref: String, traditional_metadata: Option<TraditionalMedicineMetadata>` | `Result<u64, Error>` | Write a medical record with optional traditional medicine metadata.  This is the canonical entry-point for records that may involve traditional healing practices. When `traditional_metadata` is `Some`, the metadata is stored encrypted alongside the main record and the record ID is appended to the patient-scoped traditional-records index so it can be queried separately via `list_traditional_records`.  Calling with `traditional_metadata: None` is fully backward-compatible with the existing `add_record` behaviour. |
| `write_record_batch` | `env: Env, caller: Address, records: Vec<RecordInput>` | `Result<Vec<u64>, Error>` | Create multiple medical records in a single atomic call.  All-or-nothing semantics: if any record fails validation, the entire batch is rejected and no records are persisted.  ## Limits - Max 50 records per batch. |
| `list_traditional_records` | `env: Env, caller: Address, patient_id: Address` | `Result<Vec<u64>, Error>` | Return the record IDs of all traditional-medicine records for a patient.  Only the patient themselves, an admin, or a caller with `ReadRecord` permission may invoke this function. |
| `add_record_with_did` | `env: Env, caller: Address, patient: Address, diagnosis: String, treatment: String, is_confidential: bool, tags: Vec<String>, category: String, treatment_type: String, data_ref: String, _credential_ref: Option<String>` | `Result<u64, Error>` | — |
| `get_record` | `env: Env, caller: Address, record_id: u64` | `Result<MedicalRecord, Error>` | Retrieve a medical record by ID; enforces caller authorization and access control. |
| `get_record_with_did` | `env: Env, caller: Address, record_id: u64, purpose: String` | `Result<Option<MedicalRecord>, Error>` | — |
| `get_record_metadata` | `env: Env, record_id: u64` | `Result<RecordMetadata, Error>` | — |
| `get_history` | `env: Env, caller: Address, patient: Address, page: u32, page_size: u32` | `Result<Vec<(u64, RecordMetadata)>, Error>` | — |
| `get_record_count` | `env: Env` | `u64` | Return the total number of records stored in the contract. |
| `get_patient_record_count` | `env: Env, patient: Address` | `u64` | — |
| `get_patient_record_id` | `env: Env, patient: Address, index: u64` | `Option<u64>` | — |
| `list_records` | `env: Env, caller: Address, cursor: Option<u64>, limit: u32` | `Result<ListRecordsResult, Error>` | List medical records using cursor-based pagination. Returns up to `limit` records starting after the given cursor. `cursor` is the last record_id from a previous page (None for first page). `limit` must be between 1 and 100. |
| `set_zk_verifier_contract` | `env: Env, caller: Address, verifier: Address` | `Result<bool, Error>` | — |
| `get_zk_verifier_contract` | `env: Env` | `Option<Address>` | — |
| `set_credential_registry_contract` | `env: Env, caller: Address, registry: Address` | `Result<bool, Error>` | — |
| `get_credential_registry_contract` | `env: Env` | `Option<Address>` | — |
| `set_patient_consent_contract` | `env: Env, caller: Address, consent_contract: Address` | `Result<bool, Error>` | — |
| `get_patient_consent_contract` | `env: Env` | `Option<Address>` | — |
| `set_zk_enforced` | `env: Env, caller: Address, enforced: bool` | `Result<bool, Error>` | — |
| `is_zk_enforced` | `env: Env` | `bool` | — |
| `set_zk_grant_ttl` | `env: Env, caller: Address, ttl_secs: u64` | `Result<bool, Error>` | — |
| `get_zk_grant_ttl` | `env: Env` | `u64` | — |
| `get_record_commitment` | `env: Env, record_id: u64` | `Option<BytesN<32>>` | — |
| `has_valid_zk_access_grant` | `env: Env, requester: Address, record_id: u64` | `bool` | — |
| `submit_zk_access_proof` | `env: Env, caller: Address, record_id: u64, purpose: String, public_inputs: ZkPublicInputs, proof: Bytes` | `Result<bool, Error>` | — |
| `update_record_metadata` | `env: Env, caller: Address, record_id: u64, tags: Vec<String>, custom_fields: Map<String, String>` | `Result<(), Error>` | Updates tags and custom metadata fields for an existing record. Only the record's doctor or an admin may call this. Each update creates a versioned history entry. |
| `search_records_by_tag` | `env: Env, caller: Address, tag: String, page: u32, page_size: u32` | `Result<Vec<u64>, Error>` | Returns record IDs that are indexed under a given tag, paginated. Any authenticated user may search. |
| `export_record_metadata` | `env: Env, caller: Address, record_id: u64` | `Result<RecordMetadata, Error>` | Exports full metadata (including history) for a record. Accessible by the patient, the record's doctor, or an admin. |
| `import_record_metadata` | `env: Env, caller: Address, record_id: u64, tags: Vec<String>, custom_fields: Map<String, String>` | `Result<(), Error>` | Admin-only: imports (overwrites) tags and custom fields for a record. Useful for data migration. Creates a history entry before overwriting. |
| `set_crypto_registry` | `env: Env, caller: Address, registry: Address` | `Result<bool, Error>` | — |
| `get_crypto_registry` | `env: Env` | `Option<Address>` | — |
| `set_homomorphic_registry` | `env: Env, caller: Address, registry: Address` | `Result<bool, Error>` | — |
| `get_homomorphic_registry` | `env: Env` | `Option<Address>` | — |
| `set_mpc_manager` | `env: Env, caller: Address, manager: Address` | `Result<bool, Error>` | — |
| `get_mpc_manager` | `env: Env` | `Option<Address>` | — |
| `set_encryption_required` | `env: Env, caller: Address, required: bool` | `Result<bool, Error>` | — |
| `is_encryption_required` | `env: Env` | `bool` | — |
| `set_regulatory_compliance` | `env: Env, caller: Address, compliance: Address` | `Result<bool, Error>` | — |
| `get_regulatory_compliance` | `env: &Env` | `Option<Address>` | — |
| `set_require_pq_envelopes` | `env: Env, caller: Address, required: bool` | `Result<bool, Error>` | — |
| `is_require_pq_envelopes` | `env: Env` | `bool` | — |
| `propose_crypto_config_update` | `env: Env, caller: Address, new_crypto_registry: Option<Address>, new_homomorphic_registry: Option<Address>, new_mpc_manager: Option<Address>, encryption_required: Option<bool>, require_pq_envelopes: Option<bool>` | `Result<u64, Error>` | — |
| `approve_crypto_config_update` | `env: Env, caller: Address, proposal_id: u64` | `Result<bool, Error>` | — |
| `execute_crypto_config_update` | `env: Env, caller: Address, proposal_id: u64` | `Result<bool, Error>` | — |
| `get_crypto_config_proposal` | `env: Env, caller: Address, proposal_id: u64` | `Result<Option<CryptoConfigProposal>, Error>` | — |
| `set_quantum_threat_level` | `env: Env, admin: Address, level: u32` | `Result<(), Error>` | — |
| `get_quantum_threat_level` | `env: Env` | `u32` | — |
| `upgrade_record_to_quantum_safe` | `env: Env, caller: Address, record_id: u64, new_envelope: KeyEnvelope` | `Result<(), Error>` | Migrates a record to include a new quantum-safe envelope. Accessible by the patient or an authorized doctor. |
| `add_advanced_encrypted_record` | `env: Env, caller: Address, patient: Address, is_confidential: bool, tags: Vec<String>, category: String, treatment_type: String, advanced: AdvancedEncryptedRecordInput` | `Result<u64, Error>` | — |
| `add_encrypted_record` | `env: Env, caller: Address, patient: Address, is_confidential: bool, tags: Vec<String>, category: String, treatment_type: String, ciphertext_ref: String, ciphertext_hash: BytesN<32>, envelopes: Vec<KeyEnvelope>` | `Result<u64, Error>` | — |
| `bind_encrypted_record_abe_policy` | `env: Env, caller: Address, record_id: u64, policy_ref: String, policy_hash: BytesN<32>, access_ciphertext_ref: String, access_ciphertext_hash: BytesN<32>, required_permission: Permission, attribute_count: u32, valid_until: u64, revocation_epoch: u32` | `Result<bool, Error>` | — |
| `get_encrypted_record_header` | `env: Env, caller: Address, record_id: u64` | `Result<Option<EncryptedRecordHeader>, Error>` | — |
| `get_encrypted_record_envelope` | `env: Env, caller: Address, record_id: u64` | `Result<Option<KeyEnvelope>, Error>` | — |
| `upsert_encrypted_record_envelope` | `env: Env, caller: Address, record_id: u64, envelope: KeyEnvelope` | `Result<bool, Error>` | — |
| `get_encrypted_record_abe_policy` | `env: Env, caller: Address, record_id: u64` | `Result<Option<AbePolicyMetadata>, Error>` | — |
| `get_crypto_audit_logs` | `env: Env, caller: Address, page: u32, page_size: u32` | `Result<Vec<CryptoAuditEntry>, Error>` | — |
| `set_identity_registry` | `env: Env, caller: Address, registry: Address` | `Result<bool, Error>` | — |
| `get_identity_registry` | `env: Env` | `Option<Address>` | — |
| `set_did_auth_level` | `env: Env, caller: Address, level: DIDAuthLevel` | `Result<bool, Error>` | — |
| `get_did_auth_level` | `env: Env` | `DIDAuthLevel` | — |
| `link_did_to_user` | `env: Env, caller: Address, user: Address, did: String` | `Result<bool, Error>` | — |
| `get_user_did` | `env: Env, user: Address` | `Option<String>` | — |
| `verify_professional_credential` | `env: Env, user: Address` | `bool` | Minimal on-chain verifier used by tests: returns true iff the user is an active Doctor. |
| `set_ai_config` | `env: Env, caller: Address, ai_coordinator: Address, dp_epsilon: u32, min_participants: u32` | `Result<bool, Error>` | — |
| `get_ai_config` | `env: Env` | `Option<AIConfig>` | — |
| `submit_anomaly_score` | `env: Env, caller: Address, record_id: u64, model_id: BytesN<32>, score_bps: u32, explanation_ref: String, explanation_summary: String, model_version: String, _feature_importance: Vec<(String, u32` | `()` | — |
| `get_anomaly_score` | `env: Env, caller: Address, record_id: u64` | `Result<Option<AIInsight>, Error>` | — |
| `submit_risk_score` | `env: Env, caller: Address, patient: Address, model_id: BytesN<32>, score_bps: u32, explanation_ref: String, explanation_summary: String, model_version: String, _feature_importance: Vec<(String, u32` | `()` | — |
| `get_latest_risk_score` | `env: Env, caller: Address, patient: Address` | `Result<Option<AIInsight>, Error>` | — |
| `grant_emergency_access` | `env: Env, caller: Address, grantee: Address, duration_secs: u64, record_scope: Vec<u64>` | `Result<bool, Error>` | — |
| `has_emergency_access` | `env: Env, grantee: Address, patient: Address, record_id: u64` | `bool` | — |
| `revoke_emergency_access` | `env: Env, caller: Address, grantee: Address` | `Result<bool, Error>` | — |
| `get_patient_emergency_grants` | `env: Env, patient: Address` | `Vec<EmergencyAccess>` | — |
| `get_patient_access_logs` | `env: Env, caller: Address, patient: Address, page: u32, page_size: u32` | `Vec<AccessRequest>` | — |
| `get_access_logs` | `env: Env, page: u32, page_size: u32` | `Vec<AccessRequest>` | — |
| `propose_recovery` | `env: Env, caller: Address, token_contract: Address, to: Address, amount: i128` | `Result<u64, Error>` | — |
| `approve_recovery` | `env: Env, caller: Address, proposal_id: u64` | `Result<bool, Error>` | — |
| `execute_recovery` | `env: Env, caller: Address, proposal_id: u64` | `Result<bool, Error>` | — |
| `set_cross_chain_contracts` | `env: Env, caller: Address, bridge: Address, identity: Address, access: Address` | `Result<bool, Error>` | — |
| `set_cross_chain_enabled` | `env: Env, caller: Address, enabled: bool` | `Result<bool, Error>` | — |
| `is_cross_chain_enabled` | `env: Env` | `bool` | — |
| `register_cross_chain_ref` | `env: Env, caller: Address, record_id: u64, chain: ChainId, external_record_hash: BytesN<32>` | `Result<bool, Error>` | — |
| `update_cross_chain_sync` | `env: Env, caller: Address, record_id: u64, chain: ChainId, new_external_hash: BytesN<32>` | `Result<bool, Error>` | — |
| `get_cross_chain_ref` | `env: Env, record_id: u64, chain: ChainId` | `Option<CrossChainRecordRef>` | — |
| `get_all_cross_chain_refs` | `env: Env, record_id: u64` | `Vec<CrossChainRecordRef>` | — |
| `get_record_cross_chain` | `env: Env, caller: Address, record_id: u64, _chain: ChainId, _access_token: String` | `Result<Option<MedicalRecord>, Error>` | — |
| `upgrade` | `env: Env, caller: Address, new_wasm_hash: BytesN<32>, new_version: u32` | `Result<(), Error>` | — |
| `validate_upgrade` | `env: Env, new_wasm_hash: BytesN<32>` | `Result<upgradeability::UpgradeValidation, Error>` | — |
| `version` | `env: Env` | `u32` | — |
| `export_patient_data` | `env: Env, patient_id: Address, format: ExportFormat` | `Result<Bytes, Error>` | Export all patient data in the requested format for data portability. Only the patient themselves can request their export. Rate-limited to one export per 24 hours per patient. |
| `set_rate_limit_config` | `env: Env, admin: Address, op: u32, config: RateLimitConfig` | `Result<bool, Error>` | Configure the rate limit for a specific operation (admin only). |
| `set_rate_limit_bypass` | `env: Env, admin: Address, account: Address, bypass: bool` | `Result<bool, Error>` | Grant or revoke rate-limit bypass for an account (admin only). |
| `validate_record_quality` | `env: Env, caller: Address, record_id: u64` | `Result<ValidationReport, Error>` | Validates a stored medical record and returns a comprehensive quality report.  Performs completeness checks, format validation, consistency verification, and FHIR compliance assessment. Emits a `DataQualityValidated` event. |
| `get_field_completeness` | `env: Env, caller: Address, record_id: u64` | `Result<FieldCompleteness, Error>` | Returns field-level completeness / gap detection for a stored record. |
| `validate_record_type` | `env: Env, caller: Address, record_id: u64, record_type: MedicalRecordType` | `Result<bool, Error>` | Validates a stored record against type-specific rules. |
| `get_correction_workflow` | `env: Env, caller: Address, record_id: u64` | `Result<CorrectionWorkflow, Error>` | Returns a prioritised `CorrectionWorkflow` for a stored medical record.  The workflow maps every validation issue into an actionable `CorrectionItem` (with severity-based priority and suggested fix), counts issues by category, and sets `can_auto_fix` when only minor, non-blocking issues remain.  Callers with `ReadRecord` permission may invoke this function to build a step-by-step remediation plan without modifying the stored record. |
| `cleanse_record_data` | `env: Env, caller: Address, record_id: u64` | `Result<CleanseResult, Error>` | Auto-cleanses a stored medical record using deterministic normalization rules.  Applies safe, non-clinical transformations: - Normalises category casing to the canonical allowed value. - Removes empty `doctor_did` strings (replaces `Some("")` with `None`).  If any changes were made, the updated record is persisted and a `DataQualityValidated` event is emitted with the post-cleanse quality score. Returns a `CleanseResult` describing what (if anything) changed.  Requires `UpdateRecord` permission. |
| `initialize` | `env: Env, admin: Address, config: soroban_sdk::Val` | `()` | — |
| `has_role` | `env: Env, address: Address, role: RbacRole` | `Result<bool, RbacError>` | — |
| `assign_role` | `env: Env, address: Address, role: RbacRole` | `Result<bool, RbacError>` | — |
| `remove_role` | `env: Env, address: Address, role: RbacRole` | `Result<bool, RbacError>` | — |
| `add_record_with_traditional` | `env: Env, caller: Address, patient: Address, diagnosis: String, treatment: String, is_confidential: bool, tags: Vec<String>, category: String, treatment_type: String, data_ref: String, traditional_metadata: Option<TraditionalMedicineMetadata>` | `Result<u64, Error>` | Store a medical record with optional traditional medicine metadata. When `traditional_metadata` is provided, the record is also indexed for separate querying via `list_traditional_records`. |
| `list_traditional_records` | `env: Env, caller: Address, patient: Address` | `Result<Vec<u64>, Error>` | List traditional medicine records for a patient. Returns record IDs that have associated traditional medicine metadata. |

## Types

### `enum ChainId`

| Variant | Value | Description |
|---|---|---|
| `Stellar` | — | — |
| `Ethereum` | — | — |
| `Polygon` | — | — |
| `Avalanche` | — | — |
| `BinanceSmartChain` | — | — |
| `Arbitrum` | — | — |
| `Optimism` | — | — |
| `Custom(u32)` | — | — |

### `struct CrossChainRecordRef`

| Field | Type | Description |
|---|---|---|
| `local_record_id` | `u64` | — |
| `external_chain` | `ChainId` | — |
| `external_record_hash` | `BytesN<32>` | — |
| `sync_timestamp` | `u64` | — |
| `is_synced` | `bool` | — |

### `struct RecordMetadata`

| Field | Type | Description |
|---|---|---|
| `record_id` | `u64` | — |
| `patient_id` | `Address` | — |
| `timestamp` | `u64` | — |
| `category` | `String` | — |
| `is_confidential` | `bool` | — |
| `record_hash` | `BytesN<32>` | — |
| `tags` | `Vec<String>` | — |
| `custom_fields` | `Map<String, String>` | — |
| `version` | `u32` | — |
| `history` | `Vec<RecordMetadataHistoryEntry>` | — |

### `struct RecordMetadataHistoryEntry`

| Field | Type | Description |
|---|---|---|
| `version` | `u32` | — |
| `timestamp` | `u64` | — |
| `tags` | `Vec<String>` | — |
| `custom_fields` | `Map<String, String>` | — |

### `enum Role`

| Variant | Value | Description |
|---|---|---|
| `Admin` | — | — |
| `Doctor` | — | — |
| `Patient` | — | — |
| `None` | — | — |

### `enum RbacRole`

| Variant | Value | Description |
|---|---|---|
| `Admin` | 0 | — |
| `Doctor` | 1 | — |
| `Patient` | 2 | — |
| `Staff` | 3 | — |
| `Insurer` | 4 | — |
| `Researcher` | 5 | — |
| `Auditor` | 6 | — |
| `Service` | 7 | — |

### `enum RbacError`

| Variant | Value | Description |
|---|---|---|
| `Unauthorized` | 100 | — |
| `NotInitialized` | 300 | — |
| `AlreadyInitialized` | 301 | — |

### `enum Permission`

| Variant | Value | Description |
|---|---|---|
| `ManageUsers` | 1 | — |
| `ManageSystem` | 2 | — |
| `CreateRecord` | 10 | — |
| `ReadRecord` | 11 | — |
| `UpdateRecord` | 12 | — |
| `DeleteRecord` | 13 | — |
| `ReadConfidential` | 20 | — |
| `DelegatePermission` | 30 | — |

### `struct PermissionGrant`

| Field | Type | Description |
|---|---|---|
| `permission` | `Permission` | — |
| `granter` | `Address` | — |
| `expires_at` | `u64` | — |
| `is_delegatable` | `bool` | — |

### `struct UserProfile`

| Field | Type | Description |
|---|---|---|
| `role` | `Role` | — |
| `active` | `bool` | — |
| `did_reference` | `Option<String>` | — |
| `qkd_capable` | `bool` | — |

### `enum DIDAuthLevel`

| Variant | Value | Description |
|---|---|---|
| `None` | — | — |
| `Basic` | — | — |
| `CredentialRequired` | — | — |
| `Full` | — | — |

### `struct AccessRequest`

| Field | Type | Description |
|---|---|---|
| `requester` | `Address` | — |
| `patient` | `Address` | — |
| `record_id` | `u64` | — |
| `purpose` | `String` | — |
| `timestamp` | `u64` | — |
| `granted` | `bool` | — |

### `struct EmergencyAccess`

| Field | Type | Description |
|---|---|---|
| `grantee` | `Address` | — |
| `patient` | `Address` | — |
| `expires_at` | `u64` | — |
| `record_scope` | `Vec<u64>` | — |
| `is_active` | `bool` | — |

### `struct ZkPublicInputs`

| Field | Type | Description |
|---|---|---|
| `record_id` | `u64` | — |
| `record_commitment` | `BytesN<32>` | — |
| `credential_root` | `BytesN<32>` | — |
| `issuer` | `Address` | — |
| `requester_commitment` | `BytesN<32>` | — |
| `provider_commitment` | `BytesN<32>` | — |
| `claim_commitment` | `BytesN<32>` | — |
| `min_timestamp` | `u64` | — |
| `max_timestamp` | `u64` | — |
| `nullifier` | `BytesN<32>` | — |
| `pseudonym` | `BytesN<32>` | — |
| `vk_version` | `u32` | — |

### `struct ZkAccessGrant`

| Field | Type | Description |
|---|---|---|
| `record_id` | `u64` | — |
| `requester` | `Address` | — |
| `expires_at` | `u64` | — |
| `nullifier` | `BytesN<32>` | — |
| `pseudonym` | `BytesN<32>` | — |
| `vk_version` | `u32` | — |

### `struct ZkAuditRecord`

| Field | Type | Description |
|---|---|---|
| `record_id` | `u64` | — |
| `pseudonym` | `BytesN<32>` | — |
| `timestamp` | `u64` | — |
| `proof_verified` | `bool` | — |
| `nullifier_present` | `bool` | — |
| `nullifier` | `BytesN<32>` | — |

### `struct MedicalRecord`

| Field | Type | Description |
|---|---|---|
| `patient_id` | `Address` | — |
| `doctor_id` | `Address` | — |
| `timestamp` | `u64` | — |
| `diagnosis` | `String` | — |
| `treatment` | `String` | — |
| `is_confidential` | `bool` | — |
| `tags` | `Vec<String>` | — |
| `category` | `String` | — |
| `treatment_type` | `String` | — |
| `data_ref` | `String` | — |
| `doctor_did` | `Option<String>` | — |

### `struct TraditionalMedicineMetadata`

| Field | Type | Description |
|---|---|---|
| `practice_type` | `String` | — |
| `practitioner_tradition` | `String` | — |
| `remedies_used` | `String` | — |
| `cultural_context` | `String` | — |
| `language` | `String` | — |

### `enum AIInsightType`

| Variant | Value | Description |
|---|---|---|
| `AnomalyScore` | — | — |
| `RiskScore` | — | — |

### `struct AIInsight`

| Field | Type | Description |
|---|---|---|
| `patient` | `Address` | — |
| `record_id` | `u64` | — |
| `model_id` | `BytesN<32>` | — |
| `insight_type` | `AIInsightType` | — |
| `score_bps` | `u32` | — |
| `explanation_ref` | `String` | — |
| `explanation_summary` | `String` | — |
| `created_at` | `u64` | — |
| `model_version` | `String` | — |

### `struct AIConfig`

| Field | Type | Description |
|---|---|---|
| `ai_coordinator` | `Address` | — |
| `dp_epsilon` | `u32` | — |
| `min_participants` | `u32` | — |

### `struct RecoveryProposal`

| Field | Type | Description |
|---|---|---|
| `proposal_id` | `u64` | — |
| `token_contract` | `Address` | — |
| `to` | `Address` | — |
| `amount` | `i128` | — |
| `created_at` | `u64` | — |
| `executed` | `bool` | — |
| `approvals` | `Vec<Address>` | — |

### `enum EnvelopeAlgorithm`

| Variant | Value | Description |
|---|---|---|
| `X25519` | — | — |
| `Kyber768` | — | — |
| `Kyber1024` | — | — |
| `HybridX25519Kyber768` | — | — |
| `HybridX25519Kyber1024` | — | — |
| `HybridKyberMcEliece` | — | — |
| `McEliece` | — | — |
| `Custom(u32)` | — | — |

### `struct KeyEnvelope`

| Field | Type | Description |
|---|---|---|
| `recipient` | `Address` | — |
| `key_version` | `u32` | — |
| `algorithm` | `EnvelopeAlgorithm` | — |
| `wrapped_key` | `Bytes` | — |
| `pq_wrapped_key` | `Option<Bytes>` | — |

### `struct EncryptedRecord`

| Field | Type | Description |
|---|---|---|
| `patient_id` | `Address` | — |
| `doctor_id` | `Address` | — |
| `timestamp` | `u64` | — |
| `is_confidential` | `bool` | — |
| `tags` | `Vec<String>` | — |
| `category` | `String` | — |
| `treatment_type` | `String` | — |
| `ciphertext_ref` | `String` | — |
| `ciphertext_hash` | `BytesN<32>` | — |
| `envelopes` | `Vec<KeyEnvelope>` | — |
| `doctor_did` | `Option<String>` | — |

### `struct EncryptedRecordHeader`

| Field | Type | Description |
|---|---|---|
| `record_id` | `u64` | — |
| `patient_id` | `Address` | — |
| `doctor_id` | `Address` | — |
| `timestamp` | `u64` | — |
| `is_confidential` | `bool` | — |
| `tags` | `Vec<String>` | — |
| `category` | `String` | — |
| `treatment_type` | `String` | — |
| `ciphertext_ref` | `String` | — |
| `ciphertext_hash` | `BytesN<32>` | — |
| `doctor_did` | `Option<String>` | — |

### `struct UserAccessAttribute`

| Field | Type | Description |
|---|---|---|
| `namespace` | `String` | — |
| `value` | `String` | — |
| `issued_by` | `Address` | — |
| `issued_at` | `u64` | — |
| `expires_at` | `u64` | — |
| `revoked_at` | `u64` | — |
| `epoch` | `u32` | — |
| `is_active` | `bool` | — |
| `is_verified` | `bool` | — |

### `struct AbePolicyMetadata`

| Field | Type | Description |
|---|---|---|
| `policy_ref` | `String` | — |
| `policy_hash` | `BytesN<32>` | — |
| `access_ciphertext_ref` | `String` | — |
| `access_ciphertext_hash` | `BytesN<32>` | — |
| `required_permission` | `Permission` | — |
| `attribute_count` | `u32` | — |
| `compiled_at` | `u64` | — |
| `valid_until` | `u64` | — |
| `revocation_epoch` | `u32` | — |

### `struct AdvancedAccessState`

| Field | Type | Description |
|---|---|---|
| `record_policies` | `Map<u64, AbePolicyMetadata>` | — |
| `user_attributes` | `Map<Address, Vec<UserAccessAttribute>>` | — |
| `attribute_epochs` | `Map<BytesN<32>, u32>` | — |

### `struct AdvancedEncryptedRecordInput`

| Field | Type | Description |
|---|---|---|
| `ciphertext_ref` | `String` | — |
| `ciphertext_hash` | `BytesN<32>` | — |
| `envelopes` | `Vec<KeyEnvelope>` | — |
| `policy_ref` | `String` | — |
| `policy_hash` | `BytesN<32>` | — |
| `access_ciphertext_ref` | `String` | — |
| `access_ciphertext_hash` | `BytesN<32>` | — |
| `required_permission` | `Permission` | — |
| `attribute_count` | `u32` | — |
| `valid_until` | `u64` | — |
| `revocation_epoch` | `u32` | — |

### `enum CryptoAuditAction`

| Variant | Value | Description |
|---|---|---|
| `CryptoRegistrySet` | — | — |
| `HomomorphicRegistrySet` | — | — |
| `MpcManagerSet` | — | — |
| `EncryptionRequiredSet` | — | — |
| `EncryptedRecordCreated` | — | — |
| `EnvelopeUpdated` | — | — |
| `RequirePqEnvelopesSet` | — | — |
| `CryptoConfigProposed` | — | — |
| `CryptoConfigApproved` | — | — |
| `CryptoConfigExecuted` | — | — |
| `QuantumThreatDetected` | — | — |
| `QuantumMigrationStarted` | — | — |
| `QuantumMigrationCompleted` | — | — |

### `struct CryptoAuditEntry`

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | — |
| `timestamp` | `u64` | — |
| `actor` | `Address` | — |
| `action` | `CryptoAuditAction` | — |
| `record_id` | `Option<u64>` | — |
| `details_hash` | `BytesN<32>` | — |
| `details_ref` | `Option<String>` | — |

### `struct CryptoConfigProposal`

| Field | Type | Description |
|---|---|---|
| `proposal_id` | `u64` | — |
| `created_at` | `u64` | — |
| `executed` | `bool` | — |
| `approvals` | `Vec<Address>` | — |
| `new_crypto_registry` | `Option<Address>` | — |
| `new_homomorphic_registry` | `Option<Address>` | — |
| `new_mpc_manager` | `Option<Address>` | — |
| `encryption_required` | `Option<bool>` | — |
| `require_pq_envelopes` | `Option<bool>` | — |

### `enum DataKey`

| Variant | Value | Description |
|---|---|---|
| `Initialized` | — | — |
| `Paused` | — | — |
| `ContractVersion` | — | — |
| `RbacContract` | — | — |
| `Users` | — | — |
| `IdentityRegistry` | — | — |
| `DidAuthLevel` | — | — |
| `UserPermissions(Address)` | — | — |
| `NextId` | — | — |
| `RecordCount` | — | — |
| `Record(u64)` | — | — |
| `RecordMeta(u64)` | — | — |
| `RecordCommitment(u64)` | — | — |
| `PatientRecords(Address)` | — | — |
| `PatientRecordCount(Address)` | — | — |
| `PatientRecord(Address, u64)` | — | — |
| `TagIndex(String)` | — | — |
| `AccessLogCount,
    AccessLog(u64),
    PatientAccessLogCount(Address),
    PatientAccessLog(Address, u64),

    
    PatientEmergencyGrants(Address),

    
    AIConfig,
    PatientRisk(Address),
    RecordAnomaly(u64),

    
    Proposal(u64),
    CryptoConfigProposal(u64),

    
    BridgeContract,
    CrossChainIdentityContract,
    CrossChainAccessContract,
    CrossChainEnabled,
    CrossChainRef(u64, ChainId),

    
    CryptoRegistry,
    HomomorphicRegistry,
    MpcManager,
    EncryptionRequired,
    RequirePqEnvelopes,

    
    EncryptedRecord(u64),
    PatientEncryptedRecords(Address),

    
    CryptoAuditCount,
    CryptoAudit(u64),

    
    AuditForensicsContract,
    
    RegulatoryCompliance,

    
    ZkVerifierContract,
    CredentialRegistryContract,
    PatientConsentContract,
    ZkEnforced,
    ZkGrantTtl,
    ZkUsedNullifier(BytesN<32>),
    ZkAccessGrant(Address, u64),
    
    RateLimitCfg(u32),        
    RateLimit(Address, u32),  
    RateLimitBypass(Address), 
    QuantumThreatLevel,       
    LastExportTime(Address),  

    
    
    TraditionalMeta(u64),
    
    PatientTraditionalRecords(Address),` | — | — |

### `struct RecordInput`

| Field | Type | Description |
|---|---|---|
| `patient` | `Address` | — |
| `diagnosis` | `String` | — |
| `treatment` | `String` | — |
| `is_confidential` | `bool` | — |
| `tags` | `Vec<String>` | — |
| `category` | `String` | — |
| `treatment_type` | `String` | — |
| `data_ref` | `String` | — |
| `traditional_metadata` | `Option<TraditionalMedicineMetadata>` | — |

### `struct FailureInfo`

| Field | Type | Description |
|---|---|---|
| `index` | `u32` | — |
| `error_code` | `u32` | — |

### `struct BatchResult`

| Field | Type | Description |
|---|---|---|
| `successes` | `Vec<u64>` | — |
| `failures` | `Vec<FailureInfo>` | — |

### `struct ListRecordsResult`

| Field | Type | Description |
|---|---|---|
| `records` | `Vec<MedicalRecord>` | — |
| `next_cursor` | `Option<u64>` | — |

### `struct RateLimitConfig`

| Field | Type | Description |
|---|---|---|
| `doctor_max_calls` | `u32` | — |
| `patient_max_calls` | `u32` | — |
| `admin_max_calls` | `u32` | — |
| `window_secs` | `u64` | — |

### `struct RateLimitEntry`

| Field | Type | Description |
|---|---|---|
| `count` | `u32` | — |
| `window_start` | `u64` | — |

### `enum MedicalRecordType`

| Variant | Value | Description |
|---|---|---|
| `General` | — | — |
| `Laboratory` | — | — |
| `Prescription` | — | — |
| `Imaging` | — | — |
| `Surgical` | — | — |
| `Emergency` | — | — |

### `struct DataQualityScore`

| Field | Type | Description |
|---|---|---|
| `overall_score` | `u32` | — |
| `completeness_score` | `u32` | — |
| `format_score` | `u32` | — |
| `consistency_score` | `u32` | — |
| `fhir_compliance_score` | `u32` | — |
| `issue_count` | `u32` | — |

### `enum ValidationSeverity`

| Variant | Value | Description |
|---|---|---|
| `Info` | — | — |
| `Warning` | — | — |
| `ValidationErr` | — | — |
| `Critical` | — | — |

### `struct ValidationIssue`

| Field | Type | Description |
|---|---|---|
| `severity` | `ValidationSeverity` | — |
| `field_name` | `String` | — |
| `issue_description` | `String` | — |
| `suggestion` | `String` | — |

### `struct ValidationReport`

| Field | Type | Description |
|---|---|---|
| `record_id` | `u64` | — |
| `quality_score` | `DataQualityScore` | — |
| `issues` | `Vec<ValidationIssue>` | — |
| `is_fhir_compliant` | `bool` | — |
| `validated_at` | `u64` | — |

### `struct FieldCompleteness`

| Field | Type | Description |
|---|---|---|
| `has_diagnosis` | `bool` | — |
| `has_treatment` | `bool` | — |
| `has_category` | `bool` | — |
| `has_treatment_type` | `bool` | — |
| `has_data_ref` | `bool` | — |
| `has_tags` | `bool` | — |
| `has_doctor_did` | `bool` | — |
| `total_fields` | `u32` | — |
| `completed_fields` | `u32` | — |

### `enum CorrectionPriority`

| Variant | Value | Description |
|---|---|---|
| `Critical` | — | — |
| `High` | — | — |
| `Medium` | — | — |
| `Low` | — | — |

### `enum CorrectionAction`

| Variant | Value | Description |
|---|---|---|
| `AddMissingField` | — | — |
| `FixFormat` | — | — |
| `NormalizeValue` | — | — |
| `CheckConsistency` | — | — |
| `ReviewFhirRequirement` | — | — |

### `struct CorrectionItem`

| Field | Type | Description |
|---|---|---|
| `field_name` | `String` | — |
| `action` | `CorrectionAction` | — |
| `description` | `String` | — |
| `suggested_value` | `Option<String>` | — |
| `priority` | `CorrectionPriority` | — |

### `struct CorrectionWorkflow`

| Field | Type | Description |
|---|---|---|
| `record_id` | `u64` | — |
| `total_issues` | `u32` | — |
| `critical_count` | `u32` | — |
| `error_count` | `u32` | — |
| `warning_count` | `u32` | — |
| `info_count` | `u32` | — |
| `corrections` | `Vec<CorrectionItem>` | — |
| `can_auto_fix` | `bool` | — |
| `workflow_created_at` | `u64` | — |

### `struct CleanseResult`

| Field | Type | Description |
|---|---|---|
| `record` | `MedicalRecord` | — |
| `changes_made` | `Vec<String>` | — |
| `was_modified` | `bool` | — |

### `enum ExportFormat`

| Variant | Value | Description |
|---|---|---|
| `FHIRBundle` | — | — |
| `HL7v2` | — | — |
| `CDA` | — | — |

### `enum LogLevel`

| Variant | Value | Description |
|---|---|---|
| `Info` | — | — |
| `Warning` | — | — |
| `LogError` | — | — |

### `struct StructuredLog`

| Field | Type | Description |
|---|---|---|
| `timestamp` | `u64` | — |
| `level` | `LogLevel` | — |
| `operation` | `String` | — |
| `actor` | `Option<Address>` | — |
| `target_id` | `Option<Address>` | — |
| `record_id` | `Option<u64>` | — |
| `message` | `String` | — |


## Error Codes

| Variant | Code | Description |
|---|---|---|
| `NotAICoordinator` | 1150 | — |
| `EmergencyAccessExpired` | 1160 | — |
| `InvalidPagination` | 1202 | — |
| `InputTooLong` | 1201 | — |
| `BatchTooLarge` | 1208 | — |
| `InvalidSignature` | 1207 | — |
| `InvalidDataRefLength` | 1250 | — |
| `InvalidDataRefCharset` | 1251 | — |
| `InvalidDiagnosisLength` | 1252 | — |
| `InvalidTreatmentLength` | 1253 | — |
| `InvalidPurposeLength` | 1254 | — |
| `InvalidTagLength` | 1255 | — |
| `InvalidModelVersionLength` | 1256 | — |
| `InvalidExplanationLength` | 1257 | — |
| `InvalidTreatmentTypeLength` | 1258 | — |
| `InvalidAddress` | 1290 | — |
| `SameAddress` | 1291 | — |
| `InvalidBatch` | 1292 | — |
| `NumberOutOfBounds` | 1293 | — |
| `InvalidCategory` | 1280 | — |
| `EmptyTreatment` | 1281 | — |
| `EmptyDiagnosis` | 1282 | — |
| `EmptyTag` | 1283 | — |
| `EmptyDataRef` | 1284 | — |
| `ProposalAlreadyExecuted` | 1320 | — |
| `TimelockNotElapsed` | 1321 | — |
| `NotEnoughApproval` | 1322 | — |
| `CryptoRegistryNotSet` | 1340 | — |
| `EncryptionRequired` | 1341 | — |
| `IdentityRegistryNotSet` | 1342 | — |
| `RecordNotFound` | 1403 | — |
| `EmergencyAccessNotFound` | 1460 | — |
| `DIDNotFound` | 1470 | — |
| `DIDNotActive` | 1471 | — |
| `RecordAlreadySynced` | 1480 | — |
| `StorageFull` | 1502 | — |
| `InvalidCredential` | 1640 | — |
| `MissingRequiredCredential` | 1641 | — |
| `CredentialExpired` | 1605 | — |
| `CredentialRevoked` | 1606 | — |
| `CrossChainAccessDenied` | 1700 | — |
| `CrossChainTimeout` | 1702 | — |
| `InvalidChain` | 1703 | — |
| `CrossChainNotEnabled` | 1710 | — |
| `CrossChainContractsNotSet` | 1711 | — |
| `AIConfigNotSet` | 1830 | — |
| `InvalidAIScore` | 1831 | — |
| `InvalidScore` | 1832 | — |
| `InvalidDPEpsilon` | 1833 | — |
| `InvalidParticipantCount` | 1834 | — |

<!-- API_END -->

## Access Control

Records are protected by patient consent. Only the patient can grant/revoke access. All access is logged to the audit contract.

## Traditional Medicine Support

Records support a `metadata` field for traditional healing practice annotations.
