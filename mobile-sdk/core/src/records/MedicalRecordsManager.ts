import { MedicalRecord, RecordType, APIResponse, AccessLog } from '../types';
import { APIClient } from '../network/APIClient';
import { EncryptionManager } from '../crypto/EncryptionManager';
import { Configuration } from '../config/UzimaConfig';

/**
 * MedicalRecordsManager handles all medical record operations
 * Features:
 * - Create, read, update, delete operations
 * - Encryption/decryption
 * - Access control and logging
 * - Efficient caching
 */
export class MedicalRecordsManager {
  private apiClient: APIClient;
  private recordCache: Map<string, MedicalRecord> = new Map();
  private cacheTTL: number;

  constructor(apiClient: APIClient, cacheTTL: number = 300000) {
    this.apiClient = apiClient;
    this.cacheTTL = cacheTTL;
  }

  /**
   * Create a new medical record
   */
  async createRecord(
    patientId: string,
    providerId: string,
    recordType: RecordType,
    data: Record<string, any>,
    encryptionKey: string
  ): Promise<MedicalRecord> {
    const encryptedData = EncryptionManager.encryptWithSharedSecret(
      JSON.stringify(data),
      encryptionKey
    );

    const record: MedicalRecord = {
      id: this.generateId(),
      patientId,
      providerId,
      recordType,
      data: encryptedData,
      metadata: {
        createdAt: Date.now(),
        updatedAt: Date.now(),
        accessLog: [
          {
            accessor: providerId,
            accessTime: Date.now(),
            accessType: 'write',
          },
        ],
      },
      timestamp: Date.now(),
      isEncrypted: true,
    };

    const response = await this.apiClient.post<MedicalRecord>(
      '/records',
      record
    );

    if (response.data) {
      this.recordCache.set(record.id, response.data);
    }

    return response.data!;
  }

  /**
   * Get a medical record by ID
   */
  async getRecord(
    recordId: string,
    encryptionKey: string
  ): Promise<MedicalRecord> {
    // Check cache first
    if (this.recordCache.has(recordId)) {
      const cached = this.recordCache.get(recordId);
      if (cached) {
        return cached;
      }
    }

    const response = await this.apiClient.get<MedicalRecord>(
      `/records/${recordId}`
    );

    if (!response.data) {
      throw new Error('Record not found');
    }

    const record = response.data;

    // Decrypt data
    if (record.isEncrypted && record.data) {
      try {
        const decrypted = EncryptionManager.decryptWithSharedSecret(
          record.data,
          encryptionKey
        );
        record.data = {
          ...record.data,
          ciphertext: decrypted,
        };
      } catch (error) {
        console.error('Decryption failed:', error);
        // Return encrypted record if decryption fails
      }
    }

    // Cache the record
    this.recordCache.set(recordId, record);

    // Log access
    await this.logAccess(recordId, 'read');

    return record;
  }

  /**
   * Get records for a patient
   */
  async getPatientRecords(
    patientId: string,
    encryptionKey: string,
    limit: number = 100,
    offset: number = 0
  ): Promise<MedicalRecord[]> {
    const response = await this.apiClient.get<MedicalRecord[]>(
      `/patients/${patientId}/records?limit=${limit}&offset=${offset}`
    );

    if (!response.data) {
      return [];
    }

    // Decrypt all records
    const records = response.data.map((record) => {
      if (record.isEncrypted && record.data) {
        try {
          const decrypted = EncryptionManager.decryptWithSharedSecret(
            record.data,
            encryptionKey
          );
          record.data = {
            ...record.data,
            ciphertext: decrypted,
          };
        } catch (error) {
          console.error(`Decryption failed for record ${record.id}:`, error);
        }
      }
      return record;
    });

    return records;
  }

