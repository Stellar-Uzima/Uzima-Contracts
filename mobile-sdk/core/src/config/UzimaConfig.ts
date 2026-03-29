import { UzimaConfig } from '../types';

export class Configuration {
  static instance: UzimaConfig;

  static initialize(config: Partial<UzimaConfig>): UzimaConfig {
    this.instance = {
      apiEndpoint: config.apiEndpoint || '',
      contractId: config.contractId || '',
      networkPassphrase: config.networkPassphrase || 'Test SDF Network ; September 2015',
      serverURL: config.serverURL || 'https://soroban-testnet.stellar.org:443',
      encryptionKey: config.encryptionKey,
      offlineEnabled: config.offlineEnabled !== false,
      notificationsEnabled: config.notificationsEnabled !== false,
      biometricEnabled: config.biometricEnabled !== false,
      requestTimeout: config.requestTimeout || 30000,
      cacheEnabled: config.cacheEnabled !== false,
      cacheTTL: config.cacheTTL || 300000, // 5 minutes default
    };
    return this.instance;
  }

  static getConfig(): UzimaConfig {
    if (!this.instance) {
      throw new Error('SDK not initialized. Call Configuration.initialize() first.');
    }
    return this.instance;
  }

  static getContractId(): string {
    return this.getConfig().contractId;
  }

  static getAPIEndpoint(): string {
    return this.getConfig().apiEndpoint;
  }

  static isOfflineEnabled(): boolean {
    return this.getConfig().offlineEnabled;
  }

  static isNotificationsEnabled(): boolean {
    return this.getConfig().notificationsEnabled;
  }

  static isBiometricEnabled(): boolean {
    return this.getConfig().biometricEnabled;
  }

  static isCacheEnabled(): boolean {
    return this.getConfig().cacheEnabled;
  }
}

export class UzimaConfig {}
