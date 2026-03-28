import { Configuration, UzimaConfig as ConfigClass } from '../config/UzimaConfig';
import { APIClient } from '../network/APIClient';
import { AuthManager, BiometricAuth } from '../auth/AuthManager';
import { EncryptionManager } from '../crypto/EncryptionManager';
import { MedicalRecordsManager } from '../records/MedicalRecordsManager';
import { OfflineManager } from '../sync/OfflineManager';
import { NotificationManager } from '../notifications/NotificationManager';
import { UzimaConfig } from '../types';

/**
 * UzimaClient is the main entry point for the SDK
 * Provides unified access to all SDK features and coordinates between modules
 */
export class UzimaClient {
  private config: UzimaConfig;
  private apiClient: APIClient;
  private authManager: AuthManager;
  private biometricAuth: BiometricAuth;
  private encryptionManager: typeof EncryptionManager;
  private recordsManager: MedicalRecordsManager;
  private offlineManager: OfflineManager;
  private notificationManager: NotificationManager;

  constructor(config: Partial<UzimaConfig>) {
    // Initialize configuration
    this.config = Configuration.initialize(config);

    // Initialize managers
    this.apiClient = new APIClient(
      this.config.apiEndpoint,
      this.config.requestTimeout
    );
    this.authManager = new AuthManager();
    this.biometricAuth = new BiometricAuth();
    this.encryptionManager = EncryptionManager;
    this.recordsManager = new MedicalRecordsManager(this.apiClient);
    this.offlineManager = new OfflineManager(this.apiClient);
    this.notificationManager = new NotificationManager(this.apiClient);
  }

  /**
   * Initialize SDK with authentication
   */
  async initialize(publicKey: string, secretKey?: string): Promise<void> {
    if (publicKey && secretKey) {
      this.authManager.initializeWithKeyPair(publicKey, secretKey);
    }

    if (this.config.biometricEnabled) {
      this.authManager.setBiometricEnabled(true);
    }

    // Set auth token in API client
    const token = this.authManager.getSessionToken();
    if (token) {
      this.apiClient.setAuthToken(token);
    }
  }

  /**
   * Authenticate using biometric (if available)
   */
  async authenticateWithBiometric(): Promise<boolean> {
    if (!this.config.biometricEnabled) {
      throw new Error('Biometric authentication is not enabled in configuration');
    }

    const publicKey = this.authManager.getPublicKey();
    if (!publicKey) {
      throw new Error('No public key available for biometric authentication');
    }

    try {
      const sessionToken = await this.biometricAuth.createBiometricSession(
        publicKey
      );
      this.authManager.initializeWithSessionToken(sessionToken);
      this.apiClient.setAuthToken(sessionToken);
      return true;
    } catch (error) {
      console.error('Biometric authentication failed:', error);
      return false;
    }
  }

  /**
   * Get auth manager
   */
  getAuthManager(): AuthManager {
    return this.authManager;
  }

  /**
   * Get records manager
   */
  getRecordsManager(): MedicalRecordsManager {
    return this.recordsManager;
  }

  /**
   * Get offline manager
   */
  getOfflineManager(): OfflineManager {
    return this.offlineManager;
  }

  /**
   * Get notification manager
   */
  getNotificationManager(): NotificationManager {
    return this.notificationManager;
  }

  /**
   * Get encryption manager
   */
  getEncryptionManager(): typeof EncryptionManager {
    return this.encryptionManager;
  }

  /**
   * Get API client
   */
  getAPIClient(): APIClient {
    return this.apiClient;
  }

  /**
   * Check if SDK is ready
   */
  isReady(): boolean {
    return this.authManager.isAuthenticated();
  }

  /**
   * Check if offline mode is enabled
   */
  isOfflineEnabled(): boolean {
    return this.config.offlineEnabled;
  }

  /**
   * Check if notifications are enabled
   */
  isNotificationsEnabled(): boolean {
    return this.config.notificationsEnabled;
  }

  /**
   * Check if biometric is enabled
   */
  isBiometricEnabled(): boolean {
    return this.config.biometricEnabled;
  }

  /**
   * Register device for notifications
   */
  async registerForNotifications(
    token: string,
    platform: 'ios' | 'android' | 'web'
  ): Promise<void> {
    if (!this.config.notificationsEnabled) {
      throw new Error('Notifications are not enabled in configuration');
    }

    await this.notificationManager.registerDevice(token, platform);
  }

  /**
   * Logout and cleanup
   */
  logout(): void {
    this.authManager.logout();
    this.apiClient.clearAuth();
    this.recordsManager.clearCache();
    this.offlineManager.clearSyncQueue();
  }

  /**
   * Sync all pending operations
   */
  async syncAll(): Promise<void> {
    if (!this.config.offlineEnabled) {
      return;
    }

    const syncResult = await this.offlineManager.syncAll();
    const opResult = await this.offlineManager.syncOfflineOperations();

    console.log(
      `Sync complete: ${syncResult.synced + opResult.synced} synced, ${syncResult.failed + opResult.failed} failed`
    );
  }

  /**
   * Get SDK version
   */
  getVersion(): string {
    return '1.0.0';
  }

  /**
   * Get SDK status
   */
  getStatus(): {
    ready: boolean;
    authenticated: boolean;
    online: boolean;
    offlineQueueSize: number;
    cacheSize: number;
  } {
    return {
      ready: this.isReady(),
      authenticated: this.authManager.isAuthenticated(),
      online: this.offlineManager.isDeviceOnline(),
      offlineQueueSize: this.offlineManager.getOfflineQueueSize(),
      cacheSize: 0, // Would need to track this across managers
    };
  }
}
