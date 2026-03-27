import { PushNotification, NotificationType } from '../types';
import { APIClient } from '../network/APIClient';

export interface PushToken {
  token: string;
  platform: 'ios' | 'android' | 'web';
  os: string;
  appVersion: string;
}

/**
 * NotificationManager handles push notifications
 * Integrates with platform-specific services:
 * - iOS: Apple Push Notification service (APNs)
 * - Android: Firebase Cloud Messaging (FCM)
 */
export class NotificationManager {
  private apiClient: APIClient;
  private deviceToken: string | null = null;
  private platform: 'ios' | 'android' | 'web' | null = null;
  private pushListeners: Map<NotificationType, Function[]> = new Map();
  private allNotificationsListeners: Function[] = [];

  constructor(apiClient: APIClient) {
    this.apiClient = apiClient;
    this.detectPlatform();
  }

  /**
   * Register device for push notifications
   */
  async registerDevice(token: string, platform: 'ios' | 'android' | 'web'): Promise<void> {
    this.deviceToken = token;
    this.platform = platform;

    const pushToken: PushToken = {
      token,
      platform,
      os: this.getOSInfo(),
      appVersion: '1.0.0', // This would be set by the platform
    };

    await this.apiClient.post('/notifications/register-device', pushToken);
  }

  /**
   * Unregister device
   */
  async unregisterDevice(): Promise<void> {
    if (!this.deviceToken) return;

    await this.apiClient.post('/notifications/unregister-device', {
      token: this.deviceToken,
    });

    this.deviceToken = null;
  }

  /**
   * Subscribe to push notifications for specific event type
   */
  subscribe(notificationType: NotificationType, handler: (notification: PushNotification) => void): void {
    if (!this.pushListeners.has(notificationType)) {
      this.pushListeners.set(notificationType, []);
    }
    this.pushListeners.get(notificationType)!.push(handler);
  }

  /**
   * Unsubscribe from push notifications
   */
  unsubscribe(notificationType: NotificationType, handler: Function): void {
    const handlers = this.pushListeners.get(notificationType);
    if (handlers) {
      const index = handlers.indexOf(handler);
      if (index > -1) {
        handlers.splice(index, 1);
      }
    }
  }

  /**
   * Subscribe to all notifications
   */
  subscribeToAll(handler: (notification: PushNotification) => void): void {
    this.allNotificationsListeners.push(handler);
  }

  /**
   * Handle incoming push notification
   */
  handlePushNotification(data: Record<string, any>): void {
    const notification: PushNotification = {
      id: data.id || `notif_${Date.now()}`,
      type: data.type || NotificationType.ALERT,
      title: data.title || '',
      body: data.body || '',
      data: data.data || {},
      timestamp: data.timestamp || Date.now(),
      read: false,
    };

    // Notify type-specific listeners
    const handlers = this.pushListeners.get(notification.type);
    if (handlers) {
      handlers.forEach((handler) => handler(notification));
    }

    // Notify all listeners
    this.allNotificationsListeners.forEach((handler) => handler(notification));
  }

  /**
   * Enable notifications
   */
  async enableNotifications(): Promise<void> {
    if (!this.deviceToken) {
      throw new Error('Device not registered for push notifications');
    }

    await this.apiClient.post('/notifications/enable', {
      token: this.deviceToken,
    });
  }

  /**
   * Disable notifications
   */
  async disableNotifications(): Promise<void> {
    if (!this.deviceToken) {
      throw new Error('Device not registered for push notifications');
    }

    await this.apiClient.post('/notifications/disable', {
      token: this.deviceToken,
    });
  }

  /**
   * Request notification permission (iOS)
   */
  async requestNotificationPermission(): Promise<boolean> {
    // Platform-specific implementations will override this
    // For iOS: triggers native permission request
    // For Android: handled at app install time
    if (typeof window !== 'undefined' && 'Notification' in window) {
      const permission = (await (window as any).Notification.requestPermission?.()) || 'default';
      return permission === 'granted';
    }
    return false;
  }

  /**
   * Get notification history
   */
  async getNotificationHistory(limit: number = 50): Promise<PushNotification[]> {
    const response = await this.apiClient.get<PushNotification[]>(
      `/notifications/history?limit=${limit}`
    );
    return response.data || [];
  }

  /**
   * Mark notification as read
   */
  async markNotificationAsRead(notificationId: string): Promise<void> {
    await this.apiClient.post(`/notifications/${notificationId}/read`, {});
  }

  /**
   * Delete notification
   */
  async deleteNotification(notificationId: string): Promise<void> {
    await this.apiClient.delete(`/notifications/${notificationId}`);
  }

  /**
   * Clear all notifications
   */
  async clearAllNotifications(): Promise<void> {
    await this.apiClient.post('/notifications/clear-all', {});
  }

  /**
   * Update notification preferences
   */
  async updatePreferences(preferences: {
    recordAccess?: boolean;
    recordUpdate?: boolean;
    permissions?: boolean;
    alerts?: boolean;
    reminders?: boolean;
  }): Promise<void> {
    await this.apiClient.put('/notifications/preferences', preferences);
  }

  /**
   * Get notification count
   */
  async getUnreadCount(): Promise<number> {
    const response = await this.apiClient.get<{ count: number }>(
      '/notifications/unread-count'
    );
    return response.data?.count || 0;
  }

  /**
   * Detect platform
   */
  private detectPlatform(): void {
    if (typeof navigator !== 'undefined') {
      const ua = navigator.userAgent.toLowerCase();
      if (/iphone|ipad|ipod/.test(ua)) {
        this.platform = 'ios';
      } else if (/android/.test(ua)) {
        this.platform = 'android';
      } else {
        this.platform = 'web';
      }
    }
  }

  /**
   * Get OS info
   */
  private getOSInfo(): string {
    if (typeof navigator !== 'undefined') {
      return navigator.userAgent;
    }
    return 'unknown';
  }

  /**
   * Get current device token
   */
  getDeviceToken(): string | null {
    return this.deviceToken;
  }

  /**
   * Get current platform
   */
  getPlatform(): 'ios' | 'android' | 'web' | null {
    return this.platform;
  }
}
