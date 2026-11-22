/**
 * Auto-generated contract bindings for the Uzima Android SDK.
 *
 * DO NOT EDIT MANUALLY — run `node scripts/generate-sdk-types.mjs` instead.
 * Generator version: 1.0.0
 */

package com.uzima.sdk.generated

// ==================== Contract Enums ====================

enum class RecordType(val value: String) {
    DIAGNOSIS("diagnosis"),
    PRESCRIPTION("prescription"),
    LAB_RESULT("lab_result"),
    IMAGING("imaging"),
    CONSULTATION("consultation"),
    VITAL_SIGNS("vital_signs"),
    IMMUNIZATION("immunization"),
    MEDICATION_HISTORY("medication_history"),
    ALLERGY("allergy"),
    PROCEDURE("procedure"),
}

enum class EncryptionAlgorithm(val value: String) {
    NACL_BOX("nacl-box"),
    NACL_SECRETBOX("nacl-secretbox"),
    AES_256_GCM("aes-256-gcm"),
}

enum class ConsentStatus(val value: String) {
    ACTIVE("active"),
    REVOKED("revoked"),
    PENDING("pending"),
    EXPIRED("expired"),
}

enum class VerificationMethodType(val value: String) {
    ED25519_VERIFICATION_KEY_2020("Ed25519VerificationKey2020"),
    ECDSA_SECP256K1_VERIF_KEY_2019("EcdsaSecp256k1VerifKey2019"),
    X25519_KEY_AGREEMENT_KEY_2020("X25519KeyAgreementKey2020"),
    JSON_WEB_KEY_2020("JsonWebKey2020"),
    FIDO2_ED_DSA_2024("Fido2EdDsa2024"),
    FIDO2_ES256_2024("Fido2Es2562024"),
}

enum class VerificationRelationship(val value: String) {
    AUTHENTICATION("Authentication"),
    ASSERTION_METHOD("AssertionMethod"),
    KEY_AGREEMENT("KeyAgreement"),
    CAPABILITY_INVOCATION("CapabilityInvocation"),
    CAPABILITY_DELEGATION("CapabilityDelegation"),
}

enum class PaymentStatusEnum(val value: String) {
    SUBMITTED("submitted"),
    VERIFIED("verified"),
    APPROVED("approved"),
    REJECTED("rejected"),
    PAID("paid"),
    DISPUTED("disputed"),
}

enum class ActionType(val value: String) {
    DATA_READ("DataRead"),
    DATA_WRITE("DataWrite"),
    DATA_DELETE("DataDelete"),
    DATA_EXPORT("DataExport"),
    PERMISSION_GRANT("PermissionGrant"),
    PERMISSION_REVOKE("PermissionRevoke"),
    ROLE_ASSIGN("RoleAssign"),
    ROLE_REVOKE("RoleRevoke"),
    RECORD_CREATE("RecordCreate"),
    RECORD_UPDATE("RecordUpdate"),
    RECORD_ARCHIVE("RecordArchive"),
    RECORD_RESTORE("RecordRestore"),
    AUTH_SUCCESS("AuthSuccess"),
    AUTH_FAILURE("AuthFailure"),
    AUTH_LOGOUT("AuthLogout"),
    TOKEN_REFRESH("TokenRefresh"),
    CROSS_CHAIN_TRANSFER_INIT("CrossChainTransferInit"),
    CROSS_CHAIN_TRANSFER_COMPLETED("CrossChainTransferCompleted"),
}

// ==================== Contract Data Classes ====================

data class EncryptedData(
    val ciphertext: String,
    val nonce: String,
    val algorithm: EncryptionAlgorithm,
)

data class AccessLog(
    val accessor: String,
    val accessTime: Long,
    val accessType: String,
    val ipAddress: String? = null,
)

data class RecordMetadata(
    val createdAt: Long,
    val updatedAt: Long,
    val accessLog: List<AccessLog>,
    val tags: List<String>? = null,
    val isTraditionalHealing: Boolean? = null,
)

data class MedicalRecord(
    val id: String,
    val patientId: String,
    val providerId: String,
    val recordType: RecordType,
    val data: EncryptedData,
    val metadata: RecordMetadata,
    val timestamp: Long,
    val isEncrypted: Boolean,
    val signature: String? = null,
)

data class ConsentGrant(
    val id: String,
    val patientId: String,
    val providerId: String,
    val grantedAt: Long,
    val revokedAt: Long? = null,
    val status: ConsentStatus,
    val scope: List<String>? = null,
    val expiresAt: Long? = null,
)

