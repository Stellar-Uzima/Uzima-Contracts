/**
 * Core type definitions for Uzima SDK
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

export interface AuthCredentials {
  publicKey: string;
  secretKey?: string;
  sessionToken?: string;
}

export interface BiometricOptions {
  enabled: boolean;
  biometryType?: 'faces' | 'fingerprint' | 'iris' | 'voice';
  fallbackToPin?: boolean;
}

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

export enum RecordType {
  DIAGNOSIS = 'diagnosis',
  PRESCRIPTION = 'prescription',
  LAB_RESULT = 'lab_result',
  IMAGING = 'imaging',
  CONSULTATION = 'consultation',
  VITAL_SIGNS = 'vital_signs',
  IMMUNIZATION = 'immunization',
  MEDICATION_HISTORY = 'medication_history',
  ALLERGY = 'allergy',
  PROCEDURE = 'procedure'
}

export interface EncryptedData {
  ciphertext: string;
  nonce: string;
  algorithm: 'nacl-box' | 'nacl-secretbox' | 'aes-256-gcm';
}

export interface RecordMetadata {
  createdAt: number;
  updatedAt: number;
  accessLog: AccessLog[];
  tags?: string[];
  isTraditionalHealing?: boolean;
}

export interface AccessLog {
  accessor: string;
  accessTime: number;
  accessType: 'read' | 'write' | 'share';
  ipAddress?: string;
}

export interface SyncRecord {
  id: string;
  recordId: string;
  operation: 'create' | 'update' | 'delete';
  timestamp: number;
  synced: boolean;
  syncedAt?: number;
  data: MedicalRecord;
}

export interface PushNotification {
  id: string;
  type: NotificationType;
  title: string;
  body: string;
  data?: Record<string, any>;
  timestamp: number;
  read: boolean;
}

export enum NotificationType {
  RECORD_ACCESS = 'record_access',
  RECORD_UPDATE = 'record_update',
  PERMISSION_GRANTED = 'permission_granted',
  PERMISSION_REVOKED = 'permission_revoked',
  ALERT = 'alert',
  REMINDER = 'reminder'
}

export interface APIResponse<T> {
  success: boolean;
  data?: T;
  error?: APIError;
  timestamp: number;
  requestId: string;
}

export interface APIError {
  code: string;
  message: string;
  details?: Record<string, any>;
}

export interface OfflineQueue {
  id: string;
  operation: OfflineOperation;
  timestamp: number;
  retryCount: number;
  maxRetries: number;
}

export interface OfflineOperation {
  type: 'read' | 'write' | 'sync';
  endpoint: string;
  method: string;
  data?: Record<string, any>;
}

export interface CacheEntry<T> {
  data: T;
  ttl: number;
  expiresAt: number;
}
