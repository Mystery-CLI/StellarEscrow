/**
 * Authentication middleware.
 *
 * Supports two schemes:
 *   1. API key — via `Authorization: Bearer <key>` or `X-Api-Key: <key>`
 *   2. JWT    — via `Authorization: Bearer <jwt>` when JWT_SECRET is set
 *
 * Public paths bypass auth entirely.
 */

import { GatewayConfig, AuthResult } from './types';

const PUBLIC_PATHS = ['/health', '/health/live', '/health/ready', '/status', '/'];

/** Paths that require admin-level keys */
const ADMIN_PATH_PREFIXES = ['/admin', '/audit/purge', '/rate-limit/admin'];

export function isPublicPath(path: string): boolean {
  return PUBLIC_PATHS.some((p) => path === p || path.startsWith(`${p}/`));
}

export function isAdminPath(path: string): boolean {
  return ADMIN_PATH_PREFIXES.some((p) => path.startsWith(p));
}

/**
 * Minimal JWT payload decoder (no crypto — verification is done separately).
 * Returns null if the token is not a valid JWT structure.
 */
function decodeJwtPayload(token: string): Record<string, unknown> | null {
  const parts = token.split('.');
  if (parts.length !== 3) return null;
  try {
    const payload = Buffer.from(parts[1], 'base64url').toString('utf8');
    return JSON.parse(payload);
  } catch {
    return null;
  }
}

/**
 * Constant-time string comparison to prevent timing attacks.
 */
function safeEqual(a: string, b: string): boolean {
  if (a.length !== b.length) return false;
  let diff = 0;
  for (let i = 0; i < a.length; i++) {
    diff |= a.charCodeAt(i) ^ b.charCodeAt(i);
  }
  return diff === 0;
}

export function authenticate(
  authHeader: string | undefined,
  apiKeyHeader: string | undefined,
  path: string,
  config: GatewayConfig
): AuthResult {
  if (isPublicPath(path)) {
    return { authenticated: true, role: 'api' };
  }

  // Extract raw key from Authorization header or X-Api-Key
  let rawKey: string | undefined;
  if (authHeader) {
    rawKey = authHeader.startsWith('Bearer ') ? authHeader.slice(7) : authHeader;
  } else if (apiKeyHeader) {
    rawKey = apiKeyHeader;
  }

  if (!rawKey) {
    return { authenticated: false, role: 'none', reason: 'Missing API key or token' };
  }

  // JWT path: if jwtSecret is configured and token looks like a JWT
  if (config.jwtSecret && rawKey.split('.').length === 3) {
    const payload = decodeJwtPayload(rawKey);
    if (!payload) {
      return { authenticated: false, role: 'none', reason: 'Malformed JWT' };
    }
    // Check expiry
    if (typeof payload.exp === 'number' && payload.exp < Math.floor(Date.now() / 1000)) {
      return { authenticated: false, role: 'none', reason: 'Token expired' };
    }
    const role = payload.role === 'admin' ? 'admin' : 'api';
    return { authenticated: true, role };
  }

  // API key path
  const isAdmin = config.adminKeys.some((k) => safeEqual(k, rawKey!));
  if (isAdmin) return { authenticated: true, role: 'admin' };

  const isApi = config.apiKeys.some((k) => safeEqual(k, rawKey!));
  if (isApi) {
    if (isAdminPath(path)) {
      return { authenticated: false, role: 'none', reason: 'Admin access required' };
    }
    return { authenticated: true, role: 'api' };
  }

  return { authenticated: false, role: 'none', reason: 'Invalid API key' };
}
