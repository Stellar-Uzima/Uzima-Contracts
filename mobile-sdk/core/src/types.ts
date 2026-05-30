/**
 * Core type definitions for Uzima SDK
 * Generated from contract schemas for type safety and IDE support
 * @module @uzima/sdk-core/types
 */

// ==================== Configuration Types ====================

/**
 * Configuration for initializing the Uzima SDK
 * @interface UzimaConfig
 * @property {string} apiEndpoint - The API endpoint URL for the backend service
 * @property {string} contractId - The contract ID on the Stellar network
 * @property {string} networkPassphrase - The network passphrase (e.g., "Test SDF Network")
 * @property {string} serverURL - The Soroban RPC server URL
 * @property {string} [encryptionKey] - Optional encryption key for data at rest
 * @property {boolean} offlineEnabled - Enable offline data synchronization
 * @property {boolean} notificationsEnabled - Enable push notifications
 * @property {boolean} biometricEnabled - Enable biometric authentication
 * @property {number} [requestTimeout] - HTTP request timeout in milliseconds
 * @property {boolean} cacheEnabled - Enable response caching
 * @property {number} [cacheTTL] - Cache time-to-live in milliseconds
 */
export interface UzimaConfig {
  apiEndpoint: string;
  contractId: string;
  networkPassphrase: string;
  serverURL: string;
  encryptionKey?: string;
  offlineEnabled: boolean;
  notificationsEnabled: boolean;
  biometricEnabled: boolean;
  requestTimeout?: number;
  cacheEnabled: boolean;
  cacheTTL?: number;
}

/**
 * Authentication credentials for SDK operations
 * @interface AuthCredentials
 * @property {string} publicKey - The user's public key (Stellar address)
 * @property {string} [secretKey] - Optional secret key for signing operations
 * @property {string} [sessionToken] - JWT or session token for API requests
 */
export interface AuthCredentials {
  publicKey: string;
  secretKey?: string;
  sessionToken?: string;
}

/**
 * Options for biometric authentication
 * @interface BiometricOptions
 * @property {boolean} enabled - Whether biometric authentication is enabled
 * @property {string} [biometryType] - Type of biometry: 'faces', 'fingerprint', 'iris', or 'voice'
 * @property {boolean} [fallbackToPin] - Fallback to PIN if biometric fails
 */
export interface BiometricOptions {
  enabled: boolean;
  biometryType?: 'faces' | 'fingerprint' | 'iris' | 'voice';
  fallbackToPin?: boolean;
}

// ==================== Medical Record Types ====================

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
  PROCEDURE = 'procedure'
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
  AES_256_GCM = 'aes-256-gcm'
}

/**
 * Encrypted data container
 * @interface EncryptedData
 * @property {string} ciphertext - Base64-encoded encrypted data
 * @property {string} nonce - Base64-encoded encryption nonce
 * @property {string} algorithm - The encryption algorithm used
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
 * @property {string} accessType - Type of access: 'read', 'write', or 'share'
 * @property {string} [ipAddress] - Optional IP address of the accessor
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
 * @property {string[]} [tags] - Optional tags for categorization
 * @property {boolean} [isTraditionalHealing] - Whether this is traditional healing record
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
 * @property {string} [signature] - Optional cryptographic signature
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

// ==================== Consent Types ====================

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
  EXPIRED = 'expired'
}

/**
 * Consent grant from patient to provider
 * @interface ConsentGrant
 * @property {string} id - Unique consent identifier
 * @property {string} patientId - Patient's Stellar address
 * @property {string} providerId - Healthcare provider's Stellar address
 * @property {number} grantedAt - Timestamp when consent was granted (Unix seconds)
 * @property {number} [revokedAt] - Timestamp when consent was revoked (Unix seconds)
 * @property {ConsentStatus} status - Current consent status
 * @property {string[]} [scope] - Optional data access scope (types of records allowed)
 * @property {number} [expiresAt] - Optional consent expiration timestamp (Unix seconds)
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

// ==================== Identity Types ====================

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
  FIDO2_ES256_2024 = 'Fido2Es2562024'
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
  CAPABILITY_DELEGATION = 'CapabilityDelegation'
}

/**
 * Identity document (Decentralized Identifier) per W3C DID spec
 * @interface IdentityDocument
 * @property {string} id - The DID identifier (e.g., "did:stellar:uzima:mainnet:GXXXXXXXXX")
 * @property {string[]} context - JSON-LD context URLs
 * @property {VerificationMethod[]} verificationMethods - Public key information
 * @property {VerificationRelationship[]} [authenticationMethods] - Auth key relationships
 * @property {VerificationRelationship[]} [assertionMethods] - Assertion key relationships
 * @property {ServiceEndpoint[]} [serviceEndpoints] - Service URLs
 * @property {number} created - Creation timestamp (Unix seconds)
 * @property {number} [updated] - Last update timestamp (Unix seconds)
 * @property {string} [proof] - Cryptographic proof
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

// ==================== Payment Types ====================

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
  DISPUTED = 'disputed'
}

/**
 * Pre-authorization status
 * @enum {string}
 */
