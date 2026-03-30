import { ApiClient } from './client';
import { ApiClientConfig } from './types';

// ── Types ─────────────────────────────────────────────────────────────────────

export type IntegrationProvider =
  | 'github'
  | 'slack'
  | 'stripe'
  | 'sendgrid'
  | 'twilio'
  | 'pagerduty'
  | 'custom';

export type IntegrationStatus = 'active' | 'inactive' | 'error';

export interface IntegrationConfig {
  /** Unique integration identifier */
  id: string;
  /** Human-readable name */
  name: string;
  provider: IntegrationProvider;
  /** Base URL of the third-party API */
  baseUrl: string;
  /** Auth token / API key (Bearer header) */
  apiKey?: string;
  /** Additional static headers to send with every request */
  headers?: Record<string, string>;
  /** Request timeout override in ms (default: 15 000) */
  timeoutMs?: number;
  /** Whether this integration is currently active */
  enabled?: boolean;
}

export interface IntegrationHealth {
  integrationId: string;
  status: IntegrationStatus;
  latencyMs: number | null;
  checkedAt: string;
  error?: string;
}

export interface IntegrationMetrics {
  integrationId: string;
  totalRequests: number;
  successCount: number;
  errorCount: number;
  totalLatencyMs: number;
  lastRequestAt: string | null;
}

export interface IntegrationEvent {
  integrationId: string;
  endpoint: string;
  method: string;
  payload?: unknown;
  timestamp: string;
}

// ── Connector ─────────────────────────────────────────────────────────────────

/**
 * ApiConnector wraps an ApiClient for a specific third-party service.
 * It records per-call metrics and exposes typed helpers.
 */
export class ApiConnector {
  private client: ApiClient;
  private metrics: IntegrationMetrics;

  constructor(private config: IntegrationConfig) {
    const headers: Record<string, string> = { ...config.headers };
    if (config.apiKey) {
      headers['Authorization'] = `Bearer ${config.apiKey}`;
    }

    const clientConfig: ApiClientConfig = {
      baseURL: config.baseUrl,
      timeout: config.timeoutMs ?? 15_000,
      retryConfig: { maxRetries: 2, delayMs: 500, backoffMultiplier: 2 },
    };

    this.client = new ApiClient(clientConfig);

    // Attach static headers via request interceptor
    if (Object.keys(headers).length > 0) {
      this.client.addRequestInterceptor((reqConfig) => {
        reqConfig.headers = { ...(reqConfig.headers ?? {}), ...headers };
        return reqConfig;
      });
    }

    this.metrics = {
      integrationId: config.id,
      totalRequests: 0,
      successCount: 0,
      errorCount: 0,
      totalLatencyMs: 0,
      lastRequestAt: null,
    };
  }

  /** Send a GET request to the third-party API. */
  async get<T = unknown>(path: string): Promise<T> {
    return this.track('GET', path, () => this.client.get<T>(path));
  }

  /** Send a POST request to the third-party API. */
  async post<T = unknown>(path: string, body?: unknown): Promise<T> {
    return this.track('POST', path, () => this.client.post<T>(path, body));
  }

  /** Send a PATCH request to the third-party API. */
  async patch<T = unknown>(path: string, body?: unknown): Promise<T> {
    return this.track('PATCH', path, () => this.client.patch<T>(path, body));
  }

  /** Send a DELETE request to the third-party API. */
  async delete<T = unknown>(path: string): Promise<T> {
    return this.track('DELETE', path, () => this.client.delete<T>(path));
  }

  /** Snapshot of call metrics for this connector. */
  getMetrics(): Readonly<IntegrationMetrics> {
    return { ...this.metrics };
  }

  /** Configuration used to build this connector. */
  getConfig(): Readonly<IntegrationConfig> {
    return { ...this.config };
  }

  private async track<T>(method: string, path: string, fn: () => Promise<T>): Promise<T> {
    const start = Date.now();
    this.metrics.totalRequests++;
    this.metrics.lastRequestAt = new Date().toISOString();
    try {
      const result = await fn();
      this.metrics.successCount++;
      this.metrics.totalLatencyMs += Date.now() - start;
      return result;
    } catch (err) {
      this.metrics.errorCount++;
      this.metrics.totalLatencyMs += Date.now() - start;
      throw err;
    }
  }
}

// ── Integration monitor ───────────────────────────────────────────────────────

/**
 * IntegrationMonitor periodically pings a health-check endpoint and records
 * the result.  Call `start()` / `stop()` to manage the polling lifecycle.
 */
