export interface ApiResponse<T = any> {
  data: T;
  status: number;
  message?: string;
}

// =========================
// API ERROR (STRICT + TEST SAFE)
// =========================

export interface ApiError {
  code: string;
  message: string;
  status: number;
  details?: Record<string, any>;
}

// =========================
// RETRY CONFIG
// =========================

export interface RetryConfig {
  maxRetries: number;
  delayMs: number;
  backoffMultiplier: number;
}

// =========================
// CLIENT CONFIG
// =========================

export interface ApiClientConfig {
  baseURL: string;
  timeout?: number;
  retryConfig?: RetryConfig;
  mockEnabled?: boolean;
}

// =========================
// INTERCEPTORS (TYPED 🔥)
// =========================

export type RequestInterceptor = (config: any) => any;

export type ResponseInterceptor = (response: any) => any;

export type ErrorInterceptor = (error: any) => Promise<any>;

// =========================
// EXPORT MODELS
// =========================

export type { Trade, Event } from './models';