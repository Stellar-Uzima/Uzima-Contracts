#![no_std]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Map,
    String, Vec,
};

// ==================== IHE Profile Identifiers ====================

/// All supported IHE integration profiles (13 profiles for Connectathon compliance)
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum IHEProfile {
    XDS,  // Cross-Enterprise Document Sharing
    PIX,  // Patient Identifier Cross-referencing
    PDQ,  // Patient Demographics Query
    ATNA, // Audit Trail and Node Authentication
    XCA,  // Cross-Community Access
    MPI,  // Master Patient Index
    XDR,  // Cross-Enterprise Document Reliable Interchange
    XDM,  // Cross-Enterprise Document Media Interchange
    CT,   // Consistent Time
    BPPC, // Basic Patient Privacy Consents
    DSG,  // Document Digital Signature
    HPD,  // Healthcare Provider Directory
    SVS,  // Sharing Value Sets
}

// ==================== HL7 Message Support ====================

/// HL7 v2 and v3 message types supported across profiles
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum HL7MessageType {
    // HL7 v2
    V2ADT, // Admit, Discharge, Transfer
    V2ORM, // Order Message
    V2ORU, // Observation Result
    V2MFN, // Master File Notification
    V2QBP, // Query By Parameter
    V2RSP, // Segment Pattern Response
    V2ACK, // General Acknowledgment
    // HL7 v3
    V3ClinicalDocument,
    V3PatientQuery,
    V3PatientResponse,
    V3DeviceQuery,
}

// ==================== XDS: Cross-Enterprise Document Sharing ====================

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum DocumentStatus {
    Approved,
    Deprecated,
    Submitted,
}

/// XDS document entry stored in the registry
#[derive(Clone)]
#[contracttype]
pub struct XDSDocumentEntry {
    pub document_id: String,
    pub patient_id: String,
    pub content_hash: BytesN<32>,
    pub document_class_code: String,
    pub document_type_code: String,
    pub format_code: String,
    pub healthcare_facility_type: String,
    pub practice_setting_code: String,
    pub creation_time: u64,
    pub author: Address,
    pub confidentiality_code: String,
    pub language_code: String,
    pub hl7_message_type: HL7MessageType,
    pub status: DocumentStatus,
    pub repository_unique_id: String,
    pub submission_set_id: String,
    pub mime_type: String,
}

/// XDS submission set grouping documents from one submission
#[derive(Clone)]
#[contracttype]
pub struct XDSSubmissionSet {
    pub submission_set_id: String,
    pub patient_id: String,
    pub submission_time: u64,
    pub source_id: String,
    pub author: Address,
    pub content_type_code: String,
    pub document_ids: Vec<String>,
    pub intended_recipient: String,
}

// ==================== PIX: Patient Identifier Cross-referencing ====================

/// Single patient identifier from an assigning authority
#[derive(Clone)]
#[contracttype]
pub struct PatientIdentifier {
    pub id_value: String,
    pub assigning_authority: String,
    pub identifier_type_code: String,
}

/// PIX cross-reference linking identifiers across domains
#[derive(Clone)]
#[contracttype]
pub struct PIXCrossReference {
    pub reference_id: u64,
    pub local_id: PatientIdentifier,
    pub cross_referenced_ids: Vec<PatientIdentifier>,
    pub created_at: u64,
    pub updated_at: u64,
    pub is_merged: bool,
}

// ==================== PDQ: Patient Demographics Query ====================

/// Full patient demographics record
#[derive(Clone)]
#[contracttype]
pub struct PatientDemographics {
    pub patient_id: String,
    pub given_name: String,
    pub family_name: String,
    pub date_of_birth: String,
    pub administrative_gender: String,
    pub street_address: String,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub country_code: String,
    pub phone_home: String,
    pub phone_mobile: String,
    pub mother_maiden_name: String,
    pub marital_status: String,
    pub race: String,
    pub ethnicity: String,
    pub primary_language: String,
    pub last_updated: u64,
    pub assigning_authority: String,
}

/// PDQ query request with HL7 v2/v3 parameter set
#[derive(Clone)]
#[contracttype]
pub struct PDQQuery {
    pub query_id: u64,
    pub query_parameters: Map<String, String>,
    pub requesting_system: String,
    pub query_time: u64,
    pub hl7_message_type: HL7MessageType,
    pub domain_filter: String,
}

// ==================== ATNA: Audit Trail and Node Authentication ====================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum ATNAEventType {
    PatientRecordAccess,
    PatientRecordUpdate,
    UserAuthentication,
    NodeAuthentication,
    DocumentExport,
    DocumentImport,
    QueryRequest,
    QueryResponse,
    SecurityAlert,
    OrderMessage,
    ProcedureRecord,
}

/// DICOM/IHE event outcome indicator (numeric codes from DICOM PS 3.15)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum ATNAEventOutcome {
    Success = 0,
    MinorFailure = 4,
    SeriousFailure = 8,
    MajorFailure = 12,
}

/// Active participant in an ATNA audit event
#[derive(Clone)]
#[contracttype]
pub struct ATNAParticipant {
    pub user_id: String,
    pub user_name: String,
    pub role_id_code: String,
    pub is_requestor: bool,
    pub network_access_point: String,
}

/// Participant object (patient/document/query) in ATNA event
#[derive(Clone)]
#[contracttype]
pub struct ATNAParticipantObject {
    pub object_id_type_code: String,
    pub object_id: String,
    pub object_type_code: u32,
    pub object_sensitivity: String,
    pub object_query: String,
}

/// Full ATNA audit event (DICOM Supplement 95 compliant)
#[derive(Clone)]
#[contracttype]
pub struct ATNAAuditEvent {
    pub event_id: u64,
    pub event_type: ATNAEventType,
    pub event_action_code: String,
    pub event_date_time: u64,
    pub event_outcome: ATNAEventOutcome,
    pub source_id: String,
    pub source_type: String,
    pub active_participants: Vec<ATNAParticipant>,
    pub participant_objects: Vec<ATNAParticipantObject>,
    pub hl7_message_id: String,
    pub profile: IHEProfile,
}

