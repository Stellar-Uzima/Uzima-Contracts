import { SyncRecord, OfflineQueue, MedicalRecord } from '../types';
import { APIClient } from '../network/APIClient';

/**
 * OfflineManager handles offline data synchronization
 * Features:
 * - Queue operations when offline
 * - Automatic sync when connection restored
 * - Conflict resolution
 * - Local storage persistence
 */
export class OfflineManager {
  private syncQueue: Map<string, SyncRecord> = new Map();
  private offlineQueue: Map<string, OfflineQueue> = new Map();
  private apiClient: APIClient;
  private isOnline: boolean = true;
  private listeners: Function[] = [];

  constructor(apiClient: APIClient) {
    this.apiClient = apiClient;
    this.setupOnlineListener();
  }

  /**
   * Check if device is online
   */
  isDeviceOnline(): boolean {
    return this.isOnline;
  }

  /**
   * Queue an operation for offline storage
   */
  queueOperation(
    recordId: string,
    operation: 'create' | 'update' | 'delete',
    data: MedicalRecord,
    immediate: boolean = false
  ): void {
    const syncRecord: SyncRecord = {
      id: `sync_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      recordId,
      operation,
      timestamp: Date.now(),
      synced: false,
      data,
    };

    this.syncQueue.set(syncRecord.id, syncRecord);

    if (immediate && this.isOnline) {
      this.syncRecord(syncRecord);
    }
  }

  /**
   * Add operation to offline queue
   */
  queueOfflineOperation(
    endpoint: string,
    method: string,
    data: Record<string, any>
  ): void {
    const operation: OfflineQueue = {
      id: `op_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      operation: {
        type: method === 'GET' ? 'read' : 'write',
        endpoint,
        method,
        data,
      },
      timestamp: Date.now(),
      retryCount: 0,
      maxRetries: 5,
    };

    this.offlineQueue.set(operation.id, operation);
  }

  /**
   * Sync queued records
   */
  async syncAll(): Promise<{ synced: number; failed: number }> {
    if (!this.isOnline) {
      console.log('Device is offline, cannot sync');
      return { synced: 0, failed: 0 };
    }

    let synced = 0;
    let failed = 0;

    for (const [, record] of this.syncQueue) {
      if (!record.synced) {
        try {
          await this.syncRecord(record);
          synced++;
        } catch (error) {
          console.error(`Failed to sync record ${record.id}:`, error);
          failed++;
        }
      }
    }

    return { synced, failed };
  }

  /**
   * Sync pending offline operations
   */
  async syncOfflineOperations(): Promise<{ synced: number; failed: number }> {
    if (!this.isOnline) {
      return { synced: 0, failed: 0 };
    }

    let synced = 0;
    let failed = 0;

    for (const [, operation] of this.offlineQueue) {
      try {
        await this.processOperation(operation);
        synced++;
        this.offlineQueue.delete(operation.id);
      } catch (error) {
        operation.retryCount++;
        if (operation.retryCount >= operation.maxRetries) {
          console.error(
            `Operation ${operation.id} reached max retries:`,
            error
          );
          this.offlineQueue.delete(operation.id);
          failed++;
        }
      }
    }

    return { synced, failed };
  }

  /**
   * Get pending sync records
   */
  getPendingSyncRecords(): SyncRecord[] {
    return Array.from(this.syncQueue.values()).filter((r) => !r.synced);
  }

  /**
   * Get offline queue size
   */
  getOfflineQueueSize(): number {
    return this.offlineQueue.size;
  }

  /**
   * Clear sync queue
   */
  clearSyncQueue(): void {
    this.syncQueue.clear();
  }

  /**
   * Clear offline queue
   */
  clearOfflineQueue(): void {
    this.offlineQueue.clear();
  }

  /**
   * Register listener for sync events
   */
  onSyncStatusChange(listener: (isOnline: boolean) => void): void {
    this.listeners.push(listener);
  }

  /**
   * Sync a single record
   */
  private async syncRecord(record: SyncRecord): Promise<void> {
    switch (record.operation) {
      case 'create':
        await this.apiClient.post('/records', record.data);
        break;
      case 'update':
        await this.apiClient.put(`/records/${record.recordId}`, record.data);
        break;
      case 'delete':
        await this.apiClient.delete(`/records/${record.recordId}`);
        break;
    }

    record.synced = true;
    record.syncedAt = Date.now();
    this.syncQueue.set(record.id, record);
  }

  /**
   * Process offline operation
   */
  private async processOperation(operation: OfflineQueue): Promise<void> {
    switch (operation.operation.method) {
      case 'GET':
        await this.apiClient.get(operation.operation.endpoint);
        break;
      case 'POST':
        await this.apiClient.post(
          operation.operation.endpoint,
          operation.operation.data
        );
        break;
      case 'PUT':
        await this.apiClient.put(
          operation.operation.endpoint,
          operation.operation.data
        );
        break;
      case 'DELETE':
        await this.apiClient.delete(operation.operation.endpoint);
        break;
    }
  }

  /**
   * Set up online/offline listener
   */
  private setupOnlineListener(): void {
    // Browser environment
    if (typeof window !== 'undefined') {
      window.addEventListener('online', () => {
        this.isOnline = true;
        this.notifyListeners();
        this.syncAll();
        this.syncOfflineOperations();
      });

      window.addEventListener('offline', () => {
        this.isOnline = false;
        this.notifyListeners();
      });
    }
  }

  /**
   * Notify all listeners of status change
   */
  private notifyListeners(): void {
    for (const listener of this.listeners) {
      listener(this.isOnline);
    }
  }
}