export enum PreAuthStatus {
  /** Pre-authorization pending */
  PENDING = 'pending',
  /** Pre-authorization approved */
  APPROVED = 'approved',
  /** Pre-authorization denied */
  DENIED = 'denied',
  /** Pre-authorization expired */
  EXPIRED = 'expired'
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
 * @property {string} [serviceId] - Service identifier
 * @property {string} [policyId] - Insurance policy ID
 * @property {number} createdAt - Creation timestamp (Unix seconds)
 * @property {number} updatedAt - Last update timestamp (Unix seconds)
 * @property {number} [completedAt] - Completion timestamp (Unix seconds)
 * @property {string} [transactionHash] - Blockchain transaction hash
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

// ==================== Audit Types ====================

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
  CROSS_CHAIN_TRANSFER_COMPLETED = 'CrossChainTransferCompleted'
}

/**
 * Audit log entry for compliance and forensics
 * @interface AuditEntry
 * @property {string} id - Unique audit entry identifier
 * @property {string} actor - Address of the actor performing the action
 * @property {ActionType} action - Type of action performed
 * @property {string} [resource] - Resource identifier being acted upon
 * @property {string} [resourceType] - Type of resource
 * @property {string} [result] - Operation result (success/failure)
 * @property {string} [reason] - Reason for the action
 * @property {number} timestamp - Action timestamp (Unix seconds)
 * @property {string} [ipAddress] - IP address of the actor
 * @property {Record<string, string>} [metadata] - Additional context
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

// ==================== Synchronization Types ====================

/**
 * Sync record for offline operation
 * @interface SyncRecord
 * @property {string} id - Unique sync record identifier
 * @property {string} recordId - ID of the synced record
 * @property {string} operation - Operation type: 'create', 'update', or 'delete'
 * @property {number} timestamp - Operation timestamp (Unix seconds)
 * @property {boolean} synced - Whether the operation has been synced
 * @property {number} [syncedAt] - Sync completion timestamp (Unix seconds)
 * @property {MedicalRecord} data - The synced medical record
 */
export interface SyncRecord {
  id: string;
  recordId: string;
  operation: 'create' | 'update' | 'delete';
  timestamp: number;
  synced: boolean;
  syncedAt?: number;
  data: MedicalRecord;
}

// ==================== Notification Types ====================

/**
 * Push notification type enumeration
 * @enum {string}
 */
export enum NotificationType {
  /** Record access notification */
  RECORD_ACCESS = 'record_access',
  /** Record update notification */
  RECORD_UPDATE = 'record_update',
  /** Permission granted notification */
  PERMISSION_GRANTED = 'permission_granted',
  /** Permission revoked notification */
  PERMISSION_REVOKED = 'permission_revoked',
  /** Alert notification */
  ALERT = 'alert',
  /** Reminder notification */
  REMINDER = 'reminder'
}

/**
 * Push notification
 * @interface PushNotification
 * @property {string} id - Unique notification identifier
 * @property {NotificationType} type - Notification type
 * @property {string} title - Notification title
 * @property {string} body - Notification body text
 * @property {Record<string, any>} [data] - Custom notification data
 * @property {number} timestamp - Notification timestamp (Unix seconds)
 * @property {boolean} read - Whether the notification has been read
 */
export interface PushNotification {
  id: string;
  type: NotificationType;
  title: string;
  body: string;
  data?: Record<string, string | number | boolean>;
  timestamp: number;
  read: boolean;
}

// ==================== API Response Types ====================

/**
 * Generic API response wrapper
 * @interface APIResponse
 * @template T The type of the response data
 * @property {boolean} success - Whether the request was successful
 * @property {T} [data] - Response data
 * @property {APIError} [error] - Error information if request failed
 * @property {number} timestamp - Response timestamp (Unix seconds)
 * @property {string} requestId - Unique request identifier for tracking
 */
export interface APIResponse<T> {
  success: boolean;
  data?: T;
  error?: APIError;
  timestamp: number;
  requestId: string;
}

/**
 * API error response
 * @interface APIError
 * @property {string} code - Error code
 * @property {string} message - Human-readable error message
 * @property {Record<string, any>} [details] - Additional error details
 */
export interface APIError {
  code: string;
  message: string;
  details?: Record<string, string | number | boolean>;
}

// ==================== Offline Operation Types ====================

/**
 * Offline operation queued for later sync
 * @interface OfflineQueue
 * @property {string} id - Unique queue entry identifier
 * @property {OfflineOperation} operation - The queued operation
 * @property {number} timestamp - Queue timestamp (Unix seconds)
 * @property {number} retryCount - Number of retry attempts
 * @property {number} maxRetries - Maximum retry attempts
 */
export interface OfflineQueue {
  id: string;
  operation: OfflineOperation;
  timestamp: number;
  retryCount: number;
  maxRetries: number;
}

/**
 * Offline operation to be synced
 * @interface OfflineOperation
 * @property {string} type - Operation type: 'read', 'write', or 'sync'
 * @property {string} endpoint - API endpoint
 * @property {string} method - HTTP method
 * @property {Record<string, any>} [data] - Optional operation data
 */
export interface OfflineOperation {
  type: 'read' | 'write' | 'sync';
  endpoint: string;
  method: string;
  data?: Record<string, string | number | boolean>;
}

// ==================== Cache Types ====================

/**
 * Cached data entry with TTL
 * @interface CacheEntry
 * @template T The type of cached data
 * @property {T} data - The cached data
 * @property {number} ttl - Time-to-live in milliseconds
 * @property {number} expiresAt - Expiration timestamp (Unix milliseconds)
 */
export interface CacheEntry<T> {
  data: T;
  ttl: number;
  expiresAt: number;
}
