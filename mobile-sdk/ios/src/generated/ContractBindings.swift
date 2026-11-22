/**
 * Auto-generated contract bindings for the Uzima iOS SDK.
 *
 * DO NOT EDIT MANUALLY — run `node scripts/generate-sdk-types.mjs` instead.
 * Generator version: 1.0.0
 */

// ==================== Contract Enums ====================

public enum RecordType: String {
    case diagnosis = "diagnosis"
    case prescription = "prescription"
    case labResult = "lab_result"
    case imaging = "imaging"
    case consultation = "consultation"
    case vitalSigns = "vital_signs"
    case immunization = "immunization"
    case medicationHistory = "medication_history"
    case allergy = "allergy"
    case procedure = "procedure"
}

public enum EncryptionAlgorithm: String {
    case naclBox = "nacl-box"
    case naclSecretbox = "nacl-secretbox"
    case aes256Gcm = "aes-256-gcm"
}

public enum ConsentStatus: String {
    case active = "active"
    case revoked = "revoked"
    case pending = "pending"
    case expired = "expired"
}

public enum VerificationMethodType: String {
    case ed25519VerificationKey2020 = "Ed25519VerificationKey2020"
    case ecdsaSecp256k1VerifKey2019 = "EcdsaSecp256k1VerifKey2019"
    case x25519KeyAgreementKey2020 = "X25519KeyAgreementKey2020"
    case jsonWebKey2020 = "JsonWebKey2020"
    case fido2EdDsa2024 = "Fido2EdDsa2024"
    case fido2Es2562024 = "Fido2Es2562024"
}

public enum VerificationRelationship: String {
    case authentication = "Authentication"
    case assertionMethod = "AssertionMethod"
    case keyAgreement = "KeyAgreement"
    case capabilityInvocation = "CapabilityInvocation"
    case capabilityDelegation = "CapabilityDelegation"
}

public enum PaymentStatusEnum: String {
    case submitted = "submitted"
    case verified = "verified"
    case approved = "approved"
    case rejected = "rejected"
    case paid = "paid"
    case disputed = "disputed"
}

public enum ActionType: String {
    case dataRead = "DataRead"
    case dataWrite = "DataWrite"
    case dataDelete = "DataDelete"
    case dataExport = "DataExport"
    case permissionGrant = "PermissionGrant"
    case permissionRevoke = "PermissionRevoke"
    case roleAssign = "RoleAssign"
    case roleRevoke = "RoleRevoke"
    case recordCreate = "RecordCreate"
    case recordUpdate = "RecordUpdate"
    case recordArchive = "RecordArchive"
    case recordRestore = "RecordRestore"
    case authSuccess = "AuthSuccess"
    case authFailure = "AuthFailure"
    case authLogout = "AuthLogout"
    case tokenRefresh = "TokenRefresh"
    case crossChainTransferInit = "CrossChainTransferInit"
    case crossChainTransferCompleted = "CrossChainTransferCompleted"
}

// ==================== Contract Data Types ====================

public struct EncryptedData {
    public var ciphertext: String
    public var nonce: String
    public var algorithm: EncryptionAlgorithm

    public init(ciphertext: String, nonce: String, algorithm: EncryptionAlgorithm) {
        self.ciphertext = ciphertext
        self.nonce = nonce
        self.algorithm = algorithm
    }
}

public struct AccessLog {
    public var accessor: String
    public var accessTime: Int64
    public var accessType: String
    public var ipAddress: String?

    public init(accessor: String, accessTime: Int64, accessType: String, ipAddress: String? = nil) {
        self.accessor = accessor
        self.accessTime = accessTime
        self.accessType = accessType
        self.ipAddress = ipAddress
    }
}

public struct RecordMetadata {
    public var createdAt: Int64
    public var updatedAt: Int64
    public var accessLog: [AccessLog]
    public var tags: [String]?
    public var isTraditionalHealing: Bool?

