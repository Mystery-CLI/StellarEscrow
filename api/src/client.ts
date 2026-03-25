import axios, { AxiosInstance, AxiosError, AxiosResponse } from 'axios';
import { ApiClientConfig, ApiError, RetryConfig, RequestInterceptor, ResponseInterceptor, ErrorInterceptor } from './types';

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

  private setupInterceptors() {
    this.client.interceptors.request.use((config) => {
      for (const interceptor of this.requestInterceptors) {
        config = interceptor(config);
      }
      return config;
    });

    this.client.interceptors.response.use(
      (response) => {
        for (const interceptor of this.responseInterceptors) {
          response = interceptor(response);
        }
        return response;
      },
      (error) => this.handleError(error)
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

  private async handleError(error: AxiosError): Promise<any> {
    for (const interceptor of this.errorInterceptors) {
      try {
        return await interceptor(error);
      } catch (e) {
        // Continue to next interceptor
      }
    }
    throw this.parseError(error);
  }

  private parseError(error: AxiosError): ApiError {
    return {
      code: error.code || 'UNKNOWN_ERROR',
      message: error.message,
      status: error.response?.status || 0,
      details: error.response?.data,
    };
  }

  private async retryRequest<T>(fn: () => Promise<T>, attempt = 0): Promise<T> {
    try {
      return await fn();
    } catch (error) {
      if (attempt < this.retryConfig.maxRetries && this.shouldRetry(error)) {
        const delay = this.retryConfig.delayMs * Math.pow(this.retryConfig.backoffMultiplier, attempt);
        await new Promise((resolve) => setTimeout(resolve, delay));
        return this.retryRequest(fn, attempt + 1);
      }
      throw error;
    }
  }

  private shouldRetry(error: any): boolean {
    if (error.response?.status) {
      return error.response.status >= 500 || error.response.status === 408 || error.response.status === 429;
    }
    return error.code === 'ECONNABORTED' || error.code === 'ENOTFOUND';
  }

  async get<T = any>(url: string, config?: any): Promise<T> {
    return this.retryRequest(() =>
      this.client.get<T>(url, config).then((res) => res.data)
    );
  }

  async post<T = any>(url: string, data?: any, config?: any): Promise<T> {
    return this.retryRequest(() =>
      this.client.post<T>(url, data, config).then((res) => res.data)
    );
  }

  async patch<T = any>(url: string, data?: any, config?: any): Promise<T> {
    return this.retryRequest(() =>
      this.client.patch<T>(url, data, config).then((res) => res.data)
    );
  }

  async delete<T = any>(url: string, config?: any): Promise<T> {
    return this.retryRequest(() =>
      this.client.delete<T>(url, config).then((res) => res.data)
    );
  }
}
