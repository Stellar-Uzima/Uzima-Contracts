/**
 * Core type definitions for Uzima SDK
 * Contract bindings are generated from on-chain schemas — see generated/contract-bindings.ts
 * @module @uzima/sdk-core/types
 */

export * from './generated/contract-bindings';

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

// ==================== Payment Types ====================

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