// ==================== XCA: Cross-Community Access ====================

/// XCA gateway registration for cross-community queries
#[derive(Clone)]
#[contracttype]
pub struct XCAGateway {
    pub gateway_id: String,
    pub community_id: String,
    pub gateway_address: String,
    pub supported_profiles: Vec<IHEProfile>,
    pub registered_by: Address,
    pub registration_time: u64,
    pub is_active: bool,
}

// ==================== MPI: Master Patient Index ====================

/// MPI master patient record linking local identities
#[derive(Clone)]
#[contracttype]
pub struct MPIMasterPatient {
    pub master_id: u64,
    pub global_patient_id: String,
    pub linked_identifiers: Vec<PatientIdentifier>,
    pub demographics: PatientDemographics,
    pub created_at: u64,
    pub updated_at: u64,
    pub confidence_score: u32,
}

// ==================== BPPC: Basic Patient Privacy Consents ====================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum ConsentStatus {
    Active,
    Revoked,
    Expired,
}

/// Patient privacy consent document (BPPC profile)
#[derive(Clone)]
#[contracttype]
pub struct BPPCConsent {
    pub consent_id: u64,
    pub patient_id: String,
    pub policy_id: String,
    pub consent_status: ConsentStatus,
    pub access_consent_list: Vec<String>,
    pub date_of_consent: u64,
    pub expiry_time: u64,
    pub author: Address,
    pub document_ref: String,
}

// ==================== DSG: Document Digital Signature ====================

/// Digital signature record for a document (DSG profile)
#[derive(Clone)]
#[contracttype]
pub struct DSGSignature {
    pub signature_id: u64,
    pub document_id: String,
    pub signer: Address,
    pub signature_hash: BytesN<32>,
    pub signature_algorithm: String,
    pub signing_time: u64,
    pub certificate_ref: String,
    pub signature_purpose: String,
    pub is_valid: bool,
}

// ==================== HPD: Healthcare Provider Directory ====================

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum ProviderType {
    Individual,
    Organization,
    Department,
}

/// Provider entry in the Healthcare Provider Directory
#[derive(Clone)]
#[contracttype]
pub struct HPDProvider {
    pub provider_id: u64,
    pub provider_type: ProviderType,
    pub given_name: String,
    pub family_name: String,
    pub organization_name: String,
    pub specialty_code: String,
    pub license_number: String,
    pub npi: String,
    pub address: String,
    pub electronic_service_info: String,
    pub registered_by: Address,
    pub registration_time: u64,
    pub is_active: bool,
}

// ==================== SVS: Sharing Value Sets ====================

/// A single coded concept in a value set
#[derive(Clone)]
#[contracttype]
pub struct SVSConcept {
    pub code: String,
    pub code_system: String,
    pub code_system_name: String,
    pub display_name: String,
    pub level: u32,
    pub type_code: String,
}

/// A named value set with a list of concepts
#[derive(Clone)]
#[contracttype]
pub struct SVSValueSet {
    pub value_set_id: u64,
    pub oid: String,
    pub name: String,
    pub version: String,
    pub status: String,
    pub description: String,
    pub concepts: Vec<SVSConcept>,
    pub effective_date: u64,
    pub source_url: String,
    pub registered_by: Address,
}

// ==================== Connectathon Compliance Record ====================

/// IHE Connectathon test result for a profile
#[derive(Clone)]
#[contracttype]
pub struct ConnectathonTestResult {
    pub test_id: u64,
    pub profile: IHEProfile,
    pub actor_name: String,
    pub test_name: String,
    pub passed: bool,
    pub tested_at: u64,
    pub tested_by: Address,
    pub notes: String,
}

// ==================== Storage Keys ====================

#[contracttype]
pub enum DataKey {
    Admin,
    // Counters
    NextDocumentId,
    NextPixRefId,
    NextPdqQueryId,
    NextAtnaEventId,
    NextMasterPatientId,
    NextConsentId,
    NextSignatureId,
    NextProviderId,
    NextValueSetId,
    NextTestResultId,
    // XDS
    XDSDocument(String),         // document_id -> XDSDocumentEntry
    XDSSubmissionSet(String),    // submission_set_id -> XDSSubmissionSet
    PatientDocuments(String),    // patient_id -> Vec<String> document IDs
    // PIX
    PIXCrossRef(u64),            // reference_id -> PIXCrossReference
    PIXPatientRefs(String),      // patient_id -> Vec<u64> reference IDs
    // PDQ
    PatientDemographics(String), // patient_id -> PatientDemographics
    PDQQuery(u64),               // query_id -> PDQQuery
    // ATNA
    ATNAEvent(u64),              // event_id -> ATNAAuditEvent
    // XCA
    XCAGateway(String),          // gateway_id -> XCAGateway
    // MPI
    MPIMasterPatient(u64),       // master_id -> MPIMasterPatient
    MPIGlobalIndex(String),      // global_patient_id -> master_id
    // BPPC
    BPPCConsent(u64),            // consent_id -> BPPCConsent
    PatientConsents(String),     // patient_id -> Vec<u64> consent IDs
    // DSG
    DSGSignature(u64),           // signature_id -> DSGSignature
    DocumentSignatures(String),  // document_id -> Vec<u64> signature IDs
    // HPD
    HPDProvider(u64),            // provider_id -> HPDProvider
    // SVS
    SVSValueSet(u64),            // value_set_id -> SVSValueSet
    SVSValueSetByOid(String),    // oid -> value_set_id
    // Connectathon
    ConnectathonResult(u64),     // test_id -> ConnectathonTestResult
    ProfileTestIds(IHEProfile),  // profile -> Vec<u64> test IDs
}

