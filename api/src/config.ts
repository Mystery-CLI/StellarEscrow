/**
 * Centralized config loader for the API client.
 *
 * Resolution order (highest wins):
 *   1. Explicit options passed to `loadConfig()`
 *   2. Environment variables (STELLAR_ESCROW_*)
 *   3. Compiled-in defaults
 */

export interface ApiConfig {
  baseUrl: string;
  wsUrl: string;
  timeoutMs: number;
  retryMax: number;
  retryDelayMs: number;
  retryBackoffMultiplier: number;
  mockEnabled: boolean;
}

const DEFAULTS: ApiConfig = {
  baseUrl: 'http://localhost:3000/api',
  wsUrl: 'ws://localhost:3000/api/ws',
  timeoutMs: 30_000,
  retryMax: 3,
  retryDelayMs: 1_000,
  retryBackoffMultiplier: 2,
  mockEnabled: false,
};

function fromEnv(): Partial<ApiConfig> {
  // Works in Node.js (process.env) and bundlers that replace process.env at build time
  const env = typeof process !== 'undefined' ? process.env : {};
  const partial: Partial<ApiConfig> = {};

  if (env.STELLAR_ESCROW_API_BASE_URL) partial.baseUrl = env.STELLAR_ESCROW_API_BASE_URL;
  if (env.STELLAR_ESCROW_WS_URL)       partial.wsUrl = env.STELLAR_ESCROW_WS_URL;
  if (env.STELLAR_ESCROW_TIMEOUT_MS)   partial.timeoutMs = Number(env.STELLAR_ESCROW_TIMEOUT_MS);
  if (env.STELLAR_ESCROW_RETRY_MAX)    partial.retryMax = Number(env.STELLAR_ESCROW_RETRY_MAX);
  if (env.STELLAR_ESCROW_MOCK === 'true') partial.mockEnabled = true;

  return partial;
}

function validate(config: ApiConfig): void {
  const errors: string[] = [];

  if (!config.baseUrl) errors.push('baseUrl must not be empty');
  if (config.timeoutMs < 1000) errors.push('timeoutMs must be >= 1000');
  if (config.retryMax < 0) errors.push('retryMax must be >= 0');
  if (config.retryDelayMs < 0) errors.push('retryDelayMs must be >= 0');
  if (config.retryBackoffMultiplier < 1) errors.push('retryBackoffMultiplier must be >= 1');

  if (errors.length > 0) {
    throw new Error(`API config validation failed:\n  - ${errors.join('\n  - ')}`);
  }
}

/**
 * Load and validate the API config.
 * Pass explicit overrides to take precedence over env vars and defaults.
 */
export function loadConfig(overrides: Partial<ApiConfig> = {}): ApiConfig {
  const config: ApiConfig = { ...DEFAULTS, ...fromEnv(), ...overrides };
  validate(config);
  return config;
}
