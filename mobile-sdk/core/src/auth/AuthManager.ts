import { AuthCredentials, BiometricOptions } from '../types';
import { EncryptionManager } from '../crypto/EncryptionManager';

/**
 * AuthManager handles user authentication and session management
 * Supports public key authentication and Stellar signatures
 */
export class AuthManager {
  private credentials: AuthCredentials | null = null;
  private sessionToken: string | null = null;
  private sessionExpiresAt: number = 0;
  private biometricEnabled: boolean = false;

  /**
   * Initialize authentication with public/secret key pair
   */
  initializeWithKeyPair(publicKey: string, secretKey: string): void {
    this.credentials = { publicKey, secretKey };
  }

  /**
   * Initialize with session token only (read-only mode)
   */
  initializeWithSessionToken(token: string, expiresIn: number = 3600000): void {
    this.sessionToken = token;
    this.sessionExpiresAt = Date.now() + expiresIn;
  }

  /**
   * Get current credentials
   */
  getCredentials(): AuthCredentials | null {
    return this.credentials;
  }

  /**
   * Get public key
   */
  getPublicKey(): string | null {
    return this.credentials?.publicKey || null;
  }

  /**
   * Get session token
   */
  getSessionToken(): string | null {
    if (this.sessionToken && Date.now() < this.sessionExpiresAt) {
      return this.sessionToken;
    }
    return null;
  }

  /**
   * Check if authenticated
   */
  isAuthenticated(): boolean {
    return (
      !!(this.credentials?.publicKey) ||
      (!!this.sessionToken && Date.now() < this.sessionExpiresAt)
    );
  }

  /**
   * Create a signed message for authentication
   */
  createSignedMessage(message: string): { message: string; signature: string } {
    if (!this.credentials?.secretKey) {
      throw new Error('No secret key available for signing');
    }

    const signature = EncryptionManager.sign(message, this.credentials.secretKey);

    return {
      message,
      signature,
    };
  }

  /**
   * Verify a message signature
   */
  verifySignature(message: string, signature: string, publicKey?: string): boolean {
    const key = publicKey || this.credentials?.publicKey;
    if (!key) {
      throw new Error('No public key available for verification');
    }

    const result = EncryptionManager.verify(signature, key);
    return result.valid && result.data === message;
  }

  /**
   * Logout and clear credentials
   */
  logout(): void {
    this.credentials = null;
    this.sessionToken = null;
    this.sessionExpiresAt = 0;
  }

  /**
   * Refresh session token
   */
  refreshSessionToken(newToken: string, expiresIn: number = 3600000): void {
    this.sessionToken = newToken;
    this.sessionExpiresAt = Date.now() + expiresIn;
  }

  /**
   * Enable biometric authentication
   */
  setBiometricEnabled(enabled: boolean): void {
    this.biometricEnabled = enabled;
  }

  /**
   * Check if biometric is enabled
   */
  isBiometricEnabled(): boolean {
    return this.biometricEnabled;
  }
}

/**
 * BiometricAuth provides platform-agnostic biometric authentication
 * Platform-specific implementations will replace these methods
 */
export class BiometricAuth {
  private isAvailable: boolean = false;
  private biometryType: 'faces' | 'fingerprint' | 'iris' | 'voice' | null = null;
  private fallbackToPin: boolean = false;

  /**
   * Check if biometric authentication is available
   */
  async isAvailable(): Promise<boolean> {
    // This method will be overridden by platform-specific implementations
    return this.isAvailable;
  }

  /**
   * Get available biometry type
   */
  async getAvailableBiometry(): Promise<'faces' | 'fingerprint' | 'iris' | 'voice' | null> {
    // This method will be overridden by platform-specific implementations
    return this.biometryType;
  }

  /**
   * Authenticate using biometric
   */
  async authenticate(options: BiometricOptions = {}): Promise<boolean> {
    if (!this.isAvailable) {
      if (options.fallbackToPin) {
        return await this.authenticateWithPin();
      }
      throw new Error('Biometric authentication is not available');
    }

    // This method will be overridden by platform-specific implementations
    try {
      // Platform-specific implementation would go here
      console.log(`Biometric authentication with: ${this.biometryType}`);
      return true;
    } catch (error) {
      if (options.fallbackToPin) {
        return await this.authenticateWithPin();
      }
      throw error;
    }
  }

  /**
   * Authenticate with PIN fallback
   */
  private async authenticateWithPin(): Promise<boolean> {
    // This would typically show a PIN input dialog
    // Platform-specific implementations will override this
    console.log('Falling back to PIN authentication');
    return true;
  }

  /**
   * Enroll biometric
   */
  async enroll(): Promise<boolean> {
    // This method will be overridden by platform-specific implementations
    console.log('Starting biometric enrollment');
    return true;
  }

  /**
   * Remove biometric enrollment
   */
  async removeEnrollment(): Promise<boolean> {
    // This method will be overridden by platform-specific implementations
    console.log('Removing biometric enrollment');
    return true;
  }

  /**
   * Create encrypted session token using biometric
   */
  async createBiometricSession(publicKey: string): Promise<string> {
    const authenticated = await this.authenticate();
    if (!authenticated) {
      throw new Error('Biometric authentication failed');
    }

    // Generate session token
    const sessionToken = this.generateSessionToken(publicKey);
    return sessionToken;
  }

  /**
   * Generate session token
   */
  private generateSessionToken(publicKey: string): string {
    const timestamp = Date.now();
    const random = Math.random().toString(36).substring(2, 15);
    const data = `${publicKey}:${timestamp}:${random}`;
    return Buffer.from(data).toString('base64');
  }
}