// ==================== Errors ====================

#[contracterror]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    NotAuthorized = 3,
    DocumentNotFound = 4,
    DocumentAlreadyExists = 5,
    DocumentDeprecated = 6,
    PatientNotFound = 7,
    CrossReferenceNotFound = 8,
    DemographicsNotFound = 9,
    AuditEventNotFound = 10,
    GatewayNotFound = 11,
    GatewayAlreadyExists = 12,
    MasterPatientNotFound = 13,
    ConsentNotFound = 14,
    ConsentRevoked = 15,
    ConsentExpired = 16,
    SignatureNotFound = 17,
    SignatureInvalid = 18,
    ProviderNotFound = 19,
    ValueSetNotFound = 20,
    ValueSetOidExists = 21,
    InvalidHL7Message = 22,
    ConnectathonTestNotFound = 23,
    EmptyPatientId = 24,
    EmptyDocumentId = 25,
}

// ==================== Contract ====================

#[contract]
pub struct IHEIntegrationContract;

#[contractimpl]
impl IHEIntegrationContract {
    // ==================== Initialization ====================

    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::NextDocumentId, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::NextPixRefId, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::NextPdqQueryId, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::NextAtnaEventId, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::NextMasterPatientId, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::NextConsentId, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::NextSignatureId, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::NextProviderId, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::NextValueSetId, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::NextTestResultId, &0u64);

        env.events().publish(
            (symbol_short!("IHE"), symbol_short!("INIT")),
            admin,
        );

        Ok(())
    }

    // ==================== XDS: Cross-Enterprise Document Sharing ====================

    /// Register a new document in the XDS registry
    pub fn xds_register_document(
        env: Env,
        author: Address,
        entry: XDSDocumentEntry,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        author.require_auth();
        Self::xds_store_document(&env, &author, &entry)
    }

    fn xds_store_document(env: &Env, author: &Address, entry: &XDSDocumentEntry) -> Result<(), Error> {
        let doc_id = entry.document_id.clone();

        if env
            .storage()
            .persistent()
            .has(&DataKey::XDSDocument(doc_id.clone()))
        {
            return Err(Error::DocumentAlreadyExists);
        }

        env.storage()
            .persistent()
            .set(&DataKey::XDSDocument(doc_id.clone()), entry);

        // Index by patient
        let patient_id = entry.patient_id.clone();
        let mut patient_docs: Vec<String> = env
            .storage()
            .persistent()
            .get(&DataKey::PatientDocuments(patient_id.clone()))
            .unwrap_or(Vec::new(env));
        patient_docs.push_back(doc_id.clone());
        env.storage()
            .persistent()
            .set(&DataKey::PatientDocuments(patient_id), &patient_docs);

        Self::log_atna_internal(
            env,
            ATNAEventType::DocumentImport,
            String::from_str(env, "C"),
            ATNAEventOutcome::Success,
            String::from_str(env, "XDS_REGISTRY"),
            String::from_str(env, "4"),
            author.clone(),
            IHEProfile::XDS,
        );

        env.events().publish(
            (symbol_short!("XDS"), symbol_short!("REG")),
            (doc_id, entry.submission_set_id.clone(), author.clone()),
        );

        Ok(())
    }

    /// Deprecate an existing XDS document entry
    pub fn xds_deprecate_document(
        env: Env,
        author: Address,
        document_id: String,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        author.require_auth();

        let key = DataKey::XDSDocument(document_id.clone());
        let mut entry: XDSDocumentEntry = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::DocumentNotFound)?;

        entry.status = DocumentStatus::Deprecated;
        env.storage().persistent().set(&key, &entry);

        Self::log_atna_internal(
            &env,
            ATNAEventType::PatientRecordUpdate,
            String::from_str(&env, "U"),
            ATNAEventOutcome::Success,
            String::from_str(&env, "XDS_REGISTRY"),
            String::from_str(&env, "4"),
            author.clone(),
            IHEProfile::XDS,
        );

        env.events().publish(
            (symbol_short!("XDS"), symbol_short!("DEPR")),
            (document_id, author),
        );

        Ok(())
    }

    /// Query XDS documents for a patient
    pub fn xds_query_documents(
        env: Env,
        requester: Address,
        patient_id: String,
    ) -> Result<Vec<XDSDocumentEntry>, Error> {
        Self::require_initialized(&env)?;
        requester.require_auth();

        let doc_ids: Vec<String> = env
            .storage()
            .persistent()
            .get(&DataKey::PatientDocuments(patient_id.clone()))
            .unwrap_or(Vec::new(&env));

        let mut results = Vec::new(&env);
        for doc_id in doc_ids.iter() {
            if let Some(entry) = env
                .storage()
                .persistent()
                .get::<DataKey, XDSDocumentEntry>(&DataKey::XDSDocument(doc_id.clone()))
            {
                if entry.status != DocumentStatus::Deprecated {
                    results.push_back(entry);
                }
            }
        }

        Self::log_atna_internal(
            &env,
            ATNAEventType::QueryRequest,
            String::from_str(&env, "E"),
            ATNAEventOutcome::Success,
            String::from_str(&env, "XDS_REGISTRY"),
            String::from_str(&env, "4"),
            requester,
            IHEProfile::XDS,
        );

        Ok(results)
    }

    /// Retrieve a single XDS document entry
    pub fn xds_retrieve_document(
        env: Env,
        requester: Address,
        document_id: String,
    ) -> Result<XDSDocumentEntry, Error> {
        Self::require_initialized(&env)?;
        requester.require_auth();

        let entry: XDSDocumentEntry = env
            .storage()
            .persistent()
            .get(&DataKey::XDSDocument(document_id.clone()))
            .ok_or(Error::DocumentNotFound)?;

        if entry.status == DocumentStatus::Deprecated {
            return Err(Error::DocumentDeprecated);
        }

        Self::log_atna_internal(
            &env,
            ATNAEventType::PatientRecordAccess,
            String::from_str(&env, "R"),
            ATNAEventOutcome::Success,
            String::from_str(&env, "XDS_REPOSITORY"),
            String::from_str(&env, "4"),
            requester,
            IHEProfile::XDS,
        );

        Ok(entry)
    }

    /// Submit an XDS submission set (groups documents from one clinical event)
    pub fn xds_submit_document_set(
        env: Env,
        author: Address,
        submission_set: XDSSubmissionSet,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        author.require_auth();

        let ss_id = submission_set.submission_set_id.clone();
        env.storage()
            .persistent()
            .set(&DataKey::XDSSubmissionSet(ss_id.clone()), &submission_set);

        env.events().publish(
            (symbol_short!("XDS"), symbol_short!("SUBMIT")),
            (ss_id, author),
        );

        Ok(())
    }

    // ==================== PIX: Patient Identifier Cross-referencing ====================

    /// Register a patient identity and return the cross-reference record ID
    pub fn pix_register_patient(
        env: Env,
        actor: Address,
        local_id: PatientIdentifier,
        cross_ids: Vec<PatientIdentifier>,
    ) -> Result<u64, Error> {
        Self::require_initialized(&env)?;
        actor.require_auth();

        let ref_id = Self::next_id(&env, DataKey::NextPixRefId);
        let cross_ref = PIXCrossReference {
            reference_id: ref_id,
            local_id: local_id.clone(),
            cross_referenced_ids: cross_ids,
            created_at: env.ledger().timestamp(),
            updated_at: env.ledger().timestamp(),
            is_merged: false,
        };

        env.storage()
            .persistent()
            .set(&DataKey::PIXCrossRef(ref_id), &cross_ref);

        // Index by patient id value
        let pid = local_id.id_value.clone();
        let mut refs: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::PIXPatientRefs(pid.clone()))
            .unwrap_or(Vec::new(&env));
        refs.push_back(ref_id);
        env.storage()
            .persistent()
            .set(&DataKey::PIXPatientRefs(pid), &refs);

        Self::log_atna_internal(
            &env,
            ATNAEventType::PatientRecordAccess,
            String::from_str(&env, "C"),
            ATNAEventOutcome::Success,
            String::from_str(&env, "PIX_MANAGER"),
            String::from_str(&env, "4"),
            actor.clone(),
            IHEProfile::PIX,
        );

        env.events().publish(
            (symbol_short!("PIX"), symbol_short!("REG")),
            (ref_id, actor),
        );

        Ok(ref_id)
    }

    /// Query all cross-referenced identifiers for a patient
    pub fn pix_query_identifiers(
        env: Env,
        requester: Address,
        patient_id: String,
    ) -> Result<Vec<PIXCrossReference>, Error> {
        Self::require_initialized(&env)?;
        requester.require_auth();

        let ref_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::PIXPatientRefs(patient_id.clone()))
            .unwrap_or(Vec::new(&env));

        let mut results = Vec::new(&env);
        for id in ref_ids.iter() {
            if let Some(cross_ref) = env
                .storage()
                .persistent()
                .get::<DataKey, PIXCrossReference>(&DataKey::PIXCrossRef(id))
            {
                results.push_back(cross_ref);
            }
        }

        if results.is_empty() {
            return Err(Error::PatientNotFound);
        }

        Self::log_atna_internal(
            &env,
            ATNAEventType::QueryRequest,
            String::from_str(&env, "E"),
            ATNAEventOutcome::Success,
            String::from_str(&env, "PIX_MANAGER"),
            String::from_str(&env, "4"),
            requester,
            IHEProfile::PIX,
        );

        Ok(results)
    }

    /// Merge two patient identities (PIX merge operation)
    pub fn pix_merge_patients(
        env: Env,
        actor: Address,
        surviving_ref_id: u64,
        subsumed_ref_id: u64,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        actor.require_auth();

        let mut surviving: PIXCrossReference = env
            .storage()
            .persistent()
            .get(&DataKey::PIXCrossRef(surviving_ref_id))
            .ok_or(Error::CrossReferenceNotFound)?;

        let mut subsumed: PIXCrossReference = env
            .storage()
            .persistent()
            .get(&DataKey::PIXCrossRef(subsumed_ref_id))
            .ok_or(Error::CrossReferenceNotFound)?;

        // Absorb subsumed identifiers into surviving
        for id in subsumed.cross_referenced_ids.iter() {
            surviving.cross_referenced_ids.push_back(id);
        }
        surviving.updated_at = env.ledger().timestamp();

        subsumed.is_merged = true;
        subsumed.updated_at = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::PIXCrossRef(surviving_ref_id), &surviving);
        env.storage()
            .persistent()
            .set(&DataKey::PIXCrossRef(subsumed_ref_id), &subsumed);

        env.events().publish(
            (symbol_short!("PIX"), symbol_short!("MERGE")),
            (surviving_ref_id, subsumed_ref_id),
        );

        Ok(())
    }

    // ==================== PDQ: Patient Demographics Query ====================

    /// Register or update patient demographics
    pub fn pdq_register_demographics(
        env: Env,
        actor: Address,
        demographics: PatientDemographics,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        actor.require_auth();

        let pid = demographics.patient_id.clone();
        env.storage()
            .persistent()
            .set(&DataKey::PatientDemographics(pid.clone()), &demographics);

        Self::log_atna_internal(
            &env,
            ATNAEventType::PatientRecordUpdate,
            String::from_str(&env, "C"),
            ATNAEventOutcome::Success,
            String::from_str(&env, "PDQ_SUPPLIER"),
            String::from_str(&env, "4"),
            actor.clone(),
            IHEProfile::PDQ,
        );

        env.events().publish(
            (symbol_short!("PDQ"), symbol_short!("REG")),
            (pid, actor),
        );

        Ok(())
    }

    /// Execute a PDQ demographics query; returns matching records
    pub fn pdq_query(
        env: Env,
        requester: Address,
        query_params: Map<String, String>,
        requesting_system: String,
        hl7_type: HL7MessageType,
        domain_filter: String,
    ) -> Result<u64, Error> {
        Self::require_initialized(&env)?;
        requester.require_auth();

        let query_id = Self::next_id(&env, DataKey::NextPdqQueryId);
        let query = PDQQuery {
            query_id,
            query_parameters: query_params,
            requesting_system: requesting_system.clone(),
            query_time: env.ledger().timestamp(),
            hl7_message_type: hl7_type,
            domain_filter,
        };

        env.storage()
            .persistent()
            .set(&DataKey::PDQQuery(query_id), &query);

        Self::log_atna_internal(
            &env,
            ATNAEventType::QueryRequest,
            String::from_str(&env, "E"),
            ATNAEventOutcome::Success,
            String::from_str(&env, "PDQ_CONSUMER"),
            String::from_str(&env, "4"),
            requester,
            IHEProfile::PDQ,
        );

        Ok(query_id)
    }

    /// Retrieve patient demographics by patient ID
    pub fn pdq_get_demographics(
        env: Env,
        requester: Address,
        patient_id: String,
    ) -> Result<PatientDemographics, Error> {
        Self::require_initialized(&env)?;
        requester.require_auth();

        let demographics: PatientDemographics = env
            .storage()
            .persistent()
            .get(&DataKey::PatientDemographics(patient_id.clone()))
            .ok_or(Error::DemographicsNotFound)?;

        Self::log_atna_internal(
            &env,
            ATNAEventType::PatientRecordAccess,
            String::from_str(&env, "R"),
            ATNAEventOutcome::Success,
            String::from_str(&env, "PDQ_SUPPLIER"),
            String::from_str(&env, "4"),
            requester,
            IHEProfile::PDQ,
        );

        Ok(demographics)
    }

    // ==================== ATNA: Audit Trail and Node Authentication ====================

    /// Log an ATNA-compliant audit event (used by external actors and other profiles)
    pub fn atna_log_event(
        env: Env,
        actor: Address,
        event_type: ATNAEventType,
        event_action_code: String,
        event_outcome: ATNAEventOutcome,
        source_id: String,
        source_type: String,
        active_participants: Vec<ATNAParticipant>,
        participant_objects: Vec<ATNAParticipantObject>,
        hl7_message_id: String,
        profile: IHEProfile,
    ) -> Result<u64, Error> {
        Self::require_initialized(&env)?;
        actor.require_auth();

        let event_id = Self::next_id(&env, DataKey::NextAtnaEventId);
        let event = ATNAAuditEvent {
            event_id,
            event_type,
            event_action_code,
            event_date_time: env.ledger().timestamp(),
            event_outcome,
            source_id,
            source_type,
            active_participants,
            participant_objects,
            hl7_message_id,
            profile,
        };

        env.storage()
            .persistent()
            .set(&DataKey::ATNAEvent(event_id), &event);

        env.events().publish(
            (symbol_short!("ATNA"), symbol_short!("LOG")),
            (event_id, event_type, event_outcome),
        );

        Ok(event_id)
    }

    /// Retrieve an ATNA audit event by ID
    pub fn atna_get_event(env: Env, event_id: u64) -> Result<ATNAAuditEvent, Error> {
        Self::require_initialized(&env)?;

        env.storage()
            .persistent()
            .get(&DataKey::ATNAEvent(event_id))
            .ok_or(Error::AuditEventNotFound)
    }

    /// Authenticate a node and record the ATNA authentication event
    pub fn atna_authenticate_node(
        env: Env,
        node: Address,
        node_id: String,
        certificate_hash: BytesN<32>,
    ) -> Result<u64, Error> {
        Self::require_initialized(&env)?;
        node.require_auth();

        let event_id = Self::next_id(&env, DataKey::NextAtnaEventId);
        let participant = ATNAParticipant {
            user_id: node_id.clone(),
            user_name: node_id.clone(),
            role_id_code: String::from_str(&env, "110153"),
            is_requestor: true,
            network_access_point: node_id.clone(),
        };

        let mut participants = Vec::new(&env);
        participants.push_back(participant);

        let event = ATNAAuditEvent {
            event_id,
            event_type: ATNAEventType::NodeAuthentication,
            event_action_code: String::from_str(&env, "E"),
            event_date_time: env.ledger().timestamp(),
            event_outcome: ATNAEventOutcome::Success,
            source_id: node_id,
            source_type: String::from_str(&env, "4"),
            active_participants: participants,
            participant_objects: Vec::new(&env),
            hl7_message_id: String::from_str(&env, ""),
            profile: IHEProfile::ATNA,
        };

        env.storage()
            .persistent()
            .set(&DataKey::ATNAEvent(event_id), &event);

        env.events().publish(
            (symbol_short!("ATNA"), symbol_short!("AUTH")),
            (event_id, certificate_hash),
        );

        Ok(event_id)
    }

    // ==================== XCA: Cross-Community Access ====================

    /// Register a cross-community gateway
    pub fn xca_register_gateway(
        env: Env,
        admin: Address,
        gateway: XCAGateway,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        Self::require_admin(&env, &admin)?;
        admin.require_auth();

        let gw_id = gateway.gateway_id.clone();

        if env
            .storage()
            .persistent()
            .has(&DataKey::XCAGateway(gw_id.clone()))
        {
            return Err(Error::GatewayAlreadyExists);
        }

        env.storage()
            .persistent()
            .set(&DataKey::XCAGateway(gw_id.clone()), &gateway);

        env.events().publish(
            (symbol_short!("XCA"), symbol_short!("REG")),
            (gw_id, admin),
        );

        Ok(())
    }

    /// Initiate a cross-gateway query (returns gateway record for routing)
    pub fn xca_initiate_query(
        env: Env,
        requester: Address,
        gateway_id: String,
        patient_id: String,
    ) -> Result<XCAGateway, Error> {
        Self::require_initialized(&env)?;
        requester.require_auth();

        let gateway: XCAGateway = env
            .storage()
            .persistent()
            .get(&DataKey::XCAGateway(gateway_id.clone()))
            .ok_or(Error::GatewayNotFound)?;

        Self::log_atna_internal(
            &env,
            ATNAEventType::QueryRequest,
            String::from_str(&env, "E"),
            ATNAEventOutcome::Success,
            gateway_id,
            String::from_str(&env, "4"),
            requester,
            IHEProfile::XCA,
        );

        env.events().publish(
            (symbol_short!("XCA"), symbol_short!("QUERY")),
            patient_id,
        );

        Ok(gateway)
    }

    // ==================== MPI: Master Patient Index ====================

    /// Register a master patient record linking multiple local identifiers
    pub fn mpi_register_master_patient(
        env: Env,
        actor: Address,
        global_patient_id: String,
        demographics: PatientDemographics,
        linked_ids: Vec<PatientIdentifier>,
        confidence_score: u32,
    ) -> Result<u64, Error> {
        Self::require_initialized(&env)?;
        actor.require_auth();

        let master_id = Self::next_id(&env, DataKey::NextMasterPatientId);
        let master = MPIMasterPatient {
            master_id,
            global_patient_id: global_patient_id.clone(),
            linked_identifiers: linked_ids,
            demographics,
            created_at: env.ledger().timestamp(),
            updated_at: env.ledger().timestamp(),
            confidence_score,
        };

        env.storage()
            .persistent()
            .set(&DataKey::MPIMasterPatient(master_id), &master);
        env.storage()
            .persistent()
            .set(&DataKey::MPIGlobalIndex(global_patient_id.clone()), &master_id);

        env.events().publish(
            (symbol_short!("MPI"), symbol_short!("REG")),
            (master_id, global_patient_id),
        );

        Ok(master_id)
    }

    /// Find a master patient record by global patient ID
    pub fn mpi_find_patient(
        env: Env,
        requester: Address,
        global_patient_id: String,
    ) -> Result<MPIMasterPatient, Error> {
        Self::require_initialized(&env)?;
        requester.require_auth();

        let master_id: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::MPIGlobalIndex(global_patient_id))
            .ok_or(Error::MasterPatientNotFound)?;

        env.storage()
            .persistent()
            .get(&DataKey::MPIMasterPatient(master_id))
            .ok_or(Error::MasterPatientNotFound)
    }

    // ==================== XDR: Cross-Enterprise Document Reliable Interchange ====================

    /// Reliable document interchange — wraps XDS registration with delivery confirmation
    pub fn xdr_send_document(
        env: Env,
        sender: Address,
        entry: XDSDocumentEntry,
        intended_recipient: String,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        sender.require_auth();

        // Store directly without re-checking auth to avoid double require_auth
        Self::xds_store_document(&env, &sender, &entry)?;

        env.events().publish(
            (symbol_short!("XDR"), symbol_short!("SEND")),
            (entry.document_id, intended_recipient, sender),
        );

        Ok(())
    }

    // ==================== XDM: Cross-Enterprise Document Media Interchange ====================

    /// Record a media interchange package (content hash stored on-chain)
    pub fn xdm_record_media_package(
        env: Env,
        actor: Address,
        package_id: String,
        patient_id: String,
        content_hash: BytesN<32>,
        media_type: String,
        document_ids: Vec<String>,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        actor.require_auth();

        // Build a minimal XDS submission set to track the package
        let ss = XDSSubmissionSet {
            submission_set_id: package_id.clone(),
            patient_id,
            submission_time: env.ledger().timestamp(),
            source_id: media_type,
            author: actor.clone(),
            content_type_code: String::from_str(&env, "XDM"),
            document_ids,
            intended_recipient: String::from_str(&env, "MEDIA"),
        };

        env.storage()
            .persistent()
            .set(&DataKey::XDSSubmissionSet(package_id.clone()), &ss);

        env.events().publish(
            (symbol_short!("XDM"), symbol_short!("PKG")),
            (package_id, content_hash, actor),
        );

        Ok(())
    }

    // ==================== CT: Consistent Time ====================

    /// Record a time synchronization event on-chain
    pub fn ct_record_time_sync(
        env: Env,
        actor: Address,
        node_id: String,
        reported_time: u64,
    ) -> Result<u64, Error> {
        Self::require_initialized(&env)?;
        actor.require_auth();

        let ledger_time = env.ledger().timestamp();
        let drift = if reported_time > ledger_time {
            reported_time.saturating_sub(ledger_time)
        } else {
            ledger_time.saturating_sub(reported_time)
        };

        env.events().publish(
            (symbol_short!("CT"), symbol_short!("SYNC")),
            (node_id, ledger_time, reported_time, drift),
        );

        Ok(drift)
    }

    // ==================== BPPC: Basic Patient Privacy Consents ====================

    /// Register a patient privacy consent document
    pub fn bppc_register_consent(
        env: Env,
        author: Address,
        patient_id: String,
        policy_id: String,
        access_consent_list: Vec<String>,
        expiry_time: u64,
        document_ref: String,
    ) -> Result<u64, Error> {
        Self::require_initialized(&env)?;
        author.require_auth();

        let consent_id = Self::next_id(&env, DataKey::NextConsentId);
        let consent = BPPCConsent {
            consent_id,
            patient_id: patient_id.clone(),
            policy_id,
            consent_status: ConsentStatus::Active,
            access_consent_list,
            date_of_consent: env.ledger().timestamp(),
            expiry_time,
            author: author.clone(),
            document_ref,
        };

        env.storage()
            .persistent()
            .set(&DataKey::BPPCConsent(consent_id), &consent);

        let mut patient_consents: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::PatientConsents(patient_id.clone()))
            .unwrap_or(Vec::new(&env));
        patient_consents.push_back(consent_id);
        env.storage()
            .persistent()
            .set(&DataKey::PatientConsents(patient_id), &patient_consents);

        Self::log_atna_internal(
            &env,
            ATNAEventType::PatientRecordAccess,
            String::from_str(&env, "C"),
            ATNAEventOutcome::Success,
            String::from_str(&env, "BPPC"),
            String::from_str(&env, "4"),
            author.clone(),
            IHEProfile::BPPC,
        );

        env.events().publish(
            (symbol_short!("BPPC"), symbol_short!("REG")),
            (consent_id, author),
        );

        Ok(consent_id)
    }

    /// Revoke a privacy consent
    pub fn bppc_revoke_consent(
        env: Env,
        author: Address,
        consent_id: u64,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        author.require_auth();

        let key = DataKey::BPPCConsent(consent_id);
        let mut consent: BPPCConsent = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::ConsentNotFound)?;

        consent.consent_status = ConsentStatus::Revoked;
        env.storage().persistent().set(&key, &consent);

        env.events().publish(
            (symbol_short!("BPPC"), symbol_short!("REVOKE")),
            (consent_id, author),
        );

        Ok(())
    }

    /// Verify consent is active and not expired
    pub fn bppc_verify_consent(
        env: Env,
        consent_id: u64,
    ) -> Result<BPPCConsent, Error> {
        Self::require_initialized(&env)?;

        let consent: BPPCConsent = env
            .storage()
            .persistent()
            .get(&DataKey::BPPCConsent(consent_id))
            .ok_or(Error::ConsentNotFound)?;

        match consent.consent_status {
            ConsentStatus::Revoked => return Err(Error::ConsentRevoked),
            ConsentStatus::Expired => return Err(Error::ConsentExpired),
            ConsentStatus::Active => {}
        }

        if consent.expiry_time > 0 && env.ledger().timestamp() > consent.expiry_time {
            return Err(Error::ConsentExpired);
        }

        Ok(consent)
    }

    // ==================== DSG: Document Digital Signature ====================

    /// Record a digital signature for a document
    pub fn dsg_sign_document(
        env: Env,
        signer: Address,
        document_id: String,
        signature_hash: BytesN<32>,
        signature_algorithm: String,
        certificate_ref: String,
        signature_purpose: String,
    ) -> Result<u64, Error> {
        Self::require_initialized(&env)?;
        signer.require_auth();

        let sig_id = Self::next_id(&env, DataKey::NextSignatureId);
        let sig = DSGSignature {
            signature_id: sig_id,
            document_id: document_id.clone(),
            signer: signer.clone(),
            signature_hash,
            signature_algorithm,
            signing_time: env.ledger().timestamp(),
            certificate_ref,
            signature_purpose,
            is_valid: true,
        };

        env.storage()
            .persistent()
            .set(&DataKey::DSGSignature(sig_id), &sig);

        let mut doc_sigs: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::DocumentSignatures(document_id.clone()))
            .unwrap_or(Vec::new(&env));
        doc_sigs.push_back(sig_id);
        env.storage()
            .persistent()
            .set(&DataKey::DocumentSignatures(document_id.clone()), &doc_sigs);

        Self::log_atna_internal(
            &env,
            ATNAEventType::DocumentExport,
            String::from_str(&env, "C"),
            ATNAEventOutcome::Success,
            String::from_str(&env, "DSG"),
            String::from_str(&env, "4"),
            signer.clone(),
            IHEProfile::DSG,
        );

        env.events().publish(
            (symbol_short!("DSG"), symbol_short!("SIGN")),
            (sig_id, document_id, signer),
        );

        Ok(sig_id)
    }

    /// Verify a document signature by signature ID
    pub fn dsg_verify_signature(
        env: Env,
        signature_id: u64,
    ) -> Result<DSGSignature, Error> {
        Self::require_initialized(&env)?;

        let sig: DSGSignature = env
            .storage()
            .persistent()
            .get(&DataKey::DSGSignature(signature_id))
            .ok_or(Error::SignatureNotFound)?;

        if !sig.is_valid {
            return Err(Error::SignatureInvalid);
        }

        Ok(sig)
    }

    /// Get all signatures for a document
    pub fn dsg_get_document_signatures(
        env: Env,
        document_id: String,
    ) -> Result<Vec<DSGSignature>, Error> {
        Self::require_initialized(&env)?;

        let sig_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::DocumentSignatures(document_id))
            .unwrap_or(Vec::new(&env));

        let mut results = Vec::new(&env);
        for id in sig_ids.iter() {
            if let Some(sig) = env
                .storage()
                .persistent()
                .get::<DataKey, DSGSignature>(&DataKey::DSGSignature(id))
            {
                results.push_back(sig);
            }
        }

        Ok(results)
    }

    // ==================== HPD: Healthcare Provider Directory ====================

    /// Register a provider in the Healthcare Provider Directory
    pub fn hpd_register_provider(
        env: Env,
        actor: Address,
        provider: HPDProvider,
    ) -> Result<u64, Error> {
        Self::require_initialized(&env)?;
        actor.require_auth();

        let provider_id = Self::next_id(&env, DataKey::NextProviderId);
        let mut stored = provider;
        stored.provider_id = provider_id;
        stored.registration_time = env.ledger().timestamp();
        stored.registered_by = actor.clone();

        env.storage()
            .persistent()
            .set(&DataKey::HPDProvider(provider_id), &stored);

        env.events().publish(
            (symbol_short!("HPD"), symbol_short!("REG")),
            (provider_id, actor),
        );

        Ok(provider_id)
    }

    /// Query a provider by ID
    pub fn hpd_get_provider(
        env: Env,
        provider_id: u64,
    ) -> Result<HPDProvider, Error> {
        Self::require_initialized(&env)?;

        env.storage()
            .persistent()
            .get(&DataKey::HPDProvider(provider_id))
            .ok_or(Error::ProviderNotFound)
    }

    // ==================== SVS: Sharing Value Sets ====================

    /// Register a named value set
    pub fn svs_register_value_set(
        env: Env,
        actor: Address,
        value_set: SVSValueSet,
    ) -> Result<u64, Error> {
        Self::require_initialized(&env)?;
        actor.require_auth();

        let oid = value_set.oid.clone();

        if env
            .storage()
            .persistent()
            .has(&DataKey::SVSValueSetByOid(oid.clone()))
        {
            return Err(Error::ValueSetOidExists);
        }

        let vs_id = Self::next_id(&env, DataKey::NextValueSetId);
        let mut stored = value_set;
        stored.value_set_id = vs_id;
        stored.registered_by = actor.clone();

        env.storage()
            .persistent()
            .set(&DataKey::SVSValueSet(vs_id), &stored);
        env.storage()
            .persistent()
            .set(&DataKey::SVSValueSetByOid(oid.clone()), &vs_id);

        env.events().publish(
            (symbol_short!("SVS"), symbol_short!("REG")),
            (vs_id, oid, actor),
        );

        Ok(vs_id)
    }

    /// Retrieve a value set by OID
    pub fn svs_get_value_set_by_oid(
        env: Env,
        oid: String,
    ) -> Result<SVSValueSet, Error> {
        Self::require_initialized(&env)?;

        let vs_id: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::SVSValueSetByOid(oid))
            .ok_or(Error::ValueSetNotFound)?;

        env.storage()
            .persistent()
            .get(&DataKey::SVSValueSet(vs_id))
            .ok_or(Error::ValueSetNotFound)
    }

    // ==================== IHE Connectathon Compliance ====================

    /// Record the result of a Connectathon conformance test
    pub fn connectathon_record_test(
        env: Env,
        tester: Address,
        profile: IHEProfile,
        actor_name: String,
        test_name: String,
        passed: bool,
        notes: String,
    ) -> Result<u64, Error> {
        Self::require_initialized(&env)?;
        tester.require_auth();

        let test_id = Self::next_id(&env, DataKey::NextTestResultId);
        let result = ConnectathonTestResult {
            test_id,
            profile,
            actor_name,
            test_name,
            passed,
            tested_at: env.ledger().timestamp(),
            tested_by: tester.clone(),
            notes,
        };

        env.storage()
            .persistent()
            .set(&DataKey::ConnectathonResult(test_id), &result);

        // Index by profile
        let mut profile_tests: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::ProfileTestIds(profile))
            .unwrap_or(Vec::new(&env));
        profile_tests.push_back(test_id);
        env.storage()
            .persistent()
            .set(&DataKey::ProfileTestIds(profile), &profile_tests);

        env.events().publish(
            (symbol_short!("CONN"), symbol_short!("TEST")),
            (test_id, profile, passed),
        );

        Ok(test_id)
    }

    /// Get all Connectathon test results for a profile
    pub fn connectathon_get_profile_results(
        env: Env,
        profile: IHEProfile,
    ) -> Vec<ConnectathonTestResult> {
        let test_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::ProfileTestIds(profile))
            .unwrap_or(Vec::new(&env));

        let mut results = Vec::new(&env);
        for id in test_ids.iter() {
            if let Some(r) = env
                .storage()
                .persistent()
                .get::<DataKey, ConnectathonTestResult>(&DataKey::ConnectathonResult(id))
            {
                results.push_back(r);
            }
        }
        results
    }

    /// Check if a profile passes all recorded Connectathon tests
    pub fn connectathon_is_compliant(env: Env, profile: IHEProfile) -> bool {
        let results = Self::connectathon_get_profile_results(env, profile);
        if results.is_empty() {
            return false;
        }
        for r in results.iter() {
            if !r.passed {
                return false;
            }
        }
        true
    }

    // ==================== Internal Helpers ====================

    fn require_initialized(env: &Env) -> Result<(), Error> {
        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::NotInitialized);
        }
        Ok(())
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if *caller != admin {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    fn next_id(env: &Env, key: DataKey) -> u64 {
        let id: u64 = env.storage().instance().get(&key).unwrap_or(0u64);
        env.storage()
            .instance()
            .set(&key, &id.saturating_add(1));
        id
    }

    #[allow(clippy::too_many_arguments)]
    fn log_atna_internal(
        env: &Env,
        event_type: ATNAEventType,
        action_code: String,
        outcome: ATNAEventOutcome,
        source_id: String,
        source_type: String,
        actor: Address,
        profile: IHEProfile,
    ) {
        let event_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextAtnaEventId)
            .unwrap_or(0u64);

        let participant = ATNAParticipant {
            user_id: String::from_str(env, ""),
            user_name: String::from_str(env, ""),
            role_id_code: String::from_str(env, "110153"),
            is_requestor: true,
            network_access_point: String::from_str(env, ""),
        };
        let mut participants = Vec::new(env);
        participants.push_back(participant);

        let event = ATNAAuditEvent {
            event_id,
            event_type,
            event_action_code: action_code,
            event_date_time: env.ledger().timestamp(),
            event_outcome: outcome,
            source_id,
            source_type,
            active_participants: participants,
            participant_objects: Vec::new(env),
            hl7_message_id: String::from_str(env, ""),
            profile,
        };

        env.storage()
            .persistent()
            .set(&DataKey::ATNAEvent(event_id), &event);

        env.storage()
            .instance()
            .set(&DataKey::NextAtnaEventId, &event_id.saturating_add(1));

        env.events().publish(
            (symbol_short!("ATNA"), symbol_short!("AUTO")),
            (event_id, event_type, actor),
        );
    }
}
