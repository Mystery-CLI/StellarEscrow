import { ApiClient } from './client';
import { TradesApi, EventsApi, BlockchainApi } from './resources';
import {
  ApiClientConfig,
  ErrorInterceptor,
  RequestInterceptor,
  ResponseInterceptor,
} from './types';
import { loadConfig, ApiConfig } from './config';
import { API_ENDPOINT_CONTRACTS } from './contracts';

function normalizeBaseUrl(baseURL: string): string {
  const trimmed = baseURL.replace(/\/+$/, '');

  try {
    const url = new URL(trimmed);
    const normalizedPath = url.pathname.replace(/\/+$/, '');
    if (!normalizedPath || normalizedPath === '/') {
      url.pathname = '/api';
    }
    return url.toString().replace(/\/+$/, '');
  } catch {
    if (!trimmed || trimmed.endsWith('/api')) {
      return trimmed;
    }
    return `${trimmed}/api`;
  }
}

export class EscrowApi {
  private client: ApiClient;
  public trades: TradesApi;
  public events: EventsApi;
  public blockchain: BlockchainApi;

  constructor(config: ApiClientConfig) {
    this.client = new ApiClient(config);
    this.trades = new TradesApi(this.client);
    this.events = new EventsApi(this.client);
    this.blockchain = new BlockchainApi(this.client);
  }

  addRequestInterceptor(interceptor: RequestInterceptor) {
    this.client.addRequestInterceptor(interceptor);
  }

  addAuthToken(token: string) {
    this.client.addRequestInterceptor((config) => {
      const headers = (config.headers ?? {}) as Record<string, string>;
      headers.Authorization = `Bearer ${token}`;
      config.headers = headers;
      return config;
    });
  }

  addResponseInterceptor(interceptor: ResponseInterceptor) {
    this.client.addResponseInterceptor(interceptor);
  }

  addErrorHandler(handler: (error: any) => void) {
    this.client.addErrorInterceptor(async (error) => {
      handler(error);
      throw error;
    });
  }

  addErrorInterceptor(interceptor: ErrorInterceptor) {
    this.client.addErrorInterceptor(interceptor);
  }

  addResponseLogger() {
    this.client.addResponseInterceptor((response) => {
      console.log(`[API] ${response.config.method?.toUpperCase()} ${response.config.url} - ${response.status}`);
      return response;
    });
  }
}

export const createApi = (baseURL: string, mockEnabled = false): EscrowApi => {
  const cfg = loadConfig({ baseUrl: normalizeBaseUrl(baseURL), mockEnabled });
  return new EscrowApi({
    baseURL: cfg.baseUrl,
    timeout: cfg.timeoutMs,
    mockEnabled: cfg.mockEnabled,
    retryConfig: {
      maxRetries: cfg.retryMax,
      delayMs: cfg.retryDelayMs,
      backoffMultiplier: cfg.retryBackoffMultiplier,
    },
  });
};

export { ApiClient } from './client';
export { TradesApi, EventsApi, BlockchainApi } from './resources';
export { loadConfig } from './config';
export { API_ENDPOINT_CONTRACTS } from './contracts';
export {
  PerformanceMonitor,
  evaluateThresholds,
  executeScenario,
} from './performance';
export type { Trade, Event } from './models';
export type {
  ApiClientConfig,
  ApiError,
  ErrorInterceptor,
  RequestInterceptor,
  ResponseInterceptor,
  RetryConfig,
} from './types';
export type { ApiConfig };
export {
  ApiConnector,
  IntegrationMonitor,
  IntegrationService,
} from './integration';
export type {
  IntegrationConfig,
  IntegrationEvent,
  IntegrationHealth,
  IntegrationMetrics,
  IntegrationProvider,
  IntegrationStatus,
} from './integration';
export type {
  OperationPerformanceSummary,
  PerformanceAlert,
  PerformanceError,
  PerformanceSample,
  PerformanceScenarioConfig,
  PerformanceSummary,
  PerformanceThresholds,
  ScenarioExecutionContext,
} from './performance';