    public init(createdAt: Int64, updatedAt: Int64, accessLog: [AccessLog], tags: [String]? = nil, isTraditionalHealing: Bool? = nil) {
        self.createdAt = createdAt
        self.updatedAt = updatedAt
        self.accessLog = accessLog
        self.tags = tags
        self.isTraditionalHealing = isTraditionalHealing
    }
}

public struct MedicalRecord {
    public var id: String
    public var patientId: String
    public var providerId: String
    public var recordType: RecordType
    public var data: EncryptedData
    public var metadata: RecordMetadata
    public var timestamp: Int64
    public var isEncrypted: Bool
    public var signature: String?

    public init(id: String, patientId: String, providerId: String, recordType: RecordType, data: EncryptedData, metadata: RecordMetadata, timestamp: Int64, isEncrypted: Bool, signature: String? = nil) {
        self.id = id
        self.patientId = patientId
        self.providerId = providerId
        self.recordType = recordType
        self.data = data
        self.metadata = metadata
        self.timestamp = timestamp
        self.isEncrypted = isEncrypted
        self.signature = signature
    }
}

public struct ConsentGrant {
    public var id: String
    public var patientId: String
    public var providerId: String
    public var grantedAt: Int64
    public var revokedAt: Int64?
    public var status: ConsentStatus
    public var scope: [String]?
    public var expiresAt: Int64?

    public init(id: String, patientId: String, providerId: String, grantedAt: Int64, revokedAt: Int64? = nil, status: ConsentStatus, scope: [String]? = nil, expiresAt: Int64? = nil) {
        self.id = id
        self.patientId = patientId
        self.providerId = providerId
        self.grantedAt = grantedAt
        self.revokedAt = revokedAt
        self.status = status
        self.scope = scope
        self.expiresAt = expiresAt
    }
}

public struct VerificationMethod {
    public var id: String
    public var methodType: VerificationMethodType
    public var controller: String
    public var publicKey: String
    public var isActive: Bool
    public var created: Int64
    public var lastRotated: Int64

    public init(id: String, methodType: VerificationMethodType, controller: String, publicKey: String, isActive: Bool, created: Int64, lastRotated: Int64) {
        self.id = id
        self.methodType = methodType
        self.controller = controller
        self.publicKey = publicKey
        self.isActive = isActive
        self.created = created
        self.lastRotated = lastRotated
    }
}

public struct ServiceEndpoint {
    public var id: String
    public var type: String
    public var url: String

    public init(id: String, type: String, url: String) {
        self.id = id
        self.type = type
        self.url = url
    }
}

public struct IdentityDocument {
    public var id: String
    public var context: [String]
    public var verificationMethods: [VerificationMethod]
    public var authenticationMethods: [VerificationRelationship]?
    public var assertionMethods: [VerificationRelationship]?
    public var serviceEndpoints: [ServiceEndpoint]?
    public var created: Int64
    public var updated: Int64?
    public var proof: String?

    public init(id: String, context: [String], verificationMethods: [VerificationMethod], authenticationMethods: [VerificationRelationship]? = nil, assertionMethods: [VerificationRelationship]? = nil, serviceEndpoints: [ServiceEndpoint]? = nil, created: Int64, updated: Int64? = nil, proof: String? = nil) {
        self.id = id
        self.context = context
        self.verificationMethods = verificationMethods
        self.authenticationMethods = authenticationMethods
        self.assertionMethods = assertionMethods
        self.serviceEndpoints = serviceEndpoints
        self.created = created
        self.updated = updated
        self.proof = proof
    }
}

public struct PaymentStatus {
    public var id: String
    public var patientId: String
    public var providerId: String
    public var amount: Int64
    public var currency: String
    public var status: PaymentStatusEnum
    public var serviceId: String?
    public var policyId: String?
    public var createdAt: Int64
    public var updatedAt: Int64
    public var completedAt: Int64?
    public var transactionHash: String?

