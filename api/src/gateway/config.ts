/**
 * Gateway configuration loader.
 * All sensitive values come from environment variables.
 */

import { GatewayConfig, UpstreamConfig } from './types';

function requireEnv(name: string): string {
  const val = process.env[name];
  if (!val) throw new Error(`Missing required environment variable: ${name}`);
  return val;
}

function parseKeys(raw: string): string[] {
  return raw.split(',').map((k) => k.trim()).filter(Boolean);
}

function parseUpstreams(): UpstreamConfig[] {
  // Default upstreams matching the docker-compose topology
  const defaults: UpstreamConfig[] = [
    {
      name: 'indexer',
      url: process.env.INDEXER_URL ?? 'http://indexer:3000',
      pathPrefix: '/events',
      healthPath: '/health',
    },
    {
      name: 'indexer-search',
      url: process.env.INDEXER_URL ?? 'http://indexer:3000',
      pathPrefix: '/search',
      healthPath: '/health',
    },
    {
      name: 'indexer-audit',
      url: process.env.INDEXER_URL ?? 'http://indexer:3000',
      pathPrefix: '/audit',
      healthPath: '/health',
    },
    {
      name: 'indexer-ws',
      url: process.env.INDEXER_URL ?? 'http://indexer:3000',
      pathPrefix: '/ws',
      healthPath: '/health',
    },
  ];

  // Allow additional instances via GATEWAY_INSTANCES=name:url:prefix,...
  const extra = process.env.GATEWAY_INSTANCES;
  if (extra) {
    for (const entry of extra.split(',')) {
      const [name, url, pathPrefix] = entry.trim().split(':');
      if (name && url && pathPrefix) {
        defaults.push({ name, url: `${url}`, pathPrefix });
      }
    }
  }

  return defaults;
}

export function loadGatewayConfig(): GatewayConfig {
  const apiKeysRaw = process.env.API_KEYS ?? '';
  const adminKeysRaw = process.env.ADMIN_KEYS ?? '';

  if (!apiKeysRaw && !adminKeysRaw) {
    throw new Error('At least one of API_KEYS or ADMIN_KEYS must be set');
  }

  return {
    port: Number(process.env.GATEWAY_PORT ?? process.env.PORT ?? 4000),
    apiKeys: parseKeys(apiKeysRaw),
    adminKeys: parseKeys(adminKeysRaw),
    jwtSecret: process.env.JWT_SECRET,
    upstreams: parseUpstreams(),
    rateLimit: {
      windowMs: Number(process.env.RATE_LIMIT_WINDOW_MS ?? 60_000),
      maxRequests: Number(process.env.RATE_LIMIT_MAX ?? 60),
    },
  };
}