data class VerificationMethod(
    val id: String,
    val methodType: VerificationMethodType,
    val controller: String,
    val publicKey: String,
    val isActive: Boolean,
    val created: Long,
    val lastRotated: Long,
)

data class ServiceEndpoint(
    val id: String,
    val type: String,
    val url: String,
)

data class IdentityDocument(
    val id: String,
    val context: List<String>,
    val verificationMethods: List<VerificationMethod>,
    val authenticationMethods: List<VerificationRelationship>? = null,
    val assertionMethods: List<VerificationRelationship>? = null,
    val serviceEndpoints: List<ServiceEndpoint>? = null,
    val created: Long,
    val updated: Long? = null,
    val proof: String? = null,
)

data class PaymentStatus(
    val id: String,
    val patientId: String,
    val providerId: String,
    val amount: Long,
    val currency: String,
    val status: PaymentStatusEnum,
    val serviceId: String? = null,
    val policyId: String? = null,
    val createdAt: Long,
    val updatedAt: Long,
    val completedAt: Long? = null,
    val transactionHash: String? = null,
)

data class AuditEntry(
    val id: String,
    val actor: String,
    val action: ActionType,
    val resource: String? = null,
    val resourceType: String? = null,
    val result: String? = null,
    val reason: String? = null,
    val timestamp: Long,
    val ipAddress: String? = null,
    val metadata: Map<String, String>? = null,
)

// ==================== Contract Client Bindings ====================

/**
 * Generated client binding for the contract_monitoring public interface.
 */
interface ContractMonitoringClient {
    /** Get dashboard snapshot */
    fun getDashboard(): Any
    /** Initialize the monitoring contract */
    fun initialize(admin: String, alertConfig: Any): Unit
    /** Record a successful function call */
    fun recordCall(caller: String, functionName: String, gasUsed: Long): Unit
    /** Record a failed function call */
    fun recordError(functionName: String): Unit
}

/**
 * Generated client binding for the identity_registry public interface.
 */
interface IdentityRegistryClient {
    /** Initialize the identity registry */
    fun initialize(admin: String): Unit
    /** Register a new identity */
    fun registerIdentity(identity: String, role: String, metadata: String): Unit
    /** Revoke an identity */
    fun revokeIdentity(identity: String): Unit
    /** Verify an identity exists and has role */
    fun verifyIdentity(identity: String, role: String): Boolean
}

/**
 * Generated client binding for the medical_records public interface.
 */
interface MedicalRecordsClient {
    /** Create a new medical record */
    fun createRecord(patient: String, data: String, recordType: String): Long
    /** Soft-delete a medical record */
    fun deleteRecord(patient: String, recordId: Long): Unit
    /** Grant a provider access to patient records */
    fun grantAccess(patient: String, provider: String): Unit
    /** Initialize the contract with admin and RBAC */
    fun initialize(admin: String, rbacId: String): Unit
    /** Read a medical record by ID */
    fun readRecord(patient: String, recordId: Long): MedicalRecord
    /** Revoke provider access to patient records */
    fun revokeAccess(patient: String, provider: String): Unit
    /** Update an existing medical record */
    fun updateRecord(patient: String, recordId: Long, data: String): Unit
}

/**
 * Generated client binding for the patient_consent_management public interface.
 */
interface PatientConsentManagementClient {
    /** Check if consent is active */
    fun checkConsent(patient: String, provider: String, dataType: String): Boolean
    /** Grant consent for data access */
    fun grantConsent(patient: String, provider: String, dataType: String, expiryLedger: Long): Unit
    /** Initialize the consent contract */
    fun initialize(admin: String): Unit
    /** Revoke previously granted consent */
    fun revokeConsent(patient: String, provider: String, dataType: String): Unit
}

// ==================== Binding Coverage ====================

val SDK_CONTRACT_INTERFACE_COVERAGE: Map<String, List<String>> = mapOf(
    "contract_monitoring" to listOf("get_dashboard", "initialize", "record_call", "record_error"),
    "identity_registry" to listOf("initialize", "register_identity", "revoke_identity", "verify_identity"),
    "medical_records" to listOf("create_record", "delete_record", "grant_access", "initialize", "read_record", "revoke_access", "update_record"),
    "patient_consent_management" to listOf("check_consent", "grant_consent", "initialize", "revoke_consent"),
)
