/**
 * API Gateway — shared types.
 */

export interface GatewayConfig {
  port: number;
  /** Comma-separated API keys */
  apiKeys: string[];
  /** Comma-separated admin keys */
  adminKeys: string[];
  /** JWT secret for Bearer token validation (optional) */
  jwtSecret?: string;
  /** Upstream service instances for load balancing */
  upstreams: UpstreamConfig[];
  rateLimit: RateLimitConfig;
}

export interface UpstreamConfig {
  name: string;
  /** Base URL, e.g. http://indexer:3000 */
  url: string;
  /** URL path prefix this upstream handles, e.g. /events */
  pathPrefix: string;
  healthPath?: string;
}

export interface RateLimitConfig {
  /** Requests per window per IP */
  windowMs: number;
  maxRequests: number;
}

export interface GatewayRequest {
  method: string;
  path: string;
  headers: Record<string, string>;
  body?: unknown;
  /** Resolved from x-forwarded-for or socket */
  clientIp: string;
}

export interface GatewayResponse {
  status: number;
  headers: Record<string, string>;
  body: unknown;
  /** Upstream that served the request */
  upstream?: string;
  latencyMs: number;
}

export interface AuthResult {
  authenticated: boolean;
  role: 'admin' | 'api' | 'none';
  reason?: string;
}