  /**
   * Update a medical record
   */
  async updateRecord(
    recordId: string,
    updates: Partial<MedicalRecord>,
    encryptionKey: string
  ): Promise<MedicalRecord> {
    // Fetch current record
    const currentRecord = await this.getRecord(recordId, encryptionKey);

    // Prepare update
    const updated: Partial<MedicalRecord> = {
      ...currentRecord,
      ...updates,
      metadata: {
        ...currentRecord.metadata,
        updatedAt: Date.now(),
        accessLog: [
          ...currentRecord.metadata.accessLog,
          {
            accessor: 'current-user', // This would be set by the platform
            accessTime: Date.now(),
            accessType: 'write',
          },
        ],
      },
    };

    const response = await this.apiClient.put<MedicalRecord>(
      `/records/${recordId}`,
      updated
    );

    if (response.data) {
      this.recordCache.delete(recordId);
    }

    return response.data!;
  }

  /**
   * Delete a medical record
   */
  async deleteRecord(recordId: string): Promise<boolean> {
    await this.apiClient.delete(`/records/${recordId}`);
    this.recordCache.delete(recordId);
    return true;
  }

  /**
   * Search records by type and date range
   */
  async searchRecords(
    patientId: string,
    filters: {
      recordType?: RecordType;
      startDate?: number;
      endDate?: number;
      tags?: string[];
    },
    encryptionKey: string
  ): Promise<MedicalRecord[]> {
    const queryParams = new URLSearchParams();
    queryParams.append('patientId', patientId);

    if (filters.recordType) {
      queryParams.append('recordType', filters.recordType);
    }
    if (filters.startDate) {
      queryParams.append('startDate', filters.startDate.toString());
    }
    if (filters.endDate) {
      queryParams.append('endDate', filters.endDate.toString());
    }
    if (filters.tags) {
      queryParams.append('tags', filters.tags.join(','));
    }

    const response = await this.apiClient.get<MedicalRecord[]>(
      `/records/search?${queryParams.toString()}`
    );

    if (!response.data) {
      return [];
    }

    // Decrypt all records
    return response.data.map((record) => {
      if (record.isEncrypted && record.data) {
        try {
          const decrypted = EncryptionManager.decryptWithSharedSecret(
            record.data,
            encryptionKey
          );
          record.data = {
            ...record.data,
            ciphertext: decrypted,
          };
        } catch (error) {
          console.error(`Decryption failed for record ${record.id}:`, error);
        }
      }
      return record;
    });
  }

  /**
   * Share record with another user
   */
  async shareRecord(
    recordId: string,
    targetPublicKey: string,
    accessType: 'read' | 'write' = 'read'
  ): Promise<boolean> {
    await this.apiClient.post(`/records/${recordId}/share`, {
      targetPublicKey,
      accessType,
      sharedAt: Date.now(),
    });

    this.recordCache.delete(recordId);
    return true;
  }

  /**
   * Revoke access to record
   */
  async revokeAccess(recordId: string, accessor: string): Promise<boolean> {
    await this.apiClient.post(`/records/${recordId}/revoke`, {
      accessor,
      revokedAt: Date.now(),
    });

    this.recordCache.delete(recordId);
    return true;
  }

  /**
   * Get access log for a record
   */
  async getAccessLog(recordId: string): Promise<AccessLog[]> {
    const response = await this.apiClient.get<AccessLog[]>(
      `/records/${recordId}/access-log`
    );
    return response.data || [];
  }

  /**
   * Log access to a record
   */
  private async logAccess(recordId: string, accessType: 'read' | 'write' | 'share'): Promise<void> {
    try {
      await this.apiClient.post(`/records/${recordId}/access-log`, {
        accessType,
        timestamp: Date.now(),
      });
    } catch (error) {
      console.error('Failed to log access:', error);
    }
  }

  /**
   * Get record statistics
   */
  async getRecordStats(patientId: string): Promise<{
    totalRecords: number;
    recordsByType: Record<string, number>;
    lastUpdated: number;
  }> {
    const response = await this.apiClient.get(
      `/patients/${patientId}/statistics`
    );
    return response.data || { totalRecords: 0, recordsByType: {}, lastUpdated: 0 };
  }

  /**
   * Clear cache
   */
  clearCache(): void {
    this.recordCache.clear();
  }

  /**
   * Generate unique record ID
   */
  private generateId(): string {
    return `rec_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
}
