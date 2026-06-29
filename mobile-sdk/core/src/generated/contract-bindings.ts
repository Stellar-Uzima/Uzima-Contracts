/**
 * Auto-generated contract bindings for the Uzima mobile SDK.
 *
 * DO NOT EDIT MANUALLY — run `node scripts/generate-sdk-types.mjs` instead.
 * Generator version: 1.0.0
 * @module @uzima/sdk-core/generated
 */

// ==================== Contract Enums ====================

/**
 * Enumeration of medical record types
 * @enum {string}
 */
export enum RecordType {
  /** Diagnosis record */
  DIAGNOSIS = 'diagnosis',
  /** Prescription record */
  PRESCRIPTION = 'prescription',
  /** Laboratory test result */
  LAB_RESULT = 'lab_result',
  /** Medical imaging record */
  IMAGING = 'imaging',
  /** Consultation notes */
  CONSULTATION = 'consultation',
  /** Vital signs measurement */
  VITAL_SIGNS = 'vital_signs',
  /** Immunization record */
  IMMUNIZATION = 'immunization',
  /** Medication history */
  MEDICATION_HISTORY = 'medication_history',
  /** Allergy information */
  ALLERGY = 'allergy',
  /** Medical procedure record */
  PROCEDURE = 'procedure',
}

/**
 * Encryption algorithm used for medical data
 * @enum {string}
 */
export enum EncryptionAlgorithm {
  /** NaCl box (public-key encryption) */
  NACL_BOX = 'nacl-box',
  /** NaCl secretbox (secret-key encryption) */
  NACL_SECRETBOX = 'nacl-secretbox',
  /** AES-256 GCM */
  AES_256_GCM = 'aes-256-gcm',
}

/**
 * Consent grant status
 * @enum {string}
 */
export enum ConsentStatus {
  /** Consent is active and valid */
  ACTIVE = 'active',
  /** Consent has been revoked */
  REVOKED = 'revoked',
  /** Consent is pending approval */
  PENDING = 'pending',
  /** Consent has expired */
  EXPIRED = 'expired',
}

/**
 * Verification method types per W3C DID specification
 * @enum {string}
 */
export enum VerificationMethodType {
  /** Ed25519 Verification Key (2020) */
  ED25519_VERIFICATION_KEY_2020 = 'Ed25519VerificationKey2020',
  /** ECDSA Secp256k1 Verification Key (2019) */
  ECDSA_SECP256K1_VERIF_KEY_2019 = 'EcdsaSecp256k1VerifKey2019',
  /** X25519 Key Agreement Key (2020) */
  X25519_KEY_AGREEMENT_KEY_2020 = 'X25519KeyAgreementKey2020',
  /** JSON Web Key (2020) */
  JSON_WEB_KEY_2020 = 'JsonWebKey2020',
  /** FIDO2 EdDSA Key (2024) */
  FIDO2_ED_DSA_2024 = 'Fido2EdDsa2024',
  /** FIDO2 ES256 Key (2024) */
  FIDO2_ES256_2024 = 'Fido2Es2562024',
}

/**
 * Verification relationship types per W3C DID specification
 * @enum {string}
 */
export enum VerificationRelationship {
  /** Authentication relationship */
  AUTHENTICATION = 'Authentication',
  /** Assertion method relationship */
  ASSERTION_METHOD = 'AssertionMethod',
  /** Key agreement relationship */
  KEY_AGREEMENT = 'KeyAgreement',
  /** Capability invocation relationship */
  CAPABILITY_INVOCATION = 'CapabilityInvocation',
  /** Capability delegation relationship */
  CAPABILITY_DELEGATION = 'CapabilityDelegation',
}

/**
 * Payment/claim status enumeration
 * @enum {string}
 */
export enum PaymentStatusEnum {
  /** Payment claim submitted */
  SUBMITTED = 'submitted',
  /** Payment claim verified */
  VERIFIED = 'verified',
  /** Payment claim approved */
  APPROVED = 'approved',
  /** Payment claim rejected */
  REJECTED = 'rejected',
  /** Payment completed */
  PAID = 'paid',
  /** Payment disputed */
  DISPUTED = 'disputed',
}

/**
 * Audit action types
 * @enum {string}
 */
