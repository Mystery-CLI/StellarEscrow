export interface ApiResponse<T = any> {
  data: T;
  status: number;
  message?: string;
}

export interface ApiError {
  code: string;
  message: string;
  status: number;
  details?: Record<string, any>;
}

export interface RetryConfig {
  maxRetries: number;
  delayMs: number;
  backoffMultiplier: number;
}

export interface ApiClientConfig {
  baseURL: string;
  timeout?: number;
  retryConfig?: RetryConfig;
  mockEnabled?: boolean;
}

export interface RequestInterceptor {
  (config: any): any;
}

export interface ResponseInterceptor {
  (response: any): any;
}

export interface ErrorInterceptor {
  (error: any): Promise<any>;
}
