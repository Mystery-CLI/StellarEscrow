import axios, {
  AxiosError,
  AxiosInstance,
  AxiosResponse,
  InternalAxiosRequestConfig,
} from 'axios';

import {
  ApiClientConfig,
  ApiError,
  RetryConfig,
  RequestInterceptor,
  ResponseInterceptor,
  ErrorInterceptor,
} from './types';

export class ApiClient {
  private client: AxiosInstance;
  private retryConfig: RetryConfig;

  private requestInterceptors: RequestInterceptor[] = [];
  private responseInterceptors: ResponseInterceptor[] = [];
  private errorInterceptors: ErrorInterceptor[] = [];

  constructor(config: ApiClientConfig) {
    this.retryConfig = config.retryConfig || {
      maxRetries: 3,
      delayMs: 1000,
      backoffMultiplier: 2,
    };

    this.client = axios.create({
      baseURL: config.baseURL,
      timeout: config.timeout || 30000,
    });

    this.setupInterceptors();
  }

  // =========================
  // INTERCEPTORS
  // =========================

  private setupInterceptors() {
    this.client.interceptors.request.use((config) => {
      let nextConfig: InternalAxiosRequestConfig = config;

      for (const interceptor of this.requestInterceptors) {
        nextConfig = interceptor(nextConfig);
      }

      return nextConfig;
    });

    this.client.interceptors.response.use(
      (response: AxiosResponse) => {
        let nextResponse = response;

        for (const interceptor of this.responseInterceptors) {
          nextResponse = interceptor(nextResponse);
        }

        return nextResponse;
      },
      (error: AxiosError) => this.handleError(error)
    );
  }

  addRequestInterceptor(interceptor: RequestInterceptor) {
    this.requestInterceptors.push(interceptor);
  }

  addResponseInterceptor(interceptor: ResponseInterceptor) {
    this.responseInterceptors.push(interceptor);
  }

  addErrorInterceptor(interceptor: ErrorInterceptor) {
    this.errorInterceptors.push(interceptor);
  }

  // =========================
  // ERROR HANDLING
  // =========================

  private async handleError(error: AxiosError): Promise<any> {
    for (const interceptor of this.errorInterceptors) {
      try {
        return await interceptor(error);
      } catch {
        // try next interceptor
      }
    }

    throw this.parseError(error);
  }

  private parseError(error: AxiosError): ApiError {
    return {
      code: error.code || 'UNKNOWN_ERROR',
      message: error.message,
      status: error.response?.status || 0,
      details: error.response?.data as Record<string, any> | undefined,
    };
  }

  // =========================
  // RETRY LOGIC
  // =========================

  private async retryRequest<T>(fn: () => Promise<T>, attempt = 0): Promise<T> {
    try {
      return await fn();
    } catch (error: any) {

      if (attempt < this.retryConfig.maxRetries && this.shouldRetry(error)) {
        const delay = this.retryConfig.delayMs * Math.pow(this.retryConfig.backoffMultiplier, attempt);

      if (
        attempt < this.retryConfig.maxRetries &&
        this.shouldRetry(error)
      ) {
        const delay =
          this.retryConfig.delayMs *
          Math.pow(this.retryConfig.backoffMultiplier, attempt);

        await new Promise((resolve) => setTimeout(resolve, delay));

        return this.retryRequest(fn, attempt + 1);
      }

      // If error is already a parsed ApiError (has 'code' and 'status'), rethrow as-is.
      // Otherwise parse it so callers always receive ApiError shape.
      if (error.code !== undefined && typeof error.status === 'number') {
        throw error;
      }
      throw this.parseError(error);
    }
  }

  private shouldRetry(error: any): boolean {

    const status = error.status || error.response?.status;
    if (status) {
      return status >= 500 || status === 408 || status === 429;
    }
    const status = error.response?.status ?? error.status ?? 0;

    if (status) {
      return (
        status === 502 ||
        status === 503 ||
        status === 504 ||
        status === 408 ||
        status === 429
      );

    }

    return (
      error.code === 'ECONNABORTED' ||
      error.code === 'ENOTFOUND'
    );
  }

  // =========================
  // HTTP METHODS
  // =========================

  async get<T = any>(url: string, config?: any): Promise<T> {
    return this.retryRequest(() =>
      this.client
        .get<T>(url, config)
        .then((res: AxiosResponse<T>) => res.data)
    );
  }

  async post<T = any>(url: string, data?: any, config?: any): Promise<T> {
    return this.retryRequest(() =>
      this.client
        .post<T>(url, data, config)
        .then((res: AxiosResponse<T>) => res.data)
    );
  }

  async patch<T = any>(url: string, data?: any, config?: any): Promise<T> {
    return this.retryRequest(() =>
      this.client
        .patch<T>(url, data, config)
        .then((res: AxiosResponse<T>) => res.data)
    );
  }

  async delete<T = any>(url: string, config?: any): Promise<T> {
    return this.retryRequest(() =>
      this.client
        .delete<T>(url, config)
        .then((res: AxiosResponse<T>) => res.data)
    );
  }
}