    public init(id: String, patientId: String, providerId: String, amount: Int64, currency: String, status: PaymentStatusEnum, serviceId: String? = nil, policyId: String? = nil, createdAt: Int64, updatedAt: Int64, completedAt: Int64? = nil, transactionHash: String? = nil) {
        self.id = id
        self.patientId = patientId
        self.providerId = providerId
        self.amount = amount
        self.currency = currency
        self.status = status
        self.serviceId = serviceId
        self.policyId = policyId
        self.createdAt = createdAt
        self.updatedAt = updatedAt
        self.completedAt = completedAt
        self.transactionHash = transactionHash
    }
}

public struct AuditEntry {
    public var id: String
    public var actor: String
    public var action: ActionType
    public var resource: String?
    public var resourceType: String?
    public var result: String?
    public var reason: String?
    public var timestamp: Int64
    public var ipAddress: String?
    public var metadata: [String: String]?

    public init(id: String, actor: String, action: ActionType, resource: String? = nil, resourceType: String? = nil, result: String? = nil, reason: String? = nil, timestamp: Int64, ipAddress: String? = nil, metadata: [String: String]? = nil) {
        self.id = id
        self.actor = actor
        self.action = action
        self.resource = resource
        self.resourceType = resourceType
        self.result = result
        self.reason = reason
        self.timestamp = timestamp
        self.ipAddress = ipAddress
        self.metadata = metadata
    }
}

// ==================== Contract Client Bindings ====================

/// Generated client binding for the contract_monitoring public interface.
public protocol ContractMonitoringClient {
    /// Get dashboard snapshot
    func getDashboard() -> Any
    /// Initialize the monitoring contract
    func initialize(admin: String, alertConfig: Any)
    /// Record a successful function call
    func recordCall(caller: String, functionName: String, gasUsed: Int64)
    /// Record a failed function call
    func recordError(functionName: String)
}

/// Generated client binding for the identity_registry public interface.
public protocol IdentityRegistryClient {
    /// Initialize the identity registry
    func initialize(admin: String)
    /// Register a new identity
    func registerIdentity(identity: String, role: String, metadata: String)
    /// Revoke an identity
    func revokeIdentity(identity: String)
    /// Verify an identity exists and has role
    func verifyIdentity(identity: String, role: String) -> Bool
}

/// Generated client binding for the medical_records public interface.
public protocol MedicalRecordsClient {
    /// Create a new medical record
    func createRecord(patient: String, data: String, recordType: String) -> Int64
    /// Soft-delete a medical record
    func deleteRecord(patient: String, recordId: Int64)
    /// Grant a provider access to patient records
    func grantAccess(patient: String, provider: String)
    /// Initialize the contract with admin and RBAC
    func initialize(admin: String, rbacId: String)
    /// Read a medical record by ID
    func readRecord(patient: String, recordId: Int64) -> MedicalRecord
    /// Revoke provider access to patient records
    func revokeAccess(patient: String, provider: String)
    /// Update an existing medical record
    func updateRecord(patient: String, recordId: Int64, data: String)
}

/// Generated client binding for the patient_consent_management public interface.
public protocol PatientConsentManagementClient {
    /// Check if consent is active
    func checkConsent(patient: String, provider: String, dataType: String) -> Bool
    /// Grant consent for data access
    func grantConsent(patient: String, provider: String, dataType: String, expiryLedger: Int64)
    /// Initialize the consent contract
    func initialize(admin: String)
    /// Revoke previously granted consent
    func revokeConsent(patient: String, provider: String, dataType: String)
}

// ==================== Binding Coverage ====================

public let sdkContractInterfaceCoverage: [String: [String]] = [
    "contract_monitoring": ["get_dashboard", "initialize", "record_call", "record_error"],
    "identity_registry": ["initialize", "register_identity", "revoke_identity", "verify_identity"],
    "medical_records": ["create_record", "delete_record", "grant_access", "initialize", "read_record", "revoke_access", "update_record"],
    "patient_consent_management": ["check_consent", "grant_consent", "initialize", "revoke_consent"],
]
