package com.uzima.sdk

import android.content.Context
import android.os.Build
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricPrompt
import androidx.fragment.app.FragmentActivity
import com.google.firebase.messaging.FirebaseMessaging
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator

/**
 * UzimaClient Android wrapper for native Android integration
 * Provides biometric authentication and platform-specific features
 */
class UzimaClientAndroid(
    private val context: Context,
    private val config: UzimaConfig
) {
    private val biometricManager = BiometricManager.from(context)
    private val keychainManager = KeychainManager(context)
    private var biometricCallback: BiometricCallback? = null

    /**
     * Check if biometric authentication is available
     */
    fun isBiometricAvailable(): Boolean {
        return when (biometricManager.canAuthenticate(
            BiometricManager.Authenticators.BIOMETRIC_STRONG or
            BiometricManager.Authenticators.BIOMETRIC_WEAK
        )) {
            BiometricManager.BIOMETRIC_SUCCESS -> true
            else -> false
        }
    }

    /**
     * Get available biometry type
     */
    fun getAvailableBiometry(): String {
        val authenticators = BiometricManager.Authenticators.BIOMETRIC_STRONG or
                             BiometricManager.Authenticators.BIOMETRIC_WEAK

        return when {
            biometricManager.canAuthenticate(authenticators) == BiometricManager.BIOMETRIC_SUCCESS ->
                "fingerprint" // Android doesn't distinguish between types in standard way
            else -> "none"
        }
    }

    /**
     * Authenticate using biometric (requires FragmentActivity)
     */
    fun authenticateWithBiometric(
        activity: FragmentActivity,
        callback: BiometricCallback
    ) {
        if (!isBiometricAvailable()) {
            callback.onBiometricError("Biometric authentication not available")
            return
        }

        this.biometricCallback = callback

        val promptInfo = BiometricPrompt.PromptInfo.Builder()
            .setTitle("Uzima Medical Records")
            .setSubtitle("Authenticate to access your medical records")
            .setNegativeButtonText("Cancel")
            .build()

        val biometricPrompt = BiometricPrompt(
            activity,
            { _, result ->
                when (result) {
                    BiometricPrompt.AuthenticationResult.SUCCESS -> {
                        val token = generateSessionToken()
                        keychainManager.store("session_token", token)
                        callback.onBiometricSuccess()
                    }
                    BiometricPrompt.AuthenticationResult.FAILURE -> {
                        callback.onBiometricError("Authentication failed")
                    }
                }
            }
        )

        biometricPrompt.authenticate(promptInfo)
    }

    /**
     * Store credentials securely in Android Keystore
     */
    fun storeCredentials(publicKey: String, secretKey: String): Boolean {
        return keychainManager.store("public_key", publicKey) &&
               keychainManager.store("secret_key", secretKey)
    }

    /**
     * Retrieve stored credentials
     */
    fun getStoredCredentials(): Pair<String, String>? {
        val publicKey = keychainManager.retrieve("public_key") ?: return null
        val secretKey = keychainManager.retrieve("secret_key") ?: return null
        return Pair(publicKey, secretKey)
    }

    /**
     * Register device for Firebase Cloud Messaging (FCM)
     */
    fun registerForPushNotifications(callback: (String) -> Unit) {
        FirebaseMessaging.getInstance().token.addOnCompleteListener { task ->
            if (task.isSuccessful) {
                val token = task.result
                keychainManager.store("fcm_token", token)
                callback(token)
            } else {
                callback("")
            }
        }
    }

    /**
     * Handle incoming FCM message
     */
    fun handlePushNotification(remoteMessage: String): Map<String, String> {
        // Parse notification data
        return mapOf(
            "type" to "notification",
            "message" to remoteMessage
        )
    }

    /**
     * Generate session token
     */
    private fun generateSessionToken(): String {
        val timestamp = System.currentTimeMillis()
        val random = (0..999999).random()
        return "android_${timestamp}_$random"
    }

    /**
     * Interface for biometric authentication callbacks
     */
    interface BiometricCallback {
        fun onBiometricSuccess()
        fun onBiometricError(error: String)
    }
}

/**
 * Secure credential storage using Android Keystore
 */
class KeychainManager(private val context: Context) {
    private val keyStore = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
    private val sharedPreferences = context.getSharedPreferences("uzima_prefs", Context.MODE_PRIVATE)

    fun store(key: String, value: String): Boolean {
        return try {
            val cipher = getEncryptionCipher(key)
            val encryptedData = cipher.doFinal(value.toByteArray())
            sharedPreferences.edit().putString(key, android.util.Base64.encodeToString(encryptedData, android.util.Base64.DEFAULT)).apply()
            true
        } catch (e: Exception) {
            false
        }
    }

    fun retrieve(key: String): String? {
        return try {
            val encryptedData = sharedPreferences.getString(key, null) ?: return null
            val decodedData = android.util.Base64.decode(encryptedData, android.util.Base64.DEFAULT)
            val cipher = getDecryptionCipher(key)
            val decryptedData = cipher.doFinal(decodedData)
            String(decryptedData)
        } catch (e: Exception) {
            null
        }
    }

    fun delete(key: String): Boolean {
        return try {
            sharedPreferences.edit().remove(key).apply()
            true
        } catch (e: Exception) {
            false
        }
    }

    private fun getEncryptionCipher(key: String): Cipher {
        createKeyIfNeeded(key)
        val cipher = Cipher.getInstance(
            "${KeyProperties.KEY_ALGORITHM_AES}/${KeyProperties.BLOCK_MODE_GCM}/${KeyProperties.ENCRYPTION_PADDING_NONE}"
        )
        val secretKey = keyStore.getKey(key, null)
        cipher.init(Cipher.ENCRYPT_MODE, secretKey)
        return cipher
    }

    private fun getDecryptionCipher(key: String): Cipher {
        val cipher = Cipher.getInstance(
            "${KeyProperties.KEY_ALGORITHM_AES}/${KeyProperties.BLOCK_MODE_GCM}/${KeyProperties.ENCRYPTION_PADDING_NONE}"
        )
        val secretKey = keyStore.getKey(key, null)
        cipher.init(Cipher.DECRYPT_MODE, secretKey)
        return cipher
    }

    private fun createKeyIfNeeded(key: String) {
        if (keyStore.getKey(key, null) == null) {
            val keyGenSpec = KeyGenParameterSpec.Builder(
                key,
                KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT
            )
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .build()

            val keyGenerator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES)
            keyGenerator.init(keyGenSpec)
            keyGenerator.generateKey()
        }
    }
}

/**
 * Configuration for Android SDK
 */
data class UzimaConfig(
    val apiEndpoint: String,
    val contractId: String,
    val offlineEnabled: Boolean = true,
    val notificationsEnabled: Boolean = true,
    val biometricEnabled: Boolean = true
)