export enum ActionType {
  /** Data read operation */
  DATA_READ = 'DataRead',
  /** Data write operation */
  DATA_WRITE = 'DataWrite',
  /** Data delete operation */
  DATA_DELETE = 'DataDelete',
  /** Data export operation */
  DATA_EXPORT = 'DataExport',
  /** Permission grant */
  PERMISSION_GRANT = 'PermissionGrant',
  /** Permission revoke */
  PERMISSION_REVOKE = 'PermissionRevoke',
  /** Role assignment */
  ROLE_ASSIGN = 'RoleAssign',
  /** Role revocation */
  ROLE_REVOKE = 'RoleRevoke',
  /** Record creation */
  RECORD_CREATE = 'RecordCreate',
  /** Record update */
  RECORD_UPDATE = 'RecordUpdate',
  /** Record archive */
  RECORD_ARCHIVE = 'RecordArchive',
  /** Record restore */
  RECORD_RESTORE = 'RecordRestore',
  /** Authentication success */
  AUTH_SUCCESS = 'AuthSuccess',
  /** Authentication failure */
  AUTH_FAILURE = 'AuthFailure',
  /** User logout */
  AUTH_LOGOUT = 'AuthLogout',
  /** Token refresh */
  TOKEN_REFRESH = 'TokenRefresh',
  /** Cross-chain transfer initiated */
  CROSS_CHAIN_TRANSFER_INIT = 'CrossChainTransferInit',
  /** Cross-chain transfer completed */
  CROSS_CHAIN_TRANSFER_COMPLETED = 'CrossChainTransferCompleted',
}

// ==================== Contract Interfaces ====================

/**
 * Encrypted data container
 * @interface EncryptedData
 * @property {string} ciphertext - Base64-encoded encrypted data
 * @property {string} nonce - Base64-encoded encryption nonce
 * @property {EncryptionAlgorithm} algorithm - The encryption algorithm used
 */
export interface EncryptedData {
  ciphertext: string;
  nonce: string;
  algorithm: EncryptionAlgorithm;
}

/**
 * Access log entry for audit trail
 * @interface AccessLog
 * @property {string} accessor - The address of who accessed the record
 * @property {number} accessTime - Timestamp of access (Unix seconds)
 * @property {'read' | 'write' | 'share'} accessType - Type of access
 * @property {string} [optional] ipAddress - Optional IP address of the accessor
 */
export interface AccessLog {
  accessor: string;
  accessTime: number;
  accessType: 'read' | 'write' | 'share';
  ipAddress?: string;
}

/**
 * Metadata for medical records
 * @interface RecordMetadata
 * @property {number} createdAt - Creation timestamp (Unix seconds)
 * @property {number} updatedAt - Last update timestamp (Unix seconds)
 * @property {AccessLog[]} accessLog - Complete access history
 * @property {string[]} [optional] tags - Optional tags for categorization
 * @property {boolean} [optional] isTraditionalHealing - Whether this is traditional healing record
 */
export interface RecordMetadata {
  createdAt: number;
  updatedAt: number;
  accessLog: AccessLog[];
  tags?: string[];
  isTraditionalHealing?: boolean;
}

/**
 * Medical record structure matching contract schema
 * @interface MedicalRecord
 * @property {string} id - Unique record identifier
 * @property {string} patientId - Patient's Stellar address
 * @property {string} providerId - Healthcare provider's Stellar address
 * @property {RecordType} recordType - Type of medical record
 * @property {EncryptedData} data - Encrypted record content
 * @property {RecordMetadata} metadata - Record metadata and access log
 * @property {number} timestamp - Record creation timestamp (Unix seconds)
 * @property {boolean} isEncrypted - Whether the data is encrypted
 * @property {string} [optional] signature - Optional cryptographic signature
 */
export interface MedicalRecord {
  id: string;
  patientId: string;
  providerId: string;
  recordType: RecordType;
  data: EncryptedData;
  metadata: RecordMetadata;
  timestamp: number;
  isEncrypted: boolean;
  signature?: string;
}

/**
 * Consent grant from patient to provider
 * @interface ConsentGrant
 * @property {string} id - Unique consent identifier
 * @property {string} patientId - Patient's Stellar address
 * @property {string} providerId - Healthcare provider's Stellar address
 * @property {number} grantedAt - Timestamp when consent was granted (Unix seconds)
 * @property {number} [optional] revokedAt - Timestamp when consent was revoked (Unix seconds)
 * @property {ConsentStatus} status - Current consent status
 * @property {string[]} [optional] scope - Optional data access scope
 * @property {number} [optional] expiresAt - Optional consent expiration timestamp (Unix seconds)
 */
