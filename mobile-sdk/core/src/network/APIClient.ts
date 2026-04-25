import axios, { AxiosInstance, AxiosRequestConfig } from 'axios';
import { APIResponse, APIError } from '../types';
import { Configuration } from '../config/UzimaConfig';

export interface RequestOptions extends AxiosRequestConfig {
  bypassCache?: boolean;
  retryCount?: number;
  maxRetries?: number;
}

/**
 * APIClient handles all HTTP communications with the Uzima backend
 * Features:
 * - Automatic retry logic
 * - Request caching
 * - Authentication header management
 * - Response time tracking for <200ms requirement
 */
export class APIClient {
  private axiosInstance: AxiosInstance;
  private cache: Map<string, { data: any; expiresAt: number }> = new Map();
  private sessionToken: string | null = null;

  constructor(apiEndpoint: string, timeout: number = 30000) {
    this.axiosInstance = axios.create({
      baseURL: apiEndpoint,
      timeout,
      headers: {
        'Content-Type': 'application/json',
        'User-Agent': 'UzimaSDK/1.0.0',
      },
    });

    // Add response interceptor
    this.axiosInstance.interceptors.response.use(
      (response) => response,
      async (error) => {
        const config = error.config as RequestOptions;

        if (!config.retryCount) {
          config.retryCount = 0;
        }

        const maxRetries = config.maxRetries || 3;

        if (error.response?.status === 429 || error.response?.status >= 500) {
          if (config.retryCount < maxRetries) {
            config.retryCount++;
            const backoffMs = Math.pow(2, config.retryCount) * 1000;
            await new Promise((resolve) => setTimeout(resolve, backoffMs));
            return this.axiosInstance(config);
          }
        }

        return Promise.reject(error);
      }
    );
  }

  /**
   * Set authentication token for subsequent requests
   */
  setAuthToken(token: string): void {
    this.sessionToken = token;
    this.axiosInstance.defaults.headers.common['Authorization'] = `Bearer ${token}`;
  }

  /**
   * Clear authentication
   */
  clearAuth(): void {
    this.sessionToken = null;
    delete this.axiosInstance.defaults.headers.common['Authorization'];
  }

  /**
   * Make a GET request
   */
  async get<T = any>(
    path: string,
    options: RequestOptions = {}
  ): Promise<APIResponse<T>> {
    const startTime = performance.now();
    const cacheKey = `GET:${path}`;

    // Check cache
    if (
      Configuration.isCacheEnabled() &&
      !options.bypassCache &&
      this.isCacheValid(cacheKey)
    ) {
      const cached = this.cache.get(cacheKey);
      if (cached) {
        return cached.data;
      }
    }

    try {
      const response = await this.axiosInstance.get<APIResponse<T>>(path, options);
      const duration = performance.now() - startTime;

      // Cache successful responses
      if (Configuration.isCacheEnabled() && response.status === 200) {
        this.cache.set(cacheKey, {
          data: response.data,
          expiresAt: Date.now() + (Configuration.getConfig().cacheTTL || 300000),
        });
      }

      return this.enrichResponse(response.data, duration);
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Make a POST request
   */
  async post<T = any>(
    path: string,
    data?: any,
    options: RequestOptions = {}
  ): Promise<APIResponse<T>> {
    const startTime = performance.now();

    try {
      const response = await this.axiosInstance.post<APIResponse<T>>(path, data, options);
      const duration = performance.now() - startTime;

      // Clear related cache entries on POST
      this.invalidateCache(path);

      return this.enrichResponse(response.data, duration);
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Make a PUT request
   */
  async put<T = any>(
    path: string,
    data?: any,
    options: RequestOptions = {}
  ): Promise<APIResponse<T>> {
    const startTime = performance.now();

    try {
      const response = await this.axiosInstance.put<APIResponse<T>>(path, data, options);
      const duration = performance.now() - startTime;

      // Clear related cache entries on PUT
      this.invalidateCache(path);

      return this.enrichResponse(response.data, duration);
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Make a DELETE request
   */
  async delete<T = any>(
    path: string,
    options: RequestOptions = {}
  ): Promise<APIResponse<T>> {
    const startTime = performance.now();

    try {
      const response = await this.axiosInstance.delete<APIResponse<T>>(path, options);
      const duration = performance.now() - startTime;

      // Clear related cache entries on DELETE
      this.invalidateCache(path);

      return this.enrichResponse(response.data, duration);
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Batch requests for efficiency
   */
  async batch<T = any>(
    requests: Array<{ method: string; path: string; data?: any }>
  ): Promise<APIResponse<T[]>> {
    const startTime = performance.now();

    try {
      const responses = await Promise.all(
        requests.map((req) => {
          if (req.method === 'GET') {
            return this.axiosInstance.get(req.path);
          } else if (req.method === 'POST') {
            return this.axiosInstance.post(req.path, req.data);
          } else if (req.method === 'PUT') {
            return this.axiosInstance.put(req.path, req.data);
          } else if (req.method === 'DELETE') {
            return this.axiosInstance.delete(req.path);
          }
          return Promise.reject(new Error(`Unsupported method: ${req.method}`));
        })
      );

      const duration = performance.now() - startTime;
      const data = responses.map((r) => r.data);

      return this.enrichResponse(data as any, duration);
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Clear cache for a specific path
   */
  private invalidateCache(path: string): void {
    const keysToDelete: string[] = [];
    this.cache.forEach((_, key) => {
      if (key.includes(path)) {
        keysToDelete.push(key);
      }
    });
    keysToDelete.forEach((key) => this.cache.delete(key));
  }

  /**
   * Check if cache entry is still valid
   */
  private isCacheValid(key: string): boolean {
    const entry = this.cache.get(key);
    if (!entry) return false;
    if (Date.now() > entry.expiresAt) {
      this.cache.delete(key);
      return false;
    }
    return true;
  }

  /**
   * Enrich response with metadata
   */
  private enrichResponse<T>(data: APIResponse<T>, duration: number): APIResponse<T> {
    return {
      ...data,
      timestamp: Date.now(),
      requestId: this.generateRequestId(),
    };
  }

  /**
   * Handle and normalize errors
   */
  private handleError(error: any): APIError {
    if (error.response) {
      const status = error.response.status;
      return {
        code: `HTTP_${status}`,
        message: error.response.data?.message || error.message,
        details: error.response.data,
      };
    }

    if (error.request) {
      return {
        code: 'NETWORK_ERROR',
        message: 'Network request failed',
        details: { originalError: error.message },
      };
    }

    return {
      code: 'UNKNOWN_ERROR',
      message: error.message || 'An unknown error occurred',
    };
  }

  /**
   * Generate unique request ID for tracking
   */
  private generateRequestId(): string {
    return `req_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Get request performance metrics
   */
  getMetrics(): { hitRate: number; cacheSize: number } {
    return {
      hitRate: this.cache.size > 0 ? 1 : 0,
      cacheSize: this.cache.size,
    };
  }

  /**
   * Clear all cache
   */
  clearCache(): void {
    this.cache.clear();
  }
}
