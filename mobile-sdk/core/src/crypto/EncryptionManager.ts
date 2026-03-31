import nacl from 'tweetnacl';
import { EncryptedData } from '../types';
import { util } from 'tweetnacl-util';

/**
 * EncryptionManager handles end-to-end encryption for medical data using NaCl
 * Supports both public-key and secret-key cryptography
 */
export class EncryptionManager {
  private static readonly ALGORITHM_SECRETBOX = 'nacl-secretbox';
  private static readonly ALGORITHM_BOX = 'nacl-box';

  /**
   * Generate a new key pair for asymmetric encryption
   */
  static generateKeyPair(): { publicKey: string; secretKey: string } {
    const keyPair = nacl.box.keyPair();
    return {
      publicKey: this.bytesToBase64(keyPair.publicKey),
      secretKey: this.bytesToBase64(keyPair.secretKey),
    };
  }

  /**
   * Generate a shared secret for symmetric encryption
   */
  static generateSharedSecret(): string {
    const key = nacl.randomBytes(nacl.secretbox.keyLength);
    return this.bytesToBase64(key);
  }

  /**
   * Encrypt data using secret-key cryptography (symmetric)
   * More efficient for single-recipient scenarios
   */
  static encryptWithSharedSecret(data: string, sharedSecret: string): EncryptedData {
    const key = this.base64ToBytes(sharedSecret);
    const nonce = nacl.randomBytes(nacl.secretbox.nonceLength);
    const plaintext = this.stringToBytes(data);

    const encrypted = nacl.secretbox(plaintext, nonce, key);

    return {
      ciphertext: this.bytesToBase64(encrypted),
      nonce: this.bytesToBase64(nonce),
      algorithm: this.ALGORITHM_SECRETBOX,
    };
  }

  /**
   * Decrypt data using secret-key cryptography (symmetric)
   */
  static decryptWithSharedSecret(encrypted: EncryptedData, sharedSecret: string): string {
    const key = this.base64ToBytes(sharedSecret);
    const nonce = this.base64ToBytes(encrypted.nonce);
    const ciphertext = this.base64ToBytes(encrypted.ciphertext);

    const decrypted = nacl.secretbox.open(ciphertext, nonce, key);
    if (!decrypted) {
      throw new Error('Decryption failed: authentication tag verification failed');
    }

    return this.bytesToString(decrypted);
  }

  /**
   * Encrypt data for a specific recipient using public-key cryptography
   */
  static encryptForRecipient(
    data: string,
    recipientPublicKey: string,
    senderSecretKey: string
  ): EncryptedData {
    const publicKey = this.base64ToBytes(recipientPublicKey);
    const secretKey = this.base64ToBytes(senderSecretKey);
    const nonce = nacl.randomBytes(nacl.box.nonceLength);
    const plaintext = this.stringToBytes(data);

    const encrypted = nacl.box(plaintext, nonce, publicKey, secretKey);

    return {
      ciphertext: this.bytesToBase64(encrypted),
      nonce: this.bytesToBase64(nonce),
      algorithm: this.ALGORITHM_BOX,
    };
  }

  /**
   * Decrypt data encrypted for recipient
   */
  static decryptFromSender(
    encrypted: EncryptedData,
    senderPublicKey: string,
    recipientSecretKey: string
  ): string {
    const publicKey = this.base64ToBytes(senderPublicKey);
    const secretKey = this.base64ToBytes(recipientSecretKey);
    const nonce = this.base64ToBytes(encrypted.nonce);
    const ciphertext = this.base64ToBytes(encrypted.ciphertext);

    const decrypted = nacl.box.open(ciphertext, nonce, publicKey, secretKey);
    if (!decrypted) {
      throw new Error('Decryption failed: authentication tag verification failed');
    }

    return this.bytesToString(decrypted);
  }

  /**
   * Sign data with secret key
   */
  static sign(data: string, secretKey: string): string {
    const message = this.stringToBytes(data);
    const key = this.base64ToBytes(secretKey);
    const signature = nacl.sign(message, key);
    return this.bytesToBase64(signature);
  }

  /**
   * Verify signed data with public key
   */
  static verify(signature: string, publicKey: string): { valid: boolean; data?: string } {
    try {
      const sig = this.base64ToBytes(signature);
      const key = this.base64ToBytes(publicKey);
      const opened = nacl.sign.open(sig, key);

      if (opened) {
        return {
          valid: true,
          data: this.bytesToString(opened),
        };
      }
      return { valid: false };
    } catch (error) {
      return { valid: false };
    }
  }

  /**
   * Hash sensitive data for comparison without storing plaintext
   */
  static hash(data: string): string {
    const bytes = this.stringToBytes(data);
    const hashed = nacl.hash(bytes);
    return this.bytesToBase64(hashed);
  }

  // Helper methods
  private static stringToBytes(str: string): Uint8Array {
    const encoder = new TextEncoder();
    return encoder.encode(str);
  }

  private static bytesToString(bytes: Uint8Array): string {
    const decoder = new TextDecoder();
    return decoder.decode(bytes);
  }

  private static base64ToBytes(base64: string): Uint8Array {
    const binaryString = atob(base64);
    const bytes = new Uint8Array(binaryString.length);
    for (let i = 0; i < binaryString.length; i++) {
      bytes[i] = binaryString.charCodeAt(i);
    }
    return bytes;
  }

  private static bytesToBase64(bytes: Uint8Array): string {
    let binary = '';
    for (let i = 0; i < bytes.length; i++) {
      binary += String.fromCharCode(bytes[i]);
    }
    return btoa(binary);
  }
}
