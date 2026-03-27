import Foundation
import LocalAuthentication
import CryptoKit

/**
 * UzimaClient iOS wrapper for native iOS integration
 * Provides biometric authentication and platform-specific features
 */
@available(iOS 13.0, *)
public class UzimaClientiOS {
    private let jsClient: String // Reference to JS SDK core
    private let biometricContext = LAContext()
    private let keychain = KeychainManager()
    
    public init(config: UzimaConfig) {
        // Initialize JS SDK core through bridge
    }
    
    /**
     * Request biometric permission and authenticate
     */
    public func authenticateWithBiometric(
        completion: @escaping (Bool, Error?) -> Void
    ) {
        var error: NSError?
        guard biometricContext.canEvaluatePolicy(
            .deviceOwnerAuthenticationWithBiometrics,
            error: &error
        ) else {
            completion(false, error)
            return
        }
        
        biometricContext.evaluatePolicy(
            .deviceOwnerAuthenticationWithBiometrics,
            localizedReason: "Authenticate to access medical records"
        ) { success, error in
            DispatchQueue.main.async {
                if success {
                    // Generate session token
                    let token = self.generateSessionToken()
                    self.keychain.store(token, for: "session_token")
                }
                completion(success, error)
            }
        }
    }
    
    /**
     * Enroll biometric (Face ID or Touch ID)
     */
    public func enrollBiometric() -> Bool {
        return biometricContext.canEvaluatePolicy(
            .deviceOwnerAuthenticationWithBiometrics,
            error: nil
        )
    }
    
    /**
     * Get available biometry type
     */
    public func getAvailableBiometry() -> String {
        if #available(iOS 11.0, *) {
            switch biometricContext.biometryType {
            case .none:
                return "none"
            case .touchID:
                return "fingerprint"
            case .faceID:
                return "faces"
            @unknown default:
                return "unknown"
            }
        }
        return "touchid"
    }
    
    /**
     * Store credentials securely
     */
    public func storeCredentials(
        publicKey: String,
        secretKey: String
    ) -> Bool {
        let stored = keychain.store(publicKey, for: "public_key") &&
                     keychain.store(secretKey, for: "secret_key")
        return stored
    }
    
    /**
     * Retrieve stored credentials
     */
    public func getStoredCredentials() -> (publicKey: String, secretKey: String)? {
        guard let publicKey = keychain.retrieve(for: "public_key"),
              let secretKey = keychain.retrieve(for: "secret_key") else {
            return nil
        }
        return (publicKey, secretKey)
    }
    
    /**
     * Register device for push notifications
     */
    public func registerForPushNotifications() {
        DispatchQueue.main.async {
            UIApplication.shared.registerForRemoteNotifications()
        }
    }
    
    /**
     * Handle push notification
     */
    public func handlePushNotification(
        userInfo: [AnyHashable: Any]
    ) {
        // Extract notification data and call JS SDK
        print("Handling push notification: \(userInfo)")
    }
    
    /**
     * Generate session token
     */
    private func generateSessionToken() -> String {
        let timestamp = Int(Date().timeIntervalSince1970)
        let random = UUID().uuidString
        return "ios_\(timestamp)_\(random)"
    }
}

/**
 * Keychain manager for secure credential storage
 */
class KeychainManager {
    private let service = "com.uzima.medical"
    
    func store(_ value: String, for key: String) -> Bool {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: key,
            kSecValueData as String: value.data(using: .utf8) ?? Data(),
        ]
        
        SecItemDelete(query as CFDictionary)
        return SecItemAdd(query as CFDictionary, nil) == errSecSuccess
    }
    
    func retrieve(for key: String) -> String? {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: key,
            kSecReturnData as String: true,
        ]
        
        var result: AnyObject?
        let status = SecItemCopyMatching(query as CFDictionary, &result)
        
        guard status == errSecSuccess,
              let data = result as? Data,
              let value = String(data: data, encoding: .utf8) else {
            return nil
        }
        
        return value
    }
    
    func delete(for key: String) -> Bool {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: key,
        ]
        
        return SecItemDelete(query as CFDictionary) == errSecSuccess
    }
}

/**
 * Configuration for iOS SDK
 */
public struct UzimaConfig {
    public let apiEndpoint: String
    public let contractId: String
    public let offlineEnabled: Bool
    public let notificationsEnabled: Bool
    public let biometricEnabled: Bool
    
    public init(
        apiEndpoint: String,
        contractId: String,
        offlineEnabled: Bool = true,
        notificationsEnabled: Bool = true,
        biometricEnabled: Bool = true
    ) {
        self.apiEndpoint = apiEndpoint
        self.contractId = contractId
        self.offlineEnabled = offlineEnabled
        self.notificationsEnabled = notificationsEnabled
        self.biometricEnabled = biometricEnabled
    }
}
