import React, { useCallback, useEffect, useState } from 'react';
import { UzimaClient } from '@uzima/sdk-core';
import { UzimaConfig } from '@uzima/sdk-core';
import * as Keychain from 'react-native-keychain';
import PushNotification from 'react-native-push-notification';

/**
 * UzimaProvider context provider for React Native applications
 */
export interface UzimaContextType {
  client: UzimaClient | null;
  isReady: boolean;
  isAuthenticated: boolean;
  error: Error | null;
  authenticate: (publicKey: string, secretKey: string) => Promise<void>;
  authenticateWithBiometric: () => Promise<boolean>;
  logout: () => void;
}

const UzimaContext = React.createContext<UzimaContextType | undefined>(undefined);

export interface UzimaProviderProps {
  config: Partial<UzimaConfig>;
  children: React.ReactNode;
}

/**
 * Provider component for Uzima SDK
 */
export const UzimaProvider: React.FC<UzimaProviderProps> = ({
  config,
  children,
}) => {
  const [client, setClient] = useState<UzimaClient | null>(null);
  const [isReady, setIsReady] = useState(false);
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    const initializeSDK = async () => {
      try {
        const uzimaClient = new UzimaClient(config);
        setClient(uzimaClient);
        
        // Check for stored credentials
        const credentials = await Keychain.getGenericPassword();
        if (credentials) {
          await uzimaClient.initialize(
            credentials.username,
            credentials.password
          );
          setIsAuthenticated(true);
        }
        
        setIsReady(true);
      } catch (err) {
        setError(err as Error);
        setIsReady(true);
      }
    };

    initializeSDK();
  }, [config]);

  const authenticate = useCallback(
    async (publicKey: string, secretKey: string) => {
      try {
        if (!client) throw new Error('SDK not initialized');

        await Keychain.setGenericPassword(publicKey, secretKey);
        await client.initialize(publicKey, secretKey);
        setIsAuthenticated(true);
        setError(null);
      } catch (err) {
        setError(err as Error);
        throw err;
      }
    },
    [client]
  );

  const authenticateWithBiometric = useCallback(async () => {
    try {
      if (!client) throw new Error('SDK not initialized');

      const success = await client.authenticateWithBiometric();
      if (success) {
        setIsAuthenticated(true);
      }
      return success;
    } catch (err) {
      setError(err as Error);
      return false;
    }
  }, [client]);

  const logout = useCallback(() => {
    if (client) {
      client.logout();
      Keychain.resetGenericPassword();
      setIsAuthenticated(false);
    }
  }, [client]);

  const value: UzimaContextType = {
    client,
    isReady,
    isAuthenticated,
    error,
    authenticate,
    authenticateWithBiometric,
    logout,
  };

  return (
    <UzimaContext.Provider value={value}>{children}</UzimaContext.Provider>
  );
};

/**
 * Hook to use Uzima context
 */
export const useUzima = (): UzimaContextType => {
  const context = React.useContext(UzimaContext);
  if (!context) {
    throw new Error('useUzima must be used within UzimaProvider');
  }
  return context;
};

/**
 * Hook for medical records
 */
export const useMedicalRecords = () => {
  const { client } = useUzima();

  const createRecord = useCallback(
    async (patientId, providerId, recordType, data, encryptionKey) => {
      if (!client) throw new Error('SDK not initialized');
      return client.getRecordsManager().createRecord(
        patientId,
        providerId,
        recordType,
        data,
        encryptionKey
      );
    },
    [client]
  );

  const getRecord = useCallback(
    async (recordId, encryptionKey) => {
      if (!client) throw new Error('SDK not initialized');
      return client.getRecordsManager().getRecord(recordId, encryptionKey);
    },
    [client]
  );

  const updateRecord = useCallback(
    async (recordId, updates, encryptionKey) => {
      if (!client) throw new Error('SDK not initialized');
      return client.getRecordsManager().updateRecord(recordId, updates, encryptionKey);
    },
    [client]
  );

  const deleteRecord = useCallback(
    async (recordId) => {
      if (!client) throw new Error('SDK not initialized');
      return client.getRecordsManager().deleteRecord(recordId);
    },
    [client]
  );

  return {
    createRecord,
    getRecord,
    updateRecord,
    deleteRecord,
  };
};

/**
 * Hook for push notifications
 */
export const usePushNotifications = () => {
  const { client } = useUzima();

  const registerDevice = useCallback(
    async (token: string) => {
      if (!client) throw new Error('SDK not initialized');
      await client.registerForNotifications(token, 'android');
    },
    [client]
  );

  const subscribe = useCallback(
    (notificationType, handler) => {
      if (!client) throw new Error('SDK not initialized');
      client.getNotificationManager().subscribe(notificationType, handler);
    },
    [client]
  );

  return { registerDevice, subscribe };
};