export class IntegrationMonitor {
  private healthRecords: Map<string, IntegrationHealth> = new Map();
  private timers: Map<string, ReturnType<typeof setInterval>> = new Map();

  /**
   * Start health-check polling for a connector.
   * @param connector      The connector to check.
   * @param healthPath     Endpoint to ping (default `/health`).
   * @param intervalMs     How often to poll (default 60 000 ms).
   */
  start(
    connector: ApiConnector,
    healthPath = '/health',
    intervalMs = 60_000,
  ): void {
    const id = connector.getConfig().id;
    if (this.timers.has(id)) return; // already polling

    const check = async () => {
      const start = Date.now();
      try {
        await connector.get(healthPath);
        this.healthRecords.set(id, {
          integrationId: id,
          status: 'active',
          latencyMs: Date.now() - start,
          checkedAt: new Date().toISOString(),
        });
      } catch (err: unknown) {
        const message = err instanceof Error ? err.message : String(err);
        this.healthRecords.set(id, {
          integrationId: id,
          status: 'error',
          latencyMs: Date.now() - start,
          checkedAt: new Date().toISOString(),
          error: message,
        });
      }
    };

    void check();
    this.timers.set(id, setInterval(check, intervalMs));
  }

  /** Stop polling for a specific connector. */
  stop(connectorId: string): void {
    const timer = this.timers.get(connectorId);
    if (timer !== undefined) {
      clearInterval(timer);
      this.timers.delete(connectorId);
    }
  }

  /** Stop all polling timers. */
  stopAll(): void {
    for (const id of this.timers.keys()) {
      this.stop(id);
    }
  }

  /** Get the latest health record for an integration. */
  getHealth(connectorId: string): IntegrationHealth | undefined {
    return this.healthRecords.get(connectorId);
  }

  /** Get all recorded health statuses. */
  getAllHealth(): IntegrationHealth[] {
    return Array.from(this.healthRecords.values());
  }
}

// ── Integration service ───────────────────────────────────────────────────────

/**
 * IntegrationService is the central registry for third-party API connectors.
 *
 * Usage:
 * ```ts
 * const svc = new IntegrationService();
 * svc.register({ id: 'gh', name: 'GitHub', provider: 'github', baseUrl: 'https://api.github.com', apiKey: token });
 * const repos = await svc.connector('gh').get<Repo[]>('/user/repos');
 * ```
 */
export class IntegrationService {
  private connectors: Map<string, ApiConnector> = new Map();
  private eventLog: IntegrationEvent[] = [];
  readonly monitor = new IntegrationMonitor();

  /**
   * Register a new integration.
   * @throws if an integration with the same `id` already exists.
   */
  register(config: IntegrationConfig): ApiConnector {
    if (this.connectors.has(config.id)) {
      throw new Error(`Integration "${config.id}" is already registered`);
    }

    const enabled = config.enabled !== false;
    if (!enabled) {
      throw new Error(`Integration "${config.id}" is disabled`);
    }

    const connector = new ApiConnector(config);
    this.connectors.set(config.id, connector);
    this.log({ integrationId: config.id, endpoint: '(registered)', method: 'REGISTER', timestamp: new Date().toISOString() });
    return connector;
  }

  /**
   * Returns the connector for an existing integration.
   * @throws if the integration is not registered.
   */
  connector(id: string): ApiConnector {
    const c = this.connectors.get(id);
    if (!c) throw new Error(`Integration "${id}" is not registered`);
    return c;
  }

  /** Returns true if an integration with the given id is registered. */
  has(id: string): boolean {
    return this.connectors.has(id);
  }

  /** Deregister an integration and stop its health monitoring. */
  deregister(id: string): void {
    this.monitor.stop(id);
    this.connectors.delete(id);
  }

  /** Deregister all integrations. */
  clear(): void {
    this.monitor.stopAll();
    this.connectors.clear();
  }

  /** Aggregate metrics for all registered integrations. */
  getMetrics(): IntegrationMetrics[] {
    return Array.from(this.connectors.values()).map((c) => c.getMetrics());
  }

  /** Recent integration event log (up to last 500 entries). */
  getEventLog(): IntegrationEvent[] {
    return [...this.eventLog];
  }

  private log(event: IntegrationEvent): void {
    this.eventLog.push(event);
    if (this.eventLog.length > 500) {
      this.eventLog.shift();
    }
  }
}