export interface ConsentGrant {
  id: string;
  patientId: string;
  providerId: string;
  grantedAt: number;
  revokedAt?: number;
  status: ConsentStatus;
  scope?: string[];
  expiresAt?: number;
}

/**
 * Verification method (public key) for W3C DID
 * @interface VerificationMethod
 * @property {string} id - Fragment identifier
 * @property {VerificationMethodType} methodType - Type of verification method
 * @property {string} controller - Controller address
 * @property {string} publicKey - Base64-encoded public key
 * @property {boolean} isActive - Whether this key is active
 * @property {number} created - Creation timestamp (Unix seconds)
 * @property {number} lastRotated - Last rotation timestamp (Unix seconds, 0 if never)
 */
export interface VerificationMethod {
  id: string;
  methodType: VerificationMethodType;
  controller: string;
  publicKey: string;
  isActive: boolean;
  created: number;
  lastRotated: number;
}

/**
 * Service endpoint for W3C DID
 * @interface ServiceEndpoint
 * @property {string} id - Service identifier
 * @property {string} type - Service type
 * @property {string} url - Service endpoint URL
 */
export interface ServiceEndpoint {
  id: string;
  type: string;
  url: string;
}

/**
 * Identity document (Decentralized Identifier) per W3C DID spec
 * @interface IdentityDocument
 * @property {string} id - The DID identifier
 * @property {string[]} context - JSON-LD context URLs
 * @property {VerificationMethod[]} verificationMethods - Public key information
 * @property {VerificationRelationship[]} [optional] authenticationMethods - Auth key relationships
 * @property {VerificationRelationship[]} [optional] assertionMethods - Assertion key relationships
 * @property {ServiceEndpoint[]} [optional] serviceEndpoints - Service URLs
 * @property {number} created - Creation timestamp (Unix seconds)
 * @property {number} [optional] updated - Last update timestamp (Unix seconds)
 * @property {string} [optional] proof - Cryptographic proof
 */
export interface IdentityDocument {
  id: string;
  context: string[];
  verificationMethods: VerificationMethod[];
  authenticationMethods?: VerificationRelationship[];
  assertionMethods?: VerificationRelationship[];
  serviceEndpoints?: ServiceEndpoint[];
  created: number;
  updated?: number;
  proof?: string;
}

/**
 * Payment status for healthcare claims and payments
 * @interface PaymentStatus
 * @property {string} id - Unique payment identifier
 * @property {string} patientId - Patient's Stellar address
 * @property {string} providerId - Provider's Stellar address
 * @property {number} amount - Payment amount in smallest unit
 * @property {string} currency - Currency code (e.g., "USDC")
 * @property {PaymentStatusEnum} status - Current payment status
 * @property {string} [optional] serviceId - Service identifier
 * @property {string} [optional] policyId - Insurance policy ID
 * @property {number} createdAt - Creation timestamp (Unix seconds)
 * @property {number} updatedAt - Last update timestamp (Unix seconds)
 * @property {number} [optional] completedAt - Completion timestamp (Unix seconds)
 * @property {string} [optional] transactionHash - Blockchain transaction hash
 */
export interface PaymentStatus {
  id: string;
  patientId: string;
  providerId: string;
  amount: number;
  currency: string;
  status: PaymentStatusEnum;
  serviceId?: string;
  policyId?: string;
  createdAt: number;
  updatedAt: number;
  completedAt?: number;
  transactionHash?: string;
}

/**
 * Audit log entry for compliance and forensics
 * @interface AuditEntry
 * @property {string} id - Unique audit entry identifier
 * @property {string} actor - Address of the actor performing the action
 * @property {ActionType} action - Type of action performed
 * @property {string} [optional] resource - Resource identifier being acted upon
 * @property {string} [optional] resourceType - Type of resource
 * @property {string} [optional] result - Operation result (success/failure)
 * @property {string} [optional] reason - Reason for the action
 * @property {number} timestamp - Action timestamp (Unix seconds)
 * @property {string} [optional] ipAddress - IP address of the actor
 * @property {Record<string, string>} [optional] metadata - Additional context
 */
export interface AuditEntry {
  id: string;
  actor: string;
  action: ActionType;
  resource?: string;
  resourceType?: string;
  result?: string;
  reason?: string;
  timestamp: number;
  ipAddress?: string;
  metadata?: Record<string, string>;
}
