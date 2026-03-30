/**
 * Core API Gateway.
 *
 * Wires together: auth → rate limiting → load balancing → upstream proxy
 * → response transformation.
 *
 * Designed to run as a standalone Node.js HTTP server (no Express dependency)
 * or to be embedded in an existing Express/Fastify app via `handleRequest`.
 */

import http from 'node:http';
import https from 'node:https';
import { URL } from 'node:url';

import { GatewayConfig, GatewayRequest, GatewayResponse } from './types';
import { authenticate, isPublicPath } from './auth';
import { LoadBalancer } from './loadBalancer';
import { RateLimiter } from './rateLimiter';
import { transformRequest, transformResponse } from './transform';

export class ApiGateway {
  private lb = new LoadBalancer();
  private rl: RateLimiter;
  private server?: http.Server;

  constructor(private config: GatewayConfig) {
    this.rl = new RateLimiter(config.rateLimit);
    // Evict stale rate-limit windows every minute
    setInterval(() => this.rl.evict(), 60_000).unref();
  }

  // ---------------------------------------------------------------------------
  // Core request handler
  // ---------------------------------------------------------------------------

  async handleRequest(req: GatewayRequest): Promise<GatewayResponse> {
    const start = Date.now();

    // Special case: metrics endpoint
    if (req.path === '/metrics') {
      return this.handleMetricsRequest();
    }

    // 1. Authentication
    const authResult = authenticate(
      req.headers['authorization'],
      req.headers['x-api-key'],
      req.path,
      this.config
    );

    if (!authResult.authenticated) {
      return {
        status: 401,
        headers: { 'content-type': 'application/json' },
        body: { error: 'Unauthorized', reason: authResult.reason },
        latencyMs: Date.now() - start,
      };
    }

    // 2. Rate limiting (skip for admin keys)
    if (authResult.role !== 'admin' && !isPublicPath(req.path)) {
      const rl = this.rl.check(req.clientIp);
      if (!rl.allowed) {
        return {
          status: 429,
          headers: {
            'content-type': 'application/json',
            'retry-after': String(Math.ceil(rl.resetMs / 1000)),
            'x-ratelimit-limit': String(this.config.rateLimit.maxRequests),
            'x-ratelimit-remaining': '0',
          },
          body: { error: 'Too Many Requests' },
          latencyMs: Date.now() - start,
        };
      }
    }

    // 3. Load balancing — select upstream
    const upstream = this.lb.select(req.path, this.config.upstreams);
    if (!upstream) {
      return {
        status: 502,
        headers: { 'content-type': 'application/json' },
        body: { error: 'No upstream available for path', path: req.path },
        latencyMs: Date.now() - start,
      };
    }

    // 4. Request transformation
    const transformedBody = transformRequest(req.path, req.body);

    // 5. Proxy to upstream
    let upstreamStatus: number;
    let upstreamBody: unknown;

    try {
      const result = await this.proxyRequest(upstream.url, req, transformedBody);
      upstreamStatus = result.status;
      upstreamBody = result.body;
    } catch (err: any) {
      return {
        status: 502,
        headers: { 'content-type': 'application/json' },
        body: { error: 'Upstream unavailable', upstream: upstream.name, detail: err.message },
        latencyMs: Date.now() - start,
      };
    }

    const latencyMs = Date.now() - start;

    // 6. Response transformation
    const responseBody = transformResponse(upstreamBody, upstreamStatus, upstream.name, latencyMs);

    return {
      status: upstreamStatus,
      headers: { 'content-type': 'application/json' },
      body: responseBody,
      upstream: upstream.name,
      latencyMs,
    };
  }

  // ---------------------------------------------------------------------------
  // Metrics endpoint handler
  // ---------------------------------------------------------------------------

  private handleMetricsRequest(): GatewayResponse {
    // In a real implementation, this would collect metrics from a global metrics store
    // For now, return a basic metrics response
    const metrics = `# HELP stellar_escrow_gateway_up Gateway is up and running
# TYPE stellar_escrow_gateway_up gauge
stellar_escrow_gateway_up 1

# HELP stellar_escrow_gateway_requests_total Total number of requests processed
# TYPE stellar_escrow_gateway_requests_total counter
stellar_escrow_gateway_requests_total 0
`;

    return {
      status: 200,
      headers: {
        'content-type': 'text/plain; charset=utf-8',
        'cache-control': 'no-cache',
      },
      body: metrics,
      latencyMs: 0,
    };
  }

  // ---------------------------------------------------------------------------
  // HTTP proxy
  // ---------------------------------------------------------------------------

  private proxyRequest(
    baseUrl: string,
    req: GatewayRequest,
    body: unknown
  ): Promise<{ status: number; body: unknown }> {
    return new Promise((resolve, reject) => {
      const target = new URL(req.path, baseUrl);
      const isHttps = target.protocol === 'https:';
      const transport = isHttps ? https : http;

      const bodyStr = body ? JSON.stringify(body) : undefined;

      const options: http.RequestOptions = {
        hostname: target.hostname,
        port: target.port || (isHttps ? 443 : 80),
        path: target.pathname + target.search,
        method: req.method,
        headers: {
          ...req.headers,
          host: target.host,
          'content-type': 'application/json',
          ...(bodyStr ? { 'content-length': Buffer.byteLength(bodyStr).toString() } : {}),
        },
        timeout: 30_000,
      };

      const proxyReq = transport.request(options, (res) => {
        let raw = '';
        res.setEncoding('utf8');
        res.on('data', (chunk) => (raw += chunk));
        res.on('end', () => {
          let parsed: unknown;
          try {
            parsed = JSON.parse(raw);
          } catch {
            parsed = raw;
          }
          resolve({ status: res.statusCode ?? 502, body: parsed });
        });
      });

      proxyReq.on('error', reject);
      proxyReq.on('timeout', () => {
        proxyReq.destroy();
        reject(new Error('Upstream request timed out'));
      });

      if (bodyStr) proxyReq.write(bodyStr);
      proxyReq.end();
    });
  }

  // ---------------------------------------------------------------------------
  // Standalone HTTP server
  // ---------------------------------------------------------------------------

  listen(port = this.config.port): http.Server {
    this.server = http.createServer(async (req, res) => {
      const clientIp =
        (req.headers['x-forwarded-for'] as string)?.split(',')[0].trim() ??
        req.socket.remoteAddress ??
        '0.0.0.0';

      let body: unknown;
      if (req.method !== 'GET' && req.method !== 'HEAD') {
        body = await readBody(req);
      }

      const gatewayReq: GatewayRequest = {
        method: req.method ?? 'GET',
        path: req.url ?? '/',
        headers: req.headers as Record<string, string>,
        body,
        clientIp,
      };

      const response = await this.handleRequest(gatewayReq);

      res.writeHead(response.status, response.headers);
      res.end(JSON.stringify(response.body));
    });

    this.server.listen(port, () => {
      console.log(`[gateway] Listening on port ${port}`);
    });

    return this.server;
  }

  close(): void {
    this.server?.close();
  }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function readBody(req: http.IncomingMessage): Promise<unknown> {
  return new Promise((resolve) => {
    let raw = '';
    req.setEncoding('utf8');
    req.on('data', (chunk) => (raw += chunk));
    req.on('end', () => {
      try {
        resolve(JSON.parse(raw));
      } catch {
        resolve(raw || undefined);
      }
    });
    req.on('error', () => resolve(undefined));
  });
}
